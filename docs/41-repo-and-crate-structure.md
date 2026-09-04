# 41 — Repository and Crate Structure

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `10-system-architecture.md` §3 and §7, `11-domain-model.md` §1.
> **Governs:** repository topology, the Rust workspace layout, crate naming and layering, mechanical dependency-direction enforcement, feature flags and compilation targets, build performance, the front-end and generated-code layout, the `xtask` task runner, the code-generation pipeline, phase-by-phase repository growth, CODEOWNERS, and fixture policy.
> **Does not govern:** coding standards, test taxonomy, review rules, ADR format or performance budgets — those are `40-engineering-standards.md`. Branching, commit conventions, agent-driven commit hygiene, release tagging and changelog policy are `42-source-control-automation.md`. This chapter defines *where things live and what may reference what*; `40` defines *how they are written* and `42` defines *how they are landed*.

---

## 1. Position — The Tree Is a Load-Bearing Artifact

`10 §3` states a dependency rule with the word FORBIDDEN in it three times. A rule stated in prose is a suggestion. The purpose of this chapter is to make that rule a property of the filesystem, so that violating it requires editing a manifest that CI reads, rather than merely writing an import that nobody notices.

Three facts about this project make the repository layout unusually consequential.

**The work is done substantially by coding agents.** An agent proposing a change navigates by path. If `crates/domain/ledger/` cannot, by construction, contain a Postgres import, the agent does not need to have internalized P5 — it will get a red build in ninety seconds instead of a design review in three weeks. Structure is the cheapest form of instruction, and the only form that scales to an arbitrary number of contributors who have no memory between sessions.

**P13 makes cross-cutting change the normal case, not the exception.** "A feature is not shipped until it exists in the API, the CLI, and at least one GUI" means the modal pull request touches a domain crate, an application crate, a schema, a generated TypeScript client, a React route, and a CLI subcommand. Any topology that makes that transaction expensive will be routed around, and the parity gate will quietly become advisory.

**P5 promises swappability that has not been exercised yet.** The `Ledger` trait will one day have a Fractal Node L1 implementation behind it. The only way to know the abstraction holds is to have kept the domain layer free of vendor types every single day until then. That is not a review discipline. It is a lint over a dependency graph, and the dependency graph is the directory tree.

Everything below follows from those three facts.

---

## 2. One Repository

**Decision: a single polyglot monorepo containing the Rust workspace, every front end, the SDKs, the schemas, the infrastructure code, the first-party Extensions, and this blueprint.**

### 2.1 Why

| Reason | Which principle it serves | What it makes possible |
|---|---|---|
| One atomic commit spans schema + core + CLI + GUI | P3, P13 | The parity gate (`P13` falsification test) is a CI job over one commit, not a cross-repo reconciliation problem |
| One source of truth for generated code, one regeneration step | P3 | `cargo xtask codegen` produces an empty diff or the build fails — impossible to check across repositories without a publishing dance |
| One `Cargo.lock`, one lockfile per language ecosystem | P8, N-supply-chain | The SBOM is a property of a commit. "Which dependency versions were in production on Tuesday" has one answer |
| One dependency graph to lint | P5 | The forbidden directions in `10 §3` are checkable because every edge is visible in one `cargo metadata` output |
| Refactoring across a boundary is a normal PR | `02 §7` | Boundaries can be corrected cheaply while we are still learning where they are (`10 §2`: we do not yet know the true seams) |
| Agents get complete context from one clone | P4 operations | An agent that must clone six repositories to answer one question will answer it wrong |
| One meaning of "green" | `40` | A release tag is a commit hash, not a compatibility matrix |

The decisive argument is the second row combined with the first. Fractal Node's whole architecture is a single core with generated contracts radiating outward to many front ends. A polyrepo turns every contract change into a version-negotiation problem between repositories that are always, by definition, at different commits. We would spend the project's velocity budget on the very coordination that the monorepo makes free.

### 2.2 The Honest Costs, and What We Do About Each

| Cost | Why it is real | Mitigation | Residual risk we accept |
|---|---|---|---|
| **CI time** — every PR could build everything | A 40-crate Rust workspace plus a web build is minutes, and it grows superlinearly with careless dependency edges | Path-filtered job selection; `cargo nextest` with test partitioning; `sccache` shared across CI; merge queue so the expensive full build runs once per batch, not once per push | Path filtering can mask a break. **Four jobs are never filtered out**: `lint-deps`, `codegen-diff`, `targets` (N2), and `parity` (P13). They run on every PR regardless of what changed |
| **Review scope** — a PR can touch anything | Large diffs get rubber-stamped, which is worse than no review | CODEOWNERS per path (§14); PR size limits from `40`; sensitive paths (ledger, economy, identity, ports, schemas, CI config) require an explicit human approval that agent identities cannot supply | A determined author can still split a dangerous change across small PRs. `42` addresses this with milestone-granular review |
| **Git history size** | Binaries and generated artifacts compound forever; agents cloning repeatedly amplify the cost | No binaries over 512 KB, enforced in CI (§15); no vendored dependencies; generated *contracts* are committed but generated *code* mostly is not; CI and agents clone with `--filter=blob:none --depth=1` | The committed `schemas/` and TypeScript client will grow. Budgeted: they are text, they compress, and their diffs are the review artifact that justifies them |
| **Tooling** — polyglot builds are fragmented | `cargo`, `pnpm`, `tauri`, `terraform` do not share a task model | A single Rust `xtask` runner is the only supported entry point for cross-language tasks (§11); one `rust-toolchain.toml`, one `.node-version`, one dev container | Developers can still run the underlying tools directly and get subtly different results. Accepted; CI runs only `xtask` |
| **Coupling temptation** | A monorepo makes it *trivially easy* to reach across a boundary that should be respected | This is exactly what §7 exists for. The monorepo does not weaken the boundary; it makes the boundary enforceable at build time instead of at publish time | None. This is the trade we are making deliberately |
| **Blast radius of a bad merge** | One broken commit blocks everyone | Merge queue with required checks; `main` is always releasable; revert-first policy (`42`) | Accepted |

### 2.3 Alternatives Rejected

**Polyrepo (one repository per service and per client).** Rejected. It optimizes for independent release cadence, which we explicitly do not have: `10 §2` ships a modular monolith, and `P13` requires lockstep feature parity across front ends. In a polyrepo, the schema lives somewhere and everyone else consumes a published version of it — meaning that at any moment the CLI is built against schema v7, the web app against v6, and the core emits v8. The P13 gate becomes unenforceable, the codegen-diff check becomes unwritable, and the cost of moving a boundary (which `10 §2` says we will do, in a predicted order) rises from "one PR" to "a migration project". Revisit only if the organization grows independent teams with genuinely independent products — not services.

**A single mega-crate.** Rejected, and this is the more seductive error because it starts out faster. One crate means no dependency edges to lint, therefore no way to enforce `10 §3` at all; the compiler becomes the only reviewer and it has no opinion about whether the ledger imports Postgres. It also means every change recompiles everything, no crate can be selectively compiled to `wasm32`, and feature flags become a global mess of `#[cfg]` inside a single unit. The layering rule is the product of the crate boundaries; deleting the boundaries deletes the rule.

**Polyglot orchestration via nx or turborepo.** Rejected as the *primary* build system. Both are excellent at JavaScript-shaped work and both treat Rust as an opaque shell command, which means their caching is coarse (any Rust change invalidates the whole Rust task) and their task graph duplicates information `cargo` already owns. We use `pnpm` workspaces for the JS packages and let `cargo` own the Rust graph; `xtask` is the thin seam between them. Revisit if the JS side grows past roughly ten packages with meaningful inter-package build fan-out.

**Bazel.** Rejected, with the most reluctance of the four. Bazel genuinely solves the two problems we will feel most — remote caching with correct invalidation, and a single hermetic graph across Rust, TypeScript, protobuf and containers. It is rejected because `rules_rust` requires maintaining a translated dependency graph alongside `Cargo.toml`, because every contributor and every agent already knows `cargo` and knows nothing about Bazel, and because `02 §5`'s complexity budget would be consumed by the build system before the spine sentence is true. **Stated revisit trigger:** if a clean CI build exceeds 25 minutes at more than 120 crates, and `sccache` hit rates are already above 80%, Bazel is reconsidered by ADR rather than argued about ad hoc.

---

## 3. The Repository Tree

Every directory below has exactly one purpose. Directories that do not yet exist at the current phase are marked in §13, not omitted here — the shape is decided once, and it is populated over time.

