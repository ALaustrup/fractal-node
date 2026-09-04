# ADR-0016 — Two `EventStore` implementations from commit one

Status: Accepted
Date: 2026-09-04
Deciders: Andrew
Phase: PH0 (M0.7)

## Context

P5 requires that every swappable boundary have "at least two implementations at
the time the boundary is created (typically the real one and an in-memory test
double)". `docs/10 §7` names Postgres as the Phase 1 `EventStore`.

Taken literally, PH0 would ship one real store and one test fake. That satisfies
the letter of P5 and misses its point. A test double written by the same person,
in the same session, against the same mental model as the interface will agree
with that interface by construction. It proves nothing about whether the boundary
is real — it proves only that the author was consistent for an afternoon.

The failure this guards against is specific and common: an interface that has
quietly been shaped around one implementation's assumptions, discovered only when
the second implementation arrives and does not fit. By then the interface has
callers, and the correction is a refactor rather than an edit.

## Decision

Ship **two genuinely different `EventStore` implementations in PH0**, and hold
them to a shared behavioural equivalence test:

- `fractal-adapter-store-memory` — `BTreeMap` in an `RwLock`.
- `fractal-adapter-store-jsonl` — one append-only file per Society, `O_APPEND`
  writes with an explicit `sync_all`, position implied by line number.

`crates/adapter/store-jsonl/tests/equivalence.rs` runs identical scripts against
both and asserts identical observations: appends, reads, paging, optimistic
concurrency conflicts, per-Society scoping, and unknown-Society reads. A
divergence fails the build.

Postgres joins that test in PH1 rather than replacing either implementation.

Neither store is thrown away. The JSONL store is the shape `docs/10 §7` already
names as a future option ("per-society segment files"), and it gives PH0 durable
storage with no database dependency — which is also what keeps the phase inside
its five-dependency-family budget (`docs/02 §5`).

## Consequences

**Positive**

- The port is proven, not assumed. The equivalence test caught the interface
  question that matters most — that `Seq` is scoped to one Society and there is no
  global sequence — because two implementations had to agree on it.
- PH0 gets durability without Postgres, keeping the dependency budget intact.
- PH1's Postgres work has a specification: pass this test.
- A local-first Node (PH2) has a plausible on-disk story already in the tree.

**Negative**

- Two implementations to maintain, and every `EventStore` change is two edits
  plus a test. That is the cost of the guarantee, and it is charged on every
  future change, not just this one.
- The JSONL store reads the whole log to find its head. Fine at PH0 volumes,
  wrong by PH2. It needs an index or a length cache before it is used in anger,
  and that is tracked rather than pretended away.
- `layers.toml` had to exempt `[dev-dependencies]` from the dependency-direction
  lint so the equivalence test can see both adapters. That exemption is narrow
  and documented in the file, but it is a hole in an otherwise total rule.

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| One store plus an in-memory fake | Satisfies P5's wording, not its purpose. A fake written against the interface cannot falsify the interface |
| SQLite as the second store | A sixth dependency family in PH0, breaching `docs/02 §5`, to prove something a file already proves |
| Defer the second implementation to PH1 with Postgres | The interface would then have callers. Corrections become refactors, which is the exact cost P5 exists to avoid |

## Exit Cost

Deleting the JSONL store once Postgres lands is roughly one engineer-day: remove
the crate, drop its arm from the equivalence test. Deliberately cheap. But the
recommendation is to keep it — it is the local-first Node's storage layer.

## Principle Served

P5 (the boundary is real), P6 (the log is the source of truth, so it must survive
a restart), P2 (a Node holds its own durable replica).

## Falsification Test

`cargo test -p fractal-adapter-store-jsonl` runs the equivalence suite; any
behavioural divergence fails. The stronger test is procedural: when Postgres
arrives in PH1, it must pass this suite **unmodified**. If the suite has to be
weakened to accommodate it, the abstraction was shaped around the first two
implementations and this ADR did not achieve its aim.

## Maintenance Horizon

Both stores are maintained through PH2. Reassess when Postgres is in production
and the local-first Node's storage requirements are known.

## Review Trigger

Any change to the `EventStore` trait, or the first `EventStore` bug that appears
in one implementation and not the other.
