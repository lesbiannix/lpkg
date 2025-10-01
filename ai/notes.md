# Integrating jhalfs Source Metadata

- Goal: reuse jhalfs wget-list and md5sums to populate package `source.urls` and
auto-fill checksums when harvesting metadata for MLFS/BLFS/GLFS packages.
- Data source: `https://anduin.linuxfromscratch.org/` hosts per-release
  `wget-list`/`md5sums` files already curated by the jhalfs project.
- Approach:
  1. Fetch (and optionally cache under `ai/cache/`) the lists for each book.
  2. When harvesting, map `<package>-<version>` against the list to gather all
     relevant URLs.
  3. Pull matching checksum entries to populate `source.checksums`.
  4. Keep the existing HTML scrape for chapter/stage text; jhalfs covers only
     sources.
- Benefits: avoids fragile HTML tables, keeps URLs aligned with official build
  scripts, and ensures checksums are up-to-date.

# Metadata → Rust Module Strategy

Goal: emit Rust modules under `src/pkgs/by_name` directly from harvested
metadata once MLFS/BLFS/GLFS records are validated.

Outline:
1. **Schema alignment** – Ensure harvested JSON carries everything the
   `PackageDefinition` constructor expects (source URLs, checksums, build
   commands, dependencies, optimisation flags, notes/stage metadata).
2. **Translation layer** – Implement a converter (likely in a new module,
   e.g. `src/pkgs/generator.rs`) that reads a metadata JSON file and produces a
   `ScaffoldRequest` or directly writes the module source via the existing
   scaffolder.
3. **Naming/layout** – Derive module paths from `package.id` (e.g.
   `mlfs/binutils-pass-1` → `src/pkgs/by_name/bi/binutils/pass_1/mod.rs`) while
   preserving the prefix/slug conventions already used by the scaffolder.
4. **CLI integration** – Add a subcommand (`metadata_indexer generate`) that
   accepts a list of package IDs or a glob, feeds each through the translator,
   and optionally stages the resulting Rust files.
5. **Diff safety** – Emit modules to a temporary location first, compare
   against existing files, and only overwrite when changes are detected; keep a
   `--dry-run` mode for review.
6. **Tests/checks** – After generation, run `cargo fmt` and `cargo check` to
   ensure the new modules compile; optionally add schema fixtures covering edge
   cases (variants, multiple URLs, absent checksums).

Open questions:
- How to represent optional post-install steps or multi-phase builds inside the
  generated module (additional helper functions vs. raw command arrays).
- Where to store PGO workload hints once the PGO infrastructure is defined.

# Lightweight Networking Rewrite

- Motivation: remove heavy async stacks (tokio + reqwest) from the default
  feature set to keep clean builds fast and reduce binary size.
- HTTP stack baseline: [`ureq`](https://github.com/algesten/ureq) (blocking,
  TLS via rustls, small dependency footprint) plus `scraper` for DOM parsing.
- Migration checklist:
  - [x] Replace `reqwest` usage in `src/html.rs`, `md5_utils.rs`,
    `wget_list.rs`, `mirrors.rs`, and the ingest pipelines.
  - [x] Rework `binutils` cross toolchain workflow to operate synchronously,
    eliminating tokio runtime/bootstrap.
  - [ ] Drop `tokio` and `reqwest` from `Cargo.toml` once TUI workflows stop
    using tracing instrumentation hooks that pulled them in transitively.
  - [ ] Audit for remaining `tracing` dependencies and migrate to the
    lightweight logging facade (`log` + `env_logger` or custom adapter) for
    non-TUI code.
- Follow-up ideas:
  - Provide feature flag `full-net` that re-enables async clients when needed
    for high-concurrency mirror probing.
  - Benchmark `ureq` vs `reqwest` on `metadata_indexer harvest` to ensure we
    don’t regress throughput noticeably.

# README Generation Framework (Markdown RFC)

- Goal: author the project README in Rust, using a small domain-specific
  builder that outputs GitHub-flavoured Markdown (GFM) from structured
  sections.
- Design sketch:
  - New crate/workspace member `readme_builder` under `tools/` exposing a
    fluent API (`Doc::new().section("Intro", |s| ...)`).
  - Source-of-truth lives in `tools/readme/src/main.rs`; running `cargo run -p
    readme_builder` writes to `README.md`.
  - Provide reusable primitives: `Heading`, `Paragraph`, `CodeBlock`,
    `Table::builder()`, `Callout::note("...")`, `Badge::docsrs()`, etc.
  - Keep rendering deterministic (sorted sections, stable wrapping) so diffs
    remain reviewable.
- Tasks:
  - [ ] Scaffold `tools/readme` crate with CLI that emits to stdout or
    specified path (`--output README.md`).
  - [ ] Model README sections as enums/structs with `Display` impls to enforce
    consistency.
  - [ ] Port current README structure into builder code, annotate with inline
    comments describing regeneration steps.
  - [ ] Add `make readme` (or `cargo xtask readme`) to rebuild documentation as
    part of release workflow.
  - [ ] Document in CONTRIBUTING how to edit the Rust source instead of the
    raw Markdown.
- Stretch goals:
  - Emit additional artefacts (e.g., `docs/CHANGELOG.md`) from the same source
    modules.
- Allow embedding generated tables from Cargo metadata (dependency stats,
  feature lists).

# Dependency Slimming Log

- 2025-03: Replaced `reqwest`/`tokio` async stack with `ureq`; default builds
  now avoid pulling in hyper/quinn/tower trees. GraphQL feature gate still pulls
  Actix/tokio, but only when enabled.
- Added `.cargo/config.toml` profiles: dev stays at `opt-level=0`, release uses
  LTO fat + `-O3`, and PGO profiles expose `cargo pgo-instrument`/`cargo
  pgo-build` aliases.
- All SVG artefacts (core logo, Nixette logo/mascot/wallpaper) are now generated
  by Rust binaries under `src/bin/*_gen.rs` using a shared `svg_builder` module.
  Regeneration steps:
  ```bash
  cargo run --bin logo_gen
  cargo run --bin nixette_logo_gen
  cargo run --bin nixette_mascot_gen
  cargo run --bin nixette_wallpaper_gen
  ```
- README is produced via `cargo run --bin readme_gen`; contributors should edit
  the builder source instead of the Markdown output.
- Remaining work: trim tracing/Actix dependencies inside the TUI path,
  investigate replacing `gptman` for non-critical disk UI builds, and pin a
  cargo `deny` audit to alert on large transitive graphs.
