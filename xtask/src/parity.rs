//! The P13 parity gate.
//!
//! `docs/00` P13: "Any feature present in the GUI but absent from the CLI, or
//! vice versa, at a release tag is a violation and blocks the tag."
//!
//! Before M0.4 this compared two hand-written lists, which caught drift after it
//! happened. Now every surface is generated from `crates/support/schema`, so
//! drift cannot happen — and this gate's job changes accordingly. It checks the
//! thing codegen cannot check about itself: that **every operation in the
//! contract actually reached every surface**, and that no surface has grown an
//! operation the contract does not declare.
//!
//! A generator with a silent gap is worse than a hand-written list, because
//! nobody is looking. This is the thing looking.

use anyhow::{bail, Result};
use fractal_schema as sch;
use std::collections::BTreeSet;

/// Where an operation must appear, and how to recognise it there.
struct Surface {
    name: &'static str,
    path: &'static str,
    /// How the operation id is spelled in this file.
    spelling: fn(&sch::Operation) -> String,
}

pub(crate) fn run() -> Result<()> {
    let root = crate::root();

    let surfaces = [
        Surface {
            name: "HTTP gateway",
            path: "crates/api/http/src/generated.rs",
            spelling: |op| format!("id: {:?}", op.id),
        },
        Surface {
            name: "CLI",
            path: "crates/bin/cli/src/generated.rs",
            spelling: |op| format!("id: {:?}", op.id),
        },
        Surface {
            name: "OpenAPI",
            path: "schemas/openapi/v1.json",
            spelling: |op| format!("\"operationId\": \"{}\"", op.id),
        },
        Surface {
            name: "TypeScript client",
            path: "packages/api-client/src/index.ts",
            spelling: |op| format!("{}:", camel(op.cli.noun, op.cli.verb)),
        },
        Surface {
            name: "JavaScript client",
            path: "packages/api-client/dist/index.js",
            spelling: |op| format!("{}:", camel(op.cli.noun, op.cli.verb)),
        },
    ];

    let mut missing = Vec::new();
    for s in &surfaces {
        let body = std::fs::read_to_string(root.join(s.path)).unwrap_or_default();
        if body.is_empty() {
            bail!(
                "parity: {} has not been generated — run `cargo xtask codegen`",
                s.path
            );
        }
        for op in sch::OPERATIONS {
            let needle = (s.spelling)(op);
            if !body.contains(&needle) {
                missing.push(format!(
                    "{} is missing from the {} ({})",
                    op.id, s.name, s.path
                ));
            }
        }
    }

    // The reverse direction: a surface must not advertise something the contract
    // does not declare. Checked on the CLI, because that is where a well-meaning
    // hand-added convenience command would go.
    let declared: BTreeSet<&str> = sch::OPERATIONS.iter().map(|o| o.id).collect();
    let cli = std::fs::read_to_string(root.join("crates/bin/cli/src/generated.rs"))?;
    for found in extract(&cli, "id: \"") {
        if !declared.contains(found.as_str()) {
            missing.push(format!(
                "{found} is in the CLI but is not declared in the contract (P3)"
            ));
        }
    }

    // P3: a front end reaches the Runtime through the generated client and
    // nothing else. A hand-written fetch call in an app is how a private path
    // gets added "just this once", and then the CLI is behind the GUI forever.
    for (label, path) in [("web app", "apps/web/app.js")] {
        let body = std::fs::read_to_string(root.join(path)).unwrap_or_default();
        for (needle, why) in [
            ("fetch(", "calls fetch directly"),
            ("XMLHttpRequest", "uses XMLHttpRequest"),
        ] {
            if body.contains(needle) {
                missing.push(format!(
                    "the {label} {why} ({path}) — front ends use the generated client (P3)"
                ));
            }
        }
    }

    // Every event kind must have a published JSON Schema.
    for e in sch::EVENTS {
        let path = root.join(format!("schemas/events/{}.json", e.kind));
        if !path.exists() {
            missing.push(format!(
                "{} has no published schema at {}",
                e.kind,
                path.display()
            ));
        }
    }

    if missing.is_empty() {
        println!(
            "parity: {} operations × {} surfaces, {} event schemas — all present",
            sch::OPERATIONS.len(),
            surfaces.len(),
            sch::EVENTS.len()
        );
        return Ok(());
    }
    for m in &missing {
        eprintln!("  ✕ {m}");
    }
    bail!("parity: {} gap(s)", missing.len())
}

fn camel(noun: &str, verb: &str) -> String {
    let mut out = String::from(noun);
    let mut chars = verb.chars();
    if let Some(first) = chars.next() {
        out.extend(first.to_uppercase());
        out.push_str(chars.as_str());
    }
    out
}

fn extract(src: &str, needle: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (i, _) in src.match_indices(needle) {
        let Some(rest) = src.get(i + needle.len()..) else {
            continue;
        };
        if let Some(end) = rest.find('"') {
            if let Some(v) = rest.get(..end) {
                out.insert(v.to_owned());
            }
        }
    }
    out
}
