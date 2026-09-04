//! Ports: the complete swappable surface (P5, `docs/10 §7`).
//!
//! Every trait here has at least two implementations at the moment it is created
//! — typically the real one and a deterministic test double. That is not a
//! convention; it is what makes the abstraction real rather than aspirational.
//!
//! Nothing in this crate performs I/O. Nothing in this crate names a vendor.

mod ambient;
mod event_store;

pub use ambient::{Clock, IdGen, Rng};
pub use event_store::{
    AppendError, EventEnvelope, EventKind, EventStore, ReadError, Seq, StoredEvent,
};
