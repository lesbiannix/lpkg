#!/bin/bash

# Build metadata indexer
cargo build --bin metadata_indexer

# Refresh manifests for all books
./target/debug/metadata_indexer refresh --books mlfs,lfs,blfs,glfs --force

# Function to harvest a package
harvest_package() {
    local book=$1
    local page=$2
    local base_url=$3
    
    echo "Harvesting $book/$page..."
    if [ -n "$base_url" ]; then
        ./target/debug/metadata_indexer harvest --book "$book" --page "$page" --base-url "$base_url"
    else
        ./target/debug/metadata_indexer harvest --book "$book" --page "$page"
    fi
}

# MLFS Packages
MLFS_PACKAGES=(
    "linux"
    "glibc"
    "binutils-pass-1"
    "gcc-pass-1"
)

# Base System Packages
BASE_PACKAGES=(
    "acl"
    "attr"
    "autoconf"
    "automake"
    "bash"
    "bc"
    "binutils"
    "bison"
    "bzip2"
    "check"
    "coreutils"
    "dejagnu"
    "diffutils"
    "e2fsprogs"
    "elfutils"
    "expat"
    "expect"
    "file"
    "findutils"
    "flex"
    "flit_core"
    "gawk"
    "gcc"
    "gdbm"
    "gettext"
    "gmp"
    "gperf"
    "grep"
    "groff"
    "grub"
    "gzip"
    "iana-etc"
    "inetutils"
    "intltool"
    "iproute2"
    "jinja2"
    "kbd"
    "kmod"
    "less"
    "libcap"
    "libffi"
    "libpipeline"
    "libtool"
    "libxcrypt"
    "m4"
    "make"
    "man-db"
    "man-pages"
    "markupsafe"
    "meson"
    "mpc"
    "mpfr"
    "ncurses"
    "ninja"
    "openssl"
    "patch"
    "perl"
    "pkgconf"
    "procps"
    "psmisc"
    "python"
    "readline"
    "sed"
    "setuptools"
    "shadow"
    "sysklogd"
    "systemd"
    "sysvinit"
    "tar"
    "tcl"
    "texinfo"
    "tzdata"
    "util-linux"
    "vim"
    "wheel"
    "xml-parser"
    "xz"
    "zlib"
    "zstd"
)

# Harvest MLFS packages
for pkg in "${MLFS_PACKAGES[@]}"; do
    harvest_package "mlfs" "$pkg" "https://linuxfromscratch.org/~thomas/multilib-m32"
done

# Harvest base system packages
for pkg in "${BASE_PACKAGES[@]}"; do
    harvest_package "lfs" "$pkg" "https://linuxfromscratch.org/lfs/view/development"
done

# Update index
./target/debug/metadata_indexer index

# Print summary
echo "Done! Packages have been harvested and index has been updated."