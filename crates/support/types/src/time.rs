//! Time.
//!
//! There is no `now()` here. Wall-clock time enters the system through the
//! `Clock` port only (ADR-0014) — that is what makes every history replayable
//! and every invariant in `docs/11 §7` actually checkable.

use std::fmt;

/// A UTC instant, milliseconds since the Unix epoch.
///
/// `docs/10 §10`: domain time (`occurred_at`) and wall time (`recorded_at`) are
/// distinct fields and are never conflated. This type is used for both; which
/// one you are holding is the field's name, not the type's.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    pub const EPOCH: Self = Self(0);

    #[must_use]
    pub const fn from_millis(ms: i64) -> Self {
        Self(ms)
    }

    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0
    }

    /// # Errors
    /// Returns `None` on overflow.
    #[must_use]
    pub const fn checked_add(self, d: Duration) -> Option<Self> {
        match self.0.checked_add(d.0) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    #[must_use]
    pub const fn saturating_since(self, earlier: Self) -> Duration {
        Duration(self.0.saturating_sub(earlier.0))
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T{}ms", self.0)
    }
}

/// A span of time in milliseconds.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct Duration(i64);

impl Duration {
    #[must_use]
    pub const fn from_millis(ms: i64) -> Self {
        Self(ms)
    }
    #[must_use]
    pub const fn from_secs(s: i64) -> Self {
        Self(s.saturating_mul(1_000))
    }
    #[must_use]
    pub const fn from_days(d: i64) -> Self {
        Self(d.saturating_mul(86_400_000))
    }
    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0
    }
}

impl fmt::Debug for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn durations_compose() {
        let t = Timestamp::from_millis(1_000);
        let later = t.checked_add(Duration::from_days(14)).unwrap();
        assert_eq!(later.saturating_since(t), Duration::from_days(14));
    }

    #[test]
    fn there_is_no_now_function() {
        // This test exists as documentation. If someone adds `Timestamp::now()`,
        // the Clock port is dead and every replay test becomes a lie (ADR-0014).
        // The compile-time enforcement is the `std::time::SystemTime` ban in layers.toml.
        assert_eq!(Timestamp::EPOCH.as_millis(), 0);
    }
}
