//! kpkg - Kyanite package manager.
//! Interfaz estilo apk (add/del/update/upgrade/search/info/list/...) sobre repositorios Debian;
//! si se invoca como `apt`/`apt-get` tambien ofrece la sintaxis y la ayuda de apt.
//! Ligero: sin scripts de mantenedor (hooks declarativos), indices firmados (OpenPGP),
//! descargas verificadas por SHA256, extraccion segura y actualizacion del sistema base.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fs;
use std::convert::TryInto;
use std::io::{self, IsTerminal, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use tar::EntryType;
use xz2::read::XzDecoder;

use ed25519_dalek::{Signature, VerifyingKey};
use rsa::traits::PublicKeyParts;
use rsa::{BigUint, Pkcs1v15Sign, RsaPublicKey};

type Res<T> = Result<T, Box<dyn Error>>;

const VERSION: &str = "0.3.0";
const DEFAULT_MIRROR: &str = "https://deb.debian.org/debian";
const DEFAULT_SUITE: &str = "stable";
const COMPONENT: &str = "main";
const DEFAULT_SECURITY_MIRROR: &str = "https://deb.debian.org/debian-security";
const KYANITE_REPO: &str = "StringManolo/Kyanite";
const GITHUB: &str = "https://github.com";
const MAX_SMALL: u64 = 64 * 1024;

/// Limites anti "bombas" (descompresion / respuestas infinitas).
const MAX_ENTRY_SIZE: u64 = 256 * 1024 * 1024;
const MAX_TOTAL_SIZE: u64 = 1024 * 1024 * 1024;
const MAX_RELEASE: u64 = 4 * 1024 * 1024;
const MAX_INDEX_XZ: u64 = 128 * 1024 * 1024;
const MAX_INDEX_TEXT: u64 = 512 * 1024 * 1024;
const MAX_DEB: u64 = 1024 * 1024 * 1024;

// ============================================================
// CONTEXTO
// ============================================================

#[derive(Clone)]
struct Ctx {
    root: PathBuf,
    arch: String,
    mirror: String,
    security_mirror: String,
    suite: String,
    keyring: Option<PathBuf>,
    simulate: bool,
    force: bool,
    yes: bool,
    apt: bool,
    no_security: bool,
}

impl Ctx {
    fn db_dir(&self) -> PathBuf {
        self.root.join("var/lib/kpkg")
    }
    fn pkgs_dir(&self) -> PathBuf {
        self.db_dir().join("pkgs")
    }
    fn index_path(&self) -> PathBuf {
        self.db_dir().join("index.tsv")
    }
    fn meta_path(&self) -> PathBuf {
        self.db_dir().join("index.meta")
    }
    fn cache_pkgs(&self) -> PathBuf {
        self.root.join("var/cache/kpkg/pkgs")
    }
    fn cache_base(&self) -> PathBuf {
        self.root.join("var/cache/kpkg/base")
    }
    fn manual_path(&self) -> PathBuf {
        self.db_dir().join("manual")
    }
    fn base_pending_path(&self) -> PathBuf {
        self.db_dir().join("base-update")
    }
    fn staging_dir(&self) -> PathBuf {
        self.db_dir().join("staging")
    }
}

fn default_arch() -> String {
    if cfg!(target_arch = "aarch64") {
        "arm64".to_string()
    } else {
        "amd64".to_string()
    }
}

// ============================================================
// UTILIDADES
// ============================================================

fn human(n: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB"];
    let mut v = n as f64;
    let mut u = 0usize;
    while v >= 1024.0 && u < units.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{} B", n)
    } else {
        format!("{:.1} {}", v, units[u])
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

fn sha256_file(path: &Path) -> Res<String> {
    let mut f = fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

fn write_atomic(path: &Path, data: &[u8]) -> Res<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut tmp_os = path.as_os_str().to_owned();
    tmp_os.push(".tmp");
    let tmp = PathBuf::from(tmp_os);
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

fn valid_pkg_name(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && b.len() <= 100
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'+' || *c == b'-' || *c == b'.')
}

fn valid_filename(s: &str) -> bool {
    s.starts_with("pool/")
        && !s.contains("..")
        && !s.contains("//")
        && s.bytes().all(|c| {
            c.is_ascii_alphanumeric() || c == b'.' || c == b'_' || c == b'+' || c == b'~' || c == b'-' || c == b'/'
        })
}

fn valid_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn valid_version(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'+' || c == b'~' || c == b':' || c == b'-')
}

fn valid_suite(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'.')
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Dias desde 1970-01-01 (algoritmo de Howard Hinnant, probado contra calendar.timegm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// "Sat, 18 Oct 2025 08:19:01 UTC" -> segundos unix (Debian publica siempre en UTC).
fn parse_http_date(s: &str) -> Option<i64> {
    let s = s.trim();
    let s = match s.split_once(", ") {
        Some((_, rest)) => rest,
        None => s,
    };
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }
    let day: i64 = parts[0].parse().ok()?;
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let mon = months.iter().position(|m| *m == parts[1])? as i64 + 1;
    let year: i64 = parts[2].parse().ok()?;
    let hms: Vec<&str> = parts[3].split(':').collect();
    if hms.len() != 3 {
        return None;
    }
    let h: i64 = hms[0].parse().ok()?;
    let mi: i64 = hms[1].parse().ok()?;
    let se: i64 = hms[2].parse().ok()?;
    Some(days_from_civil(year, mon, day) * 86400 + h * 3600 + mi * 60 + se)
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit())
}

/// Solo versiones de release estrictas: "0.2.0" o "v0.2.0". Cualquier otra cosa
/// ("v0.1.0-3-g1a2b3c4", "1a2b3c4") es una build de desarrollo y NO se actualiza.
fn parse_release_version(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.trim();
    let s = s.strip_prefix('v').unwrap_or(s);
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 || !parts.iter().all(|p| all_digits(p)) {
        return None;
    }
    Some((parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?))
}

// ============================================================
// HTTP
// ============================================================

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(60))
        .build()
}

/// Descarga a memoria con tope de tamano.
fn http_get(url: &str, limit: u64) -> Res<Vec<u8>> {
    let resp = agent().get(url).call()?;
    let mut buf: Vec<u8> = Vec::new();
    let n = resp.into_reader().take(limit + 1).read_to_end(&mut buf)?;
    if n as u64 > limit {
        return Err(format!("response too large (> {} bytes): {}", limit, url).into());
    }
    Ok(buf)
}

/// Descarga en streaming a un fichero, verificando tamano y SHA256 antes de dejarlo en su sitio.
fn download_verified(url: &str, path: &Path, want_size: u64, want_sha: &str) -> Res<()> {
    let limit = if want_size > 0 { want_size } else { MAX_DEB };
    let resp = agent().get(url).call()?;
    let mut reader = resp.into_reader().take(limit + 1);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut tmp_os = path.as_os_str().to_owned();
    tmp_os.push(".part");
    let tmp = PathBuf::from(tmp_os);
    let mut f = fs::File::create(&tmp)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 65536];
    let mut total: u64 = 0;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > limit {
            drop(f);
            let _ = fs::remove_file(&tmp);
            return Err("download is larger than the size announced by the index".into());
        }
        hasher.update(&buf[..n]);
        f.write_all(&buf[..n])?;
    }
    drop(f);
    if want_size > 0 && total != want_size {
        let _ = fs::remove_file(&tmp);
        return Err(format!("size mismatch: index says {} bytes, got {}", want_size, total).into());
    }
    let got = hex::encode(hasher.finalize());
    if got != want_sha {
        let _ = fs::remove_file(&tmp);
        return Err(format!(
            "SECURITY ALERT: SHA256 mismatch!\n  expected:   {}\n  calculated: {}",
            want_sha, got
        )
        .into());
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

// ============================================================
// VERSIONES DEBIAN (algoritmo de dpkg; probado contra dpkg --compare-versions)
// ============================================================

fn split_version(v: &str) -> (u64, &str, &str) {
    let (epoch, rest) = match v.find(':') {
        Some(i) => (v[..i].parse::<u64>().unwrap_or(0), &v[i + 1..]),
        None => (0, v),
    };
    match rest.rfind('-') {
        Some(i) => (epoch, &rest[..i], &rest[i + 1..]),
        None => (epoch, rest, ""),
    }
}

fn order(c: u8) -> i32 {
    if c.is_ascii_digit() {
        0
    } else if c.is_ascii_alphabetic() {
        c as i32
    } else if c == b'~' {
        -1
    } else {
        c as i32 + 256
    }
}

fn verrevcmp(a: &str, b: &str) -> Ordering {
    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut i = 0usize;
    let mut j = 0usize;
    while i < a.len() || j < b.len() {
        while (i < a.len() && !a[i].is_ascii_digit()) || (j < b.len() && !b[j].is_ascii_digit()) {
            let ac = if i < a.len() { order(a[i]) } else { 0 };
            let bc = if j < b.len() { order(b[j]) } else { 0 };
            if ac != bc {
                return ac.cmp(&bc);
            }
            i += 1;
            j += 1;
        }
        while i < a.len() && a[i] == b'0' {
            i += 1;
        }
        while j < b.len() && b[j] == b'0' {
            j += 1;
        }
        let mut first_diff = 0i32;
        while i < a.len() && a[i].is_ascii_digit() && j < b.len() && b[j].is_ascii_digit() {
            if first_diff == 0 {
                first_diff = a[i] as i32 - b[j] as i32;
            }
            i += 1;
            j += 1;
        }
        if i < a.len() && a[i].is_ascii_digit() {
            return Ordering::Greater;
        }
        if j < b.len() && b[j].is_ascii_digit() {
            return Ordering::Less;
        }
        if first_diff != 0 {
            return first_diff.cmp(&0);
        }
    }
    Ordering::Equal
}

fn cmp_version(a: &str, b: &str) -> Ordering {
    let (ea, ua, ra) = split_version(a);
    let (eb, ub, rb) = split_version(b);
    match ea.cmp(&eb) {
        Ordering::Equal => {}
        o => return o,
    }
    match verrevcmp(ua, ub) {
        Ordering::Equal => {}
        o => return o,
    }
    verrevcmp(ra, rb)
}

fn ver_satisfies(have: &str, op: &str, want: &str) -> bool {
    let c = cmp_version(have, want);
    match op {
        "<<" => c == Ordering::Less,
        "<=" => c != Ordering::Greater,
        "=" => c == Ordering::Equal,
        ">=" => c != Ordering::Less,
        ">>" => c == Ordering::Greater,
        _ => false,
    }
}

// ============================================================
// PAQUETES, INDICE Y DEPENDENCIAS
// ============================================================

#[derive(Clone, Default)]
struct Pkg {
    name: String,
    version: String,
    arch: String,
    depends: String,
    pre_depends: String,
    provides: String,
    conflicts: String,
    breaks: String,
    filename: String,
    sha256: String,
    size: u64,
    installed_size: u64, // KiB, como en Debian
    desc: String,
    origin: String, // mirror del que viene (vacio = ctx.mirror); stable-security usa otro
}

fn clean_field(s: &str) -> String {
    s.replace(|c: char| c == '\t' || c == '\n' || c == '\r', " ")
}

impl Pkg {
    fn to_line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            clean_field(&self.name),
            clean_field(&self.version),
            clean_field(&self.arch),
            clean_field(&self.depends),
            clean_field(&self.pre_depends),
            clean_field(&self.provides),
            clean_field(&self.conflicts),
            clean_field(&self.breaks),
            clean_field(&self.filename),
            clean_field(&self.sha256),
            self.size,
            self.installed_size,
            clean_field(&self.desc),
            clean_field(&self.origin)
        )
    }

    fn from_line(line: &str) -> Option<Pkg> {
        let f: Vec<&str> = line.splitn(14, '\t').collect();
        if f.len() < 13 {
            return None;
        }
        Some(Pkg {
            name: f[0].to_string(),
            version: f[1].to_string(),
            arch: f[2].to_string(),
            depends: f[3].to_string(),
            pre_depends: f[4].to_string(),
            provides: f[5].to_string(),
            conflicts: f[6].to_string(),
            breaks: f[7].to_string(),
            filename: f[8].to_string(),
            sha256: f[9].to_string(),
            size: f[10].parse().unwrap_or(0),
            installed_size: f[11].parse().unwrap_or(0),
            desc: f[12].to_string(),
            origin: f.get(13).map(|s| s.to_string()).unwrap_or_default(),
        })
    }
}

fn parse_packages(text: &str, out: &mut Vec<Pkg>) {
    for block in text.split("\n\n") {
        let mut p = Pkg::default();
        let mut have_name = false;
        for line in block.lines() {
            if line.starts_with(' ') || line.starts_with('\t') {
                continue; // lineas de continuacion (solo nos importa la 1a de Description)
            }
            let (k, v) = match line.split_once(':') {
                Some((k, v)) => (k, v.trim()),
                None => continue,
            };
            match k {
                "Package" => {
                    p.name = v.to_string();
                    have_name = true;
                }
                "Version" => p.version = v.to_string(),
                "Architecture" => p.arch = v.to_string(),
                "Depends" => p.depends = v.to_string(),
                "Pre-Depends" => p.pre_depends = v.to_string(),
                "Provides" => p.provides = v.to_string(),
                "Conflicts" => p.conflicts = v.to_string(),
                "Breaks" => p.breaks = v.to_string(),
                "Filename" => p.filename = v.to_string(),
                "SHA256" => p.sha256 = v.to_lowercase(),
                "Size" => p.size = v.parse().unwrap_or(0),
                "Installed-Size" => p.installed_size = v.parse().unwrap_or(0),
                "Description" => p.desc = v.to_string(),
                _ => {}
            }
        }
        if have_name && !p.version.is_empty() && !p.filename.is_empty() && !p.sha256.is_empty() {
            out.push(p);
        }
    }
}

#[derive(Clone)]
struct Dep {
    name: String,
    op: Option<String>,
    ver: Option<String>,
}

fn dep_to_string(d: &Dep) -> String {
    match (&d.op, &d.ver) {
        (Some(o), Some(v)) => format!("{} ({} {})", d.name, o, v),
        _ => d.name.clone(),
    }
}

fn parse_dep(s: &str) -> Option<Dep> {
    let mut op: Option<String> = None;
    let mut ver: Option<String> = None;
    let head: &str;
    if let Some(open) = s.find('(') {
        head = &s[..open];
        if let Some(close_rel) = s[open..].find(')') {
            let inner = s[open + 1..open + close_rel].trim();
            let split = inner
                .find(|c: char| c != '<' && c != '>' && c != '=')
                .unwrap_or(inner.len());
            let o = &inner[..split];
            let v = inner[split..].trim();
            let o = match o {
                "<" => "<=",
                ">" => ">=",
                "" => "=",
                other => other,
            };
            if !v.is_empty() {
                op = Some(o.to_string());
                ver = Some(v.to_string());
            }
        }
    } else {
        head = s;
    }
    let head = head.split(|c: char| c == '[' || c == '<').next().unwrap_or("").trim();
    let name = head.split(':').next().unwrap_or("").trim();
    if name.is_empty() {
        return None;
    }
    Some(Dep { name: name.to_string(), op, ver })
}

/// "a (>= 1) | b, c" -> [[a|b], [c]]
fn parse_dep_list(s: &str) -> Vec<Vec<Dep>> {
    let mut out = Vec::new();
    for entry in s.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let mut alts = Vec::new();
        for alt in entry.split('|') {
            if let Some(d) = parse_dep(alt.trim()) {
                alts.push(d);
            }
        }
        if !alts.is_empty() {
            out.push(alts);
        }
    }
    out
}

fn parse_provides(s: &str) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    for item in s.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if let Some(d) = parse_dep(item) {
            let v = match d.op.as_deref() {
                Some("=") => d.ver.clone(),
                _ => None,
            };
            out.push((d.name, v));
        }
    }
    out
}

fn dep_matches_version(d: &Dep, have: Option<&str>) -> bool {
    match (&d.op, &d.ver) {
        (Some(op), Some(want)) => match have {
            Some(h) => ver_satisfies(h, op, want),
            None => false,
        },
        _ => true,
    }
}

/// Un paquete satisface una dependencia por su nombre real o por un Provides.
/// Un Provides sin version no satisface dependencias versionadas (igual que apt).
fn pkg_satisfies(p: &Pkg, d: &Dep) -> bool {
    if p.name == d.name && dep_matches_version(d, Some(p.version.as_str())) {
        return true;
    }
    for (n, v) in parse_provides(&p.provides) {
        if n == d.name && dep_matches_version(d, v.as_deref()) {
            return true;
        }
    }
    false
}

struct Index {
    pkgs: Vec<Pkg>,
    by_name: HashMap<String, usize>,
    providers: HashMap<String, Vec<(usize, Option<String>)>>,
}

impl Index {
    fn build(mut pkgs: Vec<Pkg>) -> Index {
        pkgs.sort_by(|a, b| a.name.cmp(&b.name));
        let mut by_name: HashMap<String, usize> = HashMap::new();
        let mut providers: HashMap<String, Vec<(usize, Option<String>)>> = HashMap::new();
        for (i, p) in pkgs.iter().enumerate() {
            by_name.insert(p.name.clone(), i);
            for (n, v) in parse_provides(&p.provides) {
                providers.entry(n).or_insert_with(Vec::new).push((i, v));
            }
        }
        Index { pkgs, by_name, providers }
    }

