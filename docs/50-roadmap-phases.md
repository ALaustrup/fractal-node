# 50 — Phased Roadmap: Day Zero to Production

> **Prerequisites:** the entire blueprint. This chapter sequences it.
> **Governs:** what gets built, in what order, with what gate. No work begins outside the current phase (`02 §6`).
> **Reading note:** durations assume a small human team (1–3) directing a fleet of AI coding agents, with one human approving Milestones and releases (`42`). They are estimates for sequencing, not commitments. `60 §3.11` puts the realistic multiple at **2.0–2.6×**, and `03 §1` carries both columns so no reader sees only the optimistic figure.
> **Phase authority:** this chapter owns sequencing rationale, milestones, entry criteria, risks, acceptance criteria and exit gates. **`03-phase-authority.md` owns which phase a capability belongs to**, and where the two disagreed, `61` records the resolution — eleven of them overrode this chapter, with the reason stated in `03 §2`.

---

## 1. The Sequencing Logic

The roadmap is ordered by **three rules**, applied in this order:

1. **Nothing before its dependency.** A phase may not begin until its entry criteria hold.
2. **Reversibility last.** The most irreversible decisions (chain, external liquidity, third-party code execution) come latest, when we know the most.
3. **The spine first, breadth after.** Phases 0–3 make the spine sentence from `02 §2` true and excellent. Everything after widens it.

```
     PH0        PH1        PH2        PH3        PH4        PH5        PH6      PH7    PH8   PH9
   ┌──────┐  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌───────┐ ┌────┐ ┌───┐ ┌───┐
   │FOUND-│  │ THE    │ │ THE    │ │ THE    │ │ THE    │ │ THE    │ │ THE   │ │EXP-│ │FN │ │EX-│
   │ATION │─►│ SPINE  │─►│ NODE   │─►│ AGENT │─►│ MESH  │─►│FRACTURE│─►│MARKET│─►│ER- │─►│L1 │─►│CH-│
   │      │  │(WEB GUI│ │(LOCAL- │ │(POLICY│ │(P2P +  │ │(SPLIT +│ │(CREATOR│ │IEN-│ │   │ │AN-│
   │      │  │ + CLI) │ │ FIRST) │ │+ EXT) │ │ VOICE) │ │ MOBILE)│ │ ECON) │ │CES │ │   │ │GE │
   └──────┘  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ └───────┘ └────┘ └───┘ └───┘
    ~3 wk      ~10 wk     ~10 wk     ~10 wk     ~14 wk     ~16 wk     ~14 wk   ~16wk  ~20wk  gated
      │           │          │          │          │          │          │
      │           └── FIRST REAL USERS  │          │          │          └── revenue
      │                      └── OFFLINE WORKS     │          └── the namesake operation
      │                                 └── AGENTS ARE REAL
      └── the repo can build itself

  ══════════════════ irreversibility increases ═══════════════════════════════════►
```

---

## 2. Phase Template

Every phase below is specified with:

- **Goal** — one sentence. If it takes two, the phase is two phases.
- **Entry criteria** — what must be true before starting.
- **Milestones** — the `42`-managed units, each with deliverables.
- **Dependencies** — what it needs from earlier phases.
- **Complexity budget check** — against `02 §5`.
- **Risks** — with mitigation and the trigger that means the risk fired.
- **Acceptance criteria** — binary, testable, no adjectives.
- **Exit gate** — the checklist that closes the phase.

---

## PH0 — FOUNDATION

**Goal.** The repository can build, test, and release itself, and one trivial end-to-end path works from browser to database and back.

**Entry criteria.** The Canon (`00`, `01`, `02`, `03`) is approved by Andrew, and `docs/phases.toml` matches `03 §2`.

**Duration.** ~3 weeks.

### Milestones

| ID | Milestone | Deliverables |
|---|---|---|
| M0.1 | Repo and Canon | Monorepo per `41`, the Canon committed, CODEOWNERS, `.git-blame-ignore-revs`, licence, security policy |
| M0.2 | Toolchain and gates | `rust-toolchain.toml`, `rustfmt.toml`, clippy lint set, the four Canon lints (dependency direction, no-literal-hex, banned terminology, `#[secret]`), hooks per `42 §16` |
| M0.3 | CI/CD skeleton | Fast lane <5 min, full lane, merge queue, `cargo xtask` (codegen, tokens, lint-deps, sim, parity, targets), tri-target build (x86_64 / aarch64 / wasm32) per N2 |
| M0.4 | Schema-first codegen | `fractal-schema` as the single source of truth; generation of OpenAPI, protobuf, JSON Schema, TS client, and the CLI command tree; the `codegen-diff` gate |
| M0.5 | Design token pipeline | `tokens/` source, `cargo xtask tokens` emitting CSS / Rust / Swift / Kotlin / ANSI, the drift gate (N7) |
| M0.6 | Determinism harness | `fractal-testkit`: `Clock`, `Rng`, `IdGen` fakes; the simulation runner; the first three of the fifteen `11 §7` invariants as property tests |
| M0.7 | Walking skeleton | `POST /v1/societies` → event → projection → `GET`; the same operation via `fn society create`; a web page that lists societies. Deployed to staging. |
| M0.8 | Agent working agreement | `AGENTS.md` at the repo root: the Canon loading rule, the commit protocol (`42`), the definition of done (`00 §5`), the halt-on-conflict rule |

