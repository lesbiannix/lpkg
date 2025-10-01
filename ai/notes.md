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
