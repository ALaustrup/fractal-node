//! `cargo xtask` — the task runner.
//!
//! Rust rather than Make or shell for one reason: these tasks encode Canon, and
//! Canon has to run identically on Windows, macOS and Linux for every agent and
//! every CI job. A shell script would not (`docs/41 §11`).

#![allow(clippy::print_stdout, clippy::print_stderr)]

mod lint_deps;
mod parity;
mod tokens;

use anyhow::{bail, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map_or("help", String::as_str);
    let check = args.iter().any(|a| a == "--check");

    match cmd {
        "lint-deps" => lint_deps::run(),
        "tokens" => tokens::run(check),
        "parity" => parity::run(),
        "verify" => verify(),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => {
            print_help();
            bail!("unknown task `{other}`")
        }
    }
}

fn print_help() {
    println!("cargo xtask <task>");
    println!();
    println!("  lint-deps        Enforce layers.toml: dependency direction and vendor bans (P5)");
    println!("  tokens [--check] Generate the design tokens for every target; --check fails on drift (N7)");
    println!("  parity           Every API operation must have a CLI command (P13)");
    println!("  verify           Everything a commit must pass, in the order CI runs it");
}

/// The gate. `docs/42`: an agent runs this before composing a commit, and a red
/// result is never committed — not even with a note.
fn verify() -> Result<()> {
    step("format", &["fmt", "--all", "--check"])?;
    step(
        "clippy",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    step("test", &["test", "--workspace"])?;
    println!("\n== canon ==");
    lint_deps::run()?;
    tokens::run(true)?;
    parity::run()?;
    println!("\n\u{1b}[38;2;140;232;223m⌁\u{1b}[0m verify: all gates green");
    Ok(())
}

fn step(name: &str, args: &[&str]) -> Result<()> {
    println!("\n== {name} ==");
    let status = std::process::Command::new(env!("CARGO"))
        .args(args)
        .status()?;
    if !status.success() {
        bail!("{name} failed");
    }
    Ok(())
}

/// The workspace root, found by walking up from this crate.
pub(crate) fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        )
}
