# ADR-0007 — NATS JetStream as the event bus

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 1

## 1. Context

ADR-0005 makes every read model a consumer of the per-Society log. `10 §5` names five simultaneous consumers of a single `ChamberMessagePosted`: the discourse read model, the search index, the XP/progression projection, the ledger where a Posting is implied, and Relay fan-out to subscribed front ends. Those consumers have different failure characteristics — a search indexer may lag minutes without harm; Relay fan-out that lags seconds is a broken product against `14 §3`.

The Runtime is a single binary (ADR-0003), so in-process channels are available and are the obviously cheapest thing. They fail on two counts. First, `10 §9` runs three or more stateless Runtime replicas from Phase 1; a projection consumer that lives in one replica's memory either duplicates work across replicas or has no failover. Second, ADR-0003's predicted extraction order puts Relay, Transcoder and Agent Executor outside the binary by Phase 4–6; a consumer that was an in-process channel becomes a network hop with no delivery guarantee at exactly the moment its reliability starts to matter.

Postgres is already present (ADR-0006) and could carry this with `LISTEN/NOTIFY` or a polled outbox table. That is worth taking seriously, and it is the alternative that loses on the narrowest margin.

## 2. Decision

We use **NATS JetStream** as the durable event bus. Subjects mirror the domain structure exactly:

```
society.<society_id>.<boundary>.<event_kind>
society.7f3a….discourse.message.posted.v1
```

Delivery is at-least-once with replay from a stored sequence, which pairs with the `10 §5` idempotency contract: every command carries an `idempotency_key`, deduped on `(principal, idempotency_key)` for 24 hours, and every projection consumer is idempotent on `(society_id, seq)`. `fractal-adapter-nats` implements the durable `Relay` port and the bus; `fractal-adapter-ws` implements the live in-process and WebSocket path (`41 §5.5`). The bus is **transport for projections and Signals, never the source of truth** — the log in Postgres is (ADR-0005, ADR-0006).

## 3. Consequences

### Positive
- The subject hierarchy is the domain hierarchy, so a consumer subscribes to `society.*.ledger.>` and gets exactly the boundary it wants without a routing layer.
- Replay from a stored sequence means a consumer that was down does not need a projection rebuild — it resumes.
- Consumers survive Runtime replica restarts and rolling deploys, which in-process channels do not.
- ADR-0003's predicted extractions become cheap: Relay, Transcoder and Agent Executor already talk to the bus, so extraction changes deployment, not code.
- NATS is embeddable, so a single-binary self-hosted Node (`10 §9`, Phase 6) runs the same code path without operating a cluster.

### Negative
- **A second stateful system to operate**, with its own storage, its own upgrade path and its own failure modes. That is a real cost and it consumes a `02 §5` budget slot.
- At-least-once means duplicates are normal traffic, not an anomaly. Every consumer must be idempotent, and a consumer that is accidentally not idempotent fails intermittently under load rather than obviously in test — which is precisely why `40 §7.5` injects duplicate delivery at 0.5–3% and reordering at 1–10%.
- Ordering is per-subject, not global, and the bus must never be the thing that establishes order. Order comes from `seq` in the log.
- Operational familiarity is lower than Postgres. Fewer engineers have debugged a stuck JetStream consumer at 3am than a stuck Postgres query.

### Neutral / follow-on work
`14 §12` defers WebTransport over HTTP/3 as a future carrier behind the same `Relay` port — a different transport, the same `SignalFrame`, evaluated in Phase 5.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **Kafka** | The default answer; unmatched replay and retention; enormous operational literature | Disproportionate to our volume and shape. Partition-per-topic scaling is the wrong granularity when the natural key is `society_id` with a long tail of small Societies, and the operational weight (ZooKeeper-or-KRaft, rebalancing, consumer-group semantics) is a full budget slot for capabilities we would not use before Phase 5. Not embeddable, so the self-hosted Node story degrades |
| **Postgres `LISTEN/NOTIFY`** | Zero new dependencies; already operated; transactional with the write | No durability and no replay: a notification delivered while a consumer is down is gone, so any consumer restart becomes a projection rebuild. Payloads are capped at 8KB, and the fan-out is per-connection, which does not survive `10 §9`'s multi-replica topology. This is the closest alternative and it fails on the single property the bus exists to provide |
| **Redis Streams** | Lightweight, familiar, consumer groups, low latency | Weaker delivery and durability guarantees than JetStream for the same operational cost, and persistence semantics that are configuration-dependent in ways that are easy to get subtly wrong. Redis was also considered for ephemeral presence and **rejected there too** (`61 X3`): presence is 45-second TTL data with no durability requirement, held in Relay-process memory and gossiped over the JetStream subject `presence.<society_id>`, which is one fewer stateful system to operate |
| **A polled Postgres outbox table** | No new system at all; transactional with the append; trivially debuggable | Genuinely viable through Phase 2 and we keep it as the fallback. It loses on Relay latency — polling adds a floor that `14 §3` cannot absorb — and on fan-out cost: every consumer polling every Society's outbox is a load pattern that grows with consumers × Societies |

## 5. Exit Cost

**2–3 engineer-weeks.** The bus sits behind the `Relay` port plus an internal `EventBus` trait; `fractal-adapter-nats` may depend only on `fractal-ports`, `fractal-types` and the NATS SDK (`41 §5.5`). Replacing it means writing one adapter, passing the existing port conformance suite, and running both buses in parallel for one release with consumers double-subscribed and deduping on `(society_id, seq)`. The parallel-run window is the bulk of the estimate, not the code. The number stays low only while no consumer reads a NATS-specific header outside the adapter.

## 6. Principle Served

**P6** — projections are consumers, and a bus with replay is what makes "a new projection is a new consumer, not a migration" true operationally. **P5** — the `Relay` port has an in-process double and a WebSocket implementation alongside the NATS one. **P10** — sub-100ms Signal delivery is achievable without polling. Indirectly serves ADR-0003 by pre-paying the extraction path for the three modules most likely to leave.

## 7. Falsification Test

1. **Source of truth**: a lint fails any read in `fractal-app-*` that treats a bus message as authoritative — consumers may take `(society_id, seq)` from a message and must load the event from `EventStore`. If the bus can be the source of truth, ADR-0005 is already violated.
2. **Idempotency under duplication**: `fractal-sim` injects duplicate delivery (0.5–3%) and intra-subject reordering (1–10%) per `40 §7.5`, and asserts invariant 10 — every projection reproducible by replay — after every step. A consumer that is not idempotent fails within seeds.
3. **Vendor containment**: `nats` appearing in the transitive closure of any crate outside `crates/adapter/nats` fails `lint-deps` A2.
4. **Replay**: an integration test stops a consumer, appends 10k events, restarts it, and asserts the projection converges with no rebuild.

## 8. Maintenance Horizon

NATS is a CNCF graduated project with a commercial maintainer (Synadia) and multiple independent operators; it is not single-maintainer critical-path. The Rust client is the narrower exposure and is pinned; because it is confined to one adapter crate, replacing the client is an adapter-internal change reviewed under `40 §10.2`. The embedded-server path used by self-hosted Nodes is exercised in CI so it does not rot behind the hosted deployment.

## 9. Review Trigger

Reopen if (a) sustained bus throughput exceeds 50k messages/second per cluster, where Kafka's partition model starts to earn its operational weight; (b) JetStream consumer lag causes two Signal-delivery SLO breaches in a quarter (`40 §9.4`); or (c) the extraction of Relay (ADR-0003, predicted ②) shows the durable bus and the live path want different transports, in which case the `Relay` port splits and this ADR covers only the durable half.
