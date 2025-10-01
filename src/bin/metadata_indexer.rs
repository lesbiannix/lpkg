use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use jsonschema::JSONSchema;
use regex::Regex;
use reqwest::{blocking::Client, redirect::Policy};
use scraper::{ElementRef, Html, Selector};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "metadata-indexer",
    about = "Validate and regenerate AI metadata index"
)]
struct Cli {
    /// Repository root containing the `ai/metadata` directory
    #[arg(long, default_value = ".")]
    base_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate all package metadata against the JSON schema
    Validate,
    /// Validate metadata and regenerate ai/metadata/index.json
    Index {
        /// Emit compact JSON instead of pretty printing
        #[arg(long)]
        compact: bool,
    },
    /// Refresh cached jhalfs manifests for the given book(s)
    Refresh {
        /// Books to refresh (defaults to all known books)
        #[arg(long, value_delimiter = ',', default_value = "mlfs,lfs,blfs,glfs")]
        books: Vec<String>,
        /// Force re-download even if cache files exist
        #[arg(long)]
        force: bool,
    },
    /// Fetch and draft metadata for a specific package page
    Harvest {
        /// Book identifier (lfs, mlfs, blfs, glfs)
        #[arg(long)]
        book: String,
        /// Page path (relative to base) or full URL
        #[arg(long)]
        page: String,
        /// Override base URL for the selected book
        #[arg(long)]
        base_url: Option<String>,
        /// Optional explicit output file path
        #[arg(long)]
        output: Option<PathBuf>,
        /// Do not write to disk, just print JSON to stdout
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let base_dir = cli.base_dir.canonicalize().unwrap_or(cli.base_dir);
    let metadata_dir = base_dir.join("ai").join("metadata");
    let schema_path = metadata_dir.join("schema.json");
    let packages_dir = metadata_dir.join("packages");

    let (_schema_value, schema) = load_schema(&schema_path)?;
    let packages = scan_packages(&packages_dir)?;

    let mut had_errors = false;
    for package in &packages {
        let validation = schema.validate(&package.value);
        if let Err(errors) = validation {
            had_errors = true;
            eprintln!(
                "Schema validation failed for {}:",
                package.relative_path.display()
            );
            for err in errors {
                eprintln!("  - {}", err);
            }
        }

        if let Some(err) = &package.summary_error {
            had_errors = true;
            eprintln!(
                "Summary extraction failed for {}: {}",
                package.relative_path.display(),
                err
            );
        }
    }

    match cli.command {
        Command::Validate => {
            if had_errors {
                anyhow::bail!("metadata validation failed");
            }
        }
        Command::Index { compact } => {
            if had_errors {
                anyhow::bail!("metadata validation failed; index not updated");
            }

            let summaries: Vec<_> = packages
                .iter()
                .filter_map(|pkg| pkg.summary.clone())
                .collect();

            let schema_version = summaries
                .first()
                .map(|s| s.schema_version.as_str())
                .unwrap_or("v0.0.0");

            let generated_at = Utc::now().to_rfc3339();
            let packages_json: Vec<Value> = summaries
                .iter()
                .map(|s| {
                    json!({
                        "id": s.id.clone(),
                        "name": s.name.clone(),
                        "version": s.version.clone(),
                        "stage": s.stage.clone(),
                        "book": s.book.clone(),
                        "variant": s.variant.clone(),
                        "status": s.status.clone(),
                        "path": s.relative_path.clone(),
                    })
                })
                .collect();

            let index = json!({
                "generated_at": generated_at,
                "schema_version": schema_version,
                "packages": packages_json,
            });

            let index_path = metadata_dir.join("index.json");
            let serialized = if compact {
                serde_json::to_string(&index)?
            } else {
                serde_json::to_string_pretty(&index)?
            };
            fs::write(&index_path, serialized)
                .with_context(|| format!("writing {}", index_path.display()))?;
            println!("Updated {}", index_path.display());
        }
        Command::Harvest {
            book,
            page,
            base_url,
            output,
            dry_run,
        } => {
            let book_lower = book.to_lowercase();
            let harvest = harvest_metadata(&metadata_dir, &book_lower, &page, base_url.as_deref())?;

            if dry_run {
                println!("{}", serde_json::to_string_pretty(&harvest.value)?);
            } else {
                let output_path = output.unwrap_or_else(|| {
                    metadata_dir
                        .join("packages")
                        .join(&book_lower)
                        .join(format!("{}.json", harvest.slug))
                });
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("creating directory {}", parent.display()))?;
                }
                fs::write(&output_path, serde_json::to_string_pretty(&harvest.value)?)
                    .with_context(|| format!("writing {}", output_path.display()))?;
                println!(
                    "Harvested metadata for {} -> {}",
                    harvest.package_id,
                    output_path.display()
                );
                println!(
                    "Run `metadata_indexer --base-dir {} index` to refresh the index.",
                    base_dir.display()
                );
            }
        }
        Command::Refresh { books, force } => {
            let unique: HashSet<_> = books.into_iter().map(|b| b.to_lowercase()).collect();
            let mut refreshed = 0usize;
            for book in unique {
                for kind in [ManifestKind::WgetList, ManifestKind::Md5Sums] {
                    match refresh_manifest(&metadata_dir, &book, kind, force) {
                        Ok(path) => {
                            refreshed += 1;
                            println!(
                                "Refreshed {} manifest for {} -> {}",
                                kind.description(),
                                book,
                                path.display()
                            );
                        }
                        Err(err) => {
                            eprintln!(
                                "warning: failed to refresh {} manifest for {}: {}",
                                kind.description(),
                                book,
                                err
                            );
                        }
                    }
                }
            }

            if refreshed == 0 {
                println!("No manifests refreshed (check warnings above).");
            }
        }
    }

    Ok(())
}

