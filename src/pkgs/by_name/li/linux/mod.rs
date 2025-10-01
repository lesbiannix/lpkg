// MLFS metadata: stage: cross-toolchain

use crate::pkgs::package::{OptimizationSettings, PackageDefinition};

pub fn definition() -> PackageDefinition {
    let mut pkg = PackageDefinition::new("Linux", "6.16.9 API Headers");
    pkg.source = None;
    pkg.md5 = None;
    pkg.configure_args = Vec::new();
    pkg.build_commands = vec![
        "make mrproper".to_string(),
        "make headers".to_string(),
        "find usr/include -type f ! -name '*.h' -delete".to_string(),
        "cp -rv usr/include $LFS/usr".to_string(),
    ];
    pkg.install_commands = Vec::new();
    pkg.dependencies = Vec::new();
    let profdata = None;
    let profdata_clone = profdata.clone();
    pkg.optimizations = match profdata_clone {
        Some(path) => OptimizationSettings::for_pgo_replay(path),
        None => OptimizationSettings::default(),
    };
    pkg.optimizations.enable_lto = true;
    pkg.optimizations.enable_pgo = true;
    pkg.optimizations.cflags = vec!["-O3".to_string(), "-flto".to_string()];
    pkg.optimizations.ldflags = vec!["-flto".to_string()];
    pkg.optimizations.profdata = profdata;
    pkg
}
