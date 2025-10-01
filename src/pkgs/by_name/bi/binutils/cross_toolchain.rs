// Cross-toolchain runner that uses parser.rs info (no hardcoding).
use crate::pkgs::by_name::bi::binutils::parser::{BinutilsInfo, fetch_page, parse_binutils};
use anyhow::{Context, Result, anyhow};
use shell_words;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::Command,
};

/// Configuration object - uses environment if values omitted.
#[derive(Debug, Clone)]
pub struct BinutilsConfig {
    pub lfs_root: PathBuf,  // where the LFS tree will be (used for $LFS)
    pub target: String,     // LFS_TGT (e.g. x86_64-lfs-linux-gnu)
    pub info: BinutilsInfo, // parsed page info
}

impl BinutilsConfig {
    /// create from env or params. If target is None, tries $LFS_TGT env var.
    pub fn new(lfs_root: impl AsRef<Path>, target: Option<String>, info: BinutilsInfo) -> Self {
        let lfs_root = lfs_root.as_ref().to_path_buf();
        let target = target
            .or_else(|| std::env::var("LFS_TGT").ok())
            .unwrap_or_else(|| "x86_64-lfs-linux-gnu".to_string());

        Self {
            lfs_root,
            target,
            info,
        }
    }

    /// default places (non-hardcoded) where sources live.
    /// If env `BINUTILS_SRC_DIR` is set, use that; else try LFS layout:
    /// - $LFS/src/pkgs/by-name/bi/binutils
    pub fn source_base_dir(&self) -> PathBuf {
        if let Ok(s) = std::env::var("BINUTILS_SRC_DIR") {
            PathBuf::from(s)
        } else {
            self.lfs_root
                .join("src")
                .join("pkgs")
                .join("by-name")
                .join("bi")
                .join("binutils")
        }
    }

    /// build directory inside LFS tree (following LFS style)
    pub fn build_dir(&self) -> PathBuf {
        self.lfs_root.join("build").join("binutils-pass1")
    }

    /// install dir (tools)
    pub fn install_dir(&self) -> PathBuf {
        self.lfs_root.join("tools")
    }
}

/// High-level orchestration.
pub fn build_binutils_from_page(
    page_url: &str,
    lfs_root: impl AsRef<Path>,
    target: Option<String>,
) -> Result<()> {
    println!("Fetching page: {page_url}");
    let html = fetch_page(page_url).context("fetching binutils instructions")?;
    let info = parse_binutils(&html).context("parsing binutils instructions")?;
    println!("Parsed info: {:?}", info);

    let cfg = BinutilsConfig::new(lfs_root, target, info.clone());

    let src_base = cfg.source_base_dir();
    if !src_base.exists() {
        println!("Creating source base dir: {:?}", src_base);
        fs::create_dir_all(&src_base)
            .with_context(|| format!("creating source base dir {:?}", src_base))?;
    }

    let mut source_dir = locate_binutils_dir(&src_base)?;
    if source_dir.is_none() {
        source_dir = download_and_extract(&cfg, &src_base)?;
    }

    let source_dir = source_dir
        .ok_or_else(|| anyhow!("Could not locate or download/extract Binutils source"))?;
    println!("Using source dir: {:?}", source_dir);

    let build_dir = cfg.build_dir();
    if !build_dir.exists() {
        println!("Creating build dir {:?}", build_dir);
        fs::create_dir_all(&build_dir)
            .with_context(|| format!("creating build dir {:?}", build_dir))?;
    }

    let configure_path = source_dir.join("configure");
    if !configure_path.exists() {
        return Err(anyhow!(
            "configure script not found at {:?}",
            configure_path
        ));
    }

    let args = if !cfg.info.configure_args.is_empty() {
        cfg.info.configure_args.clone()
    } else {
        vec![
            format!("--prefix={}", cfg.install_dir().display()),
            format!("--with-sysroot={}", cfg.lfs_root.display()),
            format!("--target={}", cfg.target),
            "--disable-nls".to_string(),
            "--disable-werror".to_string(),
        ]
    };

    let args: Vec<String> = args
        .into_iter()
        .map(|a| {
            a.replace("$LFS", &cfg.lfs_root.to_string_lossy())
                .replace("$LFS_TGT", &cfg.target)
        })
        .collect();

    println!("Configuring with args: {:?}", args);
    let mut configure_cmd = Command::new(&configure_path);
    configure_cmd.current_dir(&build_dir);
    configure_cmd.args(&args);
    run_command(&mut configure_cmd).context("configure step failed")?;
    println!("configure completed");

    if !cfg.info.build_cmds.is_empty() {
        for raw in &cfg.info.build_cmds {
            run_shell_command(raw, &build_dir)
                .with_context(|| format!("build step failed: {raw}"))?;
        }
    } else {
        let mut make_cmd = Command::new("make");
        make_cmd.current_dir(&build_dir);
        run_command(&mut make_cmd).context("make failed")?;
    }
    println!("build completed");

    if !cfg.info.install_cmds.is_empty() {
        for raw in &cfg.info.install_cmds {
            run_shell_command(raw, &build_dir)
                .with_context(|| format!("install step failed: {raw}"))?;
        }
    } else {
        let mut install_cmd = Command::new("make");
        install_cmd.arg("install");
        install_cmd.current_dir(&build_dir);
        run_command(&mut install_cmd).context("make install failed")?;
    }
    println!("install completed");

    Ok(())
}

