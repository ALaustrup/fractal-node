# 40 — Engineering Standards

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), plus `10-system-architecture.md` and `11-domain-model.md`.
> **Governs:** code style and lints, documentation, ADRs, the testing strategy, CI/CD, observability, security engineering, backup and disaster recovery, accessibility engineering, performance engineering, code review, and the working agreements binding AI coding agents.
> **Does not govern:** repository and crate layout (`41-repository-structure.md`) or commit, branch, and release automation (`42-source-control-automation.md`). This document says what quality means; those two say where files live and how changes land.

---

## 1. Position

Fractal Node is written substantially by AI coding agents against a specification that is larger than any one reviewer can hold in working memory. That single fact determines the entire shape of this document.

An agent will satisfy any constraint you state and violate any constraint you leave implicit. A human reviewer reading the eleventh pull request of the day will approve a plausible diff. Neither of those failure modes is fixed by exhortation. They are fixed by making the constraint **mechanical**: a lint, a property test, a CI gate, a budget with a number attached. Everything in this document that could be a rule enforced by a machine is written as one; everything that genuinely requires human judgement is named explicitly so that reviewers spend their scarce attention there and nowhere else.

The standing bias from `00 §3` applies throughout: **correctness > operability > developer velocity > raw benchmark performance.** Where a standard here slows an agent down, that is the intended trade. Agent throughput is abundant; reviewer attention and production correctness are not.

Three rules subsume most of what follows:

1. **If it is not enforced, it is not a standard.** It is a preference, and preferences drift within two phases.
2. **A gate that is routinely bypassed is worse than no gate**, because it teaches the team that gates are negotiable. Delete it or make it pass.
3. **The definition of done is the eight points in `00 §5`.** Everything in this chapter exists to make one of those eight points checkable.

---

## 2. Rust Code Standards

### 2.1 Toolchain pinning

The toolchain is pinned in `rust-toolchain.toml` at the workspace root and is identical on every developer machine, every agent sandbox, and every CI runner. An agent that cannot reproduce a CI failure locally will "fix" the wrong thing.

```toml
# rust-toolchain.toml
[toolchain]
channel    = "1.83.0"          # exact patch; bumped by a dedicated PR, never incidentally
components = ["rustfmt", "clippy", "rust-src", "llvm-tools-preview"]
targets    = [
  "x86_64-unknown-linux-gnu",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "wasm32-unknown-unknown",    # N2: core crates compile to wasm from Phase 0
]
profile    = "minimal"
```

Edition **2021** across the workspace. `resolver = "2"`. The `wasm32` target is in the pinned set from Phase 0 because N2 says cross-platform is never a later port — a core crate that stops compiling to `wasm32` fails CI on the commit that broke it, not in Phase 5 when someone finally tries.

Toolchain bumps are their own PR, contain no other change, and must show a clean `cargo +new clippy --workspace` plus a green full lane. Rust releases every six weeks; we adopt on a **two-release lag** (roughly twelve weeks behind stable) so that regressions surface in other people's CI first. Nightly is used for exactly two things — `cargo fmt` unstable options and `cargo udeps` — both in non-blocking CI jobs, never in a build that produces a shipped artifact.

### 2.2 Formatting

Formatting is not a matter of taste and is never discussed in review. `cargo fmt --check` is the first thing the fast lane runs and the cheapest possible signal.

```toml
# rustfmt.toml
edition                     = "2021"
max_width                   = 100
use_small_heuristics        = "Max"
imports_granularity         = "Crate"      # nightly-gated; applied by the format job
group_imports               = "StdExternalCrate"
newline_style               = "Unix"
use_field_init_shorthand    = true
match_block_trailing_comma  = true
normalize_comments          = true
wrap_comments               = true
comment_width               = 100
format_code_in_doc_comments = true
format_macro_matchers       = true
```

`format_code_in_doc_comments` matters more than it looks: documentation examples are compiled and run (§5.3), so they are code, and code is formatted.

### 2.3 The clippy lint set

Lints are declared once in the workspace manifest and inherited by every crate. A per-crate deviation requires a comment naming the reason and expires with a `TODO(#issue)`.

```toml
# Cargo.toml (workspace root)
[workspace.lints.rust]
unsafe_code            = "forbid"
missing_docs           = "warn"     # denied for public items in `fractal-*` libs, see §5
unreachable_pub        = "deny"
rust_2018_idioms       = "deny"
unused_qualifications  = "deny"

[workspace.lints.clippy]
# Baseline groups
all            = { level = "deny", priority = -1 }
pedantic       = { level = "warn", priority = -1 }
cargo          = { level = "warn", priority = -1 }

# Panic surface — a Runtime that panics is a Node that dropped a Society
unwrap_used             = "deny"
expect_used             = "deny"
panic                   = "deny"
unimplemented           = "deny"
todo                    = "deny"
indexing_slicing        = "deny"
integer_arithmetic      = "warn"   # denied in ledger crates, see below
unwrap_in_result        = "deny"
exit                    = "deny"

# Correctness of conversions — `as` silently truncates and we move money
as_conversions          = "deny"
cast_possible_truncation= "deny"
cast_sign_loss          = "deny"
cast_precision_loss     = "deny"

# Determinism — required for the simulation harness (§6.4)
disallowed_types        = "deny"   # HashMap/HashSet in domain crates; see clippy.toml
disallowed_methods      = "deny"   # SystemTime::now, rand::thread_rng, Instant::now

# Async and locking hygiene
await_holding_lock      = "deny"
await_holding_refcell_ref = "deny"

# Style
mod_module_files        = "deny"
str_to_string           = "deny"
dbg_macro               = "deny"
print_stdout            = "deny"   # allowed only in the CLI crate and examples
```

Additional lints applied **only to ledger and economy crates** (`fractal-domain-ledger`, `fractal-domain-economy`, `fractal-adapter-ledger-*`):

```toml
[lints.clippy]
float_arithmetic    = "deny"   # 17 §2.3: no floating point in money, ever
float_cmp           = "deny"
integer_arithmetic  = "deny"   # forces checked_add / checked_sub / saturating_*
arithmetic_side_effects = "deny"
```

The reasoning is not stylistic. `11 §2.6` states the ledger invariants — `Σ debits == Σ credits`, `balance >= locked >= 0`, total supply is `-EmissionAccount.balance`. Every one of those is destroyed by a silent overflow or a float rounding error, and neither is visible in review. Denying the operation is the only reliable prevention. Ledger arithmetic is written as `a.checked_add(b).ok_or(LedgerError::Overflow)?` and the noise is the point.

`unwrap_used` and `expect_used` are denied in all production code and **allowed in test code only**, via the standard `#[cfg(test)]` module attribute or the `tests/` directory, where a panic is the reporting mechanism. There is no third category. A test helper compiled into a production binary is a production path.

`disallowed_methods` in `clippy.toml` names the determinism traps directly:

```toml
# clippy.toml
disallowed-methods = [
  { path = "std::time::SystemTime::now",  reason = "use the Clock port (10 §7)" },
  { path = "std::time::Instant::now",     reason = "use the Clock port (10 §7)" },
  { path = "rand::thread_rng",            reason = "use the Rng port (10 §7)" },
  { path = "uuid::Uuid::new_v4",          reason = "use the IdGen port (10 §7)" },
]
disallowed-types = [
  { path = "std::collections::HashMap", reason = "non-deterministic iteration; use BTreeMap in domain crates" },
  { path = "std::collections::HashSet", reason = "non-deterministic iteration; use BTreeSet in domain crates" },
]
```

This is the lint that makes §6.4 possible. Ambient time, ambient randomness, and ambient identifiers are the three things that make a system unreplayable, and every one of them is a one-line call that looks harmless in a diff.

### 2.4 `unsafe` policy

`#![forbid(unsafe_code)]` is workspace-wide. `forbid`, not `deny`: `deny` can be overridden by an inner `#[allow]` that a reviewer will not notice; `forbid` cannot be overridden at all, which is exactly the property we want.

The exception process, in full, because a blanket ban with no escape hatch gets circumvented rather than followed:

1. `unsafe` may exist only in a crate named `fractal-unsafe-<purpose>` that does one thing and exposes a safe API.
2. The crate carries an ADR stating the specific capability that cannot be obtained safely, the measured benefit if the motivation is performance, and the invariants the caller must uphold.
3. Every `unsafe` block carries a `// SAFETY:` comment naming the invariant it relies on and why it holds. A block without one fails review automatically.
4. The crate is in the fuzzing set (§6.9) and runs under Miri in the full lane.
5. Two human reviewers, one of whom did not write it, and neither of whom may be an agent.

Expected members of this set through Phase 6: **zero**. Cryptography comes from audited crates, media codecs sit behind FFI in an adapter crate that is itself `forbid(unsafe_code)` at the Rust level with the FFI surface isolated, and there is no performance problem in this system that `unsafe` solves and profiling does not.

### 2.5 Error handling doctrine

| Layer | Error type | Rule |
|---|---|---|
| Domain crates (`fractal-domain-*`) | `thiserror` enum, exhaustive, `#[non_exhaustive]` | Errors are domain facts. Every variant is a case a caller can distinguish and act on. |
| Ports (`fractal-ports`) | `thiserror` enum per port | Port errors are classified `Transient` / `Permanent` / `Precondition` so callers can decide retry without string matching. |
| Adapters | Wrap the vendor error, never leak the type (P5) | `#[from] sqlx::Error` inside the adapter crate is fine; the port's error must not name it. |
| Application layer | `thiserror`, mapped to API problem types | One-to-one map from domain error to RFC 9457 `problem+json` and to a CLI exit code. |
| Binary edges only (`main.rs`, CLI commands, test harnesses) | `anyhow` | The only place context-chaining without typed variants is acceptable, because the only consumer is a human reading a message. |

Rules that follow:

- **No `unwrap` or `expect` in any production path.** If a value is provably present, encode that in the type (`NonZeroU32`, a newtype with a validating constructor, a `Vec1`) rather than asserting it at runtime.
- **No `panic!` as control flow.** The Runtime hosts many Societies in one process (`10 §2`); a panic that unwinds a task can leave a projection half-applied. Errors propagate as values.
- **Every fallible boundary returns `Result`.** A function that cannot fail returns the value directly — a `Result<T, Infallible>` is a design smell that means the boundary is in the wrong place.
- **`?` at the call site, context at the edge.** Domain code does not decorate errors with prose; the application layer maps them to a user-facing problem type with a `correlation_id`.
- **Errors never carry secrets.** Types marked `#[secret]` (§4.4) cannot appear in an error's `Display`, enforced by lint.

Ledger code goes one step further: `LedgerError` has no `Other` variant, mirroring the closed `PostingReason` enum from `11 §2.6`. If a new failure is possible, it gets a name.

---

## 3. TypeScript Standards

TypeScript exists in the web GUI, the Tauri shell glue, and the SDK. It is held to the equivalent standard, adjusted for a language whose type system is unsound by design.

```jsonc
// tsconfig.base.json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,      // the TS equivalent of indexing_slicing
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true,
    "noFallthroughCasesInSwitch": true,
    "noPropertyAccessFromIndexSignature": true,
    "useUnknownInCatchVariables": true,
    "verbatimModuleSyntax": true,
    "isolatedModules": true,
    "target": "ES2022",
    "moduleResolution": "bundler",
    "skipLibCheck": false
  }
}
```