    /// Candidatos para una dependencia: primero el paquete con ese nombre, luego los que lo "Provide".
    fn candidates(&self, d: &Dep) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        if let Some(&i) = self.by_name.get(&d.name) {
            if dep_matches_version(d, Some(self.pkgs[i].version.as_str())) {
                out.push(i);
            }
        }
        if let Some(list) = self.providers.get(&d.name) {
            let mut extra: Vec<usize> = Vec::new();
            for (i, v) in list {
                if dep_matches_version(d, v.as_deref()) && !out.contains(i) && !extra.contains(i) {
                    extra.push(*i);
                }
            }
            // entre varios proveedores de un paquete virtual: el mas pequeno, luego por nombre
            extra.sort_by(|a, b| {
                let pa = &self.pkgs[*a];
                let pb = &self.pkgs[*b];
                pa.installed_size.cmp(&pb.installed_size).then(pa.name.cmp(&pb.name))
            });
            out.extend(extra);
        }
        out
    }
}

fn load_index(ctx: &Ctx) -> Res<Index> {
    let text = fs::read_to_string(ctx.index_path())
        .map_err(|_| "package index not found: run 'kpkg update' first")?;
    let mut pkgs = Vec::new();
    for line in text.lines() {
        if let Some(p) = Pkg::from_line(line) {
            pkgs.push(p);
        }
    }
    Ok(Index::build(pkgs))
}

// ============================================================
// BASE DE DATOS DE PAQUETES INSTALADOS
// ============================================================
// <root>/var/lib/kpkg/pkgs/<nombre>/info   -> linea del indice
// <root>/var/lib/kpkg/pkgs/<nombre>/files  -> "<tipo>\t<extra>\t<ruta>"  (f=sha256, l/h=destino, d=directorio)

struct Db {
    pkgs: HashMap<String, Pkg>,
    assumed: Vec<(String, Option<String>)>,
}

#[derive(Clone)]
struct FileRec {
    kind: char,
    path: String,
    extra: String,
}

impl FileRec {
    fn to_line(&self) -> String {
        format!("{}\t{}\t{}", self.kind, self.extra, self.path)
    }
    fn from_line(l: &str) -> Option<FileRec> {
        let mut it = l.splitn(3, '\t');
        let k = it.next()?;
        let extra = it.next()?;
        let path = it.next()?;
        Some(FileRec {
            kind: k.chars().next()?,
            path: path.to_string(),
            extra: extra.to_string(),
        })
    }
}

/// Paquetes que se dan por instalados (p.ej. la libc que ya trae el rootfs).
/// Si existe <root>/etc/kpkg/assume-installed se usa SOLO ese fichero ("nombre" o "nombre=version", una por linea).
/// Si no existe y root es "/", se asumen libc6 y libgcc-s1.
fn load_assumed(ctx: &Ctx) -> Vec<(String, Option<String>)> {
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    match fs::read_to_string(ctx.root.join("etc/kpkg/assume-installed")) {
        Ok(text) => {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                match line.split_once('=') {
                    Some((n, v)) => out.push((n.trim().to_string(), Some(v.trim().to_string()))),
                    None => out.push((line.to_string(), None)),
                }
            }
        }
        Err(_) => {
            if ctx.root.as_path() == Path::new("/") {
                out.push(("libc6".to_string(), None));
                out.push(("libgcc-s1".to_string(), None));
            }
        }
    }
    out
}

fn load_db(ctx: &Ctx) -> Res<Db> {
    let mut pkgs: HashMap<String, Pkg> = HashMap::new();
    let dir = ctx.pkgs_dir();
    if dir.is_dir() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !valid_pkg_name(&name) {
                continue;
            }
            let info = match fs::read_to_string(entry.path().join("info")) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if let Some(p) = Pkg::from_line(info.lines().next().unwrap_or("")) {
                pkgs.insert(p.name.clone(), p);
            }
        }
    }
    Ok(Db { pkgs, assumed: load_assumed(ctx) })
}

fn load_files(ctx: &Ctx, name: &str) -> Vec<FileRec> {
    match fs::read_to_string(ctx.pkgs_dir().join(name).join("files")) {
        Ok(t) => t.lines().filter_map(FileRec::from_line).collect(),
        Err(_) => Vec::new(),
    }
}

fn load_owners(ctx: &Ctx, db: &Db) -> HashMap<String, String> {
    let mut m: HashMap<String, String> = HashMap::new();
    for name in db.pkgs.keys() {
        for f in load_files(ctx, name) {
            if f.kind != 'd' {
                m.insert(f.path, name.clone());
            }
        }
    }
    m
}

struct Lock {
    path: PathBuf,
}

impl Lock {
    fn acquire(ctx: &Ctx) -> Res<Lock> {
        let dir = ctx.db_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join("lock");
        match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut f) => {
                let _ = writeln!(f, "{}", std::process::id());
                Ok(Lock { path })
            }
            Err(e) => Err(format!(
                "cannot take lock {} ({}). Is another kpkg running? If not, delete that file.",
                path.display(),
                e
            )
            .into()),
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

// ============================================================
// UPDATE (InRelease + Packages.xz)
// ============================================================

fn parse_release_hashes(text: &str) -> HashMap<String, (String, u64)> {
    let mut m: HashMap<String, (String, u64)> = HashMap::new();
    let mut in_sha = false;
    for line in text.lines() {
        if line.starts_with("-----BEGIN PGP SIGNATURE") {
            break;
        }
        if line.starts_with("SHA256:") {
            in_sha = true;
            continue;
        }
        if in_sha {
            if line.starts_with(' ') {
                let f: Vec<&str> = line.split_whitespace().collect();
                if f.len() == 3 {
                    if let Ok(sz) = f[1].parse::<u64>() {
                        m.insert(f[2].to_string(), (f[0].to_lowercase(), sz));
                    }
                }
            } else {
                in_sha = false;
            }
        }
    }
    m
}

// ---- Firma OpenPGP (verificador minimo) ---------------------------------------------------
// Cadena de confianza: keyring de Debian (en el rootfs) -> InRelease firmado -> SHA256 de
// Packages.xz -> SHA256 de cada .deb. Si la firma no verifica, NO se usa nada del mirror.
//
// Verificador propio y deliberadamente pequeno: solo claves publicas RSA v4 y firmas v4 RSA de
// texto (tipo 0x01) con SHA-256/384/512, que es lo que Debian usa en InRelease. La operacion
// criptografica (RSA PKCS#1 v1.5) la hace el crate `rsa`; aqui solo se parsean paquetes.
// (Se descarto el crate `pgp`: descartaba en silencio las claves de archivo de bookworm/trixie.)
// No se comprueba la caducidad de las claves: la rotacion se hace actualizando el keyring con
// la propia release de Kyanite.

/// Cabecera de un paquete OpenPGP en la posicion i: (tag, bytes de cabecera, longitud del cuerpo).
fn pgp_header(d: &[u8], i: usize) -> Option<(u8, usize, usize)> {
    let b = *d.get(i)?;
    if b & 0x80 == 0 {
        return None;
    }
    if b & 0x40 != 0 {
        let tag = b & 0x3f;
        let l1 = *d.get(i + 1)? as usize;
        if l1 < 192 {
            Some((tag, 2, l1))
        } else if l1 < 224 {
            let l2 = *d.get(i + 2)? as usize;
            Some((tag, 3, ((l1 - 192) << 8) + l2 + 192))
        } else if l1 == 255 {
            let s = d.get(i + 2..i + 6)?;
            Some((tag, 6, u32::from_be_bytes([s[0], s[1], s[2], s[3]]) as usize))
        } else {
            None
        }
    } else {
        let tag = (b >> 2) & 0x0f;
        match b & 3 {
            0 => Some((tag, 2, *d.get(i + 1)? as usize)),
            1 => {
                let s = d.get(i + 1..i + 3)?;
                Some((tag, 3, u16::from_be_bytes([s[0], s[1]]) as usize))
            }
            2 => {
                let s = d.get(i + 1..i + 5)?;
                Some((tag, 5, u32::from_be_bytes([s[0], s[1], s[2], s[3]]) as usize))
            }
            _ => None,
        }
    }
}

/// Todos los paquetes (tag, cuerpo); se detiene en el primer paquete malformado.
fn pgp_packets(d: &[u8]) -> Vec<(u8, &[u8])> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < d.len() {
        let (tag, h, l) = match pgp_header(d, i) {
            Some(x) => x,
            None => break,
        };
        let end = match (i + h).checked_add(l) {
            Some(e) => e,
            None => break,
        };
        match d.get(i + h..end) {
            Some(body) => out.push((tag, body)),
            None => break,
        }
        i = end;
    }
    out
}

/// MPI OpenPGP en d[i..]: (bytes big-endian, posicion siguiente).
fn pgp_mpi(d: &[u8], i: usize) -> Option<(&[u8], usize)> {
    let hdr = d.get(i..i + 2)?;
    let bits = u16::from_be_bytes([hdr[0], hdr[1]]) as usize;
    let len = (bits + 7) / 8;
    let v = d.get(i + 2..i + 2 + len)?;
    Some((v, i + 2 + len))
}

fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b' ' | b'\t' | b'\r' | b'\n' => continue,
            _ => return None,
        };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1u32 << bits) - 1;
        }
    }
    Some(out)
}

/// Contenido binario de un bloque "armor" (cabeceras, base64, linea de CRC).
fn dearmor(text: &str) -> Option<Vec<u8>> {
    let mut lines = text.lines();
    lines.find(|l| l.starts_with("-----BEGIN PGP"))?;
    for l in lines.by_ref() {
        if l.trim().is_empty() {
            break;
        }
    }
    let mut b64 = String::new();
    for l in lines {
        let l = l.trim();
        if l.starts_with('=') || l.starts_with("-----END") {
            break;
        }
        b64.push_str(l);
    }
    b64_decode(&b64)
}

/// Key ID (8 ultimos bytes de la huella v4), solo para diagnostico.
fn pgp_keyid(body: &[u8]) -> String {
    let mut h = Sha1::new();
    h.update([0x99u8]);
    h.update((body.len() as u16).to_be_bytes());
    h.update(body);
    let fp = h.finalize();
    hex::encode(&fp[12..20])
}

/// Key IDs de emisor declarados en una firma v4 (subpaquetes 16 y 33), solo para diagnostico.
fn sig_issuers(b: &[u8], hashed_end: usize, ul: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let empty: &[u8] = &[];
    let areas = [b.get(6..hashed_end).unwrap_or(empty), b.get(hashed_end + 2..hashed_end + 2 + ul).unwrap_or(empty)];
    for area in areas {
        let mut i = 0usize;
        while i < area.len() {
            let l = area[i] as usize;
            let (len, hdr) = if l < 192 {
                (l, 1)
            } else if l < 255 {
                if i + 1 >= area.len() {
                    break;
                }
                (((l - 192) << 8) + area[i + 1] as usize + 192, 2)
            } else {
                if i + 5 > area.len() {
                    break;
                }
                (u32::from_be_bytes([area[i + 1], area[i + 2], area[i + 3], area[i + 4]]) as usize, 5)
            };
            let end = i + hdr + len;
            if len == 0 || end > area.len() {
                break;
            }
            let ty = area[i + hdr] & 0x7f;
            let data = &area[i + hdr + 1..end];
            if ty == 16 && data.len() == 8 {
                out.push(hex::encode(data));
            }
            if ty == 33 && data.len() == 21 {
                out.push(hex::encode(&data[13..21]));
            }
            i = end;
        }
    }
    out
}

/// Claves publicas RSA v4 de un keyring binario: primarias (tag 6) Y subclaves (tag 14).
/// Las claves de firma de los archivos 12/bookworm y 13/trixie de Debian son subclaves.
fn rsa_keys_from_packets(data: &[u8], keys: &mut Vec<RsaPublicKey>) {
    for (tag, b) in pgp_packets(data) {
        if (tag != 6 && tag != 14) || b.len() < 8 || b[0] != 4 || b[5] != 1 {
            continue;
        }
        if let Some((n, i)) = pgp_mpi(b, 6) {
            if let Some((e, _)) = pgp_mpi(b, i) {
                if env::var("KPKG_DEBUG").is_ok() {
                    println!("[debug] loaded RSA key {} ({} bits)", pgp_keyid(b), n.len() * 8);
                }
                match RsaPublicKey::new(BigUint::from_bytes_be(n), BigUint::from_bytes_be(e)) {
                    Ok(k) => keys.push(k),
                    Err(err) => {
                        if env::var("KPKG_DEBUG").is_ok() {
                            println!("[debug]   rejected by the rsa crate: {}", err);
                        }
                    }
                }
            }
        }
    }
}

/// Carga las claves publicas de confianza. Fuentes (y solo estas):
///   --keyring <fichero>   (si se indica, sustituye a las demas)
///   <root>/usr/share/keyrings/debian-archive-keyring.gpg
///   <root>/etc/kpkg/trusted.gpg.d/*.gpg|*.asc
fn load_keyring(ctx: &Ctx) -> Res<Vec<RsaPublicKey>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    match &ctx.keyring {
        Some(k) => paths.push(k.clone()),
        None => {
            paths.push(ctx.root.join("usr/share/keyrings/debian-archive-keyring.gpg"));
            if let Ok(rd) = fs::read_dir(ctx.root.join("etc/kpkg/trusted.gpg.d")) {
                let mut extra: Vec<PathBuf> = rd
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| matches!(p.extension().and_then(|x| x.to_str()), Some("gpg") | Some("asc")))
                    .collect();
                extra.sort();
                paths.extend(extra);
            }
        }
    }
    let mut keys: Vec<RsaPublicKey> = Vec::new();
    for p in &paths {
        let data = match fs::read(p) {
            Ok(d) => d,
            Err(_) => continue,
        };
        if data.starts_with(b"-----BEGIN PGP") {
            if let Some(bin) = dearmor(&String::from_utf8_lossy(&data)) {
                rsa_keys_from_packets(&bin, &mut keys);
            }
        } else {
            rsa_keys_from_packets(&data, &mut keys);
        }
    }
    if env::var("KPKG_DEBUG").is_ok() {
        println!("[debug] keyring files: {:?}; {} RSA key(s) loaded", paths, keys.len());
    }
    if keys.is_empty() {
        return Err("no trusted OpenPGP (RSA) keys found (looked in usr/share/keyrings/debian-archive-keyring.gpg and \
etc/kpkg/trusted.gpg.d/). Update the Kyanite base, or pass --keyring <file>. \
Refusing to use unsigned repositories."
            .into());
    }
    Ok(keys)
}

fn rsa_verify(k: &RsaPublicKey, hash_algo: u8, data: &[u8], sig: &[u8]) -> bool {
    let size = k.size();
    if sig.len() > size {
        return false;
    }
    let mut padded = vec![0u8; size - sig.len()];
    padded.extend_from_slice(sig);
    match hash_algo {
        8 => k.verify(Pkcs1v15Sign::new::<Sha256>(), &Sha256::digest(data), &padded).is_ok(),
        9 => k.verify(Pkcs1v15Sign::new::<Sha384>(), &Sha384::digest(data), &padded).is_ok(),
        10 => k.verify(Pkcs1v15Sign::new::<Sha512>(), &Sha512::digest(data), &padded).is_ok(),
        _ => false,
    }
}

/// Verifica un InRelease "clearsigned" y devuelve SOLO el texto cubierto por la firma
/// (nunca el texto crudo: un MITM podria anadir contenido sin firmar alrededor).
/// El texto firmado es canonico: sin espacios finales por linea, con CRLF, sin el ultimo salto.
fn verify_inrelease(clearsigned: &str, keys: &[RsaPublicKey]) -> Res<String> {
    let norm = clearsigned.replace("\r\n", "\n");
    let lines: Vec<&str> = norm.split('\n').collect();
    let start = lines
        .iter()
        .position(|l| *l == "-----BEGIN PGP SIGNED MESSAGE-----")
        .ok_or("InRelease is not a clearsigned message")?;
    let mut i = start + 1;
    while i < lines.len() && !lines[i].is_empty() {
        i += 1; // cabeceras armor (Hash: ...)
    }
    let rest = lines.get(i + 1..).ok_or("malformed clearsigned message")?;
    let off = rest
        .iter()
        .position(|l| *l == "-----BEGIN PGP SIGNATURE-----")
        .ok_or("InRelease has no signature block")?;
    let body = &rest[..off];
    let sigdata = dearmor(&rest[off..].join("\n")).ok_or("malformed signature armor")?;

    let mut canon: Vec<u8> = Vec::new();
    let mut text = String::new();
    for (n, l) in body.iter().copied().enumerate() {
        let l: &str = l.strip_prefix("- ").unwrap_or(l);
        let l: &str = l.trim_end_matches(|c: char| c == ' ' || c == '\t' || c == '\r');
        if n > 0 {
            canon.extend_from_slice(b"\r\n");
        }
        canon.extend_from_slice(l.as_bytes());
        text.push_str(l);
        text.push('\n');
    }

    let mut seen = 0usize;
    for (tag, b) in pgp_packets(&sigdata) {
        // firma v4, tipo 0x01 (texto), algoritmo RSA
        if tag != 2 || b.len() < 10 || b[0] != 4 || b[1] != 0x01 || b[2] != 1 {
            continue;
        }
        seen += 1;
        let hash_algo = b[3];
        let hl = u16::from_be_bytes([b[4], b[5]]) as usize;
        let hashed_end = 6 + hl;
        let ul = match b.get(hashed_end..hashed_end + 2) {
            Some(x) => u16::from_be_bytes([x[0], x[1]]) as usize,
            None => continue,
        };
        let (sig, _) = match pgp_mpi(b, hashed_end + 2 + ul + 2) {
            Some(x) => x,
            None => continue,
        };
        if env::var("KPKG_DEBUG").is_ok() {
            println!(
                "[debug] signature: hash_algo={} issuer(s)={:?} mpi_bytes={}",
                hash_algo,
                sig_issuers(b, hashed_end, ul),
                sig.len()
            );
        }
        let mut data = canon.clone();
        data.extend_from_slice(&b[..hashed_end]);
        data.extend_from_slice(&[0x04, 0xff]);
        data.extend_from_slice(&(hashed_end as u32).to_be_bytes());
        for k in keys {
            if rsa_verify(k, hash_algo, &data, sig) {
                return Ok(text);
            }
        }
    }
    if env::var("KPKG_DEBUG").is_ok() {
        println!("[debug] {} RSA signature(s) found, {} key(s) tried, none verified", seen, keys.len());
    }
    Err("SECURITY ALERT: the OpenPGP signature of InRelease is invalid or was made by an untrusted key. \
Possible man-in-the-middle, a compromised mirror, or an outdated keyring."
        .into())
}

