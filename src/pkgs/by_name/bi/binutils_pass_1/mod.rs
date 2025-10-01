// MLFS metadata: stage: cross-toolchain, variant: Pass 1

use crate::pkgs::package::{OptimizationSettings, PackageDefinition};

pub fn definition() -> PackageDefinition {
    let mut pkg = PackageDefinition::new("Binutils", "2.45");
    pkg.source =
        Some("https://sourceware.org/pub/binutils/releases/binutils-2.45.tar.xz".to_string());
    pkg.md5 = Some("dee5b4267e0305a99a3c9d6131f45759".to_string());
    pkg.configure_args = Vec::new();
    pkg.build_commands = vec![
        "mkdir -v build".to_string(),
        "cd       build".to_string(),
        "../configure --prefix=$LFS/tools \\".to_string(),
        "--with-sysroot=$LFS \\".to_string(),
        "--target=$LFS_TGT   \\".to_string(),
        "--disable-nls       \\".to_string(),
        "--enable-gprofng=no \\".to_string(),
        "--disable-werror    \\".to_string(),
        "--enable-new-dtags  \\".to_string(),
        "--enable-default-hash-style=gnu".to_string(),
        "make".to_string(),
    ];
    pkg.install_commands = vec!["make install".to_string()];
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
