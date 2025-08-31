# 🌸 MLFS Bootstrap 🌸

**Multilib Linux From Scratch Bootstrap Tool**

A catgirl-powered package management system for building MLFS (Multilib Linux From Scratch) toolchains with style! ✨

## 🎯 Features

- 📦 **LPKG Format**: Arch-inspired package definitions in `.lpkg` files
- 🔄 **Multi-Pass Builds**: Support for Pass1, Pass2, and Final build phases
- 🧪 **Dev Mode**: Use temporary directories for testing and development
- 🎨 **Fancy Output**: Beautiful colored terminal output with progress indicators
- 🐳 **Docker Support**: Containerized testing environment
- 📊 **Dependency Resolution**: Automatic build order calculation
- 🌈 **Multilib Support**: Built for x86_64 with 32-bit compatibility

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ 
- Linux system with build tools (gcc, make, wget, tar)
- For multilib: gcc-multilib, g++-multilib

### Installation

```bash
git clone <your-repo>
cd mlfs-bootstrap
cargo build --release
```

### Initialize Project

```bash
./target/release/mlfs-bootstrap --init
```

This creates:
- Sample `.lpkg` files in `packages/` directory
- Proper directory structure

### Development Mode

Test safely with temporary directories:

```bash
./target/release/mlfs-bootstrap --dev --list
./target/release/mlfs-bootstrap --dev --package binutils --pass pass1
```

### Production Build

```bash
sudo ./target/release/mlfs-bootstrap --lfs-dir /mnt/lfs
```

## 📦 LPKG Package Format

Package definitions use a simple TOML-like format inspired by Arch's PKGBUILD:

```toml
# Package metadata
[package]
version = "2.42"
description = "GNU Binary Utilities"
url = "https://ftp.gnu.org/gnu/binutils/binutils-2.42.tar.xz"
archive_format = "tar.xz"

# Dependencies
[dependencies]
depends = ["gcc", "linux-headers"]

# Build configuration
[build]
passes = ["pass1", "pass2"]

# Pass-specific configuration
[pass1]
configure_flags = [
    "--target=$LFS_TGT",
    "--prefix=/tools",
    "--disable-nls"
]
make_flags = []
pre_build = ["echo 'Starting build'"]
post_build = ["echo 'Build complete'"]

# Environment variables for this pass
CC = "$LFS_TGT-gcc"
CXX = "$LFS_TGT-g++"
```

### LPKG Sections

- **[package]**: Basic metadata (version, description, URL)
- **[dependencies]**: Package dependencies (`depends = ["pkg1", "pkg2"]`)
- **[build]**: Build configuration (`passes = ["pass1", "pass2"]`)
- **[pass1/pass2/final]**: Pass-specific build instructions
- **[patches]**: Patch files to apply (`files = ["patch1.patch"]`)

### Variables

Available variables in LPKG files:
- `$LFS_TGT`: Target triplet (x86_64-lfs-linux-gnu)
- `$LFS`: LFS root directory
- `$PWD`: Current working directory
- Any custom environment variables

## 🔨 Usage Examples

### List Available Packages
```bash
mlfs-bootstrap --list
```

### Build Specific Package
```bash
mlfs-bootstrap --package binutils --pass pass1
```

### Show Dependency Order
```bash
mlfs-bootstrap --dep-order
```

### Dev Mode Testing
```bash
mlfs-bootstrap --dev --package gcc --pass pass1
```

## 🐳 Docker Testing

### Build Container
```bash
docker build -t mlfs-bootstrap .
```

### Run Dev Mode
```bash
docker run --rm -it mlfs-bootstrap --dev --list
```

### Test Package Build
```bash
docker run --rm -it mlfs-bootstrap --dev --package binutils --pass pass1
```

### Interactive Shell
```bash
docker run --rm -it mlfs-bootstrap bash
```

## 📁 Project Structure

```
mlfs-bootstrap/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── bootstrap.rs     # Core bootstrap logic
│   ├── package.rs       # Package data structures
│   ├── lpkg.rs         # LPKG file parser
│   ├── ui.rs           # Fancy terminal UI
│   └── config.rs       # Configuration management
├── packages/           # .lpkg package definitions
│   ├── binutils.lpkg
│   ├── gcc.lpkg
│   ├── glibc.lpkg
│   └── linux-headers.lpkg
├── Dockerfile          # Container for testing
├── Cargo.toml         # Rust dependencies
└── README.md          # This file
```

## 🎭 Build Passes

### Pass 1: Cross-Compilation Toolchain
- Build cross-compilation tools
- Target: `$LFS_TGT` (x86_64-lfs-linux-gnu)
- Purpose: Create tools to build the temporary system

### Pass 2: Native Toolchain  
- Build native tools using Pass 1 tools
- Purpose: Create final toolchain for building LFS system

### Final: System Packages
- Build final system packages
- Purpose: Complete the LFS system

## 🔧 Configuration

### Global Environment Variables

Set in your shell or add to LPKG files:

```bash
export LFS=/mnt/lfs
export LFS_TGT=x86_64-lfs-linux-gnu
export LC_ALL=POSIX
export MAKEFLAGS='-j4'
```

### Package Dependencies

Dependencies are automatically resolved and built in the correct order.

## 🐛 Troubleshooting

### Common Issues

1. **Permission Denied**: Make sure you have write access to LFS directory
2. **Missing Dependencies**: Install build-essential, wget, tar, xz-utils
3. **Download Failures**: Check internet connection and URL validity
4. **Build Failures**: Check package logs and ensure all dependencies are built

### Debug Mode

Use `--dev` mode for safe testing:
```bash
mlfs-bootstrap --dev --package problematic-package --pass pass1
```

### Logs

Build logs are shown in real-time with colored output:
- 🔮 Info (blue)
- ✨ Success (green)  
- ⚠️ Warning (yellow)
- 💥 Error (red)

## 🤝 Contributing

1. Add new packages by creating `.lpkg` files
2. Test in dev mode first: `--dev --package yourpackage`
3. Ensure proper dependencies are listed
4. Test with Docker for clean environment

### Adding New Packages

Create `packages/yourpackage.lpkg`:

```toml
[package]
version = "1.0.0"
description = "Your awesome package"
url = "https://example.com/package.tar.xz"

[dependencies]
depends = ["binutils", "gcc"]

[build]
passes = ["pass2"]

[pass2]
configure_flags = ["--prefix=/tools"]
make_flags = []
```

## 📝 License

This project is released into the public domain. Use it however you want! 💖

## 🙏 Acknowledgments

- Linux From Scratch project for the amazing documentation
- Arch Linux for PKGBUILD inspiration  
- All the catgirls who made this possible 🐱✨

---

*Made with 💖 by Anonymous Catgirl*

*"Building Linux systems, one package at a time, with maximum cuteness!"* 🌸