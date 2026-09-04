# 01 — Canonical Terminology

> **Status:** Canon. Part of the four-file Canon contract every agent loads before implementation (`00`, `01`, `02`, `03`).
> **Rule:** These words mean exactly this, everywhere — in code identifiers, database columns, API paths, CLI verbs, UI copy, documentation, and commit messages. Introducing a synonym is a defect. Introducing a new term requires adding it here in the same PR.

Ambiguous vocabulary is the primary vector by which a coherent architecture rots. When one engineer's "group" is another's "channel" and an agent's "room," the schema forks, the API forks, and the mental model dies. This file is the dictionary.

---

## 1. Core Containment Hierarchy

```
                          ┌──────────────────────┐
                          │      FRACTAL NET     │   the global mesh of all Nodes
                          └───────────┬──────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    │                 │                 │
              ┌─────▼─────┐     ┌─────▼─────┐     ┌─────▼─────┐
              │   NODE    │     │   NODE    │     │   NODE    │  a running instance
              └─────┬─────┘     └───────────┘     └───────────┘
                    │  hosts
        ┌───────────┼───────────┐
        │           │           │
   ┌────▼────┐ ┌────▼────┐ ┌────▼────┐
   │ SOCIETY │ │ SOCIETY │ │ SOCIETY │   ← THE ATOMIC CONTAINER
   └────┬────┘ └─────────┘ └─────────┘
        │  owns everything below
        ├── CHAMBER (a space inside a society: text / voice / stage / gallery / board)
        │      └── THREAD ── MESSAGE
        ├── MEMBERSHIP (a Citizen's bond to this Society, carrying Standing)
        ├── TREASURY (the Society's wallet)
        ├── VAULT (the Society's storage namespace)
        ├── CHARTER (the Society's governance document)
        ├── AGENT (agents are enrolled in a Society, not free-floating)
        ├── ASSET (Facets minted under this Society)
        ├── EXTENSION INSTALL (plugins active in this Society)
        └── LEDGER SCOPE (the Society's slice of the ledger)
```

**The invariant (P1):** every persistent object resolves to exactly one `society_id`, **or** carries a null `society_id` together with a resolvable owning reference to a Global Registry entry, **or** appears on the Global Registry in §6 itself. Anything else is a violation (`11 §7.1`).

The middle clause has exactly two users, both of which mirror each other by design:

```
   CITIZEN  (Global Registry entry 1)
      ├── WALLET  (society: None)   the Citizen's global Wallet          — 11 §2.6
      └── VAULT   (society: None)   the Citizen Vault: personal storage,
                                    Profile media, Collections, Modules  — 11 §2.7, 21 §5.1
```

Neither adds a Global Registry entry and neither opens a new P1 escape hatch: both hang off entry 1 and are reachable only through it.

---

## 2. Identity and People

| Term | Definition | Never call it |
|---|---|---|
| **Citizen** | A human account. The person-level identity, portable across Societies. | user, member, profile, account |
| **Membership** | The relationship between a Citizen and a Society. Carries role, Standing, join date, and Charter acceptance. | membership is *not* the Citizen |
| **Persona** | A presentation of a Citizen within one Society — display name, avatar, pronunciation, profile modules. A Citizen has one Persona per Society. | alias, alt, character |
| **Profile** | The Citizen's global home surface. Composed of Modules. | page, wall, homepage |
| **Handle** | Globally unique human-readable identifier, `@name`. **One change, once, within 14 days of first claim; immutable thereafter.** The same rule governs a Society name (`61 X-GW`). | username, tag |
| **FNID** | The cryptographic identity: a Citizen's, Society's, Agent's, or Node's public-key-derived identifier. Format in `12-identity-and-trust.md`. | uuid, address, DID |
| **Agent** | An autonomous non-human principal with its own FNID, wallet, Envelope, and audit trail. | bot, assistant, automation, script |
| **Operator** | The Citizen legally and reputationally accountable for an Agent. Every Agent has exactly one. | owner, author |
| **Node** | A running Fractal Node instance (desktop app, headless server, or hosted tenant) that stores replicas and serves peers. | client, server, peer — use Node |
| **Principal** | Umbrella for anything that can hold capabilities: Citizen, Agent, Society, Node, Extension Install. | actor, subject |

---

## 3. Society Lifecycle Terms

