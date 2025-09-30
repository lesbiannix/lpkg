use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;

pub fn get_wget_list() -> Result<String> {
    let client = Client::builder().redirect(Policy::limited(5)).build()?;
    let res = client
        .get("https://www.linuxfromscratch.org/~thomas/multilib-m32/wget-list-sysv")
        .send()?;

    if !res.status().is_success() {
        anyhow::bail!("Failed to fetch wget-list: HTTP {}", res.status());
    }

    Ok(res.text()?)
}
