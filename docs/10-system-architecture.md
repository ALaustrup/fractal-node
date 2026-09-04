# 10 — System Architecture

> **Prerequisites:** `00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`.
> **Governs:** service boundaries, the core runtime, the front-end contract, deployment topology, and the swappable-adapter surface.

---

## 1. The Shape of the System in One Diagram

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                                 FRONT ENDS                                    │
│   All peers. All speak the same public API. None wraps another. (P3, P13)     │
│                                                                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐  │
│  │  Web GUI │ │ Desktop  │ │  Mobile  │ │   CLI    │ │  Agent   │ │ Plugin │  │
│  │ (React)  │ │ (Tauri)  │ │(PWA→Nat.)│ │ fn/fract │ │ Runtime  │ │  Host  │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └───┬────┘  │
└───────┼────────────┼────────────┼────────────┼────────────┼───────────┼───────┘
        │            │            │            │            │           │
        └────────────┴────────────┴─────┬──────┴────────────┴───────────┘
                                        │
                        ┌───────────────▼────────────────┐
                        │        EDGE / GATEWAY          │
                        │  TLS · authn · Envelope authz  │
                        │  rate limit · quota · idempot. │
                        │  HTTP/JSON · gRPC · WebSocket  │
                        └───────────────┬────────────────┘
                                        │
╔═══════════════════════════════════════▼═══════════════════════════════════════╗
║                        THE RUNTIME  (one core, Rust)                          ║
║                                                                               ║
║  ┌─────────────────────────────────────────────────────────────────────────┐  ║
║  │                        APPLICATION LAYER                                │  ║
║  │  command handlers · query handlers · sagas · policy evaluation          │  ║
║  └───────────────────────────────┬─────────────────────────────────────────┘  ║
║                                  │                                            ║
║  ┌───────────────────────────────▼─────────────────────────────────────────┐  ║
║  │                          DOMAIN LAYER                                   │  ║
║  │   pure logic · zero I/O · zero vendor types · exhaustively tested       │  ║
║  │                                                                         │  ║
║  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │  ║
║  │  │ Society │ │Identity │ │ Ledger  │ │Progress │ │ Asset   │ │Governan│ │  ║
║  │  │         │ │ /Trust  │ │/Economy │ │ /Rep    │ │ (Facet) │ │ce/Chart│ │  ║
║  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘ └────────┘ │  ║
║  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐            │  ║
║  │  │Discourse│ │ Vault   │ │ Agent   │ │Extension│ │ Market  │            │  ║
║  │  │(Chamber)│ │(Storage)│ │/Envelope│ │         │ │         │            │  ║
║  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘            │  ║
║  └───────────────────────────────┬─────────────────────────────────────────┘  ║
║                                  │  PORTS (traits) — the only way out (P5)    ║
║  ┌───────────────────────────────▼─────────────────────────────────────────┐  ║
║  │                          ADAPTER LAYER                                   │ ║
║  │  EventStore · Ledger · BlobStore · Relay · KeyStore · Search             │ ║
║  │  Transcoder · ModelProvider · Rail · Chain · Clock · Rng · Telemetry     │ ║
║  └───────────────────────────────┬─────────────────────────────────────────┘  ║
╚══════════════════════════════════┼════════════════════════════════════════════╝
                                   │
     ┌─────────────┬───────────────┼────────────────┬──────────────┐
     ▼             ▼               ▼                ▼              ▼
┌─────────┐  ┌──────────┐   ┌───────────┐   ┌───────────┐  ┌──────────┐
│Postgres │  │  Object  │   │   NATS    │   │  Meili/   │  │  Model   │
│ (events │  │  Store   │   │ JetStream │   │  Tantivy  │  │ Provider │
│  + proj)│  │(S3→P2P)  │   │  (bus)    │   │ (search)  │  │  (LLM)   │
└─────────┘  └──────────┘   └───────────┘   └───────────┘  └─────┬────┘
                                                                 │
  THREE stateful systems, not four. Presence is Relay-process     │
  memory at a 45 s TTL, gossiped over NATS (61 X3).               │
                              ┌──────────────────────────────────-┘
                              ▼
                    ┌───────────────────┐
                    │  FUTURE: FN L1    │  ← swap behind Ledger + Chain traits.
                    │  external chains  │     No domain code changes. (P11)
                    └───────────────────┘
