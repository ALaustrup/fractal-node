# FRACTAL NODE — MASTER BLUEPRINT

```
                              ⌁

                        ┌───────────┐
                     ╱  │           │  ╲
                    │   │     ◇     │   │
                     ╲  │           │  ╱
                        └───────────┘

                    F R A C T A L   N O D E
                        MASTER BLUEPRINT
```

**Version 1.0 · 2026-09-03 · Status: complete, reconciled, ready to execute**

---

## What this is

The definitive design and execution document for Fractal Node — an AI-native social ecosystem built around **Societies**: sovereign digital communities that can grow, evolve, and fracture into independent nodes.

It is written to be executed by a small human team directing AI coding agents. Every chapter is dense on purpose: an agent that must infer an unstated constraint will infer it wrongly. Roughly 271,000 words across 29 chapters, 14 ADRs, and a machine-readable phase manifest.

**It is internally consistent.** Chapter `60` is an adversarial self-critique that found 33 real cross-chapter contradictions and 16 structural weaknesses. Chapter `61` resolves 49 of them with explicit rulings and applied fixes. Six decisions remain open and are listed below.

---

## The one sentence everything serves

> A Citizen can create a Society, talk in it, store things in it, earn Fraction in it, and govern it — from a web GUI, a CLI, or an API, with an Agent helping.

Phases 0–3 make that sentence true and excellent. Everything after widens it. Anything that does not make it more true is deferred (`02-scope-guardrails.md`).

---

## Reading order

### If you are Andrew, read these five, in this order

| | Chapter | Why |
|---|---|---|
| 1 | [`00-foundational-principles.md`](docs/00-foundational-principles.md) | The thirteen principles everything is judged against. Approve or amend this first — it constrains every later decision. |
| 2 | [`50-roadmap-phases.md`](docs/50-roadmap-phases.md) | What gets built, in what order, with what gate. |
| 3 | [`70-innovation-proposals.md`](docs/70-innovation-proposals.md) | Twelve additions I am proposing. Four I would adopt into Phase 1. |
| 4 | [`60-self-critique.md`](docs/60-self-critique.md) | Where this design is weak, and what would make me abandon it. |
| 5 | [`33-brand-identity.md`](docs/33-brand-identity.md) | The identity system, derived from the real `fractalnode.net` and extended into a product and a marketing package. |

### If you are an engineer or a coding agent

**Load the Canon before writing any code.** Four files, non-negotiable:

```
  00-foundational-principles.md   the thirteen principles + the definition of done
  01-canonical-terminology.md     the vocabulary. Deviating from it is a defect.
  02-scope-guardrails.md          what we are NOT building yet, and why
  03-phase-authority.md           the authoritative capability→phase map (+ phases.toml)
```

Then read the chapter you are implementing, and its prerequisites.

### Full chapter map

| # | Chapter | Words | What it settles |
|---|---|---:|---|
| **00** | [Foundational Principles](docs/00-foundational-principles.md) | 2.6k | P1–P13, conflict resolution order, technology doctrine, the 8-point definition of done |
| **01** | [Canonical Terminology](docs/01-canonical-terminology.md) | 2.8k | Every term, every verb, every naming convention, the banned-word list |
| **02** | [Scope Guardrails](docs/02-scope-guardrails.md) | 1.5k | The spine, the Not-Yet list, the Never list, complexity budgets |
| **03** | [Phase Authority](docs/03-phase-authority.md) | 8.2k | 203 capabilities mapped to phases; the budget ledger. Authoritative over every other phase claim. |
| **10** | [System Architecture](docs/10-system-architecture.md) | 4.4k | The Runtime, 15 service boundaries, the event model, the port surface, deployment |
| **11** | [Domain Model](docs/11-domain-model.md) | 4.3k | Aggregates, invariants, state machines, Crystallization, **Fracture** |
| **12** | [Identity and Trust](docs/12-identity-and-trust.md) | 7.2k | FNID, devices, recovery, the capability grammar, Trust, Sybil resistance |
| **13** | [Data and Storage](docs/13-data-and-storage.md) | 7.1k | Chunking, erasure coding, Custodians, Attestation, the media pipeline |
| **14** | [Realtime and Social](docs/14-realtime-and-social.md) | 6.9k | Signals, MLS E2EE, moderation under encryption, voice/video, discovery without surveillance |
| **15** | [Agent Runtime](docs/15-agent-runtime.md) | 6.3k | Envelopes, Policy, the Enforcement Point, Workflows, prompt-injection defense |
| **16** | [Ledger and Assets](docs/16-ledger-and-assets.md) | 12.1k | The `Ledger` trait, double-entry, anchoring, the FN-ASSET/1 Facet standard |
| **17** | [Economy: Fraction](docs/17-economy-fraction.md) | 10.4k | Sources, Sinks, emission, anti-inflation, contribution metrics, attack margins |
| **18** | [Progression and Reputation](docs/18-progression-and-reputation.md) | 10.2k | XP, Levels, Trust, Standing, Unlocks, Achievements, Seasons |
| **19** | [Marketplace](docs/19-marketplace.md) | 10.5k | Listings, licensing, revenue share, review pipeline, recall |
| **20** | [Plugin and Extension Model](docs/20-plugin-and-extension-model.md) | 8.0k | WASM Component Model, the hook catalog, the Experience Runtime |
| **21** | [Media and Identity](docs/21-media-and-identity.md) | 12.9k | Profiles as digital homes, the Module grid, galleries, sharing, ambient accrual |
| **30** | [API and SDK](docs/30-api-and-sdk.md) | 15.1k | The contract, versioning, the resource map, six SDKs, the agent-facing surface |
| **31** | [CLI and Terminal](docs/31-cli-and-terminal.md) | 2.9k | `fractal` / `fn`, machine-readable output, the boot sequence, TUI dashboards |
| **32** | [Design System — LATTICE](docs/32-design-system.md) | 3.3k | Tokens, layout, components, states, a11y, performance budgets, the CLI palette |
| **33** | [Brand Identity](docs/33-brand-identity.md) | 3.6k | Color, type, motion, form language, voice, the marketing package |
| **34** | [Client Platform Strategy](docs/34-client-platform-strategy.md) | 12.8k | Shared Rust core; Windows flagship; web, PWA, Android, iOS, macOS, Linux, CLI |
| **40** | [Engineering Standards](docs/40-engineering-standards.md) | 15.8k | Lints, ADRs, deterministic simulation testing, CI/CD, observability, backups |
| **41** | [Repo and Crate Structure](docs/41-repo-and-crate-structure.md) | 13.6k | The monorepo, the Rust workspace, dependency enforcement, codegen |
| **42** | [Source Control Automation](docs/42-source-control-automation.md) | 12.1k | Work Units, the agent commit protocol, changelogs, milestone gates, hygiene |
| **50** | [Roadmap: Phases](docs/50-roadmap-phases.md) | 5.9k | PH0–PH9 with goals, milestones, risks, and acceptance criteria |
| **51** | [Phase 1 Web GUI](docs/51-phase-1-web-gui.md) | 15.2k | The build spec: 28 surfaces, 41 components, 9 wireframes, 82 Work Units |
| **60** | [Self-Critique](docs/60-self-critique.md) | 13.8k | 16 weaknesses, 13-axis scaling analysis, 33 contradictions, abandonment criteria |
| **61** | [Reconciliation](docs/61-reconciliation.md) | 18.3k | 49 rulings resolving the above. The authority on every conflict. |
| **70** | [Innovation Proposals](docs/70-innovation-proposals.md) | 4.3k | Twelve proposals, priced and risked. Four recommended for PH1. |
| — | [ADRs 0001–0014](docs/adr/) | 18.7k | The seed architecture decisions, each with a falsification test |
| — | [`phases.toml`](docs/phases.toml) | — | Machine-readable phase manifest for `cargo xtask phase-check` |