**Complexity budget.** 0 new services (the Runtime is the first), 1 resource family (`societies`), 5 dependencies, 0 client platforms beyond the one, 0 economic Sources. Inside budget.

### Risks

| Risk | Mitigation | Fired when |
|---|---|---|
| Codegen pipeline becomes a bottleneck everyone routes around | Build it in M0.4 before there is anything to route around; make hand-edited generated code a CI failure | Anyone proposes "just this once" hand-editing |
| Rust compile times sink velocity immediately | Workspace splitting from the start (`41`), `sccache`, `cargo-chef`; measure clean and incremental build in CI from M0.3 | Incremental build > 40s |
| Over-scaffolding — building crates with nothing in them | `02 §7` anti-gold-plating; a crate is created when its first real type is written | An empty crate exists for more than one Milestone |

### Acceptance criteria

1. `git clone && cargo xtask ci` passes from a cold cache on Linux, macOS, and Windows.
2. Core crates compile to `x86_64`, `aarch64`, and `wasm32` in CI (N2).
3. Regenerating from `fractal-schema` produces a zero diff.
4. Regenerating tokens produces a zero diff across all five targets.
5. Creating a Society via the web page, the CLI, and the API produces byte-identical event streams (the first P13 parity test).
6. The simulation harness runs 2,000 seeded histories and asserts three invariants.
7. Fast-lane CI completes in under 5 minutes.

**Exit gate.** All acceptance criteria green · zero open ADRs · ADRs 0001–0014 accepted · `AGENTS.md` approved by Andrew.

---

## PH1 — THE SPINE  *(the working web GUI)*

**Goal.** A Citizen can register, create a Society, talk in it, hold Fraction, and see their progression — from a web GUI, a CLI, and the API, at production quality.

This is the phase the brief names as non-negotiable: **the first phase must provide a working web GUI.** Not a prototype. A deployed, accessible, fast, branded application that real people use.

**Entry criteria.** PH0 exit gate passed.

**Duration.** ~10 weeks.

### Milestones

| ID | Milestone | Deliverables |
|---|---|---|
| M1.1 | Identity | FNID, Handles, passkey registration and login, device enrolment, sessions, social-recovery configuration (`12`) |
| M1.2 | Society and Charter v0 | Society CRUD, Charter with roles and permissions (Founder governance only), Memberships, join policies (`11`) |
| M1.3 | Discourse | Chambers (text), Threads, Messages, reactions, edit/delete semantics, transport encryption, presence and typing (`14`) |
| M1.4 | Relay | Signal WebSocket, subscribe/resume, backpressure, reconnect (`14 §2`) |
| M1.5 | Ledger and Wallet | Internal double-entry ledger behind the `Ledger` trait, Citizen and Society wallets, Transfers, Postings, the balance invariant as a property test (`16`) |
| M1.6 | Progression v1 | XP, Levels 0–12, Trust, Standing, the first Unlock gates, `ContributionReceipt` (`18`) |
| M1.7 | Design system v1 | LATTICE implemented: tokens, the 40 Phase-1 components, all nine required artifacts each, Storybook, visual regression (`32`) |
| M1.8 | **Web GUI** | The Society shell (`32 §4.1`), the Chamber surface, the Wallet surface, the Profile surface, onboarding, the command palette, all four data states, a11y AA, performance budgets met (`51`) |
| M1.9 | CLI v1 | Full parity with the GUI, machine-readable contracts, `fn auth login`, the boot sequence, the Society dashboard (`31`) |
| M1.10 | Observability | OpenTelemetry end to end, SLOs, dashboards, alerting, the on-call runbook (`40 §10`) |
| M1.11 | Launch readiness | Backups with a restore drill, incident procedure, abuse reporting, ToS/privacy, staged rollout |

**Complexity budget.** 1 new service (Runtime) · 3 resource families (`citizens`, `societies`, `wallets` — chambers and messages are sub-resources of societies) · 5 dependencies · 1 client platform · 0 economic Sources (Fraction exists and moves; nothing emits it yet). Inside budget.

