use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::pkgs::package::{OptimizationSettings, PackageDefinition};

#[derive(Debug, Clone)]
pub struct ScaffoldRequest {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub md5: Option<String>,
    pub configure_args: Vec<String>,
    pub build_commands: Vec<String>,
    pub install_commands: Vec<String>,
    pub dependencies: Vec<String>,
    pub enable_lto: bool,
    pub enable_pgo: bool,
    pub cflags: Vec<String>,
    pub ldflags: Vec<String>,
    pub profdata: Option<String>,
    pub stage: Option<String>,
    pub variant: Option<String>,
    pub notes: Option<String>,
    pub module_override: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScaffoldResult {
    pub module_path: PathBuf,
    pub prefix_module: PathBuf,
    pub by_name_module: PathBuf,
    pub definition: PackageDefinition,
}

pub fn scaffold_package(
    base_dir: impl AsRef<Path>,
    request: ScaffoldRequest,
) -> Result<ScaffoldResult> {
    let base_dir = base_dir.as_ref();
    if !base_dir.ends_with("by_name") {
        return Err(anyhow!("expected base directory ending with 'by_name'"));
    }

    let module_source_name = request.module_override.as_deref().unwrap_or(&request.name);
    let module_name = sanitize(module_source_name);
    let prefix = prefix(&module_name);

    let prefix_dir = base_dir.join(&prefix);
    fs::create_dir_all(&prefix_dir)
        .with_context(|| format!("creating prefix directory {:?}", prefix_dir))?;

    let by_name_mod = base_dir.join("mod.rs");
    ensure_mod_entry(&by_name_mod, &prefix)?;

    let prefix_mod = prefix_dir.join("mod.rs");
    ensure_mod_entry(&prefix_mod, &module_name)?;

    let package_dir = prefix_dir.join(&module_name);
    if package_dir.exists() {
        return Err(anyhow!("package module {:?} already exists", package_dir));
    }
    fs::create_dir_all(&package_dir)
        .with_context(|| format!("creating package directory {:?}", package_dir))?;

    let module_path = package_dir.join("mod.rs");
    let definition = build_definition(&request);
    let source = generate_module_source(&request, &definition);
    fs::write(&module_path, source)
        .with_context(|| format!("writing module source to {:?}", module_path))?;

    Ok(ScaffoldResult {
        module_path,
        prefix_module: prefix_mod,
        by_name_module: by_name_mod,
        definition,
    })
}

fn ensure_mod_entry(path: &Path, module: &str) -> Result<()> {
    let entry = format!("pub mod {};", module);
    if path.exists() {
        let contents =
            fs::read_to_string(path).with_context(|| format!("reading module file {:?}", path))?;
        if contents.contains(&entry) || contents.contains(&entry.trim()) {
            return Ok(());
        }
        let mut file = OpenOptions::new()
            .append(true)
            .open(path)
            .with_context(|| format!("opening module file {:?}", path))?;
        writeln!(file, "pub mod {};", module)
            .with_context(|| format!("appending to module file {:?}", path))?;
    } else {
        fs::write(path, format!("pub mod {};\n", module))
            .with_context(|| format!("creating module file {:?}", path))?;
    }
    Ok(())
}

fn build_definition(request: &ScaffoldRequest) -> PackageDefinition {
    let mut pkg = PackageDefinition::new(&request.name, &request.version);
    pkg.source = request.source.clone();
    pkg.md5 = request.md5.clone();
    pkg.configure_args = request.configure_args.clone();
    pkg.build_commands = request.build_commands.clone();
    pkg.install_commands = request.install_commands.clone();
    pkg.dependencies = request.dependencies.clone();

    let mut cflags = if request.cflags.is_empty() {
        default_cflags(request)
    } else {
        request.cflags.clone()
    };
    let mut ldflags = if request.ldflags.is_empty() {
        default_ldflags(request)
    } else {
        request.ldflags.clone()
    };
    dedup(&mut cflags);
    dedup(&mut ldflags);

    let profdata = request.profdata.clone();
    let profdata_clone = profdata.clone();
    pkg.optimizations = match profdata_clone {
        Some(path) => OptimizationSettings::for_pgo_replay(path),
        None => OptimizationSettings::default(),
    };
    pkg.optimizations.enable_lto = request.enable_lto;
    pkg.optimizations.enable_pgo = request.enable_pgo;
    pkg.optimizations.cflags = cflags;
    pkg.optimizations.ldflags = ldflags;
    pkg.optimizations.profdata = profdata;

    pkg
}

fn default_cflags(request: &ScaffoldRequest) -> Vec<String> {
    let mut flags = vec!["-O3".to_string(), "-flto".to_string()];
    if request.enable_pgo {
        if request.profdata.is_some() {
            flags.push("-fprofile-use".to_string());
        } else {
            flags.push("-fprofile-generate".to_string());
        }
    }
    flags
}

fn default_ldflags(request: &ScaffoldRequest) -> Vec<String> {
    let mut flags = vec!["-flto".to_string()];
    if request.enable_pgo {
        if request.profdata.is_some() {
            flags.push("-fprofile-use".to_string());
        } else {
            flags.push("-fprofile-generate".to_string());
        }
    }
    flags
}

fn dedup(values: &mut Vec<String>) {
    let mut seen = std::collections::BTreeSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

fn generate_module_source(request: &ScaffoldRequest, definition: &PackageDefinition) -> String {
    let mut metadata = Vec::new();
    if let Some(stage) = &request.stage {
        metadata.push(format!("stage: {}", stage));
    }
    if let Some(variant) = &request.variant {
        metadata.push(format!("variant: {}", variant));
    }
    if let Some(notes) = &request.notes {
        metadata.push(format!("notes: {}", notes));
    }
    let metadata = if metadata.is_empty() {
        String::new()
    } else {
        format!("// MLFS metadata: {}\n\n", metadata.join(", "))
    };
    let configure_args = format_vec(&definition.configure_args);
    let build_commands = format_vec(&definition.build_commands);
    let install_commands = format_vec(&definition.install_commands);
    let dependencies = format_vec(&definition.dependencies);
    let cflags = format_vec(&definition.optimizations.cflags);
    let ldflags = format_vec(&definition.optimizations.ldflags);
    let source = format_option(&definition.source);
    let md5 = format_option(&definition.md5);
    let profdata = format_option(&definition.optimizations.profdata);

    format!(
        "{metadata}use crate::pkgs::package::{{OptimizationSettings, PackageDefinition}};\n\n\
         pub fn definition() -> PackageDefinition {{\n\
            let mut pkg = PackageDefinition::new(\"{name}\", \"{version}\");\n\
            pkg.source = {source};\n\
            pkg.md5 = {md5};\n\
             pkg.configure_args = {configure_args};\n\
             pkg.build_commands = {build_commands};\n\
             pkg.install_commands = {install_commands};\n\
             pkg.dependencies = {dependencies};\n\
             let profdata = {profdata};\n\
             let profdata_clone = profdata.clone();\n\
             pkg.optimizations = match profdata_clone {{\n\
                 Some(path) => OptimizationSettings::for_pgo_replay(path),\n\
                 None => OptimizationSettings::default(),\n\
             }};\n\
             pkg.optimizations.enable_lto = {enable_lto};\n\
             pkg.optimizations.enable_pgo = {enable_pgo};\n\
             pkg.optimizations.cflags = {cflags};\n\
             pkg.optimizations.ldflags = {ldflags};\n\
             pkg.optimizations.profdata = profdata;\n\
             pkg\n\
         }}\n",
        metadata = metadata,
        name = request.name,
        version = request.version,
        source = source,
        md5 = md5,
        configure_args = configure_args,
        build_commands = build_commands,
        install_commands = install_commands,
        dependencies = dependencies,
        profdata = profdata,
        enable_lto = request.enable_lto,
        enable_pgo = request.enable_pgo,
        cflags = cflags,
        ldflags = ldflags,
    )
}

fn format_vec(values: &[String]) -> String {
    if values.is_empty() {
        "Vec::new()".to_string()
    } else {
        let items: Vec<String> = values
            .iter()
            .map(|v| format!("\"{}\".to_string()", escape(v)))
            .collect();
        format!("vec![{}]", items.join(", "))
    }
}

fn format_option(value: &Option<String>) -> String {
    match value {
        Some(v) => format!("Some(\"{}\".to_string())", escape(v)),
        None => "None".to_string(),
    }
}

fn sanitize(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == '_' || ch == '+' {
            out.push('_');
        } else if ch == '-' {
            out.push('_');
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("pkg");
    }
    if out
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        out.insert(0, 'p');
    }
    out
}

fn prefix(module: &str) -> String {
    let mut chars = module.chars();
    let first = chars.next().unwrap_or('p');
    let second = chars.next().unwrap_or('k');
    let mut s = String::new();
    s.push(first);
    s.push(second);
    s
}

fn escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}