```

Read the double-walled box as the thing that must stay pure. Everything outside it is replaceable.

---

## 2. Why a Modular Monolith (and when it stops being one)

**Decision:** the Runtime ships as a **single deployable binary** containing internally-bounded modules, with a hard dependency-direction lint between them, until measured load forces extraction.

**Rationale.** Microservices buy independent scaling and independent deployment at the cost of distributed transactions, network partitions between things that used to be a function call, an order-of-magnitude increase in operational surface, and — most damaging at this stage — *premature boundary commitment*. We do not yet know where the true seams are. A modular monolith with enforced internal boundaries gives us the discipline of services with the debuggability of a process.

**The extraction rule.** A module is extracted into its own service when **two** of these are true, measured, not predicted:

1. It has a resource profile that differs by >5× from the rest (CPU, memory, GPU, egress).
2. It has a failure-isolation requirement (its outage must not take the platform down).
3. It has a deploy cadence that differs by >5× (e.g. shipping many times a day while the core ships weekly).
4. It has a distinct security boundary requiring OS-level isolation.

**Predicted extraction order** (do not pre-build): ① Media Transcoder (criterion 1 — GPU/CPU spike), ② Relay/SFU (1 and 2 — egress and isolation), ③ Agent Executor (1, 2, 4 — untrusted-ish work), ④ Search Indexer (1 and 3), ⑤ Custodian Coordinator (1). Everything else stays in the Runtime, possibly forever.

**Alternatives considered.** *Microservices from day one* — rejected: the complexity budget in `02` would be consumed by infrastructure before the spine sentence is true. *Serverless functions* — rejected: the Runtime is stateful (event log, subscriptions, presence, agent sessions) and cold starts are irreconcilable with P10's startup budget. *Actor system as the top-level architecture (e.g. a full Erlang-style topology)* — rejected as the *primary* structure but adopted *within* the Relay and Agent Executor, where it genuinely fits.

---

## 3. Service Boundaries

Each boundary below is a Rust module with its own crate, its own events, and no direct access to another module's tables. Cross-module reads go through a published query interface; cross-module writes go through events.

| # | Boundary | Owns | Emits (examples) | Never touches |
|---|---|---|---|---|
| S1 | **Identity** | Citizens, FNIDs, Handles, devices, sessions, key material references | `CitizenRegistered`, `DeviceEnrolled`, `HandleClaimed` | Society internals |
| S2 | **Society** | Societies, Memberships, Chambers, Charters, Lineage | `SocietyCreated`, `MemberJoined`, `SocietyFractured` | Ledger internals |
| S3 | **Discourse** | Threads, Messages, reactions, presence, typing, read state | `ChamberMessagePosted`, `ThreadResolved` | Wallets |
| S4 | **Vault** | Objects, Manifests, Shards, Custodians, Attestations, versions, ACLs | `ObjectStored`, `ShardAttested`, `ReplicaLost` | Governance |
| S5 | **Ledger** | Wallets, Postings, Transfers, Stakes, Emission accounting | `PostingRecorded`, `TransferSettled`, `StakeSlashed` | Message content |
| S6 | **Economy** | Sources, Sinks, Contribution Scores, emission policy, settlement runs | `ContributionScored`, `EmissionExecuted`, `SinkBurned` | Direct wallet writes (goes through S5) |
| S7 | **Progression** | XP, Levels, Trust, Standing, Achievements, Unlocks, Seasons | `XpAwarded`, `LevelReached`, `TrustAdjusted`, `UnlockGranted` | Fraction balances |
| S8 | **Asset** | Facets, Facet Standards, evolution state, licenses, provenance | `FacetMinted`, `FacetEvolved`, `LicenseGranted` | Chat |
| S9 | **Governance** | Charters, roles, proposals, votes, enactments, moderation, appeals | `ProposalOpened`, `CharterEnacted`, `ModerationActionTaken` | Ledger internals |
| S10 | **Agent** | Agents, Envelopes, Policies, Workflows, runs, audit trail | `EnvelopeGranted`, `WorkflowInvoked`, `AgentActionBlocked` | Anything outside its Envelope |
| S11 | **Extension** | Extension registry, Installs, versions, permissions, sandbox | `ExtensionInstalled`, `ExtensionPermissionRequested` | Domain internals except via hooks |
| S12 | **Market** | Listings, ratings, purchases, licenses, payouts, revenue share | `ListingPublished`, `PurchaseCompleted`, `PayoutIssued` | Ledger internals (uses S5) |
| S13 | **Discovery** | Interest declarations, Convergences, matching, search index feeds | `ConvergenceOpened`, `InterestDeclared` | Private content |
| S14 | **Relay** | Signal subscriptions, fan-out, presence, realtime transport, SFU control | (transport, not domain) | Persistence |
| S15 | **Atlas** | The cross-Society read models — Citizen unified inbox, cross-Society search index, marketplace statistics, and the **Shard reference count** that governs GC (`13 §10.2`) | (projections, not domain) | **Writes of any kind.** Emits no domain event, holds no Wallet, takes no lock |

**S15 is read-only and never authoritative.** It is the one boundary that reads across partitions, and it exists because P1's per-Society partitioning charges for itself at exactly one place: reads that span Societies. Its guarantees, stated once and not negotiable per-caller:

- **Eventually consistent, monotonic per reader.** A reader never observes Atlas at a lower frontier than one they have already observed; an `atlas_frontier` vector rides every request and response.
- **Staleness bound ≤ 5 s at p99, ≤ 30 s at p99.9**, published as an SLO beside `40 §9.4`'s 2 s projection lag. Every response carries `as_of` and `stale`.
- **Never consulted by a command handler.** `16 §5`'s prohibition on reading a projection to make a decision applies to Atlas absolutely. Where Atlas and a Society's Log disagree, the Log wins and Atlas is rebuilt.
- **The Shard refcount is monotone-safe:** over-count permitted, under-count forbidden. Collection requires *positive confirmation of non-reference* from every Society that has ever referenced the hash, at that Society's current `seq`. The absence of a reference is never sufficient, because `13 §10.2` is explicit that an under-counting refcount destroys data.

Like S14, S15 has **no domain crate** — it owns projections, not invariants (`41 §5.4`, `61 N15`). It runs inside `fractal-app-projection` from PH1 and extracts as a service at PH5.

**The dependency rule, enforced in CI:**

```
front ends ──► gateway ──► application ──► domain ──► ports(traits)
                                                          ▲
                                              adapters ────┘  (implement, never import domain policy)

