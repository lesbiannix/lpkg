mod downloader;
mod md5_utils;
mod mirrors;
mod wget_list;

use console::style;
use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // LFS sources Pfad
    let lfs_sources = match env::var("LFS") {
        Ok(lfs) => PathBuf::from(lfs).join("sources"),
        Err(_) => {
            let mut rng = rand::rng();
            let random_number: u32 = rng.random_range(1000..=9999);
            let tmp_path = format!("/tmp/lfs_{}", random_number);
            println!(
                "{} Using temporary path {}",
                style("ℹ️").blue(),
                style(&tmp_path).yellow()
            );
            PathBuf::from(tmp_path).join("sources")
        }
    };

    // Mirror für Pakete auswählen
    let package_mirror = mirrors::choose_package_mirror();

    // Wget-Liste vom Original LFS-Mirror holen
    let wget_list = wget_list::get_wget_list()?;

    // MD5 Map vorbereiten
    let mut md5_map: HashMap<String, String> = HashMap::new();
    let md5_content = md5_utils::get_md5sums()?;
    for line in md5_content.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(hash), Some(filename)) = (parts.next(), parts.next()) {
            md5_map.insert(filename.to_string(), hash.to_string());
        }
    }

    // Pakete herunterladen + Live-MD5 prüfen
    downloader::download_files(&wget_list, &lfs_sources, package_mirror, Some(&md5_map))?;

    println!("{} All done!", style("🎉").green().bold());
    Ok(())
}
