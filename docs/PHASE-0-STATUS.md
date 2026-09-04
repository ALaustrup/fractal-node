# Phase 0 — Status

> Generated at the end of the first build session. `docs/50 PH0` defines the
> exit gate; this file records honestly which parts of it are met.

## Acceptance criteria (docs/50 PH0)

| # | Criterion | Status | Note |
|---|---|---|---|
| 1 | `cargo xtask verify` passes cold on Linux, macOS, Windows | **Partial** | Green on Linux. macOS and Windows unverified — no runner has executed yet. |
| 2 | Core compiles to x86_64, aarch64, wasm32 | **Partial** | x86_64 and wasm32 verified locally. aarch64 is in CI but has not run. |
| 3 | Regenerating from `fractal-schema` produces a zero diff | **Met** | `cargo xtask codegen --check`: 6 artefacts, 0 drift. |
| 4 | Regenerating tokens produces a zero diff across five targets | **Met** | `cargo xtask tokens --check` |
| 5 | Creating a Society via web, CLI and API produces identical event streams | **Partial** | Verified by hand, and all three now share one generated client/table. Still not a test. |
| 6 | Simulation runs 2,000 seeded histories asserting three invariants | **Met** | `cargo xtask sim`: 2,000 × 40 = 80,000 operations, 0 violations. Asserts five properties, not three. |
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
- **Schema-first codegen (M0.4).** `crates/support/schema` is the contract;
  `cargo xtask codegen` emits the OpenAPI document, per-event JSON Schema, the
  gateway's operation table, the CLI command tree, and the TypeScript and
  JavaScript clients. `--check` fails on drift, so no surface can fall behind.
- **The simulation harness (M0.6).** 2,000 seeded histories × 40 steps = 80,000
  generated operations, asserting five properties after every single step:
  I-1 (every event names its owning Society, and sequences are dense), I-10
  (the projection always equals a fresh replay), I-14 (a sealed log never
  grows), idempotency (a retried command never mints a second Society), and P4
  (no Society was ever founded by a non-human principal). A failure names the
  seed and step, so it reproduces exactly.
- Two gates that catch different failures: `parity` proves every operation
  reached every generated surface; a CLI integration test proves the binary can
  actually *run* each one. The second was demonstrated to catch a gap the first
  waves through.
- `cargo xtask verify`: format, clippy `-D warnings`, tests, and all three
  Canon gates — green.

## What remains before the gate closes

1. **Run CI once.** Every workflow here is unexecuted. A CI file that has never
   run is a hypothesis.

2. **A remote.** The repository is local-only. Push to an origin so the commit
   protocol gate has something to check against.

3. **Choose a licence.** `LICENSE` is a deliberate placeholder. It interacts
   with the marketplace (`docs/19`), the Extension SDK and self-hosted Nodes
   (`docs/50 PH6`), and should be decided with those in view.

## Note on falsifying the simulation

Three bugs were deliberately injected to check the harness could see them: a
no-op `seal()`, a replay that drops a field, and a Policy Enforcement Point that
waves Agents through. The second and third were caught immediately.

**The first was not**, and that finding was worth more than the other two. I-14
was asserted after every step but never *exercised* — nothing in the simulation
ever tried to write to a sealed Society, so the assertion could not fire. The
step that was supposed to do it was a comment saying the invariant would handle
it. After adding a real append attempt, the same injected bug is caught at seed
0, step 29.

An invariant that is checked but never exercised is decoration, and the only way
to tell the difference is to break the thing on purpose.

## Note on the two gates

Adding an operation to the contract and not implementing it in the CLI passes
`parity` — the operation is present in all five generated surfaces, because the
generator put it there. `crates/bin/cli/tests/reachable.rs` walks the real
argument parser and catches it. Both were verified by deliberately breaking
them; a gate that has never failed is a gate nobody has tested.

## Note on the toolchain

`rust-toolchain.toml` moved from 1.83.0 to 1.98.1 during this session. The
initial conservative pin failed on the first real dependency: the crate
ecosystem now requires edition 2024. Recorded as ADR-0015.
