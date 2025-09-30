mod downloader;
mod md5_utils;
mod mirrors;
mod version_check;
mod wget_list;

#[cfg(feature = "tui")]
mod tui; // Importiere das TUI-Modul, wenn das Feature aktiv ist

use console::style;
use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "tui")]
    {
        // Wenn das TUI-Feature aktiv ist, starte das TUI-Menü
        tui::tui_menu()?;
        Ok(())
    }

    #[cfg(not(feature = "tui"))]
    {
        // Wenn das TUI-Feature NICHT aktiv ist, führe die CLI-Logik aus
        // --- Run host system version checks ---
        if version_check::run_version_checks() {
            eprintln!(
                "{} Host system does not meet minimum requirements. Exiting.",
                style("❌").red().bold()
            );
            std::process::exit(1);
        }

        println!(
            "{} All version checks passed. Starting downloader...",
            style("✅").green().bold()
        );

        // --- Determine LFS sources path ---
        let lfs_sources = match env::var("LFS") {
            Ok(lfs) => PathBuf::from(lfs).join("sources"),
            Err(_) => {
                let mut rng = rand::thread_rng(); // Verwende thread_rng() statt rng()
                let random_number: u32 = rng.gen_range(1000..=9999); // Verwende gen_range() statt random_range()
                let tmp_path = format!("/tmp/lfs_{}", random_number);
                println!(
                    "{} Using temporary path {}",
                    style("ℹ️").blue(),
                    style(&tmp_path).yellow()
                );
                PathBuf::from(tmp_path).join("sources")
            }
        };

        // --- Choose mirror and fetch wget list ---
        // Diese Zeile wird entfernt, da die Mirror-Auswahl in der TUI erfolgt
        // let package_mirror = mirrors::choose_package_mirror();

        // Da die Mirror-Auswahl nun in der TUI erfolgt, müssen wir hier einen Standardwert oder eine andere Logik verwenden,
        // wenn das TUI nicht aktiv ist. Für dieses Beispiel nehmen wir an, dass wir keinen Mirror verwenden,
        // wenn das TUI nicht aktiv ist, oder wir könnten eine andere CLI-basierte Auswahl implementieren.
        // Für den Moment setzen wir es auf None, was bedeutet, dass der Standard-Mirror verwendet wird.
        let package_mirror: Option<String> = None;


        let wget_list = wget_list::get_wget_list()?;

        // --- Prepare MD5 map ---
        let mut md5_map: HashMap<String, String> = HashMap::new();
        let md5_content = md5_utils::get_md5sums()?;
        for line in md5_content.lines() {
            let mut parts = line.split_whitespace();
            if let (Some(hash), Some(filename)) = (parts.next(), parts.next()) {
                md5_map.insert(filename.to_string(), hash.to_string());
            }
        }

        // --- Download files ---
        downloader::download_files(&wget_list, &lfs_sources, package_mirror, Some(&md5_map))?;

        println!("{} All done!", style("🎉").green().bold());
        Ok(())
    }
}