### Dependencies

```
  M1.1 Identity ──┬──► M1.2 Society ──┬──► M1.3 Discourse ──► M1.4 Relay ──┐
                  │                    │                                    │
                  └──► M1.5 Ledger ────┴──► M1.6 Progression ───────────────┤
                                                                            ▼
                          M1.7 Design System ──────────────────────► M1.8 WEB GUI
                                                                            │
                                                                            ├──► M1.9 CLI
                                                                            └──► M1.10/11
```

M1.7 (design system) runs in parallel from day one and is the long pole for M1.8. Start it on day one of the phase, not when the backend is ready.

### Risks

| Risk | Mitigation | Fired when |
|---|---|---|
| The GUI becomes a thin client that bypasses the API | P3 lint: no database or internal path access from any client; the parity test | Any client-side code imports anything but the generated client |
| Design system underestimated; UI ships inconsistent | Treat M1.7 as the critical path; 40 components with all nine artifacts is a real body of work, budgeted as such | Any screen ships with a one-off component |
| Passkey UX blocks registration on some browsers | Ship a device-code fallback and measure completion rate from day one | Registration completion < 85% |
| Performance budgets missed late, requiring rework | Budgets enforced in CI from M1.8's first commit, not at the end | Any budget red for more than one Work Unit |
| Scope creep into agents, media, or the marketplace | `02 §6` four questions; nothing enters without something leaving | A PR touches an out-of-phase boundary |

### Acceptance criteria

1. A new Citizen completes registration → Society creation → first message in under 3 minutes, unassisted, on desktop and mobile web.
2. Every Phase-1 feature is reachable via GUI, CLI, and API, with identical event streams (P13 parity suite green).
3. Web cold start to interactive ≤ 2.5s p75 on mid-tier hardware; interaction-to-paint ≤ 100ms p95.
4. axe-core reports zero violations on every surface; the manual a11y audit passes at AA with AAA text contrast in the default theme.
5. All fifteen `11 §7` invariants pass under 500,000 simulated histories.
6. `Σ debits == Σ credits` holds in production, verified continuously, with automatic freeze on divergence.
7. A full Society export/restore round-trips byte-identically (the P2 sovereignty proof, and the backup drill).
8. SLOs defined and met for 14 consecutive days on staging with production-like load.
9. Zero `unwrap` in production paths; zero open ADRs; zero known secrets in history.

**Exit gate.** All acceptance criteria green · security review passed · 14 days of stable staging · Andrew signs the release tag · **first external Citizens onboarded.**

---

## PH2 — THE NODE

**Goal.** The application works offline, runs as a real desktop Node, and holds media.

**Entry criteria.** PH1 shipped to external Citizens; SLOs met for 30 days.

**Duration.** ~10 weeks.

### Milestones

| ID | Milestone | Deliverables |
|---|---|---|
| M2.1 | `fractal-core` client core | Local SQLite store, replicated event log, outbox, sync engine, `sync_step(budget)` (`34 §2`) |
| M2.2 | Conflict policy | The per-data-class table from `10 §6` implemented, CRDTs for reactions/read-state/presence, server-authoritative money |
| M2.3 | Offline UX | Staleness indicators, queued-write rendering, the four data states everywhere, reconciliation surfacing (`32 §6`) |
| M2.4 | Desktop Node (Windows) | Tauri v2 shell embedding the Runtime, installer, signed differential updates, tray, notifications, deep links, autostart (`34`) |
| M2.5 | Responsive and ultrawide | The six breakpoints, density modes, the 88ch measure invariant, gamepad focus navigation (`32 §4.2`, `34`) |
| M2.6 | Vault v1 | Objects, versions, ACLs, upload/download, `BlobStore` on S3-compatible storage, quotas (`13`) |
| M2.7 | Media pipeline | Transcoding ladder, AV1/AVIF/Opus with fallbacks, thumbnails, blurhash, adaptive playback, verified streaming (`13 §9`) |
| M2.8 | Galleries and Profiles | Gallery Chambers, the Module grid, the customization contract with save-time a11y validation (`21`) |
| M2.9 | PWA | Installable, offline-capable, push, share target (`34`) |
| M2.10 | E2EE for private messages | MLS groups for DMs and Private Chambers, multi-device, franking for reports (`14 §4`) |

**Complexity budget.** 1 new service (Transcoder — the first `10 §2` extraction) · 3 resource families (`objects`, `profiles`, `devices`) · 5 dependencies · 1 platform (desktop; PWA is the same web build) · 0 Sources. Inside budget.

### Risks

