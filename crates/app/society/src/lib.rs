//! Society command and query handlers.
//!
//! The handler is the seam where the pure domain meets the world: it authorises,
//! loads state by replaying the log, calls the domain for a decision, and appends
//! whatever the domain returned. It contains no business rules of its own — if a
//! rule appears here, it is in the wrong crate.

use fractal_app_kernel::{ActionClass, CommandContext, IdempotencyKey, PolicyDenied};
use fractal_domain_society as domain;
use fractal_ports::{AppendError, EventStore, ReadError, Seq};
use fractal_types::{Handle, Principal, SocietyId, Visibility};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Denied(#[from] PolicyDenied),
    #[error(transparent)]
    Rejected(#[from] domain::CreateError),
    #[error("the log for {society_id} is unreadable: {source}")]
    Read {
        society_id: SocietyId,
        #[source]
        source: ReadError,
    },
    #[error(transparent)]
    Append(#[from] AppendError),
    #[error("log for {society_id} is corrupt at seq {seq} and cannot be replayed")]
    Corrupt { society_id: SocietyId, seq: Seq },
}

/// What a caller asks for. Deliberately not the domain command: the transport
/// shape and the domain shape are allowed to diverge, and this is the seam.
#[derive(Debug, Clone)]
pub struct CreateSocietyRequest {
    pub actor: Principal,
    pub name: String,
    pub handle: Handle,
    pub visibility: Visibility,
    pub idempotency_key: Option<IdempotencyKey>,
    /// Supplied by the caller in PH0. From PH1 this is read from the Citizen
    /// projection; the handler signature does not change when it is.
    pub societies_founded: u32,
    pub founder_level: u16,
}

/// A Society as read back. A projection, and therefore disposable (P6).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SocietyView {
    pub society_id: SocietyId,
    pub name: String,
    pub handle: String,
    pub founder: String,
    pub visibility: Visibility,
    pub status: domain::Status,
    pub member_count: u32,
    pub created_at: i64,
    pub seq: u64,
}

impl SocietyView {
    fn of(s: &domain::Society, seq: Seq) -> Self {
        Self {
            society_id: s.society_id,
            name: s.name.clone(),
            handle: s.handle.to_string(),
            founder: s.founder.to_string(),
            visibility: s.visibility,
            status: s.status,
            member_count: s.member_count,
            created_at: s.created_at.as_millis(),
            seq: seq.get(),
        }
    }
}

pub struct SocietyService {
    store: Arc<dyn EventStore>,
    ctx: Arc<CommandContext>,
}

impl std::fmt::Debug for SocietyService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocietyService").finish_non_exhaustive()
    }
}

impl SocietyService {
    #[must_use]
    pub fn new(store: Arc<dyn EventStore>, ctx: Arc<CommandContext>) -> Self {
        Self { store, ctx }
    }

    /// Found a Society.
    ///
    /// # Errors
    /// See [`ServiceError`].
    pub fn create(&self, req: &CreateSocietyRequest) -> Result<SocietyView, ServiceError> {
        // 1. Policy first, always, and inside the trust boundary (docs/10 §8).
        //    Founding a Society enacts a Charter, which is a Policy action.
        self.ctx.pep.authorise(&req.actor, ActionClass::Policy)?;

        let now = self.ctx.now();

        // 2. Idempotency: a retried command returns the original outcome.
        if let Some(key) = &req.idempotency_key {
            if let Some(prior) = self.ctx.idempotency.seen(&req.actor, key, now) {
                if let Some(view) = self.find_by_correlation(prior)? {
                    return Ok(view);
                }
            }
        }

        let society_id = SocietyId::new(self.ctx.next_id());
        let ambient = domain::Ambient {
            society_id,
            event_id: self.ctx.next_id(),
            correlation_id: self.ctx.next_id(),
            now,
        };

        // 3. The domain decides. It sees no clock, no store and no transport.
        let cmd = domain::CreateSociety {
            actor: req.actor.clone(),
            name: req.name.clone(),
            handle: req.handle.clone(),
            visibility: req.visibility,
            societies_founded: req.societies_founded,
            founder_level: req.founder_level,
        };
        let (society, event) = domain::create(None, &cmd, ambient)?;
        let correlation_id = event.correlation_id;

        // 4. Append. A conflict here is a real conflict: this Society is new, so
        //    its log must be empty.
        let stored = self.store.append(society_id, Seq::FIRST, vec![event])?;
        let seq = stored.first().map_or(Seq::FIRST, |e| e.seq);

        if let Some(key) = &req.idempotency_key {
            self.ctx
                .idempotency
                .record(&req.actor, key, correlation_id, now);
        }

        Ok(SocietyView::of(&society, seq))
    }

    /// Read one Society by replaying its log.
    ///
    /// # Errors
    /// See [`ServiceError`].
    pub fn get(&self, society_id: SocietyId) -> Result<Option<SocietyView>, ServiceError> {
        let events = self
            .store
            .read(society_id, Seq::FIRST, 10_000)
            .map_err(|source| ServiceError::Read { society_id, source })?;
        if events.is_empty() {
            return Ok(None);
        }
        let head = events.last().map_or(Seq::FIRST, |e| e.seq);
        let state = domain::replay(events.iter().map(|e| (e.seq, &e.envelope)))
            .map_err(|seq| ServiceError::Corrupt { society_id, seq })?;
        Ok(state.as_ref().map(|s| SocietyView::of(s, head)))
    }

