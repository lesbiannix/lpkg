// async cross-toolchain runner that uses parser.rs info (no hardcoding)
use crate::pkgs::by_name::bi::binutils::parser::{BinutilsInfo, fetch_page, parse_binutils};
use reqwest::Client;
use std::{
    error::Error,
    ffi::OsStr,
    path::{Path, PathBuf},
};
use tokio::process::Command;
use tracing::{error, info, warn};

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
            .unwrap_or_else(|| {
                // fallback best-effort
                if cfg!(target_os = "linux") {
                    "x86_64-lfs-linux-gnu".to_string()
                } else {
                    "x86_64-lfs-linux-gnu".to_string()
                }
            });

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

/// High-level orchestration. Async.
pub async fn build_binutils_from_page(
    page_url: &str,
    lfs_root: impl AsRef<std::path::Path>,
    target: Option<String>,
) -> Result<(), Box<dyn Error>> {
    // 1) fetch page
    info!("Fetching page: {}", page_url);
    let html = fetch_page(page_url).await?;
    let info = parse_binutils(&html)?;
    info!("Parsed info: {:?}", info);

    // 2) build config
    let cfg = BinutilsConfig::new(lfs_root, target, info.clone());

    // 3) ensure source base dir exists
    let src_base = cfg.source_base_dir();
    if !src_base.exists() {
        info!("Creating source base dir: {:?}", src_base);
        tokio::fs::create_dir_all(&src_base).await?;
    }

    // 4) find extracted source directory (binutils-*)
    let mut source_dir: Option<PathBuf> = None;
    if let Ok(mut rd) = tokio::fs::read_dir(&src_base).await {
        while let Some(entry) = rd.next_entry().await? {
            let ft = entry.file_type().await?;
            if ft.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.to_lowercase().contains("binutils") {
                    source_dir = Some(entry.path());
                    break;
                }
            }
        }
    }

    // 5) if not found, attempt to download & extract
    if source_dir.is_none() {
        if let Some(dl) = &cfg.info.download_url {
            info!("No extracted source found; will download {}", dl);

            // download file into src_base
            let client = Client::new();
            let resp = client.get(dl).send().await?;
            if !resp.status().is_success() {
                return Err(format!("Download failed: {}", resp.status()).into());
            }

            // pick a filename from URL
            let url_path = url::Url::parse(dl)?;
            let filename = url_path
                .path_segments()
                .and_then(|seg| seg.last())
                .and_then(|s| {
                    if !s.is_empty() {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .ok_or("Cannot determine filename from URL")?;

            let outpath = src_base.join(&filename);
            info!("Saving archive to {:?}", outpath);
            let bytes = resp.bytes().await?;
            tokio::fs::write(&outpath, &bytes).await?;

            // extract using tar (async spawn). Use absolute path to src_base
            info!("Extracting archive {:?}", outpath);
            let tar_path = outpath.clone();
            let mut tar_cmd = Command::new("tar");
            tar_cmd.arg("-xf").arg(&tar_path).arg("-C").arg(&src_base);
            let status = tar_cmd.status().await?;
            if !status.success() {
                return Err("tar extraction failed".into());
            }

            // look for extracted dir again
            if let Ok(mut rd) = tokio::fs::read_dir(&src_base).await {
                while let Some(entry) = rd.next_entry().await? {
                    let ft = entry.file_type().await?;
                    if ft.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.to_lowercase().contains("binutils") {
                            source_dir = Some(entry.path());
                            break;
                        }
                    }
                }
            }
        } else {
            warn!("No download URL found on the page and no unpacked source present.");
        }
    }

    let source_dir = match source_dir {
        Some(p) => p,
        None => return Err("Could not locate or download/extract Binutils source".into()),
    };
    info!("Using source dir: {:?}", source_dir);

    // 6) prepare build dir
    let build_dir = cfg.build_dir();
    if !build_dir.exists() {
        info!("Creating build dir {:?}", build_dir);
        tokio::fs::create_dir_all(&build_dir).await?;
    }

    // 7) run configure: use absolute configure script path in source_dir
    let configure_path = source_dir.join("configure");
    if !configure_path.exists() {
        return Err(format!("configure script not found at {:?}", configure_path).into());
    }

    // If parser produced configure args tokens, use them; otherwise fallback to common flags
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

    // replace $LFS and $LFS_TGT in args
    let args: Vec<String> = args
        .into_iter()
        .map(|a| {
            a.replace("$LFS", &cfg.lfs_root.to_string_lossy())
                .replace("$LFS_TGT", &cfg.target)
        })
        .collect();

    info!("Configuring with args: {:?}", args);

    // spawn configure
    let mut conf_cmd = Command::new(&configure_path);
    conf_cmd.current_dir(&build_dir);
    for a in &args {
        conf_cmd.arg(a);
    }
    conf_cmd.stdout(std::process::Stdio::inherit());
    conf_cmd.stderr(std::process::Stdio::inherit());
    let status = conf_cmd.status().await?;
    if !status.success() {
        return Err("configure step failed".into());
    }
    info!("configure completed");

    // 8) run build commands (make-like)
    if !cfg.info.build_cmds.is_empty() {
        for b in &cfg.info.build_cmds {
            // split into program + args
            let mut parts = shell_words::split(b).unwrap_or_else(|_| vec![b.clone()]);
            let prog = parts.remove(0);
            let mut cmd = Command::new(prog);
            if !parts.is_empty() {
                cmd.args(parts);
            }
            cmd.current_dir(&build_dir);
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());
            let status = cmd.status().await?;
            if !status.success() {
                return Err(format!("build step failed: {:?}", b).into());
            }
        }
    } else {
        // fallback to running `make`
        let mut m = Command::new("make");
        m.current_dir(&build_dir);
        m.stdout(std::process::Stdio::inherit());
        m.stderr(std::process::Stdio::inherit());
        let status = m.status().await?;
        if !status.success() {
            return Err("make failed".into());
        }
    }
    info!("build completed");

    // 9) run install commands (make install)
    if !cfg.info.install_cmds.is_empty() {
        for inst in &cfg.info.install_cmds {
            let mut parts = shell_words::split(inst).unwrap_or_else(|_| vec![inst.clone()]);
            let prog = parts.remove(0);
            let mut cmd = Command::new(prog);
            if !parts.is_empty() {
                cmd.args(parts);
            }
            cmd.current_dir(&build_dir);
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());
            let status = cmd.status().await?;
            if !status.success() {
                return Err(format!("install step failed: {:?}", inst).into());
            }
        }
    } else {
        // fallback `make install`
        let mut mi = Command::new("make");
        mi.arg("install");
        mi.current_dir(&build_dir);
        mi.stdout(std::process::Stdio::inherit());
        mi.stderr(std::process::Stdio::inherit());
        let status = mi.status().await?;
        if !status.success() {
            return Err("make install failed".into());
        }
    }

    info!("install completed. Binutils Pass 1 done.");
    Ok(())
}