---

## The architecture in ten lines

1. **The Society is the atomic container.** Every persistent object answers "which Society owns you?" — or appears on a closed nine-entry Global Registry.
2. **One Rust Runtime, many peer front ends.** GUI, CLI, agents, and plugins all use the same public API. No interface wraps another.
3. **Event-sourced, per-Society ordering.** Total order inside a Society; causal order between them. **No global consensus in the hot path** — the single most important scalability decision here.
4. **Local-first.** The device holds an authoritative replica. Everything reads offline. Money never writes optimistically.
5. **Ledger abstracted from commit one.** Internal double-entry now; a Fractal Node L1 later is a swap behind an unchanged trait, not a rewrite.
6. **Agents execute; humans define policy.** Every agent action names the Envelope that permitted it, and every Envelope traces to a human signature.
7. **Capability-secure extensions.** WASM Component Model, no ambient authority, declarative UI from design-system primitives only.
8. **Content-addressed, encrypted-before-chunked storage** with erasure coding and Custodians paid against verified Attestations.
9. **An economy with named Sources and named Sinks**, a hard emission cap, and attack margins computed to be negative.
10. **Everything mechanically enforced.** Dependency-direction lints, a parity suite, 15 domain invariants as property tests, and deterministic simulation replaying whole histories.

---

## Open decisions that need Andrew

From `61 §6`. Work can start without these, but each blocks a specific later phase.

| | Decision | Blocks | Recommendation |
|---|---|---|---|
| **D1** | Fund a second Milestone approver, or accept that human review is the throughput ceiling | PH1 velocity | Fund it, or state the ceiling honestly in `42` |
| **D2** | Jurisdictional posture on E2EE — **requires counsel** | PH2 | Publish the served-jurisdiction list and pre-commit to geoblocking rather than weakening N6 |
| **D3** | Two economic Sources in PH7, overriding the stated zero | PH7 | Accept; cut the third first-party Experience |
| **D4** | Onboarding/vouching Source currently lands at PH8 | PH5 | Ship the invite mechanism at PH5 with the reward set to zero; activate later |
| **D5** | Whether Extensions ever get a pixel escape hatch | PH6 | Host-owned, input-mediated canvas in PH6; pay for it by cutting Society storefronts |
| **D6** | Accept ADRs 0015 (phase authority) and 0016 (per-artifact dependency budget) | PH0 gate | Accept |

Plus, from `70`: **adopt The Commons, the Platform Charter, the Instrument Panel, and the Undo Window into PH1?** My recommendation is yes, paid for by deferring the full Profile Module catalogue and the search surface to PH2.

---

## Start here on day one

```
  1. Andrew approves 00-foundational-principles.md, 01, 02, 03.        ← the Canon
  2. Answer D1–D6 and the 70 §13 adoption question.
  3. Open PH0 M0.1: monorepo per docs/41, Canon committed, AGENTS.md.
  4. Then M0.2 → M0.8 in order. PH0 is ~3 weeks and its exit gate is
     "the repository can build, test, and release itself."
  5. PH1 begins the working web GUI. Its spec is docs/51 — 82 Work Units,
     ready to hand to agents.
```

**Do not start PH1 before PH0's exit gate.** The codegen pipeline, the token pipeline, and the determinism harness are what make every later phase cheap. Built after the fact, they are never built.

---

## The rule that matters most

From `02 §8`: **nothing enters a phase without something leaving it.**

Every ambitious project dies the same way — not from one bad decision, but from a hundred reasonable additions that nobody was willing to refuse. This document's real function is to make refusal easy, specific, and defensible.
