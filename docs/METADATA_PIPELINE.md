# Metadata Harvesting Pipeline

This repository tracks AI-friendly package metadata under `ai/metadata/`.
The `metadata_indexer` binary orchestrates validation and harvesting tasks.
This document explains the workflow and the supporting assets.

## Directory layout

- `ai/metadata/schema.json` – JSON Schema (Draft 2020-12) describing one
  package record.
- `ai/metadata/packages/<book>/<slug>.json` – harvested package metadata.
- `ai/metadata/index.json` – generated summary table linking package IDs to
  their JSON files.
- `ai/notes.md` – scratchpad for future improvements (e.g., jhalfs integration).

## `metadata_indexer` commands

| Command | Description |
| ------- | ----------- |
| `validate` | Loads every package JSON file and validates it against `schema.json`. Reports schema violations and summary extraction errors. |
| `index` | Re-runs validation and regenerates `index.json`. Use `--compact` to write a single-line JSON payload. |
| `harvest` | Fetches a book page, scrapes build instructions, and emits a draft metadata record (to stdout with `--dry-run` or into `ai/metadata/packages/`). Falls back to jhalfs manifests when inline source links are absent. |
| `refresh` | Updates cached jhalfs manifests (`wget-list`, `md5sums`) under `ai/metadata/cache/`. Supports `--books` filtering and `--force` to bypass the cache. |

### Harvesting flow

1. **Fetch HTML** – the requested page is downloaded with `reqwest` and parsed
   using `scraper` selectors.
2. **Heading metadata** – the `h1.sect1` title provides the chapter/section,
   canonical package name, version, and optional variant hints.
3. **Build steps** – `<pre class="userinput">` blocks become ordered `build`
   phases (`setup`, `configure`, `build`, `test`, `install`).
4. **Artifact stats** – `div.segmentedlist` entries supply SBU and disk usage.
5. **Source URLs** – the harvester tries two strategies:
   - Inline HTML links inside the page (common for BLFS articles).
   - Fallback to the cached jhalfs `wget-list` for the selected book to find
     matching `<package>-<version>` entries.
6. **Checksums** – the matching entry from the cached jhalfs `md5sums`
   manifest populates `source.checksums` when the archive name is known.
7. **Status** – unresolved items (missing URLs, anchors, etc.) are recorded in
   `status.issues` so humans can interrogate or patch the draft before
   promoting it.

### Known gaps

- **Source links via tables** – some MLFS chapters list download links inside a
  “Package Information” table. The current implementation relies on the
  jhalfs `wget-list` fallback instead of parsing that table.
- **Anchor discovery** – if the heading lacks an explicit `id` attribute, the
  scraper attempts to locate child anchors or scan the raw HTML. If none are
  found, a warning is recorded and `status.issues` contains a reminder.

## Using jhalfs manifests

The maintained `wget-list`/`md5sums` files hosted by jhalfs provide canonical
source URLs and hashes. The `metadata_indexer refresh` command keeps these
manifests cached under `ai/metadata/cache/`. Harvesting consumes the cached
copies to populate URLs and MD5 checksums.

Planned enhancements (see `ai/notes.md` and `ai/bugs.json#metadata-harvest-no-source-urls`):

1. Abstract list fetching so BLFS/GLFS variants can reuse the logic.
2. Normalise the match criteria for package + version (handling pass stages,
   suffixes, etc.).

## Manual review checklist

When a new metadata file is generated:

- `schema_version` should match `schema.json` (currently `v0.1.0`).
- `package.id` should be unique (format `<book>/<slug>`).
- `source.urls` must include at least one primary URL; add mirrors/patches as
  needed.
- Clear any `status.issues` before promoting the record from `draft`.
- Run `cargo run --bin metadata_indexer -- --base-dir . index` to regenerate
  the global index once the draft is finalised.

Refer to `README.md` for usage examples and to `docs/ARCHITECTURE.md` for a
broader overview of the crate layout.
