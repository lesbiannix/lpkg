use anyhow::{Context, Result};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::pkgs::package::PackageDefinition;

use super::schema::packages;

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct Package {
    pub id: i32,
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub md5: Option<String>,
    pub configure_args: Option<String>,
    pub build_commands: Option<String>,
    pub install_commands: Option<String>,
    pub dependencies: Option<String>,
    pub enable_lto: bool,
    pub enable_pgo: bool,
    pub cflags: Option<String>,
    pub ldflags: Option<String>,
    pub profdata: Option<String>,
}

impl Package {
    pub fn into_definition(self) -> Result<PackageDefinition> {
        Ok(PackageDefinition {
            name: self.name,
            version: self.version,
            source: self.source,
            md5: self.md5,
            configure_args: parse_vec(self.configure_args)?,
            build_commands: parse_vec(self.build_commands)?,
            install_commands: parse_vec(self.install_commands)?,
            dependencies: parse_vec(self.dependencies)?,
            optimizations: crate::pkgs::package::OptimizationSettings {
                enable_lto: self.enable_lto,
                enable_pgo: self.enable_pgo,
                cflags: parse_vec(self.cflags)?,
                ldflags: parse_vec(self.ldflags)?,
                profdata: self.profdata,
            },
        })
    }
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = packages)]
pub struct NewPackage {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub md5: Option<String>,
    pub configure_args: Option<String>,
    pub build_commands: Option<String>,
    pub install_commands: Option<String>,
    pub dependencies: Option<String>,
    pub enable_lto: bool,
    pub enable_pgo: bool,
    pub cflags: Option<String>,
    pub ldflags: Option<String>,
    pub profdata: Option<String>,
}

impl TryFrom<&PackageDefinition> for NewPackage {
    type Error = anyhow::Error;

    fn try_from(value: &PackageDefinition) -> Result<Self> {
        Ok(Self {
            name: value.name.clone(),
            version: value.version.clone(),
            source: value.source.clone(),
            md5: value.md5.clone(),
            configure_args: serialize_vec(&value.configure_args)?,
            build_commands: serialize_vec(&value.build_commands)?,
            install_commands: serialize_vec(&value.install_commands)?,
            dependencies: serialize_vec(&value.dependencies)?,
            enable_lto: value.optimizations.enable_lto,
            enable_pgo: value.optimizations.enable_pgo,
            cflags: serialize_vec(&value.optimizations.cflags)?,
            ldflags: serialize_vec(&value.optimizations.ldflags)?,
            profdata: value.optimizations.profdata.clone(),
        })
    }
}

fn serialize_vec(values: &[String]) -> Result<Option<String>> {
    if values.is_empty() {
        Ok(None)
    } else {
        serde_json::to_string(values)
            .map(Some)
            .context("serializing vector to JSON")
    }
}

fn parse_vec(raw: Option<String>) -> Result<Vec<String>> {
    match raw {
        Some(data) => serde_json::from_str(&data).context("parsing JSON vector"),
        None => Ok(Vec::new()),
    }
}
