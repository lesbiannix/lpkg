use console::style;
use md5;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

pub fn get_md5sums() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let res = client
        .get("https://www.linuxfromscratch.org/~thomas/multilib-m32/md5sums")
        .send()?
        .text()?;
    Ok(res)
}

pub fn save_md5sums(content: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(path, content)?;
    Ok(())
}

pub fn verify_md5sums(
    md5_file: &Path,
    sources_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(md5_file)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        let expected_hash = parts.next().ok_or("Malformed md5sums line")?;
        let filename = parts.next().ok_or("Malformed md5sums line")?;
        let file_path = sources_dir.join(filename);

        let mut f = File::open(&file_path)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;

        let digest = md5::compute(&buffer);
        let hex = format!("{:x}", digest);

        if hex == expected_hash {
            println!("{} {} OK", style("✅").green(), filename);
        } else {
            println!(
                "{} {} FAILED (expected {}, got {})",
                style("❌").red(),
                filename,
                expected_hash,
                hex
            );
        }
    }
    Ok(())
}