```
fractal-node/
├── AGENTS.md                    Generated agent entry point; the Canon load order and the current phase
├── README.md                    Human entry point; build, run, and the three-file contract
├── CODEOWNERS                   Path → accountable human. Never an agent identity (§14, P4)
├── LICENSE
├── Cargo.toml                   The single Rust workspace root (§6)
├── Cargo.lock                   Committed. The supply-chain record of record (P8)
├── rust-toolchain.toml          Pinned toolchain + every target triple, including wasm32 (N2)
├── deny.toml                    cargo-deny: licences, advisories, duplicate versions, vendor bans
├── layers.toml                  THE dependency-direction contract. Machine-read by xtask (§7)
├── clippy.toml                  Workspace lint config; per-layer files are generated from layers.toml
├── rustfmt.toml
├── package.json                 pnpm workspace root for the JS side
├── pnpm-workspace.yaml
├── .node-version                Pinned Node toolchain
├── .cargo/
│   └── config.toml              `cargo xtask` alias, linker choice, per-target rustflags
├── .agents/                     Agent-facing automation config — the machine-readable Canon
│   ├── context.toml             Required-reading manifest: which docs load for which paths
│   ├── phase.toml               Current phase, its deliverables, its complexity budget (02 §5)
│   ├── tasks/                   Task templates: add-a-port, add-a-domain-event, add-a-CLI-verb
│   ├── policies/                What agents may land unattended vs. what needs a human (P4, 42)
│   └── checks.toml              The four questions from 02 §6, as a PR-body schema
├── .github/
│   ├── workflows/               CI. See §8 and §12 for the non-negotiable jobs
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── ISSUE_TEMPLATE/
├── crates/                      The Rust workspace. Directory = layer. See §4, §5
│   ├── support/                 Cross-layer primitives with no layer of their own
│   │   ├── types/               fractal-types — ids, quanta, timestamps, canonical newtypes
│   │   ├── macros/              fractal-macros — derives for events, schema, secrets
│   │   ├── schema/              fractal-schema — the schema registry; SoT for codegen (§12)
│   │   └── testkit/             fractal-testkit — fakes, generators, the simulation harness
│   ├── ports/                   fractal-ports — every trait from 10 §7. The only way out (P5)
│   ├── domain/                  Pure logic. No I/O. No vendor types. One crate per boundary
│   │   ├── society/  identity/  discourse/  vault/  ledger/  economy/
│   │   ├── progression/  asset/  governance/  agent/  extension/
│   │   └── market/  discovery/
│   ├── app/                     Command/query handlers, sagas, the Policy Enforcement Point
│   │   ├── kernel/              Command bus, idempotency, saga runner, PEP (10 §8)
│   │   └── society/ identity/ discourse/ … (one per boundary, as needed)
│   ├── adapter/                 One crate per concrete implementation of a port
│   │   ├── postgres/  s3/  nats/  ledger-internal/  chain-null/  mls/
│   │   └── ffmpeg/  otel/  keystore-os/  search-pg/  model-http/  wasmtime/
│   ├── api/                     The gateway. Transport, authn, Envelope authz, rate limits
│   │   ├── gateway/  http/  grpc/  ws/
│   ├── core/                    The embeddable client core. Compiles to wasm32 (N2)
│   │   ├── core/                fractal-core — sync engine, local store, crypto, outbox
│   │   ├── wasm/                fractal-core-wasm — wasm-bindgen surface for the web app
│   │   └── ffi/                 fractal-core-ffi — UniFFI surface for iOS/Android (Phase 5)
│   └── bin/                     The shipped binaries
│       ├── node/                fractal-node — the Runtime: server and Tauri sidecar
│       ├── cli/                 fractal-cli — the `fn` binary and the Terminal (N3)
│       └── agent/               fractal-agent — the Agent executor host
├── xtask/                       The task runner. Rust, not Make (§11)
├── apps/
│   ├── web/                     React + TypeScript + Vite web GUI (§10)
│   ├── desktop/                 Tauri v2 shell. Embeds fractal-node in-process — a real Node
│   ├── ios/                     Phase 5. Swift shell over fractal-core-ffi
│   └── android/                 Phase 5. Kotlin shell over fractal-core-ffi
├── packages/                    pnpm workspace packages
│   ├── tokens/                  SOURCE of the design tokens (N7). Hand-authored
│   ├── design-system/           React components + CSS built from tokens. Web + desktop
│   ├── api-client/              GENERATED TypeScript client. Never hand-edited (§12)
│   ├── event-types/             GENERATED TS types for domain events
│   └── config/                  Shared eslint / tsconfig / vite presets
├── extensions/                  First-party Extensions, built on the public Host API (P7)
│   ├── fn-polls/  fn-digest/  fn-triage/  fn-lightbox/  fn-onboarding/
│   └── fn-terminal-dash/  fn-charter-templates/  fn-high-contrast/
├── sdks/
│   ├── typescript/              Public SDK; wraps packages/api-client with ergonomics
│   ├── python/                  Generated types + a hand-written ergonomic layer
│   └── rust/                    Thin re-export of fractal-core + generated API types
├── schemas/                     GENERATED, COMMITTED published contract (§12)
│   ├── openapi/                 OpenAPI 3.1 per API version
│   ├── proto/                   gRPC service and message definitions
│   ├── jsonschema/              Charter, Extension manifest, Facet Standard, Workflow
│   └── events/                  Per-event-kind JSON Schema + the compatibility snapshot
├── infra/
│   ├── terraform/               Cloud topology per 10 §9
│   ├── helm/                    Runtime deployment charts
│   ├── docker/                  Dockerfiles, cargo-chef layering (§9)
│   └── local/                   docker-compose dev stack: postgres, nats, minio, otel
├── fixtures/
│   ├── golden/                  Small committed golden files. Hard 512 KB per-file cap (§15)
│   ├── seeds/                   Declarative seed scenarios, replayed through the event log
│   └── media.lock               Content-addressed manifest for generated media fixtures
├── docs/                        This blueprint
│   ├── adr/                     Architecture Decision Records (40)
│   ├── proposals/               Out-of-phase ideas (02 §8). Not code
│   └── assets/
└── tools/                       Small dev utilities not worth a crate; scripts of last resort
```

**Rules about this tree that are enforced, not merely stated:**

1. `crates/<layer>/<name>/` — the path *is* the layer declaration. `layers.toml` maps path globs to layers, and the dependency lint reads paths, not attributes. A crate cannot lie about its layer without moving.
2. `schemas/`, `packages/api-client/`, `packages/event-types/` and every file carrying the generated header are **write-only by machine**. CI regenerates and diffs (§12).
3. `tools/` requires a justification comment at the top of every file explaining why it is not an `xtask` subcommand. It is a pressure valve with visible pressure.
4. No directory is created "for later". §13 states when each appears.

---

## 4. The Rust Workspace: Layers Are Directories

`01 §9` fixes the crate naming convention as `fractal-<layer>-<domain>`. This chapter adds the corresponding filesystem rule:

```
   crate name                     directory                        layer
   ─────────────────────────────  ───────────────────────────────  ──────────
   fractal-domain-ledger      ►   crates/domain/ledger/        ►   domain
   fractal-app-ledger         ►   crates/app/ledger/           ►   app
   fractal-adapter-postgres   ►   crates/adapter/postgres/     ►   adapter
   fractal-api-http           ►   crates/api/http/             ►   api
   fractal-ports              ►   crates/ports/                ►   ports
   fractal-core               ►   crates/core/core/            ►   core
   fractal-types              ►   crates/support/types/        ►   support
```

The allowed-edge graph, which is `10 §3` rendered as crate layers:

```
   ┌──────────────────────────────────────────────────────────────────────┐
   │  apps/web · apps/desktop · apps/ios · apps/android · extensions/     │
   │                          (front ends)                                │
   └───────────────────────────────┬──────────────────────────────────────┘
                                   │  HTTP / gRPC / WS only.
                                   │  No front end links a Rust crate except
                                   │  fractal-core (via wasm or ffi).
                                   ▼
   ┌──────────────────────────────────────────────────────────────────────┐
   │  api      fractal-api-gateway · -http · -grpc · -ws                  │
   └───────────────────────────────┬──────────────────────────────────────┘
                                   ▼
   ┌──────────────────────────────────────────────────────────────────────┐
   │  app      fractal-app-kernel · fractal-app-<boundary>                │
   └───────────────┬───────────────────────────────────┬──────────────────┘
                   ▼                                   ▼
   ┌───────────────────────────────┐   ┌──────────────────────────────────┐
   │  domain   fractal-domain-*    │──►│  ports    fractal-ports          │
   │  pure · no I/O · no vendors   │   │  traits only, no impls           │
   └───────────────┬───────────────┘   └──────────────▲───────────────────┘
                   ▼                                  │  implements
   ┌───────────────────────────────┐   ┌──────────────┴───────────────────┐
   │  support  fractal-types       │◄──│  adapter  fractal-adapter-*      │
   │           fractal-macros      │   │  vendor SDKs live HERE and only  │
   └───────────────────────────────┘   │  here                            │
                                       └──────────────────────────────────┘

   Composition root: crates/bin/* — the ONLY crates that may name both a
   port and its concrete adapter in the same file. Wiring is not a layer;
   it is three functions in main.rs.

   FORBIDDEN, mechanically:
     domain    ──►  adapter | api | app | any vendor crate
     domain(A) ──►  domain(B)                (events and published queries only)
     ports     ──►  anything but support
     adapter   ──►  domain | app             (adapters implement, they do not decide)
     app       ──►  api
     api       ──►  domain                   (must go through app)
     front end ──►  any crate but fractal-core / -wasm / -ffi
```

Two of those deserve their reasoning stated, because they are the ones people argue about.

**`adapter ──► domain` is forbidden.** The tempting version is an adapter that imports a domain type to persist it. That inverts the ownership of the mapping: the domain would then be unable to change a field without breaking a Postgres crate, and the vendor's constraints would start voting on domain shape. Instead, ports are defined in terms of `fractal-types` primitives and port-local DTOs, and the *application* layer translates. Honest cost: a translation layer that is genuinely tedious to write, roughly 15–20 lines per aggregate. Mitigation: `fractal-macros` provides derives for the mechanical half. We pay this cost because it is the entire content of the P11 promise.

**`domain(A) ──► domain(B)` is forbidden.** `10 §3` says cross-module reads go through a published query interface and cross-module writes go through events. Concretely: `fractal-domain-economy` may not depend on `fractal-domain-ledger`. It declares the query trait it needs (`ContributionLedgerView`) in its own crate or in `fractal-ports`, and `fractal-app-economy` supplies an implementation backed by the ledger app crate. This is the rule that keeps the predicted extractions in `10 §2` cheap: a boundary that never had a compile-time edge can be moved across a process boundary without a rewrite.

---

## 5. The Crate Catalogue

`Phase` is the phase in which the crate first exists (see §13). "May depend on" is the complete allowed set; anything not listed is a lint failure.

### 5.1 Support

| Crate | Purpose | May depend on | Phase |
|---|---|---|---|
| `fractal-types` | Canonical newtypes and value objects: `SocietyId`, `Fnid`, `Handle`, `Quanta`, `Xp`, `Trust`, `Ulid`, `Timestamp`, `Hash`, `Signature`, `Principal`. Zero behaviour beyond validity invariants and encoding | `serde`, `thiserror`, `ulid`, `blake3`, `bitflags` — nothing else, ever | 0 |
| `fractal-macros` | Derives: `#[derive(DomainEvent)]` (kind string, schema version, envelope wiring), `#[derive(Schema)]` (registry entry, §12), `#[secret]` (denies `Serialize`, per `10 §10`), `#[derive(Projection)]` | `syn`, `quote`, `proc-macro2` | 0 |
| `fractal-schema` | The schema registry: every event kind and API type, their versions, their upcasters, and the compatibility test corpus. The single source of truth for codegen | `fractal-types`, `fractal-macros`, `schemars`, `serde_json` | 0 |
| `fractal-testkit` | Deterministic simulation harness; in-memory fakes for **every** port (the P5 second implementation); `proptest` generators for every aggregate; the `11 §7` invariant property suite; fixture generators (§15) | `fractal-types`, `fractal-ports`, `fractal-domain-*`, `proptest`, `arbitrary` | 0 |

`fractal-types` is the crate everything depends on, therefore it is the crate whose every change triggers a full workspace rebuild. **Rule: a PR that modifies `fractal-types` states why in the PR body and is reviewed by a human.** Keeping it small and stable is a build-performance decision as much as a design one (§9).

`fractal-testkit` is where the P5 test doubles live, not in the adapter crates. This is deliberate: adapter crates then contain exactly one implementation each and can be excluded from a build wholesale, and the fakes are guaranteed to be built against the port and not against the vendor's behaviour.

### 5.2 Ports

| Crate | Purpose | May depend on | Phase |
|---|---|---|---|
| `fractal-ports` | Every trait from `10 §7` and nothing else: `EventStore`, `Ledger`, `Chain`, `BlobStore`, `Relay`, `KeyStore`, `Search`, `Transcoder`, `ModelProvider`, `Rail`, `Clock`, `Rng`, `IdGen`, `Telemetry` — plus the port-local DTOs and error enums those traits speak in | `fractal-types`, `async-trait`, `futures-core`, `thiserror` | 0 |

One crate, not fourteen. The traits are small, they change together when the layering changes, and fourteen crates would add fourteen compilation units to the critical path of every build for no isolation benefit — nothing implements a port without also linking the port's DTOs. If a single port grows past roughly 400 lines with its own dependency needs (the likely candidate is `Transcoder`, which may want codec descriptor types), it graduates to `fractal-ports-<name>` and `fractal-ports` re-exports it. That is a mechanical split available on demand, not a thing to pre-build (`02 §7`).

Ports are declared with `async_trait` rather than native async-fn-in-trait for as long as the ports must be object-safe — the composition root stores `Arc<dyn EventStore>`, and dynamic dispatch at an I/O boundary is free relative to the I/O. This is recorded as an ADR because it is a reversible decision with a clear expiry.

