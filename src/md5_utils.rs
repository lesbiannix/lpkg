use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;

pub fn get_md5sums() -> Result<String> {
    let client = Client::builder().redirect(Policy::none()).build()?;
    let res = client
        .get("https://www.linuxfromscratch.org/~thomas/multilib-m32/md5sums")
        .send()?
        .text()?;
    Ok(res)
}