    /// Every Society this Node holds.
    ///
    /// PH0 only. This is a fan-out read across partitions, which is exactly what
    /// `docs/61`'s S15 Atlas exists to own from PH1 onward. It is here so the
    /// walking skeleton has something to list, and it is marked so it is not
    /// mistaken for the real cross-Society read path.
    ///
    /// # Errors
    /// See [`ServiceError`].
    pub fn list(&self) -> Result<Vec<SocietyView>, ServiceError> {
        let ids = self
            .store
            .societies()
            .map_err(|source| ServiceError::Read {
                society_id: SocietyId::new(fractal_types::Ulid::from_u128(0)),
                source,
            })?;
        let mut out = Vec::new();
        for id in ids {
            if let Some(v) = self.get(id)? {
                out.push(v);
            }
        }
        out.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then(a.society_id.cmp(&b.society_id))
        });
        Ok(out)
    }

    fn find_by_correlation(
        &self,
        correlation_id: fractal_types::Ulid,
    ) -> Result<Option<SocietyView>, ServiceError> {
        for view in self.list()? {
            let events = self
                .store
                .read(view.society_id, Seq::FIRST, 16)
                .map_err(|source| ServiceError::Read {
                    society_id: view.society_id,
                    source,
                })?;
            if events
                .iter()
                .any(|e| e.envelope.correlation_id == correlation_id)
            {
                return Ok(Some(view));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use fractal_adapter_store_memory::MemoryEventStore;
    use fractal_testkit::Deterministic;
    use fractal_types::{Fnid, Timestamp};

    /// The deterministic world every test runs in. Same seed, same history,
    /// every time — which is what lets these assertions mean anything (ADR-0014).
    struct World {
        svc: SocietyService,
    }

    struct DetClock(Arc<Deterministic>);
    impl fractal_ports::Clock for DetClock {
        fn now(&self) -> Timestamp {
            self.0.clock.now()
        }
    }

    struct DetIds(Arc<Deterministic>);
    impl fractal_ports::IdGen for DetIds {
        fn next_ulid(&self) -> fractal_types::Ulid {
            self.0.ids.next_ulid()
        }
    }

    fn world() -> World {
        let det = Arc::new(Deterministic::seeded(7));
        let store = Arc::new(MemoryEventStore::new(Arc::new(|| {
            Timestamp::from_millis(1_700_000_000_000)
        })));
        let ctx = Arc::new(CommandContext::new(
            Arc::new(DetClock(Arc::clone(&det))),
            Arc::new(DetIds(det)),
        ));
        World {
            svc: SocietyService::new(store, ctx),
        }
    }

    fn req() -> CreateSocietyRequest {
        CreateSocietyRequest {
            actor: Principal::Citizen {
                fnid: Fnid::sample(1),
            },
            name: "Oracle Hall".to_owned(),
            handle: Handle::parse("oracle_hall").unwrap(),
            visibility: Visibility::Discoverable,
            idempotency_key: None,
            societies_founded: 0,
            founder_level: 0,
        }
    }

    #[test]
    fn create_then_read_back() {
        let w = world();
        let created = w.svc.create(&req()).unwrap();
        let read = w.svc.get(created.society_id).unwrap().expect("society");
        assert_eq!(
            created, read,
            "the projection must equal a fresh replay (P6)"
        );
        assert_eq!(w.svc.list().unwrap().len(), 1);
    }

    #[test]
    fn an_agent_is_refused_by_the_pep_before_the_domain_is_reached() {
        let w = world();
        let mut r = req();
        r.actor = Principal::Agent {
            fnid: Fnid::sample(2),
            operator: Fnid::sample(1),
        };
        assert!(matches!(w.svc.create(&r), Err(ServiceError::Denied(_))));
        assert!(
            w.svc.list().unwrap().is_empty(),
            "a denied command must write nothing"
        );
    }

    #[test]
    fn a_retried_command_creates_one_society_not_two() {
        let w = world();
        let mut r = req();
        r.idempotency_key = Some(IdempotencyKey::new("retry-me"));
        let a = w.svc.create(&r).unwrap();
        let b = w.svc.create(&r).unwrap();
        assert_eq!(
            a.society_id, b.society_id,
            "a retry must be a no-op, not a second Society"
        );
        assert_eq!(w.svc.list().unwrap().len(), 1);
    }

    #[test]
    fn a_second_society_needs_level_three() {
        let w = world();
        w.svc.create(&req()).unwrap();
        let mut r = req();
        r.societies_founded = 1;
        r.founder_level = 1;
        r.handle = Handle::parse("second_hall").unwrap();
        assert!(matches!(w.svc.create(&r), Err(ServiceError::Rejected(_))));
    }

    #[test]
    fn reading_an_unknown_society_is_none() {
        let w = world();
        let missing = SocietyId::new(fractal_types::Ulid::from_u128(999));
        assert!(w.svc.get(missing).unwrap().is_none());
    }
}