### 5.3 Domain — one crate per bounded context (`10 §3`)

Every crate below: pure functions and data, zero I/O, zero vendor types, exhaustively property-tested. **All of them share the same allowed dependency set**, stated once:

```
fractal-types, fractal-macros, fractal-ports (determinism module only — see §7.3),
serde, thiserror, rust_decimal, smallvec, bitflags, indexmap
```

| Crate | Boundary | Owns | Phase |
|---|---|---|---|
| `fractal-domain-society` | S2 | Society, Membership, Chamber, Charter, Lineage; Crystallization, Fracture, Fork, Dissolution state machines (`11 §3`) | 0 |
| `fractal-domain-identity` | S1 | Citizen, Handle, FNID derivation, device enrolment, recovery policy, session validity rules | 0 |
| `fractal-domain-discourse` | S3 | Thread, Message, reactions, read state, edit history; the CRDT merge functions | 1 |
| `fractal-domain-vault` | S4 | Object, Manifest, Shard, erasure scheme, ACL evaluation, Attestation validity | 1 |
| `fractal-domain-ledger` | S5 | Wallet, Posting, Transfer, Stake; the balance invariants of `11 §2.6`. The purest and most heavily tested crate in the tree | 1 |
| `fractal-domain-economy` | S6 | Source, Sink, Contribution Score, emission policy, settlement arithmetic | 2 |
| `fractal-domain-progression` | S7 | XP, Level curve, Trust dynamics, Standing, Achievement and Unlock rules | 2 |
| `fractal-domain-asset` | S8 | Facet, Facet Standard evaluation, evolution rules, licence terms, provenance chain | 3 |
| `fractal-domain-governance` | S9 | Roles, capability sets, Proposal, Vote, quorum arithmetic, enactment, moderation and appeal state machines | 2 |
| `fractal-domain-agent` | S10 | Agent, Envelope, Policy evaluation, capability intersection, Workflow graph validity | 2 |
| `fractal-domain-extension` | S11 | Extension manifest validation, Install, permission intersection, version compatibility | 3 |
| `fractal-domain-market` | S12 | Listing, rating, purchase, licence issuance, revenue-share arithmetic | 6 |
| `fractal-domain-discovery` | S13 | Interest declaration, Convergence lifecycle, match scoring over declared signals only (P9) | 2 |

S14 (Relay) has no domain crate. It is transport, not domain — `10 §3` says so explicitly ("transport, not domain"), and giving it a domain crate would invite persistence logic into a fan-out path. It exists as `fractal-ports::Relay` plus adapters.

**The Policy Enforcement Point is not a domain crate either.** `10 §8` places it in the application layer. `fractal-domain-agent` owns the *decision function* — given an Envelope, a Policy, and an action, allow or deny, purely. `fractal-app-kernel` owns *invoking it on every command path*. Splitting it this way is what makes the PEP exhaustively property-testable without a running system.

### 5.4 Application

| Crate | Purpose | May depend on | Phase |
|---|---|---|---|
| `fractal-app-kernel` | Command bus; the `idempotency_key` dedupe window (`10 §5`); the saga runner with compensation and resume (`11 §5`); the Policy Enforcement Point invocation; unit-of-work and transaction scoping; the event envelope constructor | `fractal-types`, `fractal-ports`, `fractal-domain-agent`, `fractal-schema`, `tokio`, `tracing` | 0 |
| `fractal-app-<boundary>` | Command handlers, query handlers, and sagas for one boundary. Translates between port DTOs and domain types. Emits events. Enforces policy | `fractal-app-kernel`, `fractal-ports`, its **own** `fractal-domain-<boundary>`, `fractal-types`, `fractal-schema` | with its domain crate |

An app crate may depend on exactly one domain crate: its own. Cross-boundary needs are expressed as a trait the app crate declares and the composition root satisfies — the same discipline as the domain layer, one level up, and the reason the extraction order in `10 §2` stays cheap.

| `fractal-app-atlas` | **S15 Atlas** (`10 §3`): the cross-Society read models — Citizen unified inbox, cross-Society search index, marketplace statistics, and the Shard reference count that governs GC (`13 §10.2`). Read-only; emits no domain event, holds no Wallet, takes no lock, and is never invoked by a command handler | `fractal-ports`, `fractal-types`, `fractal-schema` — and **no** `fractal-domain-*` | PH1 |

`fractal-app-projection` (Phase 1) is the one app crate that is not per-boundary: it hosts the projection runner, the replay driver, and the "rebuild every projection from seq 0" command that makes P6's falsification test executable. It also **hosts `fractal-app-atlas` in-process until PH5**, when Atlas extracts as a service (`10 §2` extraction ⑥).

**S15 has no domain crate, for the same reason S14 does not** (§5.3): it owns projections, not invariants. A domain crate for Atlas would invite read-path and persistence logic into the layer whose entire value is that it contains neither. The one correctness property Atlas must hold — the Shard refcount is **monotone-safe**, over-count permitted and under-count forbidden, with collection requiring positive confirmation of non-reference from every referencing Society at its current `seq` — lives as invariant V13 in `13 §14` and as an assertion in `fractal-sim`, not as a domain type (`61 W9`, `61 N15`).

### 5.5 Adapters — one crate per implementation

| Crate | Implements | Backing technology | Phase |
|---|---|---|---|
| `fractal-adapter-postgres` | `EventStore`, projection storage, `Search` (FTS) | `sqlx`, Postgres | 0 |
| `fractal-adapter-ledger-internal` | `Ledger` | Postgres double-entry tables | 1 |
| `fractal-adapter-chain-null` | `Chain` | Anchors to the internal Ledger (`10 §7`) | 1 |
| `fractal-adapter-s3` | `BlobStore` | S3-compatible object storage | 1 |
| `fractal-adapter-nats` | `Relay` (durable), event bus | NATS JetStream | 1 |
| `fractal-adapter-ws` | `Relay` (live) | In-process fan-out + WebSocket | 1 |
| `fractal-adapter-keystore-os` | `KeyStore` | OS keychain / libsodium | 1 |
| `fractal-adapter-otel` | `Telemetry` | OpenTelemetry OTLP | 0 |
| `fractal-adapter-clock` | `Clock`, `Rng`, `IdGen` | System time, OS entropy, ULID | 0 |
| `fractal-adapter-mls` | E2EE group operations behind `KeyStore`/group port | OpenMLS (RFC 9420) | 2 |
| `fractal-adapter-model-http` | `ModelProvider` | Hosted model APIs; per-Society endpoint config | 2 |
| `fractal-adapter-rail-internal` | `Rail` | Internal FRC only (`02 §3` defers the rest) | 2 |
| `fractal-adapter-wasmtime` | Extension sandbox host | Wasmtime component model | 3 |
| `fractal-adapter-ffmpeg` | `Transcoder` | ffmpeg | 4 |
| `fractal-adapter-tantivy` | `Search` | Tantivy, when Postgres FTS stops paying | 4 |

Allowed dependencies for every adapter crate: `fractal-ports`, `fractal-types`, and its vendor SDK. **Not** `fractal-domain-*`, **not** `fractal-app-*`. The dependency lint asserts this by name, because it is the single most likely rule to be violated under deadline pressure.

`fractal-adapter-wasmtime` implements a sandbox port that is **not** in the `10 §7` list. That list is Canon; adding to it requires an ADR, which `20` already names as required before Phase 3. Until that ADR lands, the crate is scaffolded but the port lives in `crates/ports/` behind `#[cfg(feature = "unstable-sandbox")]` so that its provisional status is visible in the code rather than in someone's memory.

### 5.6 API / Gateway

| Crate | Purpose | May depend on | Phase |
|---|---|---|---|
| `fractal-api-gateway` | Transport-agnostic middleware: authn (passkeys, device keys), Envelope authorization dispatch, per-principal token buckets, quota, idempotency key extraction, correlation-id propagation, error mapping | `fractal-app-*`, `fractal-schema`, `tower`, `fractal-types` | 0 |
| `fractal-api-http` | HTTP/JSON surface generated against `schemas/openapi` | `fractal-api-gateway`, `axum` | 0 |
| `fractal-api-ws` | WebSocket Signal subscription surface | `fractal-api-gateway`, `fractal-ports::Relay` | 1 |
| `fractal-api-grpc` | gRPC surface for SDKs and inter-Node traffic | `fractal-api-gateway`, `tonic` | 4 |

API crates may not depend on `fractal-domain-*`. If a handler needs a domain type, that is the signal that the type belongs in the schema registry and the app layer should be returning a DTO.

### 5.7 Core, Binaries, and Tooling

| Crate | Purpose | May depend on | Phase |
|---|---|---|---|
| `fractal-core` | The embeddable client core (`10 §6`): sync engine, local store, outbox, conflict policies per data class, client-side crypto, staleness tracking. **The crate that makes N2 and P13 true.** Compiles to `wasm32-unknown-unknown`, native desktop, iOS and Android | `fractal-types`, `fractal-schema`, generated API types, `serde`; storage and transport behind features (§8) | 0 |
| `fractal-core-wasm` | `wasm-bindgen` surface: the JS-facing API the web app and the Tauri renderer both consume | `fractal-core`, `wasm-bindgen`, `serde-wasm-bindgen` | 0 |
| `fractal-core-ffi` | UniFFI surface for Swift and Kotlin | `fractal-core`, `uniffi` | 5 |
| `fractal-node` | **The Runtime binary.** Composition root: reads config, constructs adapters, wires ports, serves the API. Runs headless (server) or embedded in Tauri (desktop Node) | everything except `fractal-cli`, `fractal-agent` | 0 |
| `fractal-cli` | The `fn` binary and the Terminal experience. Speaks the **public API only** — the P3 falsification test | `fractal-core`, generated API types, `clap`, `ratatui` | 0 |
| `fractal-agent` | The Agent executor host: model calls, tool dispatch, Envelope-scoped action submission | `fractal-core`, `fractal-ports::ModelProvider`, generated API types | 2 |
| `xtask` | The task runner (§11). Not published, not named `fractal-*` — it is not part of the product | `fractal-schema`, `cargo_metadata`, `xshell` | 0 |

**`fractal-cli` does not depend on `fractal-app-*` or on any adapter.** This is the structural expression of N3. If the CLI could link the application layer, it would eventually call it, and P3's "the CLI must be able to perform 100% of the actions the GUI can perform" would become "the CLI can perform some actions the GUI cannot, by a private path". The lint forbids it, and that is the whole mechanism.

`fractal-node` is the only crate permitted to name a port and its adapter together. Composition is not a layer — it is a hundred lines in `main.rs` and a config enum.

---

## 6. Cargo Manifests

### 6.1 Workspace root

```toml
# Cargo.toml
[workspace]
resolver = "2"
members  = [
  "crates/support/*",
  "crates/ports",
  "crates/domain/*",
  "crates/app/*",
  "crates/adapter/*",
  "crates/api/*",
  "crates/core/*",
  "crates/bin/*",
  "xtask",
]
# Globs, not an enumerated list: adding a crate must not require editing this file,
# because the layer is expressed by the path and enforced by layers.toml (§7).
exclude = ["apps", "packages", "extensions", "sdks", "fixtures"]

[workspace.package]
edition      = "2021"
rust-version = "1.83"
license      = "AGPL-3.0-or-later"
repository   = "https://github.com/fractal-node/fractal-node"

[workspace.lints.rust]
unsafe_code               = "forbid"   # per-crate `allow` only in adapter/ffi, with an ADR
missing_debug_implementations = "warn"
unused_crate_dependencies = "warn"     # keeps the graph shallow (§9)

[workspace.lints.clippy]
disallowed_methods = "deny"            # configured per layer via generated clippy.toml
disallowed_types   = "deny"
unwrap_used        = "deny"            # test code exempted in 40
```

