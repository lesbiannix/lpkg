use anyhow::{Context, Result};
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use super::{BookPackage, FetchOptions};

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

        let stage = stage_for_chapter(chapter_num).map(|s| s.to_string());
        let identifier = format!("{chapter_num}.{section_num:02}");

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
            section: Some(identifier),
            name,
            version: Some(version),
            href,
            md5: None,
            stage,
            variant,
            notes: None,
        });
    }

    Ok(results)
}

pub(crate) fn split_name_version(title: &str) -> Option<(String, String, Option<String>)> {
    // Find the last '-' whose next character is a digit (start of version)
    let bytes = title.as_bytes();
    for idx in (0..bytes.len()).rev() {
        if bytes[idx] == b'-' {
            if let Some(next) = bytes.get(idx + 1) {
                if next.is_ascii_digit() {
                    let name = title[..idx].trim();
                    let mut remainder = title[idx + 1..].trim();
                    if name.is_empty() || remainder.is_empty() {
                        return None;
                    }

                    let mut variant = None;
                    if let Some(pos) = remainder.find(" - ") {
                        variant = Some(remainder[pos + 3..].trim().to_string());
                        remainder = remainder[..pos].trim();
                    } else if let Some(pos) = remainder.find(" (") {
                        let note = remainder[pos + 1..].trim_end_matches(')').trim();
                        variant = Some(note.to_string());
                        remainder = remainder[..pos].trim();
                    }

                    return Some((name.to_string(), remainder.to_string(), variant));
                }
            }
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::BookKind;
    use scraper::{Html, Selector};

    #[test]
    fn parse_sample_headings() {
        let html = r#"
        <html><body>
        <h1 class=\"sect1\" id=\"ch05-binutils-pass1\">5.5. Binutils-2.45 - Pass 1</h1>
        <h1 class=\"sect1\" id=\"ch05-gcc-pass1\">5.6. GCC-15.2.0 - Pass 1</h1>
        <h1 class=\"sect1\" id=\"ch09-bootscripts\">9.3. LFS-Bootscripts-20250827</h1>
        <h1 class=\"sect1\" id=\"ch08-xml-parser\">8.41. XML::Parser-2.47</h1>
        </body></html>
        "#;
        let opts = FetchOptions::new("https://example.invalid/lfs", BookKind::Mlfs);
        let document = Html::parse_document(html);
        let selector = Selector::parse("h1.sect1").unwrap();
        assert!(
            document.select(&selector).next().is_some(),
            "sample headings selector returned no nodes"
        );
        let packages =
            parse_book_html(&opts, "https://example.invalid/lfs/book.html", html).unwrap();
        assert_eq!(packages.len(), 4);
        assert_eq!(packages[0].name, "Binutils");
        assert_eq!(packages[0].version.as_deref(), Some("2.45"));
        assert_eq!(packages[0].variant.as_deref(), Some("Pass 1"));
        assert_eq!(packages[0].stage.as_deref(), Some("cross-toolchain"));
        assert_eq!(packages[1].variant.as_deref(), Some("Pass 1"));
        assert_eq!(packages[2].variant, None);
        assert_eq!(packages[3].name, "XML::Parser");
    }
}
