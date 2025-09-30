use reqwest::blocking::Client;
use reqwest::redirect::Policy;

pub fn get_wget_list() -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder().redirect(Policy::none()).build()?;
    let res = client
        .get("https://www.linuxfromscratch.org/~thomas/multilib-m32/wget-list-sysv")
        .send()?
        .text()?;
    Ok(res)
}