| Risk | Mitigation | Fired when |
|---|---|---|
| Projection rebuild is unbounded and "drop and rebuild" is a multi-hour outage | Per-projection checkpoints generalized from `16 §4.3` land in PH1; a published rebuild SLO (≤ 15 min p99, any projection, any Society) is a PH2 gate criterion, measured against a ≥ 100M-event fixture (`61 W5`) | Any projection rebuild exceeds the SLO on the fixture |
| Sync engine correctness — the classic local-first swamp | Deterministic simulation with partition, reorder, duplication, and clock-jump injection, asserting invariants after every step | Any invariant fails under simulation |
| MLS implementation immaturity at Society scale | Limit E2EE to DMs and Private Chambers to **≤ 500 MLS leaves (≈200 Citizens at 2.5 devices/Citizen)** in PH2; measure; raise to the 1,000-leaf mechanism ceiling in PH4. Every MLS figure is stated in leaves (`61 X10`); the PH1 OpenMLS spike at 1,000 leaves on real hardware moves the `10 §12` fallback decision off this phase's critical path | Group operation latency > 500ms at 500 leaves |
| Tauri webview inconsistency threatens P10 on Windows | Measure on WebView2 from M2.4's first build; the `10 §12` fallback (Rust-native render path) is pre-costed | Desktop cold start > 1.5s p75 |
| Media storage costs outrun the model before the economy exists | Hard per-Citizen and per-Society quotas from M2.6; no unmetered storage, ever | Storage cost per active Citizen exceeds the `17` model by 50% |
| Offline UX becomes a source of user distrust ("did my message send?") | The staleness and pending-write specification in `32 §6` is an acceptance criterion, not a polish item | Any surface renders an error where it should render stale data |

### Acceptance criteria