ESLint rules that are errors, not warnings: `@typescript-eslint/no-explicit-any`, `no-unsafe-assignment`, `no-unsafe-member-access`, `no-unsafe-call`, `no-unsafe-return`, `no-floating-promises`, `await-thenable`, `switch-exhaustiveness-check`, `consistent-type-imports`, `no-restricted-imports` (see §4.1).

Two rules carry disproportionate weight:

- **`no-explicit-any` with no escape.** `unknown` plus a parse is always available. Every value crossing the API boundary is validated by a generated **Zod** schema derived from the OpenAPI document — not hand-written, because hand-written validators drift from the contract they claim to enforce (§6.7).
- **Exhaustive switches over domain unions.** Every domain enum that reaches the client — `ChamberKind`, `MembershipState`, `SocietyStatus`, `PostingReason` — is discriminated, and every `switch` ends in a `default: assertNever(x)`. When `11` adds a variant, the client fails to compile. That is the desired behaviour: a silently unhandled `SocietyStatus::Fracturing` is a UI that lies about a Society mid-fracture.

React specifics: function components only, no class components, no `useEffect` for derived state, TanStack Query for all server state (never in component state), and no direct `fetch` outside the generated SDK client — enforced by `no-restricted-imports`, which is also how P3's falsification test ("grep the clients for HTTP calls to undocumented internal paths") becomes a lint instead of an audit.

---

## 4. The Canon-Enforcing Lints

Four custom lints exist because four Canon rules are otherwise unenforceable by review. They run in the fast lane as a single `cargo xtask lint-canon` step (under 20s on a warm cache) and their failure messages cite the principle and document section, so an agent that trips one is told exactly which rule it broke.

### 4.1 Dependency direction (P5, `10 §3`)

Implemented with `cargo-deny`'s `bans` section plus an `xtask` that parses `cargo metadata` and asserts the layering:

```
front ends ──► gateway ──► application ──► domain ──► ports
                                                        ▲
                                            adapters ───┘

FORBIDDEN: fractal-domain-*    ──► fractal-adapter-*
FORBIDDEN: fractal-domain-A    ──► fractal-domain-B (internals)
FORBIDDEN: fractal-domain-*    ──► sqlx | aws-sdk-* | reqwest | tokio (runtime) | any vendor SDK
FORBIDDEN: fractal-web/desktop ──► anything but fractal-sdk
```

The lint additionally scans domain crate source for `use` statements naming any crate on the vendor list, catching the case where a dependency arrives transitively through a feature flag. This is the direct mechanical implementation of P5's falsification test.

### 4.2 No literal colour (N7, `32-design-system.md`)

A regex lint over `.css`, `.tsx`, `.rs`, `.swift`, `.kt` denying `#RRGGBB`, `rgb(`, `hsl(`, and raw ANSI SGR colour codes outside the generated token files. N7 says the design system is single-source across every surface *including the CLI*; a hard-coded hex is a fork of the design system that no one will ever find again. Exactly one directory is exempt: the token pipeline's own output.

### 4.3 No banned terminology (`01 §10`)

A lint over identifiers, doc comments, API paths, CLI help text, migration names, and commit subjects, denying the banned list: `user`, `server`, `channel`, `room`, `forum`, `NFT`, `gas`, `mining`, `feed`, `algorithm`, `karma`, `points`, `admin`, `microservice`, and `decentralized` used as a bare claim. Each hit prints the canonical replacement from `01 §10`.

Necessary carve-outs, allowlisted by path and reviewed at each phase gate: third-party type names we do not control (`std::net::TcpListener`, HTTP `User-Agent`), quoted external specifications, and the `docs/` prose in `01` itself which must name the banned words to ban them. The allowlist is short and every entry has an owner. Terminology drift is slow, invisible, and by the time it is obvious the schema has already forked — this lint is cheap insurance against the failure mode `01` exists to prevent.

### 4.4 Secret serialization (`10 §10`, P8)

Types wrapping key material, tokens, recovery phrases, or session secrets are marked `#[secret]`, a derive macro that:

- implements `Debug` and `Display` as the literal string `[redacted]`;
- **refuses to implement `Serialize`** — a `#[secret]` type in a `#[derive(Serialize)]` struct is a compile error, not a runtime redaction;
- implements `Zeroize` and `Drop`;
- registers the type with the lint, which then denies its appearance in any event payload struct, any `tracing` macro argument, any error variant, and any OpenAPI-exposed type.

P8 says secrets never touch the event log. Runtime redaction is a mitigation; a type that cannot be serialized is a guarantee. The distinction matters because the event log is permanent (P6) — a secret written there cannot be unwritten, only re-keyed.

---

## 5. Documentation Standards

### 5.1 The shipping rule

From `00 §5`: an undocumented public API is not shipped. Operationally, `#![warn(missing_docs)]` in every crate and `#![deny(missing_docs)]` in every crate whose name does not end in `-internal`. `cargo doc --no-deps` runs with `RUSTDOCFLAGS="-D warnings"`, so a broken intra-doc link fails the build like any other error.

### 5.2 The doc comment template

Every public item answers four questions in this order. Prose beyond this is welcome; less than this is incomplete.

```rust
/// Records a balanced Posting pair against two Wallets.
///
/// Every Fraction movement in the platform is one or more Postings; there is no
/// other write path to a balance (`11 §2.6`). The debit and credit are applied in
/// a single transaction, so a partially-applied Posting is not observable.
///
/// # Invariants
/// - `Σ debits == Σ credits` holds after this call (invariant 2, `11 §7`).
/// - `balance >= locked >= 0` holds for both Wallets (invariant 3).
///
/// # Errors
/// - [`LedgerError::InsufficientFunds`] if the debit Wallet's unlocked balance is
///   below `amount`. No partial transfer occurs.
/// - [`LedgerError::Overflow`] if the credit would exceed `Quanta::MAX`.
/// - [`LedgerError::WalletFrozen`] if either Wallet belongs to a Society with
///   `status = Fracturing` (`11 §3.2` seals the Treasury).
///
/// # Examples
/// ```
/// # use fractal_domain_ledger::{Ledger, PostingReason, Quanta};
/// # let mut ledger = fractal_domain_ledger::testing::in_memory();
/// # let (from, to) = ledger.testing_wallets(Quanta::from_frc(10), Quanta::ZERO);
/// let posting = ledger.post(from, to, Quanta::from_frc(3), PostingReason::Transfer)?;
/// assert_eq!(ledger.balance(from)?, Quanta::from_frc(7));
/// assert!(ledger.is_balanced());
/// # Ok::<(), fractal_domain_ledger::LedgerError>(())
/// ```
pub fn post(&mut self, debit: WalletId, credit: WalletId,
            amount: Quanta, reason: PostingReason) -> Result<Posting, LedgerError>
```

The `# Errors` section is mandatory on every function returning `Result` and must enumerate every variant reachable from this call, with the state left behind. "Returns an error if something goes wrong" is a rejected review.

### 5.3 Doctests are tests

Every example in the template above compiles and runs in the full lane. This is deliberate load-bearing redundancy: examples are the documentation most likely to rot and the documentation most likely to be copied verbatim by the next agent. A doctest that fails is a broken build, identical in severity to a unit test failure. `no_run` is permitted for examples requiring a live adapter; `ignore` requires a comment explaining why and is reviewed at the phase gate.

### 5.4 Module and crate documentation

Module-level `//!` docs explain **why the module exists and what invariant it protects** — not what its functions are named, which the reader can see. The template:

```
//! # Envelope evaluation
//!
//! ## Why this module exists
//! P4 requires that policy is authored by humans and that no Agent can widen its own
//! authority. This module is the Policy Enforcement Point named in `10 §8`. It sits in
//! the application layer, on the path every command takes, because a check that lives
//! in the gateway can be bypassed by a second front end and a check that lives in the
//! agent can be bypassed by the agent.
//!
//! ## Invariants protected
//! - Invariant 5 (`11 §7`): no Envelope grants a capability its grantor lacks.
//! - Invariant 7: every Agent action event carries a valid, unexpired `envelope_ref`.
//!
//! ## What lives elsewhere
//! Envelope *storage* is in `fractal-adapter-store`; Envelope *granting* UX is in `12`.
```

Every crate carries a `README.md` with: one-sentence purpose, its layer (domain / port / adapter / application / front end), the ports it depends on, the invariants it owns, the principles it serves, and how to run its tests in isolation. The README is what an agent reads first when assigned work in that crate; it is a working document, not a courtesy.

Changelogs follow Keep-a-Changelog with a line per user-visible change, which is `00 §5` criterion 5. Generation from commits is specified in `42`.

---

## 6. Architecture Decision Records

### 6.1 When an ADR is required

`00 §3` requires one for every technology choice. Concretely, an ADR is required — and CI blocks the PR without one — when a change:

1. introduces, replaces, or removes a third-party runtime dependency;
2. adds or changes a **port** in `fractal-ports` (P5 surface);
3. changes a domain invariant in `11 §7`, an event schema in a breaking way, or the Global Registry in `01 §6`;
4. adds an economic Source or Sink (`17`), or changes an emission parameter;
5. changes a security posture: cryptographic primitive, key custody, authn/authz flow, or the E2EE boundary;
6. amends the Canon (`00 §7`);
7. introduces `unsafe` (§2.4);
8. crosses a `02 §5` complexity budget line — new top-level service, new public API resource family, new client platform.

Anything else does not need one. ADR inflation is real: a corpus of 300 ADRs is a corpus nobody reads, which is the same as no corpus with extra ceremony.

### 6.2 Location, numbering, lifecycle

ADRs live in `docs/adr/NNNN-kebab-title.md`, numbered sequentially from `0001`, never renumbered, never deleted. A superseded ADR stays in place with its status changed and a forward link — the record of what we believed and why we stopped believing it is the actual value of the practice.

```
 Proposed ──► Accepted ──┬──► Superseded by NNNN
     │                   └──► Deprecated (no longer relevant; nothing replaced it)
     └──► Rejected (kept; the reasoning prevents the idea returning every six months)
```

`Proposed` ADRs are the ones that count against `02 §5`'s **zero open ADRs at a phase gate** rule. That rule is not bureaucratic: an open ADR is an unresolved architectural question, and code written on top of an unresolved question is code that will be rewritten. At the gate, every proposal is accepted, rejected, or explicitly deferred by converting it to a `docs/proposals/` entry with a named phase.

### 6.3 The template

Nine required sections. Sections 6, 7, 8, and 9 are the ones most templates omit and the ones that make the record useful two years later.

```markdown
# ADR-NNNN — <Title: the decision, stated as a decision>

**Status:** Proposed | Accepted | Rejected | Superseded by ADR-MMMM | Deprecated
**Date:** YYYY-MM-DD
**Deciders:** <humans; agents may draft but may not decide>
**Phase:** <roadmap phase this lands in>

## 1. Context
The forces in play. What is true today, what pressure created this decision, what
constraints are fixed. No solutions here.

## 2. Decision
One paragraph, present tense, active voice: "We use X for Y." Then the specifics.

## 3. Consequences
### Positive
### Negative        ← must be non-empty. A decision with no downside is not understood.
### Neutral / follow-on work

## 4. Alternatives Considered
| Alternative | Why it was plausible | Why rejected |
Each rejection must be a fact about the alternative, not a preference. "Less popular"
is not a rejection reason; "no wasm32 target, which breaks N2" is.

## 5. Exit Cost
Engineer-weeks to replace this in 18 months, and the specific work involved. If the
answer is unbounded, this must sit behind a P5 port or be rejected (`00 §3.3`).

## 6. Principle Served
Cite P#. If a principle is traded away, cite the conflict order in `00 §2` and show
this decision follows it.

## 7. Falsification Test
How a reviewer or agent proves, mechanically, that this decision has been violated or
has stopped holding. Name the test, lint, or metric. A decision without one decays.

## 8. Maintenance Horizon
Who maintains the dependency in five years. Single-maintainer critical-path
dependencies require the vendoring plan from `00 §3.5` / §9.2 here.

## 9. Review Trigger
The condition that reopens this decision — a metric threshold, a phase boundary, a
named upstream event.
```