FORBIDDEN:  domain ──► adapters
FORBIDDEN:  domain(A) ──► domain(B) internals   (only published query traits + events)
FORBIDDEN:  front end ──► anything but gateway
```

---

## 4. The Society as the Sharding Unit

P1 is not only a mental model — it is the physical partitioning strategy, and this is where the architecture pays for itself.

```
        ┌──────────────────────── SOCIETY PARTITION ────────────────────────┐
        │  society_id = 0x7f3a…                                             │
        │                                                                   │
        │   ┌───────────────┐    append-only, per-society ordering only     │
        │   │  EVENT LOG    │──────────────────────────────────────────┐    │
        │   │  seq 1..N     │                                          │    │
        │   └───────┬───────┘                                          │    │
        │           │ fan-out                                          │    │
        │   ┌───────┴────────┬─────────────┬──────────────┐            │    │
        │   ▼                ▼             ▼              ▼            │    │
        │ ┌──────────┐ ┌──────────┐ ┌───────────┐ ┌────────────┐       │    │
        │ │Discourse │ │  Ledger  │ │Progression│ │  Search    │       │    │
        │ │projection│ │projection│ │projection │ │  index     │       │    │
        │ └──────────┘ └──────────┘ └───────────┘ └────────────┘       │    │
        │                                                              │    │
        │   ┌────────────┐  ┌──────────┐  ┌───────────┐                │    │
        │   │  TREASURY  │  │  VAULT   │  │  CHARTER  │                │    │
        │   └────────────┘  └──────────┘  └───────────┘                │    │
        │                                                              │    │
        │   ANCHOR: every N events or T seconds, commit the log's ─────┘    │
        │           merkle root to the Ledger  →  later, to FN L1            │
        └───────────────────────────────────────────────────────────────────┘
