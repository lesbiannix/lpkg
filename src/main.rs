use std::{collections::BTreeSet, env, fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{CommandFactory, Parser, Subcommand};

use package_management::{
    db, html, md5_utils,
    pkgs::{
        by_name::bi::binutils::cross_toolchain::build_binutils_from_page,
        generator, mlfs,
        scaffolder::{self, ScaffoldRequest},
    },
    version_check, wget_list,
};

#[cfg(feature = "tui")]
use package_management::tui::disk_manager::DiskManager;

#[derive(Parser)]
#[command(name = "lpkg", version, about = "LPKG – Lightweight Package Manager", long_about = None)]
struct Cli {
    /// Command to run. Defaults to launching the TUI (when available).
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run one of the automated workflows.
    Workflow {
        #[command(subcommand)]
        workflow: WorkflowCommand,
    },
    /// Launch interactive terminal UIs.
    #[cfg(feature = "tui")]
    #[command(subcommand)]
    Tui(TuiCommand),
}

#[derive(Subcommand)]
enum WorkflowCommand {
    /// Fetch <pre> blocks from the given URL and run version checks found inside them.
    EnvCheck {
        /// URL of the Linux From Scratch page containing ver_check/ver_kernel snippets.
        url: String,
    },
    /// Download the LFS wget-list and md5sums, optionally writing them to disk.
    FetchManifests {
        /// Output directory to store wget-list and md5sums files. Uses current dir if omitted.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Parse the Binutils Pass 1 page and build it using the extracted steps.
    BuildBinutils {
        /// URL of the Binutils Pass 1 instructions to parse.
        url: String,
        /// Root directory of the LFS workspace (used for $LFS paths).
        #[arg(long = "lfs-root")]
        lfs_root: PathBuf,
        /// Optional explicit cross-compilation target (defaults to $LFS_TGT env or x86_64-lfs-linux-gnu).
        #[arg(long)]
        target: Option<String>,
    },
    /// Scaffold a new package module under `src/pkgs/by_name` with tuned optimizations.
    ScaffoldPackage {
        /// Logical package name (used for module layout and metadata).
        #[arg(long)]
        name: String,
        /// Upstream version string.
        #[arg(long)]
        version: String,
        /// Optional source archive URL.
        #[arg(long)]
        source: Option<String>,
        /// Optional MD5 checksum of the source archive.
        #[arg(long)]
        md5: Option<String>,
        /// Additional configure arguments (repeat flag).
        #[arg(long = "configure-arg", value_name = "ARG")]
        configure_arg: Vec<String>,
        /// Build commands (repeat flag).
        #[arg(long = "build-cmd", value_name = "CMD")]
        build_cmd: Vec<String>,
        /// Install commands (repeat flag).
        #[arg(long = "install-cmd", value_name = "CMD")]
        install_cmd: Vec<String>,
        /// Declared dependencies (repeat flag).
        #[arg(long = "dependency", value_name = "PKG")]
        dependency: Vec<String>,
        /// Whether to enable LTO (defaults to true).
        #[arg(long = "enable-lto", default_value_t = true)]
        enable_lto: bool,
        /// Whether to enable PGO instrumentation/use (defaults to true).
        #[arg(long = "enable-pgo", default_value_t = true)]
        enable_pgo: bool,
        /// Additional CFLAGS (repeat flag).
        #[arg(long = "cflag", value_name = "FLAG")]
        cflag: Vec<String>,
        /// Additional LDFLAGS (repeat flag).
        #[arg(long = "ldflag", value_name = "FLAG")]
        ldflag: Vec<String>,
        /// Optional profile data file name for PGO replay (enables -fprofile-use).
        #[arg(long)]
        profdata: Option<String>,
        /// Base directory for module generation (defaults to src/pkgs/by_name).
        #[arg(long, default_value = "src/pkgs/by_name")]
        base: PathBuf,
    },
    /// Import all packages from the MLFS catalogue, scaffolding modules and persisting metadata.
    ImportMlfs {
        /// Perform a dry run without writing files or touching the database.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Only process the first N records (after deduplication).
        #[arg(long)]
        limit: Option<usize>,
        /// Base directory for module generation (defaults to src/pkgs/by_name).
        #[arg(long, default_value = "src/pkgs/by_name")]
        base: PathBuf,
        /// Overwrite existing modules by deleting and regenerating them.
        #[arg(long, default_value_t = false)]
        overwrite: bool,
        /// Source URL for the MLFS book (defaults to the canonical mirror).
        #[arg(long = "source-url")]
        source_url: Option<String>,
    },
}

#[cfg(feature = "tui")]
#[derive(Subcommand)]
enum TuiCommand {
    /// Launch the disk manager UI.
    DiskManager,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Workflow { workflow }) => run_workflow(workflow)?,
        #[cfg(feature = "tui")]
        Some(Command::Tui(cmd)) => run_tui(cmd)?,
        None => {
            #[cfg(feature = "tui")]
            {
                println!(
                    "No command specified. Launching disk manager TUI. Use `lpkg help` for more options."
                );
                DiskManager::run_tui().map_err(|e| anyhow!(e.to_string()))?;
            }

            #[cfg(not(feature = "tui"))]
            {
                Cli::command().print_help()?;
                println!();
            }
        }
    }

    Ok(())
}

