# 🧬 LPKG – Lightweight Package Manager

LPKG is a minimalistic package manager written in Rust, designed for fast and simple software management on Unix-like systems. It emphasizes reproducibility and declarative configuration, leveraging **Nix Flakes** for development and deployment.

---

## 🚀 Features

* **Fast & Lightweight** – Minimal resource usage and quick operations.
* **Rust-Powered** – Safe and concurrent code with Rust.
* **Cross-Platform** – Works on Linux and macOS.
* **Declarative Builds** – Fully reproducible with Nix Flakes.
* **Simple CLI** – Intuitive commands for managing packages.

---

## ⚙️ Installation

### Using Cargo

```bash
cargo install lpkg
```

### Using Nix Flakes

If you have Nix with flakes enabled:

```bash
nix profile install github:lesbiannix/lpkg
```

Or to run without installing:

```bash
nix run github:lesbiannix/lpkg
```

---

## 🧰 Usage

Basic command structure:

```bash
lpkg [command] [package]
```

Common commands:

* `install` – Install a package
* `remove` – Remove a package
* `update` – Update the package list
* `upgrade` – Upgrade all installed packages

For detailed usage:

```bash
lpkg --help
```

---

## 🔧 Development with Flakes

Clone the repository:

```bash
git clone https://github.com/lesbiannix/lpkg.git
cd lpkg
```

Enter the flake development shell:

```bash
nix develop
```

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

You can also run the project directly in the flake shell:

```bash
nix run
```

---

## 📄 License

LPKG is licensed under the [MIT License](LICENSE).


