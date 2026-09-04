//! The Society aggregate — the atomic container (P1, `docs/11 §2.2`).
//!
//! This crate is **pure**. It takes a command and current state and returns
//! events or a rejection. It reads no clock, opens no socket, and allocates no
//! identifier of its own: those arrive as inputs so that any history can be
//! replayed exactly (ADR-0014).
//!
//! Phase 0 scope: create a Society, and enforce the founding rules. Chambers,
//! Charter amendment, Membership and Fracture arrive in their phases per
//! `docs/03-phase-authority.md`.

use fractal_ports::{EventEnvelope, EventKind, Seq};
use fractal_types::{Fnid, Handle, Principal, SocietyId, Timestamp, Ulid, Visibility};

pub const EVENT_SOCIETY_CREATED: EventKind = EventKind::from_static("society.created.v1");

/// The state a decision is made against.
///
/// A newly-forming Society has no state; `Option<Society>` in the handler is the
/// difference between "create" and "already exists".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Society {
    pub society_id: SocietyId,
    pub name: String,
    pub handle: Handle,
    pub founder: Fnid,
    pub visibility: Visibility,
    pub status: Status,
    pub created_at: Timestamp,
    pub member_count: u32,
}

/// `docs/11 §4`. Phase 0 reaches only `Forming` → `Active`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Forming,
    Active,
    Dormant,
    Fracturing,
    Dissolving,
    Archived,
}

/// The command to found a Society.
#[derive(Debug, Clone)]
pub struct CreateSociety {
    /// Who is founding it. Must be a Citizen — a Society cannot found itself and
    /// an Agent cannot found one (P4).
    pub actor: Principal,
    pub name: String,
    pub handle: Handle,
    pub visibility: Visibility,
    /// How many Societies this Citizen has already founded.
    ///
    /// Passed in rather than looked up: the domain layer does no I/O. The
    /// "first hearth" rule (`docs/61`, adopted) reads this.
    pub societies_founded: u32,
    /// The founder's Level. `docs/18 §5.1` gates founding at Level 3 — except
    /// for a Citizen's first, which is free at Level 0.
    pub founder_level: u16,
}

/// Inputs the domain must not invent for itself.
#[derive(Debug, Clone, Copy)]
pub struct Ambient {
    pub society_id: SocietyId,
    pub event_id: Ulid,
    pub correlation_id: Ulid,
    pub now: Timestamp,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CreateError {
    #[error("only a Citizen may found a Society — {0} cannot")]
    NotAHuman(&'static str),
    #[error("a Society name is 1–64 characters; got {0}")]
    NameLength(usize),
    #[error("a Society name cannot be only whitespace")]
    NameBlank,
    #[error(
        "founding a second Society requires Level 3; you are Level {level}. \
         Your first Society is free at Level 0 — you have founded {founded}."
    )]
    LevelRequired { level: u16, founded: u32 },
    #[error("that Society already exists")]
    AlreadyExists,
}

pub const NAME_MAX: usize = 64;

/// The founding rule adopted from `docs/70` proposal 1 and ruled in `docs/61`:
///
/// > Every Citizen may found exactly ONE Society at Level 0 (the "first hearth").
/// > Founding a second requires Level 3.
///
/// This exists because `docs/02 §2`'s spine sentence and `docs/50 PH1`'s
/// three-minute acceptance criterion both require a Level 0 Citizen to be able
/// to create a Society, while `docs/18` needs a gate to stop Society spam.
pub const FIRST_HEARTH_IS_FREE: bool = true;
pub const SECOND_SOCIETY_LEVEL: u16 = 3;

/// Decide whether a Society may be founded, and if so what happened.
///
/// # Errors
/// See [`CreateError`]. A rejection is a value, not a panic: the caller renders it.
pub fn create(
    state: Option<&Society>,
    cmd: &CreateSociety,
    ambient: Ambient,
) -> Result<(Society, EventEnvelope), CreateError> {
    if state.is_some() {
        return Err(CreateError::AlreadyExists);
    }

    let founder = match &cmd.actor {
        Principal::Citizen { fnid } => *fnid,
        Principal::Agent { .. } => return Err(CreateError::NotAHuman("an Agent")),
        Principal::Society { .. } => return Err(CreateError::NotAHuman("a Society")),
        Principal::Node { .. } => return Err(CreateError::NotAHuman("a Node")),
        Principal::System => return Err(CreateError::NotAHuman("the System")),
    };

    let trimmed = cmd.name.trim();
    if trimmed.is_empty() {
        return Err(CreateError::NameBlank);
    }
    let len = trimmed.chars().count();
    if len > NAME_MAX {
        return Err(CreateError::NameLength(len));
    }

    let first_hearth = FIRST_HEARTH_IS_FREE && cmd.societies_founded == 0;
    if !first_hearth && cmd.founder_level < SECOND_SOCIETY_LEVEL {
        return Err(CreateError::LevelRequired {
            level: cmd.founder_level,
            founded: cmd.societies_founded,
        });
    }

    let society = Society {
        society_id: ambient.society_id,
        name: trimmed.to_owned(),
        handle: cmd.handle.clone(),
        founder,
        visibility: cmd.visibility,
        // A Society is Active from its first event: it has one member, its
        // founder, which satisfies `member_count >= 1` (`docs/11 §2.2`).
        status: Status::Active,
        created_at: ambient.now,
        member_count: 1,
    };

    let payload = serde_json::json!({
        "society_id": society.society_id,
        "name": society.name,
        "handle": society.handle,
        "founder": society.founder,
        "visibility": society.visibility,
        "status": society.status,
        "origin": if first_hearth { "first_hearth" } else { "founded" },
    });

    let event = EventEnvelope {
        society_id: ambient.society_id,
        event_id: ambient.event_id,
        kind: EVENT_SOCIETY_CREATED,
        schema_version: 1,
        occurred_at: ambient.now,
        actor: cmd.actor.clone(),
        on_behalf_of: None,
        envelope_ref: None,
        correlation_id: ambient.correlation_id,
        causation_id: None,
        payload,
    };

    Ok((society, event))
}