### 6.4 Worked example

The following is a real ADR in the corpus — `adr/0014-deterministic-simulation-testing.md` — reproduced in full because a template without an example gets filled in shallowly.

**The block below is a transclusion point, not a copy.** `cargo xtask lint-docs` renders it from the ADR file and fails on a diff. It was previously numbered ADR-0009 (which is the native Facet asset standard) and dated 2025-11-14, in a chapter whose subject is ADR discipline — the corpus's most visible self-contradiction, and drift with a mechanical fix (`61 X1`).

```markdown
# ADR-0014 — Deterministic simulation as the primary correctness gate for domain logic

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 1

## 1. Context
`10 §7` makes `Clock`, `Rng`, and `IdGen` ports. `11 §7` states fifteen invariants that
must hold over every reachable history. P6 requires that every projection be
reconstructible by replaying the log. Together these mean the domain layer is a pure
function from (initial state, ordered inputs) to (events, state) — there is no ambient
nondeterminism inside the double-walled box in `10 §1`.

The bugs that will actually hurt this system are not single-function bugs; they are
interleaving bugs. A Fracture that races a Transfer. A settlement run that overlaps a
Charter amendment. A revoked Envelope with an action in flight. Example-based tests do
not find these, because a human (or an agent) writing a test writes the interleaving
they already thought of.

## 2. Decision
We build a deterministic simulation harness (`fractal-sim`) that drives the whole
domain and application layer against in-memory port implementations whose behaviour —
time, scheduling order, message delivery, partial failure, retry, partition — is
derived from a single u64 seed. Each run generates a random but legal history of
commands across multiple Societies, executes it, and asserts all fifteen `11 §7`
invariants after every step. A failing seed is a complete, replayable bug report; a
shrinker reduces it to a minimal history before it is filed. The harness runs
continuously in the full lane and nightly at high volume, and every fixed bug's seed
is added to a permanent regression corpus.

## 3. Consequences
### Positive
- Finds concurrency and ordering defects that no other technique in our budget finds.
- Every failure is deterministically reproducible from `seed + commit`, which removes
  the single worst debugging experience in distributed systems.
- Forces the port discipline to stay honest: any ambient `SystemTime::now` breaks
  replay loudly on the commit that introduces it, not months later.
- Directly discharges P6's falsification test as a continuously running assertion.

### Negative
- The harness is real engineering: roughly 6 engineer-weeks to first value and an
  ongoing tax, because every new port needs a deterministic double and every new
  invariant needs an assertion. This cost is permanent.
- It biases us toward keeping the domain pure, which occasionally makes a feature more
  awkward than the direct implementation would be. We consider this a benefit; it is
  listed here because it is a real constraint on future design.
- It gives no coverage of the real adapters. Postgres, S3, and NATS misbehaviours are
  invisible to it and need integration tests (§7.6) regardless.

## 4. Alternatives Considered
| Alternative | Why plausible | Why rejected |
|---|---|---|
| Property tests only (proptest per invariant) | Much cheaper; we do this anyway | Operates on one aggregate at a time; cannot express multi-Society interleaving, which is where Fracture and settlement bugs live |
| Chaos testing against a live deployment | Tests the real stack including adapters | Non-reproducible; a failure is an anecdote, not a test case. Also finds bugs after deploy, not before merge. Adopted later as a complement, not a substitute |
| Formal methods (TLA+ for Fracture and Ledger) | Strongest possible guarantee for the two riskiest operations | Verifies the model, not the code, and the gap between them is where our bugs will be. We do write a TLA+ spec for Fracture specifically (`11 §3.2`) as an input to the harness's invariants, but it does not replace executable testing |
| Buy a testing platform | No build cost | Nothing on the market simulates *our* domain invariants; the value is entirely in the invariant assertions, which are ours |

## 5. Exit Cost
Effectively zero as an *exit*: the harness is additive and deleting it removes
assurance, not capability. The cost is in the coupling it creates — 3–4 engineer-weeks
to retrofit deterministic doubles if a future port is introduced without one, which is
why "ships with a deterministic double" is a merge requirement for every new port.

## 6. Principle Served
P6 (event-driven, replayable) directly; P12 (economically honest — invariants 2, 3, 4
are economic and are asserted on every simulated step); P5 (the port discipline this
depends on is enforced by the harness's existence). No principle is traded away.

## 7. Falsification Test
`cargo sim --seeds 10000` must pass on every commit to `main`, and the harness must
fail within 200 seeds when any single `11 §7` invariant assertion is deliberately
inverted. That second half is a real CI job (`sim-mutation`) — a harness that cannot
detect a broken invariant is decorative, and this proves weekly that ours can.

## 8. Maintenance Horizon
First-party code; no external maintainer risk. Dependencies are `proptest` (shrinking)
and `rand` with a pinned reproducible PRNG algorithm. If `proptest` were abandoned, the
shrinker is ~500 lines to replace.

## 9. Review Trigger
Reconsider scope if (a) a production incident's root cause was reachable by the harness
but not found within 100k seeds, indicating the generator's distribution is wrong, or
(b) full-lane simulation time exceeds 15 minutes, at which point volume moves nightly
and the per-PR run becomes a fixed regression corpus plus 1,000 fresh seeds.
```

### 6.5 Review process

An ADR is a PR. Agents may draft ADRs — they are good at enumerating alternatives — but **an agent may not move an ADR to `Accepted`**. That is a human act, consistent with P4: policy is authored by humans, and architecture is policy. Review turnaround target is 3 working days; an ADR open longer than 10 days is escalated at the weekly gate, because an open ADR blocks the code that depends on it.

---

## 7. Testing

### 7.1 Risk classes

Test depth is set by risk class, not uniformly. A uniform bar over-tests display code and under-tests the ledger.

| Class | Contents | Required levels | Coverage floor | Mutation score floor |
|---|---|---|---|---|
| **R0 — Catastrophic** | Ledger, emission, Fracture, Envelope evaluation, E2EE key handling, Charter enactment | Unit + property + simulation + fuzz + mutation + adversarial review | 95% line / 90% branch | 85% |
| **R1 — Serious** | Event append/replay, projections, Vault manifests, Standing/XP, Attestation scoring, saga compensation | Unit + property + simulation + integration | 85% / 75% | 70% |
| **R2 — Standard** | Command/query handlers, API surface, CLI verbs, Relay fan-out | Unit + integration + contract | 75% | — |
| **R3 — Peripheral** | Presentation, formatting, dev tooling, non-user-facing telemetry glue | Unit where behaviour is non-obvious | none | — |

The class is declared in the crate README and asserted by the coverage job. Promoting a crate from R2 to R1 is a normal PR; demoting requires a reviewer sign-off, because demotion is how standards quietly erode.

### 7.2 The pyramid and its target ratios

```
                       ┌─────────────────┐
                       │   E2E  ~0.5%    │   < 20 total. Minutes, not seconds.
                     ┌─┴─────────────────┴─┐
                     │  Contract   ~4%     │  every API + event schema
                   ┌─┴─────────────────────┴─┐
                   │   Integration   ~10%    │  real adapters, testcontainers
                 ┌─┴─────────────────────────┴─┐
                 │  Simulation (seeds)  ~1%    │  ← counted as tests, but each
                 │  covers the whole domain    │    seed is a whole history
               ┌─┴─────────────────────────────┴─┐
               │   Property tests     ~10%       │  the 15 invariants + more
             ┌─┴─────────────────────────────────┴─┐
             │        Unit tests      ~75%         │  pure domain logic
             └─────────────────────────────────────┘
```

Ratios are descriptive targets that we check quarterly, not gates. The gate that matters is the risk-class table.

### 7.3 Unit tests

Pure domain logic, no I/O, no async runtime, no ports beyond in-memory doubles. A domain unit test that needs `tokio::test` indicates I/O has leaked into the domain layer and is a design failure, not a test problem.

Naming — one convention across the workspace, and it is enforced by a lint on `#[test]` function names:

```rust
fn <unit_under_test>__<condition>__<expected_outcome>()

// examples
fn post__debit_wallet_below_amount__returns_insufficient_funds()
fn envelope_eval__expired_envelope__denies_and_emits_agent_action_blocked()
fn crystallize__two_of_three_accept__does_not_create_society()
```

Three segments, double underscore separators. This reads badly and greps beautifully, which is the correct trade for a corpus that will exceed 20,000 tests and be navigated mostly by agents. Test bodies follow arrange / act / assert with a blank line between, one logical assertion per test, and no branching — a test with an `if` is two tests.

### 7.4 Property tests — the fifteen invariants

Each of the fifteen invariants in `11 §7` is a named property test in `fractal-domain-*/tests/invariants/`, using `proptest`. They are also asserted by the simulation harness (§7.5); the standalone property tests exist because they fail *faster* and *smaller*, giving a tighter loop.

Worked example — invariant 5, "no Envelope grants a capability its grantor lacks", which is P4's structural guarantee against privilege escalation:

```rust
proptest! {
    #![proptest_config(ProptestConfig { cases: 2048, max_shrink_iters: 8192, ..Default::default() })]

    /// Invariant 5 (`11 §7`): an Envelope can never contain a capability the granting
    /// Citizen does not hold at grant time. This is the mechanism by which P4's
    /// "an agent may never widen its own envelope" is structural rather than policed.
    #[test]
    fn invariant_05__granted_envelope_never_exceeds_grantor_capabilities(
        charter   in arb_charter(),
        grantor   in arb_membership_with_roles(),
        requested in arb_capability_set(),
        ttl       in 1u32..=90,
    ) {
        let mut society = Society::from_charter(charter.clone());
        society.admit(grantor.clone());
        let grantor_caps = charter.capabilities_for(&grantor.roles);

        let outcome = society.grant_envelope(GrantEnvelope {
            grantor:      grantor.citizen,
            grantee:      Principal::Agent(arb_fixed_agent()),
            capabilities: requested.clone(),
            ttl_days:     ttl,
        });

        match outcome {
            Ok(env) => {
                prop_assert!(env.capabilities.is_subset_of(&grantor_caps),
                    "granted {:?} exceeds grantor {:?}", env.capabilities, grantor_caps);
                prop_assert!(env.expires_at > env.granted_at);           // invariant 6
                prop_assert!(env.expires_at <= env.granted_at + Days(90));
                prop_assert!(matches!(env.granted_by_kind(), PrincipalKind::Citizen));
            }
            Err(GovernanceError::CapabilityNotHeld { .. }) => {
                prop_assert!(!requested.is_subset_of(&grantor_caps));    // rejection was justified
            }
            Err(e) => prop_assert!(false, "unexpected error variant: {e:?}"),
        }
    }
}
```

