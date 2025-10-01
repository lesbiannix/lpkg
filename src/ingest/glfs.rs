use anyhow::{Context, Result};
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use super::{BookPackage, FetchOptions};
use crate::ingest::lfs::split_name_version;

pub fn fetch_book(options: &FetchOptions) -> Result<Vec<BookPackage>> {
    let base = options.base_url.trim_end_matches('/');
    let url = format!("{base}/book.html");

    let client = Client::builder().build().context("building HTTP client")?;
    let body = client
        .get(&url)
        .send()
        .with_context(|| format!("fetching {}", url))?
        .error_for_status()
        .with_context(|| format!("request failed for {}", url))?
        .text()
        .context("reading response body")?;

    parse_book_html(options, &url, &body)
}

pub fn parse_book_html(
    options: &FetchOptions,
    book_url: &str,
    body: &str,
) -> Result<Vec<BookPackage>> {
    let document = Html::parse_document(body);
    let selector = Selector::parse("h1.sect1").unwrap();
    let numbering_re =
        Regex::new(r"^(?P<chapter>\d+)\.(?P<section>\d+)\.\s+(?P<title>.+)$").unwrap();

    let mut results = Vec::new();

    for heading in document.select(&selector) {
        let text = heading
            .text()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .replace('\n', " ")
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }

        let caps = match numbering_re.captures(&text) {
            Some(caps) => caps,
            None => continue,
        };

        let chapter_num: u32 = caps["chapter"].parse().unwrap_or(0);
        let section_num: u32 = caps["section"].parse().unwrap_or(0);
        let title = caps["title"].trim();

        let (name, version, variant) = match split_name_version(title) {
            Some(parts) => parts,
            None => continue,
        };

        let href = heading.value().id().map(|id| {
            let mut base = book_url.to_string();
            if !base.contains('#') {
                base.push('#');
            }
            format!("{}{}", base, id)
        });

        results.push(BookPackage {
            book: options.book,
            chapter: Some(chapter_num),
            section: Some(format!("{}.{}", chapter_num, section_num)),
            name,
            version: Some(version),
            href,
            md5: None,
            stage: None,
            variant,
            notes: None,
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::BookKind;

    #[test]
    fn parse_glfs_sample() {
        let html = r#"
        <html><body>
        <h1 class=\"sect1\" id=\"ch12-coreutils\">12.4. Coreutils-9.8</h1>
        </body></html>
        "#;
        let opts = FetchOptions::new("https://example.invalid/glfs", BookKind::Glfs);
        let items = parse_book_html(&opts, "https://example.invalid/glfs/book.html", html).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Coreutils");
        assert_eq!(items[0].version.as_deref(), Some("9.8"));
    }
}
