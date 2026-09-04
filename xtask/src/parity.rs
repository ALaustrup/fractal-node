//! The P13 parity gate.
//!
//! `docs/00` P13: "Any feature present in the GUI but absent from the CLI, or
//! vice versa, at a release tag is a violation and blocks the tag."
//!
//! PH0 checks it structurally: every operation the gateway advertises must have a
//! CLI command, and every CLI operation must be a real gateway operation. From
//! PH1 both sides are generated from `fractal-schema` and this becomes a check on
//! the generator's output rather than on two hand-written lists — but the gate
//! itself never moves.

use anyhow::{bail, Result};
use std::collections::BTreeSet;

pub(crate) fn run() -> Result<()> {
    let root = crate::root();

    let api_src = std::fs::read_to_string(root.join("crates/api/http/src/lib.rs"))?;
    let cli_src = std::fs::read_to_string(root.join("crates/bin/cli/src/main.rs"))?;

    let api_ops = extract(&api_src, "\"id\": \"");
    let cli_ops = extract(&cli_src, "\"operation\": \"");

    let cli_only: BTreeSet<_> = cli_ops.difference(&api_ops).collect();
    let api_only: BTreeSet<_> = api_ops.difference(&cli_ops).collect();

    // `cli.schema` is the CLI describing itself; it has no server operation and
    // is the one permitted asymmetry. Anything else is a violation.
    let cli_only: BTreeSet<_> = cli_only
        .into_iter()
        .filter(|o| o.as_str() != "cli.schema")
        .collect();

    if api_ops.is_empty() {
        bail!("parity: found no API operations to compare — the /v1/meta list is the source");
    }

    if cli_only.is_empty() && api_only.is_empty() {
        println!(
            "parity: {} operations, reachable from both the API and the CLI",
            api_ops.len()
        );
        return Ok(());
    }
    for op in &api_only {
        eprintln!("  ✕ {op} is in the API but has no CLI command (P13)");
    }
    for op in &cli_only {
        eprintln!("  ✕ {op} is in the CLI but is not an API operation (P3)");
    }
    bail!("parity: {} mismatch(es)", api_only.len() + cli_only.len())
}

fn extract(src: &str, needle: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (i, _) in src.match_indices(needle) {
        let rest = &src[i + needle.len()..];
        if let Some(end) = rest.find('"') {
            out.insert(rest[..end].to_owned());
        }
    }
    out
}
