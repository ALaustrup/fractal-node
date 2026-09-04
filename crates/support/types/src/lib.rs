//! Canonical newtypes for Fractal Node.
//!
//! Every term here comes verbatim from `docs/01-canonical-terminology.md`. A bare
//! `String` or `i64` crossing a boundary is a defect: the type is the documentation.

mod id;
mod quanta;
mod time;

pub use id::{Fnid, Handle, HandleError, IdError, SocietyId, Ulid};
pub use quanta::{Quanta, QuantaError, FRC, MAX_LIFETIME_FRC};
pub use time::{Duration, Timestamp};

/// A principal: anything that can hold capabilities (`docs/01 §2`).
///
/// The variants are closed on purpose. A new kind of actor is an architectural
/// decision, not a convenience.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Principal {
    /// A human account.
    Citizen { fnid: Fnid },
    /// An autonomous non-human principal, accountable to exactly one Operator.
    Agent { fnid: Fnid, operator: Fnid },
    /// A Society acting as itself (Treasury movements, Charter enactment).
    Society { society_id: SocietyId },
    /// A running Fractal Node instance.
    Node { fnid: Fnid },
    /// The Runtime itself. Reserved for scheduled and migration events.
    System,
}

impl Principal {
    /// True when this principal is a human being.
    ///
    /// P4 depends on this distinction being cheap and impossible to get wrong:
    /// only a Citizen may author Policy or sign a capability grant.
    #[must_use]
    pub const fn is_human(&self) -> bool {
        matches!(self, Self::Citizen { .. })
    }

    /// The FNID this principal acts under, where it has one.
    #[must_use]
    pub const fn fnid(&self) -> Option<&Fnid> {
        match self {
            Self::Citizen { fnid } | Self::Agent { fnid, .. } | Self::Node { fnid } => Some(fnid),
            Self::Society { .. } | Self::System => None,
        }
    }
}

/// How visible a Society is (`docs/11 §2.2`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Listed and readable by anyone; joinable per Charter.
    Public,
    /// Listed in discovery; contents hidden until joined.
    #[default]
    Discoverable,
    /// Unlisted, invite only.
    Private,
    /// Unlisted, invite only, no new members, archive-only.
    Sealed,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn only_citizens_are_human() {
        let c = Principal::Citizen {
            fnid: Fnid::sample(1),
        };
        let a = Principal::Agent {
            fnid: Fnid::sample(2),
            operator: Fnid::sample(1),
        };
        assert!(c.is_human());
        assert!(!a.is_human(), "an Agent must never read as human (P4)");
        assert!(!Principal::System.is_human());
    }

    #[test]
    fn principal_round_trips_through_json() {
        let p = Principal::Agent {
            fnid: Fnid::sample(7),
            operator: Fnid::sample(3),
        };
        let s = serde_json::to_string(&p).expect("serialize");
        let back: Principal = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(p, back);
        assert!(
            s.contains("\"kind\":\"agent\""),
            "the wire form names the principal class"
        );
    }
}
