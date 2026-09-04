//! Every operation in the contract must be *runnable*, not merely listed.
//!
//! `cargo xtask parity` checks that each operation reached each generated
//! surface. That is necessary and not sufficient: the generated table could
//! name `society.fracture` while the clap tree has no such command, and both
//! the generator and the gate would be satisfied while `fn society fracture`
//! printed "unrecognized subcommand".
//!
//! This test walks the actual argument parser and asserts that every declared
//! `<noun> <verb>` resolves. It is the difference between "the contract says so"
//! and "the binary does so" — which is the whole of P13.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::Command;

/// The operations the contract declares, read from the generated CLI table.
///
/// Parsed from the generated source rather than linked, because the table is
/// `pub(crate)` — and it should stay that way: it is an implementation detail of
/// the binary, not a public API.
fn declared() -> Vec<(String, String)> {
    // Scanned across the whole file, not line by line: rustfmt is free to break
    // a struct literal wherever it likes, and a test that depends on its line
    // choices is a test that fails for the wrong reason.
    let src = include_str!("../src/generated.rs");
    let nouns = all(src, "noun: \"");
    let verbs = all(src, "verb: \"");
    assert_eq!(nouns.len(), verbs.len(), "malformed generated table");
    nouns.into_iter().zip(verbs).collect()
}

fn all(src: &str, needle: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in src.match_indices(needle) {
        let Some(rest) = src.get(i + needle.len()..) else {
            continue;
        };
        let Some(end) = rest.find('"') else { continue };
        if let Some(v) = rest.get(..end) {
            out.push(v.to_owned());
        }
    }
    out
}

fn fn_binary() -> std::path::PathBuf {
    // The test binary sits beside the one under test.
    let mut p = std::env::current_exe().expect("test binary path");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join("fn")
}

#[test]
fn every_declared_command_is_runnable() {
    let bin = fn_binary();
    assert!(bin.exists(), "build `fn` first: {}", bin.display());

    let ops = declared();
    assert!(
        !ops.is_empty(),
        "the generated command table is empty — run `cargo xtask codegen`"
    );

    let mut unreachable = Vec::new();
    for (noun, verb) in &ops {
        // `--help` resolves the path without performing the operation, so this
        // needs no running Node and has no side effects.
        let out = Command::new(&bin)
            .args([noun, verb, "--help"])
            .output()
            .expect("run fn");
        if !out.status.success() {
            unreachable.push(format!(
                "fn {noun} {verb} — {}",
                String::from_utf8_lossy(&out.stderr)
                    .lines()
                    .next()
                    .unwrap_or("no such command")
            ));
        }
    }

    assert!(
        unreachable.is_empty(),
        "the contract declares operations the binary cannot run (P13):\n  {}",
        unreachable.join("\n  ")
    );
}

#[test]
fn the_shorthand_still_works() {
    // `fn status` is typed constantly; it stays as an alias for `fn node status`.
    let out = Command::new(fn_binary())
        .args(["status", "--help"])
        .output()
        .expect("run fn");
    assert!(out.status.success());
}
