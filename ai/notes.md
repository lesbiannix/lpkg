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
