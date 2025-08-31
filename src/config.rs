// src/config.rs
use crate::package::{PackageConfig, BuildPass};
use std::collections::HashMap;
use std::fs;
use std::io;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MLFSConfig {
    pub target_triplet: String,
    pub host_triplet: String,
    pub global_env_vars: HashMap<String, String>,
    pub packages: Vec<PackageConfig>,
    pub build_order: Vec<BuildOrder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildOrder {
    pub package: String,
    pub pass: BuildPass,
}

impl MLFSConfig {
    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, 
                format!("Failed to parse TOML config: {}", e)))
    }

    pub fn create_sample_config() -> Self {
        let mut global_env = HashMap::new();
        global_env.insert("LC_ALL".to_string(), "POSIX".to_string());
        global_env.insert("MAKEFLAGS".to_string(), "-j4".to_string());

        Self {
            target_triplet: "x86_64-lfs-linux-gnu".to_string(),
            host_triplet: "x86_64-pc-linux-gnu".to_string(),
            global_env_vars: global_env,
            packages: vec![
                create_binutils_config(),
                create_gcc_config(),
                create_linux_headers_config(),
                create_glibc_config(),
            ],
            build_order: vec![
                BuildOrder { package: "binutils".to_string(), pass: BuildPass::Pass1 },
                BuildOrder { package: "gcc".to_string(), pass: BuildPass::Pass1 },
                BuildOrder { package: "linux-headers".to_string(), pass: BuildPass::Pass1 },
                BuildOrder { package: "glibc".to_string(), pass: BuildPass::Pass1 },
                BuildOrder { package: "binutils".to_string(), pass: BuildPass::Pass2 },
                BuildOrder { package: "gcc".to_string(), pass: BuildPass::Pass2 },
            ],
        }
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                format!("Failed to serialize config: {}", e)))?;
        
        fs::write(path, toml_string)?;
        Ok(())
    }
}

fn create_binutils_config() -> PackageConfig {
    let mut configure_flags = HashMap::new();
    let mut make_flags = HashMap::new();

    // Pass 1: Cross-compilation binutils
    configure_flags.insert(BuildPass::Pass1, vec![
        "--target=x86_64-lfs-linux-gnu".to_string(),
        "--prefix=/tools".to_string(),
        "--with-sysroot=$LFS".to_string(),
        "--disable-nls".to_string(),
        "--enable-gprofng=no".to_string(),
        "--disable-werror".to_string(),
        "--enable-64-bit-bfd".to_string(),
        "--enable-multilib".to_string(),
        "--enable-default-hash-style=gnu".to_string(),
    ]);

    // Pass 2: Native binutils
    configure_flags.insert(BuildPass::Pass2, vec![
        "--prefix=/tools".to_string(),
        "--with-lib-path=/tools/lib:/tools/lib32".to_string(),
        "--enable-64-bit-bfd".to_string(),
        "--enable-multilib".to_string(),
        "--enable-default-hash-style=gnu".to_string(),
        "--disable-werror".to_string(),
    ]);

    make_flags.insert(BuildPass::Pass1, vec![]);
    make_flags.insert(BuildPass::Pass2, vec![]);

    PackageConfig {
        name: "binutils".to_string(),
        version: "2.42".to_string(),
        url: "https://ftp.gnu.org/gnu/binutils/binutils-2.42.tar.xz".to_string(),
        archive_format: Some("tar.xz".to_string()),
        build_passes: vec![BuildPass::Pass1, BuildPass::Pass2],
        dependencies: vec![],
        configure_flags,
        make_flags,
        env_vars: HashMap::new(),
        pre_build_commands: None,
        post_build_commands: None,
        patches: None,
        description: Some("GNU Binary Utilities - assembler, linker, and binary tools".to_string()),
    }
}

