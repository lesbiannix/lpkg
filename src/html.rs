use scraper::{Html, Selector};

pub fn fetch_pre_blocks(url: &str) -> anyhow::Result<Vec<String>> {
    let body = reqwest::blocking::get(url)?.text()?;
    let document = Html::parse_document(&body);
    let selector = Selector::parse("pre").unwrap();

    let mut results = Vec::new();
    for element in document.select(&selector) {
        results.push(element.inner_html());
    }

    Ok(results)
}
