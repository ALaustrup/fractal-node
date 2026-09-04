//! The simulation harness (M0.6, ADR-0014).
//!
//! `docs/50 PH0` acceptance criterion 6: *"The simulation harness runs 2,000
//! seeded histories and asserts three invariants."*
//!
//! ## What this is, and why it is not just more unit tests
//!
//! A unit test asserts that a case a human thought of behaves correctly. This
//! asserts that **properties hold over histories nobody thought of** — thousands
//! of randomly ordered command sequences, including ones that fail, retry, race
//! and interleave across Societies.
//!
//! It only works because every ambient input is a port: the clock, the RNG and
//! the id generator all come from `fractal-testkit`, so a whole history is a
//! pure function of one `u64`. A failure is therefore never "it broke sometimes"
//! — it is "seed 41 fails at step 380", which reproduces exactly, forever. That
//! is the entire justification for ADR-0014, and this file is where the bill for
//! it gets paid back.
//!
//! ## Scale
//!
//! Defaults to 200 histories so `cargo test` stays fast. `cargo xtask sim` runs
//! the full 2,000 that the acceptance criterion names, and CI runs that.
//! Override with `FRACTAL_SIM_HISTORIES`.

// A simulation harness is test code that reports to a human: it panics on a
// violation (that IS the signal), prints its coverage, and is one long
// dispatch by construction. The workspace lints are written for production
// paths; these four are relaxed here deliberately rather than by accident.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::too_many_lines,
    clippy::needless_pass_by_value
)]

use fractal_adapter_store_memory::MemoryEventStore;
use fractal_app_kernel::{CommandContext, IdempotencyKey};
use fractal_app_society::{CreateSocietyRequest, ServiceError, SocietyService};
use fractal_ports::{Clock, EventStore, IdGen, Seq};
use fractal_testkit::{Deterministic, Gen};
use fractal_types::{Fnid, Handle, Principal, SocietyId, Timestamp, Ulid, Visibility};
use std::collections::BTreeMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Wiring
// ---------------------------------------------------------------------------

struct DetClock(Arc<Deterministic>);
impl Clock for DetClock {
    fn now(&self) -> Timestamp {
        self.0.clock.now()
    }
}
struct DetIds(Arc<Deterministic>);
impl IdGen for DetIds {
    fn next_ulid(&self) -> Ulid {
        self.0.ids.next_ulid()
    }
}

/// One simulated world.
struct World {
    svc: SocietyService,
    store: Arc<MemoryEventStore>,
    det: Arc<Deterministic>,
}

fn world(seed: u64) -> World {
    let det = Arc::new(Deterministic::seeded(seed));
    let clock = Arc::new(DetClock(Arc::clone(&det)));
    let now = {
        let c = Arc::clone(&clock);
        Arc::new(move || c.now())
    };
    let store = Arc::new(MemoryEventStore::new(now));
    let ctx = Arc::new(CommandContext::new(
        clock,
        Arc::new(DetIds(Arc::clone(&det))),
    ));
    let svc = SocietyService::new(Arc::clone(&store) as Arc<dyn EventStore>, ctx);
    World { svc, store, det }
}

// ---------------------------------------------------------------------------
// The history
// ---------------------------------------------------------------------------

/// What the simulation believes is true, maintained independently of the system
/// under test. Comparing the two is the point: a model that merely re-implements
/// the code cannot catch the code being wrong.
#[derive(Default)]
struct Model {
    /// Societies we successfully created, and the Principal that founded each.
    created: BTreeMap<SocietyId, Principal>,
    /// Idempotency key → the Society it produced.
    by_key: BTreeMap<String, SocietyId>,
    /// Societies we sealed. Their logs must never grow again.
    sealed: BTreeMap<SocietyId, u64>,
    /// How many Societies each Citizen has founded, for the first-hearth rule.
    founded_by: BTreeMap<String, u32>,
}

#[derive(Debug)]
enum Step {
    CreateValid { idem: Option<String> },
    CreateBadName,
    CreateBadHandle,
    CreateAsAgent,
    ReadOne,
    ReadMissing,
    List,
    Seal,
    CreateInSealed,
}

