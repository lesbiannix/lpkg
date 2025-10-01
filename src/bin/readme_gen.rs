use std::fs;

fn main() -> anyhow::Result<()> {
    let readme = Readme::build();
    fs::write("README.md", readme)?;
    Ok(())
}

struct MarkdownDoc {
    buffer: String,
}

impl MarkdownDoc {
    fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    fn heading(mut self, level: u8, text: &str) -> Self {
        self.buffer.push_str(&"#".repeat(level as usize));
        self.buffer.push(' ');
        self.buffer.push_str(text);
        self.buffer.push_str("\n\n");
        self
    }

    fn raw(mut self, text: &str) -> Self {
        self.buffer.push_str(text);
        self.buffer.push('\n');
        self
    }

    fn paragraph(mut self, text: &str) -> Self {
        self.buffer.push_str(text);
        self.buffer.push_str("\n\n");
        self
    }

    fn horizontal_rule(mut self) -> Self {
        self.buffer.push_str("---\n\n");
        self
    }

    fn bullet_list<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for item in items {
            self.buffer.push_str("* ");
            self.buffer.push_str(item.as_ref());
            self.buffer.push('\n');
        }
        self.buffer.push('\n');
        self
    }

    fn code_block(mut self, language: &str, code: &str) -> Self {
        self.buffer.push_str("```");
        self.buffer.push_str(language);
        self.buffer.push('\n');
        self.buffer.push_str(code.trim_matches('\n'));
        self.buffer.push_str("\n```\n\n");
        self
    }

    fn finish(self) -> String {
        self.buffer
    }
}

struct Readme;