```

**Consequences of this choice, all of them good:**

- **Ordering.** We need total order *within* a Society and only causal order *between* Societies. This is the single most important scalability decision in the document: it removes global consensus from the hot path entirely.
- **Sharding.** Route by `society_id`. A hot Society moves to its own partition without touching anything else.
- **Fracture.** Splitting a Society is a well-defined operation on one partition's log, treasury, vault, and membership — not a distributed migration across the whole platform.
- **Self-hosting.** A Society's entire state is a portable, verifiable bundle. "Take your Society and leave" is an export, not a negotiation.
- **Backup and audit.** Per-Society, per-tenant, independently restorable.
- **Blast radius.** A corrupt projection affects one Society.

**The cost, stated honestly, and now owned:** cross-Society operations (Federation, global search, marketplace-wide statistics, a Citizen's unified inbox) are *harder* — they require a fan-out read or a global projection maintained by consuming many logs. We accept this. It is the correct trade because cross-Society operations are rarer, more tolerant of eventual consistency, and mostly read-only. **That cost has an owner: S15 Atlas (§3).** It was named as a cost in three chapters and designed in none until `61 W9`; "stated as a cost" is not a substitute for "designed as a subsystem", and the Shard refcount in particular is a cross-partition projection whose under-count is data loss. See `60-self-critique.md` §3.9 and `61 W9`.

---

## 5. The Event Model

```
   COMMAND                    DOMAIN                     EVENT                PROJECTIONS
 ┌──────────┐            ┌──────────────┐          ┌─────────────┐         ┌────────────┐
 │PostChamb-│            │ load agg.    │          │ChamberMess- │────────►│ Discourse  │
 │erMessage │───────────►│ check policy │─────────►│agePosted    │    │    │ read model │
 │          │            │ check Envelope│         │ seq: 4471   │    │    └────────────┘
 └──────────┘            │ decide       │          │ society: …  │    │    ┌────────────┐
      │                  └──────────────┘          │ actor: …    │    ├───►│ Search idx │
      │ idempotency_key                            │ causation:… │    │    └────────────┘
      │ correlation_id                             └─────────────┘    │    ┌────────────┐
      └──────────────────────────────────────────────────────────────►├───►│ XP/Progress│
                                                                      │    └────────────┘
                                                                      │    ┌────────────┐
                                                                      └───►│ Signal fan-│
                                                                           │ out (Relay)│
                                                                           └────────────┘