fn next_step(g: &Gen<'_>, model: &Model) -> Step {
    // Weighted so that most steps do real work, with a steady drip of the
    // failure paths — a simulation that only exercises the happy path is a
    // slower unit test.
    match g.below(100) {
        0..=34 => Step::CreateValid {
            idem: g.chance(40).then(|| format!("k{}", g.below(6))),
        },
        35..=41 => Step::CreateBadName,
        42..=48 => Step::CreateBadHandle,
        49..=54 => Step::CreateAsAgent,
        55..=74 => {
            if model.created.is_empty() {
                Step::List
            } else {
                Step::ReadOne
            }
        }
        75..=79 => Step::ReadMissing,
        80..=92 => Step::List,
        93..=95 => Step::Seal,
        _ => Step::CreateInSealed,
    }
}

/// Run one seeded history and assert every invariant after every step.
///
/// Returns the number of steps executed, so the caller can report coverage
/// rather than just "it passed".
fn run_history(seed: u64, steps: usize) -> usize {
    let w = world(seed);
    let rng = &w.det.rng;
    let g = Gen::new(rng);
    let mut model = Model::default();

    for step_no in 0..steps {
        let step = next_step(&g, &model);
        // Rendered eagerly: the closure would borrow `step`, and the branches
        // below need to move out of it.
        let label = format!("seed {seed}, step {step_no}, {step:?}");
        let ctx = || label.clone();

        match step {
            Step::CreateValid { idem } => {
                let citizen = Fnid::sample(u8::try_from(g.below(6)).unwrap_or(0));
                let who = citizen.to_string();
                let founded = model.founded_by.get(&who).copied().unwrap_or(0);
                let req = CreateSocietyRequest {
                    actor: Principal::Citizen { fnid: citizen },
                    name: g.name(),
                    handle: Handle::parse(&g.handle()).expect("generator emits valid handles"),
                    visibility: *g.pick(&[
                        Visibility::Public,
                        Visibility::Discoverable,
                        Visibility::Private,
                    ]),
                    idempotency_key: idem.clone().map(IdempotencyKey::new),
                };

                // Standing is no longer something this harness can hand the
                // service — it is derived from the log. So the model's count
                // becomes the ORACLE for that derivation, checked before every
                // founding. If the derivation ever drifts from the history the
                // simulation actually produced, this is where it is caught.
                assert_eq!(
                    w.svc
                        .standing_of(&req.actor)
                        .expect("standing is readable")
                        .societies_founded,
                    founded,
                    "derived standing disagrees with the history: {}",
                    ctx()
                );

                // An idempotent REPLAY is not re-evaluated by the domain: it
                // returns the outcome the original call produced. The simulation
                // found this within 34 steps, and it is worth stating plainly
                // because it looks like a hole and is not one:
                //
                //   The PEP still runs on every attempt (authority is
                //   re-checked, so a revoked Envelope blocks a retry), but the
                //   business decision is replayed rather than re-made. That is
                //   what "safe to retry" has to mean — a retry that could
                //   produce a *different* answer than the call it is retrying
                //   is not idempotent, it is a second command wearing the same
                //   key.
                let replay_key = idem.as_ref().map(|k| format!("{who}/{k}"));
                let is_replay = replay_key
                    .as_ref()
                    .is_some_and(|k| model.by_key.contains_key(k));

                // PH0 awards no levels (`SocietyService::level_of`), so the
                // first hearth is the ONLY founding that can succeed. When XP
                // lands, this line gains the level term back and the harness
                // will be the thing that proves the gate still holds.
                let first_hearth = founded == 0;
                let should_succeed = is_replay || first_hearth;

                match w.svc.create(&req) {
                    Ok(v) => {
                        assert!(
                            should_succeed,
                            "created a Society that should have been refused: {}",
                            ctx()
                        );
                        // Idempotency: the same key must never mint a second Society.
                        if let Some(key) = replay_key {
                            if let Some(prior) = model.by_key.get(&key) {
                                assert_eq!(
                                    *prior,
                                    v.society_id,
                                    "I-IDEM: a retried command created a second Society: {}",
                                    ctx()
                                );
                            } else {
                                model.by_key.insert(key, v.society_id);
                                model.created.insert(v.society_id, req.actor.clone());
                                *model.founded_by.entry(who).or_insert(0) += 1;
                            }
                        } else {
                            model.created.insert(v.society_id, req.actor.clone());
                            *model.founded_by.entry(who).or_insert(0) += 1;
                        }
                    }
                    Err(ServiceError::Rejected(_)) => {
                        assert!(
                            !should_succeed,
                            "refused a Society that should have been allowed: {}",
                            ctx()
                        );
                    }
                    Err(e) => panic!("unexpected failure: {e} at {}", ctx()),
                }
            }

            Step::CreateBadName => {
                let (bad, why) = g.bad_name();
                let req = base_request(bad, g.handle());
                assert!(
                    matches!(w.svc.create(&req), Err(ServiceError::Rejected(_))),
                    "accepted a {why} name: {}",
                    ctx()
                );
            }

            Step::CreateBadHandle => {
                // Rejected before the service is reached — the Handle newtype is
                // the boundary, and this asserts it actually is one.
                let (bad, why) = g.bad_handle();
                assert!(
                    Handle::parse(&bad).is_err(),
                    "accepted a {why} handle: {}",
                    ctx()
                );
            }

            Step::CreateAsAgent => {
                // P4: policy is human-only. An Agent must never found a Society,
                // and must never leave a trace when it tries.
                let before = w.store.societies().unwrap().len();
                let mut req = base_request(g.name(), g.handle());
                req.actor = Principal::Agent {
                    fnid: Fnid::sample(200),
                    operator: Fnid::sample(1),
                };
                assert!(
                    matches!(w.svc.create(&req), Err(ServiceError::Denied(_))),
                    "P4: an Agent founded a Society: {}",
                    ctx()
                );
                assert_eq!(
                    w.store.societies().unwrap().len(),
                    before,
                    "P4: a denied command wrote to the log: {}",
                    ctx()
                );
            }

            Step::ReadOne => {
                let ids: Vec<_> = model.created.keys().copied().collect();
                if ids.is_empty() {
                    continue;
                }
                let id = *g.pick(&ids);
                let view = w
                    .svc
                    .get(id)
                    .unwrap()
                    .expect("a created Society must be readable");
                assert_eq!(
                    view.society_id,
                    id,
                    "read returned a different Society: {}",
                    ctx()
                );
            }

            Step::ReadMissing => {
                let ghost = SocietyId::new(Ulid::from_u128(u128::from(g.below(1_000)) + 900_000));
                assert!(
                    w.svc.get(ghost).unwrap().is_none(),
                    "a Society that was never created was readable: {}",
                    ctx()
                );
            }

            Step::List => {
                let listed = w.svc.list().unwrap();
                assert_eq!(
                    listed.len(),
                    model.created.len(),
                    "the Node lists a different number of Societies than were created: {}",
                    ctx()
                );
            }

            Step::Seal => {
                let ids: Vec<_> = model.created.keys().copied().collect();
                if ids.is_empty() {
                    continue;
                }
                let id = *g.pick(&ids);
                let len = w.store.read(id, Seq::FIRST, 10_000).unwrap().len() as u64;
                w.store.seal(id);
                model.sealed.insert(id, len);
            }

            Step::CreateInSealed => {
                // Actually TRY to grow a sealed log.
                //
                // The first draft of this step did nothing, on the reasoning
                // that the I-14 invariant below would catch a sealed log that
                // grew. It would have — but nothing ever made one grow, so the
                // assertion was inert. A deliberately broken `seal()` passed
                // 300 histories without a murmur. An invariant that is checked
                // but never exercised is decoration.
                let ids: Vec<_> = model.sealed.keys().copied().collect();
                if ids.is_empty() {
                    continue;
                }
                let id = *g.pick(&ids);
                let head = w.store.head(id).unwrap();
                let before = w.store.read(id, Seq::FIRST, 10_000).unwrap().len();

                let attempt = w.store.append(id, head, vec![probe_event(id, &w)]);
                assert!(
                    matches!(attempt, Err(fractal_ports::AppendError::Sealed { .. })),
                    "I-14: a sealed Society accepted an append: {}",
                    ctx()
                );
                assert_eq!(
                    w.store.read(id, Seq::FIRST, 10_000).unwrap().len(),
                    before,
                    "I-14: a sealed Society's log grew: {}",
                    ctx()
                );
            }
        }

        assert_invariants(&w, &model, step_no, seed);
    }
    steps
}

