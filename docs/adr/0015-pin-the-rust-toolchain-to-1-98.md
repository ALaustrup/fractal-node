# ADR-0015 — Pin the Rust toolchain to 1.98.1

Status: Accepted
Date: 2026-09-04
Deciders: Andrew
Phase: PH0 (M0.2)
Supersedes: the `1.83.0` pin written into `rust-toolchain.toml` at M0.1

## Context

`rust-toolchain.toml` was initially pinned to 1.83.0 — a deliberately conservative
choice, on the reasoning that a pinned toolchain is a stability decision and older
is safer.

That reasoning was wrong, and it failed on the first real dependency. `clap 4.6.6`
depends on `clap_lex 1.1.0`, which declares `edition2024`. Cargo 1.83 does not
stabilise that edition, so the build fails at manifest-parse time — before any of
our code is compiled. The same wall is now standard across the ecosystem: edition
2024 stabilised in 1.85 (February 2025) and, eighteen months on, is what
maintained crates target.

Two paths existed:

1. Pin every affected dependency to its last pre-edition-2024 release.
2. Move the toolchain forward.

Path 1 was rejected after costing it honestly. It is not one pin; it is a pin per
affected crate, re-litigated on every dependency addition, on a curve that only
gets worse. It would also silently freeze us out of security patches on those
crates, which converts a build-convenience decision into a supply-chain decision
(P8) without anyone noticing.

## Decision

Pin `rust-toolchain.toml` to **1.98.1** and set `rust-version = "1.98"` in the
workspace manifest. Both CI lanes pin the same version explicitly rather than
tracking `stable`.

The toolchain remains **pinned, not floating**. The failure mode this ADR corrects
is "pinned too far back", not "pinned at all": a floating toolchain means the
compiler that agents and CI use can change under us between two runs of the same
commit, which would make a red build unreproducible.

## Consequences

**Positive**

- Dependencies resolve normally. No per-crate version archaeology.
- Edition 2024 is available to us, including `gen` blocks and the stricter
  `unsafe_op_in_unsafe_fn` default — the latter matters because `unsafe_code` is
  `forbid` workspace-wide and we want the compiler as strict as it can be.
- CI, agents and developer machines run identical compilers, byte for byte.

**Negative**

- The pin is now close to the release frontier, so we inherit regressions faster
  than a conservative pin would. Mitigated by the pin itself: a bad release cannot
  reach us until someone edits this file.
- Bumping it is a deliberate act with a blast radius across every crate, every
  agent and every CI job. That cost is the point.
- Contributors on distribution-packaged Rust older than 1.98 must use `rustup`.
  Acceptable: `rustup` is already the documented setup path.

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| Pin dependencies to pre-edition-2024 releases | Recurring cost on every dependency change; freezes out security patches; converts a convenience decision into a supply-chain one |
| Track `stable` unpinned | A red build stops being reproducible; two agents can compile the same commit with different compilers |
| Pin to the oldest toolchain the dependency graph allows | Same failure, deferred. We would be back here on the next dependency |

## Exit Cost

Two engineer-hours: edit two files, re-run CI. Deliberately cheap — that is why
the toolchain lives in one pinned file rather than in developer setup docs.

## Principle Served

P5 (technology stays swappable, including the compiler), P8 (security patches
reach us through normal dependency updates rather than requiring a pin audit).

## Falsification Test

`cargo build --workspace` succeeds from a cold cache on Linux, macOS and Windows,
and `rustc --version` reports 1.98.1 in every CI job. If a dependency addition
ever requires editing a `=x.y.z` pin to satisfy the compiler, this decision has
failed and the toolchain is too old again.

## Maintenance Horizon

Review at each phase gate. Bump when a dependency we want requires it, or every
two minor releases, whichever comes first. Never bump inside a Milestone.

## Review Trigger

Any dependency that cannot be added without a version pin, or a rustc regression
that affects us.