```

**Event envelope (every event, no exceptions):**

```rust
struct EventEnvelope {
    society_id:     SocietyId,        // P1 — always present, or GLOBAL sentinel
    seq:            u64,              // per-society monotonic
    event_id:       Ulid,
    kind:           &'static str,     // "discourse.message.posted.v1"
    schema_version: u16,
    occurred_at:    Timestamp,        // domain time
    recorded_at:    Timestamp,        // wall time
    actor:          Principal,        // Citizen | Agent | Society | Node | System
    on_behalf_of:   Option<Principal>,// agent acting for a citizen
    envelope_ref:   Option<EnvelopeId>,// which grant authorized this (P4 audit)
    correlation_id: Ulid,             // the user-visible operation
    causation_id:   Ulid,             // the event/command that caused this
    payload:        Bytes,            // versioned, schema-registered
    integrity:      Hash,             // chained: H(prev_hash || payload)
}
```

`envelope_ref` is the field that makes P4 auditable. Every action an Agent takes names the grant that permitted it, and every grant traces to a human signature. Without this field the principle is a slogan.

**Event evolution rules:** additive only within a version (new optional fields). A breaking change means a new `.v2` kind plus an upcaster registered in the replay path. Old events are **never rewritten** — they are historical fact. `40-engineering-standards.md` specifies the schema registry and the compatibility test that runs on every PR.

**Idempotency.** Every command carries a client-generated `idempotency_key`. The application layer dedupes on `(principal, idempotency_key)` for 24 hours. This is what makes the CLI, agents, and flaky mobile networks safe to retry.

---

## 6. Local-First Synchronization (P2)

```
   ┌──────────────────── DEVICE (Node) ─────────────────────┐
   │                                                        │
   │  UI ──► Local Store (SQLite) ──► Outbox (pending cmds) │
   │           ▲          │                    │            │
   │           │          │ optimistic         │            │
   │           │          ▼ apply              ▼            │
   │           │   ┌─────────────┐      ┌─────────────┐     │
   │           └───│  Replicated │      │  Sync Engine│     │
   │               │  Event Log  │◄─────│             │     │
   │               │ (per society)│      └──────┬──────┘    │
   │               └─────────────┘             │            │
   └───────────────────────────────────────────┼────────────┘
                                               │
                    pull: since(seq)  ─────────┤
                    push: outbox     ◄─────────┤
                    live: Signal WS  ◄─────────┘
                                               │
                                      ┌────────▼────────┐
                                      │  Runtime (auth. │
                                      │   event log)    │
                                      └─────────────────┘
```

**Conflict policy, per data class — chosen deliberately, not uniformly:**

| Data class | Strategy | Why |
|---|---|---|
| Chamber messages | Append-only, server-assigned `seq` | Messages never conflict; only order needs deciding |
| Message edits | Last-writer-wins on `(edited_at, device_id)` | Simple, matches user expectation |
| Reactions, read state, presence | CRDT (OR-Set / LWW-Register) | Genuinely concurrent, must never lose a user's action |
| Collaborative documents / canvas | CRDT (Yjs-compatible text/map) | Real concurrent editing |
| Wallet balances | **Server-authoritative. No optimistic write. Ever.** | Money does not do eventual consistency (P12) |
| Facet state | Server-authoritative | Ownership is not negotiable |
| Governance votes | Server-authoritative, signed | Integrity requirement |
| Profile, settings, drafts | LWW per field | Single-user data |

The wallet row above is the one people get wrong. Reads of a balance are local and instant and clearly marked with their staleness; *writes* are never optimistic. A transfer shows `PENDING` until settled. This is honest and it is what a financial instrument does.

---

## 7. The Port Surface (P5 — the complete swappable list)

Every one of these is a Rust trait in `fractal-ports`. Each has at least two implementations at introduction (real + test double). No domain crate may reference a concrete implementation.

| Port | Phase 1 impl | Later impls | Why abstracted |
|---|---|---|---|
| `EventStore` | Postgres | FoundationDB, per-society files | Partitioning strategy will evolve |
| `Ledger` | Internal double-entry (Postgres) | FN L1, external chain | **The core P11 commitment** |
| `Chain` | `NullChain` (anchors to Ledger) | FN L1, EVM, SVM adapters | Chain-agnostic by decree |
| `BlobStore` | S3-compatible | Custodian mesh (P2P) | Distribution is an implementation detail |
| `Relay` | In-process + WebSocket | NATS-backed, SFU | Extraction candidate |
| `KeyStore` | OS keychain / libsodium | HSM, secure enclave, passkey | Security posture will harden |
| `Search` | Postgres FTS | Tantivy, Meilisearch | Cost/quality trade shifts with scale |
| `Transcoder` | ffmpeg local | GPU farm, third party | Extraction candidate |
| `ModelProvider` | Hosted API | Local models, per-Society choice | Sovereignty requirement |
| `Rail` | Internal FRC only | Fiat processor, chain bridge | Regulatory gating |
| `Clock`, `Rng`, `IdGen` | System | Deterministic | **Required for replayable tests** |
| `Telemetry` | OpenTelemetry | any OTLP sink | Vendor independence |

`Clock`, `Rng`, and `IdGen` being ports is not pedantry. It is what makes the entire domain layer deterministically testable and the event log replayable — which is what makes P6 and P12 verifiable rather than aspirational.

---

## 8. The Agent Boundary (P4 in architecture)

```
 ┌──────────┐   authors    ┌──────────┐   constrains   ┌───────────────┐
 │ CITIZEN  │─────────────►│  POLICY  │───────────────►│   ENVELOPE    │
 │ (human)  │  signs        │(human    │                │ (capabilities,│
 └──────────┘               │ only)    │                │  limits, TTL) │
      ▲                     └──────────┘                └───────┬───────┘
      │                                                          │ grants
      │ confirmation required for                                ▼
      │ any action outside pre-approved class            ┌───────────────┐
      │                                                  │     AGENT     │
      └──────────────────────────────────────────────────┤  (executes)   │
                                                         └───────┬───────┘
                                                                 │ every action
                                                                 ▼
                                            ┌────────────────────────────────┐
                                            │  POLICY ENFORCEMENT POINT      │
                                            │  in the APPLICATION layer —    │
                                            │  not in the agent, not in the  │
                                            │  gateway, not in the front end │
                                            └────────────────┬───────────────┘
                                                             │
                                        ┌────────────────────┴──────────────┐
                                        ▼                                   ▼
                                  ALLOW → execute,                    DENY → AgentActionBlocked
                                  emit event with                     event, surfaced to the
                                  envelope_ref                        Operator, counted against
                                                                      the Agent's Trust
