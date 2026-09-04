//! The append-only, per-Society event log (P1, P6, `docs/10 §5`).
//!
//! The critical property encoded here: ordering is **per Society**, never global.
//! `Seq` is scoped to one `society_id`. There is no global sequence number in
//! this interface and there must never be one — removing global consensus from
//! the hot path is the single most important scalability decision in `docs/10`.

use fractal_types::{Principal, SocietyId, Timestamp, Ulid};
use std::borrow::Cow;

/// A monotonic position within ONE Society's log. Starts at 1.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct Seq(u64);

impl Seq {
    pub const FIRST: Self = Self(1);

    #[must_use]
    pub const fn new(v: u64) -> Self {
        Self(v)
    }
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl std::fmt::Display for Seq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The versioned name of an event, e.g. `society.created.v1`.
///
/// Events are never rewritten (`docs/10 §5`). A breaking change is a new `.v2`
/// kind plus an upcaster in the replay path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct EventKind(Cow<'static, str>);

impl EventKind {
    #[must_use]
    pub const fn from_static(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An event as offered to the log, before it has a position.
///
/// `envelope_ref` is the field that makes P4 auditable rather than aspirational:
/// every action an Agent takes names the grant that permitted it, and every grant
/// traces to a human signature. It arrives in PH3 with the Envelope system; the
/// field exists from PH0 so the shape of the log never has to change.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventEnvelope {
    pub society_id: SocietyId,
    pub event_id: Ulid,
    pub kind: EventKind,
    pub schema_version: u16,
    /// Domain time: when the thing happened.
    pub occurred_at: Timestamp,
    pub actor: Principal,
    /// Set when an Agent acts for a Citizen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<Principal>,
    /// Which Envelope authorised this. `None` until PH3 (P4 audit chain).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope_ref: Option<Ulid>,
    /// The user-visible operation this belongs to.
    pub correlation_id: Ulid,
    /// The event or command that caused this one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<Ulid>,
    /// The event body, already serialised. The log does not interpret it.
    pub payload: serde_json::Value,
}

/// An event that has been accepted into the log and given a position.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredEvent {
    pub seq: Seq,
    /// Wall time: when the Runtime recorded it. Distinct from `occurred_at`,
    /// and never conflated with it (`docs/10 §10`).
    pub recorded_at: Timestamp,
    #[serde(flatten)]
    pub envelope: EventEnvelope,
}

#[derive(Debug, thiserror::Error)]
pub enum AppendError {
    /// Someone else wrote to this Society's log first. The caller reloads and retries.
    #[error("optimistic concurrency conflict on {society_id}: expected seq {expected}, log is at {actual}")]
    Conflict {
        society_id: SocietyId,
        expected: Seq,
        actual: Seq,
    },
    /// The Society is Sealed: its log length never increases (`docs/11 §7` invariant 14).
    #[error("{society_id} is sealed — its log cannot grow")]
    Sealed { society_id: SocietyId },
    #[error("event store unavailable: {0}")]
    Unavailable(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("event store unavailable: {0}")]
    Unavailable(String),
    #[error("log for {society_id} is corrupt at seq {seq}: {detail}")]
    Corrupt {
        society_id: SocietyId,
        seq: Seq,
        detail: String,
    },
}

/// The append-only log.
///
/// Deliberately synchronous. PH0 has no async runtime in the domain or app
/// layers, and making this async later is a mechanical change confined to the
/// adapter and app crates — whereas making it sync again would not be.
pub trait EventStore: Send + Sync + 'static {
    /// Append events to one Society's log.
    ///
    /// `expected_seq` is the position the caller believes the log is at; passing
    /// [`Seq::FIRST`] means "this Society's log is empty". A mismatch is a
    /// [`AppendError::Conflict`], never a silent overwrite.
    ///
    /// # Errors
    /// See [`AppendError`].
    fn append(
        &self,
        society_id: SocietyId,
        expected_seq: Seq,
        events: Vec<EventEnvelope>,
    ) -> Result<Vec<StoredEvent>, AppendError>;

    /// Read one Society's log from `from` (inclusive), at most `limit` events.
    ///
    /// # Errors
    /// See [`ReadError`].
    fn read(
        &self,
        society_id: SocietyId,
        from: Seq,
        limit: usize,
    ) -> Result<Vec<StoredEvent>, ReadError>;

    /// The next position this Society's log will assign.
    ///
    /// # Errors
    /// See [`ReadError`].
    fn head(&self, society_id: SocietyId) -> Result<Seq, ReadError>;

    /// Every Society this store holds a log for.
    ///
    /// Present for the PH0 walking skeleton and for operational tooling. It is
    /// explicitly NOT the cross-Society read path — that is S15 Atlas
    /// (`docs/61`), a separate, eventually-consistent, read-only projection.
    ///
    /// # Errors
    /// See [`ReadError`].
    fn societies(&self) -> Result<Vec<SocietyId>, ReadError>;
}