Note what the test asserts on the error path. A property test that only checks the success case passes trivially against an implementation that rejects everything. Every property test in this corpus asserts that rejections were *necessary*, not merely permitted.

Generators (`arb_*`) live in a shared `fractal-testkit` crate and are held to production standards, because a generator that cannot produce a Society with a Council charter, a restricted membership, and a mid-fracture status is a generator that certifies the absence of bugs it cannot reach. Generator coverage is itself reviewed at each phase gate.

### 7.5 Deterministic simulation testing

**This is the single highest-leverage testing decision in the project.** Everything else in §7 is table stakes; this is the part that determines whether the system is actually correct under concurrency, and it exists only because `10 §7` made `Clock`, `Rng`, and `IdGen` ports.

The design is taken from FoundationDB and TigerBeetle: run the entire system, single-threaded, on simulated time, with every source of nondeterminism seeded, then torture it.

```
        seed: u64 ──────────────────────────────────────────────┐
             │                                                  │
             ▼                                                  ▼
   ┌───────────────────┐                            ┌──────────────────────┐
   │  HISTORY GENERATOR│  legal command streams     │  DETERMINISTIC PORTS │
   │  across N=1..8    │  weighted by risk class    │  Clock  (logical)    │
   │  Societies        │  + adversarial biases      │  Rng    (seeded)     │
   └─────────┬─────────┘                            │  IdGen  (counter)    │
             │                                      │  EventStore (mem)    │
             ▼                                      │  BlobStore  (mem)    │
   ┌───────────────────┐                            │  Relay      (mem)    │
   │   SCHEDULER       │  deterministic interleave  │  Ledger     (mem)    │
   │   single thread,  │◄───────────────────────────┤  + fault injection   │
   │   seeded order    │                            └──────────────────────┘
   └─────────┬─────────┘
             │  after EVERY step
             ▼
   ┌────────────────────────────────────────────────────────────────────┐
   │  INVARIANT ORACLE — all 15 of `11 §7`, plus per-saga postconditions │
   │  fail ──► shrink ──► minimal history ──► file with seed + commit    │
   └────────────────────────────────────────────────────────────────────┘
```

**Fault injection is not optional.** Without it the harness only proves the happy path is consistent. The injected faults, all seeded:

| Fault | Models | Frequency band |
|---|---|---|
| Port call fails transiently | Postgres timeout, S3 5xx, NATS redelivery | 0.1%–5% of calls |
| Port call fails after commit | The write succeeded, the ack was lost | 0.05%–1% |
| Message duplicated | At-least-once bus semantics | 0.5%–3% |
| Message reordered within a subject | Relay fan-out under load | 1%–10% |
| Process restart mid-saga | Deploy, crash, OOM | every 1k–20k steps |
| Clock jump forward | NTP correction, VM pause | every 5k steps |
| Partition between Node and Runtime | P2 offline behaviour | 1%–20% of wall time |

The last one earns its place specifically because P2 promises offline correctness. Local-first sync reconciliation is exactly the kind of logic that is correct in the two cases a developer imagined and wrong in the fifth.

Operationally:

- **Per PR:** 2,000 fresh seeds plus the entire regression corpus. Budget: 6 minutes on 8 cores.
- **Nightly:** 500,000 seeds, longer histories (up to 100k steps), wider fault bands. Budget: 4 hours.
- **Pre-phase-gate:** **500,000** seeds against the phase's feature set, in the same 4-hour budget as the nightly run. Any failure blocks the gate. This agrees with `50 PH1` AC-5, and it is deliberately not larger: at the published rates 5,000,000 seeds is 40 hours on a burst fleet or 250 core-hours serial, and **a gate nobody can afford to run is not a gate** — it is a step that gets waived, which is worse than a smaller gate because it teaches that gates are negotiable (`40 §8.3` has no bypass list for exactly this reason). Reconciled in `61 X14`.
- **Annual soak:** 5,000,000 seeds, funded explicitly on a burst fleet (~40 minutes across 64 machines at the nightly rate). **Not a gate.**

The per-PR figure and the nightly figure are not inconsistent rates: the ~6-minute per-PR run is 10,000 *fresh* seeds **plus the full regression corpus**, which is the larger share of its wall time. Stated here because `60 §4` row 12 correctly derived a 6.25× discrepancy from two numbers that were each describing a different workload.
- **Every fixed bug contributes its minimal seed to the permanent corpus**, forever. The corpus is the institutional memory of everything this system has ever gotten wrong.
- **`sim-mutation`, weekly:** invert each of the 15 invariant assertions in turn and confirm the harness detects it within 200 seeds. A harness that has silently stopped asserting is the most dangerous artifact in the repository — it converts "we tested it" into a false statement while every dashboard stays green.

The honest cost: roughly 6 engineer-weeks to build, and a permanent tax of a deterministic double per port and an oracle assertion per invariant. We pay it because the alternative is discovering an interleaving bug in the Ledger after Fraction has moved, and P6's event log makes that visible but does not make it reversible.

### 7.6 Integration tests

Real adapters against real dependencies via `testcontainers` — Postgres, MinIO (S3-compatible), NATS JetStream. **Not Redis:** presence is Relay-process memory gossiped over NATS, not a fourth stateful system (`61 X3`). They answer the question the simulation cannot: does our SQL actually do what we think, does JetStream redeliver the way the docs say, does the S3 client retry idempotently.

Rules: one container set per test binary, not per test. Every test creates its own Society and asserts only on that `society_id` (P1 makes this natural — test isolation is a free consequence of the partitioning strategy). No `sleep`; wait on a condition with a bounded timeout. Every adapter has a **port conformance suite** — one shared test battery, parameterized over every implementation of the trait including the in-memory double. That suite is what keeps the deterministic double honest; a double that diverges from the real adapter turns §7.5 into a well-tested fiction.

### 7.7 Contract tests

The OpenAPI 3.1 document and the protobuf/event schemas are **source**, not output. Generated: the Rust server stubs, the TypeScript SDK, the CLI argument surface, and the Zod validators.

Three gates run per PR:

1. **Backward compatibility.** `oasdiff` on the OpenAPI document and `buf breaking` on protobuf, against the last released tag. A breaking change without a `/v2` path fails.
2. **Event schema compatibility.** Every event kind is registered (`10 §10`). New fields must be optional; a breaking change requires a new `.v2` kind plus a registered upcaster, and a test proving that every historical event fixture upcasts cleanly. Old events are never rewritten — they are historical fact (P6).
3. **Provider verification.** The generated client is exercised against the running server; recorded response fixtures are replayed against the client's parser.

`00 §5` criterion 2 requires every capability to be reachable through the API, the CLI, and a GUI. A **parity test** derives the CLI verb list from the OpenAPI operation list and fails on any operation with no CLI verb. That is P13's falsification test as a build step rather than a release-day audit, which is the difference between a principle and a slogan.

### 7.8 End-to-end tests

E2E tests are expensive, slow, and flaky in proportion to their number. We cap them at **20 total** through Phase 5 and require a named justification per test. They exist only for flows where the integration of front end, gateway, Runtime, and adapters is itself the risk:

| # | Flow | Why it justifies an E2E |
|---|---|---|
| 1 | Register Citizen with a passkey, claim Handle | WebAuthn crosses browser, gateway, and KeyStore; unmockable in aggregate |
| 2 | Create Society, post in a Chamber, see it via Signal in a second browser | The realtime path end to end |
| 3 | Transfer Fraction, observe `PENDING` then settled | The one flow where a UI lie is a financial lie (`10 §6`) |
| 4 | Upload media, view rendition, verify E2EE ciphertext at rest | Encrypt-before-chunk boundary (`13 §3.1`) |
| 5 | Grant an Envelope, agent acts, agent is blocked outside it | P4's whole promise, visibly |
| 6 | Go offline, act, come back, reconcile | P2's falsification test, automated |
| 7 | Crystallize a Convergence, verify message IDs and tenure preserved | Invariant 12 through the real stack |
| 8 | Fracture dry run, execute, verify totals in both children | Invariant 11; the most dangerous operation in the system |

Playwright, against a full stack in Docker Compose. They run in the full lane and are allowed 12 minutes. Everything not on this list is tested at a lower level, and adding to the list requires a reviewer to explain why the risk is genuinely integrative.

### 7.9 Fuzzing

`cargo-fuzz` (libFuzzer) plus `arbitrary`, with corpora committed and continuously grown. Targets:

| Target | Why | Cadence |
|---|---|---|
| Event payload deserializer + every upcaster | Untrusted bytes from replicas and replay; a panic here bricks a Society's log | Nightly 30 min/target |
| Manifest and Shard parser (`13`) | Untrusted bytes from Custodians | Nightly 30 min |
| MLS message handling (`12`, N6) | Untrusted ciphertext from group members; the E2EE trust boundary | Nightly 60 min |
| FNID / Handle / VaultPath parsers | Confusable normalization, canonicalization bugs are identity bugs | Nightly 15 min |
| WASM host API surface (`20`) | The Extension sandbox boundary — the whole point of choosing WASM (P8 > P7) | Nightly 60 min + 4h weekly |
| Charter evaluator | Governance-as-data means a Charter is untrusted input | Nightly 30 min |
| CRDT merge functions (`10 §6`) | Merge must be commutative, associative, idempotent under adversarial input | Nightly 30 min |

A fuzz crash is a P1 bug regardless of reachability analysis. The WASM host API gets the longest budget because `02 §3` calls the Experience Runtime the most seductive scope trap in the document, and the mitigation is that nothing untrusted executes until this surface has been hammered for two phases.

### 7.10 Load, soak, and economic simulation

- **Load tests** (`k6` against a production-shaped environment, weekly and pre-release): 10,000 concurrent Signal subscribers, 1,000 messages/sec sustained across 500 Societies, 50 concurrent media ingests. Gate: p99 within the §12 budget, error rate below 0.1%, and no memory growth beyond 2% per hour.
- **Soak tests** (72 hours, nightly-started, weekly-evaluated): fixed moderate load, watching for leaks, unbounded queues, connection exhaustion, projection lag drift, and file-descriptor growth. The failure this catches is the one that appears in week three of production and never in CI.
- **Economic simulation** (`17 §12`, cross-referenced not duplicated): the adversarial agent-based harness that must show bounded circulating supply under farming at 100× normal actor volume. It is a **release gate for any change touching a Source, a Sink, a Contribution metric, or an emission parameter**. P12's falsification test says a mechanic that fails simulation ships disabled, and CI enforces exactly that: the feature flag defaults to `off` and cannot be flipped without a passing simulation artifact attached.

### 7.11 Mutation testing

`cargo-mutants` on R0 and R1 crates. Weekly full run; per-PR incremental run over changed files only, budgeted at 8 minutes.

Mutation testing answers the question coverage cannot: does the suite *detect* a wrong behaviour, or merely execute the line? For a ledger, that distinction is the whole game. A surviving mutant in an R0 crate is a defect ticket with a 5-working-day SLA. Floors are in §7.1; the R0 floor of 85% is deliberately below 100% because the last 15% is dominated by equivalent mutants, and chasing them produces tests written to kill mutants rather than to express intent.

### 7.12 Coverage policy

**Floors, not goals:** R0 95% line / 90% branch, R1 85% / 75%, R2 75%, R3 none. Measured with `cargo-llvm-cov`, enforced per-crate rather than as a workspace average — an average lets a well-tested parser subsidize an untested Charter evaluator.