### 6.2 Shared dependency versions

Every third-party version is declared **once**, at the workspace root. A crate that writes a version number in its own manifest fails `cargo xtask lint-deps`. This is what makes `02 §5`'s "5 new third-party runtime dependencies per phase" budget countable: the diff of this table *is* the dependency budget report.

```toml
[workspace.dependencies]
# ── support: allowed in every layer, including domain ──────────────────────
serde        = { version = "1", features = ["derive"] }
thiserror    = "2"
ulid         = { version = "1", default-features = false }
blake3       = { version = "1", default-features = false }
rust_decimal = { version = "1", default-features = false }
indexmap     = "2"
bitflags     = "2"

# ── app / api / adapter only — NEVER reachable from a domain crate ─────────
tokio     = { version = "1", default-features = false }
tower     = "0.5"
axum      = "0.8"
sqlx      = { version = "0.8", default-features = false, features = ["postgres", "macros"] }
async-nats = "0.38"
tracing   = "0.1"
opentelemetry = "0.27"

# ── client core: must compile to wasm32 ───────────────────────────────────
wasm-bindgen = "0.2"
gloo-timers  = "0.3"
rusqlite     = { version = "0.32", features = ["bundled"] }

# ── internal ──────────────────────────────────────────────────────────────
fractal-types  = { path = "crates/support/types" }
fractal-macros = { path = "crates/support/macros" }
fractal-schema = { path = "crates/support/schema" }
fractal-ports  = { path = "crates/ports" }
```

### 6.3 A domain crate manifest — the shape of purity

```toml
# crates/domain/ledger/Cargo.toml
[package]
name    = "fractal-domain-ledger"
edition.workspace = true
version.workspace = true

[dependencies]
fractal-types.workspace  = true
fractal-macros.workspace = true
serde.workspace          = true
thiserror.workspace      = true
rust_decimal.workspace   = true

[dev-dependencies]
fractal-testkit = { path = "../../support/testkit" }
proptest        = "1"

[lints]
workspace = true

# No [features]. A domain crate has no build-time variability: its behaviour is
# its specification. Conditional domain logic is a defect, not a configuration.
```

### 6.4 The client core manifest — the shape of portability

```toml
# crates/core/core/Cargo.toml
[package]
name = "fractal-core"

[features]
default = ["native"]

# Exactly one platform feature must be active. Enforced by a compile_error! guard.
native = ["dep:tokio", "dep:rusqlite", "dep:reqwest", "tokio/rt-multi-thread", "tokio/net"]
wasm   = ["dep:wasm-bindgen-futures", "dep:gloo-timers", "dep:idb", "dep:web-sys"]

# Capability features, orthogonal to platform.
e2ee      = ["dep:openmls"]
telemetry = ["dep:tracing"]
sim       = []                 # deterministic Clock/Rng; used by fractal-testkit

[dependencies]
fractal-types.workspace  = true
fractal-schema.workspace = true
serde.workspace          = true

tokio        = { workspace = true, optional = true }
rusqlite     = { workspace = true, optional = true }
reqwest      = { version = "0.12", optional = true, default-features = false, features = ["rustls-tls"] }

wasm-bindgen-futures = { version = "0.4", optional = true }
gloo-timers          = { workspace = true, optional = true }
idb                  = { version = "0.6", optional = true }
web-sys              = { version = "0.3", optional = true }

openmls = { version = "0.6", optional = true }
```

```rust
// crates/core/core/src/lib.rs — the guard that makes a misconfiguration a
// compile error rather than a runtime surprise in a browser.
#[cfg(all(feature = "native", feature = "wasm"))]
compile_error!("fractal-core: features `native` and `wasm` are mutually exclusive");
#[cfg(not(any(feature = "native", feature = "wasm")))]
compile_error!("fractal-core: exactly one of `native` or `wasm` must be enabled");
```

**Feature discipline, stated as rules because features are how workspaces rot:**

1. Features are **additive** except for the platform pair, which is guarded by `compile_error!`. No feature ever removes an item from the public API of a crate.
2. No `default = ["everything"]`. Defaults are the minimum that makes `cargo test -p <crate>` meaningful.
3. Features never gate *domain behaviour*. A feature may choose an implementation; it may never change what a Society is.
4. `cargo hack --feature-powerset --depth 2` runs nightly on `fractal-core` and `fractal-node`. Feature combinations that nobody builds are combinations that do not compile.

---

## 7. Dependency-Direction Enforcement

Three mechanisms, in increasing order of specificity. All three run on every pull request in the unfilterable `lint-deps` job.

### 7.1 `layers.toml` — the contract

```toml
# layers.toml — read by `cargo xtask lint-deps`. This file IS 10 §3.
schema_version = 1

[layer.support]
paths          = ["crates/support/types", "crates/support/macros", "crates/support/schema"]
may_depend_on  = ["support"]

[layer.ports]
paths          = ["crates/ports"]
may_depend_on  = ["support"]

[layer.domain]
paths          = ["crates/domain/*"]
may_depend_on  = ["support", "ports"]
# The transitive third-party closure of every crate in this layer must be a
# SUBSET of this list. Not the direct dependencies — the closure. This is the
# rule that catches tokio arriving three hops down inside a "pure" helper.
third_party_allowlist = [
  "serde", "serde_derive", "thiserror", "ulid", "blake3", "rust_decimal",
  "indexmap", "bitflags", "smallvec", "equivalent", "hashbrown", "arrayvec",
  "cfg-if", "zerocopy", "constant_time_eq", "arrayref",
]
# Additional import restriction, checked by source scan (§7.3):
restrict_imports = [
  { from = "fractal-ports", allow_modules = ["determinism"] },
]
forbid_std = ["std::fs", "std::net", "std::process", "std::time::SystemTime",
              "std::env", "std::thread"]

[layer.app]
paths         = ["crates/app/*"]
may_depend_on = ["support", "ports", "domain", "app"]
rule          = "one_domain_crate_max"   # an app crate owns exactly one boundary

[layer.adapter]
paths         = ["crates/adapter/*"]
may_depend_on = ["support", "ports"]     # NOT domain. NOT app.

[layer.api]
paths         = ["crates/api/*"]
may_depend_on = ["support", "ports", "app"]   # NOT domain.

[layer.core]
paths         = ["crates/core/*"]
may_depend_on = ["support", "core"]
targets       = ["wasm32-unknown-unknown", "x86_64-unknown-linux-gnu", "aarch64-apple-darwin"]

[layer.bin]
paths          = ["crates/bin/*"]
may_depend_on  = ["support", "ports", "domain", "app", "adapter", "api", "core"]
composition_root = true

[crate."fractal-cli"]
# N3: the CLI is a first-class API client, not a privileged insider.
may_depend_on  = ["support", "core"]
deny_layers    = ["app", "adapter", "api", "domain"]

[crate."fractal-agent"]
may_depend_on  = ["support", "core", "ports"]
deny_layers    = ["app", "adapter", "api", "domain"]
```

### 7.2 The `xtask` lint

`cargo xtask lint-deps` runs `cargo metadata --all-features --filter-platform=<each target>` and walks the resolved graph. Four assertions:

| Assertion | Catches |
|---|---|
| **A1 — layer edges.** Every internal edge `X → Y` satisfies `layer(Y) ∈ may_depend_on[layer(X)]`, plus any per-crate override | A domain crate importing an adapter; the CLI reaching into the app layer |
| **A2 — third-party closure.** For each crate in a layer declaring `third_party_allowlist`, the full transitive external closure is a subset of the allowlist | `tokio` arriving in the domain layer via a convenience crate three hops away — the failure mode a direct-dependency check misses entirely |
| **A3 — version centralization.** No member manifest contains a literal version for a third-party crate; all use `.workspace = true` | Silent version drift and an uncountable dependency budget (`02 §5`) |
| **A4 — path/name agreement.** `crates/<layer>/<dir>/Cargo.toml` declares `name = "fractal-<layer>-<dir>"` (with the documented exceptions for `fractal-ports`, `fractal-core*`, `fractal-node`, `fractal-cli`, `fractal-agent`) | A crate quietly relocated into a layer whose rules it does not satisfy |

`--filter-platform` matters. A dependency that is `cfg(unix)`-gated is still a dependency, and a check that only inspects the host platform's graph will pass on Linux CI and fail on a contributor's Mac. The lint runs the graph walk once per supported target triple.

Failure output names the exact edge and quotes the principle, because the primary consumer is an agent:

```
error[lint-deps/A2]: fractal-domain-ledger depends on a crate outside the domain allowlist

    fractal-domain-ledger
      └─ money-fmt v0.4
           └─ chrono v0.4
                └─ iana-time-zone v0.1        ← reads the filesystem

  P5: "No concrete vendor type may appear in a domain crate."
  10 §3: FORBIDDEN: domain ──► adapters
  Fix: express the need as a port in fractal-ports, or as a pure function over
  fractal-types. If you need the current time, take a `Clock` (10 §7).
```

### 7.3 `cargo-deny`, generated clippy configs, and the source scan

`cargo-deny` covers what a graph walk cannot: licences, advisories, duplicate versions, and specific named bans.

```toml
# deny.toml (excerpt)
[bans]
multiple-versions = "warn"
wildcards         = "deny"          # every version is pinned (P8)

deny = [
  { name = "openssl",     reason = "rustls only — one TLS stack, one audit surface" },
  { name = "chrono",      reason = "use fractal-types::Timestamp; chrono pulls tz db I/O" },
  { name = "lazy_static", reason = "use std::sync::OnceLock" },
]

[[bans.deny]]
name    = "tokio"
wrappers = []                        # no crate may re-export tokio into a lower layer

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC", "Unicode-3.0", "MPL-2.0"]
# Copyleft in a linked dependency is an ADR-level decision, not a lockfile accident.

[advisories]
yanked = "deny"
```

Two rules are not expressible in a dependency graph at all, so they are enforced at the source level:

**Restricted imports.** `10 §3` allows `domain ──► ports`, but the intent is narrow: a domain crate may take a `Clock`, `Rng`, or `IdGen` so that it is deterministically testable (`10 §7`: "this is what makes the entire domain layer deterministically testable"). It may not take an `EventStore`. `xtask` scans every `use fractal_ports::` line in `crates/domain/**` and requires the path to begin with `fractal_ports::determinism`. Fourteen lines of code, and it closes the one loophole the layer graph leaves open.

**Forbidden std surfaces.** A generated `clippy.toml` per layer, produced by `cargo xtask gen-lint-config` from `layers.toml` and diff-checked in CI like all generated files:

