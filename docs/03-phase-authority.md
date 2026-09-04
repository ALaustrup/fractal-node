# 03 — Phase Authority

> **Status:** Canon. The fourth file of the Canon contract every agent loads before implementation (`00 §6`).
> **Prerequisites:** `00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`.
> **Governs:** the phase in which every shippable capability in the blueprint may be built, the complexity budget each phase actually consumes, and the machine-readable companion `docs/phases.toml` from which every other phase claim is generated.

---

## 0. Precedence

**This file wins.** Where any chapter states, implies, or tabulates a phase for a capability and this file states a different one, this file is correct and the chapter is stale. A phase column, "Phase Placement" section or "Phase" table cell appearing anywhere else in the corpus — `16 §20`, `17 §3.2`, `17 §4`, `19 §17`, `20 §2`, `21 §14`, `30 §4.2`, `41 §5.3`, `41 §5.4`, `41 §5.5`, `50` — is a **non-normative convenience view**. It exists so a reader of that chapter can see the shape without a second lookup. It is generated from `docs/phases.toml`, it is never hand-authored, and `cargo xtask lint-phases` fails the build on any phase claim that does not match.

`02 §6` question 2 — *"which phase does the roadmap place it in?"* — is answered here and nowhere else. Before this file existed there were eight roadmaps; an agent could satisfy its acceptance criteria, cite a chapter, and still breach `02 §5`. That class of misdirection is what this file removes.

`50-roadmap-phases.md` remains the authority on **sequencing rationale, milestones, entry criteria, risks, acceptance criteria and exit gates**. This file is the authority on **which phase a capability belongs to**. Where the two disagreed, `50` was preferred and the other chapter corrected — except in the eleven cases marked **[overrides `50`]** in the Notes column, each of which states why.

```
   00 principles ──┐
   01 terminology ─┼──► THE CANON ──► every agent, before any implementation task
   02 guardrails ──┤
   03 phase authority ─┴──► docs/phases.toml ──► every generated phase column
                                             └──► cargo xtask phase-check (fast lane)
```

---

## 1. Canonical Phase Identifiers

Phase identifiers are `PH0`–`PH9`. **Bare numbers ("Phase 2", "phase 4–5", "2") are forbidden** in every document and every Work Unit; `42 §2.1` already requires `PH<n>` notation and this file makes that requirement enforceable. A range ("4–5") is not a phase and never was — it is an unmade decision wearing a number.

| ID | Name | Goal (verbatim from `50`) | Nominal | Realistic (`60 §3.11`) |
|---|---|---|---|---|
| **PH0** | Foundation | The repository can build, test, and release itself, and one trivial end-to-end path works from browser to database and back. | 3 wk | 3–4 wk |
| **PH1** | The Spine | A Citizen can register, create a Society, talk in it, hold Fraction, and see their progression — from a web GUI, a CLI, and the API, at production quality. | 10 wk | 24–30 wk |
| **PH2** | The Node | The application works offline, runs as a real desktop Node, and holds media. | 10 wk | 20–25 wk |
| **PH3** | The Agent | Agents are real, bounded, auditable participants, and the platform extends itself through the same API third parties will use. | 10 wk | 20–24 wk |
| **PH4** | The Mesh | Storage becomes distributed and compensated, the economy starts emitting, and people can talk and see each other. | 14 wk | 30–40 wk |
| **PH5** | Fracture | The namesake operation works, and the platform is native on every screen. | 16 wk | 30–40 wk |
| **PH6** | The Market | Creators and developers earn a living inside the ecosystem, and Societies can host themselves. | 14 wk | 25–35 wk |
| **PH7** | Experiences | Societies host and govern interactive experiences. | 16 wk | 30–40 wk |
| **PH8** | FN L1 | The ledger migrates to a Fractal Node chain without a rewrite, proving P11. | 20 wk | 35–50 wk |
| **PH9** | Exchange | Fraction becomes exchangeable outside the platform, if and only if that is legal, safe, and good for the ecosystem. | gated | may never open |

The "Realistic" column is `60 §3.11`'s 2.0–2.6× multiple, carried here because a phase table that quotes only the optimistic figure is how a plan becomes a promise. **Neither column is a commitment** (`50 §1`). Sequencing is the commitment; duration is an estimate.

---

## 2. The Master Capability → Phase Table

One row per shippable capability. **Owning chapter §** is where the capability is *specified*; **Phase** is where it may be *built*. A Work Unit whose capability row names a phase other than the current one is out of scope by `02 §6` and is rejected by `cargo xtask phase-check`.

Kind codes in the Notes column: `[crate]` a crate created, `[api]` a public API resource family, `[svc]` a top-level service, `[src]` an economic Source, `[snk]` a Sink, `[plat]` a client platform.

### 2.1 PH0 — Foundation

| Capability | Owning chapter § | Phase | Notes |
|---|---|---|---|
| Monorepo layout, workspace, CODEOWNERS, licence, security policy | `41 §2–§4` | PH0 | M0.1 |
| Toolchain pinning, rustfmt, clippy lint set | `40 §2`, `41 §8.2` | PH0 | M0.2 |
| The four Canon lints: dependency direction, no-literal-hex, banned terminology, `#[secret]` | `40 §4`, `41 §7` | PH0 | M0.2 |
| `cargo xtask phase-check` and `docs/phases.toml` | `03 §4` | PH0 | **New.** The mechanism this file exists to enable. Must land in M0.2 with the other Canon lints |
| CI fast lane <5 min, full lane, merge queue | `40 §12`, `42 §14` | PH0 | M0.3 |
| Tri-target build x86_64 / aarch64 / wasm32 | `41 §8.1` | PH0 | M0.3, N2 |
| `fractal-schema` codegen: OpenAPI, protobuf, JSON Schema, TS client, CLI command tree | `30 §3`, `41 §12` | PH0 | M0.4 |
| `codegen-diff` gate | `40 §5` | PH0 | M0.4 |
| Design token pipeline, five emitters, drift gate | `32 §2`, `41 §10.2` | PH0 | M0.5, N7. Source of record is `packages/tokens/src/` (§3.11 ruling) |
| `fractal-testkit`: `Clock`, `Rng`, `IdGen` fakes | `40 §7.4` | PH0 | M0.6 |
| `fractal-sim` deterministic simulation runner | `40 §7.5`, ADR-0014 | PH0 | M0.6. 6 engineer-weeks to first value; this is the phase's long pole |
| First three `11 §7` invariants as property tests | `11 §7` | PH0 | M0.6 |
| Walking skeleton: `POST /v1/societies` → event → projection → `GET` | `30 §4.3` | PH0 | M0.7 |
| `societies` resource family (skeleton) | `30 §4.2` | PH0 | `[api]` 1 of 3. The only family PH0 claims |
| `AGENTS.md` working agreement | `42 §3` | PH0 | M0.8 |
| `fractal-types`, `fractal-macros`, `fractal-schema`, `fractal-ports` | `41 §5.1–5.2` | PH0 | `[crate]` |
| `fractal-domain-society`, `fractal-domain-identity` | `41 §5.3` | PH0 | `[crate]` |
| `fractal-app-kernel` (command bus, idempotency, saga runner, UoW) | `41 §5.4` | PH0 | `[crate]`. Depends on `fractal-domain-agent`, which is therefore also PH0-scaffolded — see §3.21 |
| `fractal-api-gateway`, `fractal-api-http` | `41 §5.6` | PH0 | `[crate]` |
| `fractal-adapter-postgres`, `-otel`, `-clock` | `41 §5.5` | PH0 | `[crate]` |
| ADRs 0001–0014 accepted | `adr/` | PH0 | Exit gate. `02 §5`: zero open ADRs at a gate |

