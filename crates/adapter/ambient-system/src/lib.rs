//! The production implementations of the three ambient ports.
//!
//! This crate exists so that `SystemTime::now()` appears in exactly one place in
//! the entire workspace. `layers.toml` bans it everywhere else; the ban has to
//! have somewhere to point at, and this is it (ADR-0014).

use fractal_ports::{Clock, IdGen, Rng};
use fractal_types::{Timestamp, Ulid};
use std::time::{SystemTime, UNIX_EPOCH};

/// Wall-clock time, UTC.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        // A clock before 1970 is a broken machine, not a state we model.
        // Saturating to the epoch keeps the type total without hiding the fault:
        // an event stamped at the epoch is conspicuous in any log.
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
        Timestamp::from_millis(ms)
    }
}

/// Operating-system randomness.
///
/// PH0 sources entropy from ULID generation, which is seeded by the OS. When the
/// first genuinely security-sensitive consumer arrives in PH1 (key generation,
/// docs/12), this is replaced by an explicit CSPRNG and the swap is confined to
/// this file — which is the point of the port.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemRng;

impl Rng for SystemRng {
    fn fill_bytes(&self, dest: &mut [u8]) {
        let mut written = 0;
        while written < dest.len() {
            let block = ulid::Ulid::new().0.to_le_bytes();
            for b in block {
                if let Some(slot) = dest.get_mut(written) {
                    *slot = b;
                    written += 1;
                } else {
                    return;
                }
            }
        }
    }
}

/// Time-ordered identifiers.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemIdGen;

impl IdGen for SystemIdGen {
    fn next_ulid(&self) -> Ulid {
        Ulid::from_u128(ulid::Ulid::new().0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_is_after_2020() {
        // Catches a machine with a wildly wrong clock, and catches the saturating
        // branch above silently becoming the normal path.
        assert!(SystemClock.now().as_millis() > 1_577_836_800_000);
    }

    #[test]
    fn ids_are_unique_and_ordered() {
        let g = SystemIdGen;
        let a = g.next_ulid();
        let b = g.next_ulid();
        assert_ne!(a, b);
        assert!(a < b);
    }

    #[test]
    fn rng_fills_every_byte() {
        let mut buf = [0u8; 37];
        SystemRng.fill_bytes(&mut buf);
        assert!(buf.iter().any(|&b| b != 0));
    }
}