impl Readme {
    fn build() -> String {
        let doc = MarkdownDoc::new()
            .heading(1, "🧬 LPKG – Lightweight Package Manager")
            .raw("<p align=\"center\">\n  <img src=\"assets/logo.svg\" alt=\"LPKG logo\" width=\"360\" />\n</p>\n")
            .paragraph("LPKG is a minimalistic package manager written in Rust, designed for fast and simple software management on Unix-like systems. It emphasizes reproducibility and declarative configuration, leveraging **Nix Flakes** for development and deployment.")
            .horizontal_rule()
            .heading(2, "🚀 Features")
            .bullet_list([
                "**Fast & Lightweight** – Minimal resource usage and quick operations.",
                "**Rust-Powered** – Safe and concurrent code with Rust.",
                "**Cross-Platform** – Works on Linux and macOS.",
                "**Declarative Builds** – Fully reproducible with Nix Flakes.",
                "**Simple CLI** – Intuitive commands for managing packages.",
            ])
            .horizontal_rule()
            .heading(2, "⚙️ Installation")
            .heading(3, "Using Cargo")
            .code_block("bash", "cargo install lpkg")
            .heading(3, "Using Nix Flakes")
            .paragraph("If you have Nix with flakes enabled:")
            .code_block("bash", "nix profile install github:lesbiannix/lpkg")
            .paragraph("Or to run without installing:")
            .code_block("bash", "nix run github:lesbiannix/lpkg")
            .horizontal_rule()
            .heading(2, "🧰 Usage")
            .paragraph("Basic command structure:")
            .code_block("bash", "lpkg [command] [package]")
            .paragraph("Common commands:")
            .bullet_list([
                "`install` – Install a package",
                "`remove` – Remove a package",
                "`update` – Update the package list",
                "`upgrade` – Upgrade all installed packages",
            ])
            .paragraph("For detailed usage:")
            .code_block("bash", "lpkg --help")
            .horizontal_rule()
            .heading(2, "🔧 Development with Flakes")
            .paragraph("Clone the repository:")
            .code_block("bash", "git clone https://github.com/lesbiannix/lpkg.git\ncd lpkg")
            .paragraph("Enter the flake development shell:")
            .code_block("bash", "nix develop")
            .paragraph("Build the project:")
            .code_block("bash", "cargo build")
            .paragraph("LPKG ships with tuned Cargo profiles:")
            .bullet_list([
                "**Dev builds** (`cargo build`) use `opt-level=0`, lots of codegen units, and incremental compilation for quick feedback while hacking.",
                "**Release builds** (`cargo build --release`) enable `-O3`, fat LTO, and panic aborts for slim, fast binaries.",
                "**GraphQL builds** add the server components when you need them:",
            ])
            .code_block("bash", "cargo build --features graphql")
            .paragraph("**PGO builds** are a two-step flow using the provided Cargo aliases:")
            .code_block(
                "bash",
                r#"# 1) Instrument
RUSTFLAGS="-Cprofile-generate=target/pgo-data" cargo pgo-instrument
# run representative workloads to emit *.profraw files under target/pgo-data
llvm-profdata merge -o target/pgo-data/lpkg.profdata target/pgo-data/*.profraw

# 2) Optimise with the collected profile
RUSTFLAGS="-Cprofile-use=target/pgo-data/lpkg.profdata -Cllvm-args=-pgo-warn-missing-function" \
  cargo pgo-build"#,
            )
            .paragraph("Run tests:")
            .code_block("bash", "cargo test")
            .paragraph("You can also run the project directly in the flake shell:")
            .code_block("bash", "nix run")
            .heading(2, "🕸️ GraphQL API")
            .paragraph("LPKG now ships a lightweight GraphQL server powered by Actix Web and Juniper.")
            .bullet_list([
                "Start the server with `cargo run --features graphql --bin graphql_server` (set `LPKG_GRAPHQL_ADDR` to override `127.0.0.1:8080`).",
                "Query endpoint: `http://127.0.0.1:8080/graphql`",
                "Interactive playground: `http://127.0.0.1:8080/playground`",
            ])
            .paragraph("Example query:")
            .code_block("graphql", r"{
  packages(limit: 5) {
    name
    version
    enableLto
  }
  randomJoke {
    package
    text
  }
}")
            .heading(3, "AI metadata tooling")
            .paragraph("The AI metadata store under `ai/metadata/` comes with a helper CLI to validate package records against the JSON schema and regenerate `index.json` after adding new entries:")
            .code_block("bash", r"cargo run --bin metadata_indexer -- --base-dir . validate
cargo run --bin metadata_indexer -- --base-dir . index")
            .paragraph("Use `--compact` with `index` if you prefer single-line JSON output.")
            .paragraph("To draft metadata for a specific book page, you can run the harvest mode. It fetches the XHTML, scrapes the build commands, and emits a schema-compliant JSON skeleton (pass `--dry-run` to inspect the result without writing to disk):")
            .code_block("bash", r"cargo run --bin metadata_indexer -- \
  --base-dir . harvest \
  --book mlfs \
  --page chapter05/binutils-pass1 \
  --dry-run")
            .paragraph("Keep the jhalfs manifests current with:")
            .code_block("bash", "cargo run --bin metadata_indexer -- --base-dir . refresh")
            .paragraph("Passing `--books mlfs,blfs` restricts the refresh to specific books, and `--force` bypasses the local cache.")
            .paragraph("To materialise a Rust module from harvested metadata:")
            .code_block("bash", r"cargo run --bin metadata_indexer -- \
  --base-dir . generate \
  --metadata ai/metadata/packages/mlfs/binutils-pass-1.json \
  --output target/generated/by_name")
            .paragraph("Add `--overwrite` to regenerate an existing module directory.")
            .heading(2, "📚 Documentation")
            .bullet_list([
                "[Architecture Overview](docs/ARCHITECTURE.md) – high-level tour of the crate layout, binaries, and supporting modules.",
                "[Metadata Harvesting Pipeline](docs/METADATA_PIPELINE.md) – how the metadata indexer produces and validates the JSON records under `ai/metadata/`.",
                "[Package Module Generation](docs/PACKAGE_GENERATION.md) – end-to-end guide for converting harvested metadata into Rust modules under `src/pkgs/by_name/`.",
                "Concept corner: [Nixette](concepts/nixette/README.md) – a NixOS × Gentoo transfemme mash-up dreamed up for fun brand explorations.",
                "`ai/notes.md` – scratchpad for ongoing research tasks (e.g., deeper jhalfs integration).",
            ])
            .horizontal_rule()
            .heading(2, "📄 License")
            .paragraph("LPKG is licensed under the [MIT License](LICENSE).");

        doc.finish()
    }
}