### 2.2 PH1 — The Spine

| Capability | Owning chapter § | Phase | Notes |
|---|---|---|---|
| FNID derivation, base32 encoding, checksum | `12 §2` | PH1 | M1.1 |
| Handle claim, confusable normalization, 14-day grace window (once) | `12 §2.3`, `11 §6` | PH1 | M1.1. Grace ruled at §3.14 |
| Passkey registration and login, device-code fallback | `12 §5.1` | PH1 | M1.1 |
| Device enrolment and the signed device chain with fork detection | `12 §3` | PH1 | M1.1 |
| Sessions, DPoP, token lifetimes | `12 §5`, `30 §5` | PH1 | M1.1 |
| Social-recovery configuration (guardians, Shamir, delay-and-notify) | `12 §6` | PH1 | M1.1 |
| Second device as the primary recovery story, ahead of guardians | `12 §6.1` | PH1 | **New.** `60 §3.13` remedy 2; recovery-set formation rate becomes a PH1 acceptance criterion |
| Platform guardian of last resort (one sub-threshold Shamir share) | `12 §6.1` | PH1 | **New.** `60 §3.13` remedy 1. Not custody under `02 §4`: a single sub-threshold share under an explicit user-initiated delegation |
| `citizens` resource family | `30 §4.2` | PH1 | `[api]` 1 of 3 |
| Society CRUD; Charter v0 with roles and permissions; Founder governance | `11 §2.2–2.3` | PH1 | M1.2 |
| **First-hearth Society creation at Level 0** | `18 §5.1`, `11 §2.3` | PH1 | **Ruling §3.7.** Exactly one Society per Citizen at L0, free (`17` K1). The second requires L3 and 250 FRC. Without this, `50 PH1` AC-1 and the `02 §2` spine sentence are both unreachable |
| Memberships, join policies, Charter acknowledgement | `11 §2.4` | PH1 | M1.2 |
| Chambers (text), Threads, Messages, reactions, edit/delete semantics | `14 §1`, `11 §2.5` | PH1 | M1.3 |
| Transport encryption, presence, typing | `14 §2` | PH1 | M1.3. Presence is Relay-process memory, not Redis (§3.3) |
| `chambers`/`threads`/`messages` — the Discourse family | `30 §4.2` | PH1 | `[api]` 2 of 3 |
| Signal WebSocket: subscribe, resume, backpressure ladder, replay ring, `Gap` semantics | `14 §2` | PH1 | M1.4. Protocol declared PH0; delivered PH1. **Not a resource family** (§3.1) |
| Internal double-entry ledger behind the `Ledger` trait | `16 §2`, `16 §4` | PH1 | M1.5, N4 |
| `Quanta(i64)` persisted / `i128` intermediate, checked arithmetic | `17 §2.1` | PH1 | **Ruling §3.4.** Unfixable after the first Posting |
| Citizen, Society and Agent Wallets; Transfers; Postings; ordered locking | `16 §4.4`, `11 §2.6` | PH1 | M1.5, N5 |
| `EmissionAccount` sharded into K = 64 sub-accounts | `16 §4.4` | PH1 | **New.** `60 §3.6`. `11 §7.4` becomes `total supply == -Σ EmissionAccount[i].balance` |
| `PostingReason::GenesisAllocation` | `17 §4` | PH1 | **Ruling §3.8.** Bounded, published, non-Source. Retired at the PH4 exit gate. **consumes no `02 §5` Source budget slot** |
| Continuous ledger reconciler with freeze-on-divergence | `16 §4.5` | PH1 | M1.5 |
| `wallets`/`transfers`/`postings` — the Ledger family | `30 §4.2` | PH1 | `[api]` 3 of 3. **PH1 is now exactly at budget** |
| XP, Levels 0–12, the level curve | `18 §3–§4` | PH1 | M1.6. Overrides `41 §5.3`, which places `fractal-domain-progression` at PH2 |
| Trust, Standing, the first Unlock gates, `ContributionReceipt` | `18 §2`, `18 §5` | PH1 | M1.6. `ContributionReceipt` renders XP only until PH5 brings S4 |
| Progression API under `/v1/citizens/{id}/progression` | `30 §4.2` | PH1 | **Not a resource family** — a sub-resource of `citizens` (§3.1) |
| LATTICE design system: tokens, 40 components, nine artifacts each, Storybook, visual regression | `32`, `51 §14.1` | PH1 | M1.7. The critical path; start day one |
| Themes `void` and `contrast` | `32 §9` | PH1 | `daylight` is PH2 (`51` Q5). Names ruled at §3.11 |
| Web GUI: Society shell, Chamber, Wallet, Profile, onboarding, command palette, four data states | `51 §4–§5` | PH1 | M1.8 |
| Web client platform | `34 §4` | PH1 | `[plat]` 1 of 1 |
| No wasm core, no Service Worker, no PWA in PH1 | `51 §2.2` R3/R8 | PH1 | Ruled in `51`; restated here because three chapters imply otherwise |
| CLI v1 with full GUI parity, generated command tree, boot sequence, Society dashboard | `31` | PH1 | M1.9, N3, P13 |
| OpenTelemetry end to end, SLOs, dashboards, alerting, runbook per alert | `40 §9–§10` | PH1 | M1.10 |
| Backups with a restore drill, incident procedure, abuse reporting, ToS/privacy, staged rollout | `40 §11`, `50 M1.11` | PH1 | M1.11 |
| Safety operations model: report volume projection, review staffing, appeal SLO, reviewer welfare | `14 §6`, `50 §3` | PH1 | **New.** `60 §3.15` remedy 2. A PH1 exit criterion, because PH1 is when external Citizens arrive |
| `ShardRouter` in the composition root, one shard configured | `10 §12` | PH1 | **New.** §3.6. The seam must exist and be exercised before it is needed |
| **S15 Atlas**: unified inbox, in-Society search feed, Shard refcount skeleton | `10 §3`, `41 §5.4` | PH1 | **New boundary, §3.9.** `[crate] fractal-app-atlas`. **No new `[api]` family, no new `[svc]`** |
| In-Society scoped search over the Discourse projection | `51` Q8 | PH1 | Global search is PH5 with the Discovery family |
| `fractal-domain-discourse`, `-ledger`, `-progression`, `-governance`, `-agent` (CapabilitySet only) | `41 §5.3` | PH1 | `[crate]`. Overrides `41 §5.3`'s PH2 for progression, governance and agent |
| `fractal-adapter-ledger-internal`, `-chain-null`, `-nats`, `-ws`, `-keystore-os`, `-rail-internal` | `41 §5.5` | PH1 | `[crate]`. Overrides `41 §5.5`, which places `-rail-internal` at PH2; `16 §9` is right |
| `fractal-app-projection` with the replay driver | `41 §5.4` | PH1 | `[crate]` |
| Per-projection checkpoints (generalized from `16 §4.3`) | `16 §4.3` | PH1 | **New.** `60 §3.5`. Cheap now, a 5-to-37-hour outage later |
| Runtime as the first top-level service | `10 §2` | PH1 | `[svc]` 1 of 2 |
| OpenMLS spike at 1,000 leaves on real hardware | ADR-0010 | PH1 | **New.** `60` I16. Moves the `10 §12` fallback decision off PH2's critical path |
| `economy/rates.toml` as the single economic rate table | `17 §3`, `13 §8.2` | PH1 | **New.** §3.20 ruling; generates `13 §8.2` and `18 §5.1`/`§5.2` |

