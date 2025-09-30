use std::process::Command;

/// Führt ein Kommando aus und gibt stdout zurück
fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Vergleicht Versionen (semver für Programme)
fn check_version(installed: &str, required: &str) -> bool {
    match (
        semver::Version::parse(installed),
        semver::Version::parse(required),
    ) {
        (Ok(i), Ok(r)) => i >= r,
        _ => false,
    }
}

/// Prüft einen <pre>-Block auf Versionen
pub fn run_version_checks_from_block(block: &str) -> bool {
    let mut ok = true;

    for line in block.lines() {
        let line = line.trim();
        if line.starts_with("ver_check") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let prog = parts[1];
                let cmd = parts[2];
                let ver = parts[3];
                let installed = run_command(cmd, &["--version"]).unwrap_or_default();
                let ver_inst = installed
                    .lines()
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .last()
                    .unwrap_or("");
                if check_version(ver_inst, ver) {
                    println!("OK: {} {} >= {}", prog, ver_inst, ver);
                } else {
                    eprintln!("ERROR: {} {} < {}", prog, ver_inst, ver);
                    ok = false;
                }
            }
        } else if line.starts_with("ver_kernel") {
            if let Some(ver) = line.split_whitespace().nth(1) {
                let kernel = run_command("uname", &["-r"]).unwrap_or_default();
                let installed = kernel
                    .split(['-', '.'])
                    .filter_map(|s| s.parse::<u32>().ok())
                    .collect::<Vec<_>>();
                let required = ver
                    .split(['-', '.'])
                    .filter_map(|s| s.parse::<u32>().ok())
                    .collect::<Vec<_>>();
                if installed >= required {
                    println!("OK: Linux Kernel {} >= {}", kernel, ver);
                } else {
                    eprintln!("ERROR: Linux Kernel {} < {}", kernel, ver);
                    ok = false;
                }
            }
        }
    }

    ok
}
