# ADR-0002 — Rust for the core Runtime

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 0

## 1. Context

P13 requires exactly one core Runtime with GUI, CLI, agents and plugins as peer front ends. N2 requires cross-platform from day zero, not a later port: `41 §8.1` makes core crates compile to `x86_64-msvc`, `x86_64-linux`, `aarch64-darwin`, `aarch64-ios`, `aarch64-android` and `wasm32-unknown-unknown` on every commit. `34 §2.1` states the structural consequence — the product is a library with several shells, and the desktop shell links that library in-process because a Node must hold an authoritative local replica (P2).

Those constraints eliminate most languages before performance is discussed. The core must compile to `wasm32` small enough for a browser tab, link into a Tauri process, and expose a UniFFI façade to Swift and Kotlin, from one source. It must also carry the two most correctness-sensitive subsystems: the double-entry Ledger, where `16 §5` forbids floating point, non-deterministic iteration and ambient time in the money path, and the Relay, with its `14 §3` latency budget under sustained fan-out.

`00 §3.4` sets the rule: prefer modern where the modern option removes an entire defect category. Memory safety without a garbage collector is such a category, and it sits on the two paths — key material and Fraction — where a use-after-free is not a crash but a loss.

## 2. Decision

We implement the Runtime, the client core, the CLI, and the agent executor in Rust, as one Cargo workspace whose layers are directories (`41 §4`). Rust is the only language in which a domain rule may be written (`34 §I1`): a validation, permission check, balance arithmetic, conflict resolution or XP formula implemented in TypeScript, Swift or Kotlin is a violation. Shell languages are permitted only above the binding layer. The workspace pins its toolchain in `rust-toolchain.toml` and enforces the `40 §2.3` clippy set, including the panic-surface, integer-conversion and determinism groups.

## 3. Consequences

### Positive
- One implementation of every domain rule on every target. `34 §2.3` budgets 55–71% of shipped source as shared core; two implementations of the XP formula would be two economies (P12).
- The `wasm32` target doubles as a purity oracle (`41 §8.1`): a domain crate that acquires a filesystem-touching transitive dependency fails the wasm build immediately, often before the dependency lint notices.
- No GC pauses in the Relay fan-out path, and no GC at all in the embedded desktop and mobile cores.
- The type system carries invariants that would otherwise be tests: `Quanta(u128)` with no `From<f64>` makes `16 §5`'s no-floating-point rule unrepresentable rather than forbidden.

### Negative
- **Slower initial velocity and a smaller hiring pool.** This is real and permanent, not a ramp-up cost, and it is the single largest staffing risk in the project.
- **Long compile times.** Mitigated by workspace splitting, `sccache`, and the shallow-graph rules of `41 §9.2`, but a change to `fractal-types` still triggers a full workspace rebuild — which is why `41 §5.1` requires a human reviewer on any PR touching it.
- Debugging the desktop app spans two runtimes and two toolchains (`34 §4.1`), and async Rust pushes us to `async_trait` plus `Arc<dyn Port>` at the port boundary for object safety at the composition root.

### Neutral / follow-on work
`unsafe` is governed by `40 §2.4` and requires its own ADR per use. The domain layer's third-party closure is allowlisted in `layers.toml`, so "Rust" does not imply "the crates.io ecosystem" inside the double-walled box of `10 §1`.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **Go** | Fast to write, excellent operational story, strong concurrency primitives, large hiring pool | No usable `wasm32` story for a *shared* core — TinyGo's subset excludes reflection-heavy serialization, and full Go's wasm output is multi-megabyte with a runtime per instance. That breaks N2 at the browser target, which is the target that makes P13 cheap. GC pauses also sit directly on the Relay's `14 §3` latency budget |
| **TypeScript everywhere (Node/Deno core)** | One language across core and web front end; the fastest possible Phase 1; the largest ecosystem | No memory or numeric guarantees on the ledger path. `16 §5` requires `u128` integer arithmetic with no floating point anywhere in the money path; JS numbers are `f64` and `BigInt` is a bolted-on second numeric tower. Also no path to an in-process embedded core on mobile, so the Node model degrades to a thin client (P2) |
| **Elixir/BEAM** | Genuinely superb for the Relay: supervision trees, per-connection processes, hot upgrades | Wrong for the other two thirds. The deterministic ledger wants integer-exact arithmetic and byte-identical canonical encoding (`16 §5`), and BEAM has no wasm or mobile embedding story, so the core could not be one library. We would be choosing a language for the one boundary `10 §2` already predicts extracting |

## 5. Exit Cost

**Unbounded — and therefore Rust is not a swappable choice, it is a foundation.** `00 §3.3` permits that only for a choice that cannot sit behind a P5 port, which a language cannot. Replacing Rust in 18 months means rewriting `fractal-domain-*`, `fractal-app-*`, `fractal-core` and the port layer: an estimated **80–120 engineer-weeks** for the Phase 3 surface, plus re-establishing every property test and the entire `fractal-sim` corpus, which is the part that cannot be ported at all. The mitigation is not exit; it is that the alternatives were evaluated against the constraint that actually binds (N2's target matrix) rather than against velocity.

## 6. Principle Served

**P8** (memory safety removes a defect class on the key and money paths), **P10** (no GC on the Relay and startup paths), **P13** and **N2** (one core that compiles to every target is *how* one-core-many-front-ends becomes cheap rather than aspirational), **P5** (the trait/port discipline is enforceable at compile time). Traded away: developer velocity, which the `00 §3` standing bias ranks below correctness and operability.

## 7. Falsification Test

The `targets` workflow (`41 §8.3`), required on every PR and never path-filtered, builds `fractal-types`, `fractal-schema`, `fractal-ports`, `fractal-domain-*` and `fractal-core` for all six target triples. No target may be marked `allow-failure`. A second check, `lint-langs`, greps the shell trees (`apps/web`, `apps/desktop`, `apps/ios`, `apps/android`) for domain-rule identifiers — balance arithmetic, XP or Trust computation, capability intersection, conflict resolution — and fails on a hit. The decision has been violated the moment a business rule exists twice.

## 8. Maintenance Horizon

Rust has a foundation, a six-week release train, and multiple independent commercial maintainers. The toolchain is pinned in `rust-toolchain.toml` with a documented bump cadence. The risk is not the language but individual crates: `40 §10.2` requires a vendoring plan for any single-maintainer critical-path dependency, and `layers.toml` caps the domain layer's transitive closure at a named allowlist so that risk cannot silently enter the pure core.

## 9. Review Trigger

Reopen if (a) the `wasm32` core payload exceeds 2.5 MB gzipped, at which point the browser target's `32 §8` budget is threatened and the shared-core premise for web needs re-scoring; (b) full-workspace clean build time on the reference CI machine exceeds 25 minutes despite `41 §9` mitigations; or (c) the project cannot fill a Rust role within two consecutive quarters, which converts the hiring-pool cost from a known trade into a delivery risk.