### 2.3 PH2 — The Node

| Capability | Owning chapter § | Phase | Notes |
|---|---|---|---|
| `fractal-core` client core: local SQLite store, replicated log, outbox, `sync_step(budget)` | `34 §2`, `41 §5.7` | PH2 | M2.1 |
| `fractal-core-wasm` and the OPFS-backed browser replica | `34 §4.2` | PH2 | M2.1. `34 §4.2` is corrected from PH1 (`51` R3) |
| Conflict policy per data class; CRDTs for reactions, read state, presence | `10 §6` | PH2 | M2.2 |
| Offline UX: staleness indicators, queued-write rendering, reconciliation surfacing | `32 §6` | PH2 | M2.3 |
| Desktop Node (Windows): Tauri v2 shell, installer, signed differential updates, tray, deep links | `34 §5` | PH2 | M2.4 |
| Desktop client platform | `34 §5` | PH2 | `[plat]` 1 of 1 |
| Six breakpoints, density modes, the 88ch measure invariant, gamepad focus navigation | `32 §4.2` | PH2 | M2.5. `60 §3.11`'s designated PH1 cut candidate lands here |
| Vault v1: Objects, Versions, ACLs, upload/download, quotas | `13 §3`, `13 §10.1` | PH2 | M2.6 |
| **Citizen Vault** (`Vault.society: Option<SocietyId>`) | `11 §2.7`, `21 §17` | PH2 | **Ruling §3.10.** The Canon amendment lands in PH1; the first Citizen Vault Object is written in PH2 |
| `BlobStore` on S3-compatible storage | `13 §11.3` | PH2 | M2.6 |
| `vault/objects`/`vault/manifests` — the Vault family | `30 §4.2` | PH2 | `[api]` 1 of 3. Overrides `30 §4.2`, which places it at PH1 |
| `/v1/subscriptions` — durable Signal subscriptions | `30 §4.2` | PH2 | `[api]` 2 of 3 |
| Media pipeline: transcoding ladder, AV1/AVIF/Opus with fallbacks, thumbnails, blurhash | `13 §9` | PH2 | M2.7 |
| Transcoder service | `10 §2` ① | PH2 | `[svc]` 1 of 2 |
| `fractal-adapter-ffmpeg` | `41 §5.5` | PH2 | `[crate]`. Overrides `41 §5.5`'s PH4; `50 M2.7` calls this the first extraction |
| Adaptive playback with verified streaming | `13 §5` | PH2 | M2.7 |
| Gallery Chambers, the Module grid, the customization contract, save-time a11y validation | `21 §3`, `21 §5` | PH2 | M2.8 |
| Personal Collections, viewer, EXIF handling, offline pin | `21 §5` | PH2 | M2.8 |
| PWA: installable, offline-capable, push, share target | `34 §4.3` | PH2 | M2.9. `34 §4.3`'s "Phase 3" is corrected (`51` Q4) |
| E2EE for DMs and Private Chambers via MLS | `14 §4` | PH2 | M2.10 |
| MLS ceiling in PH2: **500 leaves (≈200 Citizens at 2.5 devices/Citizen)** | `14 §4.4` | PH2 | **Ruling §3.12.** Every MLS figure is now stated in leaves |
| `HistoryKey` wrapped separately from the identity recovery key, off by default | `12 §6.3`, `14 §4.5` | PH2 | **New.** `60 §3.7`. Adds I-12.13 |
| `fractal-adapter-mls`, `-model-http` | `41 §5.5` | PH2 | `[crate]` |
| `fractal-domain-vault`, `-economy` | `41 §5.3` | PH2 | `[crate]`. `-economy` holds settlement arithmetic; no Source emits until PH4 |
| Two-primary composition-root proof (the `EventStore` shard rehearsal) | `10 §12` | PH2 | **New.** `60` I14. §3.6 |
| Projection rebuild SLO: any projection, any Society, ≤ 15 min at p99 | `40 §9.4` | PH2 | **New.** `60 §3.5`. A PH2 gate criterion with a large-Society fixture |
| Internal anchoring and the standalone verifier | `16 §6` | PH2 | Matches `16 §20`. `50 PH2` gains a milestone for it (M2.11) |
| `daylight` theme | `32 §9` | PH2 | Deferred from PH1 per `51` Q5 |

### 2.4 PH3 — The Agent