```toml
# crates/domain/ledger/clippy.toml  — @generated from layers.toml. Do not edit.
disallowed-types = [
  { path = "std::time::SystemTime", reason = "take a fractal_ports::determinism::Clock (10 §7)" },
  { path = "std::fs::File",         reason = "domain crates perform no I/O (P5)" },
]
disallowed-methods = [
  { path = "std::time::Instant::now",  reason = "non-deterministic; breaks replay (P6)" },
  { path = "rand::thread_rng",         reason = "take a fractal_ports::determinism::Rng" },
]
```

### 7.4 Alternatives rejected for enforcement

**`cargo-modules` / `cargo-depgraph` assertions.** Both are visualization tools first. Building the check on their output means depending on a rendering format for a build gate; neither exposes the transitive-closure query that assertion A2 needs. They are kept as *diagnostic* tools — `cargo xtask graph` renders a layer diagram for review — but they gate nothing.

**`cargo-deny` alone.** Handles A3 and vendor bans well; cannot express "which layer is this crate in", which is the actual rule. Used for what it is good at, not stretched.

**Convention plus code review.** Rejected outright. The contributor population is mostly agents with no institutional memory, and the rule's violations are individually invisible and collectively fatal. P5's own falsification test specifies "a dependency-direction lint in CI" — this section is that lint.

---

## 8. Compilation Targets and the Feature Matrix (N2)

N2 is stated as non-negotiable: "core crates must compile to `wasm32`, `x86_64`, `aarch64` in CI from Phase 0". Cross-platform is not a port we do later; it is a constraint that shapes what may enter `fractal-core` on day one. The mechanism is that the `targets` job exists before the first feature does.

### 8.1 The matrix

| Crate group | server (`x86_64-linux`, `aarch64-linux`) | desktop (`aarch64-darwin`, `x86_64-msvc`) | browser (`wasm32-unknown-unknown`) | mobile (`aarch64-ios`, `aarch64-android`) |
|---|---|---|---|---|
| `fractal-types`, `fractal-macros`, `fractal-schema` | ✔ | ✔ | ✔ | ✔ |
| `fractal-ports` | ✔ | ✔ | ✔ (traits only; no impls linked) | ✔ |
| `fractal-domain-*` | ✔ | ✔ | ✔ | ✔ |
| `fractal-core` | ✔ `native` | ✔ `native` | ✔ `wasm` | ✔ `native` |
| `fractal-core-wasm` | — | — | ✔ | — |
| `fractal-core-ffi` | — | ✔ (host build) | — | ✔ |
| `fractal-app-*`, `fractal-api-*` | ✔ | ✔ (embedded Node) | ✖ by design | ✖ |
| `fractal-adapter-postgres/s3/nats/ffmpeg` | ✔ | ✔ | ✖ | ✖ |
| `fractal-node` | ✔ | ✔ (Tauri sidecar / in-process) | ✖ | ✖ |
| `fractal-cli` | ✔ | ✔ | ✖ | ✖ |

**Domain crates compile to `wasm32` from Phase 0 even though nothing in the browser calls them yet.** This is not speculative work — it is the cheapest available proof that the purity rules hold. A domain crate that accidentally acquires a filesystem-touching transitive dependency fails the wasm build immediately, often before the dependency lint notices. The wasm target is a purity oracle we get for free.

The reason `fractal-app-*` and `fractal-adapter-*` are marked ✖ rather than "not yet" is P13's real shape: the browser is a *front end*, and front ends talk to the gateway. A world in which the application layer compiles to wasm is a world in which someone will eventually run policy enforcement in the browser, which `10 §8` forbids ("the Policy Enforcement Point lives in the application layer, inside the trust boundary").

### 8.2 Toolchain pinning

```toml
# rust-toolchain.toml
[toolchain]
channel    = "1.83.0"                # exact. Not "stable". Reproducibility is P8 hygiene
components = ["rustfmt", "clippy", "rust-src", "llvm-tools"]
targets = [
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "wasm32-unknown-unknown",
  "aarch64-apple-ios",
  "aarch64-linux-android",
]
```

### 8.3 The CI job that makes N2 real

```yaml
# .github/workflows/targets.yml — runs on EVERY pull request. Never path-filtered.
name: targets
on: [pull_request, merge_group]

jobs:
  compile-matrix:
    strategy:
      fail-fast: false
      matrix:
        include:
          - { name: server,   target: x86_64-unknown-linux-gnu, os: ubuntu-latest,
              args: "--workspace --exclude fractal-core-wasm" }
          - { name: server-arm, target: aarch64-unknown-linux-gnu, os: ubuntu-24.04-arm,
              args: "--workspace --exclude fractal-core-wasm" }
          - { name: desktop,  target: aarch64-apple-darwin, os: macos-latest,
              args: "--workspace --exclude fractal-core-wasm" }
          - { name: browser,  target: wasm32-unknown-unknown, os: ubuntu-latest,
              args: "-p fractal-core -p fractal-core-wasm --no-default-features --features wasm" }
          - { name: browser-domain, target: wasm32-unknown-unknown, os: ubuntu-latest,
              args: "-p fractal-types -p fractal-schema -p fractal-ports
                     $(cargo xtask list-crates --layer domain --as-package-args)" }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
        with: { filter: blob:none }
      - uses: dtolnay/rust-toolchain@1.83.0
        with: { targets: "${{ matrix.target }}" }
      - uses: mozilla-actions/sccache-action@v0.0.6
      - run: cargo check --locked --target ${{ matrix.target }} ${{ matrix.args }}

  wasm-size-budget:
    # P10 is an acceptance criterion, not polish. The wasm bundle has a number.
    needs: compile-matrix
    runs-on: ubuntu-latest
    steps:
      - run: cargo xtask wasm-build --release
      - run: cargo xtask budget --check wasm.gz --max 1200KB
```

The `browser-domain` row enumerates domain crates dynamically from `layers.toml`, so a new domain crate is covered by N2 on the commit that creates it. A matrix that must be hand-edited to stay complete will not stay complete.

`wasm-size-budget` is included here rather than in `40` because it is the budget most likely to be broken by a *structural* mistake — a dependency edge that drags a native-shaped crate into the core — and the structural mistake is this chapter's subject.

---

## 9. Build Performance

Rust's compile times are the honest cost recorded in `10 §11` ("long compile times, mitigated by workspace splitting + sccache"). This section is that mitigation, with numbers, because a target without a number is a wish.

### 9.1 Targets

| Scenario | Target | Failure threshold | Measured by |
|---|---|---|---|
| Clean debug build, cold cache, 16-core CI | ≤ 8 min | 12 min | `targets` job wall time |
| Clean debug build, warm `sccache` | ≤ 3 min | 5 min | CI, p50 over 7 days |
| `cargo check -p fractal-domain-<x>` after touching that crate | ≤ 8 s | 15 s | `cargo xtask bench-build` |
| `cargo check -p fractal-node` after touching one app crate | ≤ 45 s | 90 s | same |
| Touching `fractal-types` (full rebuild) | ≤ 4 min | 6 min | same |
| Release build with thin LTO, for a tag | ≤ 25 min | 40 min | release workflow |
| Web app dev server cold start | ≤ 4 s | 8 s | `vite` startup log |

`cargo xtask bench-build` runs weekly, writes results to `docs/assets/build-times.csv`, and opens an issue when a threshold is crossed. Compile time is tracked like any other budget: it does not regress silently, and it is not fixed in a panic.

### 9.2 Rules that keep the graph shallow

1. **Depth over breadth is the enemy.** Parallelism is bounded by the longest chain, not the crate count. Forty independent leaf crates compile faster than eight in a chain. The layer graph in §4 is four deep by construction.
2. **`unused_crate_dependencies = "warn"` is on at the workspace root.** Dependencies rot into manifests and never leave; this makes their departure visible.
3. **Proc macros are a serialization point.** `fractal-macros` must compile before anything that derives from it, so it takes only `syn`/`quote`/`proc-macro2` and is forbidden from growing runtime dependencies.
4. **`fractal-types` is the root of the world.** Every change rebuilds everything (see the 4-minute row above). Changes require a stated reason (§5.1).
5. **No crate may exceed 12 direct internal dependencies** without an ADR. A crate with twenty is either the composition root or a boundary violation wearing a manifest.
6. **`default-features = false` on every heavy third-party dependency**, opting features back in explicitly. `tokio` with default features drags in a scheduler nobody asked for.
7. **Integration tests are consolidated.** Each `tests/*.rs` file is a separate binary with its own link step; per-crate integration tests live in a single `tests/it.rs` with modules. This is worth roughly a minute of link time across the workspace.

### 9.3 Profiles

```toml
# Cargo.toml (workspace root)
[profile.dev]
opt-level     = 0
debug         = 1                 # line tables only; full debuginfo doubles link time
split-debuginfo = "unpacked"      # macOS/Linux: keeps the binary small, links faster
incremental   = true

[profile.dev.package."*"]
opt-level = 2                     # dependencies compiled once, fast at runtime;
debug     = false                 # our crates stay at opt-level 0 for fast iteration

[profile.ci]                      # used by every CI job
inherits      = "dev"
debug         = 0
incremental   = false             # incremental artifacts do not survive a cache boundary
codegen-units = 256

[profile.release]
lto           = "thin"            # "fat" buys ~3% runtime for ~2.5x link time. Not worth it
codegen-units = 1
panic         = "abort"           # except for fractal-node, which catches per-request panics
strip         = "debuginfo"

[profile.sim]                     # deterministic simulation: needs speed AND assertions
inherits    = "release"
debug-assertions = true
overflow-checks  = true
```

`overflow-checks = true` in the simulation profile is not optional. `11 §7` invariant 2 (`Σ debits == Σ credits`) is only meaningful if arithmetic that would wrap is caught, and the economy simulation under `17` runs at 100× adversarial volume — precisely the regime where a `u64` quanta calculation would silently wrap in a release build.

### 9.4 Caching

**`sccache`** with an S3 backend shared by CI and by developers who opt in. Keyed on compiler version, target, and crate metadata hash. Expected hit rate above 80% on a typical PR; below 60% for a week is treated as a defect and investigated.

**`cargo-chef`** for Docker, which is the difference between a 20-second image rebuild and an 8-minute one:

```dockerfile
# infra/docker/runtime.Dockerfile
FROM rust:1.83-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
# Layer 1: every third-party dependency. Invalidated only by Cargo.lock changes.
RUN cargo chef cook --release --recipe-path recipe.json -p fractal-node
# Layer 2: our source. Invalidated by every commit — but only ours rebuilds.
COPY . .
RUN cargo build --release --locked -p fractal-node

FROM gcr.io/distroless/cc-debian12 AS runtime
COPY --from=builder /build/target/release/fractal-node /usr/local/bin/
USER nonroot
ENTRYPOINT ["/usr/local/bin/fractal-node"]
```

**`mold`** as the linker on Linux (`.cargo/config.toml`), `lld` on macOS. Linking dominates incremental builds of a binary crate; mold takes `fractal-node`'s link from several seconds to a few hundred milliseconds.

**`cargo-hakari`** (a `workspace-hack` crate that unifies feature selections across the workspace) is **deferred, not adopted**. It solves real duplicate-compilation waste but adds a generated crate that every member depends on — which is precisely the shallow-graph rule inverted. Adopt it when the workspace passes 40 crates *and* `cargo xtask bench-build` shows more than 15% of build time in features-mismatched rebuilds. Naming the trigger in advance is what stops it becoming a taste argument.

---

## 10. Front-End Layout

### 10.1 The web app