```

**The architectural commitment:** the Policy Enforcement Point lives in the application layer, *inside* the trust boundary, on the path every command takes regardless of front end. An agent cannot route around it because there is no other route. A blocked action is a first-class domain event — not a log line — so it is auditable, alertable, and reputationally consequential.

---

## 9. Deployment Topology

**Phase 1–3 (hosted, single region):**

```
   Cloudflare ──► Load Balancer ──► Runtime (3+ replicas, stateless)
                                        │
                       ┌────────────────┼────────────────┐
                       ▼                ▼                ▼
                  Postgres         Object Store       NATS
                  (primary +       (S3-compatible)   JetStream
                   replica)
```

**Phase 4–6 (multi-region + Custodian mesh):** Runtime replicas per region; Societies pinned to a home region with read replicas elsewhere; Custodian mesh handles Shard distribution; Relay extracted; Transcoder extracted.

**Phase 6+ (self-hosted Nodes):** the same binary runs as a personal or organizational Node, joining the Fractal Net for discovery and Custodianship, holding full local replicas of its Societies. The desktop app *is* a Node with a GUI — this is why the desktop shell embeds the Runtime rather than being a thin client.

---

## 10. Cross-Cutting Concerns

| Concern | Mechanism |
|---|---|
| **AuthN** | Passkeys (WebAuthn) primary; device-bound keypairs; OIDC as an optional adapter. No passwords in the primary flow. |
| **AuthZ** | Envelope evaluation at the Policy Enforcement Point. Deny by default (P8). Cached per request, never per session. |
| **Rate limiting** | Per-principal token buckets at the gateway; per-Society quotas; separate, tighter buckets for Agents. |
| **Idempotency** | `idempotency_key` on every command, 24h dedupe window. |
| **Observability** | OpenTelemetry traces carrying `correlation_id` from the front end through to the event. Every domain event is a span. RED metrics per boundary. |
| **Backpressure** | Bounded channels everywhere; shed load at the gateway with `429` + `Retry-After`, never by silently dropping. |
| **Secrets** | Never in events, never in logs, never in telemetry. A lint denies serializing types marked `#[secret]`. |
| **Time** | All timestamps UTC, RFC3339 with explicit precision. Domain time (`occurred_at`) and wall time (`recorded_at`) are distinct fields and are never conflated. |
| **Schema registry** | Every event kind and API type registered with a compatibility test in CI. |