fn load_schema(path: &Path) -> Result<(&'static Value, JSONSchema)> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("reading schema file {}", path.display()))?;
    let value: Value = serde_json::from_str(&data)
        .with_context(|| format!("parsing JSON schema {}", path.display()))?;
    let leaked = Box::leak(Box::new(value));
    let schema = JSONSchema::compile(leaked).context("compiling JSON schema")?;
    Ok((leaked, schema))
}

fn scan_packages(dir: &Path) -> Result<Vec<PackageRecord>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
    {
        let path = entry.into_path();
        let data = fs::read_to_string(&path)
            .with_context(|| format!("reading package metadata {}", path.display()))?;
        let value: Value = serde_json::from_str(&data)
            .with_context(|| format!("parsing package JSON {}", path.display()))?;

        let relative_path = path
            .strip_prefix(dir.parent().unwrap_or(Path::new("")))
            .unwrap_or(&path)
            .to_path_buf();

        let (summary, summary_error) = match extract_summary(&value, &relative_path) {
            Ok(summary) => (Some(summary), None),
            Err(err) => (None, Some(err)),
        };

        records.push(PackageRecord {
            value,
            relative_path,
            summary,
            summary_error,
        });
    }

    Ok(records)
}

#[derive(Clone)]
struct PackageSummary {
    schema_version: String,
    id: String,
    name: String,
    version: String,
    stage: Option<String>,
    book: String,
    variant: Option<String>,
    status: String,
    relative_path: String,
}

struct PackageRecord {
    value: Value,
    relative_path: PathBuf,
    summary: Option<PackageSummary>,
    summary_error: Option<anyhow::Error>,
}

fn extract_summary(value: &Value, relative_path: &Path) -> Result<PackageSummary> {
    let schema_version = value
        .get("schema_version")
        .and_then(Value::as_str)
        .context("missing schema_version")?
        .to_string();
    let package = value.get("package").context("missing package block")?;
    let status = value.get("status").context("missing status block")?;

    let id = package
        .get("id")
        .and_then(Value::as_str)
        .context("missing package.id")?
        .to_string();
    let name = package
        .get("name")
        .and_then(Value::as_str)
        .context("missing package.name")?
        .to_string();
    let version = package
        .get("version")
        .and_then(Value::as_str)
        .context("missing package.version")?
        .to_string();
    let book = package
        .get("book")
        .and_then(Value::as_str)
        .context("missing package.book")?
        .to_string();
    let stage = package
        .get("stage")
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let variant = package
        .get("variant")
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let status_state = status
        .get("state")
        .and_then(Value::as_str)
        .context("missing status.state")?
        .to_string();

    Ok(PackageSummary {
        schema_version,
        id,
        name,
        version,
        stage,
        book,
        variant,
        status: status_state,
        relative_path: relative_path
            .to_str()
            .unwrap_or_default()
            .replace('\\', "/"),
    })
}

