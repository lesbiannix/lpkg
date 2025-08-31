use crate::package::{PackageConfig, BuildPass};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

pub struct LPKGParser;

impl LPKGParser {
    pub fn parse_file(path: &Path) -> io::Result<PackageConfig> {
        let content = fs::read_to_string(path)?;
        let package_name = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid filename"))?;
        
        Self::parse_content(&content, package_name)
    }

    pub fn parse_content(content: &str, package_name: &str) -> io::Result<PackageConfig> {
        let mut config = PackageConfig {
            name: package_name.to_string(),
            version: String::new(),
            url: String::new(),
            archive_format: None,
            build_passes: vec![],
            dependencies: vec![],
            configure_flags: HashMap::new(),
            make_flags: HashMap::new(),
            env_vars: HashMap::new(),
            pre_build_commands: None,
            post_build_commands: None,
            patches: None,
            description: None,
        };

        let mut current_section = String::new();
        let mut current_pass: Option<BuildPass> = None;
        let mut pre_build_commands = HashMap::new();
        let mut post_build_commands = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Section headers [section_name]
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len()-1].to_string();
                current_pass = match current_section.as_str() {
                    "pass1" => Some(BuildPass::Pass1),
                    "pass2" => Some(BuildPass::Pass2), 
                    "final" => Some(BuildPass::Final),
                    _ => None,
                };
                continue;
            }

            // Parse key=value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = Self::parse_value(value.trim());

                match current_section.as_str() {
                    "package" => {
                        Self::parse_package_section(&mut config, key, &value);
                    }
                    "dependencies" => {
                        if key == "depends" {
                            config.dependencies = Self::parse_array(&value);
                        }
                    }
                    "build" => {
                        if key == "passes" {
                            config.build_passes = Self::parse_passes(&value)?;
                        }
                    }
                    "patches" => {
                        if key == "files" {
                            config.patches = Some(Self::parse_array(&value));
                        }
                    }
                    "pass1" | "pass2" | "final" => {
                        if let Some(pass) = current_pass.clone() {
                            Self::parse_pass_section(&mut config, &mut pre_build_commands, 
                                                   &mut post_build_commands, pass, key, &value);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Store build commands if any were defined
        if !pre_build_commands.is_empty() {
            config.pre_build_commands = Some(pre_build_commands);
        }
        if !post_build_commands.is_empty() {
            config.post_build_commands = Some(post_build_commands);
        }

        Self::validate_config(&config)?;
        Ok(config)
    }

    fn parse_package_section(config: &mut PackageConfig, key: &str, value: &str) {
        match key {
            "version" => config.version = value.to_string(),
            "description" => config.description = Some(value.to_string()),
            "url" => config.url = value.to_string(),
            "archive_format" => config.archive_format = Some(value.to_string()),
            _ => {}
        }
    }

    fn parse_pass_section(
        config: &mut PackageConfig,
        pre_build_commands: &mut HashMap<BuildPass, Vec<String>>,
        post_build_commands: &mut HashMap<BuildPass, Vec<String>>,
        pass: BuildPass,
        key: &str,
        value: &str,
    ) {
        match key {
            "configure_flags" => {
                config.configure_flags.insert(pass, Self::parse_array(value));
            }
            "make_flags" => {
                config.make_flags.insert(pass, Self::parse_array(value));
            }
            "pre_build" => {
                pre_build_commands.insert(pass, Self::parse_array(value));
            }
            "post_build" => {
                post_build_commands.insert(pass, Self::parse_array(value));
            }
            _ => {
                // Treat as environment variable
                let env_map = config.env_vars.entry(pass).or_insert_with(HashMap::new);
                env_map.insert(key.to_string(), value.to_string());
            }
        }
    }

    fn parse_value(value: &str) -> String {
        let value = value.trim();
        
        // Remove quotes if present
        if (value.starts_with('"') && value.ends_with('"')) ||
           (value.starts_with('\'') && value.ends_with('\'')) {
            value[1..value.len()-1].to_string()
        } else {
            value.to_string()
        }
    }

    fn parse_array(value: &str) -> Vec<String> {
        let value = value.trim();
        
        if value.starts_with('[') && value.ends_with(']') {
            // Array format: ["item1", "item2", "item3"]
            let inner = &value[1..value.len()-1];
            inner.split(',')
                .map(|s| Self::parse_value(s.trim()))
                .filter(|s| !s.is_empty())
                .collect()
        } else if value.contains(',') {
            // Comma-separated: item1, item2, item3
            value.split(',')
                .map(|s| Self::parse_value(s.trim()))
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // Single value or space-separated
            value.split_whitespace()
                .map(|s| s.to_string())
                .collect()
        }
    }

    fn parse_passes(value: &str) -> io::Result<Vec<BuildPass>> {
        Self::parse_array(value)
            .into_iter()
            .map(|s| s.parse::<BuildPass>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn validate_config(config: &PackageConfig) -> io::Result<()> {
        if config.name.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Package name is required"));
        }
        if config.version.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Package version is required"));
        }
        if config.url.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Package URL is required"));
        }
        if config.build_passes.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "At least one build pass is required"));
        }

        Ok(())
    }

    pub fn load_packages_from_directory(dir: &str) -> io::Result<HashMap<String, PackageConfig>> {
        let mut packages = HashMap::new();
        let dir_path = Path::new(dir);

        if !dir_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Packages directory '{}' not found", dir)
            ));
        }

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("lpkg") {
                match Self::parse_file(&path) {
                    Ok(config) => {
                        packages.insert(config.name.clone(), config);
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(packages)
    }
}