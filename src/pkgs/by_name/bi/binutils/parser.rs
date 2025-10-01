// Parser for Binutils Pass 1 page using lightweight HTTP fetching.
use anyhow::{Context, Result};
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct BinutilsInfo {
    /// "2.45" or derived version text
    pub version: Option<String>,
    /// first archive download URL found (.tar.xz or .tar.gz)
    pub download_url: Option<String>,
    /// tokens for configure flags (everything after ../configure)
    pub configure_args: Vec<String>,
    /// build commands discovered (e.g. ["make"])
    pub build_cmds: Vec<String>,
    /// install commands discovered (e.g. ["make install"])
    pub install_cmds: Vec<String>,
    /// optional SBU, disk space
    pub sbu: Option<String>,
    pub disk_space: Option<String>,
}

impl Default for BinutilsInfo {
    fn default() -> Self {
        Self {
            version: None,
            download_url: None,
            configure_args: Vec::new(),
            build_cmds: Vec::new(),
            install_cmds: Vec::new(),
            sbu: None,
            disk_space: None,
        }
    }
}

/// Fetch page content synchronously
pub fn fetch_page(url: &str) -> Result<String> {
    ureq::get(url)
        .call()
        .map_err(|err| match err {
            ureq::Error::Status(code, _) => anyhow::anyhow!("Failed to fetch {url}: HTTP {code}"),
            other => anyhow::anyhow!("Failed to fetch {url}: {other}"),
        })?
        .into_string()
        .with_context(|| format!("reading body from {url}"))
}

/// Parse the LFS Binutils pass1 page; robust to small formatting changes.
/// - extracts version (from <h1> text like "Binutils-2.45 - Pass 1")
/// - finds a download URL ending with .tar.xz/.tar.gz
/// - finds configure pre block(s), builds token list
/// - finds `make` / `make install` pre blocks
pub fn parse_binutils(html: &str) -> Result<BinutilsInfo> {
    let document = Html::parse_document(html);

    let mut info = BinutilsInfo::default();

    // 1) Version from h1.sect1 (contains "Binutils-2.45 - Pass 1")
    if let Ok(h1_sel) = Selector::parse("h1.sect1") {
        if let Some(h1) = document.select(&h1_sel).next() {
            let txt = h1.text().collect::<Vec<_>>().join(" ");
            // try to pick the token containing "Binutils-" or "binutils-"
            if let Some(tok) = txt
                .split_whitespace()
                .find(|s| s.to_lowercase().contains("binutils"))
            {
                // extract digits from token, e.g. "Binutils-2.45"
                if let Some(pos) = tok.find('-') {
                    let ver = tok[pos + 1..]
                        .trim()
                        .trim_matches(|c: char| !c.is_ascii() && c != '.')
                        .to_string();
                    if !ver.is_empty() {
                        info.version = Some(ver);
                    }
                } else {
                    // fallback: try to find "2.45" somewhere in the h1 string
                    for part in txt.split_whitespace() {
                        if part.chars().next().map(|c| c.is_digit(10)).unwrap_or(false) {
                            info.version = Some(part.trim().to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    // 2) Download URL: look for anchors with href ending .tar.xz/.tar.gz
    if let Ok(a_sel) = Selector::parse("a[href]") {
        for a in document.select(&a_sel) {
            if let Some(href) = a.value().attr("href") {
                let href = href.trim();
                if href.ends_with(".tar.xz") || href.ends_with(".tar.gz") || href.ends_with(".tgz")
                {
                    // Make absolute if relative to page; the typical LFS pages use relative links like ../../... or ../..
                    // If it's already absolute (starts with http), keep it.
                    let url = href.to_string();
                    info.download_url = Some(url);
                    break;
                }
            }
        }
    }

    // 3) Parse "segmentedlist" entries for SBU and disk space
    if let Ok(segtitle_sel) =
        Selector::parse("div.package .segmentedlist .seglistitem .seg strong.segtitle")
    {
        if let Ok(segbody_sel) =
            Selector::parse("div.package .segmentedlist .seglistitem .seg span.segbody")
        {
            for (t, b) in document
                .select(&segtitle_sel)
                .zip(document.select(&segbody_sel))
            {
                let title = t.text().collect::<String>().to_lowercase();
                let body = b.text().collect::<String>().trim().to_string();
                if title.contains("approximate build time") {
                    info.sbu = Some(body.clone());
                } else if title.contains("required disk space") {
                    info.disk_space = Some(body.clone());
                }
            }
        }
    }

    // 4) `pre.kbd.command` blocks for configure & make lines
    if let Ok(pre_sel) = Selector::parse("div.installation pre.kbd.command, pre.kbd.command") {
        for pre in document.select(&pre_sel) {
            let text = pre.text().collect::<Vec<_>>().join("\n");
            let trimmed = text.trim();

            // handle configure block (starts with ../configure or ./configure)
            if trimmed.starts_with("../configure")
                || trimmed.starts_with("./configure")
                || trimmed.starts_with(".. /configure")
            {
                // normalize: remove trailing backslashes and join lines
                let mut joined = String::new();
                for line in trimmed.lines() {
                    let line = line.trim_end();
                    if line.ends_with('\\') {
                        joined.push_str(line.trim_end_matches('\\').trim());
                        joined.push(' ');
                    } else {
                        joined.push_str(line.trim());
                        joined.push(' ');
                    }
                }
                // remove leading "../configure" token and split into args
                let pieces: Vec<&str> = joined.split_whitespace().collect();
                let mut args = Vec::new();
                let mut started = false;
                for p in pieces {
                    if !started {
                        if p.ends_with("configure")
                            || p.ends_with("configure")
                            || p.contains("configure")
                        {
                            started = true;
                            continue;
                        }
                        // skip until configure found
                        continue;
                    } else {
                        args.push(p.to_string());
                    }
                }
                // fallback: if no tokens parsed, try chopping first token
                if args.is_empty() {
                    // attempt to remove the first token (../configure) by index
                    if let Some(pos) = joined.find("configure") {
                        let after = &joined[pos + "configure".len()..];
                        for t in after.split_whitespace() {
                            args.push(t.to_string());
                        }
                    }
                }
                info.configure_args = args
                    .into_iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                continue;
            }

            // handle make / make install lines
            // consider blocks that are exactly "make" or "make install" or lines containing them
            for line in trimmed.lines().map(|l| l.trim()) {
                if line == "make" {
                    if !info.build_cmds.contains(&"make".to_string()) {
                        info.build_cmds.push("make".to_string());
                    }
                } else if line == "make install" {
                    if !info.install_cmds.contains(&"make install".to_string()) {
                        info.install_cmds.push("make install".to_string());
                    }
                } else if line.starts_with("make ") {
                    // e.g., "make -j2"
                    let t = line.to_string();
                    if !info.build_cmds.contains(&t) {
                        info.build_cmds.push(t);
                    }
                } else if line.starts_with("time {") && line.contains("make") {
                    // handle the time wrapper line in the note; ignore
                    // skip
                }
            }
        }
    }

    // final sanity: if build_cmds empty but install_cmds contains "make install", add "make"
    if info.build_cmds.is_empty() && !info.install_cmds.is_empty() {
        info.build_cmds.push("make".to_string());
    }

    Ok(info)
}
