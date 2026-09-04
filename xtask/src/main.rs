//! `cargo xtask` — the task runner.
//!
//! Rust rather than Make or shell for one reason: these tasks encode Canon, and
//! Canon has to run identically on Windows, macOS and Linux for every agent and
//! every CI job. A shell script would not (`docs/41 §11`).

#![allow(clippy::print_stdout, clippy::print_stderr)]

mod codegen;
mod lint_deps;
mod offline;
mod parity;
mod tokens;

use anyhow::{bail, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map_or("help", String::as_str);
    let check = args.iter().any(|a| a == "--check");

    match cmd {
        "codegen" => codegen::run(check),
        "lint-deps" => lint_deps::run(),
        "tokens" => tokens::run(check),
        "offline" => offline::run(),
        "parity" => parity::run(),
        "sim" => sim(),
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
    println!("  codegen [--check] Generate every surface from crates/support/schema; --check fails on drift");
    println!("  lint-deps        Enforce layers.toml: dependency direction and vendor bans (P5)");
    println!("  tokens [--check] Generate the design tokens for every target; --check fails on drift (N7)");
    println!("  offline          The web app must load nothing from another origin (P2/P9)");
    println!("  parity           Every API operation must have a CLI command (P13)");
    println!("  sim              2,000 seeded histories against the domain invariants (ADR-0014)");
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
    codegen::run(true)?;
    tokens::run(true)?;
    parity::run()?;
    offline::run()?;
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

/// The full simulation: 2,000 seeded histories, the number `docs/50 PH0`
/// acceptance criterion 6 names.
///
/// Not part of `verify` at full scale — `cargo test` runs 200 histories, which
/// is enough to catch a regression while keeping the inner loop fast. CI runs
/// this one. The distinction matters: a gate slow enough that people stop
/// running it is not a gate.
fn sim() -> Result<()> {
    println!("\n== simulation ==");
    let status = std::process::Command::new(env!("CARGO"))
        .args([
            "test",
            "--release",
            "-p",
            "fractal-app-society",
            "--test",
            "simulation",
            "--",
            "--nocapture",
        ])
        .env("FRACTAL_SIM_HISTORIES", "2000")
        .status()?;
    if !status.success() {
        bail!("simulation failed — the seed and step in the panic reproduce it exactly");
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