struct Release {
    suite: String,
    codename: String,
    version: String,
    date: String,
    valid_until: String,
    hashes: HashMap<String, (String, u64)>,
}

fn is_not_found(e: &Box<dyn Error>) -> bool {
    matches!(e.downcast_ref::<ureq::Error>(), Some(ureq::Error::Status(404, _)))
}

/// Fecha del ultimo Release aceptado para esta suite (proteccion contra "rollback"/replay).
fn prev_release_date(ctx: &Ctx, suite: &str) -> Option<i64> {
    let t = fs::read_to_string(ctx.meta_path()).ok()?;
    let key = format!("Checked-{}: ", suite);
    for l in t.lines() {
        if let Some(v) = l.strip_prefix(key.as_str()) {
            return parse_http_date(v);
        }
    }
    None
}

fn fetch_release(ctx: &Ctx, mirror: &str, suite: &str, keys: &[RsaPublicKey]) -> Res<Release> {
    let url = format!("{}/dists/{}/InRelease", mirror, suite);
    println!("[*] Fetching {}", url);
    let raw = http_get(&url, MAX_RELEASE)?;
    let text = String::from_utf8(raw).map_err(|_| format!("{} is not valid UTF-8", url))?;
    let verified = verify_inrelease(&text, keys).map_err(|e| format!("{}: {}", url, e))?;
    println!("[+] Signature OK ({})", suite);

    let mut rel = Release {
        suite: String::new(),
        codename: String::new(),
        version: String::new(),
        date: String::new(),
        valid_until: String::new(),
        hashes: parse_release_hashes(&verified),
    };
    for line in verified.lines() {
        if let Some(v) = line.strip_prefix("Suite: ") {
            rel.suite = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("Codename: ") {
            rel.codename = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("Version: ") {
            rel.version = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("Date: ") {
            rel.date = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("Valid-Until: ") {
            rel.valid_until = v.trim().to_string();
        }
    }

    // Un InRelease valido pero de OTRA suite (replay) tambien se rechaza.
    if rel.suite != suite && rel.codename != suite {
        return Err(format!(
            "SECURITY ALERT: requested suite '{}' but the signed Release describes '{}' / '{}'",
            suite, rel.suite, rel.codename
        )
        .into());
    }
    let now = now_unix();
    if let Some(vu) = parse_http_date(&rel.valid_until) {
        if now > vu {
            return Err(format!(
                "{}: the Release expired on {} (check your system clock, or the mirror is serving stale data)",
                url, rel.valid_until
            )
            .into());
        }
    }
    if let (Some(prev), Some(cur)) = (prev_release_date(ctx, suite), parse_http_date(&rel.date)) {
        if cur < prev {
            return Err(format!(
                "SECURITY ALERT: {} is OLDER ({}) than the one already accepted; possible replay/rollback attack",
                url, rel.date
            )
            .into());
        }
    }
    Ok(rel)
}

/// Descarga Packages.xz de una suite (verificado contra el Release ya firmado) y lo mezcla en
/// `all`, quedandose con la version MAS ALTA de cada paquete (como hace apt entre
/// stable / stable-updates / stable-security). Cada paquete recuerda de que mirror viene.
fn fetch_packages(
    ctx: &Ctx,
    mirror: &str,
    suite: &str,
    rel: &Release,
    all: &mut HashMap<String, Pkg>,
    skipped: &mut usize,
) -> Res<()> {
    let base = format!("{}/dists/{}", mirror, suite);
    let wanted = [
        format!("{}/binary-{}/Packages.xz", COMPONENT, ctx.arch),
        format!("{}/binary-all/Packages.xz", COMPONENT),
    ];

    for (n, rel_path) in wanted.iter().enumerate() {
        let (want_sha, want_size) = match rel.hashes.get(rel_path) {
            Some(x) => x.clone(),
            None => {
                if n == 0 {
                    return Err(format!(
                        "{} is not listed in Release of '{}' (is '{}' a valid arch for this suite?)",
                        rel_path, suite, ctx.arch
                    )
                    .into());
                }
                println!("[*] {} is not published in {}, skipping", rel_path, suite);
                continue;
            }
        };
        if want_size > MAX_INDEX_XZ {
            return Err(format!("{} is unreasonably large ({} bytes)", rel_path, want_size).into());
        }
        println!("[*] Downloading {}/{} ({})", suite, rel_path, human(want_size));
        let data = http_get(&format!("{}/{}", base, rel_path), MAX_INDEX_XZ)?;
        if data.len() as u64 != want_size {
            return Err(format!(
                "size mismatch for {}: Release says {} bytes, got {}",
                rel_path,
                want_size,
                data.len()
            )
            .into());
        }
        let got = sha256_hex(&data);
        if got != want_sha {
            return Err(format!(
                "SECURITY ALERT: SHA256 mismatch for {}\n  expected:   {}\n  calculated: {}",
                rel_path, want_sha, got
            )
            .into());
        }

        let mut raw: Vec<u8> = Vec::new();
        XzDecoder::new(&data[..]).take(MAX_INDEX_TEXT + 1).read_to_end(&mut raw)?;
        if raw.len() as u64 > MAX_INDEX_TEXT {
            return Err(format!("{} decompresses to more than {} bytes", rel_path, MAX_INDEX_TEXT).into());
        }
        let text = String::from_utf8_lossy(&raw).to_string();
        drop(raw);

        let mut parsed: Vec<Pkg> = Vec::new();
        parse_packages(&text, &mut parsed);
        for mut p in parsed {
            if p.arch != ctx.arch && p.arch != "all" {
                continue;
            }
            if !valid_pkg_name(&p.name)
                || !valid_filename(&p.filename)
                || !valid_sha256(&p.sha256)
                || !valid_version(&p.version)
            {
                *skipped += 1;
                continue;
            }
            p.origin = mirror.to_string();
            let replace = match all.get(&p.name) {
                Some(old) => cmp_version(&p.version, &old.version) == Ordering::Greater,
                None => true,
            };
            if replace {
                all.insert(p.name.clone(), p);
            }
        }
    }
    Ok(())
}

fn cmd_update(ctx: &Ctx) -> Res<()> {
    let _lock = Lock::acquire(ctx)?;
    let keys = load_keyring(ctx)?;
    println!("[*] Trusting {} OpenPGP key(s) for repository signatures", keys.len());

    let mut all: HashMap<String, Pkg> = HashMap::new();
    let mut skipped = 0usize;
    let mut meta = String::new();
    let mut checked = String::new();

    let primary = fetch_release(ctx, &ctx.mirror, &ctx.suite, &keys)?;
    for (k, v) in [
        ("Suite", &primary.suite),
        ("Codename", &primary.codename),
        ("Version", &primary.version),
        ("Date", &primary.date),
    ] {
        if !v.is_empty() {
            println!("    {}: {}", k, v);
            meta.push_str(&format!("{}: {}\n", k, v));
        }
    }
    fetch_packages(ctx, &ctx.mirror, &ctx.suite, &primary, &mut all, &mut skipped)?;
    checked.push_str(&format!("Checked-{}: {}\n", ctx.suite, primary.date));

    // Debian publica los parches de seguridad en una suite APARTE ("<codename>-security") que
    // se adelanta a "stable" hasta la siguiente point release. Se mezclan las tres y se elige
    // siempre la version mas alta.
    let stable_like = matches!(primary.suite.as_str(), "stable" | "oldstable" | "oldoldstable");
    let mut extra: Vec<(String, String, bool)> = Vec::new(); // (mirror, suite, obligatoria)
    if stable_like && valid_suite(&primary.codename) {
        extra.push((ctx.mirror.clone(), format!("{}-updates", primary.codename), false));
        if !ctx.no_security {
            extra.push((ctx.security_mirror.clone(), format!("{}-security", primary.codename), true));
        }
    } else if !stable_like {
        println!("[i] '{}' has no separate security suite; using the main suite only", primary.suite);
    }
    if ctx.no_security {
        println!("[!] --no-security: security updates are NOT included");
    }
    for (mirror, suite, mandatory) in &extra {
        match fetch_release(ctx, mirror, suite, &keys) {
            Ok(rel) => {
                fetch_packages(ctx, mirror, suite, &rel, &mut all, &mut skipped)?;
                checked.push_str(&format!("Checked-{}: {}\n", suite, rel.date));
                if *mandatory {
                    meta.push_str(&format!("Security-Suite: {}\nSecurity-Mirror: {}\n", suite, mirror));
                }
            }
            Err(e) if !*mandatory && is_not_found(&e) => println!("[i] {} is not published, skipping", suite),
            Err(e) => return Err(e),
        }
    }

    let mut list: Vec<&Pkg> = all.values().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::new();
    for p in list {
        out.push_str(&p.to_line());
        out.push('\n');
    }
    write_atomic(&ctx.index_path(), out.as_bytes())?;
    meta.push_str(&format!("Mirror: {}\nArch: {}\n", ctx.mirror, ctx.arch));
    meta.push_str(&checked);
    write_atomic(&ctx.meta_path(), meta.as_bytes())?;
    println!("[+] Index updated: {} packages ({} entries rejected as invalid)", all.len(), skipped);

    check_base_update(ctx);
    Ok(())
}

// ============================================================
// RESOLUCION DE DEPENDENCIAS
// ============================================================
// Version ligera de lo que hace apt: Pre-Depends + Depends (con alternativas "a | b",
// versiones y paquetes virtuales via Provides), Conflicts/Breaks, y backtracking
// simple por alternativa. Ignora Recommends/Suggests.

struct Resolver<'a> {
    ctx: &'a Ctx,
    idx: &'a Index,
    db: &'a Db,
    plan: Vec<usize>,
    chosen: HashSet<usize>,
    chosen_names: HashSet<String>,
}

impl<'a> Resolver<'a> {
    fn new(ctx: &'a Ctx, idx: &'a Index, db: &'a Db) -> Resolver<'a> {
        Resolver {
            ctx,
            idx,
            db,
            plan: Vec::new(),
            chosen: HashSet::new(),
            chosen_names: HashSet::new(),
        }
    }

    fn is_satisfied(&self, d: &Dep) -> bool {
        for p in self.db.pkgs.values() {
            if self.chosen_names.contains(&p.name) {
                continue; // se va a reemplazar por la version elegida
            }
            if pkg_satisfies(p, d) {
                return true;
            }
        }
        for (n, v) in &self.db.assumed {
            if *n == d.name {
                let ok = match (&d.op, &d.ver, v) {
                    (Some(op), Some(want), Some(have)) => ver_satisfies(have, op, want),
                    _ => true,
                };
                if ok {
                    return true;
                }
            }
        }
        for &i in &self.chosen {
            if pkg_satisfies(&self.idx.pkgs[i], d) {
                return true;
            }
        }
        false
    }

    fn check_conflicts(&self, p: &Pkg) -> Res<()> {
        let mine: Vec<&String> = vec![&p.conflicts, &p.breaks];
        for field in mine {
            for entry in parse_dep_list(field) {
                for d in entry {
                    for q in self.db.pkgs.values() {
                        if q.name == p.name || self.chosen_names.contains(&q.name) {
                            continue;
                        }
                        if pkg_satisfies(q, &d) {
                            return Err(format!("{} conflicts with installed {} {}", p.name, q.name, q.version).into());
                        }
                    }
                    for &i in &self.chosen {
                        let q = &self.idx.pkgs[i];
                        if q.name != p.name && pkg_satisfies(q, &d) {
                            return Err(format!("{} conflicts with {} (selected in this transaction)", p.name, q.name).into());
                        }
                    }
                }
            }
        }
        // los que ya estan (instalados o elegidos) tambien pueden declarar conflicto con p
        let mut others: Vec<&Pkg> = Vec::new();
        for q in self.db.pkgs.values() {
            if q.name != p.name && !self.chosen_names.contains(&q.name) {
                others.push(q);
            }
        }
        for &i in &self.chosen {
            let q = &self.idx.pkgs[i];
            if q.name != p.name {
                others.push(q);
            }
        }
        for q in others {
            let theirs: Vec<&String> = vec![&q.conflicts, &q.breaks];
            for field in theirs {
                for entry in parse_dep_list(field) {
                    for d in entry {
                        if pkg_satisfies(p, &d) {
                            return Err(format!("{} conflicts with {} {}", q.name, p.name, p.version).into());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Anade un paquete (y, recursivamente, sus dependencias). Postorden: dependencias primero.
    fn add_pkg(&mut self, i: usize) -> Res<()> {
        if self.chosen.contains(&i) {
            return Ok(());
        }
        let p = self.idx.pkgs[i].clone();
        if self.chosen_names.contains(&p.name) {
            return Ok(());
        }
        self.check_conflicts(&p)?;
        self.chosen.insert(i);
        self.chosen_names.insert(p.name.clone());
        for entry in effective_deps(self.ctx, &p) {
            self.satisfy(&entry, &p.name)?;
        }
        self.plan.push(i);
        Ok(())
    }

    /// Como add_pkg pero deja el estado intacto si falla.
    fn try_add(&mut self, i: usize) -> Res<()> {
        let sp = self.plan.clone();
        let sc = self.chosen.clone();
        let sn = self.chosen_names.clone();
        let r = self.add_pkg(i);
        if r.is_err() {
            self.plan = sp;
            self.chosen = sc;
            self.chosen_names = sn;
        }
        r
    }

    /// Satisface una entrada "a | b | c": ya instalada -> nada; si no, primer candidato viable.
    fn satisfy(&mut self, entry: &[Dep], needed_by: &str) -> Res<()> {
        for d in entry {
            if self.is_satisfied(d) {
                return Ok(());
            }
        }
        let mut last_err: Option<String> = None;
        for d in entry {
            let cands = self.idx.candidates(d);
            for ci in cands {
                match self.try_add(ci) {
                    Ok(()) => return Ok(()),
                    Err(e) => last_err = Some(e.to_string()),
                }
            }
        }
        let wanted: Vec<String> = entry.iter().map(dep_to_string).collect();
        match last_err {
            Some(e) => Err(format!("{} needs {}: {}", needed_by, wanted.join(" | "), e).into()),
            None => Err(format!(
                "{} needs {}, but nothing in the index provides it",
                needed_by,
                wanted.join(" | ")
            )
            .into()),
        }
    }
}

/// Dependencias que se romperian al quitar `removed` (dependiente, entrada de dependencia).
fn broken_by_removal(ctx: &Ctx, db: &Db, removed: &HashSet<String>) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for q in db.pkgs.values() {
        if removed.contains(&q.name) {
            continue;
        }
        for entry in effective_deps(ctx, q) {
            let hit = entry
                .iter()
                .any(|d| db.pkgs.values().any(|r| removed.contains(&r.name) && pkg_satisfies(r, d)));
            if !hit {
                continue;
            }
            let ok = entry.iter().any(|d| {
                db.pkgs.values().any(|r| !removed.contains(&r.name) && pkg_satisfies(r, d))
                    || db.assumed.iter().any(|(n, _)| *n == d.name)
            });
            if !ok {
                let text: Vec<String> = entry.iter().map(dep_to_string).collect();
                out.push((q.name.clone(), text.join(" | ")));
            }
        }
    }
    out
}

// ============================================================
// EXTRACCION SEGURA
// ============================================================

/// Normaliza una ruta de un tar a ruta relativa; rechaza "..", absolutas y caracteres de control.
fn safe_rel(path: &Path) -> Res<PathBuf> {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Normal(c) => {
                let s = c.to_str().ok_or("path is not valid UTF-8")?;
                if s.chars().any(|ch| ch.is_control()) {
                    return Err(format!("control character in path: {:?}", path).into());
                }
                out.push(c);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!("path traversal ('..') rejected: {}", path.display()).into());
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("absolute path rejected: {}", path.display()).into());
            }
        }
    }
    Ok(out)
}

/// Verifica que `p` sigue dentro de `root` tras resolver symlinks existentes.
fn ensure_inside(root: &Path, p: &Path) -> Res<()> {
    let mut cur = p.to_path_buf();
    loop {
        match cur.canonicalize() {
            Ok(real) => {
                let root_real = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
                if !real.starts_with(&root_real) {
                    return Err(format!("path escapes the rootfs: {} -> {}", p.display(), real.display()).into());
                }
                return Ok(());
            }
            Err(_) => {
                if !cur.pop() {
                    return Err("could not resolve path".into());
                }
            }
        }
    }
}

/// Un symlink puede ser absoluto (se resuelve dentro del rootfs) o relativo sin subir por encima de la raiz.
fn symlink_target_ok(parent_depth: usize, target: &Path) -> bool {
    let mut depth = parent_depth as i64;
    for comp in target.components() {
        match comp {
            Component::Normal(_) => depth += 1,
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return true,
        }
    }
    true
}

struct LinkOp {
    hard: bool,
    dest: PathBuf,
    target: PathBuf,
}

struct Extracted {
    files: Vec<FileRec>,
    pending: Vec<(PathBuf, PathBuf)>, // (temporal, destino final)
    links: Vec<LinkOp>,
}

fn extract_entries<R: Read>(
    ctx: &Ctx,
    pkg: &Pkg,
    archive: &mut tar::Archive<R>,
    owners: &HashMap<String, String>,
    old_sha: &HashMap<String, String>,
    out: &mut Extracted,
) -> Res<()> {
    let root = &ctx.root;
    let mut total: u64 = 0;

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let etype = entry.header().entry_type();
        let raw_path = entry.path()?.into_owned();
        let rel = safe_rel(&raw_path)?;
        if rel.as_os_str().is_empty() {
            continue; // "./"
        }
        let rel_str = rel.to_str().ok_or("path is not valid UTF-8")?.to_string();
        let dest = root.join(&rel);

        match etype {
            EntryType::XGlobalHeader => continue,
            EntryType::Regular | EntryType::Directory | EntryType::Symlink | EntryType::Link => {}
            EntryType::Char | EntryType::Block | EntryType::Fifo => {
                return Err(format!(
                    "refusing to extract special file (type {:?}): /{}",
                    etype, rel_str
                )
                .into());
            }
            other => {
                return Err(format!("unsupported tar entry type {:?}: /{}", other, rel_str).into());
            }
        }

        // 1. conflictos de ficheros
        if etype != EntryType::Directory {
            match owners.get(&rel_str) {
                Some(owner) if *owner != pkg.name => {
                    if !ctx.force {
                        return Err(format!(
                            "file conflict: /{} belongs to package {} (use --force to overwrite)",
                            rel_str, owner
                        )
                        .into());
                    }
                }
                Some(_) => {}
                None => {
                    if !ctx.force && fs::symlink_metadata(&dest).is_ok() {
                        return Err(format!(
                            "/{} already exists and is not owned by any package (use --force to overwrite)",
                            rel_str
                        )
                        .into());
                    }
                }
            }
        }

        // 2. limites de tamano
        let size = entry.header().size()?;
        if size > MAX_ENTRY_SIZE {
            return Err(format!("entry /{} is too large ({} bytes)", rel_str, size).into());
        }
        total = total.saturating_add(size);
        if total > MAX_TOTAL_SIZE {
            return Err(format!("package exceeds the total extraction limit ({} bytes)", MAX_TOTAL_SIZE).into());
        }

        // 3. directorio padre dentro del rootfs
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
            ensure_inside(root, parent)?;
        }

        // 4. escribir segun el tipo
        match etype {
            EntryType::Directory => {
                fs::create_dir_all(&dest)?;
                ensure_inside(root, &dest)?;
                out.files.push(FileRec { kind: 'd', path: rel_str, extra: String::new() });
            }
            EntryType::Regular => {
                // setuid/setgid/sticky fuera: nunca en un rootfs proot
                let mode = entry.header().mode().unwrap_or(0o644) & 0o777;

                // un fichero de /etc modificado por el usuario se conserva
                if rel_str.starts_with("etc/") {
                    if let (Some(old), Ok(cur)) = (old_sha.get(&rel_str), sha256_file(&dest)) {
                        if *old != cur {
                            println!("[!] keeping your modified config /{}", rel_str);
                            out.files.push(FileRec { kind: 'f', path: rel_str.clone(), extra: old.clone() });
                            continue;
                        }
                    }
                }

                let file_name = dest.file_name().ok_or("invalid path")?.to_string_lossy().to_string();
                let tmp = dest.with_file_name(format!("{}.kpkg-new", file_name));
                let _ = fs::remove_file(&tmp);
                let mut f = fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?;
                out.pending.push((tmp.clone(), dest.clone()));
                let mut hasher = Sha256::new();
                let mut buf = vec![0u8; 65536];
                loop {
                    let n = entry.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buf[..n]);
                    f.write_all(&buf[..n])?;
                }
                drop(f);
                fs::set_permissions(&tmp, fs::Permissions::from_mode(mode))?;
                out.files.push(FileRec { kind: 'f', path: rel_str, extra: hex::encode(hasher.finalize()) });
            }
            EntryType::Symlink => {
                let link = entry.link_name()?.ok_or("symlink without target")?.into_owned();
                let depth = rel.components().count().saturating_sub(1);
                if !symlink_target_ok(depth, &link) {
                    return Err(format!(
                        "symlink /{} -> {} would escape the rootfs",
                        rel_str,
                        link.display()
                    )
                    .into());
                }
                out.files.push(FileRec {
                    kind: 'l',
                    path: rel_str,
                    extra: link.to_string_lossy().to_string(),
                });
                out.links.push(LinkOp { hard: false, dest, target: link });
            }
            EntryType::Link => {
                let link = entry.link_name()?.ok_or("hardlink without target")?.into_owned();
                let src_rel = safe_rel(&link)?;
                let src_str = src_rel.to_str().ok_or("path is not valid UTF-8")?.to_string();
                let src = root.join(&src_rel);
                out.files.push(FileRec { kind: 'h', path: rel_str, extra: src_str });
                out.links.push(LinkOp { hard: true, dest, target: src });
            }
            _ => {}
        }
    }
    Ok(())
}

fn cleanup_pending(ex: &Extracted) {
    for (tmp, _) in &ex.pending {
        let _ = fs::remove_file(tmp);
    }
}

/// Fase 2: tmp -> destino (rename atomico por fichero), luego symlinks y hardlinks.
fn commit(ex: &Extracted) -> Res<()> {
    for (tmp, dest) in &ex.pending {
        fs::rename(tmp, dest)?;
    }
    for l in &ex.links {
        let _ = fs::remove_file(&l.dest);
        if l.hard {
            fs::hard_link(&l.target, &l.dest)?;
        } else {
            std::os::unix::fs::symlink(&l.target, &l.dest)?;
        }
    }
    Ok(())
}

fn db_path_ok(path: &str) -> bool {
    safe_rel(Path::new(path)).is_ok()
}

/// Quita ficheros de una version anterior que la nueva ya no trae (solo si no fueron modificados).
fn remove_stale(ctx: &Ctx, old: &[FileRec], new_paths: &HashSet<String>) {
    for f in old {
        if f.kind == 'd' || new_paths.contains(&f.path) || !db_path_ok(&f.path) {
            continue;
        }
        let p = ctx.root.join(&f.path);
        if f.kind == 'f' {
            match sha256_file(&p) {
                Ok(cur) if cur == f.extra => {
                    let _ = fs::remove_file(&p);
                }
                Ok(_) => println!("[!] keeping modified /{}", f.path),
                Err(_) => {}
            }
        } else {
            let _ = fs::remove_file(&p);
        }
    }
}

fn fetch_deb(ctx: &Ctx, p: &Pkg) -> Res<PathBuf> {
    let base = p.filename.rsplit('/').next().unwrap_or("package.deb").to_string();
    let path = ctx.cache_pkgs().join(&base);
    if path.is_file() {
        if let Ok(h) = sha256_file(&path) {
            if h == p.sha256 {
                println!("[*] {} (cached)", base);
                return Ok(path);
            }
        }
    }
    println!("[*] Downloading {} ({})", base, human(p.size));
    let base_url = if p.origin.starts_with("https://") || p.origin.starts_with("http://") {
        p.origin.clone()
    } else {
        ctx.mirror.clone()
    };
    download_verified(&format!("{}/{}", base_url, p.filename), &path, p.size, &p.sha256)?;
    Ok(path)
}

fn install_one(ctx: &Ctx, p: &Pkg, owners: &mut HashMap<String, String>, db: &Db) -> Res<()> {
    let deb_path = fetch_deb(ctx, p)?;

    let old_files: Vec<FileRec> = if db.pkgs.contains_key(&p.name) {
        load_files(ctx, &p.name)
    } else {
        Vec::new()
    };
    let mut old_sha: HashMap<String, String> = HashMap::new();
    for f in &old_files {
        if f.kind == 'f' {
            old_sha.insert(f.path.clone(), f.extra.clone());
        }
    }

    let file = fs::File::open(&deb_path)?;
    let mut ar_archive = ar::Archive::new(file);
    let mut ex = Extracted { files: Vec::new(), pending: Vec::new(), links: Vec::new() };
    let mut found = false;

    while let Some(entry_result) = ar_archive.next_entry() {
        let entry = entry_result?;
        let name = String::from_utf8_lossy(entry.header().identifier().as_ref()).to_string();
        if !name.starts_with("data.tar") {
            continue;
        }
        found = true;
        let res: Res<()> = if name.ends_with(".xz") {
            let mut a = tar::Archive::new(XzDecoder::new(entry));
            extract_entries(ctx, p, &mut a, owners, &old_sha, &mut ex)
        } else if name.ends_with(".gz") {
            let mut a = tar::Archive::new(GzDecoder::new(entry));
            extract_entries(ctx, p, &mut a, owners, &old_sha, &mut ex)
        } else {
            Err(format!("unsupported compression inside .deb: {} (only .xz and .gz)", name).into())
        };
        if let Err(e) = res {
            cleanup_pending(&ex);
            return Err(e);
        }
        break;
    }
    if !found {
        return Err("no data.tar.* member found in the .deb".into());
    }

    let hook = load_hook(ctx, &p.name);
    if let Err(e) = run_hook_stage(ctx, &p.name, "pre-install", &hook.pre_install) {
        cleanup_pending(&ex);
        return Err(e);
    }
    if let Err(e) = commit(&ex) {
        cleanup_pending(&ex);
        return Err(e);
    }

    let new_paths: HashSet<String> = ex.files.iter().map(|f| f.path.clone()).collect();
    remove_stale(ctx, &old_files, &new_paths);

    let mut files_text = String::new();
    for f in &ex.files {
        files_text.push_str(&f.to_line());
        files_text.push('\n');
    }
    let dir = ctx.pkgs_dir().join(&p.name);
    write_atomic(&dir.join("files"), files_text.as_bytes())?;
    write_atomic(&dir.join("info"), format!("{}\n", p.to_line()).as_bytes())?;

    for f in &old_files {
        owners.remove(&f.path);
    }
    for f in &ex.files {
        if f.kind != 'd' {
            owners.insert(f.path.clone(), p.name.clone());
        }
    }
    if let Err(e) = run_hook_stage(ctx, &p.name, "post-install", &hook.post_install) {
        println!("[!] {} (the package is installed; fix the cause and run 'kpkg configure {}')", e, p.name);
    }
    Ok(())
}

fn remove_pkg(ctx: &Ctx, name: &str, purge: bool) -> Res<()> {
    let hook = load_hook(ctx, name);
    run_hook_stage(ctx, name, "pre-remove", &hook.pre_remove)?;
    let files = load_files(ctx, name);
    for f in &files {
        if f.kind == 'd' || !db_path_ok(&f.path) {
            continue;
        }
        let p = ctx.root.join(&f.path);
        if f.kind == 'f' {
            match sha256_file(&p) {
                Ok(cur) if cur == f.extra || purge => {
                    let _ = fs::remove_file(&p);
                }
                Ok(_) => println!("[!] keeping modified /{}", f.path),
                Err(_) => {}
            }
        } else {
            let _ = fs::remove_file(&p);
        }
    }
    // directorios vacios, de dentro hacia fuera; nunca los de primer/segundo nivel (usr, usr/bin, ...)
    let mut dirs: Vec<&FileRec> = files.iter().filter(|f| f.kind == 'd').collect();
    dirs.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    for d in dirs {
        if db_path_ok(&d.path) && Path::new(&d.path).components().count() > 2 {
            let _ = fs::remove_dir(ctx.root.join(&d.path));
        }
    }
    fs::remove_dir_all(ctx.pkgs_dir().join(name))?;
    forget_manual(ctx, name);
    if let Err(e) = run_hook_stage(ctx, name, "post-remove", &hook.post_remove) {
        println!("[!] {}", e);
    }
    Ok(())
}

// ============================================================
// COMANDOS
// ============================================================

/// Confirmacion interactiva: SOLO en modo apt (apt/apt-get), con terminal y sin -y.
/// En modo apk (por defecto) kpkg nunca pregunta, como apk.
fn confirm(ctx: &Ctx, question: &str) -> bool {
    if ctx.yes || !ctx.apt || !io::stdin().is_terminal() {
        return true;
    }
    print!("{} [Y/n] ", question);
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return false;
    }
    let l = line.trim().to_lowercase();
    l.is_empty() || l == "y" || l == "yes"
}

// ---- paquetes instalados "a mano" vs "como dependencia" (para autoremove) ----

fn load_manual(ctx: &Ctx) -> Option<HashSet<String>> {
    let t = fs::read_to_string(ctx.manual_path()).ok()?;
    Some(t.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
}

fn save_manual(ctx: &Ctx, set: &HashSet<String>) -> Res<()> {
    let mut v: Vec<&String> = set.iter().collect();
    v.sort();
    let mut s = String::new();
    for n in v {
        s.push_str(n);
        s.push('\n');
    }
    write_atomic(&ctx.manual_path(), s.as_bytes())
}

fn forget_manual(ctx: &Ctx, name: &str) {
    if let Some(mut m) = load_manual(ctx) {
        if m.remove(name) {
            let _ = save_manual(ctx, &m);
        }
    }
}

/// La primera vez, todo lo que ya estaba instalado se considera manual (no se puede saber mas).
fn mark_manual(ctx: &Ctx, db_before: &Db, names: &[String]) {
    let mut m: HashSet<String> = load_manual(ctx).unwrap_or_else(|| db_before.pkgs.keys().cloned().collect());
    for n in names {
        m.insert(n.clone());
    }
    if let Err(e) = save_manual(ctx, &m) {
        println!("[!] could not record manually installed packages: {}", e);
    }
}

/// Paquetes alcanzables desde los instalados manualmente siguiendo sus dependencias.
fn needed_set(ctx: &Ctx, db: &Db, manual: &HashSet<String>) -> HashSet<String> {
    let mut needed: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = manual.iter().filter(|n| db.pkgs.contains_key(*n)).cloned().collect();
    while let Some(n) = stack.pop() {
        if !needed.insert(n.clone()) {
            continue;
        }
        let p = match db.pkgs.get(&n) {
            Some(p) => p,
            None => continue,
        };
        for entry in effective_deps(ctx, p) {
            for d in &entry {
                let mut found = false;
                for q in db.pkgs.values() {
                    if pkg_satisfies(q, d) {
                        stack.push(q.name.clone());
                        found = true;
                    }
                }
                if found {
                    break;
                }
            }
        }
    }
    needed
}

fn run_plan(ctx: &Ctx, idx: &Index, db: &Db, plan: Vec<usize>) -> Res<()> {
    if plan.is_empty() {
        println!("[=] Nothing to do.");
        return Ok(());
    }
    let mut dl: u64 = 0;
    let mut inst: u64 = 0;
    println!("[*] Packages to install/upgrade ({}):", plan.len());
    for &i in &plan {
        let p = &idx.pkgs[i];
        let tag = match db.pkgs.get(&p.name) {
            Some(o) => format!("upgrade from {}", o.version),
            None => "new".to_string(),
        };
        let src = if p.origin.contains("security") { " (security)" } else { "" };
        println!("    {} {} [{}]{}", p.name, p.version, tag, src);
        dl += p.size;
        inst += p.installed_size * 1024;
    }
    println!("[*] Download: {}, installed size: {}", human(dl), human(inst));
    if ctx.simulate {
        println!("[=] --simulate: nothing was changed.");
        return Ok(());
    }
    if !confirm(ctx, "Do you want to continue?") {
        return Err("aborted by user".into());
    }
    let mut owners = load_owners(ctx, db);
    for &i in &plan {
        let p = &idx.pkgs[i];
        install_one(ctx, p, &mut owners, db)?;
        println!("[+] {} {} installed", p.name, p.version);
    }
    println!("[i] kpkg never runs maintainer scripts (preinst/postinst/...); declarative hooks are used instead.");
    Ok(())
}

fn cmd_add(ctx: &Ctx, names: &[String], reinstall: bool) -> Res<()> {
    if names.is_empty() {
        return Err("add: no package given".into());
    }
    let _lock = Lock::acquire(ctx)?;
    let idx = load_index(ctx)?;
    let db = load_db(ctx)?;
    let mut r = Resolver::new(ctx, &idx, &db);
    let mut requested: Vec<String> = Vec::new();

    for name in names {
        let i = match idx.by_name.get(name.as_str()) {
            Some(&i) => i,
            None => {
                let provs = idx.candidates(&Dep { name: name.clone(), op: None, ver: None });
                if provs.is_empty() {
                    return Err(format!("package '{}' not found in the index", name).into());
                }
                if provs.len() > 1 {
                    let list: Vec<String> = provs.iter().map(|&k| idx.pkgs[k].name.clone()).collect();
                    return Err(format!(
                        "'{}' is a virtual package provided by: {}. Choose one.",
                        name,
                        list.join(", ")
                    )
                    .into());
                }
                provs[0]
            }
        };
        let p = &idx.pkgs[i];
        if !db.pkgs.contains_key(&p.name) && db.assumed.iter().any(|(n, _)| *n == p.name) {
            println!("[=] {} is provided by the base system (assume-installed)", p.name);
            continue;
        }
        requested.push(p.name.clone());
        if let Some(inst) = db.pkgs.get(&p.name) {
            if !reinstall && cmp_version(&inst.version, &p.version) != Ordering::Less {
                println!("[=] {} {} is already installed", inst.name, inst.version);
                continue;
            }
        }
        r.try_add(i)?;
    }
    let plan = r.plan.clone();
    run_plan(ctx, &idx, &db, plan)?;
    if !ctx.simulate {
        mark_manual(ctx, &db, &requested);
    }
    Ok(())
}

fn cmd_upgrade(ctx: &Ctx) -> Res<()> {
    let _lock = Lock::acquire(ctx)?;
    // 1) sistema base de Kyanite (si 'update' encontro una release nueva)
    base_upgrade(ctx)?;
    // 2) paquetes
    let idx = load_index(ctx)?;
    let db = load_db(ctx)?;
    let mut r = Resolver::new(ctx, &idx, &db);
    let mut names: Vec<&String> = db.pkgs.keys().collect();
    names.sort();
    for n in names {
        let inst = &db.pkgs[n];
        if let Some(&i) = idx.by_name.get(n) {
            if cmp_version(&idx.pkgs[i].version, &inst.version) == Ordering::Greater {
                if let Err(e) = r.try_add(i) {
                    println!("[!] skipping {}: {}", n, e);
                }
            }
        }
    }
    let plan = r.plan.clone();
    run_plan(ctx, &idx, &db, plan)
}

fn cmd_del(ctx: &Ctx, names: &[String], purge: bool) -> Res<()> {
    if names.is_empty() {
        return Err("del: no package given".into());
    }
    let _lock = Lock::acquire(ctx)?;
    let db = load_db(ctx)?;
    let mut removed: HashSet<String> = HashSet::new();
    for n in names {
        if !db.pkgs.contains_key(n) {
            return Err(format!("'{}' is not installed", n).into());
        }
        removed.insert(n.clone());
    }
    let broken = broken_by_removal(ctx, &db, &removed);
    if !broken.is_empty() {
        for (who, dep) in &broken {
            eprintln!("[!] {} depends on {}", who, dep);
        }
        if !ctx.force {
            return Err("refusing to remove: other packages need them (use --force to override)".into());
        }
    }
    let mut list: Vec<&String> = removed.iter().collect();
    list.sort();
    if ctx.simulate {
        for n in list {
            println!("    would remove {}", n);
        }
        println!("[=] --simulate: nothing was changed.");
        return Ok(());
    }
    if !confirm(ctx, "Do you want to continue?") {
        return Err("aborted by user".into());
    }
    for n in list {
        remove_pkg(ctx, n, purge)?;
        println!("[+] {} removed", n);
    }
    Ok(())
}

fn cmd_autoremove(ctx: &Ctx) -> Res<()> {
    let _lock = Lock::acquire(ctx)?;
    let db = load_db(ctx)?;
    let manual = match load_manual(ctx) {
        Some(m) => m,
        None => {
            println!("[=] No record yet of which packages were installed explicitly; nothing to autoremove.");
            return Ok(());
        }
    };
    let needed = needed_set(ctx, &db, &manual);
    let mut removable: Vec<String> = db.pkgs.keys().filter(|n| !needed.contains(*n)).cloned().collect();
    removable.sort();
    if removable.is_empty() {
        println!("[=] Nothing to remove.");
        return Ok(());
    }
    println!("[*] Packages that are no longer required ({}):", removable.len());
    for n in &removable {
        println!("    {} {}", n, db.pkgs[n].version);
    }
    if ctx.simulate {
        println!("[=] --simulate: nothing was changed.");
        return Ok(());
    }
    if !confirm(ctx, "Do you want to continue?") {
        return Err("aborted by user".into());
    }
    for n in &removable {
        remove_pkg(ctx, n, false)?;
        println!("[+] {} removed", n);
    }
    Ok(())
}

/// Vuelve a ejecutar los hooks post-install (equivalente a "dpkg --configure").
fn cmd_configure(ctx: &Ctx, names: &[String]) -> Res<()> {
    let _lock = Lock::acquire(ctx)?;
    let db = load_db(ctx)?;
    let mut list: Vec<String> = if names.is_empty() { db.pkgs.keys().cloned().collect() } else { names.to_vec() };
    list.sort();
    let mut ran = 0usize;
    for n in &list {
        if !db.pkgs.contains_key(n) {
            return Err(format!("'{}' is not installed", n).into());
        }
        let hook = load_hook(ctx, n);
        if hook.post_install.is_empty() {
            continue;
        }
        if ctx.simulate {
            println!("    would configure {}", n);
            continue;
        }
        run_hook_stage(ctx, n, "post-install", &hook.post_install)?;
        println!("[+] {} configured", n);
        ran += 1;
    }
    if ran == 0 {
        println!("[=] No post-install hooks to run.");
    }
    Ok(())
}

fn name_match(terms: &[String], name: &str) -> bool {
    let hay = name.to_lowercase();
    terms.iter().all(|t| hay.contains(&t.to_lowercase()))
}

fn desc_match(terms: &[String], p: &Pkg) -> bool {
    let hay = format!("{} {}", p.name, p.desc).to_lowercase();
    terms.iter().all(|t| hay.contains(&t.to_lowercase()))
}

fn cmd_search(ctx: &Ctx, terms: &[String]) -> Res<()> {
    if terms.is_empty() {
        return Err("search: no term given".into());
    }
    let idx = load_index(ctx)?;
    let db = load_db(ctx)?;
    let mut n = 0usize;
    for p in &idx.pkgs {
        if desc_match(terms, p) {
            let mark = if db.pkgs.contains_key(&p.name) { " [installed]" } else { "" };
            println!("{}-{}{} - {}", p.name, p.version, mark, p.desc);
            n += 1;
        }
    }
    if n == 0 {
        println!("[=] no matches");
    }
    Ok(())
}

fn print_pkg(p: &Pkg, installed: Option<&Pkg>) {
    println!("{} {}", p.name, p.version);
    println!("  {}", p.desc);
    println!("  arch:        {}", p.arch);
    println!("  size:        {} download, {} installed", human(p.size), human(p.installed_size * 1024));
    if !p.pre_depends.is_empty() {
        println!("  pre-depends: {}", p.pre_depends);
    }
    if !p.depends.is_empty() {
        println!("  depends:     {}", p.depends);
    }
    if !p.provides.is_empty() {
        println!("  provides:    {}", p.provides);
    }
    if !p.conflicts.is_empty() {
        println!("  conflicts:   {}", p.conflicts);
    }
    if !p.breaks.is_empty() {
        println!("  breaks:      {}", p.breaks);
    }
    match installed {
        Some(i) => println!("  status:      installed ({})", i.version),
        None => println!("  status:      not installed"),
    }
}

fn cmd_info(ctx: &Ctx, args: &[String], flags: &[String]) -> Res<()> {
    let db = load_db(ctx)?;
    let has = |f: &str| flags.iter().any(|x| x == f);

    if has("-W") {
        if args.is_empty() {
            return Err("info -W: no path given".into());
        }
        let owners = load_owners(ctx, &db);
        for a in args {
            let key = a.trim_start_matches('/').to_string();
            match owners.get(&key) {
                Some(o) => println!("/{} is owned by {}", key, o),
                None => println!("/{}: no package owns this path", key),
            }
        }
        return Ok(());
    }

    if args.is_empty() {
        let mut names: Vec<&String> = db.pkgs.keys().collect();
        names.sort();
        for n in names {
            println!("{}-{}", n, db.pkgs[n].version);
        }
        return Ok(());
    }

    let idx = load_index(ctx).ok();
    for name in args {
        if has("-L") {
            if !db.pkgs.contains_key(name) {
                println!("{} is not installed", name);
                continue;
            }
            println!("{} contains:", name);
            for f in load_files(ctx, name) {
                if f.kind != 'd' {
                    println!("/{}", f.path);
                }
            }
            continue;
        }
        if has("-r") {
            let mut who: Vec<String> = Vec::new();
            for q in db.pkgs.values() {
                let mut entries = parse_dep_list(&q.pre_depends);
                entries.extend(parse_dep_list(&q.depends));
                if entries.iter().any(|e| e.iter().any(|d| d.name == *name)) {
                    who.push(q.name.clone());
                }
            }
            who.sort();
            println!("{} is required by: {}", name, if who.is_empty() { "(nothing)".to_string() } else { who.join(" ") });
            continue;
        }

        let available: Option<Pkg> = match &idx {
            Some(i) => i.by_name.get(name).map(|&k| i.pkgs[k].clone()),
            None => None,
        };
        let installed: Option<&Pkg> = db.pkgs.get(name);
        let shown: Option<&Pkg> = match (&available, installed) {
            (Some(a), _) => Some(a),
            (None, Some(i)) => Some(i),
            (None, None) => None,
        };
        let p = match shown {
            Some(p) => p,
            None => {
                println!("{}: unknown package", name);
                continue;
            }
        };
        if has("-R") {
            println!("{} depends on:", name);
            let mut entries = parse_dep_list(&p.pre_depends);
            entries.extend(parse_dep_list(&p.depends));
            for e in entries {
                let text: Vec<String> = e.iter().map(dep_to_string).collect();
                println!("  {}", text.join(" | "));
            }
            continue;
        }
        print_pkg(p, installed);
    }
    Ok(())
}

fn cmd_list(ctx: &Ctx, pos: &[String], flags: &[String]) -> Res<()> {
    let db = load_db(ctx)?;
    let only_installed = flags.iter().any(|f| f == "-I");
    let only_upgradable = flags.iter().any(|f| f == "-u");

    if only_installed {
        let mut names: Vec<&String> = db.pkgs.keys().collect();
        names.sort();
        for n in names {
            if pos.is_empty() || name_match(pos, n) {
                let p = &db.pkgs[n];
                println!("{}-{} {} [installed]", p.name, p.version, p.arch);
            }
        }
        return Ok(());
    }

    let idx = load_index(ctx)?;
    for p in &idx.pkgs {
        let inst = db.pkgs.get(&p.name);
        if only_upgradable {
            if let Some(i) = inst {
                if cmp_version(&p.version, &i.version) == Ordering::Greater {
                    println!("{}  {} -> {}", p.name, i.version, p.version);
                }
            }
            continue;
        }
        if !pos.is_empty() && !name_match(pos, &p.name) {
            continue;
        }
        let mark = if inst.is_some() { " [installed]" } else { "" };
        println!("{}-{} {}{}", p.name, p.version, p.arch, mark);
    }
    Ok(())
}

fn cmd_policy(ctx: &Ctx, names: &[String]) -> Res<()> {
    if names.is_empty() {
        return Err("policy: no package given".into());
    }
    let db = load_db(ctx)?;
    let idx = load_index(ctx)?;
    for n in names {
        println!("{}:", n);
        match db.pkgs.get(n) {
            Some(i) => println!("  installed: {}", i.version),
            None => println!("  installed: (none)"),
        }
        match idx.by_name.get(n) {
            Some(&k) => {
                let p = &idx.pkgs[k];
                if p.origin.contains("security") {
                    println!("  candidate: {} (security)", p.version)
                } else {
                    println!("  candidate: {}", p.version)
                }
            }
            None => println!("  candidate: (none)"),
        }
    }
    Ok(())
}

/// Compara cada fichero instalado con el SHA256 registrado al instalarlo.
fn cmd_audit(ctx: &Ctx) -> Res<()> {
    let db = load_db(ctx)?;
    let mut names: Vec<&String> = db.pkgs.keys().collect();
    names.sort();
    let mut problems = 0usize;
    for n in names {
        for f in load_files(ctx, n) {
            if !db_path_ok(&f.path) {
                continue;
            }
            let p = ctx.root.join(&f.path);
            if f.kind == 'f' {
                match sha256_file(&p) {
                    Ok(cur) => {
                        if cur != f.extra {
                            println!("M /{}  ({})", f.path, n);
                            problems += 1;
                        }
                    }
                    Err(_) => {
                        println!("D /{}  ({})", f.path, n);
                        problems += 1;
                    }
                }
            } else if f.kind == 'l' {
                match fs::read_link(&p) {
                    Ok(t) => {
                        let cur = t.to_string_lossy().to_string();
                        if cur != f.extra {
                            println!("M /{}  ({})", f.path, n);
                            problems += 1;
                        }
                    }
                    Err(_) => {
                        println!("D /{}  ({})", f.path, n);
                        problems += 1;
                    }
                }
            }
        }
    }
    if problems == 0 {
        println!("[+] audit: all installed files match their recorded checksums");
        Ok(())
    } else {
        Err(format!("audit: {} problem(s) found (M = modified, D = deleted)", problems).into())
    }
}

fn cmd_cache(ctx: &Ctx, args: &[String]) -> Res<()> {
    let sub = args.get(0).map(|s| s.as_str()).unwrap_or("size");
    let dir = ctx.cache_pkgs();
    let mut files = 0u64;
    let mut bytes = 0u64;
    if dir.is_dir() {
        for e in fs::read_dir(&dir)? {
            let e = e?;
            if e.path().is_file() {
                bytes += e.metadata().map(|m| m.len()).unwrap_or(0);
                if sub == "clean" && fs::remove_file(e.path()).is_err() {
                    continue;
                }
                files += 1;
            }
        }
    }
    match sub {
        "clean" => println!("[+] cache cleaned: {} files, {}", files, human(bytes)),
        "size" => println!("[*] cache: {} files, {}", files, human(bytes)),
        other => return Err(format!("cache: unknown subcommand '{}' (use 'clean' or 'size')", other).into()),
    }
    Ok(())
}

// ============================================================
// HOOKS DECLARATIVOS (en lugar de scripts de mantenedor)
// ============================================================
// kpkg NUNCA ejecuta preinst/postinst/prerm/postrm de un .deb: son shell arbitrario como root
// y Kyanite ni siquiera trae dpkg/perl/debconf. Lo que SI se admite es un fichero de texto,
// escrito por Kyanite (no por el paquete), con acciones de un conjunto CERRADO y confinadas
// al rootfs (sin shell, sin ejecutar nada):
//
//   <root>/usr/share/kpkg/hooks/<paquete>.hook   lo trae el rootfs de Kyanite
//   <root>/etc/kpkg/hooks.d/<paquete>.hook       local; sustituye al anterior
//
// Formato:
//   depends NOMBRE...          dependencias extra (p.ej. un "Recommends" que Debian no obliga)
//   ignore-depends NOMBRE...   dependencias que Kyanite no necesita (p.ej. debconf)
//   [pre-install] [post-install] [pre-remove] [post-remove]
//   <accion> <args>            una por linea; '#' = comentario
//
// Acciones (las mismas se usan en el CHANGELOG de las releases de Kyanite):
//   mkdir <ruta> [modo-octal]
//   symlink <destino> <ruta>        crea/reemplaza un enlace (como ln -sf)
//   link-default <destino> <ruta>   igual, pero solo si <ruta> no existe (update-alternatives light)
//   copy-default <origen> <ruta>    copia un fichero solo si <ruta> no existe
//   remove <ruta>                   borra un fichero o enlace (nunca directorios)
//   chmod <modo-octal> <ruta>
//   builtin update-ca-certificates  regenera /etc/ssl/certs (implementado en kpkg)

#[derive(Default, Clone)]
struct Hook {
    add_depends: Vec<String>,
    ignore_depends: Vec<String>,
    pre_install: Vec<String>,
    post_install: Vec<String>,
    pre_remove: Vec<String>,
    post_remove: Vec<String>,
}

const HOOK_STAGES: [&str; 4] = ["pre-install", "post-install", "pre-remove", "post-remove"];

fn action_arity(cmd: &str) -> Option<(usize, usize)> {
    match cmd {
        "mkdir" => Some((1, 2)),
        "symlink" | "link-default" | "copy-default" | "chmod" => Some((2, 2)),
        "remove" | "builtin" => Some((1, 1)),
        _ => None,
    }
}

fn validate_action(line: &str) -> Res<()> {
    let toks: Vec<&str> = line.split_whitespace().collect();
    if toks.is_empty() {
        return Err("empty action".into());
    }
    let cmd = toks[0];
    let (lo, hi) = action_arity(cmd).ok_or_else(|| format!("unknown action '{}'", cmd))?;
    let n = toks.len() - 1;
    if n < lo || n > hi {
        return Err(format!("'{}' takes {} to {} argument(s), got {}", cmd, lo, hi, n).into());
    }
    if cmd == "builtin" && toks[1] != "update-ca-certificates" {
        return Err(format!("unknown builtin '{}'", toks[1]).into());
    }
    Ok(())
}

fn parse_hook(text: &str) -> Res<Hook> {
    let mut h = Hook::default();
    let mut stage = String::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let s = line[1..line.len() - 1].trim().to_string();
            if !HOOK_STAGES.contains(&s.as_str()) {
                return Err(format!("line {}: unknown section [{}]", n + 1, s).into());
            }
            stage = s;
            continue;
        }
        if stage.is_empty() {
            let mut it = line.split_whitespace();
            let key = it.next().unwrap_or("");
            let vals: Vec<String> = it.map(|s| s.to_string()).collect();
            if vals.is_empty() || !vals.iter().all(|v| valid_pkg_name(v)) {
                return Err(format!("line {}: '{}' needs valid package names", n + 1, key).into());
            }
            match key {
                "depends" => h.add_depends.extend(vals),
                "ignore-depends" => h.ignore_depends.extend(vals),
                _ => return Err(format!("line {}: unknown directive '{}'", n + 1, key).into()),
            }
            continue;
        }
        validate_action(line).map_err(|e| format!("line {}: {}", n + 1, e))?;
        let list = match stage.as_str() {
            "pre-install" => &mut h.pre_install,
            "post-install" => &mut h.post_install,
            "pre-remove" => &mut h.pre_remove,
            _ => &mut h.post_remove,
        };
        list.push(line.to_string());
    }
    Ok(h)
}