| Capability | Owning chapter § | Phase | Notes |
|---|---|---|---|
| Capability grammar, `CapabilitySet` algebra, attenuation-only grants, TTLs | `12 §7`, `15 §4` | PH3 | M3.1. The lattice itself lands PH1 for Charter roles |
| Envelope revocation with in-flight semantics | `12 §7.3` | PH3 | M3.1 |
| Policy Enforcement Point in the application layer, `envelope_ref` on every event | `10 §8`, `15 §5` | PH3 | M3.2. **Server-only** — never in wasm (§3.2) |
| `AgentActionBlocked` as a domain event | `10 §8` | PH3 | M3.2 |
| Agent identity, Operator accountability, execution loop, sandboxing, metering, checkpointing | `15 §2–§3` | PH3 | M3.3 |
| `ModelProvider` port, hosted and local adapters, per-Society model choice, `ContextManifest` | `15 §9` | PH3 | M3.4 |
| Offline Agent semantics: inference refused without a local provider; authorization always deferred | `15 §6`, `34 §12.1` | PH3 | **Ruling §3.13.** Both statements are true and differently scoped |
| Workflows: declarative graph, triggers, steps, conditions, compensation, versioning | `15 §7–§8` | PH3 | M3.5 |
| Three first-party Workflows | `15 §8` | PH3 | M3.5 |
| Extension host: WASM Component Model, WIT world, manifest, install/consent, resource limits | `20 §4` | PH3 | M3.6 |
| Extension kinds `plugin`, `theme`, `template`, `sdk`, `workflow` | `20 §2` | PH3 | **Ruling §3.15.** `20 §2` places `workflow` at PH4; `50 M3.5` ships the graph in PH3 |
| Extension kind `automation-pack` | `20 §2` | PH4 | Bundles Policy proposals; needs Governance v1 settled for a phase |
| Ten first-party Extensions on the public hook surface | `20 §2` | PH3 | M3.7, P7 |
| Extension-supplied tool definitions treated as untrusted content, host-namespaced `ext.<install_id>.<name>` | `15 §13.1`, `20 §6` | PH3 | **New.** `60 §3.8`. Adds A11 |
| Per-Install fee-revenue ceiling and host-drawn fee disclosure for hook 18 | `20 §11` | PH3 | **New.** `60 §3.8` |
| Governance v1: Charter amendment, Council governance, roles beyond Founder, moderation actions, appeals | `11 §2.3`, `19 §5.5` | PH3 | M3.8. Overrides `41 §5.3`, which places `fractal-domain-governance` at PH2 |
| Agent surfaces: `EnvelopeCard`, `PolicyEditor`, audit trail, Operator and Society kill switches | `15 §11` | PH3 | M3.9 |
| `agents`/`envelopes`/`policies`/`workflows`/`runs` — the Agent family | `30 §4.2` | PH3 | `[api]` 1 of 3. Overrides `30 §4.2`'s PH1 |
| `extensions`/`installs` — the Extension family | `30 §4.2` | PH3 | `[api]` 2 of 3 |
| `charter/versions`/`proposals`/`votes`/`moderation` — the Governance family | `30 §4.2` | PH3 | `[api]` 3 of 3. **PH3 is exactly at budget; Asset moves to PH4** |
| Agent Executor service | `10 §2` ③ | PH3 | `[svc]` 1 of 2 |
| `fractal-domain-extension`, `fractal-adapter-wasmtime` | `41 §5.5` | PH3 | `[crate]` |
| Sandbox port ADR | `20 §4`, `41 §5.5` | PH3 | Required before the crate leaves `unstable-sandbox` |
| External security audit of the Envelope and Extension sandbox | `50 PH3` | PH3 | Exit gate. Non-negotiable |

### 2.5 PH4 — The Mesh

| Capability | Owning chapter § | Phase | Notes |
|---|---|---|---|
| Custodian registration, capacity commitment, graceful exit | `13 §7.4` | PH4 | M4.1 |
| **Custodian eligibility decoupled from Level**: Trust ≥ 100 ∧ proof-of-capacity ∧ bond ∧ 30-day probation | `13 §7.4`, `18 §5.5` | PH4 | **Ruling §3.16.** `18 §5.5`'s `Level ≥ 8 ∧ Stake(500 FRC)` is deleted |
| Bond scaled by committed bytes (`bond_rate × committed_bytes`, floor 500 FRC) | `13 §7.4` | PH4 | **Ruling §3.16.** Resolves X13 |
| Shard assignment by XOR distance; rendezvous hashing over a bucketed index | `13 §7.2` | PH4 | `60` I19: two days now, a rewrite at 10 k Custodians |
| Attestation: challenge scheme, proof verification, failure handling, slashing | `13 §7.5–§7.6` | PH4 | M4.2 |
| RS(10,16) erasure coding, placement across failure domains, repair triggers | `13 §6` | PH4 | M4.3. **1.60× is the replication factor in every model** (§3.6) |
| `BlobStore` swap to the Custodian mesh, dual-write, 30-day verification, cutover with rollback | `13 §11.4` | PH4 | M4.4 |
| Measured PH4 entry precondition: committed capacity ≥ 3× current logical storage across ≥ 8 failure domains | `13 §11.4` | PH4 | **New.** `60 §3.14`. Gates step 4.1 |
| **S1 — Storage Custody** | `17 §3.2` | PH4 | `[src]` 1 of 2. 0.28 FRC/replica-GB-month, sink-first |
| **S2 — Bandwidth Service** | `17 §3.2` | PH4 | `[src]` 2 of 2. **PH4 is exactly at the Source budget** |
| Settlement windows offset by `BLAKE3(society_id) mod 86400` | `13 §8.2` | PH4 | **New.** `60 §3.6`. Removes the 00:00 UTC spike |
| Emission ledger published; public supply dashboard | `17 §5`, `17 §11` | PH4 | M4.6 |
| `GenesisAllocation` retired | `17 §4` | PH4 | **Ruling §3.8.** Stops at the PH4 exit gate; the enum variant survives forever |
| Sinks K8 (storage tariff), K9 (bandwidth tariff), K13 (stake slash), K16 (vesting forfeiture) | `17 §4` | PH4 | `[snk]` — Sinks consume no `02 §5` budget line |
| `sources`/`settlements`/`contribution` — the Economy family | `30 §4.2` | PH4 | `[api]` 1 of 3 |
| `facets`/`facet-standards` — the Asset family | `30 §4.2` | PH4 | `[api]` 2 of 3. **Moved from PH3** (§3.17); `50` gains milestone M4.11 |
| FN-ASSET/1 core: mint, state, provenance, transfer; Deterministic and Owner evolution triggers | `16 §11` | PH4 | M4.11. Overrides `16 §20`'s PH3 — `50` has no PH3 asset milestone |
| Insignia and Badges rendered as Facets, minted retroactively from the progression log | `18 §5.4` | PH4 | The Facet evolution model carries the tier provenance; PH1–PH3 Achievements are progression records |
| Voice: WebRTC + SFU, Voice Chambers, jitter/echo handling, SFrame E2EE media | `14 §7` | PH4 | M4.7 |
| Video and Stage: simulcast/SVC, Stage Chamber, recording with consent | `14 §7` | PH4 | M4.8 |
| SFU / extracted Relay service | `10 §2` ② | PH4 | `[svc]` 1 of 2 |
| Custodian Coordinator service | `10 §2` ⑤ | PH4 | `[svc]` 2 of 2. **PH4 is exactly at the service budget** |
| Convergence, Serendipity, eligibility, Crystallization with full history preservation | `11 §2.10`, `11 §3.1` | PH4 | M4.9. `/v1/convergences` is in the Discourse family |
| MLS ceiling raised to **1,000 leaves (≈400 Citizens)** | `14 §4.4` | PH4 | Ruling §3.12 |
| `fractal-adapter-tantivy`, `fractal-api-grpc` | `41 §5.5–5.6` | PH4 | `[crate]` |
| Locally-repairable codes ADR, if repair egress exceeds 40% of mesh egress | `13 §15` | PH4 | `60` I30's measured trigger |