| Term | Definition |
|---|---|
| **Society** | The atomic container. A sovereign digital community with its own governance, treasury, storage, membership, and event log. |
| **Charter** | The Society's machine-readable governance document: roles, permissions, voting rules, economic parameters, moderation policy, agent policy. Versioned; changes are governance events. |
| **Standing** | A Citizen's multi-dimensional reputation *within one Society*. Distinct from XP. See `18-progression-and-reputation.md`. |
| **Chamber** | A space inside a Society. Kinds: `text`, `voice`, `stage`, `gallery`, `board`, `canvas`, `experience`. Replaces "channel"/"room"/"forum". |
| **Convergence** | A spontaneous, ephemeral conversation between Citizens who have not yet formed a Society. The pre-Society social primitive. |
| **Crystallization** | The promotion of a Convergence into a Society, preserving history, membership, and identity. |
| **Fracture** | The deliberate split of a Society into two or more independent Societies, with a defined division of treasury, storage, membership, and history. The platform's signature operation. |
| **Fork** | A *copy* of a Society's Charter and structure into a new Society, without dividing the parent's assets. Distinct from Fracture. |
| **Dissolution** | The wind-down of a Society: treasury distribution, storage disposition, archive sealing. |
| **Lineage** | The ancestry graph of Societies produced by Crystallization, Fracture, and Fork. |
| **Federation** | A voluntary alliance of Societies sharing discovery, some capabilities, or a joint treasury. Not a container — Societies remain atomic. |

---

## 4. Economy

| Term | Definition | Never call it |
|---|---|---|
| **Fraction** | The platform's native unit of account. Symbol **FRC**. Smallest unit: **1 quantum** = 1e-9 FRC. | coin, credit, point, currency |
| **Wallet** | A Fraction account bound to exactly one Principal. Every Citizen, Society, and Agent has one. | balance, account |
| **Treasury** | The Society-owned Wallet, governed by the Charter. | society wallet |
| **Ledger** | The deterministic double-entry record of all Fraction movement. Abstracted behind the `Ledger` trait (P11). | blockchain, chain, database |
| **Posting** | A single balanced debit/credit pair in the Ledger. All Fraction movement is Postings. | transaction (reserve that word for the API-level envelope) |
| **Transfer** | A user-visible movement of Fraction between Wallets; compiles to one or more Postings. | payment, send |
| **Emission** | Newly created Fraction entering circulation from a defined Source. Bounded and published. | minting, inflation |
| **Source** | A named, rate-limited mechanism that emits Fraction (contribution rewards, storage/bandwidth settlement, etc.). |
| **Sink** | A named mechanism that removes Fraction from circulation (burns, fees, stakes forfeited, expiries). |
| **Genesis Allocation** | The bounded, published, one-time grant of Fraction to a new Citizen and a new Society Treasury, posted from the `EmissionAccount` under `PostingReason::GenesisAllocation`. It exists so that PH1's Wallet and Transfer surfaces are real before any Source emits, it is capped in aggregate, it counts against total supply, and it is retired at the PH4 exit gate. **It is not a Source** — no rate, no formula, no Contribution Score, no recurrence — and it consumes no `02 §5` Source budget slot. See `61 X-GA`. |
| **Contribution Score** | The measured, Sybil-resistant quantity that Sources pay against. Defined per Source in `17-economy-fraction.md`. |
| **Stake** | Fraction locked as a bond against misbehavior; slashable. |
| **Facet** | The native digital asset primitive — a dynamic, evolving, chain-agnostic asset. Not an NFT clone. See `16-ledger-and-assets.md`. | NFT, token, collectible |
| **Facet Standard** | The `FN-ASSET/1` specification governing Facet identity, state, evolution, ownership, licensing, and transfer. |
| **Rail** | A payment/settlement adapter (internal ledger, future FN L1, external chain, fiat processor). Abstracted. |

---

## 5. Automation, Extension, and Interface