fn load_hook(ctx: &Ctx, name: &str) -> Hook {
    let mut found = Hook::default();
    if !valid_pkg_name(name) {
        return found;
    }
    for dir in ["usr/share/kpkg/hooks", "etc/kpkg/hooks.d"] {
        let p = ctx.root.join(dir).join(format!("{}.hook", name));
        if let Ok(text) = fs::read_to_string(&p) {
            match parse_hook(&text) {
                Ok(h) => found = h,
                Err(e) => eprintln!("[!] ignoring invalid hook {}: {}", p.display(), e),
            }
        }
    }
    found
}

/// Dependencias reales de un paquete: las de Debian (Pre-Depends + Depends) menos las que su
/// hook ignora, mas las que su hook anade.
fn effective_deps(ctx: &Ctx, p: &Pkg) -> Vec<Vec<Dep>> {
    let hook = load_hook(ctx, &p.name);
    let mut entries = parse_dep_list(&p.pre_depends);
    entries.extend(parse_dep_list(&p.depends));
    if !hook.ignore_depends.is_empty() {
        entries.retain(|e| !e.iter().all(|d| hook.ignore_depends.contains(&d.name)));
    }
    for n in &hook.add_depends {
        entries.push(vec![Dep { name: n.clone(), op: None, ver: None }]);
    }
    entries
}

