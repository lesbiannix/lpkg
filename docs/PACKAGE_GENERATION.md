# Package Module Generation

This document explains how harvested metadata is transformed into concrete
Rust modules under `src/pkgs/by_name/`.

## Overview

1. **Harvest metadata** – Use `metadata_indexer harvest` to capture package data
   from the LFS/BLFS/GLFS books. Each record is written to
   `ai/metadata/packages/<book>/<slug>.json`.
2. **Refresh manifests** – Run
   `metadata_indexer refresh` to ensure the jhalfs `wget-list` and `md5sums`
   caches are up to date. Harvesting relies on these caches for canonical
   source URLs and checksums.
3. **Generate modules** – Use
   `metadata_indexer generate --metadata <path> --output <by_name_dir>` to turn a
   metadata file into a full Rust module that exposes a `PackageDefinition`.

Generated modules leverage the existing scaffolder logic, so the command will
create any missing prefix directories (e.g. `bi/mod.rs`) and populate the final
`mod.rs` file with the correct code template.

## Command reference

```bash
# Harvest metadata from a book page
cargo run --bin metadata_indexer -- --base-dir . harvest \
  --book mlfs \
  --page chapter05/binutils-pass1 \
  --output ai/metadata/packages/mlfs/binutils-pass-1.json

# Refresh jhalfs manifests (optional but recommended)
cargo run --bin metadata_indexer -- --base-dir . refresh

# Generate a module under the standard src tree
cargo run --bin metadata_indexer -- --base-dir . generate \
  --metadata ai/metadata/packages/mlfs/binutils-pass-1.json \
  --output src/pkgs/by_name \
  --overwrite
```

### Flags

- `--output` defaults to `src/pkgs/by_name`. Point it to another directory if
  you want to stage modules elsewhere (e.g. `target/generated/by_name`).
- `--overwrite` deletes the existing module directory before scaffolding a new
  one.

After generation, run `cargo fmt` and `cargo check` to ensure the crate compiles
with the new modules.

## Implementation notes

- Metadata fields such as `build`, `dependencies`, and `optimizations` are
  mapped directly onto the scaffolder’s `ScaffoldRequest` type.
- Source URLs and MD5 checksums are sourced from the harvested metadata
  (populated via the jhalfs manifests).
- The module slug is derived from `package.id` (e.g.
  `mlfs/binutils-pass-1` → `src/pkgs/by_name/bi/binutils_pass_1/mod.rs`).

See the code in `src/pkgs/generator.rs` for the full translation logic.
