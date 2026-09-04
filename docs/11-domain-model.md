# 11 — Domain Model

> **Prerequisites:** the Canon (`00`, `01`, `02`) and `10-system-architecture.md`.
> **Governs:** entities, aggregates, invariants, state machines, and the lifecycle operations that define what Fractal Node *is*.

---

## 1. The Aggregate Map

An **aggregate** is a consistency boundary: everything inside it changes together, in one transaction, under one set of invariants. Everything between aggregates is eventually consistent and coordinated by events.

```
GLOBAL (the nine exceptions from 01 §6)
├── Citizen ──────────┐
├── Handle            │
├── Society (registry)│
├── Node              │
├── Extension         │
└── FacetStandard     │
                      │
                      │ member of
                      ▼
╔══════════════════════════════════════════════════════════════════════════╗
║  SOCIETY  ← the atomic container; every aggregate below is scoped to it  ║
║                                                                          ║
║  ┌────────────────────┐  ┌────────────────────┐  ┌────────────────────┐  ║
║  │ Charter            │  │ Membership         │  │ Chamber            │  ║
║  │ (governance,       │  │ (Citizen ↔ Society,│  │ (a space; owns     │  ║
║  │  roles, params)    │  │  role, Standing)   │  │  Threads)          │  ║
║  └────────────────────┘  └────────────────────┘  └─────────┬──────────┘  ║
║                                                            │             ║
║                                                  ┌─────────▼──────────┐  ║
║                                                  │ Thread → Message   │  ║
║                                                  └────────────────────┘  ║
║  ┌────────────────────┐  ┌────────────────────┐  ┌────────────────────┐  ║
║  │ Treasury (Wallet)  │  │ Vault              │  │ AgentEnrollment    │  ║
║  │                    │  │  └ Object          │  │  └ Envelope        │  ║
║  │                    │  │     └ Manifest     │  │                    │  ║
║  │                    │  │        └ Shard     │  │                    │  ║
║  └────────────────────┘  └────────────────────┘  └────────────────────┘  ║
║  ┌────────────────────┐  ┌────────────────────┐  ┌────────────────────┐  ║
║  │ Facet (asset)      │  │ ExtensionInstall   │  │ Proposal / Vote    │  ║
║  └────────────────────┘  └────────────────────┘  └────────────────────┘  ║
║  ┌────────────────────┐  ┌────────────────────┐                          ║
║  │ StandingRecord     │  │ ModerationAction   │                          ║
║  └────────────────────┘  └────────────────────┘                          ║
╚══════════════════════════════════════════════════════════════════════════╝

PRE-SOCIETY
└── Convergence (ephemeral; crystallizes into a Society or expires)
```

**Aggregate sizing rule.** An aggregate is as small as its invariants allow. `Message` is not inside `Chamber` as a collection — a Chamber with a million messages cannot be loaded to append one. `Chamber` owns *policy*; `Thread` owns *ordering*; `Message` is an event with a projection.

---

## 2. Core Entities

### 2.1 Citizen (global)

```rust
struct Citizen {
    fnid:            Fnid,             // Ed25519 pubkey → base32, never changes
    handle:          Handle,           // @name, globally unique, immutable after 14d
    display_name:    String,
    created_at:      Timestamp,
    devices:         Vec<DeviceId>,    // enrolled keys
    recovery:        RecoveryConfig,   // social recovery, see 12
    global_wallet:   WalletId,
    xp:              Xp,               // monotonic, global
    level:           Level,            // derived from xp; starts at 0
    trust:           Trust,            // global, bidirectional, decays toward neutral
    interests:       Vec<InterestTag>, // DECLARED, never inferred (P9)
    profile:         ProfileId,
    status:          CitizenStatus,    // Active | Dormant | Suspended | Departed
}
```