| Term | Definition | Never call it |
|---|---|---|
| **Envelope** | The scoped, time-boxed, revocable set of capabilities granted to an Agent or Extension. The unit of authority. | permissions, scopes, role |
| **Capability** | One atomic authority token, e.g. `society.chamber.post`, `wallet.transfer<=100FRC/day`. Deny-by-default. |
| **Policy** | A human-authored rule that determines which Envelopes may exist and what actions require human confirmation. Only Citizens author Policy (P4). |
| **Extension** | A distributable unit that adds capability. Kinds: `plugin`, `theme`, `template`, `workflow`, `automation-pack`, `sdk`, `experience`. | app, addon, mod |
| **Extension Install** | An Extension activated inside one Society, holding its own Envelope and configuration. |
| **Workflow** | A declarative, versioned automation graph an Agent executes. | flow, pipeline, script |
| **Runtime** | The single shared core (Rust) that every front end talks to. Exactly one exists. | backend, engine, server |
| **Front End** | A peer interface over the Runtime: Web GUI, Desktop, Mobile, CLI, Agent API. All equal (P13). |
| **Surface** | A distinct interactive area within a front end (e.g. the Wallet surface, the Chamber surface). |
| **Terminal** | The branded interactive TTY experience of the CLI: boot sequence, dashboards, live status. |
| **Signal** | A real-time push notification of a domain event to a subscribed front end. |
| **Relay** | The service that fans Signals out to subscribers. |
| **Experience** | An interactive, hosted, governed application running inside a Chamber via the Experience Runtime (games, tools, simulations). Phase 7+. |

---

## 6. Data, Storage, and Events

| Term | Definition | Never call it |
|---|---|---|
| **Domain Event** | An immutable, typed, ordered fact appended to a Society's log. Past tense, e.g. `ChamberMessagePosted`. | message, record |
| **Log** | The per-Society append-only sequence of Domain Events. Source of truth (P6). | stream, journal |
| **Projection** | A derived read model rebuilt from the Log. Disposable by definition. | cache, view, table |
| **Vault** | A logical storage namespace owned by exactly one Principal: a Society (`society: Some(id)`) or a Citizen (`society: None`, the Citizen Vault). | bucket, drive |
| **Shard** | A content-addressed, encrypted chunk of a stored object. The unit of distribution and reward. | chunk, block, piece |
| **Manifest** | The ordered list of Shard hashes plus metadata that reconstitutes an object. | index, torrent |
| **Custodian** | A Node storing Shards for others, compensated in Fraction. | host, seeder, provider |
| **Attestation** | A verifiable proof that a Custodian actually held a Shard at a time. Basis for payment. | proof-of-storage, challenge |
| **Replica** | One copy of a Shard held by one Custodian. Target replica count is policy. |
| **Anchor** | A periodic cryptographic commitment of a Society's Log state root to the Ledger (and later to a chain). |
| **Atlas** | The one boundary that reads across Societies (S15, `10 §3`). It owns every cross-partition read model — the Citizen unified inbox, cross-Society search, marketplace statistics, and the Shard reference count that governs garbage collection. Eventually consistent, monotonic per reader, with a published staleness bound. **Read-only and never authoritative:** Atlas emits no Domain Event, holds no Wallet, takes no lock, and is never read by a command handler. Where Atlas and a Society's Log disagree, the Log wins and Atlas is rebuilt. | global index, cache, aggregator |

**Global Registry** — the complete, closed list of objects that are *not* owned by a Society (P1 escape hatch). Adding to it requires an ADR.

