use console::Style;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::io::{self, Write};

pub fn fetch_mirrors() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let res = client
        .get("https://www.linuxfromscratch.org/lfs/mirrors.html#files")
        .send()?
        .text()?;

    let document = Html::parse_document(&res);
    let selector = Selector::parse("a[href^='http']").unwrap();

    let mirrors = document
        .select(&selector)
        .filter_map(|element| {
            let href = element.value().attr("href")?;
            if href.contains("ftp.gnu.org") || href.contains("mirror") {
                Some(href.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(mirrors)
}

pub fn choose_package_mirror() -> Option<String> {
    let mirrors = fetch_mirrors().unwrap_or_else(|_| {
        vec![
            "ftp.fau.de".to_string(),
            "mirror.kernel.org".to_string(),
            "mirror.example.org".to_string(),
        ]
    });

    println!("Optional: choose a mirror for GNU source packages:");
    for (i, mirror) in mirrors.iter().enumerate() {
        println!("  [{}] {}", i + 1, mirror);
    }

    print!("Enter number or press Enter for default: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        None
    } else {
        let choice = input.parse::<usize>().unwrap_or(1);
        let chosen = mirrors.get(choice.saturating_sub(1)).unwrap_or(&mirrors[0]);
        println!(
            "Using package mirror: {}",
            Style::new().green().apply_to(chosen)
        );
        Some(chosen.to_string())
    }
}