1. With the network disabled, every core surface renders last-known-good state with a staleness indicator; writes queue and reconcile on reconnect with no data loss across 10,000 simulated disconnect cycles.
2. Desktop cold start ≤ 1.5s p75, warm ≤ 400ms, memory **≤ 450MB RSS under a 5-Society, 10k-cached-Message soak** (`32 §8`, reconciled in `61 X9` — `34 §11`'s figure was both stricter and better specified, and this criterion moves with it).
3. The GUI is correct and usable from 800px handheld to 5120px panorama; reading measure never exceeds 88ch.
4. A 4K video uploads, transcodes, and plays back adaptively with verified streaming; time-to-first-frame ≤ 800ms.
5. A Profile with 12 Modules renders within the frame budget and passes a11y validation; an invalid theme is rejected at save with the failing pair named.
6. DMs are E2EE with no server-side plaintext path present in the code (verified by security review, not by inspection of behavior).
7. Society export includes Vault Manifests and restores completely.

**Exit gate.** All criteria green · security review of the MLS integration · desktop signed and distributed · zero open ADRs.

---

## PH3 — THE AGENT

**Goal.** Agents are real, bounded, auditable participants, and the platform extends itself through the same API third parties will use.

**Entry criteria.** PH2 exit gate; the Envelope system design reviewed adversarially.

**Duration.** ~10 weeks.

### Milestones

| ID | Milestone | Deliverables |
|---|---|---|
| M3.1 | Envelope and Policy | Capability grammar, CapabilitySet algebra, attenuation-only grants, TTLs, revocation with in-flight semantics (`12`, `15`) |
| M3.2 | Policy Enforcement Point | In the application layer, on every command path, with `envelope_ref` on every event; `AgentActionBlocked` as a domain event (`10 §8`) |
| M3.3 | Agent runtime | Agent identity, Operator accountability, the execution loop, sandboxing, resource metering, checkpointing (`15`) |
| M3.4 | `ModelProvider` port | Hosted and local model adapters, per-Society model choice as a Charter parameter, the `ContextManifest` audit (`15 §9`) |
| M3.5 | Workflows | The declarative graph, triggers, steps, conditions, compensation, versioning; three first-party workflows shipped (`15 §7`) |
| M3.6 | Extension host | WASM Component Model, WIT world, manifest, install/consent, capability re-consent on update, resource limits (`20`) |
| M3.7 | First-party Extensions | The ten named Extensions that prove P7 (`20 §2`); each built on the public hook surface |
| M3.8 | Governance v1 | Charter amendment, Council governance, roles beyond Founder, moderation actions, appeals (`11 §2.3`) |
| M3.9 | Agent surfaces | `EnvelopeCard`, `PolicyEditor`, the audit trail in GUI and CLI, the Operator and Society kill switches |

**Complexity budget.** 1 new service (Agent Executor) · 3 resource families (`agents`, `envelopes`, `extensions`) · 5 dependencies · 0 platforms · 0 Sources. Inside budget.

### Risks

| Risk | Mitigation | Fired when |
|---|---|---|
| Envelope escalation bug — the highest-severity class in the system | Attenuation computed by the constructor, never requested; property tests on the CapabilitySet lattice; external security review before any third-party code runs | Any grant produces a capability the grantor lacked |
| Prompt injection converts untrusted content into authority | Architectural defense: ungranted capabilities are absent host imports, not denied calls (`15 §12`); the PEP evaluates the Envelope, never the prompt | Any agent action succeeds that the Envelope did not permit |
| Agents become noise; Citizens disengage | Agents cannot emit notifications (`14`); agent output is always visually distinguished; per-Chamber `agent_mode` defaults to `OnMention` | Agent-authored messages exceed 20% of Chamber volume |
| WASM component tooling stalls | P8 outranks P7 (`10 §12`): ship first-party Extensions natively and delay third-party execution rather than weakening isolation | Component tooling blocks for more than one Milestone |

### Acceptance criteria

1. No Envelope in 1,000,000 generated grant histories confers a capability its grantor lacked.
2. Every agent-authored event carries a valid, unexpired, unrevoked `envelope_ref`; a revoked Envelope fails an in-flight action at the PEP.
3. An adversarial prompt-injection suite (≥ 200 cases) produces zero unauthorized actions.
4. Every agent-authored message is visually distinguished in GUI, CLI, and mobile, with its Envelope inspectable in ≤ 2 actions.
5. The ten first-party Extensions are built on the public hook surface; the Reserved Hook Register is under 5% of hook surface.
6. An Extension update requesting additional capability cannot auto-install; per-Society re-consent shows only the diff.
7. A misbehaving Extension is detected, throttled, and disabled without degrading host performance below budget.

**Exit gate.** All criteria green · **external security audit of the Envelope and Extension sandbox** · zero open ADRs.

---

## PH4 — THE MESH

**Goal.** Storage becomes distributed and compensated, the economy starts emitting, and people can talk and see each other.

**Entry criteria.** PH3 exit gate including the external security audit; the economic simulation harness passing at 100× adversarial volume.

**Duration.** ~14 weeks. The heaviest technical phase.

### Milestones

| ID | Milestone | Deliverables |
|---|---|---|
| M4.1 | Custodian protocol | Registration, capacity commitment, Stake bonds, Shard assignment by XOR distance, graceful exit (`13 §6`) |
| M4.2 | Attestation | The challenge scheme, proof verification, failure handling, slashing (`13 §6`) |
| M4.3 | Erasure and repair | RS(10,16), placement across failure domains, repair triggers and coordination, the repair bandwidth model (`13 §5`) |
| M4.4 | `BlobStore` swap | Custodian mesh behind the unchanged trait; dual-write and verification period; cutover with rollback |
| M4.5 | Economy: Sources 1–2 | Storage custody and bandwidth service. Settlement windows, emission caps, receipts (`17`) |
| M4.6 | Economy: Sinks | The Phase-4 Sink set live; the emission ledger published; the public supply dashboard (`17`) |
| M4.7 | Voice | WebRTC + SFU, Voice Chambers, jitter/echo handling, SFrame E2EE media (`14 §7`) |
| M4.8 | Video and Stage | Simulcast/SVC, the Stage Chamber, recording with consent |
| M4.9 | Convergence | The pre-Society primitive, Serendipity, eligibility, Crystallization with full history preservation (`11 §3.1`) |
| M4.10 | Relay extraction | The Relay becomes its own service (the second `10 §2` extraction) |

**Complexity budget.** 2 new services (SFU/Relay, Custodian Coordinator) · 3 resource families (`custodians`, `convergences`, `sources`) · 5 dependencies · 0 platforms · **2 economic Sources** — exactly at budget on three axes. Nothing else enters PH4.

### Risks

| Risk | Mitigation | Fired when |
|---|---|---|
| Custodian economics are wrong and the mesh loses money or data | The simulation harness gates activation; Sources ship disabled and are enabled per-region behind a flag; sink-first settlement caps exposure | Emission exceeds the modelled cap in any window |
| Data loss during the `BlobStore` swap | Dual-write with full verification, 30 days of zero divergence before cutover, 90-day inverted rollback window | Any manifest fails verification |
| SFU operational cost and expertise become a permanent drag | Start with a managed SFU behind the `Relay` port; self-host only when volume justifies it | Media egress cost exceeds 25% of infrastructure spend |
| Sybil Custodians farm storage rewards | Self-dealing is negative by construction (`17`); XOR assignment means a Custodian cannot choose its Shards; attestation sampling detects withholding | Attack margin turns positive in the simulation |
| Crystallization loses history | The preservation invariants are property tests; a Convergence promotion is dry-runnable | Any message ID, signature, or tenure changes across promotion |

### Acceptance criteria

1. Simulated durability ≥ 10 nines under 5%/month Custodian churn with a 72-hour repair window; verified against a 90-day chaos run.
2. A Custodian that withholds 1% of its Shards is detected within one attestation epoch at ≥ 99% confidence.
3. Storage and bandwidth settlement runs for 30 days with emission within cap in every window and zero unbalanced Postings.
4. The economic simulation shows a negative attack margin for every active Source at 100× adversarial volume.
5. Voice: ≤ 150ms mouth-to-ear p95 within a region; 20-participant Chambers stable for 4 hours.
6. Crystallization preserves message IDs, authorship, signatures, reactions, and backdated tenure — verified by property test.
7. `BlobStore` cutover completes with zero manifest verification failures.

**Exit gate.** All criteria green · 30 days of settlement without an economic incident · public supply dashboard live · zero open ADRs.

---

## PH5 — FRACTURE

**Goal.** The namesake operation works, and the platform is native on every screen.

**Entry criteria.** PH4 exit gate; 90 days of stable economy; at least 20 Societies at Level 5.

**Duration.** ~16 weeks.

### Milestones

| ID | Milestone | Deliverables |
|---|---|---|
| M5.1 | Governance v2 | Proposals, voting, quorum/threshold, delegation, `fracture_rules` in the Charter (`11 §2.3`) |
| M5.2 | Fracture dry run | Full simulation of a split, the diff report, invariant checking, the `dry_run_token` |
| M5.3 | Fracture execution | Log sealing, child genesis, treasury division by balanced Postings, Vault re-referencing, membership and Standing carry-over, resumability (`11 §3.2`) |
| M5.4 | Fork and Dissolution | The sibling operations |
| M5.5 | Lineage | The ancestry graph, cross-generation history readability, lineage surfaces |
| M5.6 | Android (native) | SwiftUI/Compose over UniFFI per `34`; Android first |
| M5.7 | iOS (native) | Same core, native shell; App Store review process |
| M5.8 | macOS and Linux desktop | Same Tauri shell; signing, notarization, packaging (build targets, not new platforms) |
| M5.9 | Advanced discovery | The interest graph, intelligent matching on declared signals, Society discovery (`14 §9`) |
| M5.10 | Seasons infrastructure | Additive-only season content, objectives, themed Facets (`18`) |

**Complexity budget.** 0 new services · 3 resource families (`proposals`, `fractures`, `seasons`) · 5 dependencies · **1 platform** — macOS and Linux count as build targets of the existing desktop shell, and iOS/Android as one mobile platform decision executed twice (`34`). At budget. This is the phase most likely to breach; hold the line.

### Risks

| Risk | Mitigation | Fired when |
|---|---|---|
| **Fracture corrupts a Society.** The highest-consequence risk in the project. | Mandatory dry run; log sealed before any child write; resumable-forward only; balanced Postings; property tests on preservation; first 10 fractures manually reviewed by a human before execution | Any preservation invariant fails, ever |
| Mobile fragments the architecture | The anti-fragmentation rules (`34 §15`); no platform-specific business logic; the parity matrix with capped `N/A` | Any domain rule exists in Swift or Kotlin |
| App Store review rejects the token economy | Engage review guidance early; FRC is internal-only and non-exchangeable at this phase, which is the material fact; have counsel review the submission | Rejection on economic grounds |
| PH5 breaches the complexity budget | Nothing enters without something leaving (`02 §8`); Seasons is the designated cut if the phase slips | Two platform targets slip simultaneously |

### Acceptance criteria

1. Fracture preserves, verified by property test over 100,000 generated splits: total Fraction, total Facets, total members, full readable history, and every Citizen's tenure and Standing.
2. A Fracture interrupted at any step resumes to completion; partial fracture is unreachable.
3. Fracturing a 10TB Society moves zero bytes of Shard data.
4. Android and iOS reach the `34` definition of production quality: cold start, offline, a11y, and parity criteria all met.
5. The parity matrix shows zero unjustified `N/A`; every `N/A` has a closed-list reason, an ADR, a reviewer, and an expiry.
6. Discovery uses only declared and opt-in signals; the banned-signal audit is clean.

**Exit gate.** All criteria green · 10 supervised fractures completed successfully · mobile apps live in both stores · zero open ADRs.

---

## PH6 — THE MARKET

**Goal.** Creators and developers earn a living inside the ecosystem, and Societies can host themselves.

**Entry criteria.** PH5 exit gate; the Extension sandbox has survived two phases of production use with no escape.

**Duration.** ~14 weeks.

### Milestones

| ID | Milestone | Deliverables |
|---|---|---|
| M6.1 | Listings and discovery | The category taxonomy, listing model, capability manifest shown pre-purchase, curated shelves, transparent sort (`19`) |
| M6.2 | Commerce | The purchase saga, licenses, refunds, escrow for services, disputes (`19 §6`) |
| M6.3 | Revenue share and payouts | The 12% fee with the 70% creator floor, the Sink split, settlement cadence, holdback (`19 §7`) |
| M6.4 | Review pipeline | Automated checks, human review tiers by risk class, the fast path, recall with signed orders and TTL-bounded attestations |
| M6.5 | Ratings and reviews | Purchaser-only, Trust-weighted, creator response, brigading defense |
| M6.6 | Creator portal | Sandbox Societies, privacy-respecting analytics, `fn ext publish`, the capability diff audit |
| M6.7 | Society storefronts | Society-curated shelves with their own cut and standards |
| M6.8 | Federation | Voluntary alliances between Societies: shared discovery, capability sharing, joint treasuries (`11`) |
| M6.9 | Self-hosted Nodes | The same binary as a personal or organizational Node joining the Fractal Net |
| M6.10 | Seasons v1 | The first Season, additive-only, with the graceful-end mechanics |

**Complexity budget.** 1 new service (Review pipeline) · 3 resource families (`listings`, `purchases`, `federations`) · 5 dependencies · 0 platforms · 2 Sources (extension development, curation). At budget.

### Risks

| Risk | Mitigation | Fired when |
|---|---|---|
| A malicious Extension reaches thousands of Societies | Capability manifest verified against code, not trusted; 2-of-N signed recall orders; TTL-bounded install attestations so offline Nodes fail closed | Any Extension exceeds its declared manifest at runtime |
| Marketplace captured by a handful of creators | Earnings-Gini as a tracked health metric; the reduced fee tier goes to small creators, not large ones | Top 1% of creators exceed 60% of earnings |
| Payout obligations without a legal entity or counsel | Fiat is Phase 9; FRC payouts are internal. **Counsel review required before M6.3 ships.** | Counsel has not signed off |
| Federation reintroduces global consistency requirements | Federation is discovery and capability sharing, not shared state; Societies remain atomic (P1) | Any federated operation requires a cross-Society transaction |

### Acceptance criteria

1. A creator publishes, sells, and is paid, end to end, with a correct Posting trail and a receipt.
2. An Extension update requesting more capability cannot reach an installed Society without per-Society re-consent.
3. A recall order disables a malicious Extension on every online Node within 60 seconds and on offline Nodes at next start.
4. Ranking uses no banned signal; the allowed-signal audit is clean and published.
5. Marketplace fees are recorded as `Sink::MarketplaceFee` with the burn/reserve split visible in the supply dashboard.
6. A self-hosted Node joins, syncs a Society, serves as a Custodian, and earns Fraction.

**Exit gate.** All criteria green · counsel sign-off on payouts and terms · 30 days of marketplace operation without a security incident · zero open ADRs.

---

## PH7 — EXPERIENCES

**Goal.** Societies host and govern interactive experiences.

**Entry criteria.** All seven gates in `20`'s Experience Runtime section, including: the Extension sandbox has had zero escapes in two phases; an ADR naming what is cut to pay for this phase; and the resource-accounting model validated in simulation.

**Duration.** ~16 weeks.

**Milestones.** Sandbox hardening beyond the plugin model · authoritative state and tick model · client prediction · session lifecycle · Vault-backed asset delivery · in-Experience economy with hard caps (may charge Fraction and request Facet mints; may never emit Fraction, and may never write XP, Trust, or Standing) · Experience Chambers · the engine plug-in interface · creator tooling · the first three first-party Experiences.

**Complexity budget.** 1 new service (Experience host) · 2 resource families · 5 dependencies · 0 platforms · 0 Sources.

**Principal risk.** This is the largest scope trap in the entire project (`02 §3`). The mitigation is the gate: PH7 does not start until every gate passes, and the phase is cancellable at any Milestone without stranding the rest of the platform.

**Acceptance criteria.** Zero sandbox escapes under an external red-team engagement · an Experience cannot write progression or emit Fraction (property test) · host frame budget unaffected by a running Experience · an Experience's resource cost is metered onto the hosting Society's Treasury with a hard cap that halts rather than overdraws.

---

## PH8 — FN L1

**Goal.** The ledger migrates to a Fractal Node chain without a rewrite, proving P11.

**Entry criteria.** PH6 complete · 12 months of stable internal ledger operation · a specific, articulated reason the internal ledger is insufficient. **Absent that reason, this phase does not start.** Building a chain because it is expected is how projects die.

**Duration.** ~20 weeks.

**Milestones.** Consensus and finality selection with an ADR · the `Ledger` implementation behind the unchanged trait · anchoring migration from internal to chain · the five-stage migration from `16 §7` including the shadow phase, the zero-divergence-for-30-days exit criterion, and the 90-day inverted dual-write rollback window · external `Chain` adapters as read-only projections · a public verification tool.

**Acceptance criteria.** No domain crate, API contract, or client changes · 30 days of shadow operation with zero divergence · a third party can independently verify any anchor · rollback rehearsed successfully on staging.

---

## PH9 — EXCHANGE  *(counsel-gated; may never open)*

**Goal.** Fraction becomes exchangeable outside the platform, if and only if that is legal, safe, and good for the ecosystem.

**Entry criteria.** All seven preconditions in `17 §15`, of which the seventh — jurisdiction-by-jurisdiction legal opinion — is where this blueprint stops. **Nothing in this document is legal or financial advice; PH9 requires qualified counsel and possibly licensing.**

This phase is listed for architectural completeness. It is explicitly acceptable for it never to open. The `Rail` port exists so that the decision remains available, not so that it is inevitable.

---

## 3. The Continuous Tracks

Four tracks run through every phase and are never a phase of their own. Each has a per-phase gate.

| Track | Every phase must |
|---|---|
| **Security** | Threat model updated; dependency audit clean; a security review at the exit gate; an external audit at PH3, PH6, and PH7 |
| **Accessibility** | axe-core clean; manual audit at the exit gate; the a11y debt list at zero |
| **Performance** | Every budget in `32 §8` green in CI; no regression carried across a phase boundary. `32 §8` is the source and `perf/budgets.json` is generated from it (`61 X9`) |
| **Documentation** | Every shipped capability documented in the API reference, the CLI help, and the changelog; documentation drift detection clean; `cargo xtask lint-phases`, `lint-docs` and the link checker green |
| **Safety operations** | A funded human-review function for reports on private surfaces: report-volume projection, review staffing, latency and appeal SLOs, reviewer welfare, and an escalation ladder mirroring `19 §5.5`. At 100,000 active Citizens and a 0.5% monthly report rate that is 500 reports/month requiring a human to view disclosed plaintext, and no chapter sized it. First gate is **PH1 exit**, because PH1 is when external Citizens arrive (`61 W15`) |

---

## 4. Cross-Phase Dependency Graph

```
 Canon ──► PH0 Foundation
            │
            ├──► schema codegen ────────────────────────────► (every phase)
            ├──► token pipeline ────────────────────────────► (every client)
            └──► determinism harness ───────────────────────► (every invariant)
                     │
                     ▼
            PH1 Spine ──┬──► Identity ──────────► PH3 Envelopes ──► PH6 Marketplace
                        ├──► Ledger ───────────► PH4 Economy ────► PH8 FN L1 ──► PH9
                        ├──► Discourse ────────► PH4 Voice
                        ├──► Design System ────► PH2 Desktop ────► PH5 Mobile
                        └──► Progression ──────► PH5 Seasons ────► PH6 Seasons v1
                                 │
                     PH2 Node ───┼──► local-first ──► PH5 Fracture
                                 └──► Vault ───────► PH4 Custodian mesh
                                                          │
                     PH3 Agent ──► Extension host ────────┴──► PH7 Experiences
```

**Critical path:** Canon → PH0 codegen → PH1 design system → PH1 web GUI → PH2 sync engine → PH3 Envelopes → PH4 Custodian mesh → PH5 Fracture. Everything else has slack. Protect these.

---

## 5. What Cancels or Reorders a Phase

Stated in advance so the decision is recognized rather than rationalized:

| Signal | Response |
|---|---|
| PH1 external Citizens do not return after week one | Stop. The spine sentence is wrong, not the implementation. Re-examine before building PH2. |
| Sync engine correctness is not achieved in PH2 | Extend PH2. Do not start PH3 on an unsound foundation. |
| The Envelope security audit fails in PH3 | Halt all agent and extension work until it passes. P8 outranks everything. |
| The economic simulation shows a positive attack margin in PH4 | The Source ships disabled. The phase can still exit. |
| Fracture cannot be made safe in PH5 | Ship PH5 without it and move Fracture to PH6. It is the namesake, not the spine. |
| Marketplace has no supply at PH6 | The problem is creator tooling and distribution, not commerce. Reorder within the phase. |
| PH7 gates do not pass | Do not start it. The platform is complete without Experiences. |
| No articulated need for a chain at PH8 | Do not build one. |
