// MLFS metadata: stage: cross-toolchain, variant: Pass 1

use crate::pkgs::package::{OptimizationSettings, PackageDefinition};

pub fn definition() -> PackageDefinition {
    let mut pkg = PackageDefinition::new("GCC", "15.2.0");
    pkg.source = Some("https://ftp.gnu.org/gnu/gcc/gcc-15.2.0/gcc-15.2.0.tar.xz".to_string());
    pkg.md5 = Some("7c32c39b8b6e3ae85f25156228156061".to_string());
    pkg.configure_args = Vec::new();
    pkg.build_commands = vec![
        "tar -xf ../mpfr-4.2.2.tar.xz".to_string(),
        "mv -v mpfr-4.2.2 mpfr".to_string(),
        "tar -xf ../gmp-6.3.0.tar.xz".to_string(),
        "mv -v gmp-6.3.0 gmp".to_string(),
        "tar -xf ../mpc-1.3.1.tar.gz".to_string(),
        "mv -v mpc-1.3.1 mpc".to_string(),
        "sed -e '/m64=/s/lib64/lib/' \\".to_string(),
        "-e '/m32=/s/m32=.*/m32=..\\/lib32$(call if_multiarch,:i386-linux-gnu)/' \\".to_string(),
        "-i.orig gcc/config/i386/t-linux64".to_string(),
        "sed '/STACK_REALIGN_DEFAULT/s/0/(!TARGET_64BIT \\&\\& TARGET_SSE)/' \\".to_string(),
        "-i gcc/config/i386/i386.h".to_string(),
        "mkdir -v build".to_string(),
        "cd       build".to_string(),
        "mlist=m64,m32".to_string(),
        "../configure                  \\".to_string(),
        "--target=$LFS_TGT                              \\".to_string(),
        "--prefix=$LFS/tools                            \\".to_string(),
        "--with-glibc-version=2.42                      \\".to_string(),
        "--with-sysroot=$LFS                            \\".to_string(),
        "--with-newlib                                  \\".to_string(),
        "--without-headers                              \\".to_string(),
        "--enable-default-pie                           \\".to_string(),
        "--enable-default-ssp                           \\".to_string(),
        "--enable-initfini-array                        \\".to_string(),
        "--disable-nls                                  \\".to_string(),
        "--disable-shared                               \\".to_string(),
        "--enable-multilib --with-multilib-list=$mlist  \\".to_string(),
        "--disable-decimal-float                        \\".to_string(),
        "--disable-threads                              \\".to_string(),
        "--disable-libatomic                            \\".to_string(),
        "--disable-libgomp                              \\".to_string(),
        "--disable-libquadmath                          \\".to_string(),
        "--disable-libssp                               \\".to_string(),
        "--disable-libvtv                               \\".to_string(),
        "--disable-libstdcxx                            \\".to_string(),
        "--enable-languages=c,c++".to_string(),
        "make".to_string(),
        "cd ..".to_string(),
        "cat gcc/limitx.h gcc/glimits.h gcc/limity.h > \\".to_string(),
        "`dirname $($LFS_TGT-gcc -print-libgcc-file-name)`/include/limits.h".to_string(),
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