struct HarvestResult {
    value: Value,
    slug: String,
    package_id: String,
}

fn harvest_metadata(
    metadata_dir: &Path,
    book: &str,
    page: &str,
    override_base: Option<&str>,
) -> Result<HarvestResult> {
    let page_url = resolve_page_url(book, page, override_base)?;
    let client = Client::builder()
        .user_agent("lpkg-metadata-indexer/0.1")
        .build()?;
    let response = client
        .get(&page_url)
        .send()
        .with_context(|| format!("fetching {}", page_url))?
        .error_for_status()
        .with_context(|| format!("non-success status for {}", page_url))?;
    let html = response
        .text()
        .with_context(|| format!("reading response body from {}", page_url))?;

    let document = Html::parse_document(&html);
    let harvest = build_metadata_value(metadata_dir, book, &page_url, &document, &html)?;
    Ok(harvest)
}

fn resolve_page_url(book: &str, page: &str, override_base: Option<&str>) -> Result<String> {
    if page.starts_with("http://") || page.starts_with("https://") {
        return Ok(page.to_string());
    }

    let base = override_base
        .map(|s| s.to_string())
        .or_else(|| default_base_url(book).map(|s| s.to_string()))
        .context("no base URL available for book")?;

    let base = base.trim_end_matches('/');
    let mut page_path = page.trim_start_matches('/').to_string();
    if page_path.is_empty() {
        page_path = "index.html".to_string();
    }
    if !page_path.ends_with(".html") {
        page_path.push_str(".html");
    }

    Ok(format!("{}/{}", base, page_path))
}

fn default_base_url(book: &str) -> Option<&'static str> {
    match book {
        "lfs" => Some("https://www.linuxfromscratch.org/lfs/view/12.1"),
        "mlfs" => Some("https://linuxfromscratch.org/~thomas/multilib-m32"),
        "blfs" => Some("https://www.linuxfromscratch.org/blfs/view/systemd"),
        "glfs" => Some("https://www.linuxfromscratch.org/glfs/view/glfs"),
        _ => None,
    }
}