Coverage is a floor because it measures execution, not verification. A test suite that calls every function and asserts nothing scores 100%. We therefore state plainly: **coverage may never be cited as evidence of quality in a review or a phase gate.** The evidence is the property tests, the simulation seed count, and the mutation score. Coverage exists to catch the specific case of code nobody tested at all, and that is all it is for. New code in a PR must not lower a crate's coverage by more than 0.5 percentage points; a larger drop requires the PR to say why.

### 7.13 Flaky test policy

A test that fails and then passes without a code change is quarantined **automatically** by the CI harness on the second occurrence within 14 days.

```
 detect (2 flips / 14d) ──► auto-quarantine ──► issue filed, owner assigned
                                   │             (owner = last author of the test
                                   │              or the crate owner)
                                   ▼
                         ┌── fixed within 10 working days ──► restored
                         │
                         └── not fixed within 10 working days ──► DELETED
                              and the coverage gap recorded as a defect ticket
```

Quarantined tests still run and still report; they simply do not block. The cap: **no more than 5 tests in quarantine at once**, workspace-wide. At 6, the pipeline goes red for everyone until the count drops. This is deliberately painful, because a flaky suite that is tolerated becomes a suite that is ignored, and an ignored suite is a suite that has stopped protecting anything. Deleting a flaky test is an acceptable outcome; leaving a permanently amber pipeline is not.

Simulation failures are **never** flaky by construction — they are seeded. If a simulation failure is not reproducible from its seed, the harness itself has a nondeterminism bug, which is a P1 defect that stops the line.

---

## 8. CI/CD

### 8.1 The two lanes

```
  push / PR
      │
      ▼
┌──────────────────────── FAST LANE ── target ≤ 5 min, hard cap 8 ────────────────────┐
│  ① fmt --check            15s   ② canon lints (§4)          20s                     │
│  ③ clippy -D warnings   2m30s   ④ unit + doctests           2m00s                   │
│  ⑤ cargo deny (licences, advisories, bans)                  20s                     │
│  ⑥ typecheck + eslint (web)                                 1m00s                   │
│  All six run in parallel. Any failure stops the lane and posts the exact command     │
│  to reproduce locally.                                                              │
└─────────────────────────────────┬───────────────────────────────────────────────────┘
                                  │ green
                                  ▼
┌──────────────────────── FULL LANE ── target ≤ 25 min, hard cap 40 ──────────────────┐
│  property tests (5m) │ simulation 2k seeds + corpus (6m) │ integration (8m)         │
│  contract + schema compat (2m) │ e2e (12m) │ a11y axe (3m) │ perf budgets (6m)      │
│  mutation, incremental (8m) │ wasm32 + aarch64 + windows build (9m) │ SBOM (1m)     │
│  Runs on the merge queue and on `main`. Sharded across 8 runners.                    │
└─────────────────────────────────┬───────────────────────────────────────────────────┘
                                  │ green
                                  ▼
                          MERGE QUEUE ──► main ──► deploy pipeline (§8.4)
```

**The wall-clock rule.** The fast lane has a **5-minute target and an 8-minute hard cap**; the full lane, 25 and 40. When a lane exceeds its cap on the 7-day p50, fixing it becomes the highest-priority engineering task, above feature work, until it is back under. This is not a nicety. A slow pipeline is paid for in three currencies simultaneously: agents batch changes into larger, less reviewable PRs; humans context-switch and lose the thread; and eventually somebody adds a bypass. Every one of those degrades correctness. CI duration is tracked as a first-class metric with a dashboard and an alert, exactly like production latency.

### 8.2 Caching

| Mechanism | What it saves | Notes |
|---|---|---|
| `cargo-chef` | Dependency compilation in Docker layers | Dependencies rebuild only when `Cargo.lock` changes |
| `sccache` (S3 backend, shared) | Cross-runner object reuse | Target 85%+ hit rate; the rate is a dashboard metric because a silent cache miss looks identical to a slow pipeline |
| `cargo-nextest` | Test runner | Faster, per-test process isolation, real retry semantics, machine-readable output the flake detector consumes |
| Container image cache | testcontainers pulls | Images pinned by digest; a moving tag makes a test suite nondeterministic |
| Turborepo remote cache | Web build and typecheck | Skips unchanged packages entirely |

Caches are keyed by toolchain version + `Cargo.lock` hash + target triple. Every cache is invalidatable by a single manual button, because a poisoned cache produces the worst debugging experience in existence: a failure that reproduces nowhere.

### 8.3 Merge queue and required checks

Required for merge to `main`, no exceptions and no administrator bypass:

1. Fast lane green.
2. Full lane green **at the merge-queue commit** — that is, against `main` as it will actually be, not against the branch point. Semantic conflicts between two independently green PRs are otherwise inevitable and are found in production.
3. One human approval (§13).
4. ADR present if §6.1 applies.
5. Conventional commit format (`42`).
6. No decrease in a risk-class coverage floor; no new quarantined test.
7. Changelog line if user-visible.

The queue batches up to 5 PRs, tests the batch, and bisects on failure. Branch protection has no bypass list. If an emergency requires shipping without CI, the correct action is a documented incident with a follow-up PR, not a permission that exists year-round waiting to be misused.

### 8.4 Deployment

**Environments:** `dev` (every `main` commit, auto), `staging` (every `main` commit, auto, production-shaped data volumes and synthetic load), `production` (progressive, §8.5). A fourth, `sim`, runs the nightly simulation and load work and is not a deploy target.

**Cadence — continuous deploy to production, not release trains.** Justification: batch size is the dominant variable in deployment risk. A train of 40 changes that fails is a bisect under pressure; a single change that fails is a rollback. Continuous deploy also keeps the rollback path exercised weekly rather than theoretical. The honest cost is that it demands the automation in §8.5 to actually work, and that it puts real weight on the SLO burn-rate alerting in §9.4 — with continuous deploy, those alerts *are* the safety net.

The exception: **client releases are trained.** Desktop, mobile, and PWA ship on a two-week train because store review, code signing, notarization, and the fact that users control upgrade timing all make per-commit client deploys meaningless. The Runtime is continuous; the shells are trained; the API contract between them is versioned and back-compatible for two client trains (four weeks) minimum.

### 8.5 Progressive delivery

```
 main ──► build ──► sign ──► staging (30 min soak, synthetic load)
                                  │
                                  ▼
                          canary 1% ── 15 min ──┐
                                  │             │  automatic rollback if ANY of:
                          canary 5% ── 15 min ──┤   • error-rate SLO burn > 2%/hour
                                  │             │   • p99 latency > budget × 1.5
                          canary 25% ─ 30 min ──┤   • any 11 §7 invariant assertion
                                  │             │     fires in production
                          100% ────────────────-┘   • ledger imbalance detected
                                  │
                                  ▼
                    watch 24h, then the version is "baked"
```

Rollback is automatic, requires no human, and completes in under 90 seconds. Invariant assertions run **in production**, not only in tests: a background auditor recomputes `Σ debits == Σ credits` and `total supply == -EmissionAccount.balance` every 60 seconds per Society partition and pages immediately on divergence. P12 says the economy must be honest; the only way to know that it *is* honest is to check continuously against live data.

### 8.6 Database migration discipline

**Expand / contract, always, with a release boundary between the phases.**

```
 Release N     EXPAND    add nullable column / new table / new index CONCURRENTLY
                         (old code still works; new column unused)
 Release N+1   MIGRATE   backfill in bounded batches; dual-write; read old, verify new
 Release N+2   SWITCH    read new; old column still present and still written
 Release N+3   CONTRACT  stop writing old; drop it
```

Hard rules, enforced by a migration linter in the fast lane:

- **No destructive migration in the same release as the code that requires it.** This is the rule that makes every deploy rollback-able. Violating it means a rollback restores code that queries a dropped column, which converts a bad deploy into an outage.
- No `DROP COLUMN`, `DROP TABLE`, or type-narrowing `ALTER` without an ADR and a preceding release that stopped using it.
- Every index creation is `CONCURRENTLY`. Every migration states its expected duration and locks; a migration estimated over 30 seconds on production row counts must be batched.
- Every migration has a tested down-path, or an explicit `-- IRREVERSIBLE:` comment with the recovery procedure written out.
- Migrations run as a separate deploy step, never at Runtime boot. A Runtime replica starting during a migration must not race it.

**The event log is never migrated.** Events are immutable historical fact (P6, `10 §5`). Schema evolution happens through upcasters in the replay path, tested against a fixture corpus of every historical event version. Projections, by contrast, are disposable by definition — the cheapest "migration" for a projection is to drop it and replay, and for anything under roughly 50 million events that is the preferred option because it is the same code path that P6's falsification test exercises.

### 8.7 Feature flags

Named `ff.<area>.<name>` per `01 §9`. Four kinds, with different lifetimes:

| Kind | Purpose | Max lifetime | Removal |
|---|---|---|---|
| Release | Decouple deploy from launch | 60 days | CI fails the build at expiry |
| Experiment | A/B a mechanic | 90 days | Must reference a decision doc |
| Ops kill-switch | Disable a subsystem under load | permanent | Registered, tested quarterly |
| Permission gate | Level / Standing / phase gating | permanent | Part of the domain, not a flag debt |

Every flag has an owner, an expiry, a default, and a registry entry. A release or experiment flag past expiry **fails the build** — stale flags are a combinatorial explosion of untested code paths and the second-largest source of "it works on staging" in most systems. Flag state is never read inside a domain crate; flags are evaluated in the application layer and passed in as parameters, so the domain stays a pure function and §7.5 stays valid.

---

## 9. Observability

### 9.1 Tracing

OpenTelemetry throughout, via `tracing` + `tracing-opentelemetry`. The `Telemetry` port (`10 §7`) means the sink is swappable and no vendor SDK appears in a domain crate.

**`correlation_id` is generated at the front end** — the browser, the CLI, or the Agent runtime — and travels through the gateway, the application layer, the domain, the event envelope (`10 §5`), the projection consumers, and the Signal fan-out. One identifier answers "what happened when the Citizen pressed the button", across every process. It is displayed in the UI on any error and in every CLI error message, so a support conversation begins with an exact trace instead of a reconstruction.

**Every domain event is a span**, with attributes: `society_id`, `event.kind`, `event.seq`, `actor.kind`, `envelope_ref` when present, `causation_id`, and the aggregate identity. Never the payload — payloads may contain message content, which is either private (P9) or ciphertext the Runtime cannot read (N6).

Sampling: 100% of errors, 100% of R0-class operations (ledger, envelope evaluation, fracture, governance enactment), 100% of Agent actions, and 1% head-based sampling of everything else with tail-based upsampling of anything slower than its p99 budget. Agent actions are sampled at 100% deliberately: P4's audit requirement is not satisfiable by a sampled trace.

### 9.2 Structured logging

JSON to stdout, one object per line, collected by the platform. Fields: `ts` (RFC3339 UTC, microseconds), `level`, `target`, `msg`, `correlation_id`, `society_id`, `principal` (FNID, never a name), plus typed fields. No string interpolation of variables into `msg` — variables are fields, or they cannot be queried.

| Level | Meaning | Budget |
|---|---|---|
| `error` | A human must eventually look. Every one is either alerted on or is noise to be deleted. | < 10/hour steady state |
| `warn` | Degraded but handled; a retry succeeded, a replica lagged | < 200/hour |
| `info` | Lifecycle: boot, config, deploy, migration, settlement run start/end | < 50/minute |
| `debug` | Per-request detail; off in production, enableable per-Society for 60 minutes | — |
| `trace` | Development only; never enabled in production | — |

