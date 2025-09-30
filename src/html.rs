use html_parser::Dom;
use reqwest::blocking::get;
use std::error::Error;

/// Lädt die HTML-Seite von der angegebenen URL herunter und konvertiert sie in JSON
pub fn fetch_and_parse_html_to_json(url: &str) -> Result<String, Box<dyn Error>> {
    // HTML herunterladen
    let response = get(url)?;
    if !response.status().is_success() {
        return Err(format!("Fehler beim Abrufen der URL {}: {}", url, response.status()).into());
    }

    let body = response.text()?;

    // HTML parsen
    let dom = Dom::parse(&body)?;

    // In JSON konvertieren
    let json = dom.to_json_pretty()?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_and_parse() {
        let url = "https://www.linuxfromscratch.org/~thomas/multilib-m32/chapter02/hostreqs.html";
        let json = fetch_and_parse_html_to_json(url).expect("Fehler beim Parsen");
        assert!(json.contains("Host System Requirements"));
    }
}
