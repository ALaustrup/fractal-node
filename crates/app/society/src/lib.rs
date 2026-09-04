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

/// What the founding rule is allowed to read about a Citizen.
///
/// Derived here, never accepted from the caller. The distinction is the whole
/// point: `societies_founded` is the input to the first-hearth gate, so a
/// caller that can set it can mint Societies without limit. It is computed from
/// the event log — state the caller cannot write except by going through this
/// same gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    /// Societies this Citizen has already founded, counted from the log.
    pub societies_founded: u32,
    /// Level, from the XP projection. PH0 has no XP subsystem (`docs/03`), so
    /// this is 0 for everyone. Zero is the safe value: a level that is wrongly
    /// LOW refuses a founding that should have been allowed, which is visible
    /// and recoverable. A level that is wrongly HIGH mints Societies. PH2
    /// replaces the body of `Self::level_of` with a projection read and this
    /// signature does not change.
    pub founder_level: u16,
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

        // 3. Standing is READ, not received. See `Standing`.
        let standing = self.standing_of(&req.actor)?;

        // 4. The domain decides. It sees no clock, no store and no transport.
        let cmd = domain::CreateSociety {
            actor: req.actor.clone(),
            name: req.name.clone(),
            handle: req.handle.clone(),
            visibility: req.visibility,
            societies_founded: standing.societies_founded,
            founder_level: standing.founder_level,
        };
        let (society, event) = domain::create(None, &cmd, ambient)?;
        let correlation_id = event.correlation_id;

        // 5. Append. A conflict here is a real conflict: this Society is new, so
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

    /// Read this Citizen's standing from the log.
    ///
    /// # Errors
    /// See [`ServiceError`].
    ///
    /// PH0 counts by scanning every Society on the Node, which is the same
    /// fan-out `list` documents and the same one `docs/61`'s S15 Atlas exists
    /// to own from PH1. Correct and slow beats fast and wrong at this size.
    pub fn standing_of(&self, actor: &Principal) -> Result<Standing, ServiceError> {
        let Principal::Citizen { fnid } = actor else {
            // Only a Citizen can found (the domain enforces this too). Anyone
            // else gets a standing that founds nothing.
            return Ok(Standing {
                societies_founded: 0,
                founder_level: 0,
            });
        };
        let who = fnid.to_string();

        // Count FOUNDINGS, not surviving Societies. `docs/11 §5` is explicit:
        // the first-hearth allowance is "consumed at SocietyCreated and is not
        // restored if that Society is later Dissolved, Archived, or the founder
        // departs — a renewable first-hearth allowance is a renewable Sybil
        // resource." Counting live Societies would restore it on dissolution,
        // which is the exact exploit that sentence exists to close. So the
        // count reads the creation EVENT, which no later event can retract.
        let mut founded: u32 = 0;
        for id in self
            .store
            .societies()
            .map_err(|source| ServiceError::Read {
                society_id: SocietyId::new(fractal_types::Ulid::from_u128(0)),
                source,
            })?
        {
            let first =
                self.store
                    .read(id, Seq::FIRST, 1)
                    .map_err(|source| ServiceError::Read {
                        society_id: id,
                        source,
                    })?;
            let founded_here = first.first().is_some_and(|e| {
                e.envelope.kind == domain::EVENT_SOCIETY_CREATED
                    && e.envelope.payload.get("founder").and_then(|v| v.as_str())
                        == Some(who.as_str())
            });
            if founded_here {
                founded = founded.saturating_add(1);
            }
        }
        Ok(Standing {
            societies_founded: founded,
            founder_level: Self::level_of(fnid),
        })
    }

    /// PH2 replaces this body with a read of the XP projection.
    const fn level_of(_fnid: &fractal_types::Fnid) -> u16 {
        0
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
        r.handle = Handle::parse("second_hall").unwrap();
        assert!(matches!(w.svc.create(&r), Err(ServiceError::Rejected(_))));
        assert_eq!(
            w.svc.list().unwrap().len(),
            1,
            "a refused founding must write nothing"
        );
    }

    /// The regression this exists to prevent: standing used to arrive in the
    /// request body, so a caller could hand itself `societies_founded: 0`
    /// forever and mint Societies without limit. There is now no field to set.
    /// This test asserts the derivation instead — the count the gate reads must
    /// track the log, for this Citizen and no other.
    #[test]
    fn standing_is_read_from_the_log_not_the_caller() {
        let w = world();
        let me = Principal::Citizen {
            fnid: Fnid::sample(1),
        };
        let someone_else = Principal::Citizen {
            fnid: Fnid::sample(2),
        };

        assert_eq!(w.svc.standing_of(&me).unwrap().societies_founded, 0);
        w.svc.create(&req()).unwrap();
        assert_eq!(
            w.svc.standing_of(&me).unwrap().societies_founded,
            1,
            "founding must move MY count"
        );
        assert_eq!(
            w.svc.standing_of(&someone_else).unwrap().societies_founded,
            0,
            "and nobody else's"
        );
    }

    /// PH0 has no XP subsystem, so everyone is Level 0. Stated as a test so the
    /// day someone wires up XP, this fails and forces the seam to be revisited
    /// rather than silently granting levels the projection never awarded.
    #[test]
    fn ph0_grants_nobody_a_level() {
        let w = world();
        let me = Principal::Citizen {
            fnid: Fnid::sample(1),
        };
        assert_eq!(w.svc.standing_of(&me).unwrap().founder_level, 0);
    }

    #[test]
    fn reading_an_unknown_society_is_none() {
        let w = world();
        let missing = SocietyId::new(fractal_types::Ulid::from_u128(999));
        assert!(w.svc.get(missing).unwrap().is_none());
    }
}