**Never logged, at any level:** message bodies, media content, private keys, session tokens, recovery phrases, passkey material, MLS group secrets, wallet mnemonics, IP addresses beyond a 24-hour rate-limiting window, precise geolocation, and any `#[secret]` type (§4.4 makes the last one a compile error rather than a convention). Log volume itself is a privacy surface under P9: a log that records every Chamber a Citizen opened is behavioural surveillance regardless of intent.

### 9.3 Metrics

RED per service boundary from `10 §3` — Rate, Errors, Duration — plus USE for resources. Naming per `01 §9`: `fractal_<area>_<name>_<unit>`.

```
fractal_discourse_message_posted_total{society_class,chamber_kind}
fractal_discourse_command_duration_ms{boundary,outcome}
fractal_ledger_posting_duration_ms{reason}
fractal_ledger_imbalance_detected_total              ← must be 0 forever; pages instantly
fractal_agent_action_blocked_total{reason,agent_kind}
fractal_relay_signal_latency_ms{transport}
fractal_projection_lag_events{projection,society_class}
fractal_eventstore_append_duration_ms
fractal_vault_attestation_failed_total{custodian_class}
fractal_sync_reconcile_conflicts_total{data_class}
```

Cardinality discipline: **`society_id` is never a metric label.** With 100,000 Societies that is a cardinality bomb that will take down the metrics backend before it takes down the Runtime. Per-Society detail lives in traces and in the per-Society event log, both of which are designed for high cardinality. Metrics carry `society_class` (a bucketed size/tier label) instead. Label sets are declared in code and asserted by a test with a hard cardinality budget of 10,000 series per metric.

### 9.4 SLOs, error budgets, and alerting

| Surface | SLI | SLO (28-day) | Error budget |
|---|---|---|---|
| Public API — reads | availability, non-5xx | 99.9% | 40m 19s |
| Public API — writes | availability | 99.9% | 40m 19s |
| Public API — reads | p99 latency < 200ms | 99% of minutes | — |
| Public API — writes | p99 latency < 500ms | 99% of minutes | — |
| Ledger postings | correctness (imbalance events) | **100%** | **zero** |
| Ledger postings | p99 settle < 1s | 99.9% | — |
| Signal delivery (Relay) | p95 end-to-end < 250ms | 99.5% | 3h 22m |
| Media ingest → playable | p95 < 60s for ≤ 500MB | 99% | — |
| Event append | p99 < 50ms | 99.9% | — |
| Projection freshness | lag < 2s | 99.5% | — |
| Offline reconcile (P2) | success on reconnect | 99.99% | 4m 2s |
| Agent action decision | p99 < 100ms | 99.9% | — |

The ledger row has **no error budget**. Availability degrades gracefully; correctness does not. A single `fractal_ledger_imbalance_detected_total` increment is a page, a deploy freeze, and an incident, and it is the one SLI where the correct response to burn is to stop the platform rather than to keep serving.

**Alerting philosophy: alert on symptoms and on burn rate, never on causes.** High CPU is not an alert; it is a dashboard. "The write API is burning its 28-day error budget at 14× the sustainable rate" is an alert, because a Citizen is being harmed right now. Cause-based alerting produces a pager that fires for conditions that are frequently fine, and a pager that cries wolf is a pager that gets muted.

Multi-window, multi-burn-rate, standard configuration:

| Burn rate | Window | Budget consumed | Action |
|---|---|---|---|
| 14.4× | 1h and 5m | 2% | Page immediately |
| 6× | 6h and 30m | 5% | Page |
| 3× | 24h and 2h | 10% | Ticket, next business day |
| 1× | 72h and 6h | 10% | Ticket |

The requirement for two windows to agree is what suppresses the 3am page for a 90-second blip. Exhausting an error budget freezes feature deploys for that surface until reliability work restores it — a policy that only works if it is applied without negotiation the first time it triggers.

### 9.5 Dashboards that must exist

Six, and they are code (Grafana JSON in the repository, reviewed like any other change):

1. **Platform health** — RED for every `10 §3` boundary, error budget remaining per SLO, deploy markers.
2. **Economy** — circulating supply, emission rate against the `17 §5` schedule, Source/Sink flows, imbalance counter, Sybil-signal indicators. Watched daily; P12 is an operational property, not only a design one.
3. **Society partition health** — projection lag distribution, hot partitions, event append rate, per-partition storage growth.
4. **Realtime** — Signal latency percentiles, subscriber counts, fan-out depth, reconnect storms.
5. **Agent activity** — actions per Envelope, blocked-action rate by reason, spend against caps, Agent Trust distribution. P4's audit surface, made continuous.
6. **Client experience** — real-user Core Web Vitals against the P10 budgets, per platform, at p75.

A dashboard nobody opens for 90 days is deleted. Dashboard sprawl produces the illusion of observability and the reality of nobody knowing which panel matters.

### 9.6 On-call and runbooks

Every alert links to a runbook, and **an alert without a runbook cannot be created** — the alert definition file requires the runbook path and CI validates it resolves. Each runbook contains: what the alert means in one sentence, user-visible impact, the first three diagnostic commands verbatim, the mitigation, the rollback, the escalation contact, and a link to previous incidents that fired it.

On-call is a documented rotation with a written handoff. Every page produces an incident record; every SEV1 and SEV2 produces a blameless postmortem within 5 working days containing a timeline, a contributing-factors analysis (not a single "root cause", which is almost always a fiction), and action items with owners and dates. Action items from postmortems are tracked to completion and reviewed at the phase gate; a postmortem whose actions are never done is theatre.

---

## 10. Security Engineering

### 10.1 Threat modelling cadence

STRIDE against the data-flow diagram in `10 §1`, refreshed at every phase gate and additionally on any change to: an authn/authz flow, the E2EE boundary, the Envelope model, the WASM sandbox, or a payment path. Output is a table of threat, affected principle, mitigation, residual risk, and owner, committed to `docs/security/threat-model-phase-N.md`.

The four threat surfaces that get standing attention because they are where this specific architecture is most exposed:

1. **Privilege escalation via Envelope delegation** (P4). Mitigated structurally by invariant 5 and adversarially tested each phase.
2. **Sybil attack on Contribution Scores** (P12). The economic simulation is the test; `17 §8` is the design.
3. **Malicious Extension escaping the WASM sandbox** (P7 vs P8). Mitigated by capability-based host APIs, resource metering, fuzzing (§7.9), and the `02 §3` decision that nothing untrusted executes until Phase 7.
4. **Custodian inferring content from Shards** (P9). Mitigated by encrypt-before-chunk (`13 §3.1`); the residual risk is size and timing metadata, which is documented rather than pretended away.

### 10.2 Dependency policy

- **Everything pinned.** `Cargo.lock` and `pnpm-lock.yaml` committed. Container images by digest, never by tag. Actions by commit SHA, never by version tag — a mutable tag in a CI action is a remote-code-execution primitive.
- **`cargo-deny`** in the fast lane: licence allowlist (MIT / Apache-2.0 / BSD / ISC / Unicode-DFS; anything else needs an ADR and legal review), duplicate-version bans, source allowlist (crates.io and named git repositories only), and advisory denial.
- **`cargo-audit` / `pnpm audit`** in the fast lane and nightly against the current advisory database, so a newly-published advisory against an unchanged dependency turns the pipeline red without anybody pushing.
- **Vulnerability SLA:** critical 24 hours, high 7 days, medium 30 days, low next scheduled bump. The clock starts at advisory publication, not at discovery.
- **Complexity budget:** `02 §5` allows 5 new third-party runtime dependencies per phase. The budget is checked at the gate by diffing the direct-dependency set. Transitive growth is reported but not budgeted, because it is not directly controllable — however, a dependency that brings 40 transitive crates is evaluated on that basis in its ADR.
- **`cargo-udeps` and `cargo-machete`** nightly: an unused dependency is pure supply-chain surface with zero benefit.
- **Single-maintainer rule (`00 §3.5`).** Any dependency that is (a) on the critical path — ledger, crypto, event store, sandbox, sync — and (b) maintained by one person or has had no release in 12 months, must be vendored into `third_party/` with a recorded upstream commit, a diff-review of the vendored code, and a named internal owner. We would rather maintain 3,000 lines we understand than depend on 3,000 lines that can be abandoned or, worse, transferred to a hostile maintainer. This has happened repeatedly in every package ecosystem and it will happen to us.

### 10.3 Supply chain

- **Reproducible builds.** Pinned toolchain, `--locked`, `SOURCE_DATE_EPOCH`, trimmed paths, no build-time network access. CI builds every release twice on different runners and compares digests; a mismatch blocks the release. Reproducibility is what allows a third party to verify that a published binary corresponds to published source, which is a load-bearing claim for a sovereignty product (P2).
- **Signed releases.** Sigstore/cosign, keyless with OIDC identity, plus a maintained offline root for the desktop and CLI artifacts. Platform-specific signing (Apple notarization, Windows Authenticode) on top. `fn` verifies its own update signatures before applying them.
- **Provenance.** SLSA Build Level 3: hermetic builds on ephemeral runners, in-toto attestation of source commit, build parameters, and output digests, published alongside every artifact.
- **SBOM.** CycloneDX generated per release for both the Rust workspace and the web bundle, published with the artifact, and diffed release over release so that an unexplained new component is visible.

### 10.4 Secrets

Secrets live in the platform secret manager, injected as environment variables or files at process start, never in the repository, never in an image, never in an event, never in a log (§9.2), and never in an error (§2.5). `gitleaks` runs in the fast lane and as a pre-commit hook. Rotation: 90 days routine, immediate on any suspicion. Every secret has a named owner and a documented rotation procedure that has been executed at least once — an untested rotation procedure is a wish.

Citizen key material is a different category and is governed by `12-identity-and-trust.md`: device-bound, never transmitted, never recoverable by us. The Runtime cannot decrypt private content and no code path exists that could (N6). That is verified by the security review, not merely asserted: a reviewer must be able to demonstrate the absence of the path, which is why the E2EE boundary is drawn at a crate boundary rather than inside a function.

### 10.5 The security review gate

No phase gate passes without: an updated threat model; a clean dependency posture (no unresolved critical or high advisories); a penetration test of any new externally-facing surface — external and independent from Phase 4, when the platform first holds meaningful value; a review of every new capability in the Envelope vocabulary against P8's deny-by-default falsification test; and a demonstrated key-rotation and revocation drill.

**Responsible disclosure:** `security.txt` and `SECURITY.md` published from day one, a dedicated encrypted contact, acknowledgement within 48 hours, triage within 5 working days, and a public advisory with credit on fix. A bug bounty opens at Phase 5, when there is enough value at stake to attract researchers and enough maturity to handle the volume.

**Incident response:** declare, contain, eradicate, recover, learn. Severities: SEV1 (data breach, key compromise, ledger corruption, platform down) pages immediately and notifies affected Citizens within 72 hours; SEV2 (partial outage, privilege bug with no evidence of exploitation) pages during business hours; SEV3 is a ticket. A security incident always produces a postmortem, always names the detection gap that let it run as long as it did, and always ships the detection improvement before the incident is closed.

---

## 11. Backups and Disaster Recovery

### 11.1 Why P1 is a gift here

