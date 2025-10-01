use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use crate::pkgs::scaffolder::{self, ScaffoldRequest};

#[derive(Debug, Deserialize)]
struct HarvestedPackage {
    package: HarvestedMetadata,
    source: HarvestedSource,
    #[serde(default)]
    build: Vec<CommandPhase>,
    #[serde(default)]
    dependencies: Option<HarvestedDependencies>,
    optimizations: HarvestedOptimisations,
}

#[derive(Debug, Deserialize)]
struct HarvestedMetadata {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    stage: Option<String>,
    #[serde(default)]
    variant: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HarvestedSource {
    #[serde(default)]
    archive: Option<String>,
    #[serde(default)]
    urls: Vec<HarvestedUrl>,
    #[serde(default)]
    checksums: Vec<HarvestedChecksum>,
}

#[derive(Debug, Deserialize)]
struct HarvestedUrl {
    url: String,
}

#[derive(Debug, Deserialize)]
struct HarvestedChecksum {
    alg: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct HarvestedOptimisations {
    enable_lto: bool,
    enable_pgo: bool,
    #[serde(default)]
    cflags: Vec<String>,
    #[serde(default)]
    ldflags: Vec<String>,
    #[serde(default)]
    profdata: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CommandPhase {
    #[serde(default)]
    phase: Option<String>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    requires_root: Option<bool>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HarvestedDependencies {
    #[serde(default)]
    build: Vec<String>,
    #[serde(default)]
    runtime: Vec<String>,
}

/// Generate a Rust module from harvested metadata, returning the path to the generated file.
pub fn generate_module(
    metadata_path: impl AsRef<Path>,
    base_dir: impl AsRef<Path>,
) -> Result<PathBuf> {
    let harvested = parse_metadata(metadata_path.as_ref())?;
    let request = build_request(&harvested)?;
    let result = scaffolder::scaffold_package(base_dir.as_ref(), request)?;
    Ok(result.module_path)
}

/// Compute the directory for a module derived from the given metadata.
pub fn module_directory(
    metadata_path: impl AsRef<Path>,
    base_dir: impl AsRef<Path>,
) -> Result<PathBuf> {
    let harvested = parse_metadata(metadata_path.as_ref())?;
    let slug = module_override_from_id(&harvested.package.id).ok_or_else(|| {
        anyhow!(
            "unable to derive module slug from id '{}'",
            harvested.package.id
        )
    })?;
    let module = sanitize_module_name(&slug);
    let dir = base_dir
        .as_ref()
        .join(prefix_from_module(&module))
        .join(module);
    Ok(dir)
}

fn build_request(pkg: &HarvestedPackage) -> Result<ScaffoldRequest> {
    let slug = module_override_from_id(&pkg.package.id)
        .ok_or_else(|| anyhow!("unable to derive module slug from id '{}'", pkg.package.id))?;

    let mut build_commands = Vec::new();
    let mut install_commands = Vec::new();
    for command in flatten_commands(&pkg.build) {
        if command.contains("make install") {
            install_commands.push(command);
        } else {
            build_commands.push(command);
        }
    }

    let mut dependencies = HashSet::new();
    if let Some(deps) = &pkg.dependencies {
        for dep in &deps.build {
            dependencies.insert(dep.clone());
        }
        for dep in &deps.runtime {
            dependencies.insert(dep.clone());
        }
    }
    let mut dependencies: Vec<String> = dependencies.into_iter().collect();
    dependencies.sort();

    let request = ScaffoldRequest {
        name: pkg.package.name.clone(),
        version: pkg.package.version.clone(),
        source: pkg.source.urls.first().map(|u| u.url.clone()),
        md5: pkg
            .source
            .checksums
            .iter()
            .find(|c| c.alg.eq_ignore_ascii_case("md5"))
            .map(|c| c.value.clone()),
        configure_args: Vec::new(),
        build_commands,
        install_commands,
        dependencies,
        enable_lto: pkg.optimizations.enable_lto,
        enable_pgo: pkg.optimizations.enable_pgo,
        cflags: pkg.optimizations.cflags.clone(),
        ldflags: pkg.optimizations.ldflags.clone(),
        profdata: pkg.optimizations.profdata.clone(),
        stage: pkg.package.stage.clone(),
        variant: pkg.package.variant.clone(),
        notes: pkg.package.notes.clone(),
        module_override: Some(slug),
    };

    Ok(request)
}

fn flatten_commands(phases: &[CommandPhase]) -> Vec<String> {
    phases
        .iter()
        .flat_map(|phase| phase.commands.iter().cloned())
        .collect()
}

fn module_override_from_id(id: &str) -> Option<String> {
    let slug = match id.split_once('/') {
        Some((_, slug)) => slug,
        None => id,
    };
    Some(
        slug.replace('.', "_")
            .replace('/', "_")
            .replace('-', "_")
            .replace(' ', "_")
            .to_lowercase(),
    )
}

fn parse_metadata(path: &Path) -> Result<HarvestedPackage> {
    let metadata = fs::read_to_string(path)
        .with_context(|| format!("reading metadata file {}", path.display()))?;
    let harvested: HarvestedPackage = serde_json::from_str(&metadata)
        .with_context(|| format!("parsing harvested metadata from {}", path.display()))?;
    Ok(harvested)
}

fn sanitize_module_name(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == '_' || ch == '+' || ch == '-' {
            out.push('_');
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("pkg");
    }
    if out
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        out.insert(0, 'p');
    }
    out
}

fn prefix_from_module(module: &str) -> String {
    let mut chars = module.chars();
    let first = chars.next().unwrap_or('p');
    let second = chars.next().unwrap_or('k');
    let mut s = String::new();
    s.push(first);
    s.push(second);
    s
}
