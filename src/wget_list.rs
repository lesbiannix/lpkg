use anyhow::{Context, Result};

pub fn get_wget_list() -> Result<String> {
    let url = "https://www.linuxfromscratch.org/~thomas/multilib-m32/wget-list-sysv";
    let agent = ureq::AgentBuilder::new().redirects(5).build();
    agent
        .get(url)
        .call()
        .map_err(|err| match err {
            ureq::Error::Status(code, _) => {
                anyhow::anyhow!("Failed to fetch wget-list: HTTP {code}")
            }
            other => anyhow::anyhow!("Failed to fetch wget-list: {other}"),
        })?
        .into_string()
        .with_context(|| format!("reading body from {url}"))
}
