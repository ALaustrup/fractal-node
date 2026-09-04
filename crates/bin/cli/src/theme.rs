//! The LATTICE palette, compiled to a terminal (`docs/32 §10`).
//!
//! In PH1 this file is GENERATED from `packages/tokens` by `cargo xtask tokens`,
//! exactly like the CSS, Swift and Kotlin targets — one token source, five
//! outputs (N7). It is hand-written here only because the generator lands in the
//! same phase and this is the smaller half.
//!
//! Degradation ladder: truecolor → 256 → 16 → no colour. `NO_COLOR` is honoured.

use std::sync::OnceLock;

fn colour_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        if std::env::var("TERM").is_ok_and(|t| t == "dumb") {
            return false;
        }
        std::io::IsTerminal::is_terminal(&std::io::stdout())
    })
}

fn paint(rgb: (u8, u8, u8), s: &str) -> String {
    if colour_enabled() {
        let (r, g, b) = rgb;
        format!("\u{1b}[38;2;{r};{g};{b}m{s}\u{1b}[0m")
    } else {
        s.to_owned()
    }
}

/// `--fn-c-signal` #8ce8df — human action, live, success.
pub(crate) fn signal(s: &str) -> String {
    paint((0x8c, 0xe8, 0xdf), s)
}
/// `--fn-c-electric` #55b9ff — data, value, Fraction, focus.
pub(crate) fn electric(s: &str) -> String {
    paint((0x55, 0xb9, 0xff), s)
}
/// `--fn-c-field` #9d6cff — agents. Never reassigned to anything else.
///
/// Unused until PH3, when Agents become principals. It is defined now because
/// the palette is a contract: the hue for non-human origin is decided once
/// (`docs/32 §3.2`), not negotiated when the first agent message appears.
#[allow(dead_code)]
pub(crate) fn field(s: &str) -> String {
    paint((0x9d, 0x6c, 0xff), s)
}
/// `--fn-c-rupture` #ff6b7a — error, destructive.
pub(crate) fn rupture(s: &str) -> String {
    paint((0xff, 0x6b, 0x7a), s)
}
/// `--fn-text-tertiary` #7b868d.
pub(crate) fn muted(s: &str) -> String {
    paint((0x7b, 0x86, 0x8d), s)
}
/// `--fn-text-quaternary` #586267 — mono micro-labels only, never prose.
pub(crate) fn dim(s: &str) -> String {
    paint((0x58, 0x62, 0x67), s)
}

/// The selection rail from `docs/32 §5.3`, in a character cell.
pub(crate) fn rail(selected: bool) -> String {
    if selected {
        signal("▌")
    } else {
        " ".to_owned()
    }
}
