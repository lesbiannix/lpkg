use reqwest;
use scraper::{Html, Selector};
use semver::Version;
use std::process::Command;

/// Führt ein Kommando aus und gibt die erste Zeile der Version zurück
fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Vergleicht zwei Versionen mit semver (für Programme)
fn check_version(installed: &str, required: &str) -> bool {
    let i = Version::parse(installed).ok();
    let r = Version::parse(required).ok();
    match (i, r) {
        (Some(i), Some(r)) => i >= r,
        _ => false,
    }
}

/// Vergleicht Kernel-Versionen (numerisch)
fn check_kernel_version(installed: &str, required: &str) -> bool {
    let parse_ver = |v: &str| {
        v.split(['.', '-'])
            .filter_map(|s| s.parse::<u32>().ok())
            .collect::<Vec<_>>()
    };

    let i = parse_ver(installed);
    let r = parse_ver(required);

    for (a, b) in i.iter().zip(r.iter()) {
        if a > b {
            return true;
        } else if a < b {
            return false;
        }
    }
    i.len() >= r.len()
}

/// Führt eine Version-Prüfung durch
fn ver_check(program: &str, cmd: &str, min_version: &str) -> bool {
    match run_command(cmd, &["--version"]) {
        Some(output) => {
            let ver = output
                .lines()
                .next()
                .unwrap_or("")
                .split_whitespace()
                .last()
                .unwrap_or("");
            if check_version(ver, min_version) {
                println!("OK:    {:<12} {:<8} >= {}", program, ver, min_version);
                true
            } else {
                eprintln!(
                    "ERROR: {:<12} version {} is too old ({} required)",
                    program, ver, min_version
                );
                false
            }
        }
        None => {
            eprintln!("ERROR: Cannot find {}", program);
            false
        }
    }
}

/// Führt die Kernel-Prüfung durch
fn ver_kernel(min_version: &str) -> bool {
    let kernel = run_command("uname", &["-r"]).unwrap_or_default();
    if check_kernel_version(&kernel, min_version) {
        println!("OK:    Linux Kernel {} >= {}", kernel, min_version);
        true
    } else {
        eprintln!(
            "ERROR: Linux Kernel {} is too old ({} required)",
            kernel, min_version
        );
        false
    }
}

/// Lädt die LFS-Seite und führt alle Versionsprüfungen aus
pub fn run_version_checks_from_html(url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let html_text = reqwest::blocking::get(url)?.text()?;
    let document = Html::parse_document(&html_text);
    let selector = Selector::parse("pre").unwrap();

    let mut ok = true;

    for element in document.select(&selector) {
        let pre_text = element.text().collect::<Vec<_>>().join("\n");

        for line in pre_text.lines() {
            let line = line.trim();
            if line.starts_with("ver_check") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let prog = parts[1];
                    let cmd = parts[2];
                    let ver = parts[3];
                    ok &= ver_check(prog, cmd, ver);
                }
            } else if line.starts_with("ver_kernel") {
                if let Some(ver) = line.split_whitespace().nth(1) {
                    ok &= ver_kernel(ver);
                }
            }
        }
    }

    // Alias-Checks
    let alias_check = |cmd: &str, expected: &str| {
        if let Some(output) = run_command(cmd, &["--version"]) {
            if output.to_lowercase().contains(&expected.to_lowercase()) {
                println!("OK:    {:<4} is {}", cmd, expected);
            } else {
                eprintln!("ERROR: {:<4} is NOT {}", cmd, expected);
            }
        }
    };

    alias_check("awk", "GNU");
    alias_check("yacc", "Bison");
    alias_check("sh", "Bash");

    // Compiler-Test
    if run_command("g++", &["--version"]).is_some() {
        println!("OK:    g++ works");
    } else {
        eprintln!("ERROR: g++ does NOT work");
        ok = false;
    }

    // nproc-Test
    let nproc = run_command("nproc", &[]).unwrap_or_default();
    if nproc.is_empty() {
        eprintln!("ERROR: nproc is not available or empty");
        ok = false;
    } else {
        println!("OK:    nproc reports {} logical cores available", nproc);
    }

    if !ok {
        eprintln!("Some version checks failed.");
    }

    Ok(ok)
}
