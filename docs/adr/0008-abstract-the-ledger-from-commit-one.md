# ADR-0008 — Abstract the Ledger behind a port from commit #1

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 0

## 1. Context

N4 states it as a non-negotiable: the ledger is abstracted from commit #1. P11 states why: the ledger is an internal, deterministic, auditable double-entry system behind a `Ledger` trait, external chains are adapters rather than foundations, and migration to a future Fractal Node L1 must be a swap of implementation behind an unchanged trait plus a state-root anchoring procedure — never a rewrite.

`02 §3` places the custom L1 at Phase 8+ and external chain bridges at Phase 8+. That is a gap of years between the abstraction and its second real implementation, which is exactly the situation `02 §7`'s "no abstraction without two callers" rule normally forbids. `16 §1` names the exception explicitly: the abstraction is created before its second implementation exists **because that implementation is known to be coming**.

The failure this prevents is specific and well documented in `16 §18`: without the trait, SQL leaks into the domain, the domain learns table shapes, and Phase 8 becomes a rewrite of everything that touched money — which, in an economy-bearing platform, is everything. The trait costs roughly one crate and a test double. The rewrite costs a phase.

## 2. Decision

`Ledger` is a trait in `fractal-ports` from the first commit, alongside `Chain` and `Rail`. Phase 1's implementation is `fractal-adapter-ledger-internal`: deterministic double-entry over Postgres. `fractal-adapter-chain-null` implements `Chain` by anchoring to the internal Ledger. No domain crate may name a concrete ledger implementation, a SQL type, or a chain SDK type; `layers.toml` enforces this by allowlisting the domain layer's entire transitive closure.

The trait deliberately does **not** expose transactions, table shapes, chain concepts, gas, addresses, or block heights (`16 §2.3`). It speaks in Postings, Wallets, Quanta and state roots. Anchoring (`16 §6`) is part of the abstraction from day one: Phase 1 anchors internally and publishes the proof format, so the Phase 8 procedure is one that has been exercised in production for years before it matters.

## 3. Consequences

### Positive
- The Phase 8 migration is a swap plus a state-root anchoring procedure, with a bounded and estimable cost, rather than an open-ended rewrite.
- The economy's correctness is independent of its storage: `Σ debits == Σ credits`, `balance >= locked >= 0` and `total supply == -EmissionAccount.balance` are domain invariants asserted against any implementation.
- The in-memory `Ledger` double in `fractal-testkit` is what lets `fractal-sim` (ADR-0014) drive the whole economy deterministically. Without the port, the harness could not exist.
- TigerBeetle, a future FN L1, and an external chain are all reachable as adapters rather than as architectural events.

### Negative
- **Real indirection cost with no user-visible benefit for years.** Every ledger operation crosses a trait boundary and a DTO translation the application layer writes by hand — roughly 15–20 lines per aggregate (`41 §4`).
- The trait must be designed against implementations that do not exist, so it will be wrong in places. A chain's asynchronous finality is genuinely hard to express in a trait shaped by a synchronous internal ledger; `16 §7.3` addresses finality honestly rather than pretending the trait hides it.
- `Chain` ships as `NullChain`, which is a stub that must be maintained and tested like a real adapter, or it will not work when it is first needed.
- Anchoring in Phase 1 is weaker than a public chain and we must say so plainly: an operator with total database control could rewrite both history and anchors (`16 §6.1`).

### Neutral / follow-on work
`Rail` is abstracted on the same grounds and stays internal-FRC-only until Phase 9 (`02 §3`), gated on counsel rather than on engineering.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **No abstraction — Postgres directly** | Less code, no trait indirection, faster Phase 1, and `02 §7` normally forbids the abstraction | Directly violates P5 and N4, and the exit cost is unbounded: SQL leaks into the domain, the domain learns table shapes, and Phase 8 becomes a rewrite of everything that touched money. This is the exact trade `00 §3.3` requires us to make in the other direction |
| **Build on an existing chain now** | Instant liquidity and wallets, no ledger to build, credibility with a crypto audience | Every Posting becomes a public, priced, latency-bound external transaction, which is irreconcilable with P10's interaction budget and with a social platform's write volume. Fees make the micro-rewards that `17` is built on uneconomic; external key custody contradicts P8 and adds a catastrophic onboarding tax; and it hands the economy's regulatory posture to a third party at Phase 1 |
| **A single mutable balance column, no double-entry** | The simplest possible thing that works | Loses the invariant that makes the economy auditable. `Σ debits == Σ credits` is what turns "we think the numbers are right" into a checkable statement; without it P12's falsification test cannot run at all |
| **Abstract later, when the second implementation is real** | Avoids designing against a hypothetical; the trait would be better informed | The migration would then have to be performed *and* the abstraction introduced simultaneously, on live balances, which is the highest-risk possible sequencing. It also forfeits the deterministic in-memory double for years, and with it the simulation harness that guards the money path |

## 5. Exit Cost

**Swapping the implementation: 6–10 engineer-weeks. Removing the abstraction: negative value, so it is not an exit we would take.** The swap breaks down as: write the adapter (2–3 weeks); pass the port conformance suite (1 week); reconcile historical state — replay every Posting into the new implementation and prove the resulting state root matches the anchored root at each frontier (2–4 weeks, and this is the part that dominates); run dual-write with divergence alerting for one release (1–2 weeks). No domain, application or API change appears in that list, and that absence is the whole point of N4.

## 6. Principle Served

**P11** directly, **P5** (the canonical instance of the port rule), **N4**, and **P12** (the invariants are asserted against the trait, so they survive the implementation). Nothing is traded away except velocity, which `00 §3` ranks below correctness and operability.

## 7. Falsification Test

P11's own test, made runnable: **replace the ledger implementation with a stub that anchors to a local test chain; if any domain crate, API contract, or client requires changes, the abstraction failed.**

1. `cargo test -p fractal-node --features ledger-stub-chain` boots the Runtime against a stub `Ledger` and a local test-chain `Chain`, runs the full end-to-end suite, and must pass with zero diffs outside `crates/adapter/`.
2. `lint-deps` A2: `sqlx`, `postgres`, or any chain SDK appearing in the transitive closure of `fractal-domain-*` fails the build.
3. The port conformance suite runs identically against `ledger-internal`, the in-memory double, and the stub — a divergence in any is a failure.
4. `fractal-sim` asserts `11 §7` invariants 2, 3 and 4 after every step, against the in-memory implementation.

## 8. Maintenance Horizon

The trait and the internal implementation are first-party. Third-party exposure on this path is `sqlx` and `rust_decimal`, both pinned and both confined below the port. The long-horizon obligation is the anchor proof format: it is published, third parties verify against it (`16 §6.4`), and it therefore cannot change incompatibly without a versioned migration. Treat it as a public API from the first anchor written.

## 9. Review Trigger

Reopen the *implementation* (not the abstraction) when (a) posting throughput requires TigerBeetle per ADR-0006's trigger; (b) the Phase 8 gate opens and the FN L1 exists to anchor to; or (c) the `Chain` trait proves unable to express a real adapter's finality model without leaking chain concepts upward — at which point the trait is wrong and is revised before an adapter is written against it, never after.
