# ADR-0003 — Ship a modular monolith and extract services only on measured criteria

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 0

## 1. Context

The Runtime spans fourteen bounded contexts (`10 §3`, S1–S14) with genuinely different resource profiles: transcoding is GPU-bound and spiky, the Relay is egress-bound and latency-critical, the Ledger is transactional and small, the Agent executor runs semi-trusted work. That variety is the standard argument for microservices, and it is the argument that would consume the entire `02 §5` complexity budget before the spine sentence in `02 §2` is true.

The decisive fact is that **we do not yet know where the true seams are.** A service boundary is a permanent versioning and operational commitment; drawn wrong, it converts a function call into a distributed transaction and a refactor into a migration. `02 §7` forbids abstraction without two callers for exactly this reason, and `10 §2` extends it to deployment topology.

At the same time, the discipline of services is worth keeping. `41 §7` already provides it without the process boundary: `layers.toml` forbids `domain(A) → domain(B)`, an app crate may depend on exactly one domain crate, and cross-boundary needs are expressed as a trait the app crate declares and the composition root satisfies. A boundary that never had a compile-time edge can be moved across a process boundary without a rewrite.

## 2. Decision

The Runtime ships as **a single deployable binary** (`fractal-node`) containing internally bounded modules, with dependency direction enforced mechanically in CI, until measured load forces extraction.

A module is extracted into its own service when **two** of the following four are true, **measured, not predicted**:

| # | Criterion | Threshold |
|---|---|---|
| 1 | Divergent resource profile | >5× the rest of the Runtime in CPU, memory, GPU or egress |
| 2 | Failure isolation requirement | its outage must not take the platform down |
| 3 | Divergent deploy cadence | >5× the core's, e.g. many times a day against weekly |
| 4 | Distinct security boundary | requires OS-level isolation |

**Predicted extraction order — do not pre-build:** ① Media Transcoder (criterion 1, GPU/CPU spike), ② Relay/SFU (1 and 2, egress and isolation), ③ Agent Executor (1, 2, 4, semi-trusted work), ④ Search Indexer (1 and 3), ⑤ Custodian Coordinator (1). Everything else stays in the Runtime, possibly forever.

## 3. Consequences

### Positive
- Cross-boundary calls stay function calls, so a saga (`11 §5`) is a transaction rather than a distributed protocol with compensation over a network.
- One process to deploy, profile, trace and debug. `40 §9` telemetry carries `correlation_id` end to end without cross-service propagation bugs.
- Extraction stays cheap because `41 §7` already forbids the compile-time edges that make it expensive.
- Postgres transactions can span the event append and the projection update (`16 §4.1`), which is what makes P6's replay guarantee cheap at Phase 1.

### Negative
- **One process means one blast radius.** A panic in a hook path can drop connections for every Society on that replica; `40 §2.3`'s panic-surface lint set is the mitigation and it is not a proof.
- Scaling is uniform until the first extraction: a transcoding spike buys CPU for the Ledger too.
- The criteria require instrumentation that does not exist for free. Per-boundary RED metrics (`40 §9.3`) must be in place before any extraction argument can be made, which is itself work.
- The rule invites relitigation. Everyone has a service they are sure qualifies; only two measured criteria admit them.

### Neutral / follow-on work
The Relay has no domain crate (`41 §5.3`) precisely because it is transport, which pre-shapes extraction ②. `10 §9` already stages Relay and Transcoder extraction into Phase 4–6, so the predicted order is a roadmap input, not a surprise.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **Microservices from day one** | The boundaries are already named in `10 §3`; independent scaling and deploy; avoids a future extraction project | Premature boundary commitment. Fourteen services means fourteen deploy pipelines, fourteen sets of dashboards and a distributed transaction wherever `11 §5`'s sagas cross a boundary — and `02 §5` caps new top-level services at two per phase, so this is a budget violation on the first commit, not a preference |
| **Serverless functions** | Zero idle cost, elastic, no capacity planning | The Runtime is stateful: the event log's per-Society sequence, Signal subscriptions, presence and agent sessions all live across requests (`10 §5`, `14 §2`). Cold starts are also irreconcilable with P10's 1.5s desktop and 2.5s web startup budgets |
| **Actor system as the top-level architecture** | Erlang-style supervision maps well onto Societies and connections; strong isolation per actor | Rejected as the *primary* structure: it makes every intra-boundary call a message, which destroys the transactional guarantee `16 §4.1` depends on and makes the Ledger's `Σ debits == Σ credits` a distributed assertion. **Adopted *within* the Relay and Agent Executor**, where the fit is genuine |

## 5. Exit Cost

**Extraction, not exit — 3–5 engineer-weeks per module, and it is a planned cost rather than a penalty.** Concretely, per module: define the port trait it already implicitly satisfies, add a remote adapter behind it, split the composition root, stand up its deploy pipeline and dashboards, and add contract tests to the `40 §7.7` suite. The number is that low only because `41 §7` prevents the compile-time edges; if the dependency lint were disabled for a quarter, the same extraction would cost 15–20 engineer-weeks of untangling. The lint is the exit-cost control.

## 6. Principle Served

**P10** (operability and startup budgets), **P5** (the module boundaries are already trait-shaped, so extraction is a swap), **P1** (the Society partition, not the service, is the scaling unit — see ADR-0004). Serves `02 §5` directly by not spending the service budget. No principle is traded away; failure isolation is deferred, not abandoned, and criterion 2 is the trigger that buys it.

## 7. Falsification Test

Three mechanical checks, all in the unfilterable `lint-deps` job (`41 §7.2`):

1. **A1 layer edges** — every internal edge satisfies `layers.toml`. A domain crate reaching an adapter, or an app crate depending on a second domain crate, fails the build.
2. **A2 third-party closure** — each layer's full transitive external closure is a subset of its allowlist, which catches `tokio` arriving in the domain layer three hops down.
3. **Extraction justification** — a PR that adds a crate under `crates/bin/` beyond `fractal-node`, `fractal-cli` and `fractal-agent` must cite an ADR naming which two of the four criteria were measured, with the metric and the observation window. An extraction argued from prediction rather than measurement is the violation this ADR exists to prevent.

## 8. Maintenance Horizon

First-party structure with no external dependency; the enforcement is `layers.toml` plus roughly 400 lines of `xtask`. The standing risk is social rather than technical — the lint is the most likely rule to be suppressed under deadline pressure (`41 §5.5` says so explicitly), so `lint-deps` is a required check that may not be path-filtered or skipped with a label.

## 9. Review Trigger

Reopen when (a) any module meets two criteria under measurement — extract it, on the predicted order where applicable; (b) p99 Runtime restart time exceeds 30 seconds, which means the single binary has grown past the point where deploys are cheap; or (c) a single-boundary incident causes platform-wide unavailability twice in a quarter, which is criterion 2 asserting itself through outages instead of through metrics.
