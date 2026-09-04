# Phase 0 — Status

> Generated at the end of the first build session. `docs/50 PH0` defines the
> exit gate; this file records honestly which parts of it are met.

## Acceptance criteria (docs/50 PH0)

| # | Criterion | Status | Note |
|---|---|---|---|
| 1 | `cargo xtask verify` passes cold on Linux, macOS, Windows | **Partial** | Green on Linux. macOS and Windows unverified — no runner has executed yet. |
| 2 | Core compiles to x86_64, aarch64, wasm32 | **Partial** | x86_64 and wasm32 verified locally. aarch64 is in CI but has not run. |
| 3 | Regenerating from `fractal-schema` produces a zero diff | **Not met** | M0.4 not built. See below. |
| 4 | Regenerating tokens produces a zero diff across five targets | **Met** | `cargo xtask tokens --check` |
| 5 | Creating a Society via web, CLI and API produces identical event streams | **Partial** | Verified by hand — all three emit `society.created.v1` with identical shape. Not yet a test. |
| 6 | Simulation runs 2,000 seeded histories asserting three invariants | **Not met** | Fakes exist; the runner does not. |
| 7 | Fast-lane CI completes under 5 minutes | **Unverified** | Workflow written; no run yet. |

## What is actually built and green

- 13-crate Rust workspace, layered, with the dependency direction enforced by
  `cargo xtask lint-deps` (13 crates, 0 violations).
- 46 tests passing. The load-bearing ones: a Level 0 Citizen can found their
  first Society; an Agent cannot found one; a retried command creates one
  Society rather than two; replayed state equals the projection.
- Two `EventStore` implementations under a behavioural equivalence test
  (ADR-0016).
- The Policy Enforcement Point, refusing Policy-class actions to every
  non-Citizen principal, exhaustively.
- Design tokens: one source, five generated targets, drift-checked.
- API + CLI + web GUI, all over the same public API, all writing the same log.
- `cargo xtask verify`: format, clippy `-D warnings`, tests, and all three
  Canon gates — green.

## What remains before the gate closes

1. **M0.4 — schema-first codegen.** The largest gap. Today the API operation
   list and the CLI command tree are hand-written and cross-checked by
   `cargo xtask parity`. That works, and it is not the milestone: the milestone
   is that both are *generated* from `fractal-schema`, so drift is impossible
   rather than detected. Everything downstream (the SDKs, the OpenAPI document,
   the generated TypeScript client that replaces `apps/web`'s hand-written
   fetch calls and the CLI's hand-rolled HTTP client) waits on this.

2. **The simulation runner.** `fractal-testkit` has the deterministic fakes and
   they are in use. What is missing is the harness that generates histories from
   a seed and asserts the `docs/11 §7` invariants over them. Until it exists,
   ADR-0014's claim is architectural rather than demonstrated.

3. **Run CI once.** Every workflow here is unexecuted. A CI file that has never
   run is a hypothesis.

4. **A remote.** The repository is local-only. Push to an origin so the commit
   protocol gate has something to check against.

5. **Choose a licence.** `LICENSE` is a deliberate placeholder. It interacts
   with the marketplace (`docs/19`), the Extension SDK and self-hosted Nodes
   (`docs/50 PH6`), and should be decided with those in view.

## Note on the toolchain

`rust-toolchain.toml` moved from 1.83.0 to 1.98.1 during this session. The
initial conservative pin failed on the first real dependency: the crate
ecosystem now requires edition 2024. Recorded as ADR-0015.
