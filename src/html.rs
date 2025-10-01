use anyhow::{Context, Result};
use scraper::{Html, Selector};

pub fn fetch_pre_blocks(url: &str) -> Result<Vec<String>> {
    let body = ureq::get(url)
        .call()
        .with_context(|| format!("requesting {url}"))?
        .into_string()
        .with_context(|| format!("reading body from {url}"))?;
    let document = Html::parse_document(&body);
    let selector = Selector::parse("pre").unwrap();

    let mut results = Vec::new();
    for element in document.select(&selector) {
        results.push(element.inner_html());
    }

    Ok(results)
}
