// src/bootstrap.rs
use crate::package::{PackageConfig, BuildPass, BuildContext};
use crate::lpkg::LPKGParser;
use crate::ui::UI;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io;

#[derive(Debug)]
pub struct MLFSBootstrap {
    pub toolchain_dir: PathBuf,
    pub sources_dir: PathBuf,
    pub build_dir: PathBuf,
    pub target_triplet: String,
    pub host_triplet: String,
    pub packages: HashMap<String, PackageConfig>,
    pub global_env_vars: HashMap<String, String>,
    pub dev_mode: bool,
    pub temp_dirs: Vec<PathBuf>,
    pub ui: UI,
}

impl MLFSBootstrap {
    pub fn new(lfs_dir: Option<&str>, dev_mode: bool) -> io::Result<Self> {
        let ui = UI::new();
        
        let (toolchain_dir, sources_dir, build_dir, temp_dirs) = if dev_mode {
            let temp_base = std::env::temp_dir().join(format!("mlfs-dev-{}", 
                std::process::id()));
            
            ui.dev_mode_info(&temp_base);
            
            let toolchain = temp_base.join("tools");
            let sources = temp_base.join("sources");
            let build = temp_base.join("build");
            
            (toolchain, sources, build, vec![temp_base])
        } else {
            let lfs_path = PathBuf::from(lfs_dir.unwrap_or("/mnt/lfs"));
            (
                lfs_path.join("tools"),
                lfs_path.join("sources"),
                lfs_path.join("build"),
                vec![]
            )
        };

        let mut global_env = HashMap::new();
        global_env.insert("LC_ALL".to_string(), "POSIX".to_string());
        global_env.insert("PATH".to_string(), format!("{}:{}", 
            toolchain_dir.join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        ));
        
        Ok(Self {
            toolchain_dir,
            sources_dir,
            build_dir,
            target_triplet: "x86_64-lfs-linux-gnu".to_string(),
            host_triplet: "x86_64-pc-linux-gnu".to_string(),
            packages: HashMap::new(),
            global_env_vars: global_env,
            dev_mode,
            temp_dirs,
            ui,
        })
    }

    pub fn init_directories(&self) -> io::Result<()> {
        self.ui.info("Initializing MLFS directories...");
        fs::create_dir_all(&self.toolchain_dir)?;
        fs::create_dir_all(&self.sources_dir)?;
        fs::create_dir_all(&self.build_dir)?;
        
        // Create bin directory in toolchain
        fs::create_dir_all(self.toolchain_dir.join("bin"))?;
        fs::create_dir_all(self.toolchain_dir.join("lib"))?;
        fs::create_dir_all(self.toolchain_dir.join("include"))?;
        
        self.ui.success("Directories created successfully!");
        Ok(())
    }

    pub fn load_packages_from_dir(&mut self, packages_dir: &str) -> io::Result<()> {
        self.ui.info(&format!("Loading packages from {}/", packages_dir));
        self.packages = LPKGParser::load_packages_from_directory(packages_dir)?;
        self.ui.success(&format!("Loaded {} packages", self.packages.len()));
        Ok(())
    }

    pub fn create_sample_packages(&self, packages_dir: &str) -> io::Result<()> {
        fs::create_dir_all(packages_dir)?;

        let packages = vec![
            ("binutils.lpkg", LPKGParser::create_sample_binutils()),
            ("gcc.lpkg", LPKGParser::create_sample_gcc()),
            ("linux-headers.lpkg", LPKGParser::create_sample_linux_headers()),
            ("glibc.lpkg", LPKGParser::create_sample_glibc()),
        ];

        for (filename, content) in packages {
            let path = Path::new(packages_dir).join(filename);
            fs::write(path, content)?;
        }

        Ok(())
    }

    pub fn build_package(&self, package_name: &str, pass: BuildPass) -> io::Result<()> {
        let package = self.packages.get(package_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, 
                format!("Package '{}' not found", package_name)))?;

        if !package.has_pass(&pass) {
            self.ui.warning(&format!("Skipping {} for pass {}", package_name, pass));
            return Ok(());
        }

        self.ui.package_start(package_name, &pass.to_string());

        // Check dependencies
        self.check_dependencies(package)?;

        // Download and extract
        let archive_path = self.download_package(package)?;
        let source_dir = self.extract_package(package, &archive_path)?;

