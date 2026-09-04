//! The offline gate — P2 and P9, enforced on the web app.
//!
//! `docs/00` P2: "A Node is useful with no internet." P9: "A Citizen is not
//! told about by default." A single `<link>` to a font CDN breaks both at once,
//! and does it invisibly: the page looks right on the developer's machine, and
//! degrades — or phones home — on everyone else's.
//!
//! That is not a hypothetical. The PH0 GUI shipped exactly that link, and no
//! gate caught it, because every other gate was watching the Rust. This one
//! watches the front end.
//!
//! What it refuses: any reference the browser would FETCH from an origin that
//! is not this Node. Prose is not a fetch, so a comment or a licence may name a
//! URL — the check reads reference positions (`src=`, `href=`, `url()`,
//! `@import`, ES `import ... from`, and direct `fetch`), never loose text.

use anyhow::{bail, Result};
use std::fmt::Write as _;

/// Byte patterns that begin an EXTERNAL reference, once a reference position
/// has been recognised. `//host` is included: protocol-relative URLs are still
/// third-party, and are the classic way one slips past a check for `https://`.
const EXTERNAL: [&str; 3] = ["http://", "https://", "//"];

/// Which files are front-end source. Everything served from `apps/web` that a
/// browser parses. Binary assets (fonts, images) hold no references.
const EXTENSIONS: [&str; 5] = ["html", "css", "js", "svg", "webmanifest"];

pub(crate) fn run() -> Result<()> {
    let root = crate::root();
    let web = root.join("apps/web");
    if !web.is_dir() {
        bail!("apps/web is missing — the offline gate has nothing to check");
    }

    let mut findings = Vec::new();
    let mut checked = 0usize;

    for path in walk(&web)? {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        checked += 1;
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .display()
            .to_string()
            .replace('\\', "/");

        for (line_no, line) in strip_comments(&raw, &ext).lines().enumerate() {
            for reference in references(line) {
                if EXTERNAL.iter().any(|p| reference.starts_with(p)) {
                    findings.push(format!(
                        "{rel}:{} loads `{reference}` from another origin \
                         — a Node must render with no internet (P2) and must \
                         not announce its Citizens to third parties (P9)",
                        line_no + 1
                    ));
                }
            }
        }
    }

    if !findings.is_empty() {
        let mut msg = String::from("offline: the web app reaches off this Node\n");
        for f in &findings {
            let _ = writeln!(msg, "  - {f}");
        }
        msg.push_str(
            "\nVendor the asset into apps/web and reference it by an absolute \
             path on this Node.",
        );
        bail!(msg);
    }

    println!("offline: {checked} front-end files, no third-party references");
    Ok(())
}

/// Pull out every string that sits in a fetchable position on this line.
fn references(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = line.to_ascii_lowercase();

    // attribute="value", then url(...) / @import / ES import / fetch.
    for opener in [
        "src=", "href=", "srcset=", "action=", "poster=", "data=", "url(", "@import ", "from ",
        "import(", "fetch(",
    ] {
        let mut from = 0usize;
        while let Some(rest) = lower.get(from..) {
            let Some(hit) = rest.find(opener) else { break };
            let after = from.saturating_add(hit).saturating_add(opener.len());
            if let Some(tail) = line.get(after..) {
                out.extend(head_token(tail));
            }
            from = after;
        }
    }
    out
}

/// The first quoted (or bare, for `url(...)`) token at the head of `rest`.
fn head_token(rest: &str) -> Option<String> {
    let rest = rest.trim_start();
    let mut chars = rest.chars();
    match chars.next() {
        Some(q @ ('"' | '\'')) => {
            let inner = rest.get(q.len_utf8()..)?;
            let end = inner.find(q)?;
            inner.get(..end).map(str::to_owned)
        }
        Some(_) => {
            let end = rest.find([')', ' ', ';', '>']).unwrap_or(rest.len());
            rest.get(..end)
                .map(|t| t.trim_matches(['"', '\'']).to_owned())
        }
        None => None,
    }
}

/// Blank out comments so prose that names a URL is not mistaken for a fetch.
/// Replaces with spaces rather than deleting, so line numbers survive.
fn strip_comments(src: &str, ext: &str) -> String {
    let markup = matches!(ext, "html" | "svg");
    let (open, close): (&str, &str) = if markup {
        ("<!--", "-->")
    } else {
        ("/*", "*/")
    };

    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    loop {
        // A `//` line comment in JS, but only outside a string literal.
        let block_at = rest.find(open);
        let line_at = if ext == "js" {
            find_line_comment(rest)
        } else {
            None
        };

        match (block_at, line_at) {
            (None, None) => {
                out.push_str(rest);
                return out;
            }
            (Some(b), l) if l.is_none_or(|l| b < l) => {
                let Some((before, after)) = split(rest, b) else {
                    out.push_str(rest);
                    return out;
                };
                out.push_str(before);
                let body_end = after.find(close).map_or(after.len(), |e| {
                    e.saturating_add(close.len()).min(after.len())
                });
                let Some((comment, tail)) = split(after, body_end) else {
                    return out;
                };
                blank_into(&mut out, comment);
                rest = tail;
            }
            (_, Some(l)) => {
                let Some((before, after)) = split(rest, l) else {
                    out.push_str(rest);
                    return out;
                };
                out.push_str(before);
                let eol = after.find('\n').unwrap_or(after.len());
                let Some((comment, tail)) = split(after, eol) else {
                    return out;
                };
                blank_into(&mut out, comment);
                rest = tail;
            }
            (Some(_), None) => {
                // Unreachable in practice: the guarded arm above takes every
                // `(Some, None)`. Kept because exhaustiveness is checked
                // before guards, and a `todo!()` here would be a panic in a
                // build tool.
                out.push_str(rest);
                return out;
            }
        }
    }
}

/// Byte offset of the first `//` that is NOT inside a string literal and NOT
/// part of a `://` scheme separator. Both exclusions matter: the first keeps a
/// URL in code readable, the second stops `https://x` being read as a comment.
fn find_line_comment(s: &str) -> Option<usize> {
    let mut single = false;
    let mut double = false;
    let mut prev = '\0';
    let mut prev2 = '\0';
    for (i, c) in s.char_indices() {
        if prev != '\\' {
            match c {
                '\'' if !double => single = !single,
                '"' if !single => double = !double,
                _ => {}
            }
        }
        if c == '/' && prev == '/' && prev2 != ':' && !single && !double {
            return Some(i.saturating_sub(1));
        }
        prev2 = prev;
        prev = c;
    }
    None
}

fn split(s: &str, at: usize) -> Option<(&str, &str)> {
    if s.is_char_boundary(at) {
        Some((s.get(..at)?, s.get(at..)?))
    } else {
        None
    }
}

fn blank_into(out: &mut String, comment: &str) {
    for c in comment.chars() {
        out.push(if c == '\n' { '\n' } else { ' ' });
    }
}

fn walk(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}
