use std::borrow::Cow;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::ingest::{BookKind, BookPackage, FetchOptions, lfs};
use crate::pkgs::package::PackageDefinition;

pub const DEFAULT_MLFS_BASE_URL: &str = "https://linuxfromscratch.org/~thomas/multilib-m32";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlfsPackageRecord {
    pub name: String,
    pub version: String,
    pub chapter: Option<u32>,
    pub section: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub variant: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl MlfsPackageRecord {
    pub fn id(&self) -> String {
        let mut id = self.name.replace('+', "plus");
        if let Some(variant) = &self.variant {
            id.push('_');
            id.push_str(&variant.replace('-', "_"));
        }
        id
    }

    pub fn module_alias(&self) -> String {
        self.id()
            .replace('.', "_")
            .replace('/', "_")
            .replace(' ', "_")
            .to_lowercase()
    }

    pub fn display_label(&self) -> Cow<'_, str> {
        match (&self.section, &self.variant) {
            (Some(section), Some(variant)) => Cow::from(format!("{} ({})", section, variant)),
            (Some(section), None) => Cow::from(section.as_str()),
            (None, Some(variant)) => Cow::from(variant.as_str()),
            _ => Cow::from(self.name.as_str()),
        }
    }

    pub fn to_package_definition(&self) -> PackageDefinition {
        let mut pkg = PackageDefinition::new(&self.name, &self.version);
        if let Some(stage) = &self.stage {
            pkg.optimizations
                .cflags
                .push(format!("-DLPKG_STAGE={}", stage.to_uppercase()));
        }
        if let Some(variant) = &self.variant {
            pkg.optimizations
                .cflags
                .push(format!("-DLPKG_VARIANT={}", variant.to_uppercase()));
        }
        if let Some(notes) = &self.notes {
            pkg.optimizations
                .cflags
                .push(format!("-DLPKG_NOTES={}", notes.replace(' ', "_")));
        }
        pkg
    }

    fn from_book_package(pkg: BookPackage) -> Option<Self> {
        let version = pkg.version?;
        Some(Self {
            name: pkg.name,
            version,
            chapter: pkg.chapter,
            section: pkg.section,
            stage: pkg.stage,
            variant: pkg.variant,
            notes: pkg.notes,
        })
    }
}

pub fn fetch_catalog(base_url: &str) -> Result<Vec<MlfsPackageRecord>> {
    let options = FetchOptions::new(base_url, BookKind::Mlfs);
    let packages = lfs::fetch_book(&options)?;
    let mut records = packages
        .into_iter()
        .filter_map(MlfsPackageRecord::from_book_package)
        .collect::<Vec<_>>();
    if records.is_empty() {
        return Err(anyhow!("No packages parsed from MLFS book at {base_url}."));
    }
    records.sort_by(|a, b| a.name.cmp(&b.name).then(a.variant.cmp(&b.variant)));
    Ok(records)
}

pub fn load_cached_catalog() -> Result<Vec<MlfsPackageRecord>> {
    let raw = include_str!("../../data/mlfs_ml-12.4-40-multilib.json");
    let records: Vec<MlfsPackageRecord> =
        serde_json::from_str(raw).context("parsing cached MLFS package manifest")?;
    Ok(records)
}

pub fn load_or_fetch_catalog(base_url: Option<&str>) -> Result<Vec<MlfsPackageRecord>> {
    let base = base_url.unwrap_or(DEFAULT_MLFS_BASE_URL);
    match fetch_catalog(base) {
        Ok(records) => Ok(records),
        Err(err) => {
            tracing::warn!("mlfs_fetch_error" = %err, "Falling back to cached MLFS package list");
            load_cached_catalog()
        }
    }
}
