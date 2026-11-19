use std::env;
use std::error::Error;
use std::io::{Read, Cursor};
use std::path::Path;
use xz2::read::XzDecoder;
use flate2::read::GzDecoder;
use sha2::{Sha256, Digest};

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
    let mut deb_response = ureq::get(&deb_url).call()?;
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
    let root_dir = Path::new("/");

    // Iterate through the `ar` container files inside the .deb
    while let Some(entry_result) = archive.next_entry() {
        let mut entry = entry_result?;
        let name = String::from_utf8_lossy(entry.header().identifier()).into_owned();

        // We are looking for data.tar.xz or data.tar.gz
        if name.starts_with("data.tar") {
            let mut data_bytes = Vec::new();
            entry.read_to_end(&mut data_bytes)?;
            let data_cursor = Cursor::new(data_bytes);

            if name.ends_with(".xz") {
                let decoder = XzDecoder::new(data_cursor);
                let mut tar_archive = tar::Archive::new(decoder);
                tar_archive.unpack(root_dir)?;
            } else if name.ends_with(".gz") {
                let decoder = GzDecoder::new(data_cursor);
                let mut tar_archive = tar::Archive::new(decoder);
                tar_archive.unpack(root_dir)?;
            } else {
                return Err(format!("Unsupported compression format inside .deb: {}", name).into());
            }

            return Ok(());
        }
    }

    Err("Could not find data payload (`data.tar.*`) inside the .deb archive.".into())
}
