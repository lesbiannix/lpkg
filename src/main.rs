mod downloader;
mod html;
mod md5_utils;
mod mirrors;
mod version_check;
mod wget_list;

#[cfg(feature = "tui")]
mod tui;

use console::style;
use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "tui")]
    {
        // TUI-Modus
        tui::tui_menu()?;
        return Ok(());
    }

    #[cfg(not(feature = "tui"))]
    {
        // --- Dynamische Version-Prüfung direkt aus HTML ---
        let ok = version_check::run_version_checks_from_html(
            "https://www.linuxfromscratch.org/~thomas/multilib-m32/chapter02/hostreqs.html",
        )?;

        if !ok {
            eprintln!(
                "{} Some version checks failed. Exiting.",
                style("❌").red().bold()
            );
            std::process::exit(1);
        }

        println!(
            "{} All version checks passed. Starting downloader...",
            style("✅").green().bold()
        );

        // --- Bestimme LFS-Sources-Pfad ---
        let lfs_sources = match env::var("LFS") {
            Ok(lfs) => PathBuf::from(lfs).join("sources"),
            Err(_) => {
                let mut rng = rand::thread_rng();
                let random_number: u32 = rng.gen_range(1000..=9999);
                let tmp_path = format!("/tmp/lfs_{}", random_number);
                println!(
                    "{} Using temporary path {}",
                    style("ℹ️").blue(),
                    style(&tmp_path).yellow()
                );
                PathBuf::from(tmp_path).join("sources")
            }
        };

        // --- CLI Mirror-Auswahl: default oder erweiterbar ---
        let package_mirror: Option<String> = None;

        // --- Hole wget-Liste ---
        let wget_list = wget_list::get_wget_list()?;

        // --- Bereite MD5-Map vor ---
        let mut md5_map: HashMap<String, String> = HashMap::new();
        let md5_content = md5_utils::get_md5sums()?;
        for line in md5_content.lines() {
            let mut parts = line.split_whitespace();
            if let (Some(hash), Some(filename)) = (parts.next(), parts.next()) {
                md5_map.insert(filename.to_string(), hash.to_string());
            }
        }

        // --- Lade Dateien herunter ---
        downloader::download_files(&wget_list, &lfs_sources, package_mirror, Some(&md5_map))?;

        println!("{} All done!", style("🎉").green().bold());
        Ok(())
    }
}
