# 02 — Scope Guardrails

> **Status:** Canon. The third file of the four-file Canon contract (`00`, `01`, `02`, `03`).
> **Purpose:** Prevent scope explosion. This document is what an agent or engineer points at when saying "no, not yet."
> **Phase authority:** `03-phase-authority.md` and `docs/phases.toml`. Every phase named in this file is a **convenience view rendered from them** and is non-normative (`03 §0`).

---

## 1. The Failure Mode This Prevents

Fractal Node's specification is enormous: societies, agents, economy, storage, media, realtime, governance, marketplace, assets, six client platforms, a CLI, and an experience runtime. Every one of those is a company. Built naively — all at once, at 40% depth — the result is a demo that impresses nobody and cannot be finished.

The counter-strategy is **depth-first through a narrow spine**. Ship a small number of things at production quality, in an architecture that has *room* for the rest, and add breadth only when the spine holds weight.

The architecture is designed for all of it from day one. The *implementation* is not.

---

## 2. The Spine

Everything in Phase 0–3 exists to make this single sentence true and excellent:

> **A Citizen can create a Society, talk in it, store things in it, earn Fraction in it, and govern it — from a web GUI, a CLI, or an API, with an Agent helping.**

Nothing that does not serve that sentence ships before Phase 4. When a proposal arrives, the first question is: *does the spine sentence become more true, or only wider?*

---

## 3. The Not-Yet List

These are **architected for and deliberately not built** in early phases. Each has an owning document that specifies it fully, and a phase where it becomes real. Building any of these early is a scope violation regardless of how easy it looks.

**Non-normative.** The *Built in* column below is generated from `docs/phases.toml` by `cargo xtask lint-phases` and is reproduced here so a reader of this chapter can see the shape without a second lookup. **`03-phase-authority.md` §2 is authoritative**; where this table and `03` disagree, `03` is right and this table is stale. A hand-edited cell here fails the build.

| Capability | Architected in | Built in (non-normative — see `03 §2`) | Why deferred |
|---|---|---|---|
| Custom Fractal Node L1 chain | `16` | PH8 | The `Ledger` trait makes it a swap, not a rewrite. Building a chain before there is anything to secure is the classic sequencing error. |
| External chain bridges (ETH/SOL/etc.) | `16` | PH8 | Regulatory and security surface with zero early user value. Adapter shape is fixed now; adapters come later. |
| Fiat on/off ramps | `17` | PH9 | Requires licensing, KYC/AML, and a legal entity. Rail abstraction ready; implementation gated on counsel. |
| External FRC exchangeability | `17` | PH9 | Premature external liquidity destroys a young internal economy. Internal-only until the sink/source loop is proven stable in simulation and in production. |
| Voice and video | `14` | PH4 | Text realtime + E2EE must be correct first. SFU operations are a permanent cost center. |
| Native mobile apps (iOS/Android) | `34` | PH5 | PWA covers mobile need through Phase 4. Store review cycles will slow iteration. |
| macOS / Linux desktop builds | `34` | PH5 | Same Tauri shell; cost is signing/notarization/packaging, not code. |
| Experience Runtime (games, hosted apps) | `20` | PH7 | The most seductive scope trap in the entire document. The sandbox model is designed now; nothing executes untrusted third-party code until the Envelope system has been adversarially tested for two phases. |
| Third-party paid Extensions | `19` | PH6 | Requires revenue share, licensing, payouts, dispute handling, and a security review pipeline. First-party + free third-party first. |
| Federation between Societies | `11` | PH6 | Cross-society consistency is hard. Single-society correctness first. |
| Society Fracture | `11` | PH5 | The signature feature — and the one most likely to corrupt data if built before the event log, ledger, and storage boundaries have settled. |
| P2P Custodian network with rewards | `13` | PH4 | Phase 0–3 use server-side object storage behind the same `BlobStore` trait. Distribution is an implementation swap. |
| Seasons and dynamic content | `18` | PH6 | Requires a stable progression baseline to be additive to. |
| Advanced discovery / interest matching | `14` | PH5 | Needs a population. Ships as explicit interest declarations + search first. |
| Governance beyond role-based Charter | `11` | PH5 | Voting, proposals, delegation. Roles and permissions first. |
| Plugin marketplace payments | `19` | PH6 | See third-party paid Extensions. |
| Multi-node self-hosting | `10` | PH6 | Single-tenant hosted Runtime through Phase 5, with the Node abstraction honest from day one. |

