use anyhow::{Context, Result};

pub fn get_md5sums() -> Result<String> {
    let agent = ureq::AgentBuilder::new().redirects(5).build();
    let url = "https://www.linuxfromscratch.org/~thomas/multilib-m32/md5sums";

    let response = agent.get(url).call().map_err(|err| match err {
        ureq::Error::Status(code, _) => anyhow::anyhow!("Failed to fetch MD5sums: HTTP {code}"),
        other => anyhow::anyhow!("Failed to fetch MD5sums: {other}"),
    })?;

    response
        .into_string()
        .with_context(|| format!("reading body from {url}"))
}
