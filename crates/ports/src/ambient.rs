//! Ambient authority: time, randomness, identity generation.
//!
//! These three are ports for one reason, and it is the load-bearing decision of
//! the whole test strategy (ADR-0014): with them abstracted, a whole history can
//! be replayed byte-for-byte, which is what lets `docs/11 §7`'s fifteen invariants
//! be *proved* over generated histories instead of spot-checked.
//!
//! `layers.toml` bans `SystemTime`, `thread_rng` and `Ulid::new` outright, so
//! there is no second way to obtain any of them.

use fractal_types::{Timestamp, Ulid};

/// The source of wall-clock time.
pub trait Clock: Send + Sync + 'static {
    /// The current instant, UTC.
    fn now(&self) -> Timestamp;
}

/// The source of randomness.
pub trait Rng: Send + Sync + 'static {
    /// Fill `dest` with random bytes.
    fn fill_bytes(&self, dest: &mut [u8]);

    /// A random `u64`.
    fn next_u64(&self) -> u64 {
        let mut b = [0u8; 8];
        self.fill_bytes(&mut b);
        u64::from_le_bytes(b)
    }
}

/// The source of new identifiers.
///
/// Separate from [`Rng`] because a ULID is time-ordered: a deterministic
/// implementation must be able to control the time component and the random
/// component independently.
pub trait IdGen: Send + Sync + 'static {
    /// Mint a new time-sortable identifier.
    fn next_ulid(&self) -> Ulid;
}
