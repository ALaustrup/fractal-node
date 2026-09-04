# ADR-0004 — The Society is the partition unit; ordering is per-Society, never global

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 0

## 1. Context

P1 fixes the Society as the atomic container: every persistent object answers "which Society owns you?" with a `society_id` or appears on the nine-entry Global Registry (`01 §6`). That was chosen as a mental, permission and billing model. `10 §4` makes it the **physical** partitioning strategy as well, and that is a separate and much larger commitment.

The question a distributed social platform must answer early is what it orders. A single global sequence is conceptually simple and puts consensus on the write path of every message, which sets a floor on write latency that P10's 100ms interaction budget cannot absorb and which makes multi-region a coordination problem rather than a routing problem. The alternative — order nothing globally — makes a Citizen's cross-Society inbox and platform-wide statistics harder.

Three named operations force the choice early rather than late. **Fracture** (`11 §3.2`) must seal one log at a sequence point and derive children from it; that is a single-partition operation only if the partition is the Society. **Self-hosting** (`10 §9`, Phase 6) requires a Society's entire state to be a portable, verifiable bundle. **Anchoring** (`16 §6`) commits a per-Society log state root, and a root is meaningful only over an ordered sequence.

## 2. Decision

We partition by `society_id`. The event log is **per-Society, append-only, with a monotonic `seq` scoped to that Society** (`10 §5`). We require total order *within* a Society and only **causal** order *between* Societies. **There is no global consensus in the hot path.**

Concretely: `EventEnvelope.society_id` is mandatory on every event, with a `GLOBAL` sentinel reserved for the nine registry entries; routing is by `society_id`; a hot Society moves to its own partition without touching anything else; backup, restore and audit are per-Society; and a corrupt projection affects one Society.

## 3. Consequences

### Positive
- **Global consensus is removed from the hot path entirely.** This is the single most important scalability property in the architecture, and it is bought at design time rather than optimized for later.
- Fracture is a well-defined operation on one partition's log, treasury, vault and membership — not a distributed migration across the platform.
- "Take your Society and leave" is an export, not a negotiation.
- Test isolation is free: `40 §7.6` notes that every integration test creates its own Society and asserts only on that `society_id`.
- Backup and restore are per-tenant and independently verifiable (`40 §11.1`).

### Negative
- **Cross-Society operations are genuinely harder, and we accept this.** A Citizen's unified inbox, global search, Federation (Phase 6) and marketplace-wide statistics all require either fan-out reads or a global projection maintained by consuming many logs. There is no cheap version of this.
- A Citizen-level fact — XP, global Trust, Handle uniqueness — has no natural home in a per-Society log and must live in the Global Registry with its own consistency story.
- Rebalancing a partition is an operational procedure we must build before it is needed, not a database feature we inherit.
- Cross-Society causality is *only* causal: two events in different Societies have no defined total order, and any feature that assumes one is wrong by construction rather than by bug.

### Neutral / follow-on work
The global projections that cross-Society features need (inbox, discovery, market) are ordinary P6 consumers of many logs, tolerant of eventual consistency because they are read-mostly. `10 §12` pre-commits the escape hatch: introduce a global sequencer for a narrow event subset, never globalize the whole log.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **One globally ordered log** | Trivially correct cross-Society queries; one sequence to reason about; simplest possible replay story | Puts consensus on every write. It makes multi-region a coordination problem and sets a write-latency floor incompatible with P10's 100ms interaction budget. It also makes Fracture a distributed operation over the whole platform's log rather than over one Society's, which is precisely the operation `11 §3.2` must make safe and resumable |
| **Partition by Citizen** | Matches the user-facing mental model of a personal inbox; makes per-user export trivial | Every Chamber message would span N partitions, one per participant, so the most common write in the product becomes a distributed transaction. It also has no answer for objects a Society owns and no member does — Treasury, Charter, Vault — which are the objects `11 §2` treats as central |
| **No explicit partition; rely on the database** | Postgres handles it until it does not; defers the decision | Defers nothing. The partition key must be in the event envelope from the first event or it can never be added retroactively, and `10 §12` already names Postgres partitioning by `society_id` as the first scaling move. Choosing later means choosing after the data exists, which is the expensive time to choose |

## 5. Exit Cost

**Unbounded for a change of partition key; ≈2–4 engineer-weeks for a bounded relaxation.** Re-keying an existing corpus of per-Society logs to a different partition would mean rewriting history, which `10 §5` forbids outright — old events are never rewritten. This choice is therefore a foundation, not a swappable one, and `00 §3.3` requires it to be justified as such rather than hidden behind a port. The *relaxation* is cheap and pre-designed: adding a global sequencer for a narrow, explicitly enumerated event subset (`10 §12`) is roughly 2–4 engineer-weeks, because it adds a second ordering domain without disturbing the first.

## 6. Principle Served

**P1** directly — this is P1 promoted from mental model to physical strategy. **P2**: a portable, verifiable per-Society bundle is what makes a local replica authoritative. **P10**: no consensus on the hot path. **P12**: per-Society anchoring makes the ledger's history provable to a third party (`16 §6.1`). Traded away: some of P3's uniformity, since cross-Society API resources are eventually consistent while in-Society ones are not — declared in the API contract rather than discovered.

## 7. Falsification Test

Invariant 1 of `11 §7`, asserted continuously by the `fractal-sim` oracle (ADR-0014) and by a schema lint: **every persisted row has a `society_id` or appears in the Global Registry.** Mechanically:

1. `xtask lint-schema` walks every migration and fails on a table without a `society_id` column unless its name is one of the nine `01 §6` entries. Adding a tenth requires an ADR.
2. Every `EventEnvelope` constructed without a `society_id` fails to compile — the field is non-optional in `fractal-schema`.
3. A `cross-partition` test asserts that no query in `fractal-app-*` joins across two `society_id` values outside the explicitly registered global projections.
4. `fractal-sim` runs histories across N = 1..8 Societies (`40 §7.5`) and asserts invariant 11: Fracture preserves total Fraction, total Facets, total members and full readable history.

## 8. Maintenance Horizon

First-party design with no external dependency. The durable risk is erosion by convenience: a feature that "just needs one cross-Society join" is how the property is lost. `41 §14` assigns the migration path a Tier 2 CODEOWNER, so a schema change that would add an unpartitioned table requires human review and an ADR link.

## 9. Review Trigger

Reopen if (a) a genuinely global feature with strong-consistency requirements becomes core — Federation with synchronous guarantees, or a cross-Society inbox at a scale fan-out cannot serve — in which case introduce a global sequencer for that narrow event subset only; (b) the p99 fan-out read for a Citizen's unified inbox exceeds 400ms at the Phase 5 population; or (c) a single Society's log exceeds the throughput of one partition, which makes the Society itself the unit that must be split and is a different and harder decision.
