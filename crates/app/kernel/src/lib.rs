//! The application kernel: everything a command passes through before it becomes an event.
//!
//! Two things live here that must live nowhere else:
//!
//! 1. **The Policy Enforcement Point** (`docs/10 §8`). It sits in the application
//!    layer, inside the trust boundary, on the path every command takes regardless
//!    of front end. An Agent cannot route around it because there is no other route.
//!    In PH0 it enforces one rule; the Envelope machinery lands in PH3 behind the
//!    same function signature, so nothing above it changes.
//!
//! 2. **Idempotency** (`docs/10 §10`). Every command carries a client-generated key,
//!    deduped per principal. This is what makes the CLI, agents, and flaky mobile
//!    networks safe to retry — and retry safety is a precondition for P13, not a
//!    nicety.

use fractal_ports::{Clock, IdGen};
use fractal_types::{Principal, Timestamp, Ulid};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A class of action, used by the PEP to decide what a principal may do.
///
/// Closed on purpose. Adding a class is an architectural decision and must be
/// accompanied by a rule in [`Pep::authorise`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionClass {
    /// Reads. Always permitted to an authenticated principal in PH0.
    Read,
    /// Ordinary state change: posting, uploading, editing your own things.
    Write,
    /// Authors or amends policy: Charter, roles, capability grants.
    ///
    /// P4: **only a Citizen may ever be permitted this.** No Envelope, no
    /// configuration and no future phase may grant it to an Agent.
    Policy,
    /// Cannot be undone: fund transfer above threshold, member removal, asset
    /// burn, governance enactment, external publication.
    Irreversible,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PolicyDenied {
    /// The action requires a human and the actor is not one.
    #[error("{class:?} actions require a Citizen; {actor} may not author policy (P4)")]
    NotHuman {
        class: ActionClass,
        actor: &'static str,
    },
    /// The Envelope did not permit this. Populated from PH3.
    #[error("capability `{capability}` is not in this principal's Envelope")]
    CapabilityDenied { capability: String },
    /// The action needs a live human confirmation that has not been given.
    #[error("{class:?} actions require explicit human confirmation")]
    ConfirmationRequired { class: ActionClass },
}

/// The Policy Enforcement Point.
///
/// PH0 implements the P4 rule and nothing else, because nothing else exists yet.
/// The signature is the one PH3 needs, so adding Envelopes is filling this in
/// rather than moving the boundary.
#[derive(Debug, Default, Clone, Copy)]
pub struct Pep;

impl Pep {
    /// Decide whether `actor` may perform an action of `class`.
    ///
    /// # Errors
    /// [`PolicyDenied`] when the action is refused. A denial is a value the
    /// caller renders and — from PH3 — a first-class domain event, never a panic.
    pub fn authorise(self, actor: &Principal, class: ActionClass) -> Result<(), PolicyDenied> {
        match class {
            ActionClass::Read | ActionClass::Write => Ok(()),
            ActionClass::Policy | ActionClass::Irreversible => {
                if actor.is_human() {
                    Ok(())
                } else {
                    Err(PolicyDenied::NotHuman {
                        class,
                        actor: describe(actor),
                    })
                }
            }
        }
    }
}

const fn describe(p: &Principal) -> &'static str {
    match p {
        Principal::Citizen { .. } => "a Citizen",
        Principal::Agent { .. } => "an Agent",
        Principal::Society { .. } => "a Society",
        Principal::Node { .. } => "a Node",
        Principal::System => "the System",
    }
}

/// A client-supplied key that makes a command safe to retry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Records which commands have already been executed, per principal.
///
/// PH0 keeps this in memory; PH1 moves it behind a port alongside the event
/// store. The 24-hour window is fixed by `docs/10 §10`.
#[derive(Debug, Default)]
pub struct IdempotencyLedger {
    seen: Mutex<HashMap<(String, String), (Ulid, Timestamp)>>,
}

pub const IDEMPOTENCY_WINDOW_MS: i64 = 24 * 60 * 60 * 1_000;

impl IdempotencyLedger {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the correlation id of a previous identical command, if any.
    ///
    /// A hit means "this already happened" — the caller returns the original
    /// result rather than doing the work twice.
    #[must_use]
    pub fn seen(&self, actor: &Principal, key: &IdempotencyKey, now: Timestamp) -> Option<Ulid> {
        let k = Self::key_for(actor, key);
        let guard = self.seen.lock().ok()?;
        guard.get(&k).and_then(|(id, at)| {
            (now.saturating_since(*at).as_millis() < IDEMPOTENCY_WINDOW_MS).then_some(*id)
        })
    }