Most platforms back up a database. We back up **Societies**, because P1 guarantees that every persistent object resolves to exactly one `society_id` or appears on the nine-entry Global Registry (`01 §6`). That single architectural decision, made for reasons of mental model and sharding, hands us four capabilities that are normally expensive or impossible:

1. **Per-tenant restore.** One Society can be restored to a point in time without touching any other. In a shared-schema system this is a multi-day forensic exercise; here it is a scoped operation on one partition's log.
2. **A backup is a portable artifact.** A Society's bundle — event log, manifests, treasury state, charter — is the same artifact used for export, self-hosting, and Node migration. The backup path and the "take your Society and leave" path are the *same code*, which means the export feature keeps the backup format honest and the backup drills keep the export feature tested. Neither can rot silently.
3. **Verifiable restores.** The log is hash-chained (`10 §5`) and anchored (`01 §6`, `13`). A restored Society is not merely present; its integrity is provable against its anchors.
4. **Bounded blast radius.** A corrupt backup affects one Society, not the platform.

None of this was designed for backups. It falls out of P1. It is worth stating explicitly because it is the clearest example in the document of a principle paying a dividend somewhere its author was not looking.

### 11.2 What is backed up, and to what target

| Data class | Mechanism | RPO | RTO | Retention |
|---|---|---|---|---|
| Event log (per Society) | Continuous WAL archive + hourly snapshot, hash-chain verified | **0 (sync commit)** | 1h per Society, 4h platform | 7y (it is the source of truth) |
| Projections | Not backed up — rebuilt by replay (P6) | n/a | 2h full rebuild | n/a |
| Ledger state | Derived from events; snapshot hourly for fast restore | 0 | 1h | 7y |
| Vault objects (Shards) | Erasure-coded across Custodians (`13 §6`) + cross-region cold copy | 15m | 6h | Per retention policy |
| Manifests and keys-to-ACL wrapping | With the event log (they are events) | 0 | 1h | 7y |
| Identity / Handle registry (global) | Continuous replication + hourly snapshot | 0 | 30m | 7y |
| Search indexes | Not backed up — rebuilt from events | n/a | 3h | n/a |
| Configuration and secrets | Versioned in the secret manager; config in git | 24h | 15m | 1y |
| Object store cold copy | Cross-region, cross-provider, cross-account | 24h | 12h | 90d |

Backups go to a **different provider account with independent credentials**, append-only with object-lock, and a retention hold that the production Runtime's credentials cannot lift. This is specifically a ransomware and compromised-credential control: a backup that the compromised system can delete is not a backup.

### 11.3 Encryption and key custody

Backups are encrypted at rest with keys held in a KMS that is separate from the production KMS, under a separate account. E2EE content is already ciphertext and stays that way — restoring a backup does not give us a plaintext path to private messages, which would violate N6 and would make the backup itself a target more valuable than the live system.

Key custody: 3-of-5 Shamir split of the backup root key, held by named individuals with an annual reconstitution drill. The drill is the point. An unexercised key-recovery procedure fails at exactly the moment it is needed, and the failure mode is unrecoverable.

### 11.4 Restore drills

**A backup that has never been restored is not a backup. It is a hopeful blob of bytes with a monthly bill.**

| Drill | Cadence | Pass criteria |
|---|---|---|
| Single-Society PITR to an arbitrary timestamp | **Weekly**, automated, randomly selected Society | Restored log verifies against its hash chain and anchors; projections rebuild; the invariant oracle (§7.5) passes on the restored state |
| Full-platform restore into a clean account | **Quarterly**, human-run, timed | Within the 4h RTO; a written report of every manual step, which becomes automation before the next drill |
| Key reconstitution | **Annually** | 3-of-5 assembled, root key recovered, nothing else needed |
| Region loss failover | **Semi-annually** | Traffic served from the secondary within 30 minutes |
| Ransomware scenario (production credentials assumed hostile) | **Annually** | Restore proceeds using only offline credentials |

Every drill produces a report; every manual step in a drill becomes a ticket to automate. The weekly automated drill is the one that actually keeps the system honest, because it fails on the Tuesday after somebody changes the schema rather than on the day of the outage.

### 11.5 Point-in-time recovery and log replay as the ultimate mechanism

PITR granularity is 1 second, over a 35-day window, per Society. Beyond 35 days, recovery is from the nearest snapshot plus the archived log — slower, but the data is not gone, because the log is retained for 7 years.

The deepest recovery path is P6's falsification test used in anger: **delete every projection and rebuild from the event log**. This is not an emergency-only procedure; it runs in CI on every commit (a small corpus) and nightly on a production-sized corpus. That means the platform's disaster-recovery mechanism of last resort is exercised more often than most systems exercise their backups at all. When a projection bug corrupts a read model in production, the response is not a repair script whose correctness nobody can verify; it is a replay, from facts, with a known-good outcome.

The residual risk, stated honestly: replay cannot recover from a bug that wrote *wrong events*. Events are immutable historical fact and we do not rewrite them. The remedy for a wrong event is a compensating event — the accounting-world answer — which preserves the audit trail and is why `PostingReason` includes explicit correction variants rather than allowing a delete.

---

## 12. Accessibility Engineering (N8)

Accessibility is an acceptance criterion (P10), not a phase. The floor is **WCAG 2.2 AA on every surface**, with **AAA text contrast (7:1) in the default theme** because the default theme is what most people will use forever and contrast is the cheapest AAA criterion to hold.

| Requirement | Standard | Gate |
|---|---|---|
| Automated rule coverage | axe-core, zero violations of `serious` or `critical` | CI, blocks merge |
| Text contrast, default theme | 7:1 (AAA); 4.5:1 minimum in all other themes | Token-pipeline test over every token pair |
| Non-text contrast | 3:1 for controls, focus indicators, meaningful graphics | Token test + manual |
| Keyboard operability | 100% of interactive elements; visible focus; logical order; no traps | Automated tab-walk test per surface + manual |
| Screen reader | Every interactive element named, every state announced, live regions for Signals | Manual matrix, per phase |
| Reduced motion | `prefers-reduced-motion` honoured by every animation; no vestibular triggers | Lint on animation definitions + manual |
| Target size | 24×24 CSS px minimum (2.2 AA), 44×44 on touch | Automated measurement in the component test suite |
| Text resize | 200% zoom with no loss of content or function | Automated viewport test |
| Forms | Programmatic labels, error identification, suggestion, prevention on financial actions | Component test |
| CLI accessibility | Screen-reader-friendly output mode, no colour-only meaning, `NO_COLOR` honoured | CLI snapshot tests |

The CLI row is not decoration. N3 makes the CLI a first-class citizen and N7 puts the design system on every surface including ANSI; a terminal UI that conveys state only through colour is inaccessible in exactly the same way a web UI is.

**Automated gate:** axe-core runs against every component in the Storybook-equivalent catalogue and against all eight E2E flows. Automated tooling catches roughly 30–40% of real accessibility defects, so the remainder is covered by a **manual audit at every phase gate**, performed against the screen-reader matrix — NVDA + Firefox and NVDA + Chrome on Windows, VoiceOver + Safari on macOS and iOS, TalkBack + Chrome on Android, and Orca + Firefox on Linux — plus keyboard-only and 200%-zoom passes. Findings are defects with owners, not backlog suggestions.

**An accessibility failure blocks a merge exactly as a failing test does.** There is no "a11y follow-up ticket" path, because that ticket has never once been completed in the history of software. The only exception is a documented, time-boxed exception approved by a human reviewer, recorded in `docs/a11y/exceptions.md` with an expiry date that fails the build when it passes.

---

## 13. Performance Engineering (P10)

### 13.1 The budget table

| Surface / operation | Budget | Measured how | Gate |
|---|---|---|---|
| Desktop cold start → interactive | **1.5s** | Instrumented boot on reference hardware | CI perf lane, p75 of 20 runs |
| Web cold load → interactive (p75, mid-tier, 4G) | **2.5s** | Lighthouse CI + real-user CWV | CI + RUM |
| Interaction to next paint (INP) | **100ms** p75 | RUM + Playwright trace | CI + RUM |
| Animation frame budget | 16.6ms (60fps); 8.3ms where 120Hz | Frame timing in the E2E harness | CI |
| First-party JS bundle, initial route | **180KB** gzipped | Bundle analyzer | CI, hard fail |
| Total initial payload | 400KB gzipped | Bundle analyzer | CI, hard fail |
| API read, p99 | 200ms server-side | OTel histogram | SLO + CI benchmark |
| API write, p99 | 500ms server-side | OTel histogram | SLO + CI benchmark |
| Event append, p99 | 50ms | Criterion + production metric | CI benchmark |
| Ledger posting, p99 | 20ms domain-side | Criterion | CI benchmark, R0 |
| Envelope evaluation, p99 | 5ms | Criterion | CI benchmark, R0 |
| Signal end-to-end, p95 | 250ms | Synthetic probe | SLO |
| Projection lag, p99 | 2s | Metric | SLO |
| Offline read from local store, p99 | 30ms | Instrumented on-device | CI on-device suite |
| Memory, Runtime steady state per 1k active Societies | 2GB RSS | Soak test | Soak gate |
| Desktop binary size | 25MB | Build artifact check | CI |

Client budgets come from `32-design-system.md`, which is the **source**: `perf/budgets.json` is generated from `32 §8`, and the client rows above are rendered from that file by `cargo xtask budgets --render`. `cargo xtask lint-docs` fails on a hand-authored budget number. The design system owns the client numbers, this document owns the **server-side** rows — API read/write p99, event append, ledger posting, Envelope evaluation, Signal end-to-end, projection lag, Runtime memory — and owns enforcement for all of them. Ownership boundaries and the eight reconciled figures are in `61 X9`.

### 13.2 The gate and regression detection

Benchmarks are `criterion` for Rust hot paths and Lighthouse CI plus a Playwright trace harness for clients, run on **dedicated, non-shared runners** — a performance gate on a noisy shared runner produces false failures, which produces bypasses, which produces no gate at all.

Regression detection compares against a rolling 20-run baseline from `main` with a threshold of **+5% on p99 or +3% on p50**, using a Mann–Whitney U test to avoid firing on noise. A regression fails the build. The PR author either fixes it or attaches an approved budget exception with a named expiry.

Continuous profiling (`pprof`-style, 1% sampling) runs in production and feeds a flamegraph diff per deploy, so a regression that only appears under real traffic patterns is attributable to a specific release rather than discovered three weeks later as "things feel slower".

### 13.3 The profiling workflow

`02 §7` states it plainly: **no performance optimization without a profile.** Operationally, a PR whose description claims a performance motivation must attach one of a flamegraph, a `criterion` comparison, or a production trace, both before and after. A PR with no attached measurement and a performance rationale is closed, not reviewed — this is one of the few automatic rejections in the document, and it exists because speculative optimization is the single most common way that clean code becomes unmaintainable code for no measured benefit.

The workflow: reproduce under load → profile (`cargo flamegraph`, `samply`, or the production continuous profiler) → identify the dominant cost, which is almost always allocation, a chatty port call, or an N+1 query rather than the arithmetic anyone suspected → fix the algorithm or the boundary → re-measure → attach both profiles → land. Micro-optimization inside the domain layer is the last resort and, so far, has never been the answer.

---

## 14. Code Review

### 14.1 What review is for

