use std::io::{self, Write};

pub struct UI;

impl UI {
    pub fn new() -> Self {
        Self
    }

    pub fn print_banner(&self) {
        println!();
        println!("{}╔══════════════════════════════════════════════════════════════╗{}", 
                 "\x1b[35m", "\x1b[0m");
        println!("{}║                                                              ║{}", 
                 "\x1b[35m", "\x1b[0m");
        println!("{}║  {}🌸 MLFS Bootstrap - Multilib Linux From Scratch 🌸{}       ║{}", 
                 "\x1b[35m", "\x1b[95m", "\x1b[35m", "\x1b[0m");
        println!("{}║                                                              ║{}", 
                 "\x1b[35m", "\x1b[0m");
        println!("{}║  {}💖 Catgirl-powered package management system 💖{}          ║{}", 
                 "\x1b[35m", "\x1b[96m", "\x1b[35m", "\x1b[0m");
        println!("{}║                                                              ║{}", 
                 "\x1b[35m", "\x1b[0m");
        println!("{}╚══════════════════════════════════════════════════════════════╝{}", 
                 "\x1b[35m", "\x1b[0m");
        println!();
    }

    pub fn success(&self, message: &str) {
        println!("{}✨ {}{}", "\x1b[92m", message, "\x1b[0m");
    }

    pub fn info(&self, message: &str) {
        println!("{}🔮 {}{}", "\x1b[94m", message, "\x1b[0m");
    }

    pub fn warning(&self, message: &str) {
        println!("{}⚠️  {}{}", "\x1b[93m", message, "\x1b[0m");
    }

    pub fn error(&self, message: &str) {
        eprintln!("{}💥 {}{}", "\x1b[91m", message, "\x1b[0m");
    }

    pub fn step(&self, step: usize, total: usize, message: &str) {
        println!("{}[{}/{}]{} {}🚀 {}{}", 
                 "\x1b[96m", step, total, "\x1b[0m",
                 "\x1b[95m", message, "\x1b[0m");
    }

    pub fn package_start(&self, package: &str, pass: &str) {
        println!();
        println!("{}═══════════════════════════════════════════════════════════════{}", 
                 "\x1b[95m", "\x1b[0m");
        println!("{}🎯 Building: {}{} {}({}{}){}", 
                 "\x1b[95m", "\x1b[97m", package, "\x1b[94m", pass, "\x1b[95m", "\x1b[0m");
        println!("{}═══════════════════════════════════════════════════════════════{}", 
                 "\x1b[95m", "\x1b[0m");
    }

    pub fn package_complete(&self, package: &str, pass: &str) {
        println!("{}✅ Completed: {}{} {}({}{}){}", 
                 "\x1b[92m", "\x1b[97m", package, "\x1b[94m", pass, "\x1b[92m", "\x1b[0m");
        println!();
    }

    pub fn download_progress(&self, package: &str, url: &str) {
        println!("{}📥 Downloading {}{}: {}{}{}", 
                 "\x1b[93m", "\x1b[97m", package, "\x1b[36m", url, "\x1b[0m");
    }

    pub fn extract_progress(&self, package: &str) {
        println!("{}📦 Extracting {}{}{}", 
                 "\x1b[93m", "\x1b[97m", package, "\x1b[0m");
    }

    pub fn configure_progress(&self, flags: &[String]) {
        println!("{}⚙️  Configuring with: {}{}{}", 
                 "\x1b[94m", "\x1b[36m", flags.join(" "), "\x1b[0m");
    }

    pub fn make_progress(&self, flags: &[String]) {
        println!("{}🔨 Building with: {}{}{}", 
                 "\x1b[94m", "\x1b[36m", flags.join(" "), "\x1b[0m");
    }

    pub fn install_progress(&self) {
        println!("{}📥 Installing...{}", "\x1b[94m", "\x1b[0m");
    }

    pub fn dev_mode_info(&self, temp_dir: &Path) {
        println!("{}🧪 Dev Mode Active{}", "\x1b[93m", "\x1b[0m");
        println!("{}   Temp directory: {}{}{}", 
                 "\x1b[93m", "\x1b[36m", temp_dir.display(), "\x1b[0m");
    }

    pub fn cleanup_info(&self) {
        println!("{}🧹 Cleaning up temporary directories...{}", "\x1b[93m", "\x1b[0m");
    }

    pub fn prompt_continue(&self, message: &str) -> io::Result<bool> {
        print!("{}💭 {}{} [y/N]: ", "\x1b[95m", message, "\x1b[0m");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
    }

    pub fn print_package_list(&self, packages: &HashMap<String, crate::package::PackageConfig>) {
        println!();
        println!("{}📋 Available Packages:{}", "\x1b[95m", "\x1b[0m");
        println!("{}─────────────────────────────────────────────────────{}", "\x1b[95m", "\x1b[0m");
        
        for (name, package) in packages {
            let passes_str = package.build_passes.iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>()
                .join(", ");
            
            println!("{}📦 {}{} {}v{}{} {}(passes: {}){}",
                     "\x1b[96m", "\x1b[97m", name, "\x1b[94m", package.version, "\x1b[0m",
                     "\x1b[93m", passes_str, "\x1b[0m");
            
            if !package.dependencies.is_empty() {
                println!("{}   └─ depends: {}{}{}", 
                         "\x1b[90m", "\x1b[36m", package.dependencies.join(", "), "\x1b[0m");
            }
        }
        println!();
    }

    pub fn command_output(&self, cmd: &str, success: bool, output: &str) {
        if success {
            println!("{}✓{} {}", "\x1b[92m", "\x1b[0m", cmd);
        } else {
            println!("{}✗{} {}", "\x1b[91m", "\x1b[0m", cmd);
            if !output.is_empty() {
                println!("{}   Error: {}{}", "\x1b[91m", output, "\x1b[0m");
            }
        }
    }

    pub fn progress_bar(&self, current: usize, total: usize, label: &str) {
        let percentage = (current * 100) / total;
        let filled = (current * 30) / total;
        let empty = 30 - filled;
        
        print!("\r{}{}[{}{}{}] {}%{} {}", 
               "\x1b[95m", "\x1b[97m",
               "█".repeat(filled), "░".repeat(empty),
               "\x1b[97m", percentage, "\x1b[0m", label);
        io::stdout().flush().unwrap();
        
        if current == total {
            println!();
        }
    }
}