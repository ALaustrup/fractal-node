//! Output.
//!
//! `docs/31 §4.1`: `human` has no stability guarantee; `json` is stable within an
//! API major version. `docs/31 §4.2`: when stdout is not a TTY and no format was
//! given, the CLI emits JSON — so `fn society list | jq` works without a flag.
//! That single default is the highest-leverage ergonomic decision in the CLI.

use crate::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum Format {
    Human,
    Json,
}

impl Format {
    /// Resolve the effective format: explicit flag, else TTY detection.
    #[must_use]
    pub(crate) fn resolve(explicit: Option<Self>, stdout_is_tty: bool) -> Self {
        explicit.unwrap_or(if stdout_is_tty {
            Self::Human
        } else {
            Self::Json
        })
    }
}

/// Print a success payload in the chosen format.
pub(crate) fn ok(format: Format, data: &serde_json::Value, human: impl FnOnce()) {
    match format {
        Format::Json => {
            let envelope = serde_json::json!({
                "ok": true,
                "data": data,
                "meta": { "cli_version": env!("CARGO_PKG_VERSION") },
                "warnings": [],
            });
            println!("{envelope}");
        }
        Format::Human => human(),
    }
}

/// Print an error and return the exit code for it (`docs/31 §4.3`).
pub(crate) fn fail(
    format: Format,
    code: &str,
    title: &str,
    detail: &str,
    remedy: Option<&str>,
) -> i32 {
    let exit = exit_code_for(code);
    match format {
        Format::Json => {
            let envelope = serde_json::json!({
                "ok": false,
                "error": {
                    "code": code,
                    "title": title,
                    "detail": detail,
                    "remedy": remedy.map(|r| serde_json::json!({ "human": r })),
                    "retryable": matches!(code, "conflict" | "store_unavailable" | "unreachable"),
                },
                "meta": { "cli_version": env!("CARGO_PKG_VERSION") },
            });
            eprintln!("{envelope}");
        }
        Format::Human => {
            // Cause, then remedy, no apology (docs/33 §7.3).
            eprintln!("{} {title}", theme::rupture("✕"));
            eprintln!("  {detail}");
            if let Some(r) = remedy {
                eprintln!("  {}", theme::muted(r));
            }
        }
    }
    exit
}

/// The closed exit-code set, from the contract (`crates/support/schema`).
///
/// Scripts depend on these, so they are generated rather than typed twice.
#[must_use]
pub(crate) fn exit_code_for(code: &str) -> i32 {
    crate::generated::exit_code_for(code)
}