---

## 11. Technology Choices and Their Trade-offs

Each of these becomes a numbered ADR in `docs/adr/`. Summarized here for coherence.

| Choice | Serves | Why | Honest cost | Alternatives rejected |
|---|---|---|---|---|
| **Rust** for the Runtime | P8, P10, P13 | Memory safety removes an entire defect class; one core compiles to server, desktop, and `wasm32` for the browser — which is *how* P13 becomes cheap rather than aspirational; fearless concurrency for the Relay | Slower initial velocity; smaller hiring pool; long compile times (mitigated by workspace splitting + `sccache`) | Go (no wasm story for shared core, GC pauses in the Relay); TypeScript everywhere (no memory/type guarantees at the ledger); Elixir (superb for Relay, weak for the deterministic ledger and wasm core) |
| **Postgres** as the primary store | P6, operability | One system for events, projections, FTS, and queues at Phase 1 scale; strongest operational knowledge base of any database; logical decoding gives us CDC for free | Will need partitioning by `society_id` earlier than comfortable; not a natural event store | Kafka + separate store (two systems, more ops, no transactional read-model updates); EventStoreDB (small ecosystem); FoundationDB (excellent, but the operational learning curve costs a phase) |
| **NATS JetStream** for the bus | P6, extraction path | Lightweight, embeddable, subject hierarchy maps exactly to `society.<id>.<boundary>.<event>`, at-least-once with replay | Another system to operate | Kafka (heavy for our volume); Postgres LISTEN/NOTIFY (no durability, no replay); Redis Streams (weaker delivery guarantees) |
| **React + TypeScript + Vite** for web | Phase 1 speed | Largest talent pool, mature accessibility tooling, best-in-class dev experience; Phase 1 must ship a *working* GUI | Bundle discipline required to hold P10 budgets | Svelte/SolidJS (better runtime, smaller ecosystem and hiring pool); Leptos/Dioxus (Rust-native and tempting, but immature a11y tooling would jeopardize N8) |
| **Tauri v2** for desktop | P2, P10, P13 | Embeds the Rust Runtime *in-process* — the desktop app is a real Node, not a thin client; ~10MB binaries; native webview | Webview inconsistency across OSes; some native APIs need plugins | Electron (150MB+, no Rust core embedding, worse startup); native per-OS (3× UI cost, violates the shared-core goal) |
| **WebRTC + SFU** for voice/video | `14` | The only viable path to low-latency many-party media in a browser | SFU operation is a permanent cost and expertise centre | Full mesh (breaks past ~5 peers); third-party SDK (vendor lock, E2EE compromise) |
| **MLS (RFC 9420)** for E2EE groups | P8, N6 | Purpose-built for large dynamic groups with forward secrecy and post-compromise security — the exact shape of a Society | Young ecosystem; implementation care required | Signal Double Ratchet (superb 1:1, poor scaling to large groups); Olm/Megolm (weaker PCS guarantees) |
| **WASM (Component Model)** for plugins | P7, P8 | Capability-secure by construction, language-agnostic, deterministic, resource-metered | Host bindings are work; the component tooling is still maturing | JS sandbox (weak isolation); containers (heavy, wrong granularity); native dylibs (no isolation at all — non-starter) |
| **OpenTelemetry** | Observability | Vendor-neutral, one instrumentation for traces/metrics/logs | Overhead requires sampling discipline | Vendor SDKs (lock-in, contradicts P5) |

---

## 12. What Would Make Us Change This Architecture

Stated in advance so the team recognizes the signal rather than rationalizing it:

- **Per-Society ordering proves insufficient** because a genuinely global feature (a unified cross-Society inbox at scale, or Federation with strong consistency) becomes core. → Introduce a global sequencer for a narrow event subset; do *not* globalize the whole log.
- **Postgres becomes the bottleneck before Phase 5.** The remedy ladder, in order, with what each step actually buys — because the first two buy latency and read capacity and **neither adds write throughput**, which is what a bottleneck on the posting path means:

```
 1 PARTITION            hash-partition the event and projection tables by
                        society_id, 64 partitions
   buys                 index depth, vacuum cost, locality, per-Society DROP
   DOES NOT BUY         write throughput — one instance, one WAL, one writer
   trigger              event table exceeds the primary's buffer-cache working
                        set, OR autovacuum duty cycle on it exceeds 25%

 2 OFFLOAD READS        projections to physical replicas and logical-decoding CDC
   buys                 read capacity
   DOES NOT BUY         write throughput
   trigger              replica share of total reads falls below 60%

 3 SHARD ACROSS PRIMARIES        ← the step that actually raises write capacity
                        N independent primaries; Societies mapped by rendezvous
                        hash on society_id; routing in the COMPOSITION ROOT
                        (fractal-node) — never in the domain, never in the app layer
   buys                 write throughput, linearly in N
   trigger (measured)   sustained posting-path write TPS exceeds 60% of the
                        MEASURED single-primary ceiling for 7 consecutive days,
                        OR p99 posting latency exceeds 50 ms at that load

 4 REPLACE THE ENGINE   EventStore -> FoundationDB or per-Society segment files
   trigger              only after step 3 is executed and measured
```

  **What step 3 does to the single-transaction property.** ADR-0006 exists to guarantee that the event append and the projection update commit together, and that guarantee was always *per Society* — a Society lives wholly inside one shard, so it survives sharding untouched. What does not survive is any transaction spanning two Societies. The corpus contains exactly one candidate, **Fracture**, and `11 §3.2` already specifies it as one transaction *per child* with the parent's log sealed first, so it is shard-safe by construction; every other cross-Society operation is a saga with compensation (`11 §5`). Two consequences must be planned rather than discovered: the **Global Registry** (`01 §6`) is not per-Society and moves to its own primary, hash-partitioned by FNID prefix, at step 3; and **cross-shard reads go through S15 Atlas (§3), never through a join**.

  **What must be true in the code first, so the shard is cheap.** (1) No SQL outside `fractal-adapter-postgres` — already lint-enforced. (2) Every `EventStore` and `Ledger` port method carries a routable key — a `society_id`, or a `WalletId` that resolves to one — in its signature; **audited at PH1**, because one keyless method is what forces a scatter-gather later. (3) No query joins across `society_id`. (4) The `EmissionAccount` shards use the same hash and modulus as the router (K = 64), so a settlement Posting is always same-primary as its Society. (5) A `ShardRouter` exists in the composition root from PH1 with one shard configured, so the seam is exercised continuously — the discipline `13 §11.4` already applies to `BlobStore`. (6) PH2 runs the full suite against a two-primary composition root.

  Partitioning first is still correct; it is simply not the answer to a write-throughput question, and ADR-0006 §3 says so two sections before ADR-0006 §9 used to prescribe it as one (`61 W4`).
- **Tauri's webview inconsistency threatens the P10 budget on Windows.** → Evaluate a Rust-native render path (Dioxus/Freya) for the desktop shell only, keeping the same design tokens.
- **MLS implementations prove immature at Society scale (>1,000 leaves ≈ 400 Citizens at the published 2.5 devices/Citizen).** Every MLS figure in the corpus is stated in **leaves**, because leaves are what the mechanism consumes and members are not (`14 §4.2`, `61 X10`). → Fall back to sender-key groups with periodic rekeying and publish the reduced guarantee honestly.
- **WASM component tooling stalls.** → Ship first-party extensions natively and delay third-party execution rather than weakening isolation. P8 outranks P7.
