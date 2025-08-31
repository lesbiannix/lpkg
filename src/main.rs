mod config;
mod package;
mod bootstrap;
mod ui;
mod lpkg;

use crate::bootstrap::MLFSBootstrap;
use crate::ui::UI;
use clap::{Arg, Command};
use std::io;

fn main() -> io::Result<()> {
    let matches = Command::new("mlfs-bootstrap")
        .version("0.1.0")
        .author("Anonymous Catgirl 🐱")
        .about("🌟 Multilib Linux From Scratch Bootstrap Tool")
        .arg(Arg::new("packages-dir")
            .short('d')
            .long("packages-dir")
            .value_name("DIR")
            .help("Directory containing .lpkg files")
            .default_value("packages"))
        .arg(Arg::new("dev")
            .long("dev")
            .help("Enable development mode (uses temporary directories)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("lfs-dir")
            .long("lfs-dir")
            .value_name("DIR")
            .help("LFS root directory")
            .default_value("/mnt/lfs"))
        .arg(Arg::new("package")
            .short('p')
            .long("package")
            .value_name("NAME")
            .help("Build specific package"))
        .arg(Arg::new("pass")
            .long("pass")
            .value_name("PASS")
            .help("Build pass (pass1, pass2, final)")
            .default_value("pass1"))
        .arg(Arg::new("init")
            .long("init")
            .help("Initialize directories and create sample packages")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("list")
            .short('l')
            .long("list")
            .help("List available packages")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("dependency-order")
            .long("dep-order")
            .help("Show build order based on dependencies")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    let ui = UI::new();
    ui.print_banner();

    let dev_mode = matches.get_flag("dev");
    let packages_dir = matches.get_one::<String>("packages-dir").unwrap();
    let lfs_dir = if dev_mode { 
        None 
    } else { 
        Some(matches.get_one::<String>("lfs-dir").unwrap().as_str()) 
    };

    let mut bootstrap = MLFSBootstrap::new(lfs_dir, dev_mode)?;

    if matches.get_flag("init") {
        ui.info("Initializing MLFS environment...");
        bootstrap.init_directories()?;
        bootstrap.create_sample_packages(packages_dir)?;
        ui.success(&format!("Sample packages created in {}/", packages_dir));
        return Ok(());
    }

    // Load packages from .lpkg files
    bootstrap.load_packages_from_dir(packages_dir)?;

    if matches.get_flag("list") {
        bootstrap.list_packages();
        return Ok(());
    }

    if matches.get_flag("dependency-order") {
        bootstrap.show_build_order()?;
        return Ok(());
    }

    if let Some(package_name) = matches.get_one::<String>("package") {
        let pass_str = matches.get_one::<String>("pass").unwrap();
        let pass = pass_str.parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        bootstrap.init_directories()?;
        bootstrap.build_package(package_name, pass)?;
    } else {
        ui.info("Starting full toolchain bootstrap...");
        bootstrap.init_directories()?;
        bootstrap.build_toolchain()?;
    }

    if dev_mode {
        bootstrap.cleanup_temp_dirs()?;
    }

    Ok(())
}