### 2.6 PH5 — Fracture

| Capability | Owning chapter § | Phase | Notes |
|---|---|---|---|
| Governance v2: proposals, voting, quorum/threshold, delegation, `fracture_rules` | `11 §2.3` | PH5 | M5.1 |
| Fracture dry run: full simulation, diff report, invariant checking, `dry_run_token` | `11 §3.2` | PH5 | M5.2. Mandatory |
| Fracture execution: log sealing, child genesis, treasury division, Vault re-referencing, resumability | `11 §3.2` | PH5 | M5.3 |
| **Content-key rotation on Fracture** (`VaultKeyRotatedOnFracture`) | `11 §3.2`, `13 §10.1` | PH5 | **New, ruling §3.5.** Sixth Fracture invariant (`11 §7.16`); tested in `50 PH5` AC-1 |
| Fork and Dissolution | `11 §3.3` | PH5 | M5.4 |
| Lineage: ancestry graph, cross-generation history readability | `11 §2.2` | PH5 | M5.5 |
| Android native shell over UniFFI | `34 §7` | PH5 | M5.6 |
| iOS native shell | `34 §7` | PH5 | M5.7 |
| Mobile client platform (one decision executed twice) | `34 §7` | PH5 | `[plat]` 1 of 1 |
| macOS and Linux desktop builds | `34 §5` | PH5 | M5.8. Build targets, not platforms |
| Advanced discovery: interest graph, matching on declared signals, Society discovery | `14 §8` | PH5 | M5.9 |
| `search`/`discovery`/`interests` — the Discovery family | `30 §4.2` | PH5 | `[api]` 1 of 3. Overrides `30 §4.2`'s PH3 (`51` Q8) |
| `fractal-domain-discovery` | `41 §5.3` | PH5 | `[crate]`. Overrides `41 §5.3`'s PH2 |
| Cross-Society search served by S15 Atlas | `10 §3` | PH5 | §3.9. Index-only, never authoritative |
| Atlas extracted as a service | `10 §2` ⑥ | PH5 | `[svc]` 1 of 2. PH4 has no free service slot |
| Seasons infrastructure: additive-only content, objectives, themed Facets | `18 §8` | PH5 | M5.10. `50 PH5`'s designated cut candidate |
| **S4 — Content Creation** | `17 §3.2` | PH5 | `[src]` 1 of 2. Overrides `17 §3.2`'s PH2 — `50 PH2` budgets zero Sources |
| **S6 — Moderation Work** | `17 §3.2` | PH5 | `[src]` 2 of 2. Needs PH3's moderation and appeal pipeline plus two phases of Standing data |
| Sinks K15 (dormant Treasury reclamation) | `17 §4` | PH5 | `[snk]` |
| Global Registry hash-partitioned by FNID prefix | `60 §4` row 1 | PH5 | It has no ordering requirement, so unlike the log it is genuinely partitionable |

### 2.7 PH6 — The Market

| Capability | Owning chapter § | Phase | Notes |
|---|---|---|---|
| Category taxonomy, listing model, capability manifest shown pre-purchase, curated shelves | `19 §2–§3` | PH6 | M6.1. Overrides `19 §17`'s "4–5" (§3.18) |
| Transparent sort and the allowed-signal audit | `19 §8` | PH6 | M6.1 |
| Free third-party Extensions | `19 §17`, `02 §3` | PH6 | Overrides `19 §17`. `02 §3` requires two phases of adversarial exposure after the PH3 audit |
| Purchase saga, licences, refunds, escrow for services, disputes | `19 §5` | PH6 | M6.2 |
| **Revenue share: 12% platform fee, 70% creator floor** | `19 §6.1` | PH6 | **Ruling §3.19.** `17` K7's 5% is regenerated from `19 §6.1` |
| Payouts, settlement cadence, holdback | `19 §7` | PH6 | M6.3. Counsel review required before ship |
| Review pipeline R0–R3, human review tiers, fast path, recall with signed orders and TTL attestations | `19 §10` | PH6 | M6.4 |
| Review pipeline service | `10 §2` | PH6 | `[svc]` 1 of 2 |
| Ratings and reviews: purchaser-only, Trust-weighted, creator response, brigading defence | `19 §9` | PH6 | M6.5 |
| Creator portal: sandbox Societies, privacy-respecting analytics, `fn ext publish`, capability diff audit | `19 §11` | PH6 | M6.6 |
| Society storefronts and Shelf fees | `19 §12` | PH6 | M6.7 |
| Federation: shared discovery, capability sharing, joint treasuries | `11 §2`, `50 M6.8` | PH6 | M6.8. Society family; no new API family |
| Self-hosted Nodes joining the Fractal Net | `10 §9` | PH6 | M6.9 |
| Seasons v1 with graceful-end mechanics | `18 §8.4` | PH6 | M6.10 |
| `listings`/`purchases`/`payouts` — the Market family | `30 §4.2` | PH6 | `[api]` 1 of 3 |
| `fractal-domain-market` | `41 §5.3` | PH6 | `[crate]` |
| **S5 — Curation** | `17 §3.2` | PH6 | `[src]` 1 of 2. Matches `50 PH6`'s "curation" |
| **S9 — Extension Development** | `17 §3.2` | PH6 | `[src]` 2 of 2. Matches `50 PH6`'s "extension development" |
| Sinks K7 (marketplace fee), K17 (listing bond) | `17 §4`, `19 §6.2` | PH6 | `[snk]` |
| Attested (oracle) Facet evolution, composition, licensing, royalties, escrow | `16 §20` | PH6 | Needs a Trust baseline for attesters |
| Extension UI escape hatch decision (host-owned input-mediated canvas, or rewrite `19 §1`/`19 §14`) | `20 §7` | PH6 | **Open decision D-5.** Decide before the gate, not at it |

### 2.8 PH7 — Experiences

| Capability | Owning chapter § | Phase | Notes |
|---|---|---|---|
| Experience sandbox hardening beyond the plugin model | `20 §12` | PH7 | Gated on zero sandbox escapes in two phases |
| Authoritative state and tick model, client prediction, session lifecycle | `20 §12` | PH7 | |
| Vault-backed asset delivery for Experiences | `20 §12` | PH7 | |
| In-Experience economy with hard caps (may charge FRC; may never emit FRC, XP, Trust or Standing) | `20 §12` | PH7 | Property-tested |
| Experience Chambers, engine plug-in interface, creator tooling | `20 §12` | PH7 | |
| Two first-party Experiences | `50 PH7` | PH7 | **Reduced from three** — the `02 §8` trade that pays for the two Sources below |
| Extension kind `experience` | `20 §2` | PH7 | |
| Experience host service | `10 §2` | PH7 | `[svc]` 1 of 2 |
| **S3 — Compute Contribution** | `17 §3.2` | PH7 | `[src]` 1 of 2. **[overrides `50`]** — `50 PH7` budgets zero Sources. Open decision D-3 |
| **S8 — Agent Development** | `17 §3.2` | PH7 | `[src]` 2 of 2. **[overrides `50`]**. Needs two phases of PH3 agent telemetry |