fn rel_in_root(ctx: &Ctx, p: &str) -> Res<(PathBuf, PathBuf)> {
    let rel = safe_rel(Path::new(p.trim_start_matches('/')))?;
    if rel.as_os_str().is_empty() {
        return Err(format!("invalid path '{}'", p).into());
    }
    let abs = ctx.root.join(&rel);
    Ok((rel, abs))
}

fn parse_mode(s: &str) -> Res<u32> {
    let m = u32::from_str_radix(s, 8).map_err(|_| format!("invalid octal mode '{}'", s))?;
    if m > 0o777 {
        return Err(format!("mode '{}' not allowed (no setuid/setgid/sticky)", s).into());
    }
    Ok(m)
}

fn run_action(ctx: &Ctx, line: &str) -> Res<()> {
    validate_action(line)?;
    let toks: Vec<&str> = line.split_whitespace().collect();
    let a = &toks[1..];
    match toks[0] {
        "mkdir" => {
            let (_, abs) = rel_in_root(ctx, a[0])?;
            fs::create_dir_all(&abs)?;
            ensure_inside(&ctx.root, &abs)?;
            if let Some(m) = a.get(1) {
                fs::set_permissions(&abs, fs::Permissions::from_mode(parse_mode(m)?))?;
            }
        }
        "symlink" | "link-default" => {
            let target = Path::new(a[0]);
            let (rel, abs) = rel_in_root(ctx, a[1])?;
            let depth = rel.components().count().saturating_sub(1);
            if !symlink_target_ok(depth, target) {
                return Err(format!("symlink target '{}' would escape the rootfs", a[0]).into());
            }
            if let Some(parent) = abs.parent() {
                fs::create_dir_all(parent)?;
                ensure_inside(&ctx.root, parent)?;
            }
            if let Ok(m) = fs::symlink_metadata(&abs) {
                if toks[0] == "link-default" {
                    return Ok(());
                }
                if m.is_dir() {
                    return Err(format!("/{} is a directory", rel.display()).into());
                }
                fs::remove_file(&abs)?;
            }
            std::os::unix::fs::symlink(target, &abs)?;
        }
        "copy-default" => {
            let (_, src) = rel_in_root(ctx, a[0])?;
            let (_, dst) = rel_in_root(ctx, a[1])?;
            if fs::symlink_metadata(&dst).is_ok() {
                return Ok(());
            }
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
                ensure_inside(&ctx.root, parent)?;
            }
            ensure_inside(&ctx.root, &src)?;
            if !fs::symlink_metadata(&src)?.is_file() {
                return Err("copy-default: the source must be a regular file".into());
            }
            fs::copy(&src, &dst)?;
        }
        "remove" => {
            let (_, abs) = rel_in_root(ctx, a[0])?;
            if let Some(parent) = abs.parent() {
                ensure_inside(&ctx.root, parent)?;
            }
            match fs::symlink_metadata(&abs) {
                Ok(m) if m.is_dir() => return Err("remove: refusing to remove a directory".into()),
                Ok(_) => fs::remove_file(&abs)?,
                Err(_) => {}
            }
        }
        "chmod" => {
            let mode = parse_mode(a[0])?;
            let (_, abs) = rel_in_root(ctx, a[1])?;
            ensure_inside(&ctx.root, &abs)?;
            if fs::symlink_metadata(&abs)?.file_type().is_symlink() {
                return Err("chmod: refusing to follow a symlink".into());
            }
            fs::set_permissions(&abs, fs::Permissions::from_mode(mode))?;
        }
        "builtin" => builtin_update_ca_certificates(ctx)?,
        other => return Err(format!("unknown action '{}'", other).into()),
    }
    Ok(())
}

