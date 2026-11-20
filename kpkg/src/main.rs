//! kpkg - Kyanite package manager.
//! Interfaz estilo apk (add/del/update/upgrade/search/info/list/...) sobre repositorios Debian.
//! Ligero: sin scripts de mantenedor, descargas verificadas por SHA256, extraccion segura.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::EntryType;
use xz2::read::XzDecoder;

type Res<T> = Result<T, Box<dyn Error>>;

const VERSION: &str = "0.2.0";
const DEFAULT_MIRROR: &str = "https://deb.debian.org/debian";
const DEFAULT_SUITE: &str = "stable";
const COMPONENT: &str = "main";

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

struct Ctx {
    root: PathBuf,
    arch: String,
    mirror: String,
    suite: String,
    simulate: bool,
    force: bool,
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
}

fn clean_field(s: &str) -> String {
    s.replace(|c: char| c == '\t' || c == '\n' || c == '\r', " ")
}

impl Pkg {
    fn to_line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
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
            clean_field(&self.desc)
        )
    }

    fn from_line(line: &str) -> Option<Pkg> {
        let f: Vec<&str> = line.splitn(13, '\t').collect();
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

/// PENDIENTE (seguridad): verificar la firma OpenPGP de InRelease con el keyring de Debian
/// y, SOLO entonces, usar su contenido. Hasta que exista, la cadena de confianza es HTTPS
/// + los hashes de Release; un mirror comprometido podria servir un Release falso.
fn verify_release_signature(_clearsigned: &str) -> Res<()> {
    println!("[!] WARNING: the OpenPGP signature of InRelease is NOT verified yet (trust = HTTPS + hashes).");
    Ok(())
}

fn cmd_update(ctx: &Ctx) -> Res<()> {
    let _lock = Lock::acquire(ctx)?;
    let base = format!("{}/dists/{}", ctx.mirror, ctx.suite);
    println!("[*] Fetching {}/InRelease", base);
    let rel = http_get(&format!("{}/InRelease", base), MAX_RELEASE)?;
    let rel_text = String::from_utf8_lossy(&rel).to_string();
    verify_release_signature(&rel_text)?;

    let mut meta = String::new();
    for line in rel_text.lines() {
        if line.starts_with("Suite: ")
            || line.starts_with("Codename: ")
            || line.starts_with("Version: ")
            || line.starts_with("Date: ")
        {
            println!("    {}", line);
            meta.push_str(line);
            meta.push('\n');
        }
    }

    let hashes = parse_release_hashes(&rel_text);
    let wanted = [
        format!("{}/binary-{}/Packages.xz", COMPONENT, ctx.arch),
        format!("{}/binary-all/Packages.xz", COMPONENT),
    ];

    let mut all: HashMap<String, Pkg> = HashMap::new();
    let mut skipped = 0usize;

    for (n, rel_path) in wanted.iter().enumerate() {
        let (want_sha, want_size) = match hashes.get(rel_path) {
            Some(x) => x.clone(),
            None => {
                if n == 0 {
                    return Err(format!(
                        "{} is not listed in Release (is '{}' a valid arch for this suite?)",
                        rel_path, ctx.arch
                    )
                    .into());
                }
                println!("[*] {} is not published, skipping", rel_path);
                continue;
            }
        };
        if want_size > MAX_INDEX_XZ {
            return Err(format!("{} is unreasonably large ({} bytes)", rel_path, want_size).into());
        }
        println!("[*] Downloading {} ({})", rel_path, human(want_size));
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
        for p in parsed {
            if p.arch != ctx.arch && p.arch != "all" {
                continue;
            }
            if !valid_pkg_name(&p.name)
                || !valid_filename(&p.filename)
                || !valid_sha256(&p.sha256)
                || !valid_version(&p.version)
            {
                skipped += 1;
                continue;
            }
            let replace = match all.get(&p.name) {
                Some(old) => cmp_version(&p.version, &old.version) == Ordering::Greater,
                None => true,
            };
            if replace {
                all.insert(p.name.clone(), p);
            }
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
    write_atomic(&ctx.meta_path(), meta.as_bytes())?;
    println!("[+] Index updated: {} packages ({} entries rejected as invalid)", all.len(), skipped);
    Ok(())
}

// ============================================================
// RESOLUCION DE DEPENDENCIAS
// ============================================================
// Version ligera de lo que hace apt: Pre-Depends + Depends (con alternativas "a | b",
// versiones y paquetes virtuales via Provides), Conflicts/Breaks, y backtracking
// simple por alternativa. Ignora Recommends/Suggests.

struct Resolver<'a> {
    idx: &'a Index,
    db: &'a Db,
    plan: Vec<usize>,
    chosen: HashSet<usize>,
    chosen_names: HashSet<String>,
}

impl<'a> Resolver<'a> {
    fn new(idx: &'a Index, db: &'a Db) -> Resolver<'a> {
        Resolver {
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
        let mut entries = parse_dep_list(&p.pre_depends);
        entries.extend(parse_dep_list(&p.depends));
        for entry in entries {
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
fn broken_by_removal(db: &Db, removed: &HashSet<String>) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for q in db.pkgs.values() {
        if removed.contains(&q.name) {
            continue;
        }
        let mut entries = parse_dep_list(&q.pre_depends);
        entries.extend(parse_dep_list(&q.depends));
        for entry in entries {
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
    download_verified(&format!("{}/{}", ctx.mirror, p.filename), &path, p.size, &p.sha256)?;
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
    Ok(())
}

fn remove_pkg(ctx: &Ctx, name: &str) -> Res<()> {
    let files = load_files(ctx, name);
    for f in &files {
        if f.kind == 'd' || !db_path_ok(&f.path) {
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
    // directorios vacios, de dentro hacia fuera; nunca los de primer/segundo nivel (usr, usr/bin, ...)
    let mut dirs: Vec<&FileRec> = files.iter().filter(|f| f.kind == 'd').collect();
    dirs.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    for d in dirs {
        if db_path_ok(&d.path) && Path::new(&d.path).components().count() > 2 {
            let _ = fs::remove_dir(ctx.root.join(&d.path));
        }
    }
    fs::remove_dir_all(ctx.pkgs_dir().join(name))?;
    Ok(())
}

// ============================================================
// COMANDOS
// ============================================================

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
        println!("    {} {} [{}]", p.name, p.version, tag);
        dl += p.size;
        inst += p.installed_size * 1024;
    }
    println!("[*] Download: {}, installed size: {}", human(dl), human(inst));
    if ctx.simulate {
        println!("[=] --simulate: nothing was changed.");
        return Ok(());
    }
    let mut owners = load_owners(ctx, db);
    for &i in &plan {
        let p = &idx.pkgs[i];
        install_one(ctx, p, &mut owners, db)?;
        println!("[+] {} {} installed", p.name, p.version);
    }
    println!("[i] Note: kpkg never runs maintainer scripts (preinst/postinst/...).");
    Ok(())
}

fn cmd_add(ctx: &Ctx, names: &[String], reinstall: bool) -> Res<()> {
    if names.is_empty() {
        return Err("add: no package given".into());
    }
    let _lock = Lock::acquire(ctx)?;
    let idx = load_index(ctx)?;
    let db = load_db(ctx)?;
    let mut r = Resolver::new(&idx, &db);

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
        if let Some(inst) = db.pkgs.get(&p.name) {
            if !reinstall && cmp_version(&inst.version, &p.version) != Ordering::Less {
                println!("[=] {} {} is already installed", inst.name, inst.version);
                continue;
            }
        }
        r.try_add(i)?;
    }
    let plan = r.plan.clone();
    run_plan(ctx, &idx, &db, plan)
}

fn cmd_upgrade(ctx: &Ctx) -> Res<()> {
    let _lock = Lock::acquire(ctx)?;
    let idx = load_index(ctx)?;
    let db = load_db(ctx)?;
    let mut r = Resolver::new(&idx, &db);
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

fn cmd_del(ctx: &Ctx, names: &[String]) -> Res<()> {
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
    let broken = broken_by_removal(&db, &removed);
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
    for n in list {
        remove_pkg(ctx, n)?;
        println!("[+] {} removed", n);
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
            Some(&k) => println!("  candidate: {}", idx.pkgs[k].version),
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

fn print_help() {
    println!("kpkg {} - Kyanite package manager (apk-style CLI over Debian repositories)", VERSION);
    println!();
    println!("Usage: kpkg [options] <command> [arguments]");
    println!();
    println!("Commands:");
    println!("  update                 Fetch and verify the package index (InRelease + Packages.xz)");
    println!("  add <pkg>...           Install packages and their dependencies");
    println!("  del <pkg>...           Remove packages");
    println!("  upgrade                Upgrade all installed packages");
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
    println!("  help                   Show this help");
    println!();
    println!("Options:");
    println!("  -s, --simulate         Show what would be done without changing anything");
    println!("  -f, --force            Overwrite unowned/foreign files, ignore reverse dependencies");
    println!("      --root <dir>       Operate on another root (default: /)");
    println!("      --arch <arch>      Debian architecture (default: arm64 or amd64, per build)");
    println!("      --mirror <url>     Repository mirror (default: {})", DEFAULT_MIRROR);
    println!("      --suite <name>     Debian suite (default: {})", DEFAULT_SUITE);
    println!("  -V, --version          Show the kpkg version");
    println!("  -h, --help             Show this help");
    println!();
    println!("Files:");
    println!("  /etc/kpkg/assume-installed   packages treated as already installed (one per line,");
    println!("                               'name' or 'name=version'); default: libc6, libgcc-s1");
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
    let mut ctx = Ctx {
        root: PathBuf::from("/"),
        arch: default_arch(),
        mirror: DEFAULT_MIRROR.to_string(),
        suite: DEFAULT_SUITE.to_string(),
        simulate: false,
        force: false,
    };
    let mut pos: Vec<String> = Vec::new();
    let mut flags: Vec<String> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--root" | "--arch" | "--mirror" | "--suite" => {
                i += 1;
                let v = args.get(i).ok_or_else(|| format!("{} needs a value", a))?.clone();
                match a {
                    "--root" => ctx.root = PathBuf::from(v),
                    "--arch" => ctx.arch = v,
                    "--mirror" => ctx.mirror = v.trim_end_matches('/').to_string(),
                    _ => ctx.suite = v,
                }
            }
            "-s" | "--simulate" => ctx.simulate = true,
            "-f" | "--force" => ctx.force = true,
            "-h" | "--help" => {
                print_help();
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
    if ctx.arch.is_empty()
        || !ctx.arch.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err("invalid --arch".into());
    }
    if ctx.suite.is_empty()
        || !ctx.suite.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'.')
    {
        return Err("invalid --suite".into());
    }

    if pos.is_empty() {
        print_help();
        return Ok(());
    }
    let cmd = pos.remove(0);
    match cmd.as_str() {
        "add" | "install" => cmd_add(&ctx, &pos, false),
        "fix" | "reinstall" => cmd_add(&ctx, &pos, true),
        "del" | "remove" => cmd_del(&ctx, &pos),
        "update" => cmd_update(&ctx),
        "upgrade" => cmd_upgrade(&ctx),
        "search" => cmd_search(&ctx, &pos),
        "info" => cmd_info(&ctx, &pos, &flags),
        "list" => cmd_list(&ctx, &pos, &flags),
        "version" => {
            let mut f = flags.clone();
            f.push("-u".to_string());
            cmd_list(&ctx, &pos, &f)
        }
        "policy" => cmd_policy(&ctx, &pos),
        "audit" => cmd_audit(&ctx),
        "cache" => cmd_cache(&ctx, &pos),
        "help" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown command '{}'. Try 'kpkg help'.", other).into()),
    }
}
