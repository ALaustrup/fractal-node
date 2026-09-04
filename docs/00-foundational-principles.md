# 00 — Foundational Principles

> **Status:** Canon. Locked before any technology choice.
> **Audience:** Every human engineer, designer, and AI coding agent working on Fractal Node.
> **Rule of precedence:** If any other document, ticket, or agent instruction conflicts with this file, this file wins. Amending it requires an ADR.

---

## 0. Why this document exists first

Most platforms die of one of two diseases: **incoherence** (a thousand reasonable local decisions that add up to an unusable whole) or **scope necrosis** (a roadmap so wide that nothing reaches production quality). Both are prevented the same way — by fixing a small set of non-negotiable principles *before* choosing a single library, then judging every subsequent decision against them.

Fractal Node is being built substantially by AI coding agents. Agents are extremely good at satisfying a stated constraint and extremely bad at inferring an unstated one. Therefore the constraints must be stated, numbered, testable, and quotable.

Every principle below is written as a **directive** (what you must do) plus a **falsification test** (how a reviewer or agent proves it was violated). A principle without a falsification test is a slogan, not a principle.

---

## 1. The Thirteen Principles

### P1 — Society-Centric
**Directive:** The *Society* is the atomic container of the platform. Every persistent object — chats, agents, wallets, storage, media, governance, plugins, assets, reputation scopes — is owned by exactly one Society, or is explicitly and deliberately global. There is no fourth kind of thing.

**Rationale:** A single atomic container keeps the mental model, the permission model, the billing model, the sharding model, and the migration model identical. When a Society fractures into an independent node, the boundary of what moves is already drawn.

**Falsification test:** Any table, API resource, storage bucket, or event that cannot answer the question "which Society owns you?" with a `society_id`, or is not on the explicit Global Registry list in `01-canonical-terminology.md`, is a violation.

---

### P2 — Local-First
**Directive:** The user's node holds an authoritative local replica of their own data. The network is a synchronization and discovery medium, not the source of truth for anything a user can reasonably own. Every read path must have an offline answer; every write path must have a queued-and-reconciled answer.

**Rationale:** Sovereignty is not a marketing word here — it is the product. A Society that cannot be read when the network is down was never sovereign. Local-first also delivers the perceived performance that the premium-UX principle (P10) demands, for free.

**Falsification test:** Disconnect the network. Any core surface (society timeline, chat history, wallet balance, media gallery, profile, CLI status) that renders an error rather than a last-known-good state with a staleness indicator is a violation.

**Known tension:** Local-first conflicts with server-side moderation and with strong global ordering. Resolution: see `12-identity-and-trust.md` (capability-gated reads) and `13-data-and-storage.md` (per-Society causal ordering, not global ordering).

---

### P3 — API-First
**Directive:** No capability exists until it exists as a versioned, documented, machine-consumable API. Every user-facing feature is built *on top of* that public API using the same credentials and rate limits a third party would get. There are no private back doors from the GUI to the database.

**Rationale:** This is the only mechanism that reliably prevents a first-class GUI and a second-class everything-else. It also makes the plugin marketplace, the agent runtime, and the CLI possible without a second implementation.

**Falsification test:** Grep the web, desktop, and mobile clients for any database driver, any direct queue access, or any HTTP call to an undocumented internal path. Each hit is a violation. The CLI must be able to perform 100% of the actions the GUI can perform.

---

### P4 — AI-First, Human-Governed
**Directive:** Agents are first-class principals with their own identity, wallet, reputation, rate limits, and audit trail — not scripts running as a user. But **policy is defined exclusively by humans.** An agent may propose, execute within a granted capability envelope, and report. An agent may never widen its own envelope, grant capabilities, alter governance rules, or take an irreversible action outside a pre-approved class.

**Rationale:** The value of agents is throughput. The danger of agents is unbounded authority. Separating *execution* (agent) from *policy* (human) captures the first without the second.

**Falsification test:** Trace any capability grant in the audit log to a human signature. Any grant whose provenance chain terminates in an agent rather than a human is a violation. Any irreversible action (fund transfer above threshold, member removal, asset burn, governance change, external publication) executed without a human-signed policy authorizing that *class* of action is a violation.

---

### P5 — Modular and Swappable
**Directive:** Ledger, blockchain, storage backend, media transcoder, transport, identity provider, payment rail, and AI model provider are all reached through a trait/interface boundary with at least two implementations at the time the boundary is created (typically the real one and an in-memory test double). No concrete vendor type may appear in a domain crate.

**Rationale:** The single most expensive class of rewrite is a vendor assumption that leaked into the domain. The custom Fractal Node chain does not exist yet; the architecture must not care.

**Falsification test:** A dependency-direction lint in CI. Domain crates may not depend on adapter crates. Any `use` of a vendor SDK type inside `fractal-domain-*` fails the build.