    /// Record that this command ran, with the correlation id it produced.
    pub fn record(
        &self,
        actor: &Principal,
        key: &IdempotencyKey,
        correlation_id: Ulid,
        now: Timestamp,
    ) {
        if let Ok(mut guard) = self.seen.lock() {
            guard.insert(Self::key_for(actor, key), (correlation_id, now));
        }
    }

    fn key_for(actor: &Principal, key: &IdempotencyKey) -> (String, String) {
        let who = actor
            .fnid()
            .map_or_else(|| "system".to_owned(), ToString::to_string);
        (who, key.0.clone())
    }
}

/// Everything a command handler needs that it must not create for itself.
///
/// Handlers take this rather than reaching for a clock or an id generator, which
/// is what keeps every handler replayable (ADR-0014).
pub struct CommandContext {
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGen>,
    pub pep: Pep,
    pub idempotency: Arc<IdempotencyLedger>,
}

impl std::fmt::Debug for CommandContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandContext").finish_non_exhaustive()
    }
}

impl CommandContext {
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, ids: Arc<dyn IdGen>) -> Self {
        Self {
            clock,
            ids,
            pep: Pep,
            idempotency: Arc::new(IdempotencyLedger::new()),
        }
    }

    #[must_use]
    pub fn now(&self) -> Timestamp {
        self.clock.now()
    }

    #[must_use]
    pub fn next_id(&self) -> Ulid {
        self.ids.next_ulid()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use fractal_types::Fnid;

    fn citizen() -> Principal {
        Principal::Citizen {
            fnid: Fnid::sample(1),
        }
    }
    fn agent() -> Principal {
        Principal::Agent {
            fnid: Fnid::sample(2),
            operator: Fnid::sample(1),
        }
    }

    #[test]
    fn agents_may_act_but_never_author_policy() {
        let pep = Pep;
        assert!(pep.authorise(&agent(), ActionClass::Read).is_ok());
        assert!(pep.authorise(&agent(), ActionClass::Write).is_ok());
        // P4, the whole point: execution is delegable, policy is not.
        assert!(matches!(
            pep.authorise(&agent(), ActionClass::Policy),
            Err(PolicyDenied::NotHuman { .. })
        ));
        assert!(matches!(
            pep.authorise(&agent(), ActionClass::Irreversible),
            Err(PolicyDenied::NotHuman { .. })
        ));
    }

    #[test]
    fn citizens_may_author_policy() {
        assert!(Pep.authorise(&citizen(), ActionClass::Policy).is_ok());
    }

    #[test]
    fn no_non_human_principal_can_author_policy() {
        // Exhaustive rather than illustrative: if a new Principal variant is added
        // and it is not a Citizen, this test must keep refusing it.
        let others = [
            agent(),
            Principal::Society {
                society_id: fractal_types::SocietyId::new(Ulid::from_u128(1)),
            },
            Principal::Node {
                fnid: Fnid::sample(3),
            },
            Principal::System,
        ];
        for p in &others {
            assert!(
                Pep.authorise(p, ActionClass::Policy).is_err(),
                "{p:?} must not author policy"
            );
        }
    }

    #[test]
    fn a_repeated_command_is_recognised() {
        let led = IdempotencyLedger::new();
        let key = IdempotencyKey::new("abc-123");
        let t0 = Timestamp::from_millis(0);
        assert!(led.seen(&citizen(), &key, t0).is_none());
        led.record(&citizen(), &key, Ulid::from_u128(7), t0);
        assert_eq!(led.seen(&citizen(), &key, t0), Some(Ulid::from_u128(7)));
    }

    #[test]
    fn idempotency_is_scoped_to_the_principal() {
        let led = IdempotencyLedger::new();
        let key = IdempotencyKey::new("same-key");
        let t0 = Timestamp::from_millis(0);
        led.record(&citizen(), &key, Ulid::from_u128(1), t0);
        // Another principal reusing the same key must not collide with mine.
        assert!(led.seen(&agent(), &key, t0).is_none());
    }

    #[test]
    fn idempotency_expires_after_the_window() {
        let led = IdempotencyLedger::new();
        let key = IdempotencyKey::new("k");
        led.record(
            &citizen(),
            &key,
            Ulid::from_u128(1),
            Timestamp::from_millis(0),
        );
        let later = Timestamp::from_millis(IDEMPOTENCY_WINDOW_MS + 1);
        assert!(led.seen(&citizen(), &key, later).is_none());
    }
}
