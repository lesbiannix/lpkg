# Repository Context Snapshot

- `README.md`, `docs/ARCHITECTURE.md`, and `docs/METADATA_PIPELINE.md` describe
  the crate structure, CLI entry points, and metadata workflows. Consult them
  first when revisiting the project.
- `metadata_indexer` now supports a `refresh` command that pulls jhalfs
  `wget-list`/`md5sums` manifests into `ai/metadata/cache/` and the `harvest`
  command automatically draws URLs and checksums from those manifests.
- AI state lives under `ai/`:
  - `ai/personas.json`, `ai/tasks.json`, `ai/bugs.json` track personas,
    outstanding work, and known issues.
  - `ai/metadata/` stores package records plus the JSON schema.
  - `ai/notes.md` captures ongoing research ideas (e.g., deeper BLFS/GLFS
    manifest coverage).
- Duplicate MLFS metadata entries were pruned (`binutils-pass1.json` removed in
  favour of the `binutils-pass-1.json` slug).

This file is intended as a quick orientation checkpoint alongside the richer
architecture docs.
