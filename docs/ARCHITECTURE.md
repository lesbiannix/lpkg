# Architecture Overview

This project is split into a reusable Rust library crate (`package_management`)
and several binaries that orchestrate day-to-day workflows. The sections below
outline the main entry points and how the supporting modules fit together.

## CLI entry points

| Binary | Location | Purpose |
| ------ | -------- | ------- |
| `lpkg` | `src/main.rs` | Primary command-line interface with workflow automation and optional TUI integration. |
| `metadata_indexer` | `src/bin/metadata_indexer.rs` | Harvests LFS/BLFS/GLFS package metadata, validates it against the JSON schema, and keeps `ai/metadata/index.json` up to date. |

### `lpkg` workflows

`lpkg` uses [Clap](https://docs.rs/clap) to expose multiple subcommands:

- `EnvCheck` – fetches `<pre>` blocks from an LFS-style HTML page and runs the
  embedded `ver_check` / `ver_kernel` scripts.
- `FetchManifests` – downloads the book’s canonical `wget-list` and `md5sums`
  files and writes them to disk.
- `BuildBinutils` – parses the Binutils Pass 1 page, mirrors the documented
  build steps, and executes them in a Tokio runtime.
- `ScaffoldPackage` – generates a new module under `src/pkgs/by_name/` with
  optimisation defaults (LTO/PGO/`-O3`) and persists metadata via the DB
  helpers.
- `ImportMlfs` – walks the MLFS catalogue, scaffolding definitions and storing
  them in the database (with optional `--dry-run`, `--limit`, and `--overwrite`).

When compiled with the `tui` feature flag, the CLI also exposes
`lpkg tui disk-manager`, which drops the user into the terminal UI defined in
`src/tui/`.

### `metadata_indexer`

The `metadata_indexer` binary is a companion tool for maintaining the JSON
artifacts under `ai/metadata/`:

- `validate` – validates every `packages/**.json` file against
  `ai/metadata/schema.json` and reports schema or summary extraction issues.
- `index` – revalidates the metadata and regenerates
  `ai/metadata/index.json` (use `--compact` for single-line JSON).
- `harvest` – fetches a given book page, extracts build metadata, and emits a
  schema-compliant JSON skeleton. When direct HTML parsing does not locate the
  source tarball, it falls back to the jhalfs `wget-list` data to populate
  `source.urls`.

## Module layout

```
src/
  ai/             // JSON loaders for repository personas, tasks, and bugs
  db/             // Diesel database setup and models
  html.rs         // Lightweight HTML helpers (fetch + parse <pre> blocks)
  ingest/         // Parsers for LFS / MLFS / BLFS / GLFS book content
  md5_utils.rs    // Fetches canonical md5sums from the book mirror
  mirrors.rs      // Lists official source mirrors for downloads
  pkgs/           // Package scaffolding and metadata definition helpers
  tui/            // Optional terminal UI (crossterm + tui)
  version_check.rs// Executes ver_check / ver_kernel snippets
  wget_list.rs    // Fetches jhalfs-maintained wget-list manifests
  bin/metadata_indexer.rs // AI metadata CLI described above
```

### Notable modules

- **`src/pkgs/scaffolder.rs`**
  - Generates filesystem modules and `PackageDefinition` records based on a
    `ScaffoldRequest`.
  - Normalises directory layout (prefix modules, `mod.rs` entries) and applies
    optimisation defaults (LTO, PGO, `-O3`).

- **`src/ingest/`**
  - Provides HTML parsers tailored to each book flavour (LFS, MLFS, BLFS,
    GLFS). The parsers emit `BookPackage` records consumed by the scaffolder
    and metadata importer.

- **`src/db/`**
  - Diesel models and schema for persisting package metadata. `lpkg` uses these
    helpers when scaffolding or importing packages.

- **`src/tui/`**
  - Houses the optional terminal interface (disk manager, main menu, settings,
    downloader). The entry points are conditionally compiled behind the `tui`
    cargo feature.

## Data & metadata assets

The repository keeps long-lived ARTifacts under `ai/`:

- `ai/metadata/` – JSON schema (`schema.json`), package records, and a generated
  index (`index.json`). The `metadata_indexer` binary maintains these files.
- `ai/personas.json`, `ai/tasks.json`, `ai/bugs.json` – contextual data for
  automated assistance.
- `ai/notes.md` – scratchpad for future work (e.g., jhalfs integration).

`data/` currently contains catalogues derived from the MLFS book and can be
extended with additional book snapshots.

## Database and persistence

The Diesel setup uses SQLite (via the `diesel` crate with `sqlite` and `r2d2`
features enabled). Connection pooling lives in `src/db/mod.rs` and is consumed
by workflows that scaffold or import packages.

## Optional terminal UI

The TUI resolves around `DiskManager` (a crossterm + tui based interface for
GPT partition inspection and creation). Additional stubs (`main_menu.rs`,
`settings.rs`, `downloader.rs`) are present for future expansion. The main CLI
falls back to `DiskManager::run_tui()` whenever `lpkg` is invoked without a
subcommand and is compiled with `--features tui`.

---

For more operational details around metadata harvesting, refer to
[`docs/METADATA_PIPELINE.md`](./METADATA_PIPELINE.md).
