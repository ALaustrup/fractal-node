//! Deterministic fakes and the seeded simulation harness.
//!
//! Everything ambient — time, randomness, identity — is controllable here, so a
//! generated history can be replayed byte-for-byte. That is what turns
//! `docs/11 §7`'s fifteen invariants from claims into tests (ADR-0014).
//!
//! This crate ships in `[dependencies]`, not `[dev-dependencies]`, because the
//! simulation binary and `cargo xtask sim` both link it. It is never linked into
//! a shipped binary: `layers.toml` places it in the `support` layer and the
//! release profile excludes it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod gen;

pub use gen::Gen;

use fractal_ports::{Clock, IdGen, Rng};
use fractal_types::{Timestamp, Ulid};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

/// A clock that only moves when you move it.
#[derive(Debug)]
pub struct FakeClock {
    now: AtomicI64,
    step_ms: i64,
}

impl FakeClock {
    /// A clock starting at `start_ms` that advances `step_ms` on every read.
    ///
    /// A non-zero step is usually what you want: it surfaces code that assumes
    /// two reads of the clock return the same value.
    #[must_use]
    pub const fn new(start_ms: i64, step_ms: i64) -> Self {
        Self {
            now: AtomicI64::new(start_ms),
            step_ms,
        }
    }

    /// A clock frozen at `start_ms`.
    #[must_use]
    pub const fn frozen(start_ms: i64) -> Self {
        Self::new(start_ms, 0)
    }

    /// Move the clock forward explicitly.
    pub fn advance(&self, ms: i64) {
        self.now.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Timestamp {
        let v = self.now.fetch_add(self.step_ms, Ordering::SeqCst);
        Timestamp::from_millis(v)
    }
}

/// A seeded, reproducible PRNG.
///
/// `SplitMix64`. Not cryptographic and never used for anything that needs to be:
/// its only job is to make a generated history reproducible from its seed.
#[derive(Debug)]
pub struct SeededRng {
    state: AtomicU64,
}

impl SeededRng {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self {
            state: AtomicU64::new(seed),
        }
    }

    fn next(&self) -> u64 {
        let mut z = self
            .state
            .fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::SeqCst)
            .wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

impl Rng for SeededRng {
    fn fill_bytes(&self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            let bytes = self.next().to_le_bytes();
            for (slot, b) in chunk.iter_mut().zip(bytes.iter()) {
                *slot = *b;
            }
        }
    }
    fn next_u64(&self) -> u64 {
        self.next()
    }
}

/// A counting identifier generator.
///
/// Produces ULIDs whose values ascend by one, so an event log written under it
/// is byte-identical across runs and diffable when it changes.
#[derive(Debug)]
pub struct CountingIdGen {
    next: AtomicU64,
}

impl CountingIdGen {
    #[must_use]
    pub const fn new(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start),
        }
    }
}

impl Default for CountingIdGen {
    fn default() -> Self {
        Self::new(1)
    }
}

impl IdGen for CountingIdGen {
    fn next_ulid(&self) -> Ulid {
        Ulid::from_u128(u128::from(self.next.fetch_add(1, Ordering::SeqCst)))
    }
}

/// The three ambient ports, wired deterministically, ready to hand to a handler.
#[derive(Debug)]
pub struct Deterministic {
    pub clock: FakeClock,
    pub rng: SeededRng,
    pub ids: CountingIdGen,
}

impl Deterministic {
    /// The seed used when a test does not care which world it gets. Fixed so
    /// that "it passed on my machine" and "it passed in CI" mean the same thing.
    pub const DEFAULT_SEED: u64 = 0x0FAC_7A15_EED0;

    /// A world seeded by `seed`. The same seed always produces the same history.
    #[must_use]
    pub const fn seeded(seed: u64) -> Self {
        Self {
            clock: FakeClock::new(1_700_000_000_000, 1_000),
            rng: SeededRng::new(seed),
            ids: CountingIdGen::new(1),
        }
    }
}

impl Default for Deterministic {
    fn default() -> Self {
        Self::seeded(Self::DEFAULT_SEED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_produces_the_same_history() {
        let a = SeededRng::new(42);
        let b = SeededRng::new(42);
        let av: Vec<u64> = (0..16).map(|_| a.next_u64()).collect();
        let bv: Vec<u64> = (0..16).map(|_| b.next_u64()).collect();
        assert_eq!(
            av, bv,
            "a seeded run must be reproducible or ADR-0014 is void"
        );
    }

    #[test]
    fn different_seeds_diverge() {
        let a = SeededRng::new(1);
        let b = SeededRng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn the_clock_advances_only_as_told() {
        let c = FakeClock::frozen(1_000);
        assert_eq!(c.now(), c.now());
        c.advance(500);
        assert_eq!(c.now().as_millis(), 1_500);
    }

    #[test]
    fn ids_ascend() {
        let g = CountingIdGen::default();
        assert!(g.next_ulid() < g.next_ulid());
    }
}