/// Rebuild a Society from its log. The projection is disposable; this is the truth (P6).
///
/// # Errors
/// Returns the sequence number of the first event that could not be applied.
pub fn replay<'a>(
    events: impl IntoIterator<Item = (Seq, &'a EventEnvelope)>,
) -> Result<Option<Society>, Seq> {
    let mut state: Option<Society> = None;
    for (seq, e) in events {
        if e.kind == EVENT_SOCIETY_CREATED {
            let name = e
                .payload
                .get("name")
                .and_then(serde_json::Value::as_str)
                .ok_or(seq)?;
            let handle_raw = e
                .payload
                .get("handle")
                .and_then(serde_json::Value::as_str)
                .ok_or(seq)?;
            let handle = Handle::parse(handle_raw).map_err(|_| seq)?;
            let founder_raw = e
                .payload
                .get("founder")
                .and_then(serde_json::Value::as_str)
                .ok_or(seq)?;
            let founder: Fnid = founder_raw.parse().map_err(|_| seq)?;
            let visibility: Visibility = e
                .payload
                .get("visibility")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .ok_or(seq)?;
            state = Some(Society {
                society_id: e.society_id,
                name: name.to_owned(),
                handle,
                founder,
                visibility,
                status: Status::Active,
                created_at: e.occurred_at,
                member_count: 1,
            });
        }
    }
    Ok(state)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn ambient() -> Ambient {
        Ambient {
            society_id: SocietyId::new(Ulid::from_u128(1)),
            event_id: Ulid::from_u128(2),
            correlation_id: Ulid::from_u128(3),
            now: Timestamp::from_millis(1_000),
        }
    }

    fn cmd() -> CreateSociety {
        CreateSociety {
            actor: Principal::Citizen {
                fnid: Fnid::sample(1),
            },
            name: "Oracle Hall".to_owned(),
            handle: Handle::parse("oracle_hall").unwrap(),
            visibility: Visibility::Discoverable,
            societies_founded: 0,
            founder_level: 0,
        }
    }

    #[test]
    fn a_level_zero_citizen_can_found_their_first_society() {
        // This is docs/02 §2's spine sentence and docs/50 PH1's three-minute
        // acceptance criterion. If this test fails, the product does not work.
        let (s, e) = create(None, &cmd(), ambient()).expect("first hearth must be free");
        assert_eq!(s.status, Status::Active);
        assert_eq!(s.member_count, 1);
        assert_eq!(e.kind, EVENT_SOCIETY_CREATED);
        assert_eq!(
            e.payload.get("origin").and_then(|v| v.as_str()),
            Some("first_hearth")
        );
    }

    #[test]
    fn a_second_society_requires_level_three() {
        let mut c = cmd();
        c.societies_founded = 1;
        c.founder_level = 2;
        assert_eq!(
            create(None, &c, ambient()).unwrap_err(),
            CreateError::LevelRequired {
                level: 2,
                founded: 1
            }
        );
        c.founder_level = 3;
        assert!(create(None, &c, ambient()).is_ok());
    }

    #[test]
    fn an_agent_cannot_found_a_society() {
        // P4: policy is defined by humans. Founding a Society creates a Charter,
        // and only a Citizen may sign one.
        let mut c = cmd();
        c.actor = Principal::Agent {
            fnid: Fnid::sample(2),
            operator: Fnid::sample(1),
        };
        assert!(matches!(
            create(None, &c, ambient()),
            Err(CreateError::NotAHuman(_))
        ));
    }

    #[test]
    fn names_are_trimmed_and_bounded() {
        let mut c = cmd();
        c.name = "   ".to_owned();
        assert_eq!(
            create(None, &c, ambient()).unwrap_err(),
            CreateError::NameBlank
        );
        c.name = "x".repeat(NAME_MAX + 1);
        assert!(matches!(
            create(None, &c, ambient()),
            Err(CreateError::NameLength(_))
        ));
        c.name = "  Oracle Hall  ".to_owned();
        assert_eq!(create(None, &c, ambient()).unwrap().0.name, "Oracle Hall");
    }

    #[test]
    fn state_is_reproducible_from_the_log() {
        // docs/11 §7 invariant 10: every projection is reproducible by replaying
        // the log from zero. This is that invariant, at aggregate scale.
        let (direct, event) = create(None, &cmd(), ambient()).unwrap();
        let replayed = replay([(Seq::FIRST, &event)]).unwrap().expect("state");
        assert_eq!(direct, replayed);
    }

    #[test]
    fn creating_twice_is_rejected() {
        let (s, _) = create(None, &cmd(), ambient()).unwrap();
        assert_eq!(
            create(Some(&s), &cmd(), ambient()).unwrap_err(),
            CreateError::AlreadyExists
        );
    }
}