Review does not check formatting, lint compliance, test presence, coverage, accessibility rule violations, dependency licences, or performance budgets. Machines check all of those, and a human who spends attention on them has none left for the things machines cannot check. A reviewer whose comments are mostly style comments is a signal that a lint is missing.

### 14.2 The reviewer checklist

Tied point-by-point to `00 §5`:

| # | The reviewer confirms | Maps to |
|---|---|---|
| 1 | The change does what the acceptance criteria say — verbatim, not approximately | Done 1 |
| 2 | The API, CLI, and (if user-facing) GUI surfaces all exist; parity test passes | Done 2, P13 |
| 3 | Tests match the risk class, and the property tests assert the *right* invariants — not that tests exist, which CI already knows | Done 3 |
| 4 | Domain events are emitted with correct `actor`, `envelope_ref`, `correlation_id`, and `causation_id` | Done 4, P4/P6 |
| 5 | Docs, CLI help, and changelog line are present and correct | Done 5 |
| 6 | Offline behaviour is defined (P2) and the denial path is correct and deny-by-default (P8) | Done 6 |
| 7 | Budgets respected; a11y verified for new UI | Done 7, P10/N8 |
| 8 | ADR present if §6.1 applies | Done 8 |
| 9 | Terminology is canonical (`01`), naming follows `01 §9` | Canon |
| 10 | Scope matches the ticket. No opportunistic extras | `02 §7` |
| 11 | The abstraction has two callers, or is on the P5 decreed list | `02 §7` |
| 12 | Failure modes are handled: what happens on partial failure, retry, concurrent execution, revoked Envelope mid-flight | Judgement |

### 14.3 Review when most code is written by agents

This is the section that matters most, because the ratio of generated code to reviewer attention is the binding constraint on this project's quality.

**What is fully delegated to automated gates:** style, lints, terminology, dependency direction, secret serialization, test presence, coverage floors, mutation score, invariant assertions, contract compatibility, migration safety, a11y rules, performance budgets, licence and advisory posture, and API/CLI parity. All of these are objective, and a human is strictly worse at them than a machine — slower, less consistent, and subject to fatigue.

**What a human reviewer is uniquely responsible for**, and must actively spend their attention on:

1. **Is this the right problem?** Agents solve the stated problem excellently and will not notice that the stated problem is the wrong one. This is the single highest-value thing a human does in review.
2. **Is the scope right?** Agents expand scope helpfully — adding a config option, generalizing a function, handling a case nobody asked about. `02 §7` exists because this failure is systematic, and it is invisible to every automated gate because the extra code is usually well-written and tested.
3. **Is the abstraction real?** An agent will produce a plausible-looking abstraction with one caller because the pattern is familiar from its training distribution. Two callers or the P5 list; otherwise it is speculative generality.
4. **Do the tests test the intent, or the implementation?** Agents write tests that mirror the code they just wrote, which pass whether or not the code is correct. A test that asserts the function does what the function does is worse than no test, because it creates false confidence and makes the code harder to change. This is the most common serious defect in agent-authored PRs and it is not detectable by coverage.
5. **Is a principle being satisfied in letter but violated in spirit?** An `envelope_ref` field that is populated with a placeholder satisfies the lint and defeats P4. A `society_id` column that is always the same sentinel satisfies P1's grep and defeats its purpose.
6. **Semantic diff against the domain model.** Does this change quietly alter an invariant in `11 §7` without saying so?
7. **Operational consequence.** What does this do to the on-call rotation, the cost profile, the support burden? Nothing in the diff shows it.

**Escalation triggers — an agent PR requires a second, senior human reviewer if any of these hold:**

- it touches an R0 crate (ledger, emission, Fracture, Envelope evaluation, E2EE, Charter enactment);
- it changes a `11 §7` invariant, an event schema, or the Global Registry;
- it adds a dependency, a port, or an `unsafe` block;
- it modifies the simulation harness, its oracles, or the CI gates themselves — **a change that weakens a gate is reviewed with more suspicion than a change to the code the gate protects**;
- it exceeds 400 changed lines excluding generated code and lockfiles;
- the agent's PR description does not cite a principle number where a design decision was made;
- the agent reports having disabled, skipped, or `#[ignore]`d a test.

**Review SLA:** first response within 4 working hours, full review within 1 working day for PRs under 400 lines. PRs over 400 lines may be returned unreviewed with a request to split — the evidence that review quality collapses beyond a few hundred lines is overwhelming, and with agents doing the authoring, splitting is nearly free.

**Separate refactor PRs, always.** `02 §7` forbids "while I'm in here" refactors in a feature PR. The reason is specific to review economics: a diff mixing behavioural change with mechanical change forces the reviewer to distinguish the two by hand, and they will not. Refactors get their own PR, whose review question is the single easy one — "is this behaviour-preserving?" — and feature PRs stay small enough to actually read.

---

## 15. Working Agreements for AI Coding Agents

These are binding. An agent that violates one has produced a defect regardless of whether the code works.

1. **Load the Canon first.** `00`, `01`, `02` before any implementation task, plus the owning document for the area you are touching. Do not begin from the ticket alone; the ticket is always underspecified relative to the Canon.
2. **Cite principle numbers.** Every non-obvious design decision in the PR description names the principle it serves. "Serves P8" with no explanation is not a citation; state the mechanism.
3. **Halt on principle conflict.** When two principles genuinely conflict, apply the `00 §2` order, and if the resolution is not mechanical, **stop and ask**. Escalation is cheap. A violated invariant discovered in Phase 6 is not. Do not resolve a conflict unilaterally and note it in the PR body; that is indistinguishable from not noticing.
4. **Never expand scope.** Build the smallest version that makes the spine sentence (`02 §2`) more true. No extra configuration options, no speculative generalization, no adjacent bug fixes, no drive-by renames. If you find something else broken, file it.
5. **Never self-report done without all eight criteria.** `00 §5`. Seven of eight is a defect with good marketing. If a criterion cannot be met, say which one and why, in the PR description, before requesting review.
6. **Never introduce new terminology.** If the word you need is not in `01`, either the word is wrong or `01` needs a PR — and that PR is separate, and a human accepts it. The terminology lint will catch you, but the lint is the backstop, not the process.
7. **Answer the four questions** from `02 §6` in the PR description before writing code: which principle, which phase, what is the smallest version, what does it cost forever.
8. **PR description format**, required:
   - *What changed* — one paragraph, plain language.
   - *Why* — the principle(s) served, with the mechanism.
   - *The four questions* (`02 §6`).
   - *Invariants touched* — which of `11 §7`, and how they are still upheld.
   - *Tests added* — by level, and what each one would catch that the others would not.
   - *Risk* — what breaks if this is wrong, and how it would be noticed in production.
   - *Rollback* — how to undo it, including any migration consideration.
   - *Not done* — anything deliberately left out, with a reason. An empty section is a claim; make it deliberately.
9. **A failing test you did not cause is not yours to change.** Never delete, `#[ignore]`, weaken, or "fix" an assertion in a test you did not write in order to make your PR green. Correct procedure: reproduce on a clean `main`; if it fails there, file it and note it in your PR; if it fails only on your branch, your change caused it and the test is right. Weakening an assertion to pass is the single most damaging action an agent can take in this repository, because it converts a working gate into a decorative one and nobody finds out until production.
10. **Never weaken a gate to land a change.** Lints, budgets, thresholds, invariant assertions, and CI configuration are not adjustable by the PR that they block. Changing them is a separate PR with its own justification and a senior human reviewer.
11. **Attach the profile.** Any performance claim comes with a measurement (§13.3), before and after.
12. **Report uncertainty explicitly.** If you are unsure whether an approach satisfies a principle, say so in the PR at the point of doubt. An agent that hedges honestly is more useful than one that is confidently wrong, and reviewers calibrate on this.
13. **Prefer deleting to adding.** The best PR in a system this size is frequently a smaller one. Code you do not write is code that cannot break, cannot be reviewed wrongly, and does not need a test.

---

## 16. Trade-offs and Rejected Alternatives

| Decision | Rejected alternative | Honest reason |
|---|---|---|
| Deterministic simulation as the primary correctness gate | Chaos testing in staging | Non-reproducible failures are anecdotes. We add chaos testing at Phase 5 as a complement for adapter-level surprises, but it cannot be the gate |
| Deterministic simulation | Formal verification (TLA+) throughout | Verifies a model, not the code. We use TLA+ for Fracture and the Ledger specifically, as an input to the harness's oracles, not as a substitute |
| Risk-class-based coverage floors | A single workspace-wide number | One number either under-protects the ledger or wastes effort on presentation code. Both outcomes are worse than a table |
| `forbid(unsafe_code)` | `deny` with documented allows | `deny` is silently overridable by an inner attribute; `forbid` is not. The narrow exception process (§2.4) is the pressure valve |
| Continuous deploy for the Runtime | Weekly release train | Batch size dominates deployment risk, and a rollback path used weekly is a rollback path that works. Cost: it demands the progressive-delivery automation to be genuinely reliable |
| Release trains for clients | Continuous client deploy | Store review, signing, and user-controlled upgrades make per-commit client releases meaningless |
| Alert on symptoms and burn rate | Alert on causes (CPU, memory, queue depth) | Cause alerts fire when nothing is wrong, get muted, and then the real one is muted too |
| Per-Society backup granularity | Whole-database backup only | P1 makes per-Society free; whole-database restore has a 4h RTO for a problem affecting one tenant |
| Mutation testing on R0/R1 only | Workspace-wide mutation testing | Runtime cost is superlinear and the marginal value on presentation code is near zero |
| Automatic flaky-test quarantine with a delete SLA | Retry-until-green | Retries hide real races. Our worst future bugs are races; a policy that hides them is a policy that ships them |
| Custom Canon lints | Documentation plus review discipline | Review discipline decays under volume, and volume is the defining condition of this project |
| Human-only ADR acceptance | Let agents accept ADRs to move faster | P4: architecture is policy, and policy is authored by humans. Agents draft; humans decide |
| Coverage as a floor | Coverage as a target with a rising number | A rising coverage target reliably produces assertion-free tests. Mutation score is the metric that cannot be gamed this way |
| 20 E2E tests, capped | Broad E2E suites | E2E flakiness scales superlinearly and erodes trust in the whole pipeline |

---

## 17. What Would Make Us Change These Standards

Stated in advance, so the team recognizes the signal instead of rationalizing around it.

- **The fast lane exceeds 8 minutes at p50 and cannot be brought back with caching or sharding.** Then we split the workspace further and move clippy's `pedantic` group to the full lane. We do not remove the gate.
- **Simulation seeds stop finding bugs for two consecutive phases while production incidents continue.** That means the generator's distribution does not match reality. Fix the generator using the incident corpus; the harness is not the thing at fault.
- **Mutation testing costs more reviewer time in false signal than it returns in found defects.** Then it becomes a weekly report on R0 only rather than a per-PR gate.
- **Agent PR volume exceeds human review capacity by more than 3×.** Then we do not lower the bar; we raise the automation — more invariant assertions, more contract generation, stricter escalation triggers — and we cap merge throughput. Reviewing badly is worse than merging slowly.
- **An `unsafe` requirement becomes genuinely unavoidable** (most plausibly in a media codec or SIMD path). Then §2.4's process runs, and the expected count moves from zero to a small named list, not to a general permission.
- **WCAG 2.3 or a successor lands.** We adopt the new AA within two phases and restate the AAA-contrast commitment against the new definition.
