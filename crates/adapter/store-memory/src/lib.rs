//! An `EventStore` held entirely in memory.
//!
//! This is not a toy. It is the reference implementation: the JSONL store is
//! tested for behavioural equivalence against it, which is how we know the port
//! boundary is real rather than shaped around one implementation (P5).

use fractal_ports::{AppendError, EventEnvelope, EventStore, ReadError, Seq, StoredEvent};
use fractal_types::{SocietyId, Timestamp};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

/// A clock the store uses to stamp `recorded_at`.
///
/// Taken as a closure rather than the `Clock` port so this crate stays in the
/// `adapter` layer without depending on anything that could drag domain types in.
type NowFn = Arc<dyn Fn() -> Timestamp + Send + Sync>;

pub struct MemoryEventStore {
    logs: RwLock<BTreeMap<SocietyId, Vec<StoredEvent>>>,
    sealed: RwLock<BTreeMap<SocietyId, bool>>,
    now: NowFn,
}

impl std::fmt::Debug for MemoryEventStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryEventStore").finish_non_exhaustive()
    }
}

impl MemoryEventStore {
    #[must_use]
    pub fn new(now: NowFn) -> Self {
        Self {
            logs: RwLock::new(BTreeMap::new()),
            sealed: RwLock::new(BTreeMap::new()),
            now,
        }
    }

    /// Seal a Society: its log length never increases again (`docs/11 §7` invariant 14).
    pub fn seal(&self, society_id: SocietyId) {
        if let Ok(mut s) = self.sealed.write() {
            s.insert(society_id, true);
        }
    }

    fn is_sealed(&self, society_id: SocietyId) -> bool {
        self.sealed
            .read()
            .is_ok_and(|s| s.get(&society_id).copied().unwrap_or(false))
    }
}

// `log.len()` and the enumerate index are bounded by memory, and both are
// widened to u64 rather than narrowed. No truncation is reachable.
#[allow(clippy::cast_possible_truncation)]
impl EventStore for MemoryEventStore {
    fn append(
        &self,
        society_id: SocietyId,
        expected_seq: Seq,
        events: Vec<EventEnvelope>,
    ) -> Result<Vec<StoredEvent>, AppendError> {
        if self.is_sealed(society_id) {
            return Err(AppendError::Sealed { society_id });
        }
        let mut logs = self
            .logs
            .write()
            .map_err(|_| AppendError::Unavailable("store lock poisoned".to_owned()))?;
        let log = logs.entry(society_id).or_default();
        let head = Seq::new(log.len() as u64 + 1);
        if head != expected_seq {
            return Err(AppendError::Conflict {
                society_id,
                expected: expected_seq,
                actual: head,
            });
        }
        let recorded_at = (self.now)();
        let mut out = Vec::with_capacity(events.len());
        for (i, envelope) in events.into_iter().enumerate() {
            let seq = Seq::new(head.get() + i as u64);
            let stored = StoredEvent {
                seq,
                recorded_at,
                envelope,
            };
            log.push(stored.clone());
            out.push(stored);
        }
        Ok(out)
    }

    fn read(
        &self,
        society_id: SocietyId,
        from: Seq,
        limit: usize,
    ) -> Result<Vec<StoredEvent>, ReadError> {
        let logs = self
            .logs
            .read()
            .map_err(|_| ReadError::Unavailable("store lock poisoned".to_owned()))?;
        let Some(log) = logs.get(&society_id) else {
            return Ok(Vec::new());
        };
        // usize is 32-bit on wasm32 (N2), so this conversion is checked rather
        // than cast: a log position beyond usize on that target is a read of
        // nothing, not a silent wrap to the beginning.
        let Ok(start) = usize::try_from(from.get().saturating_sub(1)) else {
            return Ok(Vec::new());
        };
        Ok(log.iter().skip(start).take(limit).cloned().collect())
    }

    fn head(&self, society_id: SocietyId) -> Result<Seq, ReadError> {
        let logs = self
            .logs
            .read()
            .map_err(|_| ReadError::Unavailable("store lock poisoned".to_owned()))?;
        Ok(Seq::new(
            logs.get(&society_id).map_or(0, Vec::len) as u64 + 1,
        ))
    }

    fn societies(&self) -> Result<Vec<SocietyId>, ReadError> {
        let logs = self
            .logs
            .read()
            .map_err(|_| ReadError::Unavailable("store lock poisoned".to_owned()))?;
        Ok(logs.keys().copied().collect())
    }
}
