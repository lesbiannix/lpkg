use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;

pub fn get_md5sums() -> Result<String> {
    let client = Client::builder().redirect(Policy::limited(5)).build()?;
    let res = client
        .get("https://www.linuxfromscratch.org/~thomas/multilib-m32/md5sums")
        .send()?;

    if !res.status().is_success() {
        anyhow::bail!("Failed to fetch MD5sums: HTTP {}", res.status());
    }

    Ok(res.text()?)
}