fn run_hook_stage(ctx: &Ctx, pkg: &str, stage: &str, actions: &[String]) -> Res<()> {
    for a in actions {
        run_action(ctx, a).map_err(|e| format!("{} hook of {} failed at '{}': {}", stage, pkg, a, e))?;
    }
    Ok(())
}

// ---- update-ca-certificates nativo ----------------------------------------
// Lo que hace el postinst de Debian (sin perl, openssl ni debconf): lee
// /etc/ca-certificates.conf (lo crea con todos los certificados si no existe), enlaza
// /etc/ssl/certs/<nombre>.pem y construye /etc/ssl/certs/ca-certificates.crt, que es el
// bundle que usan curl, wget, git, python... No genera los enlaces por hash (<hash>.0) que solo
// necesitan los programas que usan CApath.

fn collect_crt(base: &Path, dir: &Path, out: &mut Vec<String>) {
    let rd = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for e in rd.flatten() {
        let p = e.path();
        let ft = match e.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if ft.is_dir() {
            collect_crt(base, &p, out);
        } else if ft.is_file() && p.extension().and_then(|x| x.to_str()) == Some("crt") {
            if let Ok(r) = p.strip_prefix(base) {
                if let Some(s) = r.to_str() {
                    out.push(s.to_string());
                }
            }
        }
    }
}

fn add_cert(
    base: &Path,
    rel: &str,
    target_prefix: &str,
    certs_dir: &Path,
    bundle: &mut Vec<u8>,
    seen: &mut HashSet<String>,
) -> Res<bool> {
    let src = base.join(rel);
    if !fs::symlink_metadata(&src)?.is_file() {
        return Ok(false);
    }
    let data = fs::read(&src)?;
    if !String::from_utf8_lossy(&data).contains("-----BEGIN CERTIFICATE-----") {
        return Ok(false);
    }
    bundle.extend_from_slice(&data);
    if !data.ends_with(b"\n") {
        bundle.push(b'\n');
    }
    let file = Path::new(rel).file_name().and_then(|f| f.to_str()).unwrap_or("");
    let stem = file.strip_suffix(".crt").unwrap_or(file);
    let link_name = format!("{}.pem", stem);
    if !stem.is_empty() && seen.insert(link_name.clone()) {
        let link = certs_dir.join(&link_name);
        match fs::symlink_metadata(&link) {
            Ok(m) if !m.file_type().is_symlink() => {} // fichero real del usuario: no se toca
            _ => {
                let _ = fs::remove_file(&link);
                std::os::unix::fs::symlink(format!("{}{}", target_prefix, rel), &link)?;
            }
        }
    }
    Ok(true)
}

fn builtin_update_ca_certificates(ctx: &Ctx) -> Res<()> {
    let share = ctx.root.join("usr/share/ca-certificates");
    let local = ctx.root.join("usr/local/share/ca-certificates");
    let certs_dir = ctx.root.join("etc/ssl/certs");
    let conf = ctx.root.join("etc/ca-certificates.conf");
    let bundle_path = certs_dir.join("ca-certificates.crt");

    fs::create_dir_all(&certs_dir)?;
    ensure_inside(&ctx.root, &certs_dir)?;

    let mut shipped: Vec<String> = Vec::new();
    collect_crt(&share, &share, &mut shipped);
    shipped.sort();

    if !conf.exists() && !shipped.is_empty() {
        let mut t = String::new();
        for s in &shipped {
            t.push_str(s);
            t.push('\n');
        }
        write_atomic(&conf, t.as_bytes())?;
    }
    // "!ruta" en el .conf = certificado desactivado por el usuario
    let mut selected: Vec<String> = Vec::new();
    if let Ok(text) = fs::read_to_string(&conf) {
        for line in text.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') || l.starts_with('!') {
                continue;
            }
            if shipped.iter().any(|s| s == l) {
                selected.push(l.to_string());
            }
        }
    }
    let mut locals: Vec<String> = Vec::new();
    collect_crt(&local, &local, &mut locals);
    locals.sort();

    // enlaces .pem que kpkg creo en una ejecucion anterior
    if let Ok(rd) = fs::read_dir(&certs_dir) {
        for e in rd.flatten() {
            let p = e.path();
            let is_link = e.file_type().map(|t| t.is_symlink()).unwrap_or(false);
            if !is_link || p.extension().and_then(|x| x.to_str()) != Some("pem") {
                continue;
            }
            if let Ok(t) = fs::read_link(&p) {
                let t = t.to_string_lossy().to_string();
                if t.starts_with("/usr/share/ca-certificates/") || t.starts_with("/usr/local/share/ca-certificates/") {
                    let _ = fs::remove_file(&p);
                }
            }
        }
    }

    let mut bundle: Vec<u8> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut count = 0usize;
    for rel in &selected {
        if add_cert(&share, rel, "/usr/share/ca-certificates/", &certs_dir, &mut bundle, &mut seen)? {
            count += 1;
        }
    }
    for rel in &locals {
        if add_cert(&local, rel, "/usr/local/share/ca-certificates/", &certs_dir, &mut bundle, &mut seen)? {
            count += 1;
        }
    }
    if count == 0 {
        let _ = fs::remove_file(&bundle_path);
        println!("[i] ca-certificates: no certificates installed; removed the bundle");
    } else {
        write_atomic(&bundle_path, &bundle)?;
        fs::set_permissions(&bundle_path, fs::Permissions::from_mode(0o644))?;
        println!("[+] ca-certificates: {} certificates in /etc/ssl/certs/ca-certificates.crt", count);
    }
    Ok(())
}

// ============================================================
// ACTUALIZACION DEL SISTEMA BASE DE KYANITE
// ============================================================
// `kpkg update`  : mira la ultima RELEASE de GitHub (solo tags vX.Y.Z; las builds de desarrollo
//                  con version de commit nunca se actualizan) y deja una marca si hay una nueva.
// `kpkg upgrade` : descarga el rootfs, comprueba firma ed25519 + SHA256, lo extrae en un
//                  directorio de staging DENTRO del mismo sistema de ficheros y lo aplica:
//
//   1. Sincronizacion de ficheros por MANIFEST (sin que el dev tenga que listar nada): cada
//      rootfs trae usr/share/kyanite/MANIFEST (sha256 de cada fichero). Se compara el manifest
//      instalado con el nuevo y se escribe/borra solo lo que cambio. Regla de 3 vias: si el
//      fichero local es igual al antiguo (o ya igual al nuevo) se actualiza; si el usuario lo
//      modifico se conserva y la version nueva queda como <fichero>.kyanite-new.
//      Ficheros que pertenecen a paquetes de kpkg nunca se tocan.
//   2. Operaciones del CHANGELOG (usr/share/kyanite/CHANGELOG) de cada version intermedia,
//      para lo que un cambio de ficheros no expresa (p.ej. "+vim" / "-tiny-vim").
//   3. Orden seguro: librerias -> binarios -> /etc -> kpkg -> init -> MANIFEST -> os-release.
//      Cada fichero se escribe en un temporal y se mueve con rename() (atomico; los procesos en
//      marcha siguen con su inodo viejo). La version (os-release) se escribe la ULTIMA: si algo
//      se interrumpe, volver a ejecutar `kpkg upgrade` retoma donde se quedo (es idempotente).

fn os_release_field(root: &Path, key: &str) -> Option<String> {
    let t = fs::read_to_string(root.join("etc/os-release")).ok()?;
    let prefix = format!("{}=", key);
    for l in t.lines() {
        if let Some(v) = l.strip_prefix(prefix.as_str()) {
            return Some(v.trim().trim_matches('"').to_string());
        }
    }
    None
}

/// Ultima release publicada: se lee del redireccionamiento de /releases/latest
/// (sin API, sin JSON, sin limite de peticiones).
fn latest_release_tag() -> Res<String> {
    let url = format!("{}/{}/releases/latest", GITHUB, KYANITE_REPO);
    let resp = agent().get(&url).call()?;
    let final_url = resp.get_url().to_string();
    drop(resp);
    let tag = match final_url.split_once("/releases/tag/") {
        Some((_, t)) => t.split(|c: char| c == '?' || c == '#' || c == '/').next().unwrap_or("").to_string(),
        None => return Err("the project has no published releases".into()),
    };
    if !tag.starts_with('v') || parse_release_version(&tag).is_none() {
        return Err(format!("latest release tag '{}' is not a vX.Y.Z version", tag).into());
    }
    Ok(tag)
}

fn check_base_update(ctx: &Ctx) {
    let cur = match os_release_field(&ctx.root, "VERSION_ID") {
        Some(v) => v,
        None => return, // no es un rootfs de Kyanite
    };
    let cur_v = match parse_release_version(&cur) {
        Some(v) => v,
        None => {
            println!("[=] Kyanite base: development build ({}); base updates only apply to release versions", cur);
            let _ = fs::remove_file(ctx.base_pending_path());
            return;
        }
    };
    match latest_release_tag() {
        Ok(tag) => match parse_release_version(&tag) {
            Some(new_v) if new_v > cur_v => {
                let _ = write_atomic(&ctx.base_pending_path(), format!("{}\n", tag).as_bytes());
                println!("[+] Kyanite {} is available (you have v{}): run 'kpkg upgrade' to apply it", tag, cur);
            }
            _ => {
                let _ = fs::remove_file(ctx.base_pending_path());
                println!("[=] Kyanite base is up to date (v{})", cur);
            }
        },
        Err(e) => println!("[!] could not check for Kyanite base updates: {}", e),
    }
}

fn cmd_base_status(ctx: &Ctx) -> Res<()> {
    let cur = os_release_field(&ctx.root, "VERSION_ID").unwrap_or_else(|| "unknown".to_string());
    println!("Kyanite base version: {}", cur);
    if parse_release_version(&cur).is_none() {
        println!("  (development build: base updates only apply to release versions)");
    }
    match fs::read_to_string(ctx.base_pending_path()) {
        Ok(t) => println!("  update available: {} (run 'kpkg upgrade')", t.trim()),
        Err(_) => println!("  no base update pending (run 'kpkg update' to check)"),
    }
    Ok(())
}

/// Firma ed25519 sobre el fichero .sha256 de la release. Claves publicas aceptadas (hex de
/// 32 bytes, una por linea; varias = rotacion de claves): <root>/etc/kpkg/kyanite-release.pub
fn verify_release_sig(ctx: &Ctx, msg: &[u8], sig: &[u8]) -> Res<()> {
    let text = fs::read_to_string(ctx.root.join("etc/kpkg/kyanite-release.pub"))
        .map_err(|_| "etc/kpkg/kyanite-release.pub not found: cannot verify the release signature")?;
    let signature = Signature::from_slice(sig).map_err(|e| format!("malformed release signature: {}", e))?;
    let mut tried = 0usize;
    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let raw = match hex::decode(l) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let arr: [u8; 32] = match raw.try_into() {
            Ok(a) => a,
            Err(_) => continue,
        };
        let vk = match VerifyingKey::from_bytes(&arr) {
            Ok(k) => k,
            Err(_) => continue,
        };
        tried += 1;
        if vk.verify_strict(msg, &signature).is_ok() {
            return Ok(());
        }
    }
    if tried == 0 {
        return Err("no valid release public key in etc/kpkg/kyanite-release.pub".into());
    }
    Err("SECURITY ALERT: the release signature is invalid; refusing to install this update".into())
}

