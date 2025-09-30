use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::blocking::Client;

fn verify_md5(file_path: &Path, expected_hash: &str) -> bool {
    let mut f = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buffer = Vec::new();
    if f.read_to_end(&mut buffer).is_err() {
        return false;
    }
    let digest = md5::compute(&buffer);
    let hex = format!("{:x}", digest);
    hex == expected_hash
}

pub fn download_files(
    wget_list: &str,
    target_dir: &Path,
    package_mirror: Option<String>,
    md5_map: Option<&HashMap<String, String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(target_dir)?;

    let urls: Vec<&str> = wget_list.lines().filter(|l| !l.trim().is_empty()).collect();
    let total = urls.len();
    let client = Arc::new(Client::new());
    let mp = Arc::new(MultiProgress::new());

    let md5_map = md5_map.cloned();

    let mut handles = vec![];

    for (i, url) in urls.into_iter().enumerate() {
        let client = Arc::clone(&client);
        let mp = Arc::clone(&mp);
        let target_dir = target_dir.to_path_buf();
        let package_mirror = package_mirror.clone();
        let url = url.to_string();
        let md5_map = md5_map.clone();

        let handle = thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send>> {
            let filename = url.split('/').last().unwrap_or("file.tar.xz");
            let filepath = target_dir.join(filename);

            let download_url = if let Some(ref mirror) = package_mirror {
                if url.contains("ftp.gnu.org") {
                    url.replacen("ftp.gnu.org", mirror, 1)
                } else {
                    url.to_string()
                }
            } else {
                url.to_string()
            };

            let pb = mp.add(ProgressBar::new(0));
            pb.set_style(
                ProgressStyle::with_template(
                    "{bar:40.cyan/blue} {bytes}/{total_bytes} ({eta}) {msg}",
                )
                .unwrap()
                .progress_chars("=> "),
            );
            pb.set_message(format!(
                "[{}/{}] {}",
                i + 1,
                total,
                style(filename).yellow()
            ));

            let mut resp = client
                .get(&download_url)
                .send()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
            let total_size = resp.content_length().unwrap_or(0);
            pb.set_length(total_size);

            let mut file = File::create(&filepath)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
            let mut downloaded: u64 = 0;
            let mut buffer = [0u8; 8192];

            loop {
                let bytes_read = resp
                    .read(&mut buffer)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                if bytes_read == 0 {
                    break;
                }
                file.write_all(&buffer[..bytes_read])
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                downloaded += bytes_read as u64;
                pb.set_position(downloaded);
            }

            let status = if let Some(ref md5_map) = md5_map {
                if let Some(expected_hash) = md5_map.get(filename) {
                    if verify_md5(&filepath, expected_hash) {
                        style("✅").green()
                    } else {
                        style("❌").red()
                    }
                } else {
                    style("⚠️").yellow()
                }
            } else {
                style("⚠️").yellow()
            };

            pb.finish_with_message(format!("{} {}", status, style(filename).yellow()));
            Ok(())
        });

        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().unwrap();
        result.map_err(|e| e as Box<dyn std::error::Error>)?;
    }

    Ok(())
}
