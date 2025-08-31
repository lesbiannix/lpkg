// src/package.rs
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BuildPass {
    Pass1,
    Pass2,
    Final,
}

impl std::fmt::Display for BuildPass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildPass::Pass1 => write!(f, "pass1"),
            BuildPass::Pass2 => write!(f, "pass2"),
            BuildPass::Final => write!(f, "final"),
        }
    }
}

impl std::str::FromStr for BuildPass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pass1" => Ok(BuildPass::Pass1),
            "pass2" => Ok(BuildPass::Pass2),
            "final" => Ok(BuildPass::Final),
            _ => Err(format!("Invalid build pass: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    pub url: String,
    pub archive_format: Option<String>, // tar.xz, tar.gz, etc.
    pub build_passes: Vec<BuildPass>,
    pub dependencies: Vec<String>,
    pub configure_flags: HashMap<BuildPass, Vec<String>>,
    pub make_flags: HashMap<BuildPass, Vec<String>>,
    pub env_vars: HashMap<BuildPass, HashMap<String, String>>,
    pub pre_build_commands: Option<HashMap<BuildPass, Vec<String>>>,
    pub post_build_commands: Option<HashMap<BuildPass, Vec<String>>>,
    pub patches: Option<Vec<String>>,
    pub description: Option<String>,
}

impl PackageConfig {
    pub fn get_archive_name(&self) -> String {
        self.url.split('/').last().unwrap_or(&self.name).to_string()
    }

    pub fn get_source_dir_name(&self) -> String {
        let archive = self.get_archive_name();
        // Remove common archive extensions
        archive
            .replace(".tar.xz", "")
            .replace(".tar.gz", "")
            .replace(".tar.bz2", "")
            .replace(".tgz", "")
    }

    pub fn has_pass(&self, pass: &BuildPass) -> bool {
        self.build_passes.contains(pass)
    }

    pub fn get_configure_flags(&self, pass: &BuildPass) -> Vec<String> {
        self.configure_flags.get(pass).cloned().unwrap_or_default()
    }

    pub fn get_make_flags(&self, pass: &BuildPass) -> Vec<String> {
        self.make_flags.get(pass).cloned().unwrap_or_default()
    }

    pub fn get_env_vars(&self, pass: &BuildPass) -> HashMap<String, String> {
        self.env_vars.get(pass).cloned().unwrap_or_default()
    }

    pub fn get_pre_build_commands(&self, pass: &BuildPass) -> Vec<String> {
        self.pre_build_commands
            .as_ref()
            .and_then(|cmds| cmds.get(pass))
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_post_build_commands(&self, pass: &BuildPass) -> Vec<String> {
        self.post_build_commands
            .as_ref()
            .and_then(|cmds| cmds.get(pass))
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct BuildContext {
    pub package: PackageConfig,
    pub pass: BuildPass,
    pub source_dir: std::path::PathBuf,
    pub build_dir: std::path::PathBuf,
    pub install_prefix: std::path::PathBuf,
    pub target_triplet: String,
    pub host_triplet: String,
    pub env_vars: HashMap<String, String>,
}

impl BuildContext {
    pub fn new(
        package: PackageConfig,
        pass: BuildPass,
        source_dir: std::path::PathBuf,
        build_dir: std::path::PathBuf,
        install_prefix: std::path::PathBuf,
        target_triplet: String,
        host_triplet: String,
        global_env: HashMap<String, String>,
    ) -> Self {
        let mut env_vars = global_env;
        
        // Add package-specific environment variables
        for (key, value) in package.get_env_vars(&pass) {
            env_vars.insert(key, value);
        }

        // Add common build environment variables
        env_vars.insert("LFS_TGT".to_string(), target_triplet.clone());
        env_vars.insert("CONFIG_SITE".to_string(), format!("{}/share/config.site", install_prefix.display()));

        Self {
            package,
            pass,
            source_dir,
            build_dir,
            install_prefix,
            target_triplet,
            host_triplet,
            env_vars,
        }
    }

    pub fn get_configure_path(&self) -> std::path::PathBuf {
        self.source_dir.join("configure")
    }

    pub fn should_configure(&self) -> bool {
        self.get_configure_path().exists() && !self.package.get_configure_flags(&self.pass).is_empty()
    }
}