fn create_gcc_config() -> PackageConfig {
    let mut configure_flags = HashMap::new();
    let mut make_flags = HashMap::new();
    let mut pre_build = HashMap::new();

    // Pass 1: Cross-compiler GCC
    configure_flags.insert(BuildPass::Pass1, vec![
        "--target=$LFS_TGT".to_string(),
        "--prefix=/tools".to_string(),
        "--with-glibc-version=2.38".to_string(),
        "--with-sysroot=$LFS".to_string(),
        "--with-newlib".to_string(),
        "--without-headers".to_string(),
        "--enable-default-pie".to_string(),
        "--enable-default-ssp".to_string(),
        "--enable-initfini-array".to_string(),
        "--disable-nls".to_string(),
        "--disable-shared".to_string(),
        "--disable-multilib".to_string(),
        "--disable-threads".to_string(),
        "--disable-libatomic".to_string(),
        "--disable-libgomp".to_string(),
        "--disable-libquadmath".to_string(),
        "--disable-libssp".to_string(),
        "--disable-libvtv".to_string(),
        "--disable-libstdcxx".to_string(),
        "--enable-languages=c,c++".to_string(),
    ]);

    // Pass 2: Full cross-compiler
    configure_flags.insert(BuildPass::Pass2, vec![
        "--target=$LFS_TGT".to_string(),
        "--prefix=/tools".to_string(),
        "--with-build-sysroot=$LFS".to_string(),
        "--enable-default-pie".to_string(),
        "--enable-default-ssp".to_string(),
        "--enable-initfini-array".to_string(),
        "--disable-nls".to_string(),
        "--disable-multilib".to_string(),
        "--enable-languages=c,c++".to_string(),
        "--disable-libstdcxx-pch".to_string(),
        "--with-system-zlib".to_string(),
    ]);

    // Pre-build commands to download prerequisites
    pre_build.insert(BuildPass::Pass1, vec![
        "contrib/download_prerequisites".to_string(),
    ]);
    pre_build.insert(BuildPass::Pass2, vec![
        "contrib/download_prerequisites".to_string(),
    ]);

    make_flags.insert(BuildPass::Pass1, vec![]);
    make_flags.insert(BuildPass::Pass2, vec![]);

    PackageConfig {
        name: "gcc".to_string(),
        version: "13.2.0".to_string(),
        url: "https://ftp.gnu.org/gnu/gcc/gcc-13.2.0/gcc-13.2.0.tar.xz".to_string(),
        archive_format: Some("tar.xz".to_string()),
        build_passes: vec![BuildPass::Pass1, BuildPass::Pass2],
        dependencies: vec!["binutils".to_string()],
        configure_flags,
        make_flags,
        env_vars: HashMap::new(),
        pre_build_commands: Some(pre_build),
        post_build_commands: None,
        patches: None,
        description: Some("GNU Compiler Collection - C/C++ compiler".to_string()),
    }
}

fn create_linux_headers_config() -> PackageConfig {
    let mut make_flags = HashMap::new();
    let mut env_vars = HashMap::new();
    
    make_flags.insert(BuildPass::Pass1, vec![
        "mrproper".to_string(),
    ]);

    let mut pass1_env = HashMap::new();
    pass1_env.insert("INSTALL_HDR_PATH".to_string(), "/tools".to_string());
    env_vars.insert(BuildPass::Pass1, pass1_env);

    let mut post_build = HashMap::new();
    post_build.insert(BuildPass::Pass1, vec![
        "make headers_install".to_string(),
    ]);

    PackageConfig {
        name: "linux-headers".to_string(),
        version: "6.7.4".to_string(),
        url: "https://www.kernel.org/pub/linux/kernel/v6.x/linux-6.7.4.tar.xz".to_string(),
        archive_format: Some("tar.xz".to_string()),
        build_passes: vec![BuildPass::Pass1],
        dependencies: vec![],
        configure_flags: HashMap::new(),
        make_flags,
        env_vars,
        pre_build_commands: None,
        post_build_commands: Some(post_build),
        patches: None,
        description: Some("Linux kernel headers for userspace development".to_string()),
    }
}

fn create_glibc_config() -> PackageConfig {
    let mut configure_flags = HashMap::new();
    let mut make_flags = HashMap::new();
    let mut env_vars = HashMap::new();

    configure_flags.insert(BuildPass::Pass1, vec![
        "--prefix=/tools".to_string(),
        "--host=$LFS_TGT".to_string(),
        "--build=$(../glibc-2.38/scripts/config.guess)".to_string(),
        "--enable-kernel=4.19".to_string(),
        "--with-headers=/tools/include".to_string(),
        "--enable-multi-arch".to_string(),
        "--enable-stack-protector=strong".to_string(),
        "libc_cv_slibdir=/tools/lib".to_string(),
    ]);

    make_flags.insert(BuildPass::Pass1, vec![]);

    // G