```
apps/web/
├── index.html
├── vite.config.ts
├── package.json                  Depends on @fractal/design-system, @fractal/api-client
└── src/
    ├── main.tsx                  Bootstrap: init fractal-core-wasm, mount router
    ├── routes/                   ROUTING ONLY. One file per URL. No business logic.
    │   ├── _layout.tsx
    │   ├── societies.$societyId.chambers.$chamberId.tsx
    │   ├── societies.$societyId.treasury.tsx
    │   └── profile.$handle.tsx
    ├── features/                 THE UNIT OF WORK. One directory per user capability.
    │   ├── chamber/              components/ · hooks/ · state/ · index.ts (public API)
    │   ├── wallet/
    │   ├── charter/
    │   ├── agent-console/
    │   └── vault/
    ├── core/                     The wasm boundary: typed wrapper over fractal-core-wasm,
    │                             sync status, offline queue surface, staleness signals (P2)
    ├── shell/                    App chrome: navigation, command palette, Signal toasts
    └── test/                     MSW handlers generated from schemas/openapi (§12)
```

**Rules with teeth:**

- `routes/` may import from `features/` and `shell/`. `features/` may **not** import from `routes/`. A feature that needs the URL takes it as a prop. Enforced by `eslint-plugin-boundaries` configured from the same conceptual source as `layers.toml` — the frontend gets the same treatment as the workspace, because the same failure mode applies.
- A feature directory imports another feature only through its `index.ts`. Deep imports are an eslint error. Features are the front end's bounded contexts and they map, deliberately, onto `10 §3`'s service boundaries.
- **No component may call `fetch` directly.** Every network call goes through `@fractal/api-client` (generated) or through the `core/` wasm wrapper. This is P3's falsification test made greppable: "grep the web, desktop, and mobile clients for any HTTP call to an undocumented internal path" only works if there is exactly one place such a call could be.
- No feature owns a colour, a spacing value, or a font size. Those come from `@fractal/design-system` and ultimately from `packages/tokens` (N7). A raw hex literal in `apps/web/src/**` is a lint error.

### 10.2 The shared design system

```
packages/tokens/                  SOURCE (N7). Hand-authored. The only place a value is decided.
├── src/
│   ├── color.json  space.json  type.json  motion.json  elevation.json
│   └── theme/  default.json  high-contrast.json  terminal-amber.json
└── build.config.ts               Emitter registry

    cargo xtask tokens
            │
            ├──► packages/design-system/generated/tokens.css   (CSS custom properties)
            ├──► packages/design-system/generated/tokens.ts    (typed token objects)
            ├──► crates/bin/cli/src/generated/theme.rs         (ANSI + truecolor, N7)
            ├──► apps/ios/Generated/Tokens.swift               (Phase 5)
            ├──► apps/android/.../generated/Tokens.kt          (Phase 5)
            └──► docs/assets/tokens.html                       (the reference table)

packages/design-system/
├── generated/                    ← written by xtask. Never hand-edited.
├── src/
│   ├── primitives/               Button, Field, Sheet, Menu — a11y contracts live here
│   ├── patterns/                 MessageRow, WalletCard, CharterClause, AgentBadge
│   └── a11y/                     Focus management, live regions, reduced-motion helpers
└── package.json
```

The design system is consumed identically by `apps/web` and by `apps/desktop` — Tauri renders the same React tree in a native webview, so there is one component implementation, not two. The CLI consumes the *tokens* but not the components; `fractal-cli` renders with `ratatui` against `generated/theme.rs`. This is what N7 means by "one source, five targets", and it is why the token build is an `xtask` subcommand rather than a JavaScript-only pipeline: two of its five outputs are Rust and Swift.

### 10.3 The desktop shell

```
apps/desktop/
├── src-tauri/
│   ├── Cargo.toml                A workspace member? NO — see below.
│   ├── tauri.conf.json
│   └── src/main.rs               Embeds fractal-node IN-PROCESS. The desktop app is a Node
└── src/                          Thin: window chrome, deep links, tray, OS integration.
                                  All UI comes from packages/design-system + apps/web routes
```

`src-tauri` is **excluded** from the Rust workspace and consumes `fractal-node` by path dependency. The reason is practical: Tauri's build requires platform SDKs and its own `build.rs` codegen, and including it in the root workspace makes `cargo check --workspace` fail on any machine without those SDKs — including most CI runners and most agent environments. The cost is a second `Cargo.lock`; it is pinned to the root one by `cargo xtask lint-deps`, which asserts the two lockfiles agree on every shared package version. That check is cheap and it turns a real risk (a desktop build shipping a different `sqlx` than the server) into a build failure.

### 10.4 Generated code in the tree

| Path | Generated from | Committed? | Why |
|---|---|---|---|
| `schemas/**` | `fractal-schema` | **Yes** | It is the published contract. A wire change must be visible in the PR diff and reviewable by a human |
| `packages/api-client/**` | `schemas/openapi` | **Yes** | The JS build must not require a Rust toolchain; external SDK consumers read it |
| `packages/event-types/**` | `schemas/events` | **Yes** | Same |
| `packages/design-system/generated/**` | `packages/tokens` | **Yes** | Design review reads the diff; CSS custom-property changes are visually consequential |
| `crates/**/src/generated/**` | `fractal-schema` | **No** | Rust consumers already have the toolchain; generating at build time removes a whole class of stale-artifact bug |
| `apps/web/src/test/msw/**` | `schemas/openapi` | Yes | Test fixtures must be stable across machines |

Every generated file carries this header, and `cargo xtask lint-generated` fails if a file bearing it has been modified without its generator's output changing:

```
// @generated by `cargo xtask codegen` from crates/support/schema. DO NOT EDIT.
// Source hash: b3:9f2c…  ·  Generator version: 4
```

Generated paths are also listed in `.gitattributes` as `linguist-generated=true` (collapsing them in review) and are assigned in `CODEOWNERS` to a review group that only approves generator changes — so a hand-edit shows up as an unexpected reviewer request, not just a failing check.

---

## 11. The `xtask` Pattern

**Decision: every repository task is a subcommand of a Rust binary in `xtask/`, invoked as `cargo xtask <task>`.** There is no Makefile, no `justfile`, and no npm script that does anything other than delegate.

```toml
# .cargo/config.toml
[alias]
xtask = "run --package xtask --profile ci --"
```

### 11.1 The task list

| Task | Does | Runs in CI as |
|---|---|---|
| `codegen` | Regenerates schemas, clients, Rust types, CLI surface, docs (§12) | `codegen-diff` |
| `tokens` | Builds the design tokens to all five targets (§10.2, N7) | `codegen-diff` |
| `schema-check` | Event/API schema compatibility against the previous release's snapshot (`10 §5`, `10 §10`) | `schema-check` |
| `lint-deps` | The layer lint, A1–A4 (§7.2) | `lint-deps` |
| `gen-lint-config` | Emits per-layer `clippy.toml` from `layers.toml` (§7.3) | `codegen-diff` |
| `sim` | Runs the deterministic simulation harness with a seed; economy runs at 100× (P12) | `sim` (nightly + on economy paths) |
| `parity` | Diffs the API surface against the CLI command surface; fails on asymmetry (P13, N3) | `parity` |
| `budget` | Checks performance, bundle-size and axe-core budgets (P10, `40`) | `budget` |
| `fixtures` | Regenerates fixtures from generators; verifies `fixtures/media.lock` (§15) | `fixtures` |
| `targets` | Local mirror of the N2 compile matrix (§8.3) | — |
| `bench-build` | Measures the §9.1 numbers | weekly |
| `sbom` | Emits CycloneDX for every shipped artifact (P8) | `release` |
| `release` | Version bump, tag, sign, build all artifacts, publish (see `42`) | `release` |
| `changelog` | Assembles the changelog from conventional commits (see `42`) | `release` |
| `adr` | Scaffolds a new ADR from the `40` template; checks for open ADRs at a phase gate (`02 §5`) | `phase-gate` |
| `agents` | Regenerates `AGENTS.md` from `.agents/context.toml` and `.agents/phase.toml` | `codegen-diff` |
| `dev` | Brings up `infra/local` and runs the Runtime + web dev server | — |
| `graph` | Renders the crate-layer diagram for review (diagnostic only) | — |

Three of these — `parity`, `sim`, and `schema-check` — exist only because the tasks are written in Rust. `parity` links `fractal-schema` and `fractal-cli`'s `clap` command tree and compares two in-process data structures; as a shell script it would be a fragile diff of two `--help` outputs.

### 11.2 Why not `make`, `just`, or npm scripts

| Alternative | Why rejected |
|---|---|
| **Make** | Its value is the file-dependency DAG, which `cargo` and `vite` already own better; using it as a command menu means paying for its syntax without its benefit. Tab-sensitive, shell-dependent, and effectively unusable on Windows without extra tooling — and Windows is a first-class desktop target |
| **`just`** | Genuinely pleasant, and the closest call. Rejected because it is another binary every contributor and every agent container must install, and because its recipes are shell — so any task with real logic (`parity`, `lint-deps`, schema compatibility) either becomes unreadable or becomes a script in another language that `just` merely calls. At that point `just` is a menu with an install step |
| **npm scripts** | Would make Node a hard prerequisite for building the Rust core, inverting the dependency between the product's centre and its periphery. Also unable to express anything the Rust tasks need |
| **Shell scripts in `tools/`** | No types, no tests, no reuse of the crates, no Windows story, and they accumulate silently. `tools/` survives as a pressure valve (§3, rule 3) with a required justification comment, so that pressure is visible |

**Honest costs of `xtask`.** First invocation compiles the task runner (roughly 20–40 seconds cold; near-zero warm, and `profile.ci` keeps it cheap). Its dependencies are deliberately thin — `cargo_metadata`, `xshell`, `clap`, `serde_json`, plus `fractal-schema`. And there is no automatic parallel DAG: tasks declare their inputs explicitly and `cargo` does the actual build parallelism. We accept a slightly slower menu in exchange for tasks that are typed, testable, cross-platform, and able to import the codebase they operate on.

---

## 12. Code Generation Pipeline

### 12.1 The single source of truth

**Decision: `crates/support/schema` (`fractal-schema`) — annotated Rust — is the single source of truth for every API type and every domain event payload. Everything else in the contract surface is generated from it.**

