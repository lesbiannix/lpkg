pub mod ai;
pub mod db;
#[cfg(feature = "graphql")]
pub mod graphql;
pub mod html;
pub mod ingest;
pub mod md5_utils;
pub mod mirrors;
pub mod pkgs;
pub mod svg_builder;
pub mod version_check;
pub mod wget_list;

#[cfg(feature = "tui")]
pub mod tui;