fn build_metadata_value(
    metadata_dir: &Path,
    book: &str,
    page_url: &str,
    document: &Html,
    html: &str,
) -> Result<HarvestResult> {
    let heading_selector = Selector::parse("h1.sect1").unwrap();
    let heading = document
        .select(&heading_selector)
        .next()
        .context("no <h1 class=sect1> found")?;
    let heading_text = heading
        .text()
        .map(|t| t.replace('\u{00a0}', " "))
        .collect::<Vec<_>>()
        .join(" ");
    let heading_clean = normalize_whitespace(&heading_text);
    let heading_re = Regex::new(r"^(?P<section>\d+\.\d+)\.\s+(?P<title>.+)$")?;
    let caps = heading_re
        .captures(&heading_clean)
        .with_context(|| format!("unable to parse heading '{}'", heading_clean))?;
    let section = caps["section"].to_string();
    let title = caps["title"].trim().to_string();

    let (name, version, variant) = split_name_variant(&title);
    let chapter_num: u32 = section
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let stage = stage_for_chapter(chapter_num).map(|s| s.to_string());

    let slug_base = slugify(&name);
    let slug = variant
        .as_ref()
        .map(|v| format!("{}-{}", slug_base, slugify(v)))
        .unwrap_or_else(|| slug_base.clone());
    let package_id = format!("{}/{}", book, slug);
    let package_id_for_json = package_id.clone();

    let anchor_url = heading
        .value()
        .id()
        .map(|id| format!("{}#{}", page_url, id))
        .or_else(|| locate_child_anchor(&heading).map(|id| format!("{}#{}", page_url, id)))
        .or_else(|| {
            let anchor_selector = Selector::parse("a[id]").unwrap();
            document
                .select(&anchor_selector)
                .filter_map(|a| a.value().attr("id"))
                .find(|id| id.contains(&slug_base))
                .map(|id| format!("{}#{}", page_url, id))
        })
        .or_else(|| {
            let escaped = regex::escape(&slug_base);
            let pattern = format!(r#"id=\"([^\"]*{}[^\"]*)\""#, escaped);
            Regex::new(&pattern)
                .ok()
                .and_then(|re| re.captures(html))
                .and_then(|caps| caps.get(1))
                .map(|m| format!("{}#{}", page_url, m.as_str()))
        });

    let mut source_urls = collect_tarball_urls(page_url, document);
    let mut archive_name = infer_archive_from_commands(document).or_else(|| {
        source_urls.iter().find_map(|entry| {
            entry
                .url
                .path_segments()
                .and_then(|mut iter| iter.next_back())
                .map(|s| s.to_string())
        })
    });

    if source_urls.is_empty() {
        match fallback_urls_from_wget(metadata_dir, book, &slug_base, &version) {
            Ok(fallback) => {
                if !fallback.is_empty() {
                    eprintln!(
                        "info: using {} URL(s) from wget-list for {} {}",
                        fallback.len(),
                        slug_base,
                        version
                    );
                    source_urls = fallback;
                }
            }
            Err(err) => {
                eprintln!(
                    "warning: failed to consult wget-list for {} {}: {}",
                    slug_base, version, err
                );
            }
        }
    }

    if archive_name.is_none() {
        archive_name = source_urls.iter().find_map(|entry| {
            entry
                .url
                .path_segments()
                .and_then(|mut iter| iter.next_back())
                .map(|s| s.to_string())
        });
        if archive_name.is_none() {
            eprintln!(
                "warning: unable to infer archive name from source URLs for {} {}",
                slug_base, version
            );
        }
    }

    let (sbu, disk) = extract_artifacts(document);
    let build_steps = extract_build_steps(document);

    let mut issues = Vec::new();
    if anchor_url.is_none() {
        issues.push("Could not locate anchor id for primary heading".to_string());
    }
    if source_urls.is_empty() {
        issues.push("No source URLs with archive extensions detected".to_string());
    }
    if build_steps.is_empty() {
        issues.push("No <pre class=\"userinput\"> blocks found for build commands".to_string());
    }

    let source_urls_json: Vec<Value> = source_urls
        .iter()
        .map(|entry| {
            json!({
                "url": entry.url.as_str(),
                "kind": entry.kind,
            })
        })
        .collect();

    let checksum_entries = match resolve_checksums(metadata_dir, book, archive_name.as_deref()) {
        Ok(values) => values,
        Err(err) => {
            eprintln!(
                "warning: failed to resolve checksums for {} {}: {}",
                slug_base, version, err
            );
            Vec::new()
        }
    };

    let build_json: Vec<Value> = build_steps
        .iter()
        .map(|step| {
            json!({
                "phase": step.phase,
                "commands": step.commands,
                "cwd": step.cwd,
                "requires_root": step.requires_root,
                "notes": step.notes,
            })
        })
        .collect();

    let body_selector = Selector::parse("body").unwrap();
    let book_release = document
        .select(&body_selector)
        .next()
        .and_then(|body| body.value().id())
        .map(|id| id.to_string())
        .unwrap_or_default();

    let retrieved_at = Utc::now().to_rfc3339();
    let content_hash = hex::encode(Sha256::digest(html.as_bytes()));

    let anchors_value = match anchor_url {
        Some(ref href) => json!({ "section": href }),
        None => json!({}),
    };

    let status_state = "draft";

    let package_json = json!({
        "schema_version": "v0.1.0",
        "package": {
            "id": package_id_for_json,
            "name": name,
            "upstream": Option::<String>::None,
            "version": version,
            "book": book,
            "chapter": chapter_num,
            "section": section,
            "stage": stage,
            "variant": variant,
            "anchors": anchors_value,
        },
        "source": {
            "urls": source_urls_json,
            "archive": archive_name,
            "checksums": checksum_entries,
        },
        "artifacts": {
            "sbu": sbu,
            "disk": disk,
            "install_prefix": Option::<String>::None,
        },
        "dependencies": {
            "build": Vec::<Value>::new(),
            "runtime": Vec::<Value>::new(),
        },
        "environment": {
            "variables": Vec::<Value>::new(),
            "users": Vec::<Value>::new(),
        },
        "build": build_json,
        "optimizations": {
            "enable_lto": true,
            "enable_pgo": true,
            "cflags": ["-O3", "-flto"],
            "ldflags": ["-flto"],
            "profdata": Option::<String>::None,
        },
        "provenance": {
            "book_release": book_release,
            "page_url": page_url,
            "retrieved_at": retrieved_at,
            "content_hash": content_hash,
        },
        "status": {
            "state": status_state,
            "issues": issues,
        }
    });

    Ok(HarvestResult {
        value: package_json,
        slug,
        package_id,
    })
}

fn normalize_whitespace(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_space = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            prev_space = false;
            result.push(ch);
        }
    }
    result.trim().to_string()
}

