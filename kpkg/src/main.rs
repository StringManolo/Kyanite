use std::env;
use std::error::Error;
use std::fs;
use std::io::{Read, Cursor};
use std::path::{Component, Path, PathBuf};
use xz2::read::XzDecoder;
use flate2::read::GzDecoder;
use sha2::{Sha256, Digest};
use tar::EntryType;

/// Tamaño máximo por entrada individual (256 MiB) — evita bombas de descompresión.
const MAX_ENTRY_SIZE: u64 = 256 * 1024 * 1024;
/// Tamaño máximo total extraído por paquete (1 GiB).
const MAX_TOTAL_SIZE: u64 = 1024 * 1024 * 1024;
const DEBIAN_MIRROR: &str = "https://deb.debian.org/debian";
const CODENAME: &str = "stable";

fn main() {
    if let Err(e) = run() {
        eprintln!("[-] Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Kyanite Package Manager (kpkg)");
        println!("Usage:");
        println!("  kpkg add <package_name>    - Install a package securely");
        return Ok(());
    }

    let command = &args[1];
    let pkg_name = &args[2];

    match command.as_str() {
        "add" => {
            install_deb_package(pkg_name)?;
        }
        _ => {
            println!("Unknown command: '{}'. Did you mean 'kpkg add <package>'?", command);
        }
    }

    Ok(())
}

fn install_deb_package(pkg_name: &str) -> Result<(), Box<dyn Error>> {
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "amd64"
    };

    let index_url = format!("{}/dists/{}/main/binary-{}/Packages.xz", DEBIAN_MIRROR, CODENAME, arch);
    println!("[*] Fetching package index for {} ({}) from Debian...", arch, CODENAME);

    // 1. Download Packages.xz index
    let response = ureq::get(&index_url).call()?;
    let mut compressed_data = Vec::new();
    response.into_reader().read_to_end(&mut compressed_data)?;

    // 2. Decompress index in memory
    println!("[*] Decompressing package index...");
    let decoder = XzDecoder::new(&compressed_data[..]);
    let mut index_content = String::new();
    let mut decoder = decoder;
    decoder.read_to_string(&mut index_content)?;

    // 3. Parse index to find package URL and official SHA256 hash
    println!("[*] Searching for metadata of '{}'...", pkg_name);
    let (deb_filename, expected_sha256) = parse_package_index(&index_content, pkg_name)?
        .ok_or_else(|| format!("Package '{}' not found in the official Debian repository index.", pkg_name))?;

    let deb_url = format!("{}/{}", DEBIAN_MIRROR, deb_filename);
    println!("[+] Package found:\n    -> URL: {}\n    -> Official SHA256: {}", deb_url, expected_sha256);

    // 4. Download the .deb file
    println!("[*] Downloading .deb package...");
    let deb_response = ureq::get(&deb_url).call()?;
    let mut deb_bytes = Vec::new();
    deb_response.into_reader().read_to_end(&mut deb_bytes)?;

    // 5. Verify cryptographic SHA256 integrity
    println!("[*] Verifying cryptographic SHA256 integrity...");
    let mut hasher = Sha256::new();
    hasher.update(&deb_bytes);
    let calculated_sha256 = hex::encode(hasher.finalize());

    if calculated_sha256 != expected_sha256 {
        return Err(format!(
            "[!] SECURITY ALERT: SHA256 hash mismatch!\nExpected:  {}\nCalculated: {}",
            expected_sha256, calculated_sha256
        ).into());
    }

    println!("[✔] Integrity verified successfully! The .deb package is authentic.");

    // 6. Extract contents into rootfs (/)
    println!("[*] Extracting package payload into rootfs...");
    extract_deb_payload(&deb_bytes)?;

    println!("[✔] Package '{}' installed successfully!", pkg_name);
    Ok(())
}

fn parse_package_index(content: &str, target_pkg: &str) -> Result<Option<(String, String)>, Box<dyn Error>> {
    for block in content.split("\n\n") {
        let mut pkg_name = None;
        let mut filename = None;
        let mut sha256 = None;

        for line in block.lines() {
            if let Some(stripped) = line.strip_prefix("Package: ") {
                pkg_name = Some(stripped.trim());
            } else if let Some(stripped) = line.strip_prefix("Filename: ") {
                filename = Some(stripped.trim().to_string());
            } else if let Some(stripped) = line.strip_prefix("SHA256: ") {
                sha256 = Some(stripped.trim().to_string());
            }
        }

        if pkg_name == Some(target_pkg) {
            if let (Some(f), Some(s)) = (filename, sha256) {
                return Ok(Some((f, s)));
            }
        }
    }
    Ok(None)
}

fn extract_deb_payload(deb_bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let cursor = Cursor::new(deb_bytes);
    let mut archive = ar::Archive::new(cursor);

    // rootfs raíz = "/" (en proot esto es el rootfs simulado)
    let root = Path::new("/");

    while let Some(entry_result) = archive.next_entry() {
        let mut entry = entry_result?;
        let name = entry.header().identifier()?.to_string();

        if name.starts_with("data.tar") {
            let mut data_bytes = Vec::new();
            entry.read_to_end(&mut data_bytes)?;
            let data_cursor = Cursor::new(data_bytes);

            let mut total_written: u64 = 0;

            if name.ends_with(".xz") {
                let decoder = XzDecoder::new(data_cursor);
                let mut tar_archive = tar::Archive::new(decoder);
                extract_tar_safely(&mut tar_archive, root, &mut total_written)?;
            } else if name.ends_with(".gz") {
                let decoder = GzDecoder::new(data_cursor);
                let mut tar_archive = tar::Archive::new(decoder);
                extract_tar_safely(&mut tar_archive, root, &mut total_written)?;
            } else {
                return Err(format!("Unsupported compression format inside .deb: {}", name).into());
            }

            return Ok(());
        }
    }

    Err("Could not find data payload (`data.tar.*`) inside the .deb archive.".into())
}

