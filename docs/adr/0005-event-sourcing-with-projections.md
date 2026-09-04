# ADR-0005 — Event sourcing with derived projections as the state model

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 0

## 1. Context

P6 requires state changes to be immutable, ordered, typed domain events appended to a per-Society log, with every read model derived from it. P12 requires that every Fraction have a named source and sink and that the economy be auditable rather than merely logged. P4 requires that every capability grant trace to a human signature and every agent action name the grant that permitted it.

Those three principles have one common requirement: **the system must be able to answer "how did we get here?" mechanically, years later, without a human reconstructing intent.** A mutable-row model cannot. It answers "what is true now," which is exactly the question that is not in dispute when a balance is wrong, an agent acted outside its Envelope, or a Society disputes its Fracture split.

The economy makes this non-negotiable rather than merely desirable. `16 §5` states it plainly: replay is the mechanism by which P6 and P12 are *verifiable* rather than asserted, and the mechanism by which the Phase 8 ledger migration is a swap and not a rewrite. If the fold is not deterministic, none of it holds.

## 2. Decision

State changes are expressed as immutable, ordered, typed **domain events** appended to a per-Society log, which is the source of truth. Read models, notifications, reputation, XP, ledger postings, search indexes and agent triggers are **projections** — disposable by definition, rebuildable from `seq` 0. Nothing mutates a projection directly.

Every event carries the `10 §5` envelope without exception: `society_id`, per-Society `seq`, `event_id`, versioned `kind`, `occurred_at` and `recorded_at` as distinct fields, `actor`, `on_behalf_of`, `envelope_ref`, `correlation_id`, `causation_id`, payload, and a chained `integrity` hash. Event evolution is additive within a version; a breaking change is a new `.v2` kind plus a registered upcaster in the replay path. **Old events are never rewritten — they are historical fact.** Every command carries a client-generated `idempotency_key`, deduped on `(principal, idempotency_key)` for 24 hours.

## 3. Consequences

### Positive
- A new feature that needs a different view of existing facts is a new consumer, not a migration.
- `envelope_ref` on every event is what makes P4 auditable rather than a slogan: each agent action names the grant that permitted it, and each grant traces to a human signature.
- Time-travel debugging is free, and a production incident can be replayed against a fix before it ships.
- Anchoring (`16 §6`) has something to commit: a per-Society log state root is meaningful only over an ordered, immutable sequence.
- Idempotency makes the CLI, agents and flaky mobile networks safe to retry, which P2's outbox depends on.

### Negative
- **Every query is against a projection, so every read is eventually consistent with the write that caused it.** The UI must handle this honestly rather than hide it; `32 §6` makes `stale` one of the four mandatory states of every data surface.
- Schema evolution is permanent work. Every breaking change needs an upcaster with golden-vector tests, and that corpus grows monotonically for the life of the product.
- Storage grows without a natural delete. Tombstones and retention (`13 §10.2`) operate on Vault objects, not on the log, and a `Sealed` Society's log is immutable by invariant 14.
- Rebuild time is a real operational number. A Society with millions of events cannot rebuild its projections inside a deploy window, so rebuilds are per-Society, online, and staged.

### Neutral / follow-on work
`fractal-app-projection` (Phase 1) hosts the projection runner, the replay driver and the "rebuild from seq 0" command that makes P6's falsification test executable (`41 §5.4`).

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **CRUD with an audit table** | Far simpler; every engineer knows it; reads are direct | The audit table is written by the same code that mutates the row, so it records intent rather than fact and drifts silently. It cannot rebuild state, so P6's falsification test — delete every projection and rebuild — is unrunnable, and P12's "every Fraction that moved is a replayable fact" is downgraded to "we logged it" |
| **Event sourcing for the Ledger only, CRUD elsewhere** | Concentrates the cost where the value is highest; most of the product is chat and profiles | Two state models means two consistency stories, two testing strategies, and a boundary that every cross-cutting feature must cross. XP, Trust and Standing are exactly such features and they are economically load-bearing (`18`). It also forfeits `40 §7.5`'s simulation harness for everything outside the ledger, which is where the interleaving bugs actually live |
| **Change data capture from a mutable store** | Gives downstream consumers a stream without rewriting the write path; Postgres logical decoding makes it nearly free | The stream carries row diffs, not domain intent. `PostingReason` is a closed enum precisely so that every Fraction movement is categorized at the moment it moves; a CDC row diff cannot reconstruct why. Retained as a *transport* for projections, not as the source of truth |

## 5. Exit Cost

**Unbounded as a reversal; the abstraction that bounds it is the projection layer.** Abandoning event sourcing means the log stops being authoritative, at which point invariants 2, 4, 10, 11 and 12 of `11 §7` become unverifiable and the Phase 8 ledger migration reverts to a rewrite. What *is* bounded is the storage substrate: because the log sits behind the `EventStore` port (`10 §7`), moving it from Postgres to FoundationDB or per-Society segment files is **4–6 engineer-weeks** — adapter, conformance suite, and an online backfill — with no domain change. That is the swap this decision is designed to permit.

## 6. Principle Served

**P6** directly. **P12**: `Σ debits == Σ credits` after every command is checkable only because there is an "after every command" to check. **P4**: `envelope_ref` in the envelope. **P2**: the local replica syncs by `since(seq)` against an ordered log, which is what makes offline reconciliation tractable. **P1**: the log is per-Society, so ADR-0004's partition and this decision are the same decision seen twice.

## 7. Falsification Test

P6's own test, made a build step: **delete every projection and rebuild from `seq` 0; any state that cannot be reconstructed is a violation.**

1. `cargo xtask replay --society <id> --from 0` rebuilds every projection into a scratch schema and diffs it byte-for-byte against the live one. Run nightly against a production-shaped fixture corpus and at every phase gate.
2. The contract gate (`40 §7.7`) fails any PR adding a required field to an existing event kind, or a new `kind` without a registry entry, or a `.v2` without a registered upcaster and a passing historical-fixture upcast test.
3. A source lint fails any write to a projection table from outside `fractal-app-projection`.
4. `fractal-sim` asserts invariant 10 after every simulated step.

## 8. Maintenance Horizon

First-party: the log format, the envelope, the schema registry and the upcaster chain are ours. External exposure is limited to `serde` and the canonical encoder, both pinned, with a change of encoding treated as a new `schema_version` rather than a silent reserialization (`16 §5`). The upcaster corpus is the long-lived asset and the long-lived obligation; it is owned by the same CODEOWNER group as `fractal-schema`.

## 9. Review Trigger

Reopen if (a) full-projection rebuild for the median Society exceeds 10 minutes, which makes recovery drills impractical and forces snapshotting; (b) the registered upcaster chain for any event kind exceeds four hops, at which point the kind should be retired in favour of a new one; or (c) log storage growth exceeds the `13 §12` cost model by more than 2× for two consecutive quarters.
