use std::process::Command;
use std::str::FromStr;

pub fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn check_version(installed: &str, required: &str) -> bool {
    let parse_ver = |v: &str| {
        v.split(|c| c == '.' || c == '-')
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

pub fn ver_check(program: &str, arg: &str, min_version: &str) -> bool {
    match run_command(program, &[arg]) {
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
                println!(
                    "ERROR: {:<12} version {} is too old ({} required)",
                    program, ver, min_version
                );
                false
            }
        }
        None => {
            println!("ERROR: Cannot find {}", program);
            false
        }
    }
}

pub fn run_version_checks() -> bool {
    let mut ok = true;

    ok &= ver_check("bash", "--version", "3.2");
    ok &= ver_check("gcc", "--version", "5.4");
    ok &= ver_check("make", "--version", "4.0");
    ok &= ver_check("tar", "--version", "1.22");

    // Kernel check
    if let Some(kernel) = run_command("uname", &["-r"]) {
        if check_version(&kernel, "5.4") {
            println!("OK:    Linux Kernel {} >= 5.4", kernel);
        } else {
            println!("ERROR: Linux Kernel {} is too old (5.4 required)", kernel);
            ok = false;
        }
    }

    // CPU cores
    let cores = num_cpus::get();
    println!("OK:    {} logical cores available", cores);

    ok
}