/// Extrae un tar validando cada entrada. Nada se escribe fuera de `root`.
fn extract_tar_safely<R: Read>(
    archive: &mut tar::Archive<R>,
    root: &Path,
    total_written: &mut u64,
) -> Result<(), Box<dyn Error>> {
    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let entry_type = entry.header().entry_type();

        // 1. Rechazar tipos peligrosos
        match entry_type {
            EntryType::Regular | EntryType::Directory => {}
            EntryType::Symlink | EntryType::Link => {
                // Permitimos symlinks SOLO si su destino, resuelto desde el directorio
                // contenedor, sigue dentro de root.
                let link_name = entry.link_name()?
                    .ok_or("Symlink/hardlink sin destino")?
                    .into_owned();
                let entry_path = entry.path()?.into_owned();
                let safe_path = safe_join(root, &entry_path)?;
                let parent = safe_path.parent().ok_or("Ruta sin padre")?;
                let target = parent.join(&link_name);
                ensure_inside(root, &target)?;
                // Nota: no creamos el symlink con contenido externo; lo recreamos abajo.
            }
            EntryType::Char | EntryType::Block | EntryType::Fifo | EntryType::Socket => {
                return Err(format!(
                    "Refusing to extract special file (type {:?}) — potencialmente malicioso",
                    entry_type
                ).into());
            }
            other => {
                return Err(format!("Tipo de entrada no soportado: {:?}", other).into());
            }
        }

        // 2. Validar y normalizar la ruta dentro del rootfs
        let raw_path = entry.path()?.into_owned();
        let dest = safe_join(root, &raw_path)?;

        // 3. Comprobar tamaño antes de escribir
        let size = entry.header().size()?;
        if size > MAX_ENTRY_SIZE {
            return Err(format!(
                "Entrada '{}' demasiado grande ({} bytes > {})",
                dest.display(), size, MAX_ENTRY_SIZE
            ).into());
        }
        *total_written = total_written.saturating_add(size);
        if *total_written > MAX_TOTAL_SIZE {
            return Err(format!(
                "Tamaño total extraído excede el límite ({} bytes > {})",
                total_written, MAX_TOTAL_SIZE
            ).into());
        }

        // 4. Crear directorio padre si no existe
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
            // Re-verificar tras crear el padre (defensa contra symlinks preexistentes)
            ensure_inside(root, parent)?;
        }

        // 5. Escribir según tipo
        match entry_type {
            EntryType::Regular => {
                let mut out = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&dest)?;
                std::io::copy(&mut entry, &mut out)?;
            }
            EntryType::Directory => {
                fs::create_dir_all(&dest)?;
            }
            EntryType::Symlink => {
                let link_name = entry.link_name()?.ok_or("Symlink sin destino")?.into_owned();
                // Recrear el symlink (con destino ya validado arriba)
                let _ = fs::remove_file(&dest);
                std::os::unix::fs::symlink(&link_name, &dest)?;
            }
            EntryType::Link => {
                let link_name = entry.link_name()?.ok_or("Hardlink sin destino")?.into_owned();
                let target = safe_join(root, &link_name)?;
                ensure_inside(root, &target)?;
                let _ = fs::remove_file(&dest);
                fs::hard_link(&target, &dest)?;
            }
            _ => unreachable!(),
        }

        // 6. Permisos: preservar bits pero nunca setuid/setgid en un rootfs proot
        let mode = entry.header().mode().unwrap_or(0o644);
        let safe_mode = mode & 0o777; // elimina setuid/setgid/sticky
        use std::os::unix::fs::PermissionsExt;
        if entry_type == EntryType::Regular {
            let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(safe_mode));
        }
    }
    Ok(())
}

/// Une `root` + `path_relativa` rechazando componentes peligrosos.
fn safe_join(root: &Path, path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if path.is_absolute() {
        return Err(format!("Ruta absoluta rechazada: {}", path.display()).into());
    }
    let mut out = PathBuf::from(root);
    for comp in path.components() {
        match comp {
            Component::Normal(c) => {
                // Rechazar nombres de control (NUL, etc.)
                let s = c.to_string_lossy();
                if s.contains('\0') || s == ".." || s == "." {
                    return Err(format!("Componente de ruta inválido: {:?}", c).into());
                }
                out.push(c);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!(
                    "Intento de path traversal ('..') en: {}",
                    path.display()
                ).into());
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("Componente de ruta absoluto: {:?}", comp).into());
            }
        }
    }
    Ok(out)
}

/// Verifica que `p` sigue dentro de `root` tras resolver symlinks preexistentes.
fn ensure_inside(root: &Path, p: &Path) -> Result<(), Box<dyn Error>> {
    // canonicalize falla si p no existe; en ese caso resolvemos el ancestro existente más cercano.
    let mut cur = p.to_path_buf();
    loop {
        match cur.canonicalize() {
            Ok(real) => {
                let root_real = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
                if !real.starts_with(&root_real) {
                    return Err(format!(
                        "Ruta escapa del rootfs: {} -> {}",
                        p.display(),
                        real.display()
                    ).into());
                }
                return Ok(());
            }
            Err(_) => {
                if !cur.pop() {
                    return Err("No se pudo resolver la ruta".into());
                }
            }
        }
    }
}