fn slugify(input: &str) -> String {
    let mut result = String::new();
    let mut prev_dash = false;
    for ch in input.chars() {
        let normalized = match ch {
            'A'..='Z' => ch.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' => ch,
            _ => '-',
        };
        if normalized == '-' {
            if !prev_dash {
                result.push('-');
                prev_dash = true;
            }
        } else {
            prev_dash = false;
            result.push(normalized);
        }
    }
    result.trim_matches('-').to_string()
}

fn split_name_variant(title: &str) -> (String, String, Option<String>) {
    let mut base = title.trim().to_string();
    let mut variant = None;
    if let Some(idx) = base.rfind(" - ") {
        variant = Some(base[idx + 3..].trim().to_string());
        base = base[..idx].trim().to_string();
    }

    let bytes = base.as_bytes();
    for idx in (0..bytes.len()).rev() {
        if bytes[idx] == b'-' {
            if let Some(next) = bytes.get(idx + 1) {
                if next.is_ascii_digit() {
                    let name = base[..idx].trim();
                    let version = base[idx + 1..].trim();
                    if !name.is_empty() && !version.is_empty() {
                        return (name.to_string(), version.to_string(), variant);
                    }
                }
            }
        }
    }

    (base, String::from("unknown"), variant)
}

fn stage_for_chapter(chapter: u32) -> Option<&'static str> {
    match chapter {
        5 => Some("cross-toolchain"),
        6 | 7 => Some("temporary-tools"),
        8 => Some("system"),
        9 => Some("system-configuration"),
        10 => Some("system-finalization"),
        _ => None,
    }
}

struct SourceUrlEntry {
    url: url::Url,
    kind: &'static str,
}

#[derive(Clone, Copy)]
enum ManifestKind {
    WgetList,
    Md5Sums,
}

impl ManifestKind {
    fn filename(&self) -> &'static str {
        match self {
            ManifestKind::WgetList => "wget-list.txt",
            ManifestKind::Md5Sums => "md5sums.txt",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            ManifestKind::WgetList => "wget-list",
            ManifestKind::Md5Sums => "md5sums",
        }
    }
}

fn collect_tarball_urls(page_url: &str, document: &Html) -> Vec<SourceUrlEntry> {
    let base = url::Url::parse(page_url).ok();
    let link_selector = Selector::parse("a").unwrap();
    let mut seen = HashSet::new();
    let mut results = Vec::new();

    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            if let Some(kind) = classify_artifact_url(href) {
                let resolved = match (&base, url::Url::parse(href)) {
                    (_, Ok(url)) => url,
                    (Some(base_url), Err(_)) => match base_url.join(href) {
                        Ok(url) => url,
                        Err(_) => continue,
                    },
                    _ => continue,
                };
                if seen.insert(resolved.clone()) {
                    results.push(SourceUrlEntry {
                        url: resolved,
                        kind,
                    });
                }
            }
        }
    }

    results
}

fn classify_artifact_url(href: &str) -> Option<&'static str> {
    let lower = href.to_lowercase();
    if lower.ends_with(".tar")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".tar.bz2")
        || lower.ends_with(".tar.xz")
        || lower.ends_with(".tgz")
        || lower.ends_with(".zip")
    {
        Some("primary")
    } else if lower.ends_with(".patch") {
        Some("patch")
    } else if lower.ends_with(".sig") || lower.ends_with(".asc") {
        Some("signature")
    } else {
        None
    }
}

