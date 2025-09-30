use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::thread;

pub fn download_files(
    files: &[String],
    target_dir: &Path,
    mirror: Option<&str>,
    md5_map: Option<&HashMap<String, String>>,
) -> Result<()> {
    fs::create_dir_all(target_dir)?;

    let mut handles = vec![];

    for url in files.iter().cloned() {
        let target_dir = target_dir.to_path_buf();
        let mirror = mirror.map(|m| m.to_string());
        let md5_map = md5_map.cloned();

        let handle = thread::spawn(move || -> Result<()> {
            let download_url = if let Some(m) = &mirror {
                url.replace("ftp.gnu.org", m)
            } else {
                url.clone()
            };

            let filename = download_url
                .split('/')
                .last()
                .ok_or_else(|| anyhow!("Failed to extract filename"))?;
            let filepath = target_dir.join(filename);

            let mut resp = reqwest::blocking::get(&download_url)?;
            let mut buffer = Vec::new();
            resp.read_to_end(&mut buffer)?;

            let mut file = File::create(&filepath)?;
            file.write_all(&buffer)?;

            if let Some(md5s) = md5_map.as_ref() {
                if let Some(expected) = md5s.get(filename) {
                    let digest = md5::compute(&buffer);
                    if format!("{:x}", digest) != *expected {
                        return Err(anyhow!("MD5 mismatch for {}", filename));
                    }
                }
            }

            Ok(())
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().map_err(|_| anyhow!("Thread panicked"))??;
    }

    Ok(())
}