### 2.9 PH8 — FN L1, and PH9 — Exchange

| Capability | Owning chapter § | Phase | Notes |
|---|---|---|---|
| Consensus and finality selection with an ADR | `16 §7` | PH8 | Entry criterion: a specific articulated reason the internal ledger is insufficient |
| `Ledger` implementation behind the unchanged trait | `16 §7` | PH8 | The P11 proof |
| Anchoring migration from internal to chain; the five-stage migration with shadow phase | `16 §7` | PH8 | 30 days zero divergence; 90-day inverted dual-write rollback |
| External `Chain` adapters as read-only projections; public verification tool | `16 §8` | PH8 | |
| Bridges to external chains | `16 §8` | PH8 | ADR required; `02 §3` |
| **S7 — Governance Participation** | `17 §3.2` | PH8 | `[src]` 1 of 2. Open decision D-3 |
| **S10 — Onboarding & Vouching** | `17 §3.2` | PH8 | `[src]` 2 of 2. The highest-Sybil-risk Source; last for that reason. Open decision D-4 |
| Fiat `Rail`, external FRC exchangeability, off-ramp for payouts | `17 §14` | PH9 | Counsel-gated. May never open |

---

## 3. The Rulings This File Carries

Each ruling below settled a cross-chapter contradiction. The one-line statement is here so an agent reading only the Canon has the answer; the conflict, the rationale tied to a principle number, and the list of chapters that must change are in `61-reconciliation.md` under the matching ID.

