pub mod blfs;
pub mod glfs;
pub mod lfs;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BookKind {
    Lfs,
    Mlfs,
    Blfs,
    Glfs,
}

impl fmt::Display for BookKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            BookKind::Lfs => "lfs",
            BookKind::Mlfs => "mlfs",
            BookKind::Blfs => "blfs",
            BookKind::Glfs => "glfs",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone)]
pub struct BookPackage {
    pub book: BookKind,
    pub chapter: Option<u32>,
    pub section: Option<String>,
    pub name: String,
    pub version: Option<String>,
    pub href: Option<String>,
    pub md5: Option<String>,
    pub stage: Option<String>,
    pub variant: Option<String>,
    pub notes: Option<String>,
}

impl BookPackage {
    pub fn identifier(&self) -> String {
        match &self.variant {
            Some(variant) if !variant.is_empty() => {
                format!(
                    "{}-{}-{}",
                    self.book,
                    self.name,
                    variant.replace(' ', "-").to_lowercase()
                )
            }
            _ => format!("{}-{}", self.book, self.name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchOptions<'a> {
    pub base_url: &'a str,
    pub book: BookKind,
}

impl<'a> FetchOptions<'a> {
    pub fn new(base_url: &'a str, book: BookKind) -> Self {
        Self { base_url, book }
    }
}