**Invariants**
- `fnid` is derived from a public key and is immutable for the life of the Citizen. Key rotation changes the key, not the FNID (rotation is recorded as a signed chain).
- `handle` is unique across the platform, case-folded, and reserved for 12 months after `Departed`.
- `level` is a pure function of `xp` (`18 §3`). It is never set directly.
- `trust` cannot be purchased, transferred, or granted by an Agent.
- `interests` may only be written by the Citizen. No process may append an inferred interest. **This is P9 encoded as an invariant.**

---

### 2.2 Society (the atomic container)

```rust
struct Society {
    society_id:      SocietyId,
    sigil:           Sigil,            // procedurally derived from society_id
    name:            String,
    handle:          SocietyHandle,    // globally unique
    charter:         CharterVersion,   // current enacted version
    treasury:        WalletId,
    vault:           VaultId,
    lineage:         Lineage,          // origin + ancestors
    level:           Level,
    reputation:      SocietyReputation,
    member_count:    u32,              // projection, denormalized
    visibility:      Visibility,       // Public | Discoverable | Private | Sealed
    status:          SocietyStatus,
    created_at:      Timestamp,
    home_region:     RegionId,
}

enum Visibility { Public, Discoverable, Private, Sealed }
//  Public       listed, readable by anyone, joinable per Charter
//  Discoverable listed in discovery, contents hidden until joined
//  Private      unlisted, invite only
//  Sealed       unlisted, invite only, no new members, archive-only

enum SocietyStatus { Forming, Active, Dormant, Fracturing, Dissolving, Archived }

struct Lineage {
    origin:    Origin,             // Founded | Crystallized(ConvergenceId)
                                   // | Fractured{parent, sibling_ids}
                                   // | Forked{parent}
    ancestors: Vec<SocietyId>,     // root-first, capped depth 32
    generation: u16,
}
```

**Invariants**
- Exactly one enacted `CharterVersion` at any time.
- `treasury` and `vault` are created atomically with the Society and are never reassigned.
- `member_count >= 1` while `Active` — a Society always has at least one member with the founding role, or it transitions to `Dormant`.
- `lineage.ancestors` is append-only and immutable.
- A `Sealed` Society accepts no new events except archival ones.

---

### 2.3 Charter (governance as data)

The Charter is the Society's constitution, expressed as a machine-evaluable document. This is the mechanism by which a Society is genuinely sovereign: the rules are *its* data, not our hardcoded product decisions.

```rust
struct Charter {
    version:        u32,
    society_id:     SocietyId,
    roles:          Vec<Role>,
    capabilities:   BTreeMap<RoleId, CapabilitySet>,
    joining:        JoinPolicy,       // Open | Application | Invite | Stake(amount) | Vouch(n)
    governance:     GovernanceModel,
    economy:        EconomyParams,    // fee splits, treasury rules, reward weights
    moderation:     ModerationPolicy,
    agent_policy:   AgentPolicy,      // P4: what agents may do HERE
    fracture_rules: FractureRules,
    amendment:      AmendmentRule,    // how this document changes
    enacted_at:     Timestamp,
    enacted_by:     Vec<Signature>,   // human signatures (P4)
}

enum GovernanceModel {
    Founder,                                   // Level 0–2 default
    Council { seats: u8, selection: Selection },
    Direct  { quorum: Ratio, threshold: Ratio },
    Delegated { delegation_depth: u8 },
    Custom  { extension: ExtensionId },        // Phase 6+, sandboxed
}
```

**Invariants**
- `enacted_by` must contain signatures from Citizens (never Agents) satisfying `amendment` (P4).
- A Charter amendment cannot retroactively invalidate events already recorded.
- `agent_policy` cannot grant an Agent a capability the granting role does not itself hold. **No privilege escalation via delegation.**
- `economy` parameters are bounded by platform-global limits (`17 §7`). A Society cannot mint Fraction.

**Governance unlocks by Society Level** — a Society earns autonomy rather than receiving it (see `18`):