fn fallback_urls_from_wget(
    metadata_dir: &Path,
    book: &str,
    slug: &str,
    version: &str,
) -> Result<Vec<SourceUrlEntry>> {
    let manifest = load_jhalfs_manifest(metadata_dir, book, ManifestKind::WgetList)?;
    let needle = format!("{}-{}", slug.replace('_', "-"), version);
    eprintln!("debug: searching wget-list for '{}'", needle);
    let mut entries = Vec::new();
    for line in manifest.lines() {
        if line.contains(&needle) {
            if let Ok(url) = url::Url::parse(line.trim()) {
                eprintln!("info: matched wget URL {}", url);
                entries.push(SourceUrlEntry {
                    url,
                    kind: "primary",
                });
            } else {
                eprintln!(
                    "warning: unable to parse URL from wget-list line: {}",
                    line.trim()
                );
            }
        }
    }
    if entries.is_empty() {
        eprintln!("warning: no wget-list entries matched '{}'", needle);
    }
    Ok(entries)
}

fn resolve_checksums(
    metadata_dir: &Path,
    book: &str,
    archive_name: Option<&str>,
) -> Result<Vec<Value>> {
    let mut checksums = Vec::new();
    let Some(archive) = archive_name else {
        return Ok(checksums);
    };

    let manifest = load_jhalfs_manifest(metadata_dir, book, ManifestKind::Md5Sums)?;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(hash) = parts.next() else { continue };
        let Some(file) = parts.next() else { continue };
        if file == archive {
            checksums.push(json!({
                "alg": "md5",
                "value": hash.to_lowercase(),
            }));
            break;
        }
    }

    Ok(checksums)
}

fn load_jhalfs_manifest(metadata_dir: &Path, book: &str, kind: ManifestKind) -> Result<String> {
    let cache_path = refresh_manifest(metadata_dir, book, kind, false)?;
    fs::read_to_string(&cache_path)
        .with_context(|| format!("reading cached manifest {}", cache_path.display()))
}

fn refresh_manifest(
    metadata_dir: &Path,
    book: &str,
    kind: ManifestKind,
    force: bool,
) -> Result<PathBuf> {
    let cache_dir = metadata_dir.join("cache");
    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("creating cache directory {}", cache_dir.display()))?;

    let cache_path = cache_dir.join(format!("{}-{}", book, kind.filename()));
    if cache_path.exists() && !force {
        return Ok(cache_path);
    }

    let url = manifest_url(book, &kind)
        .with_context(|| format!("no manifest URL configured for book '{}'", book))?;

    let client = Client::builder().redirect(Policy::limited(5)).build()?;
    let body = client
        .get(url)
        .send()
        .with_context(|| format!("fetching {}", url))?
        .error_for_status()
        .with_context(|| format!("request failed for {}", url))?
        .text()
        .with_context(|| format!("reading response body from {}", url))?;

    fs::write(&cache_path, &body)
        .with_context(|| format!("caching manifest {}", cache_path.display()))?;

    Ok(cache_path)
}

fn manifest_url(book: &str, kind: &ManifestKind) -> Option<&'static str> {
    match (book, kind) {
        ("mlfs", ManifestKind::WgetList) => {
            Some("https://www.linuxfromscratch.org/~thomas/multilib-m32/wget-list-sysv")
        }
        ("mlfs", ManifestKind::Md5Sums) => {
            Some("https://www.linuxfromscratch.org/~thomas/multilib-m32/md5sums")
        }
        ("lfs", ManifestKind::WgetList) => {
            Some("https://www.linuxfromscratch.org/lfs/view/12.1/wget-list")
        }
        ("lfs", ManifestKind::Md5Sums) => {
            Some("https://www.linuxfromscratch.org/lfs/view/12.1/md5sums")
        }
        ("blfs", ManifestKind::WgetList) => {
            Some("https://anduin.linuxfromscratch.org/BLFS/view/systemd/wget-list")
        }
        ("blfs", ManifestKind::Md5Sums) => {
            Some("https://anduin.linuxfromscratch.org/BLFS/view/systemd/md5sums")
        }
        ("glfs", ManifestKind::WgetList) => {
            Some("https://www.linuxfromscratch.org/glfs/view/glfs/wget-list")
        }
        ("glfs", ManifestKind::Md5Sums) => {
            Some("https://www.linuxfromscratch.org/glfs/view/glfs/md5sums")
        }
        _ => None,
    }
}