1. `Citizen` (the person-level identity itself — and the Citizen's global Wallet and Citizen Vault, which hang off it and are reachable only through it)
2. `Handle` (global uniqueness namespace)
3. `FNID` key material
4. `Society` (the registry of societies)
5. `Node` (the registry of nodes)
6. `Extension` (marketplace listings, pre-install)
7. `Facet Standard` definitions (schemas, not instances)
8. Global economic parameters (emission caps, published constants)
9. Platform-level Trust records (§7)

That is the entire list. Nine entries. Anything else has a `society_id`.

---

## 7. Progression, Trust, and Safety

| Term | Definition |
|---|---|
| **XP** | Experience points. Measures *volume of meaningful contribution*. Monotonic; never decreases. Drives Level. |
| **Level** | A Citizen's or Society's progression tier, derived from XP. Citizens start at **Level 0**. Unlocks capabilities. |
| **Trust** | A separate, *bidirectional* score measuring reliability and good faith. Can decrease. Never purchasable, never earned by volume. |
| **Standing** | Society-scoped reputation: the tuple of (Trust, Contribution, Tenure, Governance participation) within one Society. |
| **Unlock** | A capability, cosmetic, or privilege gated behind Level, Trust, Standing, or Achievement. Categories in `18`. |
| **Achievement** | A named, non-repeatable recognition of a specific accomplishment. |
| **Milestone** | A quantitative threshold crossing (100th message, 1TB served). Distinct from Achievement. |
| **Season** | A time-boxed period with its own additive content and objectives. Never resets permanent progress. |
| **Insignia** | A displayable badge or collectible earned through progression. A Facet subtype. |
| **Moderation Action** | A recorded, appealable intervention taken under a Charter. |
| **Appeal** | The Charter-defined process for contesting a Moderation Action. |

**Hard rule:** XP and Trust are never the same number, never derived from each other, and never displayed as one value. XP says *how much you did*. Trust says *whether you can be relied on*. Conflating them is how reputation systems get gamed.

---

## 8. Verb Vocabulary (API, CLI, and events must use these)

| Verb | Meaning | Applies to |
|---|---|---|
| `create` / `Created` | Bring a new object into existence | any |
| `join` / `Joined` | Establish a Membership | Society, Chamber, Convergence |
| `leave` / `Left` | End a Membership voluntarily | as above |
| `post` / `Posted` | Add a Message to a Thread | Chamber |
| `grant` / `Granted` | Add a Capability to an Envelope | Envelope |
| `revoke` / `Revoked` | Remove a Capability | Envelope |
| `transfer` / `Transferred` | Move Fraction or a Facet | Wallet, Facet |
| `stake` / `Staked` | Lock Fraction as a bond | Wallet |
| `slash` / `Slashed` | Forfeit a Stake | Stake |
| `mint` / `Minted` | Create a Facet | Facet |
| `evolve` / `Evolved` | Advance a Facet's state | Facet |
| `pin` / `Pinned` | Commit a Node to custody Shards | Vault |
| `attest` / `Attested` | Prove custody | Shard |
| `crystallize` / `Crystallized` | Promote Convergence → Society | Convergence |
| `fracture` / `Fractured` | Split a Society | Society |
| `enact` / `Enacted` | Apply a governance decision | Charter |
| `install` / `Installed` | Activate an Extension in a Society | Extension |
| `invoke` / `Invoked` | Run a Workflow or Agent action | Agent, Workflow |
| `publish` / `Published` | Make a Listing or an Extension version available in the marketplace | Listing, Extension |
| `purchase` / `Purchased` | Acquire a License in exchange for Fraction | Listing |
| `refund` / `Refunded` | Reverse a settled purchase, including every fee leg and the burn | Purchase |
| `license` / `Licensed` | Grant use rights over a Facet, Extension, or Object under stated terms | Facet, Extension, Object |
| `payout` / `PaidOut` | Settle accrued creator earnings to a Wallet | Wallet |
| `recall` / `Recalled` | Disable a published version everywhere under a signed order | Extension, Listing |
| `review` / `Reviewed` | Record a purchaser's rating and written assessment | Listing |
| `curate` / `Curated` | Place a Listing on a Shelf under an attributable governance role | Shelf, Listing |

Forbidden verbs, because they are ambiguous: `update`, `process`, `handle`, `manage`, `sync`, `do`. Name the actual state change.

---

## 9. Naming Conventions in Code

| Context | Convention | Example |
|---|---|---|
| Rust crates | `fractal-<layer>-<domain>` | `fractal-domain-society` |
| Rust types | `PascalCase`, canonical term verbatim | `SocietyId`, `FacetManifest` |
| Domain events | `<Noun><PastTenseVerb>` | `ChamberMessagePosted` |
| Commands | `<ImperativeVerb><Noun>` | `PostChamberMessage` |
| API paths | `/v1/societies/{society_id}/chambers/{chamber_id}` — plural nouns, canonical terms | |
| CLI | `fn <noun> <verb>` | `fn society fracture`, `fn wallet transfer` |
| Env vars | `FRACTAL_<AREA>_<NAME>` | `FRACTAL_LEDGER_BACKEND` |
| Feature flags | `ff.<area>.<name>` | `ff.economy.storage_settlement` |
| Metrics | `fractal_<area>_<name>_<unit>` | `fractal_relay_signal_latency_ms` |

---

## 10. Terms We Deliberately Do Not Use

| Banned | Why | Use instead |
|---|---|---|
| user | Ambiguous between Citizen, Principal, and Operator | Citizen / Principal |
| server | Contradicts the Node model | Node / Runtime |
| channel, room, forum | Three words for one thing | Chamber |
| NFT | Carries chain-specific baggage we explicitly reject | Facet |
| gas, mining | We are not doing that | fee / Sink; Contribution |
| feed, algorithm | Implies surveillance ranking (P9) | Discovery / Signal stream |
| karma, points | Conflates XP and Trust | XP / Trust / Standing |
| admin | Implies unbounded authority | a named Charter role with an explicit Envelope |
| microservice | Premature; we ship a modular monolith first | Runtime module / Service boundary |
| decentralized (as a claim) | Meaningless without specifics | state the specific property (self-hosted, E2EE, replicated) |
