use serde::{Deserialize, Serialize};

/// High-level description of a package managed by LPKG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDefinition {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub md5: Option<String>,
    pub configure_args: Vec<String>,
    pub build_commands: Vec<String>,
    pub install_commands: Vec<String>,
    pub dependencies: Vec<String>,
    pub optimizations: OptimizationSettings,
}

impl PackageDefinition {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            source: None,
            md5: None,
            configure_args: Vec::new(),
            build_commands: Vec::new(),
            install_commands: Vec::new(),
            dependencies: Vec::new(),
            optimizations: OptimizationSettings::default(),
        }
    }
}

/// Tunable compiler and linker flags applied during package builds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    pub enable_lto: bool,
    pub enable_pgo: bool,
    pub cflags: Vec<String>,
    pub ldflags: Vec<String>,
    pub profdata: Option<String>,
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            enable_lto: true,
            enable_pgo: true,
            cflags: vec![
                "-O3".to_string(),
                "-flto".to_string(),
                "-fprofile-generate".to_string(),
            ],
            ldflags: vec!["-flto".to_string(), "-fprofile-generate".to_string()],
            profdata: None,
        }
    }
}

impl OptimizationSettings {
    /// Convenience helper for disabling instrumentation once profile data has been gathered.
    pub fn for_pgo_replay(profdata: impl Into<String>) -> Self {
        Self {
            enable_lto: true,
            enable_pgo: true,
            cflags: vec![
                "-O3".to_string(),
                "-flto".to_string(),
                "-fprofile-use".to_string(),
            ],
            ldflags: vec!["-flto".to_string(), "-fprofile-use".to_string()],
            profdata: Some(profdata.into()),
        }
    }
}
