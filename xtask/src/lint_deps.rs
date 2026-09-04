//! The dependency-direction lint (P5, `docs/41 §7`).
//!
//! This is the mechanism that makes the layering real. Without it, "the domain
//! must not depend on adapters" is a sentence in a document that a tired
//! engineer at 6pm will violate with a one-line import and a good reason.

use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(serde::Deserialize)]
struct Contract {
    #[serde(rename = "layer")]
    layers: Vec<Layer>,
    bans: Bans,
}

#[derive(serde::Deserialize)]
struct Layer {
    name: String,
    paths: Vec<String>,
    allows: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Bans {
    domain: Vec<String>,
    #[serde(default)]
    messages: BTreeMap<String, String>,
}

pub(crate) fn run() -> Result<()> {
    let root = crate::root();
    let text = std::fs::read_to_string(root.join("layers.toml"))?;
    let contract: Contract = toml::from_str(&text)?;

    // crate name -> (layer, manifest dir)
    let mut crates: BTreeMap<String, (String, std::path::PathBuf)> = BTreeMap::new();
    for layer in &contract.layers {
        for glob in &layer.paths {
            for dir in expand(&root, glob) {
                if let Some(name) = crate_name(&dir)? {
                    crates.insert(name, (layer.name.clone(), dir));
                }
            }
        }
    }

    let allows: BTreeMap<&str, &Vec<String>> = contract
        .layers
        .iter()
        .map(|l| (l.name.as_str(), &l.allows))
        .collect();

    let mut violations: Vec<String> = Vec::new();

    for (name, (layer, dir)) in &crates {
        let manifest = runtime_dependencies(&std::fs::read_to_string(dir.join("Cargo.toml"))?);
        // Internal edges.
        for (dep, (dep_layer, _)) in &crates {
            if dep == name {
                continue;
            }
            if !mentions_dependency(&manifest, dep) {
                continue;
            }
            let permitted = allows
                .get(layer.as_str())
                .is_some_and(|a| a.contains(dep_layer));
            if !permitted {
                violations.push(format!(
                    "{name} ({layer}) depends on {dep} ({dep_layer}) — {layer} may only depend on {:?}",
                    allows.get(layer.as_str()).map_or(&Vec::new(), |v| *v)
                ));
            }
        }
        // Vendor bans in the domain layer, checked in source rather than the
        // manifest: a banned call can arrive transitively through a re-export.
        if layer == "domain" {
            for src in rust_files(&dir.join("src")) {
                let body = std::fs::read_to_string(&src).unwrap_or_default();
                for banned in &contract.bans.domain {
                    if body.contains(banned.as_str()) {
                        let why = contract
                            .bans
                            .messages
                            .get(banned)
                            .map_or("banned in the domain layer", String::as_str);
                        violations.push(format!(
                            "{}: `{banned}` — {why}",
                            src.strip_prefix(&root).unwrap_or(&src).display()
                        ));
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        println!("lint-deps: {} crates, 0 violations", crates.len());
        return Ok(());
    }
    for v in &violations {
        eprintln!("  ✕ {v}");
    }
    bail!("{} dependency-direction violation(s)", violations.len())
}

/// Return only the `[dependencies]` section. Dev and build dependencies are
/// exempt: a test that compares two adapters has to see both of them, and that
/// is a feature of the layering rather than a hole in it.
fn runtime_dependencies(manifest: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in manifest.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            inside = t == "[dependencies]";
            continue;
        }
        if inside {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn mentions_dependency(manifest: &str, dep: &str) -> bool {
    manifest.lines().any(|l| {
        let l = l.trim();
        l.starts_with(dep)
            && l.get(dep.len()..)
                .is_some_and(|r| r.trim_start().starts_with(['.', '=']))
    })
}

fn expand(root: &Path, glob: &str) -> Vec<std::path::PathBuf> {
    if let Some(parent) = glob.strip_suffix("/*") {
        return std::fs::read_dir(root.join(parent))
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.join("Cargo.toml").exists())
            .collect();
    }
    let p = root.join(glob);
    if p.join("Cargo.toml").exists() {
        vec![p]
    } else {
        Vec::new()
    }
}

fn crate_name(dir: &Path) -> Result<Option<String>> {
    let manifest = std::fs::read_to_string(dir.join("Cargo.toml"))?;
    Ok(manifest
        .lines()
        .find_map(|l| l.strip_prefix("name = "))
        .map(|v| v.trim().trim_matches('"').to_owned()))
}

fn rust_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(rust_files(&p));
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    out
}