| # | Ruling | `61` ID |
|---|---|---|
| **3.1** | A **resource family** is `30 §4.2`'s definition — a maximal set of resources sharing an owning `10 §3` boundary, versioned together, under one capability domain. Progression is a sub-resource of `citizens`; Signals is a protocol, not a family. Under this definition PH0 claims 1, PH1 claims 3, PH3 claims 3, and Asset moves to PH4. | N1 |
| **3.2** | The **PEP is server-only**. `fractal-app-*` does not compile to `wasm32` (`41 §8.1` is right; `34 §2.1`'s diagram is wrong). The wasm core is `fractal-types`, `-macros`, `-schema`, `-ports` (traits only), `fractal-domain-*`, `fractal-core` (`wasm` feature), `fractal-core-wasm`, `fractal-sync`, `fractal-store`, `fractal-crypto`, and the generated tokens and client. A client-side capability check is an **advisory affordance hint** and is re-decided server-side on every command. | X2 |
| **3.3** | **Redis is removed.** Presence lives in Relay-process memory at a 45 s TTL, gossiped between Relay instances over the NATS subject `presence.<society_id>`. No `EphemeralStore` port, no fourth stateful system. | X3 |
| **3.4** | `Quanta` is **`i64` persisted and on the wire, `i128` intermediate**, all arithmetic checked. Unsigned cannot represent the mandatory negative `EmissionAccount` balance, which alone is dispositive. | X4 |
| **3.5** | **Fracture rotates content keys.** For every Vault path assigned to exactly one child, the content key is rotated and re-wrapped to that child's key holders only, emitting `VaultKeyRotatedOnFracture`. Cost is proportional to key holders, not bytes; `13 V6`'s "zero bytes moved" is preserved. Becomes `11 §7` invariant 16. | W10 |
| **3.6** | The remedy for the Postgres write ceiling is **sharding across primaries**, not partitioning within one. Partitioning buys latency and vacuum cost; it adds no write throughput. The four-step ladder and its measurable triggers are in `10 §12`; the `ShardRouter` seam exists in the composition root from PH1 with one shard configured. | W4 |
| **3.7** | **First hearth.** Every Citizen may found exactly one Society at Level 0, free. The allowance is consumed at `SocietyCreated` and is **not** restored if that Society is dissolved, archived, or departed. A Crystallization does not consume it. The second and subsequent Societies require `Level ≥ 3` and 250 FRC (`17` K1). | X7 |
| **3.8** | **`PostingReason::GenesisAllocation`** — 100 FRC to each new Citizen's global Wallet (held in `wallet.locked` until Level 1) and 250 FRC to each new Society Treasury, once, idempotent on the principal, aggregate hard cap 50,000,000 FRC, posted from the `EmissionAccount` so `11 §7.4` holds unchanged, drawn against `B(1)`, published as its own line in the supply dashboard, retired at the PH4 exit gate. It is an allocation, not an emission mechanism, and **consumes no `02 §5` Source slot**. | X-GA |
| **3.9** | **S15 — Atlas** owns every cross-Society read model: the Citizen unified inbox, cross-Society search, marketplace statistics, and the Shard reference count. Eventually consistent, monotonic per reader, staleness bound ≤ 5 s p99 / ≤ 30 s p99.9, **read-only and never authoritative**. The refcount is **monotone-safe**: over-count permitted, under-count forbidden. | W9 |
| **3.10** | **Citizen Vault lands.** `Vault` and `Object` gain `society: Option<SocietyId>` mirroring `Wallet`; `None` denotes the Citizen Vault, which hangs off Global Registry entry 1. `11 §7` invariant 1 becomes a three-clause test. No Global Registry entry is added. | X11 |
| **3.11** | Theme **names** are `32 §9`'s (`void`, `daylight`, `contrast`); the token **file layout** is `41 §10.2`'s (`packages/tokens/src/`). `32 §2`'s bare `tokens/` is a sketch and is corrected. PH1 ships `void` and `contrast`. | X-TH |
| **3.12** | **Every MLS figure is stated in leaves**, at a published 2.5 devices/Citizen. Mechanism ceiling 1,000 leaves. PH2: 500 leaves (≈200 Citizens). PH4 onward: 1,000 leaves (≈400 Citizens). Above the ceiling E2EE is refused, never downgraded. | X10 |
| **3.13** | **Offline Agent invocation.** Inference is refused without a local `ModelProvider`. Action authorization is *always* deferred to reconnect and never granted offline; an outboxed Agent command whose Envelope has expired or been revoked by arrival **fails at the PEP** and is never authorized retroactively against the grant that existed at enqueue. | X12 |
| **3.14** | **Grace windows: 14 days, once, for both Handle and Society name.** | X-GW |
| **3.15** | Extension kind `workflow` is **PH3** (`50 M3.5` ships the graph); `automation-pack` is **PH4** (it bundles Policy proposals and needs Governance v1 settled for a phase). | N9 |
| **3.16** | **Custodian eligibility is decoupled from Level**: `Trust ≥ 100 ∧ proof-of-capacity ∧ Stake(bond_rate × committed_bytes, floor 500 FRC) ∧ 30-day probation at reduced assignment`. `18 §5.5`'s `Level ≥ 8` and flat `Stake(500 FRC)` are deleted. | X13 |
| **3.17** | The **Asset family and FN-ASSET/1 core move to PH4** as milestone M4.11. `50` has no asset milestone in PH3, and PH3's family budget is exactly consumed by Agent, Extension and Governance. PH1–PH3 Achievements and Insignia are progression records; their Facet representation is minted retroactively at PH4 with full tier provenance from the progression log. | N4 |
| **3.18** | Listings, the review pipeline, recall, ratings and free third-party Extensions are **PH6**, not `19 §17`'s "4–5". `02 §3` requires two phases of adversarial exposure after the PH3 external audit before untrusted third-party code runs. | N8 |
| **3.19** | The **marketplace fee is 12%** (`19 §6.1`): 10% for C9 Services, 4% launch rate, 70% creator floor, split 6 pp burn / 4 pp Operations / 2 pp Assurance. `17` K7 is regenerated from `19 §6.1`. A detected self-purchase is **void**, not 100%-fee'd. | X5 |
| **3.20** | **Storage economics.** The anchor (1 FRC ≡ 1 GB-month) is administered and stands; the Custodian payout price is derived; the replication factor in every model is **1.60×** (RS(10,16)); σ becomes a **floor** of 1.25 rather than a target; `18 §5.2`'s SL4 "unmetered Vault" is deleted and the Society quota ladder is re-scaled. Corrected arithmetic in `61 §X6`. | X6 |
| **3.21** | `fractal-domain-agent` is created in **PH0** holding only the `CapabilitySet` lattice — `fractal-app-kernel` (PH0) and the Charter's `capabilities: BTreeMap<RoleId, CapabilitySet>` (PH1) both need it. Agent, Envelope, Policy and Workflow types land in PH3. `41 §5.3`'s single PH2 cell cannot express this and is replaced with two rows. | N13 |

---

## 4. The Complexity-Budget Ledger

`02 §5` sets six hard budgets per phase. This section states, for each phase, what the budget allows and what the capabilities assigned above actually consume. **A phase over budget is a phase failure, not a stretch goal** (`02 §5`), so an overrun here is a scope decision that must be made now rather than discovered at a gate.

### 4.1 Budget definitions, as amended

Two of `02 §5`'s six lines were not well-defined for this corpus and are amended here (both amendments are `61` rulings and require the ADR named in §6):

- **Resource families** count under `30 §4.2`'s definition (§3.1). `50`'s per-phase budget lines counted under a looser one and are corrected.
- **Third-party runtime dependencies** are **5 per phase *per deployable artifact*** — the Runtime binary, the web client, the CLI binary, the desktop shell, each mobile shell. `02 §5`'s rationale is supply-chain surface and maintenance horizon; a React dependency is not in the Runtime's supply chain and cannot sensibly compete with `webauthn-rs` for the same slot. As written, `51 §2.3` claims all five PH1 slots for `apps/web` alone, which leaves the Runtime zero for a phase that must ship passkeys and a WebSocket surface. See `61 §N2`.

### 4.2 The ledger

```
  LEGEND   ● at budget    ○ under budget    ▲ OVER BUDGET — a cut is required
```

| Phase | Services (≤2) | API families (≤3) | Deps (≤5/artifact) | Platforms (≤1) | Sources (≤2) | Open ADRs (0) |
|---|---|---|---|---|---|---|
| **PH0** | ○ 0 — Runtime is PH1 | ○ 1 — `societies` | ○ Runtime 5 (`tokio`, `axum`, `sqlx`, `serde`, `opentelemetry`) | ○ 0 | ○ 0 | ● 0 — ADRs 0001–0014 accepted |
| **PH1** | ● 1 — Runtime | ● 3 — `citizens`, Discourse, Ledger | ● Runtime 5 (`webauthn-rs`, `tokio-tungstenite`, `blake3`, `ulid`, `ed25519-dalek`) · web 5 (`51 §2.3`) · CLI 3 (`clap`, `ratatui`, `crossterm`) | ● 1 — web | ○ 0 | ● 0 |
| **PH2** | ● 1 — Transcoder | ○ 2 — Vault, `subscriptions` | ● Runtime 4 (`openmls`, `reed-solomon-erasure`, `image`, `reqwest`) · desktop 2 (`tauri`, `tauri-plugin-updater`) · web 2 (`sqlite-wasm`, `comlink`) | ● 1 — desktop | ○ 0 | ● 0 |
| **PH3** | ● 1 — Agent Executor | ● 3 — Agent, Extension, Governance | ● Runtime 3 (`wasmtime`, `wit-bindgen`, `cap-std`) | ○ 0 | ○ 0 | ● 0 — sandbox-port ADR must land |
| **PH4** | ● **2** — SFU/Relay, Custodian Coordinator | ○ 2 — Economy, Asset | ● Runtime 5 (`webrtc-rs`, `str0m` or SFU client, `tantivy`, `tonic`, `prost`) | ○ 0 | ● **2** — S1, S2 | ● 0 |
| **PH5** | ○ 1 — Atlas extraction | ○ 1 — Discovery | ● mobile 5 (`uniffi` + per-platform) | ● 1 — mobile | ● **2** — S4, S6 | ● 0 |
| **PH6** | ○ 1 — Review pipeline | ○ 1 — Market | ○ Runtime 2 | ○ 0 | ● **2** — S5, S9 | ● 0 |
| **PH7** | ○ 1 — Experience host | ○ 1 — Experience | ○ Runtime 2 | ○ 0 | ▲ **2 against a stated 0** — S3, S8 | ● 0 |
| **PH8** | ○ 0 | ○ 0 | ○ | ○ 0 | ● **2** — S7, S10 | ● 0 |
| **PH9** | ○ 0 | ○ 0 — `Rail` is an adapter behind an existing port, not a family | ○ | ○ 0 | ○ 0 | ● 0 |

### 4.3 The three phases that had to be rebalanced

**PH1 was over on resource families and is now exactly at budget.** Under `30 §4.2`'s definition PH1 would have delivered six families — Identity, Society, Discourse, Ledger, Progression, Signals — against three. Three moves closed it: Society was claimed in PH0 by the walking skeleton; Progression is a sub-resource of `citizens` (`/v1/citizens/{id}/progression`, which is how `30 §4.2` already paths it); and Signals is a protocol whose one REST resource, `/v1/subscriptions`, moves to PH2 with the Relay's durable subscription store. **S15 Atlas costs PH1 nothing** — its inbox is served under `/v1/citizens/me/inbox` (Identity family) and its search under Discourse (`51` Q8), and it does not become a service until PH5. This is a better trade than `60 §3.9` proposed, which would have spent a PH1 family slot on Atlas and pushed Discovery to PH4; Discovery is at PH5 for a better reason (`51` Q8) and no slot is spent.

**PH3 was over on resource families and is now exactly at budget.** Agent, Extension, Governance and Asset is four. Asset moved to PH4, where two family slots were free and where `50` gains milestone M4.11. `16 §20`'s PH3 placement was never supported by a `50` milestone.

**PH7 is over budget on Sources, deliberately, and the trade is stated.** `50 PH7` budgets zero Sources. Ten Sources exist in `17 §3.2`; two per phase from PH4 fills PH4, PH5 and PH6 and leaves four. Placing S3 and S8 at PH7 requires something to leave PH7 (`02 §8`), and the designated cut is **the third first-party Experience** — `50 PH7` ships two. If Andrew declines that trade, S3 and S8 move to PH8 alongside S7 and S10, which breaches PH8's Source budget instead and leaves the compute auction (K10, PH5) without its matching Source for three phases. **This is open decision D-3.**

### 4.4 Sources: the full ladder and why it is over-subscribed

| Phase | Sources activated | Cumulative |
|---|---|---|
| PH0–PH3 | **none** | 0 |
| PH4 | S1 Storage Custody, S2 Bandwidth Service | 2 |
| PH5 | S4 Content Creation, S6 Moderation Work | 4 |
| PH6 | S5 Curation, S9 Extension Development | 6 |
| PH7 | S3 Compute Contribution, S8 Agent Development | 8 |
| PH8 | S7 Governance Participation, S10 Onboarding & Vouching | 10 |

`17 §3.2` places S4 and S6 at "Phase 2" and S5 and S7 at "Phase 3". Both are wrong, and `50` is right: a Source cannot ship before the emission ceiling machinery, the settlement saga, the public supply dashboard and the economic simulation gate, all of which are PH4. **No Fraction is emitted by any Source before PH4.** Until then the only Fraction in existence is `GenesisAllocation` (§3.8), which is bounded, capped, published and retired.

The honest consequence: **S10 Onboarding & Vouching, the corpus's only designed growth mechanism (`60 A5`), does not activate until PH8.** That is a real cost of holding `02 §5`'s line, and it is open decision D-4.

---

## 5. `docs/phases.toml` — the machine-readable companion

### 5.1 Purpose

`docs/phases.toml` carries the same mapping as §2 in a form `cargo xtask` can assert against. It exists so that no phase claim anywhere in the corpus or in a Work Unit is hand-authored. `cargo xtask lint-phases` regenerates every phase column in every chapter and fails on a diff; `cargo xtask phase-check` fails a Work Unit that targets an out-of-phase capability.

### 5.2 Schema

```toml
schema_version = 1                  # integer; bump requires an ADR
authority      = "docs/03-phase-authority.md"
current_phase  = "PH1"              # the only phase in which work may begin (02 §6)

# ── one [[phase]] per PH0..PH9, in order ──────────────────────────────────────
[[phase]]
id        = "PH1"                   # /^PH[0-9]$/, unique, ordered
ordinal   = 1                       # 0..9, equals the numeric part of id
name      = "The Spine"
goal      = "…"                     # verbatim from 50
nominal_weeks  = 10
realistic_weeks = [24, 30]          # 60 §3.11 range; informational, never a gate
cut_candidate  = "density modes and half the component inventory"

  [phase.budget]                    # 02 §5, as amended by 03 §4.1
  services       = 2
  api_families   = 3
  dependencies   = 5                # PER DEPLOYABLE ARTIFACT
  platforms      = 1
  sources        = 2
  open_adrs      = 0

# ── one [[capability]] per row of 03 §2 ───────────────────────────────────────
[[capability]]
id        = "ledger.quanta"         # /^[a-z0-9]+(\.[a-z0-9_]+)+$/, globally unique, stable forever
name      = "Quanta(i64) persisted / i128 intermediate, checked arithmetic"
phase     = "PH1"                   # MUST match a [[phase]].id
owner     = "17 §2.1"               # chapter and section that specifies it
milestone = "M1.5"                  # optional; a 50 milestone id, or omitted
kind      = "feature"               # feature | crate | api_family | service | source | sink | platform | adr | invariant
consumes  = { }                     # optional budget draw, e.g. { api_families = 1 }
overrides = ["16 §2.1", "adr/0006 §2"]   # optional; chapters this row corrects
notes     = "Unfixable after the first Posting."

# ── budget consumption is DERIVED, never authored ─────────────────────────────
# xtask sums `consumes` across capabilities per phase and asserts it against
# [phase.budget]. A sum that exceeds the budget fails the build with the phase,
# the axis, the limit, the sum and the offending capability ids.
```

### 5.3 The assertions `cargo xtask` makes

| Check | Failure mode it prevents |
|---|---|
| `phase-check` — every Work Unit's `capability:` field names a `[[capability]].id` whose `phase` equals `current_phase` | An agent builds the right thing in the wrong phase (`60 §3.1`'s M2.6 scenario) |
| `lint-phases` — every phase column in every chapter is byte-identical to its render from this file | Eight roadmaps |
| Budget summation per `[phase.budget]` axis | `02 §5` becomes mechanically enforceable rather than reviewed |
| Every `[[capability]].owner` resolves to an existing chapter and section heading | Broken references (X15) |
| Every `[[capability]].id` present in the previous commit is still present | A capability silently disappearing from the plan |
| Every `kind = "source"` capability has a `[[capability]]` with `kind = "sink"` in the same or an earlier phase | P12: every Source has a named Sink |

`phase-check` runs in the **fast lane** (`40 §12`), because a phase violation caught after a full-lane run has already cost a reviewer's attention, and reviewer attention is the throughput ceiling (`60 §3.11`).

---

## 6. Amendment Rule

A phase assignment changes only by **an ADR plus an edit to both `03-phase-authority.md` and `docs/phases.toml` in the same commit.** All three artifacts move together or none does.

The ADR must state:

1. The capability id and its current phase.
2. The proposed phase and the concrete situation that forced reconsideration.
3. **What leaves the destination phase to pay for it** (`02 §8` — nothing enters a phase without something leaving). An amendment with no corresponding cut is a rejected amendment wearing a disguise.
4. The budget axes affected, with the before-and-after sums from §4.2.
5. Every chapter whose generated phase column changes as a consequence.

An agent may draft such an ADR. **An agent may not accept one** (`40 §6.5`, P4).

Adding a *new* capability row is the same procedure: a capability that is not in this file has no phase, and work on it is out of scope by `02 §6` regardless of how small it looks.

Removing a row requires the same ADR and an explicit statement of whether the capability is cancelled or absorbed into another row, because a capability that quietly stops existing is how a plan becomes a fiction.