```
                    ┌─────────────────────────────────────────┐
                    │   crates/support/schema/src/**          │
                    │   #[derive(Schema, DomainEvent)]        │
                    │   #[fn_schema(kind = "discourse.message │
                    │      .posted", version = 1, field = 7)] │
                    │        ◄── THE ONLY HAND-AUTHORED       │
                    │            CONTRACT ARTIFACT            │
                    └────────────────────┬────────────────────┘
                                         │  cargo xtask codegen
             ┌───────────────┬───────────┼───────────┬──────────────────┐
             ▼               ▼           ▼           ▼                  ▼
      ┌────────────┐  ┌────────────┐ ┌────────┐ ┌──────────┐  ┌──────────────┐
      │ schemas/   │  │ schemas/   │ │schemas/│ │ schemas/ │  │ crates/**/   │
      │ openapi/   │  │ proto/     │ │events/ │ │jsonschema│  │ src/generated│
      │ (3.1)      │  │ (gRPC)     │ │(per    │ │(Charter, │  │ (Rust DTOs,  │
      │            │  │            │ │ kind)  │ │ manifest)│  │  upcasters)  │
      └─────┬──────┘  └─────┬──────┘ └───┬────┘ └────┬─────┘  └──────┬───────┘
            │               │            │           │               │
      ┌─────┴───────┐  ┌────┴─────┐      │           │        ┌──────┴───────┐
      ▼             ▼  ▼          ▼      ▼           ▼        ▼              ▼
┌───────────┐ ┌────────┐ ┌───────────┐ ┌──────────────┐ ┌──────────┐ ┌────────────┐
│ packages/ │ │ sdks/  │ │ sdks/     │ │ packages/    │ │ CLI verb │ │ docs/      │
│api-client │ │python/ │ │typescript/│ │ event-types  │ │ surface  │ │ api-ref    │
│    (TS)   │ │        │ │           │ │    (TS)      │ │ (clap)   │ │  (MD)      │
└─────┬─────┘ └────────┘ └───────────┘ └──────────────┘ └────┬─────┘ └────────────┘
      │                                                       │
      ▼                                                       ▼
 apps/web · apps/desktop · MSW test handlers            fractal-cli (N3, P13)
```

**Why Rust and not an IDL.** The alternative — TypeSpec, protobuf-first, or a hand-written OpenAPI document as the source — was seriously considered and rejected on one argument: any external IDL creates a second place where the shape of a `Society` is written down, and two places drift. With the Rust types as the source, the compiler is the schema validator, the event payload that is serialized at runtime is definitionally the one the schema describes, and the upcaster required by `10 §5` for a `.v2` event lives next to both versions of the type. `11 §7` invariant 10 ("every projection is reproducible by replaying the log from its most recent verified checkpoint, and from zero within the published rebuild SLO") is only checkable if the replay path and the schema cannot disagree.

**The honest costs, and the mitigations.**

| Cost | Mitigation |
|---|---|
| A non-Rust contributor cannot propose a schema change without touching Rust | The change is ten lines in a struct. The generated diff shows them exactly what the wire change is |
| Rust → protobuf is lossy: field numbers must be stable and are not inferable | Field numbers are **explicit** in the attribute (`field = 7`). `schema-check` fails if a number is reused or a used number disappears |
| Rust's type system is richer than JSON Schema; some types have no faithful projection | The `Schema` derive rejects unrepresentable shapes at compile time (untagged enums with ambiguous variants, non-string map keys). Better a compile error than a silently wrong client |
| Generation must be re-run, and someone will forget | `codegen-diff` (below). Forgetting is a red build, not a support ticket |

### 12.2 The CI check

```yaml
# .github/workflows/codegen-diff.yml — never path-filtered.
name: codegen-diff
on: [pull_request, merge_group]
jobs:
  regenerate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.83.0
      - uses: pnpm/action-setup@v4
      - run: cargo xtask codegen
      - run: cargo xtask tokens
      - run: cargo xtask gen-lint-config
      - run: cargo xtask agents
      - name: Assert regeneration is a no-op
        run: |
          if ! git diff --exit-code --stat; then
            echo "::error::Generated artifacts are stale."
            echo "Run: cargo xtask codegen tokens gen-lint-config agents"
            echo "Then commit the result. Never hand-edit a @generated file."
            exit 1
          fi
      - run: cargo xtask lint-generated   # no @generated file edited by hand
```

`schema-check` is a separate, stricter job: it fetches the previous release tag's `schemas/` directory and asserts the compatibility rules from `10 §5` — additive-only within a version, a new `.v<n+1>` kind for any breaking change, and a registered upcaster for every version bump. A breaking change without an upcaster fails the build with the event kind named.

### 12.3 The CLI surface is generated too

`fractal-cli`'s command tree is generated from the same registry: every API resource family produces `fn <noun> <verb>` per `01 §9`, with hand-written ergonomics layered on top (`ratatui` dashboards, interactive prompts, output formatting). `cargo xtask parity` then compares the two surfaces in-process and fails on any API operation with no CLI verb. This is P13's falsification test executed by a machine on every pull request rather than remembered by a human at a release tag — which is the difference between a principle and a slogan.

---

## 13. Repository Evolution by Phase

The tree's *shape* is decided now (§3). Its *contents* arrive when the phase that needs them arrives. `02 §7`'s anti-gold-plating rules forbid scaffolding ahead of need; the counter-risk — a disruptive reorganization later — is handled by having decided the layout, the naming, and the layer rules up front, so that adding a crate is a `mkdir` and adding a boundary is a directory, never a migration.

| Phase | Spine capability | Rust crates added | Other directories added |
|---|---|---|---|
| **0** Foundations | Skeleton that compiles to all three targets; event log; one command end to end | `fractal-types`, `-macros`, `-schema`, `-testkit`, `-ports`, `-domain-society`, `-domain-identity`, `-app-kernel`, `-app-society`, `-app-identity`, `-adapter-postgres`, `-adapter-clock`, `-adapter-otel`, `-api-gateway`, `-api-http`, `fractal-core`, `-core-wasm`, `fractal-node`, `fractal-cli`, `xtask` | `.agents/`, `.github/`, `crates/`, `schemas/`, `docs/`, `infra/local/`, `fixtures/`, `packages/tokens/`, `packages/api-client/` |
| **1** Talk and store | Chambers, messages, vault, wallet, the web GUI | `-domain-discourse`, `-domain-vault`, `-domain-ledger`, `-app-discourse`, `-app-vault`, `-app-ledger`, `-app-projection`, `-adapter-s3`, `-adapter-nats`, `-adapter-ws`, `-adapter-ledger-internal`, `-adapter-chain-null`, `-adapter-keystore-os`, `-api-ws` | `apps/web/`, `apps/desktop/`, `packages/design-system/`, `packages/event-types/`, `infra/terraform/`, `infra/helm/`, `infra/docker/` |
| **2** Earn, govern, automate | Economy, progression, governance, agents, E2EE, discovery | `-domain-economy`, `-domain-progression`, `-domain-governance`, `-domain-agent`, `-domain-discovery` + their `-app-*`, `-adapter-mls`, `-adapter-model-http`, `-adapter-rail-internal`, `fractal-agent` | `sdks/typescript/` |
| **3** Assets and extensions | Facets; the Extension Host and first-party plugins (P7) | `-domain-asset`, `-domain-extension`, `-app-asset`, `-app-extension`, `-adapter-wasmtime` | `extensions/`, `sdks/python/` |
| **4** Media and distribution | Voice/video, transcoding, Custodian mesh, gRPC | `-adapter-ffmpeg`, `-adapter-tantivy`, `-api-grpc` | `infra/terraform/regions/` |
| **5** Native and Fracture | Mobile shells, Fracture, advanced discovery, governance beyond roles | `fractal-core-ffi` | `apps/ios/`, `apps/android/` |
| **6** Marketplace and federation | Paid Extensions, federation, multi-node self-hosting | `-domain-market`, `-app-market` | `sdks/rust/` |
| **7+** Experiences | The Experience Runtime | (extraction crates, see below) | — |

### 13.1 Reorganizations accepted in advance

`10 §2` names a predicted extraction order and instructs us not to pre-build it. That instruction only holds if extraction is cheap when it comes. It is cheap here for one reason: an extracted service is a **new binary crate over existing app and adapter crates**, not a new codebase.

| Predicted extraction (`10 §2`) | Phase | What actually changes | Cost |
|---|---|---|---|
| ① Media Transcoder | 4 | New `crates/bin/transcoder/`. `fractal-adapter-ffmpeg` moves behind a `Transcoder` implementation that speaks gRPC to it. No domain or app change | ~1 week |
| ② Relay / SFU | 4–5 | New `crates/bin/relay/`. `fractal-adapter-nats` + `-ws` become its internals; the Runtime holds a remote `Relay` client. S14 has no domain crate precisely so this is possible | ~2 weeks |
| ③ Agent Executor | 5 | `fractal-agent` already exists as a separate binary that speaks the public API (§5.7). Extraction is a deployment change, not a code change | days |
| ④ Search Indexer | 5–6 | New binary consuming the NATS event stream and owning `-adapter-tantivy` | ~1 week |
| ⑤ Custodian Coordinator | 6 | New binary over `-app-vault`'s attestation and settlement handlers | ~2 weeks |

Each extraction adds a binary crate under `crates/bin/`, which `layers.toml` already permits as a composition root. **The repository layout does not change.** That is the entire point of having decided it now.

Three further reorganizations are accepted in advance, with their triggers:

1. **`fractal-ports` splits** when one port exceeds ~400 lines with its own dependency needs (§5.2). Mechanical; `fractal-ports` re-exports, so no consumer changes.
2. **`crates/app/<boundary>` splits into `command/` and `query/`** when an app crate exceeds ~4,000 lines. CQRS at the crate level is a compile-time win and a clarity win; doing it before the size justifies it is gold-plating.
3. **The web app splits into per-Society lazy bundles** when the initial bundle approaches the P10 budget. `features/` is already the split boundary, so this is a router change.

**Reorganizations we refuse in advance:** renaming the crate prefix; changing the `crates/<layer>/<name>` path convention; moving away from one workspace; introducing a second source of truth for schemas. Each would invalidate `layers.toml`, the codegen pipeline, or both, and each would be defended on grounds that will sound reasonable at the time. They require an ADR that names this paragraph.

---

## 14. Ownership and CODEOWNERS

The team is one or few humans plus a large, rotating population of AI agents. This inverts the usual purpose of `CODEOWNERS`. In a human team it routes review to the person with context. Here, **there is no agent with context that survives the session**, so the file's job is different: it is the mechanism by which P4 ("policy is defined exclusively by humans") is enforced at the point where code becomes canon.

### 14.1 What the file actually gates

`CODEOWNERS`, combined with branch protection requiring owner review, gates **merge**, not authorship. Agents author freely on branches. Landing on `main` requires an approval from an account listed in `CODEOWNERS`, and **no agent identity is ever listed there.** That is the whole control. Everything else in this section is refinement.

```
# CODEOWNERS — owners are accountable humans. Agent identities never appear here.
# Default: everything has an owner. No unowned path may exist (checked by xtask).
*                                   @andrew

# ── Tier 1: the Canon. Changing these changes what the project is. ───────────
/docs/00-foundational-principles.md @andrew
/docs/01-canonical-terminology.md   @andrew
/docs/02-scope-guardrails.md        @andrew
/docs/adr/                          @andrew
/layers.toml                        @andrew
/.agents/policies/                  @andrew

# ── Tier 2: correctness-critical. Human review, and an ADR link in the PR. ───
/crates/domain/ledger/              @andrew
/crates/domain/economy/             @andrew
/crates/domain/identity/            @andrew
/crates/domain/agent/               @andrew
/crates/ports/                      @andrew
/crates/support/schema/             @andrew
/schemas/                           @andrew
/deny.toml                          @andrew
/Cargo.lock                         @andrew
/.github/workflows/                 @andrew
/infra/                             @andrew

# ── Tier 3: generated. Owned by the review group that owns the GENERATOR. ───
/packages/api-client/               @fractal/codegen
/packages/event-types/              @fractal/codegen
/packages/design-system/generated/  @fractal/codegen
/crates/**/src/generated/           @fractal/codegen
```

Three notes on why it is shaped this way.