fn base_asset_arch(arch: &str) -> Res<&'static str> {
    match arch {
        "arm64" => Ok("aarch64"),
        "amd64" => Ok("x86_64"),
        o => Err(format!("there is no Kyanite release for architecture '{}'", o).into()),
    }
}

/// Descarga + verifica (firma, SHA256) + extrae al staging. Devuelve el directorio de staging.
fn base_download(ctx: &Ctx, tag: &str) -> Res<PathBuf> {
    let asset = format!("kyanite-rootfs-{}.tar.xz", base_asset_arch(&ctx.arch)?);
    let base = format!("{}/{}/releases/download/{}", GITHUB, KYANITE_REPO, tag);
    println!("[*] Fetching release checksum and signature");
    let sha_txt = http_get(&format!("{}/{}.sha256", base, asset), MAX_SMALL)?;
    let sig = http_get(&format!("{}/{}.sha256.sig", base, asset), MAX_SMALL)?;
    verify_release_sig(ctx, &sha_txt, &sig)?;
    println!("[+] Release signature OK");

    let sha_str = String::from_utf8(sha_txt).map_err(|_| "checksum file is not valid UTF-8")?;
    let mut it = sha_str.split_whitespace();
    let want = it.next().unwrap_or("").to_lowercase();
    if !valid_sha256(&want) {
        return Err("malformed checksum file".into());
    }
    if let Some(name) = it.next() {
        if name.trim_start_matches('*') != asset {
            return Err(format!("the signed checksum is for '{}', not '{}'", name, asset).into());
        }
    }

    let tarball = ctx.cache_base().join(format!("{}-{}", tag, asset));
    println!("[*] Downloading {} ({})", asset, tag);
    download_verified(&format!("{}/{}", base, asset), &tarball, 0, &want)?;

    let stage = ctx.staging_dir().join(tag);
    if stage.exists() {
        fs::remove_dir_all(&stage)?;
    }
    fs::create_dir_all(&stage)?;
    let mut sctx = ctx.clone();
    sctx.root = stage.clone();
    sctx.force = true;
    let mut ex = Extracted { files: Vec::new(), pending: Vec::new(), links: Vec::new() };
    let file = fs::File::open(&tarball)?;
    let mut a = tar::Archive::new(XzDecoder::new(file));
    let fake = Pkg::default();
    println!("[*] Extracting to {}", stage.display());
    if let Err(e) = extract_entries(&sctx, &fake, &mut a, &HashMap::new(), &HashMap::new(), &mut ex) {
        cleanup_pending(&ex);
        return Err(e);
    }
    if let Err(e) = commit(&ex) {
        cleanup_pending(&ex);
        return Err(e);
    }
    Ok(stage)
}

fn base_upgrade(ctx: &Ctx) -> Res<()> {
    let tag = match fs::read_to_string(ctx.base_pending_path()) {
        Ok(t) => t.trim().to_string(),
        Err(_) => return Ok(()),
    };
    let new_v = match parse_release_version(&tag) {
        Some(v) if tag.starts_with('v') => v,
        _ => {
            let _ = fs::remove_file(ctx.base_pending_path());
            return Err("corrupt base-update marker removed; run 'kpkg update' again".into());
        }
    };
    let cur = os_release_field(&ctx.root, "VERSION_ID").unwrap_or_default();
    let cur_v = match parse_release_version(&cur) {
        Some(v) => v,
        None => {
            let _ = fs::remove_file(ctx.base_pending_path());
            return Ok(());
        }
    };
    if new_v <= cur_v {
        let _ = fs::remove_file(ctx.base_pending_path());
        return Ok(());
    }
    println!("[*] Kyanite base update: v{} -> {}", cur, tag);
    if ctx.simulate {
        println!("[=] --simulate: nothing was downloaded or changed.");
        return Ok(());
    }
    if !confirm(ctx, "Download and apply this Kyanite update?") {
        return Err("aborted by user".into());
    }
    let stage = base_download(ctx, &tag)?;

    // Lo aplica el kpkg NUEVO (el del release ya verificado): asi entiende las operaciones de su
    // propio CHANGELOG aunque el kpkg instalado sea mas viejo. Es un binario estatico (musl).
    let staged_kpkg = stage.join("usr/bin/kpkg");
    let mut outcome: Option<bool> = None;
    if staged_kpkg.is_file() {
        let mut cmd = Command::new(&staged_kpkg);
        cmd.arg("--root").arg(&ctx.root).arg("--arch").arg(&ctx.arch).arg("--yes");
        if ctx.force {
            cmd.arg("--force");
        }
        cmd.arg("base-apply").arg(&stage);
        match cmd.status() {
            Ok(st) => outcome = Some(st.success()),
            Err(e) => println!("[!] could not run the new kpkg ({}); applying with this version", e),
        }
    }
    match outcome {
        Some(true) => {}
        Some(false) => {
            return Err("the base update failed (see above). Nothing is lost: fix the cause and run 'kpkg upgrade' again; it resumes where it stopped".into());
        }
        None => base_apply(ctx, &stage)?,
    }
    let _ = fs::remove_file(ctx.base_pending_path());
    let _ = fs::remove_dir_all(&stage);
    let _ = fs::remove_dir_all(ctx.cache_base());
    println!("[+] Kyanite is now at {}. Restart your session to run the new init.", tag);
    Ok(())
}

/// Estado actual de una ruta del rootfs: (tipo, sha256 | destino-del-enlace).
fn fs_state(root: &Path, rel: &str) -> Option<(char, String)> {
    let p = root.join(rel);
    let m = fs::symlink_metadata(&p).ok()?;
    let ft = m.file_type();
    if ft.is_symlink() {
        Some(('l', fs::read_link(&p).ok()?.to_string_lossy().to_string()))
    } else if ft.is_file() {
        Some(('f', sha256_file(&p).ok()?))
    } else if ft.is_dir() {
        Some(('d', String::new()))
    } else {
        Some(('?', String::new()))
    }
}

fn parse_manifest(text: &str) -> Res<Vec<FileRec>> {
    let mut out: Vec<FileRec> = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let r = FileRec::from_line(line).ok_or_else(|| format!("MANIFEST line {}: malformed", n + 1))?;
        if r.path.is_empty() || !db_path_ok(&r.path) {
            return Err(format!("MANIFEST line {}: unsafe path '{}'", n + 1, r.path).into());
        }
        match r.kind {
            'f' => {
                if !valid_sha256(&r.extra) {
                    return Err(format!("MANIFEST line {}: bad sha256", n + 1).into());
                }
            }
            'l' => {
                let depth = Path::new(&r.path).components().count().saturating_sub(1);
                if !symlink_target_ok(depth, Path::new(&r.extra)) {
                    return Err(format!("MANIFEST line {}: symlink escapes the rootfs", n + 1).into());
                }
            }
            'd' => {}
            _ => return Err(format!("MANIFEST line {}: unknown entry type", n + 1).into()),
        }
        if r.path == "usr/share/kyanite/MANIFEST" {
            continue;
        }
        out.push(r);
    }
    Ok(out)
}

// ---- CHANGELOG -------------------------------------------------------------

struct ChangeSection {
    ver: (u64, u64, u64),
    lines: Vec<String>,
}

fn split_assume(spec: &str) -> (&str, Option<&str>) {
    match spec.split_once('=') {
        Some((n, v)) => (n.trim(), Some(v.trim())),
        None => (spec.trim(), None),
    }
}

fn validate_change_op(line: &str) -> Res<()> {
    if let Some(spec) = line.strip_prefix('+') {
        let (n, v) = split_assume(spec);
        if !valid_pkg_name(n) || !v.map(valid_version).unwrap_or(true) {
            return Err(format!("bad package spec '{}'", spec).into());
        }
        return Ok(());
    }
    if let Some(n) = line.strip_prefix('-') {
        if !valid_pkg_name(n.trim()) {
            return Err(format!("bad package name '{}'", n.trim()).into());
        }
        return Ok(());
    }
    if line == "note" || line.starts_with("note ") {
        return Ok(());
    }
    validate_action(line)
}

fn parse_changelog(text: &str) -> Res<Vec<ChangeSection>> {
    let mut out: Vec<ChangeSection> = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(v) = line.strip_prefix("==") {
            let ver = parse_release_version(v.trim())
                .ok_or_else(|| format!("CHANGELOG line {}: bad version header '{}'", n + 1, line))?;
            out.push(ChangeSection { ver, lines: Vec::new() });
            continue;
        }
        match out.last_mut() {
            Some(s) => {
                validate_change_op(line).map_err(|e| format!("CHANGELOG line {}: {}", n + 1, e))?;
                s.lines.push(line.to_string());
            }
            None => {
                return Err(format!("CHANGELOG line {}: operation before the first '== vX.Y.Z' header", n + 1).into());
            }
        }
    }
    out.sort_by(|a, b| a.ver.cmp(&b.ver));
    Ok(out)
}

fn save_assumed(ctx: &Ctx, list: &[(String, Option<String>)]) -> Res<()> {
    let mut s = String::new();
    for (n, v) in list {
        match v {
            Some(v) => s.push_str(&format!("{}={}\n", n, v)),
            None => {
                s.push_str(n);
                s.push('\n');
            }
        }
    }
    write_atomic(&ctx.root.join("etc/kpkg/assume-installed"), s.as_bytes())
}

/// Solo escribe si algo cambia, para no desviar el fichero del MANIFEST sin necesidad.
fn assume_add(ctx: &Ctx, spec: &str) -> Res<()> {
    let (n, v) = split_assume(spec);
    let want = (n.to_string(), v.map(|s| s.to_string()));
    let mut list = load_assumed(ctx);
    if list.contains(&want) {
        return Ok(());
    }
    list.retain(|(x, _)| x != n);
    list.push(want);
    save_assumed(ctx, &list)
}

fn assume_del(ctx: &Ctx, name: &str) -> Res<()> {
    let mut list = load_assumed(ctx);
    let before = list.len();
    list.retain(|(x, _)| x != name);
    if list.len() == before {
        return Ok(());
    }
    save_assumed(ctx, &list)
}

fn apply_change_op(ctx: &Ctx, line: &str, notes: &mut Vec<String>) -> Res<()> {
    if let Some(spec) = line.strip_prefix('+') {
        return assume_add(ctx, spec.trim());
    }
    if let Some(n) = line.strip_prefix('-') {
        return assume_del(ctx, n.trim());
    }
    if line == "note" || line.starts_with("note ") {
        notes.push(line[4..].trim().to_string());
        return Ok(());
    }
    run_action(ctx, line)
}

fn base_rank(p: &str) -> u8 {
    match p {
        "usr/bin/kpkg" => 3,
        "sbin/init" => 4,
        _ => {
            if p.starts_with("lib") || p.starts_with("usr/lib") {
                0
            } else if p.starts_with("etc/") || p.starts_with("root/") {
                2
            } else {
                1
            }
        }
    }
}

/// Coloca una entrada del staging en el rootfs (temporal + rename atomico).
/// `side` = true guarda la version nueva como <ruta>.kyanite-new (el original se respeta).
fn place_base_entry(ctx: &Ctx, stage: &Path, rec: &FileRec, side: bool) -> Res<()> {
    let rel = safe_rel(Path::new(&rec.path))?;
    let mut dest = ctx.root.join(&rel);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
        ensure_inside(&ctx.root, parent)?;
    }
    if side {
        let n = dest.file_name().ok_or("invalid path")?.to_string_lossy().to_string();
        dest = dest.with_file_name(format!("{}.kyanite-new", n));
    }
    let name = dest.file_name().ok_or("invalid path")?.to_string_lossy().to_string();
    let tmp = dest.with_file_name(format!("{}.kyanite-tmp", name));
    let _ = fs::remove_file(&tmp);
    match rec.kind {
        'f' => {
            fs::copy(stage.join(&rel), &tmp)?;
            if sha256_file(&tmp)? != rec.extra {
                let _ = fs::remove_file(&tmp);
                return Err(format!("integrity check failed for /{}", rec.path).into());
            }
        }
        'l' => std::os::unix::fs::symlink(&rec.extra, &tmp)?,
        _ => return Err("unexpected manifest entry".into()),
    }
    if let Ok(m) = fs::symlink_metadata(&dest) {
        if m.is_dir() {
            let _ = fs::remove_file(&tmp);
            return Err(format!("/{} is a directory", rec.path).into());
        }
    }
    fs::rename(&tmp, &dest)?;
    Ok(())
}

