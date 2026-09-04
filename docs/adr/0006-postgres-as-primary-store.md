# ADR-0006 — Postgres as the primary store for events, projections, search and queues

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 0

## 1. Context

ADR-0005 makes the per-Society event log the source of truth and every read model a projection. That creates a specific storage requirement that is easy to misread as "we need an event store product": we need the append and the projection update to be **one transaction**, because otherwise a crash between them produces a projection that is silently wrong until the next rebuild.

The Ledger sharpens it further. `16 §4.1` requires real ACID transactions across the event append and the balance projection, integer arithmetic with no floating point anywhere, row locks and serializable isolation where the posting path needs them, and the `Σ debits == Σ credits` check enforced *in the same transaction* that writes the Posting. `11 §2.6` states that a Posting that does not balance is not written — that is a database-level guarantee, not an application convention.

`02 §5` allows five new third-party runtime dependencies per phase and two new top-level services. `00 §3.4` prefers boring where the problem is solved. At Phase 1 volumes, one system that does events, projections, full-text search and queues is strictly cheaper to operate than four that each do one well.

## 2. Decision

Postgres is the primary store for Phase 1–3: the `EventStore` adapter, all projection tables, the internal double-entry `Ledger` adapter, `Search` via full-text search, and job/outbox tables. It is reached only through ports (`10 §7`) — `fractal-adapter-postgres` and `fractal-adapter-ledger-internal` may depend on `fractal-ports` and `fractal-types` and on nothing in the domain or application layers (`41 §5.5`).

Money is `bigint` integer arithmetic; **`Quanta` is `i64` persisted and on the wire, `i128` intermediate**, checked throughout, with no `f64` conversion in existence. Unsigned was rejected because `16 §19` LA1 requires the `EmissionAccount` shards to hold negative balances, and 128-bit storage was rejected because the domain targets `wasm32` from PH0 where it is emulated (`17 §2.1`, `61 X4`). Not `numeric`: arbitrary precision is slower and invites the `f64` conversion the lint exists to prevent. Logical decoding provides CDC for projection fan-out without a second write path.

## 3. Consequences

### Positive
- The event append and the projection update commit together, so a crash cannot produce a divergent read model.
- One system to back up, restore, monitor, tune and hire for. `40 §11` point-in-time recovery plus log replay is a single coherent story rather than a reconciliation across four products.
- Logical decoding gives CDC for free, which feeds the Relay and search without a dual-write.
- The largest operational knowledge base of any database, which matters more than throughput at this stage (`00 §3` standing bias: operability over raw performance).

### Negative
- **Postgres is not a natural event store.** An append-only per-Society sequence with a chained integrity hash is something we implement on top of a table, including the `seq` allocation, the gap detection and the read-side streaming.
- **We will need partitioning by `society_id` earlier than is comfortable.** `10 §12` names it as the first scaling move, and declarative partitioning of a live, large table is a genuine operational project, not a migration file.
- A single primary is a single write bottleneck. Read replicas help projections; they do not help the posting path.
- Full-text search is adequate, not good. Ranking quality and multilingual handling are visibly weaker than a dedicated index, and `10 §7` already names Tantivy or Meilisearch as the later implementation.

### Neutral / follow-on work
The `EventStore`, `Ledger` and `Search` ports each already have a second implementation in `fractal-testkit` (`41 §5.1`), so the P5 two-implementations rule is satisfied at introduction and the port conformance suite (`40 §7.6`) keeps the in-memory double honest against real Postgres behaviour.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **Kafka plus a separate read store** | The canonical event-sourcing stack; excellent replay and fan-out; horizontal by design | Two stateful systems, and no transactional read-model update: the append and the projection write land in different systems, which is exactly the failure mode this decision exists to prevent. Kafka's operational weight is also disproportionate to Phase 1 volume, and it consumes a top-level-service budget slot (`02 §5`) for throughput we do not need |
| **EventStoreDB** | Purpose-built for this pattern; streams, subscriptions and projections as first-class concepts | Small ecosystem and a narrow operational knowledge base, which `00 §3.5` treats as a maintenance-horizon risk rather than a taste question. It also still requires a second store for relational projections and search, so it adds a system without removing one |
| **FoundationDB** | Genuinely excellent: strict serializability, proven at scale, and the source of the deterministic-simulation approach we adopt in ADR-0014 | The operational learning curve costs a phase. It also has no query layer, so projections, search and reporting all need building on top. Retained as the named second implementation behind `EventStore` for when partitioning stops paying (`10 §12`) |
| **TigerBeetle for the ledger at Phase 1** | Purpose-built double-entry engine, correct by construction, extremely fast | The most credible Phase 4+ swap and the best technical fit for the posting engine. Rejected *for now* on complexity budget: a second stateful system before the spine sentence is true, buying throughput we do not need. The `Ledger` trait makes adopting it a swap, which is the entire point of N4 |