fn locate_binutils_dir(base: &Path) -> Result<Option<PathBuf>> {
    if !base.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(base).with_context(|| format!("reading directory {:?}", base))? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains("binutils") {
                return Ok(Some(entry.path()));
            }
        }
    }
    Ok(None)
}

fn download_and_extract(cfg: &BinutilsConfig, base: &Path) -> Result<Option<PathBuf>> {
    let url = match cfg.info.download_url.as_deref() {
        Some(url) => url,
        None => {
            eprintln!("No download URL found on the page and no unpacked source present.");
            return Ok(None);
        }
    };

    println!("Downloading {url}");
    let response = ureq::get(url).call().map_err(|err| match err {
        ureq::Error::Status(code, _) => anyhow!("Download failed: HTTP {code}"),
        other => anyhow!("Download failed: {other}"),
    })?;

    let final_url = response.get_url().to_string();
    let parsed = url::Url::parse(&final_url)
        .with_context(|| format!("parsing final download URL {final_url}"))?;
    let filename = parsed
        .path_segments()
        .and_then(|segments| segments.last())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Cannot determine filename from URL"))?;

    let outpath = base.join(&filename);
    let mut reader = response.into_reader();
    let mut file =
        File::create(&outpath).with_context(|| format!("creating archive file {:?}", outpath))?;
    io::copy(&mut reader, &mut file)
        .with_context(|| format!("writing archive to {:?}", outpath))?;

    println!("Extracting archive {:?}", outpath);
    let status = Command::new("tar")
        .arg("-xf")
        .arg(&outpath)
        .arg("-C")
        .arg(base)
        .status()
        .with_context(|| "spawning tar".to_string())?;
    if !status.success() {
        return Err(anyhow!("tar extraction failed"));
    }

    locate_binutils_dir(base)
}

fn run_command(cmd: &mut Command) -> Result<()> {
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    let status = cmd
        .status()
        .with_context(|| "spawning process".to_string())?;
    if !status.success() {
        return Err(anyhow!("command exited with status {status}"));
    }
    Ok(())
}

fn run_shell_command(raw: &str, cwd: &Path) -> Result<()> {
    let mut parts = shell_words::split(raw).unwrap_or_else(|_| vec![raw.to_string()]);
    if parts.is_empty() {
        return Ok(());
    }
    let prog = parts.remove(0);
    let mut cmd = Command::new(prog);
    if !parts.is_empty() {
        cmd.args(parts);
    }
    cmd.current_dir(cwd);
    run_command(&mut cmd)
}