        // Create build directory for this pass
        let build_dir = self.build_dir.join(format!("{}-{}", package_name, pass));
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir)?;
        }
        fs::create_dir_all(&build_dir)?;

        // Create build context
        let context = BuildContext::new(
            package.clone(),
            pass.clone(),
            source_dir,
            build_dir.clone(),
            self.toolchain_dir.clone(),
            self.target_triplet.clone(),
            self.host_triplet.clone(),
            self.global_env_vars.clone(),
        );

        // Execute build steps
        self.execute_pre_build_commands(&context)?;
        self.configure_package(&context)?;
        self.build_package_make(&context)?;
        self.install_package(&context)?;
        self.execute_post_build_commands(&context)?;

        self.ui.package_complete(package_name, &pass.to_string());
        Ok(())
    }

    fn check_dependencies(&self, package: &PackageConfig) -> io::Result<()> {
        for dep in &package.dependencies {
            if !self.packages.contains_key(dep) {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Dependency '{}' not found for package '{}'", dep, package.name)
                ));
            }
        }
        Ok(())
    }

    fn download_package(&self, package: &PackageConfig) -> io::Result<PathBuf> {
        let filename = package.get_archive_name();
        let dest_path = self.sources_dir.join(&filename);

        if dest_path.exists() {
            self.ui.info(&format!("Package {} already downloaded", package.name));
            return Ok(dest_path);
        }

        self.ui.download_progress(&package.name, &package.url);
        
        let output = Command::new("wget")
            .args(&["-P", self.sources_dir.to_str().unwrap(), &package.url])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to download {}: {}", 
                    package.name, String::from_utf8_lossy(&output.stderr))
            ));
        }

        Ok(dest_path)
    }

    fn extract_package(&self, package: &PackageConfig, archive_path: &Path) -> io::Result<PathBuf> {
        let extract_dir = self.build_dir.join(format!("{}-source", package.name));
        
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir)?;
        }
        fs::create_dir_all(&extract_dir)?;

        self.ui.extract_progress(&package.name);

        let tar_cmd = match package.archive_format.as_deref() {
            Some("tar.xz") => "tar -xJf",
            Some("tar.gz") => "tar -xzf", 
            Some("tar.bz2") => "tar -xjf",
            _ => {
                // Auto-detect based on extension
                let ext = archive_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                match ext {
                    "xz" => "tar -xJf",
                    "gz" => "tar -xzf",
                    "bz2" => "tar -xjf",
                    _ => "tar -xf",
                }
            }
        };

        let output = Command::new("sh")
            .args(&["-c", &format!("{} {} -C {} --strip-components=1", 
                tar_cmd, archive_path.display(), extract_dir.display())])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to extract {}: {}", 
                    package.name, String::from_utf8_lossy(&output.stderr))
            ));
        }

        Ok(extract_dir)
    }

    fn execute_pre_build_commands(&self, context: &BuildContext) -> io::Result<()> {
        let commands = context.package.get_pre_build_commands(&context.pass);
        if commands.is_empty() {
            return Ok(());
        }

        self.ui.info("Executing pre-build commands...");
        for cmd in commands {
            self.execute_command(&cmd, &context.source_dir, &context.env_vars)?;
        }
        Ok(())
    }

    fn configure_package(&self, context: &BuildContext) -> io::Result<()> {
        if !context.should_configure() {
            return Ok(());
        }

        let flags = context.package.get_configure_flags(&context.pass);
        self.ui.configure_progress(&flags);

        let expanded_flags = self.expand_variables(&flags, &context.env_vars);
        
        let mut cmd = Command::new("../configure");
        cmd.args(&expanded_flags)
            .current_dir(&context.build_dir)
            .envs(&context.env_vars);

        let output = cmd.output()?;
        self.ui.command_output("configure", output.status.success(), 
            &String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Configure failed for {}", context.package.name)
            ));
        }

        Ok(())
    }

    fn build_package_make(&self, context: &BuildContext) -> io::Result<()> {
        let flags = context.package.get_make_flags(&context.pass);
        self.ui.make_progress(&flags);

        let expanded_flags = self.expand_variables(&flags, &context.env_vars);
        
        let mut cmd = Command::new("make");
        cmd.args(&expanded_flags)
            .current_dir(&context.build_dir)
            .envs(&context.env_vars);

        let output = cmd.output()?;
        self.ui.command_output("make", output.status.success(), 
            &String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Make failed for {}", context.package.name)
            ));
        }

        Ok(())
    }

    fn install_package(&self, context: &BuildContext) -> io::Result<()> {
        self.ui.install_progress();
        
        let mut cmd = Command::new("make");
        cmd.args(&["install"])
            .current_dir(&context.build_dir)
            .envs(&context.env_vars);

        let output = cmd.output()?;
        self.ui.command_output("make install", output.status.success(), 
            &String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Install failed for {}", context.package.name)
            ));
        }

        Ok(())
    }

    fn execute_post_build_commands(&self, context: &BuildContext) -> io::Result<()> {
        let commands = context.package.get_post_build_commands(&context.pass);
        if commands.is_empty() {
            return Ok(());
        }

        self.ui.info("Executing post-build commands...");
        for cmd in commands {
            self.execute_command(&cmd, &context.build_dir, &context.env_vars)?;
        }
        Ok(())
    }

    fn execute_command(&self, cmd: &str, working_dir: &Path, env_vars: &HashMap<String, String>) -> io::Result<()> {
        let expanded_cmd = self.expand_variables(&[cmd.to_string()], env_vars)[0].clone();
        
        let output = Command::new("sh")
            .args(&["-c", &expanded_cmd])
            .current_dir(working_dir)
            .envs(env_vars)
            .output()?;

        self.ui.command_output(&expanded_cmd, output.status.success(), 
            &String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Command failed: {}", expanded_cmd)
            ));
        }

        Ok(())
    }

    fn expand_variables(&self, strings: &[String], env_vars: &HashMap<String, String>) -> Vec<String> {
        strings.iter()
            .map(|s| {
                let mut expanded = s.clone();
                
                // Expand common LFS variables
                expanded = expanded.replace("$LFS_TGT", &self.target_triplet);
                expanded = expanded.replace("$LFS", 
                    &self.toolchain_dir.parent().unwrap().display().to_string());
                
                // Expand environment variables
                for (key, value) in env_vars {
                    expanded = expanded.replace(&format!("${}", key), value);
                }
                
                expanded
            })
            .collect()
    }

    pub fn build_toolchain(&self) -> io::Result<()> {
        let build_order = self.calculate_build_order()?;
        let total_steps = build_order.len();

        self.ui.info(&format!("Building {} packages in dependency order", total_steps));

        for (i, (package_name, pass)) in build_order.iter().enumerate() {
            self.ui.step(i + 1, total_steps, &format!("{} ({})", package_name, pass));
            self.build_package(package_name, pass.clone())?;
        }

        self.ui.success("🎉 Toolchain bootstrap complete!");
        Ok(())
    }

    fn calculate_build_order(&self) -> io::Result<Vec<(String, BuildPass)>> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        // Standard LFS build order for toolchain
        let base_order = vec![
            ("binutils", BuildPass::Pass1),
            ("gcc", BuildPass::Pass1), 
            ("linux-headers", BuildPass::Pass1),
            ("glibc", BuildPass::Pass1),
            ("binutils", BuildPass::Pass2),
            ("gcc", BuildPass::Pass2),
        ];

        for (pkg_name, pass) in base_order {
            if self.packages.contains_key(pkg_name) {
                let key = format!("{}:{}", pkg_name, pass);
                if !visited.contains(&key) {
                    self.topological_sort(pkg_name, &pass, &mut order, &mut visited, &mut temp_visited)?;
                }
            }
        }

        Ok(order)
    }

    fn topological_sort(
        &self,
        package_name: &str,
        pass: &BuildPass,
        order: &mut Vec<(String, BuildPass)>,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
    ) -> io::Result<()> {
        let key = format!("{}:{}", package_name, pass);
        
        if temp_visited.contains(&key) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Circular dependency detected involving {}", package_name)
            ));
        }
        
        if visited.contains(&key) {
            return Ok(());
        }

        temp_visited.insert(key.clone());

        if let Some(package) = self.packages.get(package_name) {
            // Process dependencies first
            for dep in &package.dependencies {
                if self.packages.contains_key(dep) {
                    self.topological_sort(dep, pass, order, visited, temp_visited)?;
                }
            }
        }

        temp_visited.remove(&key);
        visited.insert(key);
        order.push((package_name.to_string(), pass.clone()));

        Ok(())
    }

    pub fn list_packages(&self) {
        self.ui.print_package_list(&self.packages);
    }

    pub fn show_build_order(&self) -> io::Result<()> {
        let order = self.calculate_build_order()?;
        
        println!();
        self.ui.info("Build order:");
        println!("{}─────────────────────────────────────{}", "\x1b[95m", "\x1b[0m");
        
        for (i, (package, pass)) in order.iter().enumerate() {
            println!("{}{}. {}{} {}({}{}){}", 
                "\x1b[96m", i + 1, "\x1b[97m", package, 
                "\x1b[94m", pass, "\x1b[0m");
        }
        println!();
        
        Ok(())
    }

    pub fn cleanup_temp_dirs(&self) -> io::Result<()> {
        if self.dev_mode && !self.temp_dirs.is_empty() {
            self.ui.cleanup_info();
            for temp_dir in &self.temp_dirs {
                if temp_dir.exists() {
                    fs::remove_dir_all(temp_dir)?;
                }
            }
            self.ui.success("Temporary directories cleaned up");
        }
        Ok(())
    }
}