**Tier 3 is the interesting one.** `@fractal/codegen` is a review group whose stated remit is: approve only if the diff is the mechanical output of a generator change that has itself been reviewed. Assigning generated paths to an owner group means a hand-edit produces a surprising review request — a signal a human will notice — in addition to the failing `lint-generated` check. Two independent detectors for the one rule most likely to be quietly broken under deadline pressure.

**`Cargo.lock` is Tier 2 deliberately.** `02 §5` budgets five new third-party runtime dependencies per phase, and P8 requires pinning and SBOMs. The lockfile diff is where a dependency actually enters the product. An agent may propose it; a human decides it. This is the cheapest possible implementation of the supply-chain half of P8.

**Everything has an owner, including files nobody thinks about.** `cargo xtask lint-owners` walks the tree and fails if any tracked path matches no `CODEOWNERS` rule. Unowned paths are where unreviewed change accumulates.

### 14.2 What CODEOWNERS does not do

It does not encode expertise, and it should not be read as a work-assignment map. It does not replace `40`'s review checklist. And it does not scale to a single human reviewing every diff at Phase 3 volume — which is a real, foreseeable bottleneck. The mitigation is `42`'s milestone-granular automation: agents land work on integration branches, CI enforces everything mechanizable (layer lint, codegen diff, targets, parity, budgets, sim, schema compatibility), and the human review is spent on the residue — the judgement calls that a lint cannot express. **The purpose of every mechanical check in this chapter is to make the scarcest resource on the project, human attention, land where it is irreplaceable.**

---

## 15. Fixtures and Test Data

**Decision: fixtures are generated by deterministic Rust code in `fractal-testkit`. Git LFS is not used. Committed fixture files are capped at 512 KB each and must be text or small binary golden artifacts.**

### 15.1 The three kinds

| Kind | Location | Form | Rule |
|---|---|---|---|
| **Generators** | `crates/support/testkit/src/gen/` | Rust functions and `proptest` strategies producing valid aggregates from a seed | The default. Anything expressible as code is code |
| **Golden files** | `fixtures/golden/` | Committed text: event-log snapshots, projection outputs, rendered CLI frames, OpenAPI diffs, ledger settlement reports | ≤ 512 KB each, and every one has a regenerating command in its header comment |
| **Seed scenarios** | `fixtures/seeds/` | Declarative TOML scenarios (`a Society with 3 Chambers, 40 messages, 2 Agents, a settled storage run`) replayed **through the real command path**, producing a real event log | Never a database dump. A dump encodes today's schema; a scenario encodes intent |

Seed scenarios are the load-bearing choice. A SQL dump would be faster to produce and would rot on the first schema change; replaying a scenario through the command bus means every fixture is, by construction, reachable by a real user action — and P6's replay guarantee is exercised every time the test suite runs.

### 15.2 Media fixtures without binaries

Transcoding, thumbnailing, and shard/erasure tests need real media. Real media is large and binary — exactly what must not enter the history.

```
fixtures/media.lock          # committed: {name, blake3, bytes, ffmpeg_args, source_seed}
        │
        │  cargo xtask fixtures
        ▼
  generate locally with the pinned ffmpeg from infra/docker/toolchain
        │
        ├── hash matches media.lock  ──►  cache in target/fixtures/, proceed
        └── hash differs             ──►  FAIL with both hashes and the ffmpeg version
```

Media is synthesized from a seed (test patterns, tone sweeps, generated frames), never downloaded and never committed. `media.lock` makes the output content-addressed, so a non-deterministic encoder change is a loud failure rather than a mysterious test flake. The pinned ffmpeg lives in the toolchain image, which is why determinism is achievable at all.

### 15.3 Why not Git LFS

| Argument for LFS | Why it does not hold here |
|---|---|
| Keeps large files out of the pack | It keeps them out of the *pack*, not out of the *clone*. LFS objects still download, and the cost becomes invisible — which is worse than visible, because nobody budgets for it |
| Standard tooling | It requires server-side storage configuration, an extra client install, and it interacts badly with `--filter=blob:none` partial clones and shallow clones. Agents clone this repository constantly; every clone would pay |
| Simple for contributors | It removes the pressure that produces the right answer. The real fix for "this fixture is 40 MB" is almost always "generate it", and LFS makes not-fixing-it easy |

**The honest cost of refusing LFS:** generation requires the pinned toolchain, so `cargo xtask fixtures` is slower on a cold machine (roughly 30–60 seconds) than downloading files would be, and a genuinely irreducible binary — a real-world file needed to reproduce a specific decoder bug — has no home. For that case: attach it to the issue, reference it by hash in a `#[ignore]`d test, and if it must be permanent, it goes to the content-addressed fixture bucket with an entry in `media.lock`. Never into git.

### 15.4 Enforcement

```yaml
# .github/workflows/hygiene.yml (excerpt)
- name: No large blobs
  run: |
    git diff --name-only origin/main...HEAD | while read -r f; do
      [ -f "$f" ] || continue
      size=$(wc -c < "$f")
      if [ "$size" -gt 524288 ]; then
        echo "::error file=$f::$size bytes exceeds the 512 KB fixture cap (41 §15)."
        echo "Generate it in fractal-testkit, or add it to fixtures/media.lock."
        exit 1
      fi
    done
```

A repository-wide budget is also checked weekly: total pack size, and the ten largest objects ever committed. History is expensive to fix and cheap to protect.

---

## 16. Trade-offs and Rejected Alternatives — Consolidated

Repository-topology alternatives are argued in §2.3 (polyrepo, mega-crate, nx/turborepo, Bazel); enforcement alternatives in §7.4; task-runner alternatives in §11.2; schema-source alternatives in §12.1; fixture alternatives in §15.3. The remaining structural choices, with their reasoning:

| Choice | Rejected alternative | Honest reasoning |
|---|---|---|
| One crate per bounded context | One crate per aggregate | Thirteen crates is already a long compile chain; ~50 would be worse for build time and would fragment invariants that belong together. A bounded context is the smallest unit with a coherent invariant set (`11 §1`) |
| One crate per bounded context | One `fractal-domain` crate with modules | Modules give no enforceable dependency edge. `mod ledger` could `use crate::society::*` and no tool would object. The crate boundary *is* the enforcement |
| Test doubles in `fractal-testkit` | Test doubles inside each adapter crate | Would force every consumer to compile the vendor SDK to get the fake, and would let a fake drift toward mirroring the vendor rather than the port |
| `apps/desktop/src-tauri` outside the workspace | Inside the workspace | Inside, `cargo check --workspace` requires platform SDKs on every machine, including agent containers. The cost — a second lockfile — is neutralized by a cheap consistency assertion (§10.3) |
| Generated *contracts* committed, generated *Rust* not | Commit everything / commit nothing | Committing everything makes review diffs enormous and merge conflicts constant. Committing nothing hides wire-breaking changes from human review and forces a Rust toolchain on JS-only consumers. The split is drawn at "who needs to read this in a PR" |
| `xtask` as one crate | One crate per task | Tasks share `fractal-schema`, config parsing, and process helpers; splitting them multiplies compile units for no isolation benefit. Revisit if `xtask` passes ~5,000 lines |
| Extensions in-tree (`extensions/`) | A separate marketplace repository | P7 requires first-party features to be built on the public API and shipped in lockstep with the hooks they need (`20`'s Hook Debt Rule). In-tree makes that atomic. Third-party extensions are, of course, out of tree |

---

## 17. What Would Make Us Change This

Stated in advance, in the manner of `10 §12`, so the signal is recognized rather than rationalized:

- **Clean CI exceeds 25 minutes at >120 crates with `sccache` above 80%.** → Reconsider Bazel by ADR (§2.3). Do not first try to fix it by splitting the repository; that trades a build problem for a coordination problem.
- **`cargo xtask bench-build` shows >15% of build time in feature-mismatched rebuilds at >40 crates.** → Adopt `cargo-hakari` (§9.4).
- **A domain crate genuinely cannot be expressed without a dependency outside the allowlist.** → That is a signal the need is a *port*, not a dependency. If it truly is not, the allowlist is amended by ADR with the transitive closure of the new entry stated in full. It is never amended in a feature PR.
- **A second human joins full-time.** → `CODEOWNERS` gains real per-area ownership and Tier 2 requires two approvals. Until then, more names in that file would be theatre.
- **The `parity` check becomes chronically red because a GUI-only surface is genuinely necessary.** → That is a P13 conflict, and P13 outranks P10 in `00 §2`. Escalate; do not weaken the check.
- **Generated TypeScript client diffs dominate PR review.** → Move `packages/api-client` to build-time generation and publish it to a registry for external consumers. Accept the loss of wire-change visibility in the diff only after `schema-check` is proven to catch what review was catching.

---

## 18. Invariants and Open Items

Each invariant is a CI assertion, in the manner of `11 §7`.

- **R1** No `fractal-domain-*` crate has a transitive dependency outside the `layers.toml` domain allowlist, on any supported target (§7.2 A2).
- **R2** No internal dependency edge violates `layers.toml` (§7.2 A1).
- **R3** `fractal-cli` and `fractal-agent` depend on no `app`, `api`, `adapter`, or `domain` crate (N3, P3).
- **R4** Every crate's name matches its path per `01 §9`, exceptions enumerated in `layers.toml` (§7.2 A4).
- **R5** Every third-party version is declared once, at the workspace root (§7.2 A3).
- **R6** `fractal-types`, `fractal-schema`, `fractal-ports`, every `fractal-domain-*`, and `fractal-core` compile to `wasm32-unknown-unknown`, `x86_64-unknown-linux-gnu`, and `aarch64-apple-darwin` on every PR (N2).
- **R7** `cargo xtask codegen tokens gen-lint-config agents` produces an empty diff (§12.2).
- **R8** No file bearing the `@generated` header differs from its generator's output (§10.4).
- **R9** Every API operation in `fractal-schema` has a corresponding CLI verb (P13, §12.3).
- **R10** No tracked file exceeds 512 KB; no tracked path lacks a `CODEOWNERS` owner (§14.1, §15.4).
- **R11** The `apps/desktop/src-tauri` lockfile agrees with the root lockfile on every shared package version (§10.3).
- **R12** Exactly one of `fractal-core`'s platform features is active in any build (§6.4).

**Proposed additions to `01-canonical-terminology.md`**, to be merged in the same PR as this chapter: **Layer** (one of `support`, `ports`, `domain`, `app`, `adapter`, `api`, `core`, `bin` — a crate's position in the dependency order, declared by its path); **Composition Root** (a `crates/bin/*` crate, the only place a port and a concrete adapter may be named together); **Generated Artifact** (a file produced by `cargo xtask`, carrying the `@generated` header, never hand-edited).

**ADRs required before Phase 0 implementation:**

1. Monorepo topology and the `layers.toml` contract (this chapter, ratified).
2. `async_trait` versus native async-fn-in-trait for `fractal-ports`, with an object-safety analysis and an expiry condition (§5.2).
3. `fractal-schema` as the single source of truth for the contract surface, with the protobuf field-numbering policy (§12.1).
4. The sandbox port addition to the `10 §7` list, required by `20` before Phase 3 (§5.5).
5. Licence policy for the `deny.toml` allowlist, including the position on MPL-2.0 and copyleft in linked dependencies (§7.3).