fn locate_child_anchor(heading: &ElementRef) -> Option<String> {
    let mut current = heading.first_child();
    while let Some(node) = current {
        if let Some(element) = ElementRef::wrap(node) {
            if let Some(id) = element
                .value()
                .attr("id")
                .or_else(|| element.value().attr("name"))
            {
                return Some(id.to_string());
            }
        }
        current = node.next_sibling();
    }
    None
}

fn infer_archive_from_commands(document: &Html) -> Option<String> {
    let pre_selector = Selector::parse("pre.userinput").unwrap();
    for pre in document.select(&pre_selector) {
        let text = pre.text().collect::<Vec<_>>().join("\n");
        for line in text.lines() {
            if let Some(start) = line.find("tar -xf") {
                let args = line[start + 7..].trim();
                let parts: Vec<&str> = args.split_whitespace().collect();
                if let Some(archive) = parts.get(0) {
                    let cleaned = archive.trim_matches(['"', '\'', ','].as_ref());
                    if cleaned.ends_with(".tar")
                        || cleaned.contains(".tar.")
                        || cleaned.ends_with(".tgz")
                        || cleaned.ends_with(".zip")
                    {
                        return Some(cleaned.trim_start_matches("../").to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_artifacts(document: &Html) -> (Option<f64>, Option<i64>) {
    let seg_selector = Selector::parse("div.segmentedlist div.seg").unwrap();
    let title_selector = Selector::parse("strong.segtitle").unwrap();
    let body_selector = Selector::parse("span.segbody").unwrap();
    let mut sbu = None;
    let mut disk = None;

    for seg in document.select(&seg_selector) {
        let title = seg
            .select(&title_selector)
            .next()
            .map(|n| normalize_whitespace(&n.text().collect::<Vec<_>>().join("")));
        let body = seg
            .select(&body_selector)
            .next()
            .map(|n| normalize_whitespace(&n.text().collect::<Vec<_>>().join("")));

        if let (Some(title), Some(body)) = (title, body) {
            if title.contains("Approximate build time") {
                if let Some(value) = parse_numeric(&body) {
                    sbu = Some(value);
                }
            } else if title.contains("Required disk space") {
                if let Some(value) = parse_numeric(&body) {
                    disk = Some(value as i64);
                }
            }
        }
    }

    (sbu, disk)
}

fn parse_numeric(input: &str) -> Option<f64> {
    let re = Regex::new(r"([0-9]+(?:\\.[0-9]+)?)").ok()?;
    re.captures(input)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

struct BuildStep {
    phase: &'static str,
    commands: Vec<String>,
    cwd: Option<String>,
    requires_root: bool,
    notes: Option<String>,
}

fn extract_build_steps(document: &Html) -> Vec<BuildStep> {
    let pre_selector = Selector::parse("pre.userinput").unwrap();
    let mut steps = Vec::new();

    for pre in document.select(&pre_selector) {
        let code = pre.text().collect::<Vec<_>>().join("\n");
        let commands: Vec<String> = code
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        if commands.is_empty() {
            continue;
        }

        let phase = classify_phase(&commands);
        steps.push(BuildStep {
            phase,
            commands,
            cwd: None,
            requires_root: false,
            notes: None,
        });
    }

    steps
}

fn classify_phase(commands: &[String]) -> &'static str {
    let joined = commands.join("\n").to_lowercase();
    if joined.contains("make install") {
        "install"
    } else if joined.contains("make -k check") || joined.contains("make check") {
        "test"
    } else if joined.contains("configure") {
        "configure"
    } else if joined.contains("tar -xf") || joined.contains("mkdir ") {
        "setup"
    } else {
        "build"
    }
}
