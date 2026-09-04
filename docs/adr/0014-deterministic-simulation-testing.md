# ADR-0014 — Deterministic simulation as the primary correctness gate for domain logic

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 0

## 1. Context

`10 §7` makes `Clock`, `Rng` and `IdGen` ports, and says plainly that this is not pedantry: it is what makes the entire domain layer deterministically testable and the event log replayable. `11 §7` states fifteen invariants that must hold over every reachable history. P6 requires every projection to be reconstructible by replaying the log. `16 §5` forbids floating point, wall-clock reads, non-deterministic iteration, ambient randomness and projection reads in the money path.

Together these mean the domain layer is a pure function from (initial state, ordered inputs) to (events, state). There is no ambient nondeterminism inside the double-walled box of `10 §1`. That property is expensive to maintain and worthless unless something exploits it.

**The bugs that will actually hurt this system are not single-function bugs; they are interleaving bugs.** A Fracture that races a Transfer. A settlement run that overlaps a Charter amendment. A revoked Envelope with an action in flight. A local-first sync reconciliation that is correct in the two cases a developer imagined and wrong in the fifth. Example-based tests do not find these, because a human — or an agent — writing a test writes the interleaving they already thought of.

## 2. Decision

We build a deterministic simulation harness (`fractal-sim`, over the doubles in `fractal-testkit`) that drives the whole domain and application layer against in-memory port implementations whose behaviour — time, scheduling order, message delivery, partial failure, retry, partition — is derived from a single `u64` seed. Each run generates a random but legal history of commands across N = 1..8 Societies, executes it on a single thread in seeded order, and asserts **all fifteen `11 §7` invariants after every step**.

Fault injection is not optional: transient port failure (0.1–5% of calls), failure after commit (0.05–1%), duplicated messages (0.5–3%), intra-subject reordering (1–10%), process restart mid-saga (every 1k–20k steps), clock jumps (every 5k steps), and Node/Runtime partition (1–20% of wall time). A failing seed is a complete, replayable bug report; a shrinker reduces it to a minimal history before it is filed.

Cadence: 2,000 fresh seeds plus the full regression corpus per PR (~6 minutes on 8 cores); 500,000 seeds nightly; 5,000,000 pre-phase-gate. Every fixed bug contributes its minimal seed to a permanent corpus.

## 3. Consequences

### Positive
- Finds concurrency and ordering defects that no other technique in our budget finds.
- Every failure is reproducible from `seed + commit`, which removes the single worst debugging experience in distributed systems.
- Forces the port discipline to stay honest: an ambient `SystemTime::now` breaks replay loudly on the commit that introduces it, not months later.
- Discharges P6's falsification test as a continuously running assertion rather than a periodic audit.
- Makes agent-written code safe to accept faster. An agent that satisfies a stated invariant is exactly what this harness checks, on every reachable interleaving.

### Negative
- **Roughly 6 engineer-weeks to first value, and a permanent tax:** every new port needs a deterministic double and every new invariant needs an oracle assertion. This cost never ends.
- It biases us toward keeping the domain pure, which occasionally makes a feature more awkward than the direct implementation would be. We consider that a benefit; it is listed here because it is a real constraint on future design.
- **It gives no coverage of the real adapters.** Postgres, S3, NATS and MLS misbehaviours are invisible to it and need integration tests regardless (`40 §7.6`).
- A double that diverges from its real adapter turns the harness into a well-tested fiction. The port conformance suite is what prevents this, and it is a second permanent obligation.

### Neutral / follow-on work
`fractal-testkit` is a Phase 0 crate (`41 §5.1`), and "ships with a deterministic double" is a merge requirement for every new port — which is what keeps the retrofit cost near zero.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **Property tests only (proptest per invariant)** | Much cheaper, well understood, and we do this anyway | Operates on one aggregate at a time. It cannot express multi-Society interleaving, which is exactly where Fracture, settlement and Envelope-revocation bugs live. Retained as a complement, not a substitute |
| **Chaos testing against a live deployment** | Tests the real stack including adapters, which the harness cannot | Non-reproducible: a failure is an anecdote, not a test case. It also finds bugs after deploy rather than before merge. Adopted later as a complement |
| **Formal methods (TLA+ for Fracture and Ledger)** | The strongest possible guarantee for the two riskiest operations | Verifies the model, not the code, and the gap between them is where our bugs will be. We do write a TLA+ spec for Fracture (`11 §3.2`) as an *input* to the harness's invariants; it does not replace executable testing |
| **Buy a testing platform** | No build cost, someone else maintains it | Nothing on the market simulates *our* domain invariants. The value is entirely in the fifteen `11 §7` assertions, which are ours and cannot be bought |

## 5. Exit Cost

**Effectively zero as an exit; 3–4 engineer-weeks as a retrofit risk.** The harness is additive — deleting it removes assurance, not capability. The real cost is the coupling it creates: if a future port is introduced without a deterministic double, retrofitting one is 3–4 engineer-weeks, which is why the double is a merge requirement for every new port rather than a follow-up ticket. Note also the asymmetry that justifies the 6-week build: the alternative to finding an interleaving bug in the Ledger before merge is finding it after Fraction has moved, and P6's event log makes that visible without making it reversible.

## 6. Principle Served

**P6** (event-driven and replayable) directly; **P12** — invariants 2, 3, 4 and 15 are economic and are asserted on every simulated step; **P5**, since the port discipline this depends on is enforced by the harness's mere existence; **P2**, since the partition fault models the offline behaviour P2 promises. No principle is traded away.

## 7. Falsification Test

`cargo sim --seeds 10000` must pass on every commit to `main`. The second half is the one that matters: **the harness must fail within 200 seeds when any single `11 §7` invariant assertion is deliberately inverted.** That is a real weekly CI job (`sim-mutation`) which inverts each of the fifteen in turn.

A harness that has silently stopped asserting is the most dangerous artifact in the repository — it converts "we tested it" into a false statement while every dashboard stays green — so proving weekly that ours can still detect a broken invariant is not ceremony, it is the test of the test. A third check runs the port conformance battery against both the real adapter and its double; a divergence there means the simulation is testing a fiction.

## 8. Maintenance Horizon

First-party code; no external maintainer risk. Dependencies are `proptest` for shrinking and `rand` with a pinned, reproducible PRNG algorithm — the algorithm is pinned rather than the crate version alone, because a PRNG change silently invalidates the entire regression corpus. If `proptest` were abandoned, the shrinker is roughly 500 lines to replace. The permanent asset and permanent obligation is the seed corpus: it is the institutional memory of everything this system has ever gotten wrong, and it must survive every repository reorganization (`41 §13.1`).

## 9. Review Trigger

Reconsider scope if (a) a production incident's root cause was reachable by the harness but not found within 100k seeds, which indicates the generator's distribution is wrong rather than the harness; or (b) full-lane simulation time exceeds 15 minutes, at which point volume moves nightly and the per-PR run becomes the fixed regression corpus plus 1,000 fresh seeds.