---

### P6 — Event-Driven
**Directive:** State changes are expressed as immutable, ordered, typed **domain events** appended to a per-Society log. Read models, notifications, reputation, XP, ledger postings, search indexes, and agent triggers are all derived projections. Nothing mutates a projection directly.

**Rationale:** Auditability, replay, time-travel debugging, cheap new features (a new projection is a new consumer, not a migration), and — critically — an honest economy, because every Fraction that ever moved is a replayable fact.

**Falsification test:** Delete every projection and rebuild from the event log. Any state that cannot be reconstructed is a violation.

---

### P7 — Plugin-Driven
**Directive:** First-party features are built through the same extension surfaces third parties get. If a first-party feature needs a hook the plugin API does not expose, the correct action is to add it to the plugin API, not to bypass it.

**Rationale:** "Dogfood the extension API" is the only known way to get an extension API that is actually good. It also means the marketplace launches with proof that serious things can be built on it.

**Falsification test:** For each first-party feature shipped after Phase 3, name the plugin capability it uses. A feature with no answer is a violation.

---

### P8 — Secure by Default
**Directive:** Deny by default. Least privilege everywhere. Every capability is explicit, scoped, time-boxed, and revocable. Private messages, voice, and video are end-to-end encrypted with no server-side plaintext. Secrets never touch the event log. Every dependency is pinned and SBOM'd. Every release is signed.

**Falsification test:** A new principal (user, agent, plugin, node) created with zero grants must be able to do exactly nothing except read explicitly public data. Any implicit permission is a violation.

---

### P9 — Privacy by Default
**Directive:** Collect the minimum. Default every sharing control to the most private setting that still makes the feature work. Personal data is exportable and deletable by its owner. Discovery operates on user-declared interests and opt-in signals, never on covert behavioral surveillance. There is no advertising-derived data model.

**Rationale:** The product's premise is sovereignty. A surveillance-funded sovereignty platform is a contradiction that users will correctly detect.

**Falsification test:** For every field persisted about a user, name the feature that breaks without it and the user-facing control that governs it. A field with no answer to either is a violation.

---

### P10 — Performance and Accessibility First
**Directive:** Performance and accessibility are acceptance criteria, not polish. Budgets are defined in `32-design-system.md` and enforced in CI. Targets: cold start to interactive under 1.5s (desktop), under 2.5s (web, p75, mid-tier hardware); interaction-to-paint under 100ms; sustained 60fps during animation (120fps where the display allows); WCAG 2.2 AA as the floor, AAA for text contrast in the default theme; full keyboard operability of every surface; screen-reader labels on every interactive element.

**Falsification test:** CI performance and axe-core budgets fail the build. A merged PR that regresses a budget without an approved exception is a violation.

---

### P11 — Blockchain-Ready, Chain-Agnostic
**Directive:** The ledger is an internal, deterministic, auditable double-entry system behind a `Ledger` trait from day one. Digital assets use a native Fractal Node asset standard that is not modeled on any existing chain's token standard. External chains are *adapters*, not foundations. Migration to a future Fractal Node L1 must be a swap of implementation behind an unchanged trait plus a state-root anchoring procedure — never a rewrite.

**Falsification test:** Replace the ledger implementation with a stub that anchors to a local test chain. If any domain crate, API contract, or client requires changes, the abstraction failed.

---

### P12 — Economically Honest
**Directive:** Every Fraction has a named source and a named sink. Emission is bounded, measured, and published. Contribution metrics are defined, measurable, and resistant to being farmed. No mechanic exists whose primary function is to manufacture engagement rather than value.

**Rationale:** Token economies fail in exactly two ways — unbounded emission (inflation to zero) and Sybil-farmable rewards (capture by the cheapest actor). Both are design-time problems, not tuning problems.

**Falsification test:** The economy simulation harness (`17-economy-fraction.md` §12) must show a bounded circulating supply under adversarial farming at 100x normal actor volume. If it does not, the mechanic ships disabled.

---

### P13 — One Core, Many Front Ends
**Directive:** There is exactly one core runtime. GUI, CLI, agents, and plugins are peer front ends over the same core and the same public API. No interface is a wrapper around another interface. Feature parity is a release gate: a feature is not "shipped" until it exists in the API, the CLI, and at least one GUI.

**Falsification test:** Any feature present in the GUI but absent from the CLI, or vice versa, at a release tag is a violation and blocks the tag.

---

## 2. Principle Conflict Resolution Order

When two principles genuinely conflict, resolve in this fixed order. Do not improvise; cite the order in the ADR.