| Society Level | Unlocked |
|---|---|
| 0 | Founder governance, 1 Chamber, 25 members |
| 1 | Roles, 5 Chambers, 100 members, Treasury spending |
| 2 | Council governance, custom Charter clauses, 500 members |
| 3 | Direct voting, custom sigil, Extension installs, Facet minting |
| 4 | Delegated governance, Federation, Experience hosting (Phase 7) |
| 5 | **Fracture**, self-hosted Node, custom economic parameters, unbounded members |

Member ceilings, completed — `18 §5.2` expands the storage and Chamber grants against the same ladder:

| SL | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| **Members** | 25 | 100 | 500 | **2,000** | **10,000** | **unbounded** |

The E2EE ceiling is **orthogonal to this ladder and does not scale with it**: any single `EndToEnd` Chamber holds at most **1,000 MLS leaves** (≈400 Citizens at the published 2.5 devices/Citizen, `14 §4.2`). A Society above that size simply cannot make one Chamber E2EE for everybody, and the client says so at Chamber creation rather than at message-send time (`14 §4.4`, `61 X10`). Separating the two stops a product decision — bigger Societies — from silently becoming a security decision.

Fracture at Level 5 is deliberate: the most consequential operation requires a Society that has demonstrably functioned.

**Founding, and the first hearth.** `18 §5.1` gates *founding a Society* at Citizen Level 3, with one deliberate exemption: **every Citizen may found exactly one Society at Level 0**, free of the K1 charge. The allowance is one-time and per-FNID, is consumed at `SocietyCreated`, and is **not restored** if that Society is later Dissolved, Archived, or the founder departs — a renewable first-hearth allowance is a renewable Sybil resource. A Crystallization does not consume it (`§3.1`'s thresholds are a stronger group-level gate than Level 3 is an individual one). The second and every subsequent Society requires `Level ≥ 3` and 250 FRC. The farmable quantity is Society *volume*, and that is where the gate sits (`61 X7`).

**Standing is derived, never asserted.** Both inputs to this gate — how many Societies the Citizen has founded, and their Level — are read by the Runtime from state the caller cannot write. They are not fields on `CreateSocietyRequest` and there is no way to supply them. This is not a defensive nicety: `societies_founded` is the *entire* first-hearth gate, so a caller able to set it can found without limit, and a gate whose input the attacker controls is decoration. The count is taken from the `society.created.v1` events themselves rather than from surviving Societies, because the allowance is consumed at creation and is not restored on Dissolution — counting live Societies would hand it back, which is the exploit the paragraph above exists to close.

`founder_level` has no trustworthy source until the XP projection exists (PH2, `docs/03`), so `SocietyService::level_of` returns 0 for everyone and PH0 therefore permits exactly one Society per Citizen. Zero is the correct placeholder because it fails safe: a level that is wrongly low refuses a founding that should have been allowed, which is visible and recoverable, while a level that is wrongly high mints Societies.

One half of this remains open in PH0. The Node derives the count correctly, but derives it against an identity the caller still asserts (`founder_fnid`, pending PH1's passkey session, `docs/12`). Forging standing therefore costs an FNID rather than a JSON field — better, and not closed. Every affected response carries an `unauthenticated` warning in its envelope so the seam is visible in the GUI and the CLI rather than only in this document.

---

### 2.4 Membership

```rust
struct Membership {
    society_id:  SocietyId,
    citizen:     Fnid,
    persona:     Persona,
    roles:       Vec<RoleId>,
    standing:    Standing,
    joined_at:   Timestamp,
    charter_ack: CharterVersion,     // which version they accepted
    state:       MembershipState,
}

enum MembershipState { Pending, Active, Restricted, Suspended, Departed }

struct Standing {                    // society-scoped, multi-dimensional (01 §7)
    contribution: u32,               // measured output in this Society
    trust:        i32,               // reliability here; can go negative
    tenure_days:  u32,
    governance:   u32,               // participation in proposals/votes
}
```

**Invariants**
- A Citizen has at most one Membership per Society.
- `charter_ack` must be ≤ the current Charter version; a member whose acknowledged version is superseded by a *material* amendment is prompted to re-acknowledge and is `Restricted` until they do.
- `standing.trust` is never a function of `standing.contribution`. Volume does not buy reliability (P12).

---

### 2.5 Chamber, Thread, Message

```rust
struct Chamber {
    chamber_id: ChamberId,
    society_id: SocietyId,
    kind:       ChamberKind,
    name:       String,
    topic:      String,
    access:     AccessPolicy,     // roles, Level gates, Standing gates
    encryption: EncryptionMode,   // Transport | EndToEnd(MlsGroupId)
    retention:  RetentionPolicy,
    agent_mode: ChamberAgentMode, // Forbidden | OnMention | Participant | Autonomous
}

enum ChamberKind { Text, Voice, Stage, Gallery, Board, Canvas, Experience }

struct Message {
    message_id:  Ulid,           // sortable by time
    thread_id:   ThreadId,
    author:      Principal,      // Citizen | Agent  ← rendered differently (33 §2.3)
    on_behalf_of: Option<Fnid>,
    body:        MessageBody,    // Text | Media | Facet | SystemNotice | AgentReport
    reply_to:    Option<Ulid>,
    edited:      Option<EditRecord>,
    reactions:   ReactionSet,    // CRDT OR-Set
    integrity:   Signature,      // author-signed
}
```

**Invariants**
- Every Message is signed by its author's device key. An unsigned Message is not a Message.
- An Agent-authored Message always carries `envelope_ref` in its event and is **always visually distinguished** in every client (violet, P4 + `33 §2.3`). Impersonation of a human by an Agent is impossible by construction, not by convention.
- Messages in an `EndToEnd` Chamber are stored ciphertext-only. The Runtime has no decryption path (N6).
- Editing preserves the original in the event log. `edited` is a projection convenience; history is the truth.

---

### 2.6 Wallet, Posting, Transfer

```rust
struct Wallet {
    wallet_id: WalletId,
    owner:     Principal,          // Citizen | Society(Treasury) | Agent
    society:   Option<SocietyId>,  // None for global Citizen wallet
    balance:   Quanta,             // 1 FRC = 1_000_000_000 quanta
    locked:    Quanta,             // staked or in-flight
    nonce:     u64,
}

struct Posting {                   // double-entry: always balanced
    posting_id: Ulid,
    debit:      WalletId,
    credit:     WalletId,
    amount:     Quanta,
    reason:     PostingReason,     // closed enum — every Source and Sink is named
    reference:  Option<Ulid>,      // the event that caused it
    at:         Timestamp,
}
```

**Invariants — these are the economy's spine (P12)**
- **Σ debits = Σ credits, always.** Enforced in the same transaction. A Posting that does not balance is not written.
- `balance >= locked >= 0` for every Wallet, always.
- Emission comes only from the `EmissionAccount`, which is itself a Wallet with a negative balance representing total emitted supply. **Total supply is therefore a directly queryable number, not an estimate.**
- `PostingReason` is a closed enum. There is no `Other`. Every Fraction that moves is categorized at the moment it moves — which is what makes the economy auditable rather than merely logged.
- Agents may hold and spend Fraction only within their Envelope's rate and total limits.

---

### 2.7 Vault, Object, Manifest, Shard

```rust
struct Vault {
    vault_id:   VaultId,
    society:    Option<SocietyId>,   // None => the Citizen Vault (mirrors Wallet, §2.6)
    owner:      Principal,           // Citizen | Society — always exactly one
    quota:      Quota,
}

struct Object {
    object_id:  ObjectId,
    vault:      VaultId,
    society_id: Option<SocietyId>,   // mirrors its Vault; None => the Citizen Vault
    path:       VaultPath,
    versions:   Vec<VersionRef>,     // append-only
    acl:        Acl,
    media:      Option<MediaMeta>,   // codec, dimensions, duration, derived renditions
}

struct Manifest {
    version_id:  VersionId,
    size:        u64,
    content_key: WrappedKey,         // encrypted to the ACL's key holders
    shards:      Vec<ShardRef>,      // ordered
    merkle_root: Hash,
    erasure:     ErasureScheme,      // e.g. Reed-Solomon 10-of-16
}

struct ShardRef { hash: Hash, size: u32, index: u32 }
```

**The Citizen Vault.** `society: None` denotes a Vault owned by a Citizen rather than a Society: personal storage, Profile media, Collections, and the eight Modules `21 §3.4` builds on it. It hangs off **Global Registry entry 1** (`Citizen`) through `owner`, adds **no** Global Registry entry, and opens no new P1 escape hatch — which is why invariant 1 in §7 is a three-clause test rather than a two-clause one. This mirrors `Wallet.society` in §2.6 exactly; the alternative, a `VaultScope { Society, Citizen }` enum, is better typed and was rejected because two spellings of one idea inside one domain model costs more in agent confusion than the type buys in rigour (`61 X11`). If `Wallet` is ever revisited, both move together.

**Invariants**
- Every Object resolves to exactly one Vault, and every Vault to exactly one owning Principal. `society_id` on an Object always equals `society` on its Vault.
- Shards are content-addressed and **encrypted before chunking**. A Custodian holding a Shard learns nothing about its content — not even the file boundaries.
- Versions are append-only. "Delete" writes a tombstone and schedules Shard garbage collection after the retention window.
- A Manifest's `merkle_root` must verify against its Shards on every reconstruction. A failed verification is a `ReplicaCorrupt` event and slashes the Custodian's Stake.
- Custodian rewards pay against **Attestations**, never against claimed storage (`13 §5`).

---

### 2.8 Agent and Envelope

```rust
struct Agent {
    fnid:        Fnid,
    operator:    Fnid,               // the accountable Citizen — always exactly one
    kind:        AgentKind,          // Assistant | Moderator | Curator | Custodian
                                     // | Workflow | Guardian | Custom
    model_ref:   ModelRef,           // which ModelProvider + model
    wallet:      WalletId,
    trust:       Trust,              // agents have reputation too
    enrollments: Vec<SocietyId>,
}

struct Envelope {
    envelope_id:   EnvelopeId,
    grantee:       Principal,        // Agent or ExtensionInstall
    society_id:    SocietyId,        // envelopes are society-scoped (P1)
    capabilities:  CapabilitySet,
    limits:        Limits,           // per-action rate, per-day totals, spend caps
    expires_at:    Timestamp,        // ALWAYS set; no perpetual envelopes
    granted_by:    Fnid,             // a Citizen. never an Agent. (P4)
    granted_sig:   Signature,
    revoked_at:    Option<Timestamp>,
    confirm_classes: Vec<ActionClass>, // classes requiring live human confirmation
}
```

**Invariants**
- `granted_by` must be a Citizen holding every capability being granted. Escalation is structurally impossible.
- `expires_at` is mandatory. Maximum 90 days; renewals are explicit human acts.
- Revocation is immediate and retroactive to in-flight actions: an action that began under a revoked Envelope fails at the Policy Enforcement Point.
- Every irreversible action class defaults to `confirm_classes` and can be removed only by explicit human policy.

---

### 2.9 Facet (the native asset primitive)

```rust
struct Facet {
    facet_id:   FacetId,
    standard:   StandardId,          // FN-ASSET/1
    society_id: SocietyId,           // minted under a Society (P1)
    owner:      Principal,
    creator:    Fnid,
    state:      FacetState,          // MUTABLE — this is the point
    schema:     SchemaRef,
    evolution:  EvolutionRules,      // what may change state, and under what conditions
    license:    License,
    provenance: Vec<ProvenanceEntry>,// append-only chain of custody
    bindings:   Vec<ChainBinding>,   // external representations, if any
}
```

Facets are **dynamic by default**. A Facet's state can change under declared rules — an Insignia that gains tiers, an instrument that records who played it, a document that accrues signatures, a Society artifact that reflects its Society's Level. Static-forever assets are the degenerate case (`evolution: Immutable`), not the default. Full specification in `16-ledger-and-assets.md`.

---

### 2.10 Convergence (pre-Society social)

```rust
struct Convergence {
    convergence_id: ConvergenceId,
    seed:           ConvergenceSeed, // Interest | Invitation | Proximity | Serendipity
    participants:   Vec<Fnid>,
    thread:         ThreadId,
    opened_at:      Timestamp,
    expires_at:     Timestamp,       // default 72h, extends with activity
    crystallization: CrystallizationState,
}

enum CrystallizationState {
    Ephemeral,
    Eligible { met_at: Timestamp },  // thresholds satisfied
    Proposed { by: Fnid, name: String, votes: Vec<Fnid> },
    Crystallized(SocietyId),
    Expired,
}
```

This is the **social funnel made into a first-class domain object**. A conversation is not a lesser thing that might become a community; it is the earliest state of one, with a defined promotion path that preserves everything.

---

## 3. The Signature Lifecycle Operations

### 3.1 Crystallization — Convergence → Society

```
  Convergence (ephemeral)
       │
       │  eligibility thresholds (all must hold):
       │    • ≥ 3 participants
       │    • ≥ 48h of activity OR ≥ 100 messages
       │    • ≥ 2 participants at Citizen Level ≥ 1
       ▼
  Eligible ──► any participant proposes name + initial Charter template
       │
       │  ≥ 2/3 of participants accept
       ▼
  Crystallized
       │
       ├─► SocietyCreated { origin: Crystallized(convergence_id) }
       ├─► Treasury created, funded with the Convergence's accrued Fraction
       ├─► Vault created; Convergence media re-homed by reference (no re-upload)
       ├─► Chamber "general" created; the Convergence Thread is MOVED into it,
       │     preserving message IDs, authorship, signatures, and reactions
       ├─► Membership created for each participant, tenure backdated to
       │     Convergence open (their history counts)
       └─► Charter v1 enacted, signed by accepting participants
```

**Nothing is lost and nothing is re-created.** The Thread keeps its identity; the messages keep their signatures; tenure counts from the first word spoken. This is the invariant that makes the progression from chat → group → Society feel like growth rather than migration.

### 3.2 Fracture — Society → Societies

The platform's namesake operation, and the most dangerous one in the system.

```
  Society S (Level ≥ 5)
       │
       │  a Fracture Proposal specifies:
       │    • the split: which Memberships go to which child
       │    • Chamber disposition: move | copy | archive (per Chamber)
       │    • Vault disposition: move | shared-custody | copy (per path)
       │    • Treasury division: explicit ratio or per-member formula
       │    • Facet disposition (creator-bound Facets follow their creator)
       │    • whether S survives as one of the children or dissolves
       │
       │  Charter's fracture_rules govern quorum and threshold
       ▼
  FractureProposed
       │
       │  ── DRY RUN (mandatory) ────────────────────────────────────┐
       │     simulate the entire split; produce a diff report;       │
       │     any invariant violation blocks the vote                 │
       │  ────────────────────────────────────────────────────────────┘
       ▼
  FractureApproved
       │
       │  Society enters status = Fracturing:
       │    • Log is SEALED at seq N (the fracture point)
       │    • Treasury is frozen
       │    • no new members, no new Facet mints
       ▼
  Execution (single transaction per child, idempotent, resumable)
       │
       ├─► child societies created with lineage.origin = Fractured{parent: S, siblings}
       ├─► each child's Log begins with a GENESIS event containing:
       │     - the parent's merkle root at seq N
       │     - the full split specification
       │     - the parent's Charter as the child's Charter v1 (amendable after)
       ├─► Treasury divided by balanced Postings (Σ debits = Σ credits, always)
       ├─► Vault: Manifests re-referenced, NOT re-uploaded. Shards gain a second
       │     owning society. Custodian contracts continue uninterrupted.
       ├─► Vault REKEY (mandatory, before any child becomes Active):
       │     for every Vault path assigned to EXACTLY ONE child —
       │        rotate the content key
       │        re-wrap it to THAT CHILD'S key holders only
       │        emit VaultKeyRotatedOnFracture { path, child, key_epoch }
       │     paths with disposition `shared-custody` keep their key and are
       │        listed in the dry-run diff as DELIBERATELY SHARED, path by path
       │     existing Shards are NOT re-encrypted — impossible for content-
       │        addressed data, and stated in 13 §10.1's residual register
       ├─► Memberships created in children with tenure and Standing CARRIED OVER
       ├─► parent → Archived (or continues as designated child)
       └─► FractureCompleted { parent, children, root_at_fracture }
       ▼
  Children Active. Parent history remains readable and cryptographically
  verifiable from every child, forever.
```

**Invariants that make Fracture safe**
- The parent's log is sealed *before* any child event is written. There is no window in which both accept writes.
- Treasury division is a set of balanced Postings. A rounding remainder goes to the largest child, deterministically — never burned silently, never created.
- Every Citizen retains membership in exactly the children the split assigns them, and **no Citizen loses history**: the parent log is readable from every child.
- Storage is re-referenced, never re-uploaded. Fracturing a 10TB Society must not move 10TB.
- The operation is **resumable**: if it fails midway, re-running completes it. Partial fracture is not a reachable end state.
- **Confidentiality is divided, not only ownership.** Before any child becomes `Active`, every Vault path assigned to exactly one child has its content key rotated and re-wrapped to that child's key holders alone. Without this step a member of child B who held a wrapped `content_key` for a path assigned to child A can decrypt **every future Version** of it indefinitely, because `13 §10.1`'s revocation is forward-effective and nothing in the split removes READ. The cost is proportional to key holders, not bytes — an 800-member Society at 2.5 devices each is 2,000 re-wraps per rotated path — so "fracturing a 10TB Society moves zero bytes" is fully preserved. What is **not** fixed, and is disclosed in the dry-run diff rather than papered over: a member of B who already downloaded and decrypted A's historical Versions keeps that plaintext forever. Rotation stops the future, which is the half that can be stopped (`61 W10`).
- **A dry run is mandatory.** No Fracture executes without a simulated diff that a human has seen.

### 3.3 Dissolution

```
DissolutionProposed → quorum per Charter → Treasury distributed per Charter
   → Vault: members given an export window (default 30d), then GC
   → Facets released to their owners (never destroyed)
   → Log sealed and archived; remains readable
   → status = Archived (never deleted — history is not ours to erase)
```

---

## 4. State Machines

**Society**
```
 Forming ──► Active ──┬──► Dormant ──► Active
                      │        └────► Dissolving ──► Archived
                      ├──► Fracturing ──► Archived (parent)
                      └──► Dissolving ──► Archived
```
Dormant = no events for 180 days. Auto-reversible on any member action. Dormancy reduces the Society's storage-cost subsidy but never deletes data.

**Membership**
```
 Pending ──► Active ──┬──► Restricted ──► Active
                      ├──► Suspended ──► Active | Departed
                      └──► Departed
```
`Departed` retains authored history (attributed, immutable) and zeroes forward-looking Standing.

**Facet**
```
 Minting ──► Active ──┬──► Evolving ──► Active
                      ├──► Locked (staked/escrowed) ──► Active
                      ├──► Transferring ──► Active (new owner)
                      └──► Retired (owner-initiated; state frozen, never deleted)
```

**Envelope**
```
 Requested ──► Granted ──┬──► Expired
                         ├──► Revoked
                         └──► Renewed (new envelope; old one Expired)
```

---

## 5. Cross-Aggregate Consistency

Operations spanning aggregates use **sagas** with explicit compensation. Every saga is a named, resumable, observable process — never an implicit chain of listeners.

| Saga | Steps | Compensation |
|---|---|---|
| `JoinSociety` | validate Charter join policy → create Membership → create Standing → grant default Envelope → emit welcome | delete Membership; no partial join is observable |
| `PurchaseExtension` | reserve Fraction → verify license → install → post revenue split → release reservation | release reservation; roll back install |
| `MintFacet` | validate Standard → reserve mint fee → allocate FacetId → write provenance → post fee | release reservation |
| `Fracture` | (see §3.2) | resumable-forward only; never rolls back after seal |
| `StorageSettlement` | collect Attestations → score → compute emission → post → emit receipts | idempotent per settlement window |
| `Crystallize` | create Society → move Thread → create Memberships → transfer accrued Fraction | reverse to Convergence if any step fails before Charter enactment |

**Rule:** a saga that cannot be compensated must be *resumable* instead, and must seal its inputs before it starts. Fracture is the canonical example.

---

## 6. Identifier Scheme

| Entity | Format | Rationale |
|---|---|---|
| `Fnid` | `fn1` + base32(Ed25519 pubkey) + 4-char checksum | Self-certifying, human-checkable, chain-agnostic |
| `SocietyId` | `soc_` + ULID | Sortable by creation, opaque |
| `Message` | ULID | Time-sortable without a separate index |
| `Handle` | `@` + `[a-z0-9_]{3,24}` | Human namespace, case-folded, confusable-normalized |
| `ObjectId` / Shard | multihash (BLAKE3) | Content-addressed, verifiable, dedupes for free |
| `FacetId` | `fct_` + ULID + society prefix | Traceable to origin Society |
| Events | ULID + per-society `seq` | Global uniqueness + local ordering |

BLAKE3 for content addressing: faster than SHA-256, parallelizable over large media, and its tree structure gives verified streaming — you can validate a video while playing it rather than after downloading it.

---

## 7. Invariants Enforced in Code (the test suite writes itself)

Each of these becomes a property test that runs on every PR. This list is the contract between the domain model and reality.

1. Every persisted row either (a) carries a `society_id`, or (b) carries a null `society_id` **and** a resolvable owning reference to a Global Registry entry — the Citizen Vault and the Citizen's global Wallet are the only two users of this clause — or (c) is itself a Global Registry entry.
2. `Σ ledger debits == Σ ledger credits` after every command, in every generated history.
3. `wallet.balance >= wallet.locked >= 0` for all wallets, always.
4. Total supply == `-Σ EmissionAccount[i].balance` over the K = 64 emission shards, at all times. Sharding removes the one global hot row (`16 §4.4`) without making supply an estimate: the sum is exact and directly queryable.
5. No Envelope grants a capability its grantor lacks.
6. No Envelope lacks an `expires_at`.
7. Every Agent action event carries a valid, unexpired, unrevoked `envelope_ref`.
8. Trust is never written by a process whose input includes XP or Fraction.
9. Every Message has a valid author signature.
10. Every projection is reproducible by replaying the log **from its most recent verified checkpoint**, and from zero within the published rebuild SLO (`40 §9.4`: ≤ 15 minutes at p99, any projection, any Society). Replay from zero is retained as a periodic audit, not as the recovery path — "drop and rebuild" with no checkpoint is a multi-hour outage wearing a runbook (`61 W5`).
11. Fracture preserves: total Fraction, total Facets, total members, and full readable history.
12. Crystallization preserves: message IDs, authorship, signatures, and tenure.
13. No inferred value ever appears in `Citizen.interests`.
14. A `Sealed` Society's log length never increases.
15. Every `PostingReason` variant maps to a declared Source or Sink in `17`, or to the single declared non-Source allocation, `GenesisAllocation` (`61 X-GA`).
16. **After a Fracture, no principal assigned solely to child B holds a live wrapped key for any Vault path assigned solely to child A.** Rotation is forward-effective: Versions written before the fracture point remain decryptable by anyone who held the key at that time, and that residual is disclosed in the dry-run diff. Tested over the same 100,000 generated splits as invariant 11 (`50 PH5` AC-1).