/// A well-formed event used only to prove a sealed log refuses it.
fn probe_event(society_id: SocietyId, w: &World) -> fractal_ports::EventEnvelope {
    fractal_ports::EventEnvelope {
        society_id,
        event_id: w.det.ids.next_ulid(),
        kind: fractal_ports::EventKind::from_static("test.probe.attempted.v1"),
        schema_version: 1,
        occurred_at: w.det.clock.now(),
        actor: Principal::Citizen {
            fnid: Fnid::sample(1),
        },
        on_behalf_of: None,
        envelope_ref: None,
        correlation_id: w.det.ids.next_ulid(),
        causation_id: None,
        payload: serde_json::json!({ "probe": true }),
    }
}

fn base_request(name: String, handle: String) -> CreateSocietyRequest {
    CreateSocietyRequest {
        actor: Principal::Citizen {
            fnid: Fnid::sample(1),
        },
        name,
        handle: Handle::parse(&handle).unwrap_or_else(|_| Handle::parse("fallback").unwrap()),
        visibility: Visibility::Discoverable,
        idempotency_key: None,
    }
}

// ---------------------------------------------------------------------------
// The invariants (docs/11 §7)
// ---------------------------------------------------------------------------

fn assert_invariants(w: &World, model: &Model, step: usize, seed: u64) {
    let where_ = format!("seed {seed}, after step {step}");

    for id in w.store.societies().unwrap() {
        let events = w.store.read(id, Seq::FIRST, 10_000).unwrap();

        // I-1 — every event names the Society that owns it (P1). The whole
        // partitioning strategy is built on this being true without exception.
        for e in &events {
            assert_eq!(
                e.envelope.society_id, id,
                "I-1: {} holds an event belonging to {} — {where_}",
                id, e.envelope.society_id
            );
        }

        // Per-Society ordering: seq is dense and starts at 1. A gap would mean
        // the log lost an event; a duplicate would mean it accepted one twice.
        for (i, e) in events.iter().enumerate() {
            assert_eq!(
                e.seq.get(),
                i as u64 + 1,
                "I-1: {id} has a non-dense sequence at index {i} — {where_}"
            );
        }

        // I-10 — every projection is reproducible by replaying the log from
        // zero. This is P6's whole claim, checked after every single step.
        let projected = w.svc.get(id).unwrap();
        if !events.is_empty() {
            let replayed = projected.as_ref().expect("a non-empty log must project");
            assert_eq!(
                replayed.society_id, id,
                "I-10: replay produced a different Society — {where_}"
            );
            assert!(
                replayed.member_count >= 1,
                "I-10: an Active Society with no members — {where_}"
            );
        }

        // I-14 — a Sealed Society's log length never increases.
        if let Some(len_at_seal) = model.sealed.get(&id) {
            assert_eq!(
                events.len() as u64,
                *len_at_seal,
                "I-14: a sealed Society's log grew — {where_}"
            );
        }
    }

    // P4 — no Society in any reachable state was founded by an Agent.
    for (id, actor) in &model.created {
        let events = w.store.read(*id, Seq::FIRST, 10).unwrap();
        for e in &events {
            assert!(
                e.envelope.actor.is_human(),
                "P4: {id} has an event authored by a non-human principal — {where_}"
            );
        }
        assert!(
            actor.is_human(),
            "P4: the model recorded a non-human founder — {where_}"
        );
    }
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

fn history_count() -> usize {
    std::env::var("FRACTAL_SIM_HISTORIES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200)
}

fn steps_per_history() -> usize {
    std::env::var("FRACTAL_SIM_STEPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(40)
}

#[test]
fn invariants_hold_over_generated_histories() {
    let histories = history_count();
    let steps = steps_per_history();
    let mut total = 0;
    for seed in 0..histories as u64 {
        total += run_history(seed, steps);
    }
    println!(
        "simulation: {histories} histories × {steps} steps = {total} operations, 0 violations"
    );
}

#[test]
fn a_history_is_reproducible_from_its_seed() {
    // The property the whole harness depends on. If this fails, every other
    // assertion here becomes unfalsifiable noise (ADR-0014).
    //
    // Note what is compared: the generated INPUTS, not the resulting
    // SocietyIds. `IdGen` is a counter, so ids ascend identically whatever the
    // seed — which is correct for an id generator and useless as a signal that
    // two histories differ. The first draft of this test compared ids and
    // failed for exactly that reason: it was asserting something about the id
    // generator while claiming to assert something about the harness.
    let a = collect_inputs(41);
    let b = collect_inputs(41);
    let c = collect_inputs(42);
    assert_eq!(a, b, "the same seed produced a different history");
    assert_ne!(a, c, "different seeds produced identical histories");
    assert!(
        !a.is_empty(),
        "seed 41 generated nothing — the generator is not doing anything"
    );
}

fn collect_inputs(seed: u64) -> Vec<(String, String)> {
    let det = Deterministic::seeded(seed);
    let g = Gen::new(&det.rng);
    (0..30).map(|_| (g.name(), g.handle())).collect()
}