fn run_workflow(cmd: WorkflowCommand) -> Result<()> {
    match cmd {
        WorkflowCommand::EnvCheck { url } => {
            let pre_blocks = html::fetch_pre_blocks(&url)
                .with_context(|| format!("Fetching HTML `<pre>` blocks from {url}"))?;

            let mut ran_any = false;
            let mut failures = Vec::new();

            for (idx, block) in pre_blocks.iter().enumerate() {
                if !(block.contains("ver_check") || block.contains("ver_kernel")) {
                    continue;
                }

                ran_any = true;
                println!("Running version checks from block #{idx}...");
                if !version_check::run_version_checks_from_block(block) {
                    failures.push(idx + 1);
                }
            }

            if !ran_any {
                return Err(anyhow!(
                    "No ver_check or ver_kernel snippets found at {url}."
                ));
            }

            if !failures.is_empty() {
                return Err(anyhow!("Version checks failed in block(s): {:?}", failures));
            }

            println!("All version checks passed 👍");
        }
        WorkflowCommand::FetchManifests { output } => {
            let wget_list = wget_list::get_wget_list().context("Fetching wget-list")?;
            let md5sums = md5_utils::get_md5sums().context("Fetching md5sums")?;

            println!("Fetched wget-list ({} bytes)", wget_list.len());
            println!("Fetched md5sums ({} bytes)", md5sums.len());

            let target_dir = output.unwrap_or(std::env::current_dir()?);
            fs::create_dir_all(&target_dir)
                .with_context(|| format!("Creating output directory at {:?}", target_dir))?;

            let wget_path = target_dir.join("wget-list");
            let md5_path = target_dir.join("md5sums");

            fs::write(&wget_path, wget_list).with_context(|| format!("Writing {wget_path:?}"))?;
            fs::write(&md5_path, md5sums).with_context(|| format!("Writing {md5_path:?}"))?;

            println!("Saved artifacts to {:?} and {:?}", wget_path, md5_path);
        }
        WorkflowCommand::BuildBinutils {
            url,
            lfs_root,
            target,
        } => {
            build_binutils_from_page(&url, &lfs_root, target)
                .with_context(|| format!("Building Binutils using instructions from {url}"))?;

            println!("Binutils workflow completed successfully");
        }
        WorkflowCommand::ScaffoldPackage {
            name,
            version,
            source,
            md5,
            configure_arg,
            build_cmd,
            install_cmd,
            dependency,
            enable_lto,
            enable_pgo,
            cflag,
            ldflag,
            profdata,
            base,
        } => {
            let base_dir = if base.is_relative() {
                env::current_dir()
                    .context("Resolving scaffold base directory")?
                    .join(base)
            } else {
                base
            };

            let request = ScaffoldRequest {
                name: name.clone(),
                version: version.clone(),
                source,
                md5,
                configure_args: configure_arg,
                build_commands: build_cmd,
                install_commands: install_cmd,
                dependencies: dependency,
                enable_lto,
                enable_pgo,
                cflags: cflag,
                ldflags: ldflag,
                profdata,
                stage: None,
                variant: None,
                notes: None,
                module_override: None,
            };

            let scaffold = scaffolder::scaffold_package(&base_dir, request)
                .with_context(|| format!("Scaffolding package {name}"))?;

            let pool = db::establish_pool().context("Setting up package database")?;
            db::upsert_package_via_pool(&pool, &scaffold.definition)
                .with_context(|| format!("Persisting package metadata for {name}"))?;

            println!("Generated module: {:?}", scaffold.module_path);
            println!(
                "Remember to stage and commit as `{name}: init at {version}` after reviewing the template"
            );
        }
        WorkflowCommand::ImportMlfs {
            dry_run,
            limit,
            base,
            overwrite,
            source_url,
        } => {
            let base_dir = if base.is_relative() {
                env::current_dir()
                    .context("Resolving MLFS scaffold base directory")?
                    .join(base)
            } else {
                base
            };

            let mut records = mlfs::load_or_fetch_catalog(source_url.as_deref())
                .context("Loading MLFS catalogue")?;
            records.sort_by(|a, b| a.name.cmp(&b.name).then(a.variant.cmp(&b.variant)));

            let mut seen = BTreeSet::new();
            let mut processed = 0usize;
            let mut created = 0usize;
            let mut skipped = Vec::new();

            let metadata_entries = match mlfs::load_metadata_index() {
                Ok(entries) => Some(entries),
                Err(err) => {
                    eprintln!("[mlfs] metadata index error: {err}");
                    None
                }
            };

            let pool = if dry_run {
                None
            } else {
                Some(db::establish_pool().context("Setting up package database")?)
            };

            for record in records {
                if let Some(limit) = limit {
                    if processed >= limit {
                        break;
                    }
                }
                processed += 1;

                let metadata_entry = metadata_entries
                    .as_ref()
                    .and_then(|entries| mlfs::match_metadata(&record, entries));

                let mut request = if let Some(entry) = metadata_entry {
                    let path = PathBuf::from("ai/metadata").join(&entry.path);
                    match generator::request_from_metadata(&path) {
                        Ok(req) => req,
                        Err(err) => {
                            eprintln!(
                                "[mlfs] metadata apply error for {} {}: {}",
                                record.name, record.version, err
                            );
                            ScaffoldRequest {
                                name: record.name.clone(),
                                version: record.version.clone(),
                                source: None,
                                md5: None,
                                configure_args: Vec::new(),
                                build_commands: Vec::new(),
                                install_commands: Vec::new(),
                                dependencies: Vec::new(),
                                enable_lto: true,
                                enable_pgo: true,
                                cflags: Vec::new(),
                                ldflags: Vec::new(),
                                profdata: None,
                                stage: record.stage.clone(),
                                variant: record.variant.clone(),
                                notes: record.notes.clone(),
                                module_override: None,
                            }
                        }
                    }
                } else {
                    ScaffoldRequest {
                        name: record.name.clone(),
                        version: record.version.clone(),
                        source: None,
                        md5: None,
                        configure_args: Vec::new(),
                        build_commands: Vec::new(),
                        install_commands: Vec::new(),
                        dependencies: Vec::new(),
                        enable_lto: true,
                        enable_pgo: true,
                        cflags: Vec::new(),
                        ldflags: Vec::new(),
                        profdata: None,
                        stage: record.stage.clone(),
                        variant: record.variant.clone(),
                        notes: record.notes.clone(),
                        module_override: None,
                    }
                };

                if request.stage.is_none() {
                    request.stage = record.stage.clone();
                }
                if request.variant.is_none() {
                    request.variant = record.variant.clone();
                }
                if request.notes.is_none() {
                    request.notes = record.notes.clone();
                }

                let module_alias = request
                    .module_override
                    .clone()
                    .unwrap_or_else(|| record.module_alias());

                if !seen.insert(module_alias.clone()) {
                    continue;
                }

                if request.module_override.is_none() {
                    request.module_override = Some(module_alias.clone());
                }

                if dry_run {
                    println!(
                        "Would scaffold {:<18} {:<12} -> {}",
                        record.name, record.version, module_alias
                    );
                    continue;
                }

                match scaffolder::scaffold_package(&base_dir, request) {
                    Ok(result) => {
                        if let Some(pool) = &pool {
                            db::upsert_package_via_pool(pool, &result.definition).with_context(
                                || {
                                    format!(
                                        "Persisting MLFS package metadata for {} {}",
                                        record.name, record.version
                                    )
                                },
                            )?;
                        }
                        println!(
                            "Scaffolded {:<18} {:<12} -> {}",
                            record.name, record.version, module_alias
                        );
                        created += 1;
                    }
                    Err(err) => {
                        let already_exists =
                            err.to_string().to_lowercase().contains("already exists");
                        if already_exists && !overwrite {
                            skipped.push(module_alias);
                        } else {
                            return Err(err);
                        }
                    }
                }
            }

            if dry_run {
                println!(
                    "Dry run complete. {} package definitions queued.",
                    processed
                );
            } else {
                println!(
                    "MLFS import complete. Created {} modules, skipped {} (already existed).",
                    created,
                    skipped.len()
                );
                if !skipped.is_empty() {
                    println!(
                        "Skipped modules: {}",
                        skipped
                            .iter()
                            .take(10)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    if skipped.len() > 10 {
                        println!("... and {} more", skipped.len() - 10);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "tui")]
fn run_tui(cmd: TuiCommand) -> Result<()> {
    match cmd {
        TuiCommand::DiskManager => {
            DiskManager::run_tui().map_err(|e| anyhow!(e.to_string()))?;
        }
    }

    Ok(())
}