fn base_apply(ctx: &Ctx, stage: &Path) -> Res<()> {
    // ---- 1. leer y validar todo ANTES de tocar nada --------------------------------
    let new_text = fs::read_to_string(stage.join("usr/share/kyanite/MANIFEST"))
        .map_err(|_| "the release has no usr/share/kyanite/MANIFEST")?;
    let new_recs = parse_manifest(&new_text)?;
    let new_map: HashMap<&str, &FileRec> = new_recs.iter().map(|r| (r.path.as_str(), r)).collect();
    for req in ["sbin/init", "usr/bin/kpkg", "bin/brush", "etc/os-release"] {
        match new_map.get(req) {
            Some(r) if r.kind != 'd' && stage.join(req).exists() => {}
            _ => return Err(format!("the release is missing a core file: /{}", req).into()),
        }
    }
    let old_recs: Vec<FileRec> = match fs::read_to_string(ctx.root.join("usr/share/kyanite/MANIFEST")) {
        Ok(t) => parse_manifest(&t).unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let legacy = old_recs.is_empty();
    let old_map: HashMap<&str, &FileRec> = old_recs.iter().map(|r| (r.path.as_str(), r)).collect();

    let cur_id = os_release_field(&ctx.root, "VERSION_ID").unwrap_or_default();
    let new_id = os_release_field(stage, "VERSION_ID").unwrap_or_default();
    let cur_v = parse_release_version(&cur_id).ok_or("this system is not running a Kyanite release version")?;
    let new_v = parse_release_version(&new_id).ok_or("the staged release has no valid VERSION_ID")?;
    if new_v <= cur_v {
        return Err(format!("refusing to apply v{} over v{} (not newer)", new_id, cur_id).into());
    }
    let ch_text = fs::read_to_string(stage.join("usr/share/kyanite/CHANGELOG")).unwrap_or_default();
    let sections = parse_changelog(&ch_text)?;
    let ops: Vec<String> = sections
        .iter()
        .filter(|s| s.ver > cur_v && s.ver <= new_v)
        .flat_map(|s| s.lines.clone())
        .collect();

    // ---- 2. plan -------------------------------------------------------------------
    let db = load_db(ctx)?;
    let owners = load_owners(ctx, &db);
    let mut mkdirs: Vec<&FileRec> = Vec::new();
    let mut writes: Vec<&FileRec> = Vec::new();
    let mut keeps: Vec<&FileRec> = Vec::new();
    let mut owned = 0usize;
    let mut blocked: Vec<&str> = Vec::new();
    let mut os_rec: Option<&FileRec> = None;

    for rec in &new_recs {
        if rec.kind == 'd' {
            mkdirs.push(rec);
            continue;
        }
        if rec.path == "etc/os-release" {
            os_rec = Some(rec);
            continue;
        }
        if owners.contains_key(&rec.path) {
            owned += 1;
            continue;
        }
        let cur = fs_state(&ctx.root, &rec.path);
        if let Some((k, e)) = &cur {
            if *k == rec.kind && *e == rec.extra {
                continue; // ya esta al dia (p.ej. reanudando una actualizacion interrumpida)
            }
        }
        // Sin cambios aguas arriba (mismo contenido en la release vieja y la nueva): lo que haya en
        // local, modificado por ti o por el entorno (p.ej. /etc/resolv.conf), se respeta en silencio.
        if cur.is_some() && !ctx.force {
            if let Some(o) = old_map.get(rec.path.as_str()) {
                if o.kind == rec.kind && o.extra == rec.extra {
                    continue;
                }
            }
        }
        match &cur {
            None => writes.push(rec),
            Some((k, _)) if *k == 'd' || *k == '?' => blocked.push(rec.path.as_str()),
            Some((k, e)) => {
                let untouched = match old_map.get(rec.path.as_str()) {
                    Some(o) => o.kind == *k && o.extra == *e,
                    None => false,
                };
                let core = !(rec.path.starts_with("etc/") || rec.path.starts_with("root/"));
                if untouched || ctx.force || (legacy && core) {
                    writes.push(rec);
                } else {
                    keeps.push(rec);
                }
            }
        }
    }
    let mut removals: Vec<&FileRec> = Vec::new();
    let mut kept_old: Vec<&str> = Vec::new();
    let mut old_dirs: Vec<&FileRec> = Vec::new();
    for rec in &old_recs {
        if new_map.contains_key(rec.path.as_str()) {
            continue;
        }
        if rec.kind == 'd' {
            old_dirs.push(rec);
            continue;
        }
        if owners.contains_key(&rec.path) {
            continue;
        }
        match fs_state(&ctx.root, &rec.path) {
            None => {}
            Some((k, e)) if k == rec.kind && e == rec.extra => removals.push(rec),
            Some(_) => kept_old.push(rec.path.as_str()),
        }
    }

    println!(
        "[*] Base changes: {} to write, {} to remove, {} kept (modified locally), {} owned by kpkg packages (skipped), {} operation(s)",
        writes.len(),
        removals.len(),
        keeps.len(),
        owned,
        ops.len()
    );
    if legacy {
        println!("[i] This system has no previous manifest: removed files cannot be detected this time");
    }
    for r in &keeps {
        println!("    keeping your modified /{} (new version -> /{}.kyanite-new)", r.path, r.path);
    }
    for p in &kept_old {
        println!("    keeping /{} (removed upstream, but you modified it)", p);
    }
    for p in &blocked {
        println!("[!] /{} is a directory here but a file in the release; skipped", p);
    }
    if ctx.simulate {
        println!("[=] --simulate: nothing was changed.");
        return Ok(());
    }

    // ---- 3. aplicar ----------------------------------------------------------------
    for rec in &mkdirs {
        let rel = safe_rel(Path::new(&rec.path))?;
        let dir = ctx.root.join(&rel);
        if fs::symlink_metadata(&dir).is_err() {
            fs::create_dir_all(&dir)?;
            ensure_inside(&ctx.root, &dir)?;
            if let Ok(m) = fs::metadata(stage.join(&rel)) {
                let _ = fs::set_permissions(&dir, m.permissions());
            }
        }
    }
    let mut ordered: Vec<&FileRec> = writes.clone();
    ordered.sort_by(|a, b| base_rank(&a.path).cmp(&base_rank(&b.path)).then_with(|| a.path.cmp(&b.path)));
    for rec in &ordered {
        place_base_entry(ctx, stage, rec, false)?;
    }
    for rec in &keeps {
        if let Err(e) = place_base_entry(ctx, stage, rec, true) {
            println!("[!] could not save /{}.kyanite-new: {}", rec.path, e);
        }
    }
    for rec in &removals {
        let p = ctx.root.join(&rec.path);
        if let Some(parent) = p.parent() {
            if ensure_inside(&ctx.root, parent).is_err() {
                continue;
            }
        }
        let _ = fs::remove_file(&p);
    }
    old_dirs.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    for d in &old_dirs {
        if Path::new(&d.path).components().count() > 2 {
            let _ = fs::remove_dir(ctx.root.join(&d.path)); // solo si queda vacio
        }
    }

    let mut notes: Vec<String> = Vec::new();
    for op in &ops {
        if let Err(e) = apply_change_op(ctx, op, &mut notes) {
            if ctx.force {
                println!("[!] CHANGELOG operation '{}' failed (ignored by --force): {}", op, e);
            } else {
                return Err(format!(
                    "CHANGELOG operation '{}' failed: {}. The version was NOT switched; fix the cause and run 'kpkg upgrade' again",
                    op, e
                )
                .into());
            }
        }
    }

    // marca de version: lo ultimo
    write_atomic(&ctx.root.join("usr/share/kyanite/MANIFEST"), new_text.as_bytes())?;
    if let Some(rec) = os_rec {
        place_base_entry(ctx, stage, rec, false)?;
    }
    for n in &notes {
        println!("[i] {}", n);
    }
    println!("[+] Base system updated to v{}", new_id);
    Ok(())
}

/// ¿Se invoco como apt / apt-get / apt-cache? (enlace simbolico a kpkg, como busybox) o
/// KPKG_SYNTAX=apt. OJO: un `alias apt='kpkg'` del shell NO cambia argv[0]; para eso usa
/// `alias apt='KPKG_SYNTAX=apt kpkg'` o, mejor, el enlace /usr/bin/apt que trae el rootfs.
fn detect_apt_mode() -> bool {
    if env::var("KPKG_SYNTAX").map(|v| v == "apt").unwrap_or(false) {
        return true;
    }
    let argv0 = env::args().next().unwrap_or_default();
    let base = Path::new(&argv0)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    matches!(base.as_str(), "apt" | "apt-get" | "apt-cache")
}

fn print_help(apt: bool) {
    if apt {
        print_help_apt();
    } else {
        print_help_apk();
    }
}

fn print_help_apt() {
    println!("apt (kpkg {}) - Kyanite package manager, apt-compatible interface", VERSION);
    println!();
    println!("Usage: apt [options] command");
    println!();
    println!("Most used commands:");
    println!("  update                 Retrieve new lists of packages (verified OpenPGP signatures)");
    println!("                         and check for a new Kyanite release");
    println!("  upgrade                Upgrade the Kyanite base (if a release is pending) and all packages");
    println!("  full-upgrade           Same as upgrade (also: dist-upgrade)");
    println!("  install <pkg>...       Install packages and their dependencies");
    println!("  reinstall <pkg>...     Reinstall packages");
    println!("  remove <pkg>...        Remove packages");
    println!("  purge <pkg>...         Remove packages and their modified configuration files");
    println!("  autoremove             Remove dependencies that are no longer needed");
    println!("  search <term>...       Search package names and descriptions");
    println!("  show <pkg>...          Show package details");
    println!("  list [pattern]         List packages (--installed, --upgradable)");
    println!("  policy <pkg>...        Show installed and candidate versions");
    println!("  depends <pkg>...       Show the dependencies of a package");
    println!("  rdepends <pkg>...      Show which installed packages require it");
    println!("  clean                  Delete downloaded .deb files");
    println!();
    println!("Options:");
    println!("  -y, --yes              Do not ask for confirmation");
    println!("  -s, --simulate         Show what would be done (also --dry-run)");
    println!("  -f, --force            Overwrite unowned files / ignore reverse dependencies");
    println!("  -q, --quiet            Accepted for compatibility");
    println!("      --installed        With 'list': only installed packages");
    println!("      --upgradable       With 'list': only upgradable packages");
    println!("      --no-security      Do not use the <suite>-security repository (not recommended)");
    println!("  -V, --version          Show the version");
    println!("  -h, --help             Show this help");
    println!();
    println!("Differences from real apt: no maintainer scripts are run (declarative hooks are");
    println!("used instead), Recommends/Suggests are never installed, and there is no dpkg.");
    println!("The native apk-style commands (add, del, info, fix, audit, base, ...) also work;");
    println!("run 'kpkg --help' to see them.");
}

fn print_help_apk() {
    println!("kpkg {} - Kyanite package manager (apk-style CLI over Debian repositories)", VERSION);
    println!();
    println!("Usage: kpkg [options] <command> [arguments]");
    println!();
    println!("Commands:");
    println!("  update                 Fetch the package indexes (OpenPGP-verified: stable + updates +");
    println!("                         security) and check for a new Kyanite release");
    println!("  add <pkg>...           Install packages and their dependencies");
    println!("  del <pkg>...           Remove packages");
    println!("  upgrade                Upgrade the Kyanite base (if pending) and all installed packages");
    println!("  fix <pkg>...           Reinstall packages (repair)");
    println!("  search <term>...       Search package names and descriptions");
    println!("  info [pkg]...          Show package info; without arguments, list installed packages");
    println!("      -L  list files of an installed package");
    println!("      -W  show which package owns a path");
    println!("      -R  show the dependencies of a package");
    println!("      -r  show which installed packages require it");
    println!("  list [pattern]         List available packages (-I installed, -u upgradable)");
    println!("  version                Show installed packages that can be upgraded");
    println!("  policy <pkg>...        Show installed and candidate versions");
    println!("  audit                  Verify installed files against their recorded SHA256");
    println!("  cache clean|size       Manage the downloaded .deb cache");
    println!("  purge <pkg>...         Remove packages including modified config files");
    println!("  autoremove             Remove dependencies that are no longer needed");
    println!("  configure [pkg]...     Re-run the post-install hooks of installed packages");
    println!("  base                   Show the Kyanite base version and pending update");
    println!("  help                   Show this help");
    println!();
    println!("apt syntax: invoke kpkg as 'apt' or 'apt-get' (symlinks included in Kyanite) to get");
    println!("the apt-style help and prompts; every command above keeps working too.");
    println!();
    println!("Options:");
    println!("  -s, --simulate         Show what would be done without changing anything");
    println!("  -f, --force            Overwrite unowned/foreign files, ignore reverse dependencies");
    println!("  -y, --yes              Do not ask for confirmation (only asked in apt mode)");
    println!("      --root <dir>       Operate on another root (default: /)");
    println!("      --arch <arch>      Debian architecture (default: arm64 or amd64, per build)");
    println!("      --mirror <url>     Repository mirror (default: {})", DEFAULT_MIRROR);
    println!("      --security-mirror <url>  Security mirror (default: {})", DEFAULT_SECURITY_MIRROR);
    println!("      --suite <name>     Debian suite (default: {})", DEFAULT_SUITE);
    println!("      --keyring <file>   OpenPGP keyring to trust instead of the default ones");
    println!("      --no-security      Do not use the <suite>-security repository (not recommended)");
    println!("  -V, --version          Show the kpkg version");
    println!("  -h, --help             Show this help");
    println!();
    println!("Files:");
    println!("  /etc/kpkg/assume-installed   packages treated as already installed (one per line,");
    println!("                               'name' or 'name=version'); default: libc6, libgcc-s1");
    println!("  /usr/share/kpkg/hooks/*.hook declarative per-package hooks (shipped by Kyanite)");
    println!("  /etc/kpkg/hooks.d/*.hook     local hook overrides");
    println!("  /usr/share/keyrings/debian-archive-keyring.gpg   trusted repository keys");
    println!("  /etc/kpkg/kyanite-release.pub  public key(s) for Kyanite release signatures");
    println!("  /var/lib/kpkg                index and installed-package database");
    println!("  /var/cache/kpkg/pkgs         downloaded .deb files");
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[-] Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Res<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let apt = detect_apt_mode();
    let mut ctx = Ctx {
        root: PathBuf::from("/"),
        arch: default_arch(),
        mirror: DEFAULT_MIRROR.to_string(),
        security_mirror: DEFAULT_SECURITY_MIRROR.to_string(),
        suite: DEFAULT_SUITE.to_string(),
        keyring: None,
        simulate: false,
        force: false,
        yes: false,
        apt,
        no_security: false,
    };
    let mut pos: Vec<String> = Vec::new();
    let mut flags: Vec<String> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--root" | "--arch" | "--mirror" | "--security-mirror" | "--suite" | "--keyring" => {
                i += 1;
                let v = args.get(i).ok_or_else(|| format!("{} needs a value", a))?.clone();
                match a {
                    "--root" => ctx.root = PathBuf::from(v),
                    "--arch" => ctx.arch = v,
                    "--mirror" => ctx.mirror = v.trim_end_matches('/').to_string(),
                    "--security-mirror" => ctx.security_mirror = v.trim_end_matches('/').to_string(),
                    "--keyring" => ctx.keyring = Some(PathBuf::from(v)),
                    _ => ctx.suite = v,
                }
            }
            "-s" | "--simulate" | "--dry-run" => ctx.simulate = true,
            "-f" | "--force" => ctx.force = true,
            "-y" | "--yes" | "--assume-yes" => ctx.yes = true,
            "--no-security" => ctx.no_security = true,
            // opciones de apt que se aceptan sin efecto (kpkg ya no instala Recommends)
            "-q" | "-qq" | "--quiet" | "--no-install-recommends" | "--install-recommends"
            | "--no-install-suggests" => {}
            "--installed" => flags.push("-I".to_string()),
            "--upgradable" => flags.push("-u".to_string()),
            "-h" | "--help" => {
                print_help(apt);
                return Ok(());
            }
            "-V" | "--version" => {
                println!("kpkg {}", VERSION);
                return Ok(());
            }
            _ if a.starts_with('-') && a.len() > 1 => flags.push(a.to_string()),
            _ => pos.push(a.to_string()),
        }
        i += 1;
    }

    if !(ctx.mirror.starts_with("https://") || ctx.mirror.starts_with("http://")) {
        return Err("--mirror must start with https:// (or http://)".into());
    }
    if !(ctx.security_mirror.starts_with("https://") || ctx.security_mirror.starts_with("http://")) {
        return Err("--security-mirror must start with https:// (or http://)".into());
    }
    if ctx.arch.is_empty()
        || !ctx.arch.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err("invalid --arch".into());
    }
    if !valid_suite(&ctx.suite) {
        return Err("invalid --suite".into());
    }

    if pos.is_empty() {
        print_help(apt);
        return Ok(());
    }
    let cmd = pos.remove(0);
    match cmd.as_str() {
        "add" | "install" => cmd_add(&ctx, &pos, false),
        "fix" | "reinstall" => cmd_add(&ctx, &pos, true),
        "del" | "remove" => cmd_del(&ctx, &pos, false),
        "purge" => cmd_del(&ctx, &pos, true),
        "autoremove" | "auto-remove" => cmd_autoremove(&ctx),
        "update" => cmd_update(&ctx),
        "upgrade" | "full-upgrade" | "dist-upgrade" => cmd_upgrade(&ctx),
        "search" => cmd_search(&ctx, &pos),
        "info" | "show" => cmd_info(&ctx, &pos, &flags),
        "depends" => {
            let mut f = flags.clone();
            f.push("-R".to_string());
            cmd_info(&ctx, &pos, &f)
        }
        "rdepends" => {
            let mut f = flags.clone();
            f.push("-r".to_string());
            cmd_info(&ctx, &pos, &f)
        }
        "list" => cmd_list(&ctx, &pos, &flags),
        "version" => {
            let mut f = flags.clone();
            f.push("-u".to_string());
            cmd_list(&ctx, &pos, &f)
        }
        "policy" => cmd_policy(&ctx, &pos),
        "audit" => cmd_audit(&ctx),
        "cache" => cmd_cache(&ctx, &pos),
        "clean" | "autoclean" => cmd_cache(&ctx, &["clean".to_string()]),
        "configure" => cmd_configure(&ctx, &pos),
        "base" => cmd_base_status(&ctx),
        // interno: lo invoca `kpkg upgrade` con el kpkg NUEVO ya verificado (el lock ya esta tomado)
        "base-apply" => {
            if !ctx.db_dir().join("lock").exists() {
                return Err("base-apply is internal; use 'kpkg upgrade'".into());
            }
            let stage = pos.get(0).ok_or("base-apply: missing staging directory")?;
            base_apply(&ctx, Path::new(stage))
        }
        "help" => {
            print_help(apt);
            Ok(())
        }
        other => Err(format!("unknown command '{}'. Try '{} help'.", other, if apt { "apt" } else { "kpkg" }).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64() {
        assert_eq!(b64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(b64_decode("aGVsbG8h").unwrap(), b"hello!");
        assert!(b64_decode("a$b").is_none());
    }

    #[test]
    fn dates() {
        assert_eq!(parse_http_date("Thu, 01 Jan 1970 00:00:00 UTC"), Some(0));
        assert_eq!(parse_http_date("Sat, 18 Oct 2025 08:19:01 UTC"), Some(1760775541));
        assert_eq!(parse_http_date("Tue, 29 Feb 2028 23:59:59 UTC"), Some(1835481599));
        assert_eq!(parse_http_date("garbage"), None);
    }

    #[test]
    fn release_versions() {
        assert_eq!(parse_release_version("v0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_release_version("1.10.3"), Some((1, 10, 3)));
        assert_eq!(parse_release_version("v0.1.0-3-g1a2b3c4"), None);
        assert_eq!(parse_release_version("1a2b3c4"), None);
        assert_eq!(parse_release_version("v1.2"), None);
        assert!(parse_release_version("v0.10.0") > parse_release_version("v0.9.9"));
    }

    #[test]
    fn hook_parse() {
        let h = parse_hook(
            "depends ca-certificates\nignore-depends debconf debconf-2.0\n[post-install]\nbuiltin update-ca-certificates\nmkdir /etc/x 755\n",
        )
        .unwrap();
        assert_eq!(h.add_depends, vec!["ca-certificates"]);
        assert_eq!(h.ignore_depends.len(), 2);
        assert_eq!(h.post_install.len(), 2);
        assert!(parse_hook("[post-install]\nrm -rf /\n").is_err());
        assert!(parse_hook("[post-install]\nsymlink onlyone\n").is_err());
        assert!(parse_hook("[bogus]\n").is_err());
    }

    #[test]
    fn changelog_parse() {
        let t = "# c\n== v0.2.0\n+vim\n-tiny-vim\nnote hola\n== v0.1.5\nmkdir /opt/x\n";
        let s = parse_changelog(t).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].ver, (0, 1, 5));
        assert_eq!(s[1].lines.len(), 3);
        assert!(parse_changelog("+vim\n").is_err());
        assert!(parse_changelog("== v0.2.0\nfoo bar\n").is_err());
    }

    #[test]
    fn pkg_line_roundtrip_and_legacy() {
        let mut p = Pkg::default();
        p.name = "curl".into();
        p.version = "1".into();
        p.origin = "https://deb.debian.org/debian-security".into();
        let q = Pkg::from_line(&p.to_line()).unwrap();
        assert_eq!(q.origin, p.origin);
        let legacy = "a\t1\tamd64\t\t\t\t\t\tpool/x.deb\t0\t1\t2\tdesc here";
        let q = Pkg::from_line(legacy).unwrap();
        assert_eq!(q.desc, "desc here");
        assert_eq!(q.origin, "");
    }
}
