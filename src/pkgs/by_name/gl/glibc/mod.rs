// MLFS metadata: stage: cross-toolchain

use crate::pkgs::package::{OptimizationSettings, PackageDefinition};

pub fn definition() -> PackageDefinition {
    let mut pkg = PackageDefinition::new("Glibc", "2.42");
    pkg.source = Some("https://ftp.gnu.org/gnu/glibc/glibc-2.42.tar.xz".to_string());
    pkg.md5 = Some("23c6f5a27932b435cae94e087cb8b1f5".to_string());
    pkg.configure_args = Vec::new();
    pkg.build_commands = vec![
        "ln -sfv ../lib/ld-linux-x86-64.so.2 $LFS/lib64".to_string(),
        "ln -sfv ../lib/ld-linux-x86-64.so.2 $LFS/lib64/ld-lsb-x86-64.so.3".to_string(),
        "patch -Np1 -i ../glibc-2.42-fhs-1.patch".to_string(),
        "mkdir -v build".to_string(),
        "cd       build".to_string(),
        "echo \"rootsbindir=/usr/sbin\" > configparms".to_string(),
        "../configure                             \\".to_string(),
        "--prefix=/usr                      \\".to_string(),
        "--host=$LFS_TGT                    \\".to_string(),
        "--build=$(../scripts/config.guess) \\".to_string(),
        "--disable-nscd                     \\".to_string(),
        "libc_cv_slibdir=/usr/lib           \\".to_string(),
        "--enable-kernel=5.4".to_string(),
        "make".to_string(),
        "make DESTDIR=$LFS install".to_string(),
        "sed '/RTLDLIST=/s@/usr@@g' -i $LFS/usr/bin/ldd".to_string(),
        "echo 'int main(){}' | $LFS_TGT-gcc -x c - -v -Wl,--verbose &> dummy.log".to_string(),
        "readelf -l a.out | grep ': /lib'".to_string(),
        "grep -E -o \"$LFS/lib.*/S?crt[1in].*succeeded\" dummy.log".to_string(),
        "grep -B3 \"^ $LFS/usr/include\" dummy.log".to_string(),
        "grep 'SEARCH.*/usr/lib' dummy.log |sed 's|; |\\n|g'".to_string(),
        "grep \"/lib.*/libc.so.6 \" dummy.log".to_string(),
        "grep found dummy.log".to_string(),
        "rm -v a.out dummy.log".to_string(),
        "make clean".to_string(),
        "find .. -name \"*.a\" -delete".to_string(),
        "CC=\"$LFS_TGT-gcc -m32\" \\".to_string(),
        "CXX=\"$LFS_TGT-g++ -m32\" \\".to_string(),
        "../configure                             \\".to_string(),
        "--prefix=/usr                      \\".to_string(),
        "--host=$LFS_TGT32                  \\".to_string(),
        "--build=$(../scripts/config.guess) \\".to_string(),
        "--disable-nscd                     \\".to_string(),
        "--with-headers=$LFS/usr/include    \\".to_string(),
        "--libdir=/usr/lib32                \\".to_string(),
        "--libexecdir=/usr/lib32            \\".to_string(),
        "libc_cv_slibdir=/usr/lib32         \\".to_string(),
        "--enable-kernel=5.4".to_string(),
        "make".to_string(),
        "make DESTDIR=$PWD/DESTDIR install".to_string(),
        "cp -a DESTDIR/usr/lib32 $LFS/usr/".to_string(),
        "install -vm644 DESTDIR/usr/include/gnu/{lib-names,stubs}-32.h \\".to_string(),
        "$LFS/usr/include/gnu/".to_string(),
        "ln -svf ../lib32/ld-linux.so.2 $LFS/lib/ld-linux.so.2".to_string(),
        "echo 'int main(){}' > dummy.c".to_string(),
        "$LFS_TGT-gcc -m32 dummy.c".to_string(),
        "readelf -l a.out | grep '/ld-linux'".to_string(),
        "rm -v dummy.c a.out".to_string(),
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