---

## 4. The Never List

Not deferred — rejected. Reopening any of these requires an ADR that overturns a Foundational Principle.

- **Advertising, ad targeting, or engagement-optimized ranking.** Violates P9 and P12.
- **Behavioral surveillance for recommendation.** Discovery uses declared interests and opt-in signals only.
- **Custody of user private keys without an explicit, revocable, user-initiated delegation.** Violates P8.
- **Server-side plaintext access to private messages, voice, or video.** Violates P8/N6. Not "unused" — *absent from the code*.
- **Pay-to-win progression.** Fraction can buy cosmetics, storage, compute, and Extensions. It can never buy XP, Trust, Standing, or governance weight.
- **Infinite or discretionary Fraction emission.** Violates P12.
- **A GUI-only feature.** Violates P3/P13.
- **Dark patterns**: streak anxiety, artificial scarcity timers, loot boxes, confirm-shaming, hidden costs.
- **Silent telemetry.** All telemetry is documented, categorized, and user-inspectable.

---

## 5. Complexity Budget

Each phase has a hard budget. Exceeding it is a phase failure, not a stretch goal.

| Budget | Limit | Rationale |
|---|---|---|
| New top-level services per phase | 2 | Operational surface grows superlinearly |
| New public API resource families per phase | 3 | Each is a permanent versioning commitment |
| New third-party runtime dependencies per phase, **per deployable artifact** | 5 | Each is a supply-chain surface and a maintenance horizon (P-doctrine §3.5). Counted separately for the Runtime binary, the web client, the CLI binary, the desktop shell and each mobile shell: a React dependency is not in the Runtime's supply chain and cannot sensibly compete with a crypto crate for the same slot. Per-phase consumption is tabulated in `03 §4.2`. Amended by `61 N2`; ADR-0016 |
| New client platforms per phase | 1 | Parity debt compounds |
| New economic Sources per phase | 2 | Each Source is an attack surface requiring its own simulation |
| Open ADRs at a phase gate | 0 | Undecided architecture is unbuildable architecture |

Budget consumption is not a matter of opinion: `03 §4.2` tabulates what every phase actually draws on all six axes, `docs/phases.toml` carries the same figures as data, and `cargo xtask phase-check` sums them and fails the build on an overrun. A phase over budget is a phase failure, not a stretch goal.

---

## 6. The Four Questions Before Any New Work

An agent or engineer must answer all four in the PR description before starting non-trivial work:

1. **Which principle does this serve?** (cite P#)
2. **Which phase does `03-phase-authority.md` place it in?** Look up the capability row; if its Phase is not `current_phase` in `docs/phases.toml`, stop. Do not consult a phase column in any other chapter — they are generated views, and before `03` existed there were eight of them and they disagreed (`61 X8`).
3. **What is the smallest version that makes the spine sentence more true?** Build that.
4. **What will this cost forever?** Ongoing operations, support burden, versioning commitment, security surface.

If question 2's answer is "it's not in the roadmap," the correct output is a proposal document in `docs/proposals/`, not code.

---

## 7. Anti-Gold-Plating Rules

- **No abstraction without two callers.** The exception is the P5 swappable-boundary list, which is abstracted by decree because the second implementation is known to be coming.
- **No configuration option without a user who asked for it.** Every knob is a permanent support cost and a test-matrix multiplier.
- **No performance optimization without a profile.** Attach the flamegraph or the benchmark to the PR.
- **No new UI pattern without a design-system entry.** If it is not in `32-design-system.md`, it either gets added there first or it does not ship.
- **No "while I'm in here" refactors** in a feature PR. Separate commit, separate PR, separate review.

---

## 8. Scope Escalation Path

When something genuinely important is out of phase:

```
  Discovery ──► docs/proposals/NNNN-title.md
                     │
                     ├─ states: principle served, phase impact,
                     │          complexity-budget cost, exit cost
                     ▼
              Human review (Andrew)
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
   Accept into   Defer to      Reject
   current       named phase   (record why)
   phase*
        │
        ▼
   Something else is CUT to stay inside the budget
```

\* **Nothing enters a phase without something leaving it.** This is the rule that actually holds the line. An "accepted" scope addition with no corresponding cut is a rejected scope addition wearing a disguise.