## 5. Exit Cost

**4–6 engineer-weeks per port, and each port exits independently.** Moving `EventStore` to FoundationDB or per-Society segment files: write the adapter, pass the existing port conformance suite, run a dual-write and backfill window, cut over per Society — no domain or application change, because `layers.toml` forbids the edge that would make one necessary. Moving `Ledger` to TigerBeetle is the same shape at roughly 5 weeks plus a state-root reconciliation against the historical Postings. Moving `Search` to Tantivy is ~2 weeks. **The exit cost is bounded only because nothing above the adapter names Postgres**; the day SQL appears in a domain crate, this number becomes unbounded.

## 6. Principle Served

**P6** (transactional append plus projection update is what makes replay a guarantee), **P12** (`Σ debits == Σ credits` enforced in the writing transaction), **P5** (three ports, each with a second implementation at introduction), and operability under the `00 §3` standing bias. Nothing is traded away in principle; what is traded is peak throughput, which we do not yet need and cannot yet measure.

## 7. Falsification Test

1. **Dependency lint** (`41 §7.2` A1/A2): `sqlx`, `postgres` or any SQL string appearing in the transitive closure of a `fractal-domain-*` crate fails the build. This is the test that keeps the exit cost bounded.
2. **Port conformance** (`40 §7.6`): one shared battery runs against `fractal-adapter-postgres` and against the in-memory double. A behaviour the real adapter has and the double does not — or vice versa — fails, because a divergent double turns ADR-0014's harness into a well-tested fiction.
3. **Ledger invariants**: `16 §19` L2 (`Σ` balances `== 0`) and `11 §7` invariants 2–4 are asserted after every command in `fractal-sim` and by an hourly production reconciliation job.
4. **No floating point, and no width drift**: `#![deny]` on `f32`/`f64` in `fractal-domain-ledger`; `Quanta` exposes no `From<f64>`, no `From<u128>`, and no `as` conversion anywhere in the tree; `const _: () = assert!(SUPPLY_CAP_QUANTA <= i64::MAX / 9);` fails the build if a cap amendment eats the 9.22× headroom; and a codegen test asserts the wire schema is a decimal **string**, not a JSON number, because IEEE-754 is exact only to 2^53 and the cap is 1e18.

## 8. Maintenance Horizon

Postgres has a multi-vendor ecosystem, a 5-year support window per major, and no single-maintainer risk. `sqlx` is the one crate with concentrated maintenance exposure on this path; it is pinned, and the port boundary means a replacement driver is an adapter-internal change. Managed hosting is deliberately not assumed anywhere above the adapter, so the self-hosted Node of `10 §9` runs the same code.

## 9. Review Trigger

**Correction (`61 W4`).** This trigger previously read "after partitioning by `society_id`", which §3 above already contradicts: *"a single primary is a single write bottleneck"*. Declarative partitioning within one instance reduces index depth, vacuum cost and locality — it adds **no write throughput**, because there is still one instance, one WAL and one writer. The remedy for a write-throughput ceiling is **sharding across primaries**, routed at the composition root by rendezvous hash on `society_id`, with no transaction spanning shards. The full four-step ladder, its measurable triggers, and the six code properties that must hold beforehand are in `10 §12`.

Reopen when (a) sustained posting-path write transactions per second exceed **60% of the measured single-primary ceiling** for 7 consecutive days, or p99 posting latency exceeds 50 ms at that load — that is the signal to begin step 3, shard across primaries, and separately the signal to evaluate moving `Ledger` to TigerBeetle; (b) the event table exceeds the working set of a single primary's buffer cache such that projection rebuilds start evicting hot data — move `EventStore` per `10 §12`; or (c) search relevance complaints appear in two consecutive phase-gate reviews, which is when Tantivy earns its dependency slot.