```
  1. P8  Secure by Default          ← never traded away
  2. P9  Privacy by Default
  3. P12 Economically Honest
  4. P4  AI-First, Human-Governed
  5. P1  Society-Centric
  6. P6  Event-Driven
  7. P2  Local-First
  8. P3  API-First
  9. P11 Blockchain-Ready
 10. P5  Modular and Swappable
 11. P13 One Core, Many Front Ends
 12. P10 Performance and Accessibility
 13. P7  Plugin-Driven
```

Read this as: we will accept a slower feature (P10) to keep it private (P9); we will accept a less elegant abstraction (P5) to keep the economy honest (P12); we will accept a delayed plugin surface (P7) to preserve the society boundary (P1).

**P10 sitting at position 12 does not mean performance is unimportant** — it means performance is never the *reason* to break security, privacy, or economic integrity. Within its own domain, P10 is a hard gate.

---

## 3. Technology Selection Doctrine

Principles are locked. Technology is not. Every technology choice must satisfy all of the following, and must be recorded as an ADR (`docs/adr/`) using the template in `40-engineering-standards.md`:

1. **Named principle service.** Which principle does this choice serve? A choice that serves none is decoration.
2. **Two alternatives considered.** With honest reasons for rejection, not strawmen.
3. **Exit cost stated.** How many engineer-weeks to replace it in 18 months? If the answer is "unbounded," it must sit behind a P5 trait boundary or be rejected.
4. **Maturity check.** Prefer boring where the problem is solved; prefer modern where the modern option removes an entire category of defect (memory safety, type safety, deterministic builds, structural concurrency). "New and exciting" is not a reason. "Removes a defect class" is.
5. **Maintenance horizon.** Who maintains it in five years? Single-maintainer critical-path dependencies require a vendoring plan.

**Standing bias, in order:** correctness > operability > developer velocity > raw benchmark performance. A system that is fast and wrong is worthless; a system that is fast and unobservable is unmaintainable.

---

## 4. The Non-Negotiables

These are stated separately because they have historically been the first things teams quietly abandon under deadline pressure.

| # | Non-negotiable | Enforcement |
|---|---|---|
| N1 | Source control discipline, with agent-automated hygiene at **milestone** granularity — never per file change | `42-source-control-automation.md`; CI rejects non-conforming commits |
| N2 | Cross-platform from day zero (shared core, native shells) — never a later port | `34-client-platform-strategy.md`; core crates must compile to `wasm32`, `x86_64`, `aarch64` in CI from Phase 0 |
| N3 | The CLI is a first-class citizen, not a wrapper | P13 falsification test at every release tag |
| N4 | The ledger is abstracted from commit #1 | P5/P11 dependency lint |
| N5 | Every user and Society has a native wallet on every surface | Phase gate; parity test suite |
| N6 | E2EE for private messages, voice, and video | Security review; no server-side plaintext path may exist in the code, not merely be unused |
| N7 | Design system is single-source and shared across every surface including CLI | Token pipeline emits CSS, Rust, Swift, Kotlin, and ANSI targets from one source |
| N8 | Accessibility AA floor | axe-core + manual audit per phase gate |

---

## 5. What "Done" Means

A unit of work is done when **all** of the following are true. Coding agents must treat this as the definition of the `completed` state and must not self-report completion otherwise.

1. It satisfies its written acceptance criteria verbatim.
2. It is reachable through the public API, the CLI, and (if user-facing) a GUI.
3. It has tests at the level appropriate to its risk class (`40-engineering-standards.md`).
4. It emits domain events and structured, correlated telemetry.
5. It has documentation: an API reference entry, a CLI help entry, and a changelog line.
6. It degrades correctly offline (P2) and denies correctly without permission (P8).
7. It respects the performance and accessibility budgets (P10).
8. Its ADR exists if it introduced or changed a technology choice.

Seven of eight is not done. It is a defect with good marketing.

---

## 6. How Agents Must Use This File

Coding agents working on Fractal Node must:

- Load `00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`, and `03-phase-authority.md` into context **before** any implementation task. These four files are collectively "the Canon." `03` is the single authority on which phase a capability belongs to; a phase column in any other chapter is a non-normative convenience view (`03 §0`).
- Cite the principle number in the PR description for any non-obvious design decision.
- **Halt and ask** rather than resolve a genuine principle conflict unilaterally. Escalation is cheap; a violated invariant discovered in Phase 6 is not.
- Never introduce a term not present in `01-canonical-terminology.md` without proposing it as an addition in the same PR.
- Never expand scope beyond the current phase's stated deliverables. The current phase's deliverables are the rows of `03-phase-authority.md` §2 whose Phase equals `current_phase` in `docs/phases.toml`, and nothing else. See `02-scope-guardrails.md`.

---

## 7. Amendment Procedure

This file changes only by ADR. The ADR must state: the principle affected, the concrete situation that forced reconsideration, the proposed new wording, the falsification test for the new wording, and the list of documents and code that must change as a consequence. Silent drift is the failure mode this procedure exists to prevent.
