# 30 — API and SDK

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `10-system-architecture.md` §3/§5/§10, `11-domain-model.md`, `12-identity-and-trust.md` §5/§7, `14-realtime-and-social.md` §2, `15-agent-runtime.md` §4/§6.
> **Governs:** the public API surface in all three protocols; versioning and the definition of a breaking change; the complete resource map; request, response, pagination, and conditional-request conventions; the error model and its closed code registry; authentication and authorization on the wire; rate limits and quotas; the Signal WebSocket protocol as clients consume it; batch, long-running, streaming, export and import; webhooks; the contract source of truth and everything generated from it; the six SDKs; agent-facing design; documentation standards; and API governance.
> **Does not govern:** the Capability grammar or the Trust model (`12`), the Policy Enforcement Point's evaluation order (`15 §6`), Relay internals, backpressure policy, or the Signal latency budget (`14`), Ledger mechanics (`16`), pricing and payouts (`19`), or the Extension host API (`20`). This chapter defines the **wire**; those chapters define what is on either end of it.
>
> **Terminology additions proposed by this document** (per `01 §8`, proposed in the same change that uses them): `export`/`Exported`, `import`/`Imported`, `subscribe`/`Subscribed`, `enroll`/`Enrolled`, `revise`/`Revised`, `deprecate`/`Deprecated`.

---

## 1. Position

P3 makes this chapter load-bearing rather than descriptive: *no capability exists until it exists as a versioned, documented, machine-consumable API.* Its falsification test is a grep, and the CLI must perform everything the GUI can. That test is survivable only if the API is the product's actual interface rather than a projection taken afterwards.

Three decisions in this chapter carry the weight.

**The contract is a file, not an implementation.** A single machine-readable contract generates the OpenAPI document, the protobuf services, the JSON Schemas for event payloads, the six SDKs' transport layers, the CLI command surface, and the capability registry. Nothing is written twice, which is the only durable reason to believe the surfaces agree.

**P13 becomes a build failure.** "Feature parity across front ends" is aspirational when parity is a review checklist and mechanical when a declared operation with no CLI binding fails the build. §12.4 specifies exactly that gate.

**The API is designed for a caller that cannot read prose.** Agents are first-class principals (P4), not a secondary audience. An API that answers "what may I do?" and "what would this do?" without a trial-and-error write is a different artifact from one that does not. §14 treats that as a requirement.

Everything else here is consequence.

---

## 2. Protocol Doctrine

### 2.1 Three protocols, one contract

```
                       ┌──────────────────────── THE CONTRACT ────────────────────────┐
                       │  contract/*.proto  +  fn.* custom options  (§12)              │
                       └───────┬──────────────────┬───────────────────┬───────────────┘
                               │ generates        │ generates         │ generates
                  ┌────────────▼──────┐  ┌────────▼────────┐  ┌───────▼──────────┐
                  │  OpenAPI 3.1      │  │  gRPC services  │  │  JSON Schema     │
                  │  (HTTP/JSON)      │  │  (protobuf)     │  │  2020-12 (events)│
                  └────────┬──────────┘  └────────┬────────┘  └───────┬──────────┘
                           │                      │                   │
     ══════════════════════▼══════════════════════▼═══════════════════▼════════════════
       HTTP/JSON over TLS 1.3         gRPC over HTTP/2            WebSocket (Signals)
       ── the public surface ──       ── internal + bulk ──       ── push only ──
     ══════════════════════════════════════════════════════════════════════════════════
              │                              │                            │
      every front end,             Runtime↔Relay, Runtime↔          every front end
      every third party,           Executor, Custodian              that renders live
      every Agent by default       coordination, agent              state
                                   batch traffic ≥1k ops/s
```

The three protocols are not three APIs. They are three encodings of one contract, and a capability that exists in only one of them is a defect the generator is built to prevent.

### 2.2 HTTP/JSON is the primary public surface

**Decision.** Resource-oriented HTTP/1.1 and HTTP/2 with JSON bodies is the surface every documented capability appears on first, and the only surface a third party is required to learn.

**Why.** It is the lowest-common-denominator protocol that every language, proxy, observability stack, and AI agent already speaks without a code generator. Two reasons make that decisive here. P3's falsification test is a grep across clients — a surface reproducible with `curl` gets audited; one requiring a compiled stub does not. And an agent can construct a correct HTTP request from a description; it cannot construct a correct gRPC frame.

**Honest cost.** JSON costs roughly 2.5× CBOR's serialization at the same payload (`14 §2` measured this and chose CBOR for Signals), and has no streaming story worth using for high-frequency writes. We route around both where volume actually appears (§2.3, §2.4). JSON's number problem we fix by contract: **every 64-bit integer is encoded as a decimal string**, including `seq` and every `quanta` amount. A Fraction silently losing precision in a JavaScript client is a defect class removed at the encoding rather than policed at review.

### 2.3 gRPC for internal traffic and high-throughput agents

**Decision.** gRPC over HTTP/2 carries Runtime-internal traffic across extracted boundaries (`10 §2`'s extraction list: Transcoder, Relay, Agent Executor, Search Indexer, Custodian Coordinator) and is offered publicly for agent workloads that sustain more than roughly a thousand operations per second or need bidirectional streaming.

**Why.** An extracted boundary needs a typed, versioned, low-overhead call surface with real deadlines and cancellation propagation, and HTTP/2 supplies flow control rather than a hand-rolled window. Offering it publicly for the narrow high-throughput case costs nothing — the services generate from the same contract — and it removes the pressure to distort the HTTP surface into a bulk-ingest API.

**Honest cost.** A second wire encoding means a second conformance suite, and protobuf's JSON mapping is not ours (`oneof` naming, int64-as-string, unset-versus-default). §12.3 pins one profile and tests both encodings against one golden corpus — real recurring work. gRPC stays awkward from browsers; we do not solve that, because the browser's protocol is HTTP/JSON plus the Signal socket by design.

**What gRPC is not for.** It is not a faster private path for the first-party GUI. P3 forbids that outright: the web client uses the same public HTTP surface with the same credentials and the same limits a third party gets.

### 2.4 WebSocket for Signals

The Signal protocol is specified in `14 §2` and is not restated here; §9 documents only what a client author must know to implement it against the contract. The division of labour is fixed and worth stating as a rule:

> **Invariant I-30.1 — the socket never writes domain state.** Every state change enters through HTTP or gRPC and carries an `Idempotency-Key`. The Signal socket carries pushes, plus a closed set of ephemeral client frames (presence, typing, read marks, acks) that are explicitly not Domain Events (`14 §2`).

**Why.** A socket that accepts writes must reimplement idempotency, conditional requests, capability denial, rate-limit headers, and the audit path — or quietly skip them. Every realtime platform that lets the socket write ends with two authorization implementations, one of them wrong. Ours has one, and the PEP sits on it (`15 §6`).

**Honest cost.** A round trip on send. `14 §3` prices it: perceived latency stays under 16 ms because the local replica applies optimistically (P2), and the network leg only moves the indicator from pending to committed. We pay milliseconds nobody perceives to keep authorization singular.

### 2.5 Why not GraphQL

GraphQL is a genuinely good fit for part of this problem and we are rejecting it anyway. The fair version of the argument:

**What it would buy us.** The read surface is a graph — Citizen → Memberships → Standing → Societies → Chambers → Threads. Clients on hostile networks (P2) benefit from one round trip returning exactly the shape a screen needs, and a typed, introspectable schema is a real answer to §14's discoverability requirement. If the product were read-dominated with first-party clients only, GraphQL would probably win.

**Why it loses here.**

1. **Cost is unbounded at the edge, and our edge is where authorization lives.** A query language lets a caller compose a request whose cost is not knowable from its shape, and an Agent is precisely the caller that will find the expensive query. Defending it needs cost analysis, depth limits, and persisted-query allowlists — at which point the surface is a fixed set of named operations with extra machinery.
2. **Capability denial does not compose.** A GraphQL response is partially successful by design. Our model is deny-by-default with one missing capability the caller can act on (§7.5). Expressing "you lack `chamber.message.read<in:cham_x>` for 3 of 47 nodes, with a grant path for each" inside a partial-error array is possible and miserable. A 403 naming one capability and one remedy is a better contract.
3. **HTTP features we depend on are lost.** `Idempotency-Key`, `If-Match`, `ETag`, `Retry-After`, `Sunset` are all load-bearing here (§5, §3.4, §8). Mutations over a single POST endpoint discard the cache and concurrency layer, and re-earning it inside the schema is bespoke work with no ecosystem behind it.
4. **The CLI parity gate needs a bounded operation set.** P13's gate (§12.4) enumerates operations and checks each has a CLI binding. "Every query expressible in the schema" is not enumerable, so the gate degrades to "the CLI has a `query` command" — the wrapper-around-another-interface N3 forbids.
5. **Cache locality.** Per-Society partitioning (`10 §4`) makes resource-shaped HTTP caching natural: a URL contains its `society_id`, so edge caching, ETags, and invalidation all follow the shard. A GraphQL POST does not.

**What we take from it anyway.** The two things GraphQL is loved for are addressed directly: `?expand=` for embedding related resources in one round trip and `?fields=` for sparse responses (§5.4). They recover most of the round-trip saving over a bounded, indexable, cacheable query space.

**Reopening condition.** If measured mobile round trips on the Society and Chamber surfaces exceed four per screen after `expand` is in use, we ship a small set of **persisted, Runtime-defined composite reads** — named endpoints, not a query language — before we reconsider GraphQL.

### 2.6 Protocol selection, stated as a rule

| Traffic | Protocol | Reason |
|---|---|---|
| Any documented capability, any front end, third parties | HTTP/JSON | Universality; the P3 audit surface |
| Live state arriving unbidden at a client | WebSocket (Signals) | Push; `14 §2` |
| Extracted-boundary calls inside the Runtime | gRPC | Typed, deadline-propagating, low overhead |
| Agent workloads ≥ ~1k ops/s or needing bidirectional streams | gRPC (public) | Same contract, better encoding |
| Bulk read of a Society's history | HTTP + cursor paging, or gRPC server-stream | Pageable, rate-limitable, resumable |
| Large object bytes | HTTP with content-addressed PUT/GET (`13`) | Range requests, CDN, resumable |
| A first-party GUI shortcut | **None. Forbidden.** | P3 |

---

## 3. Versioning

### 3.1 The scheme

The major version is in the URL path: `/v1/...`. gRPC packages carry it (`fractal.v1.DiscourseService`), and every event payload schema carries `schema_version` independently (`10 §5`).

**Why URL-major.** It is visible in a log line, a `curl` example, a proxy rule, and an incident timeline. Header negotiation (`Accept: application/vnd.fractal.v2+json`) is purer and worse operationally: the version disappears from every artifact read under pressure, and caches must vary on it. We take the ugly URL.

**Within a major, the API is additive only.** New endpoints, new optional request fields, new response fields, new open-enum members, new error codes on operations that declare an open error set. Nothing else.

**At most two majors are ever live.** `v2` GA starts a 24-month clock on `v1`. There is never a `v3` while two majors are already serving; that is a capacity decision about our own operational surface, and it is the reason majors will be rare.

### 3.2 What is a breaking change

Precision here is the whole point. The classifier in CI (§3.5) implements this table; the table is the specification, not a summary of it.

| Change | Breaking? | Notes |
|---|---|---|
| Remove a field from a response | **Yes** | Including one documented as optional. Consumers read what is present. |
| Remove or rename an endpoint, parameter, or field | **Yes** | Rename is remove + add. |
| Make an optional request field required | **Yes** | |
| Make a required response field optional | **Yes** | The consumer's non-null assumption breaks. |
| Add an optional request field with a backward-compatible default | No | The default must preserve prior behaviour exactly. |
| Add a response field | No | Consumers must ignore unknown fields; SDKs enforce this (§13). |
| Narrow a type (`string` → `enum`, `int64` → `int32`) | **Yes** | |
| Widen a type (`int32` → `int64`, `enum` → `string`) | **Yes** | Widening breaks the consumer's parser, not ours. |
| **Add a member to a *closed* enum** | **Yes** | See §3.3. |
| **Add a member to an *open* enum** | No | See §3.3. |
| Remove an enum member | **Yes** | Always, both kinds. |
| Change the meaning of a field without changing its shape | **Yes** | The most dangerous class; caught by review and by the golden corpus, never by a type diff. |
| Change a default value | **Yes** | It is a semantic change wearing a shape-preserving disguise. |
| Tighten validation on an existing field | **Yes** | Requests that succeeded now fail. |
| Loosen validation | No | Unless it widens an enum (above). |
| **Add an error code to an operation's declared set** | Yes if the set is closed; No if open | See §3.4. |
| Change an existing error code's HTTP status or `retryable` flag | **Yes** | Clients branch on both. |
| Reuse an error code for a different cause | **Yes** | The registry is append-only; codes are never recycled. |
| Change collection default ordering | **Yes** | Pagination determinism is a contract (§14.4). |
| Change pagination page-size limits | No | Advertised in the response; clients must not assume. |
| Shorten the idempotency dedupe window | **Yes** | Callers' retry logic depends on 24 hours (`10 §5`). |
| Change a rate-limit tier | No | Capacity, not contract — but it is announced (§8.4). |
| Add a required capability to an existing operation | **Yes** | Working callers begin failing with 403. |
| Remove a required capability | No | Strictly widening. |
| Change an operation from synchronous to `202 Operation` | **Yes** | The response shape and the client's control flow both change. |

### 3.3 Open and closed enums, declared not inferred

Enum widening is the single most argued-about compatibility question, and the honest answer is that it depends on something the schema usually does not state. So we state it. Every enum in the contract is annotated `fn.enum_kind = OPEN | CLOSED`.

- An **open** enum may gain members within a major. Generated SDKs represent it as a type with an `Unknown(String)` variant, and the documentation for every open enum names the behaviour a consumer must implement when it sees an unrecognised member. `ChamberKind`, `AchievementId`, `SignalKind`, and `ProblemCode` are open.
- A **closed** enum may not gain members within a major, because exhaustive handling is required for correctness. `PostingReason` is the case that matters: `11 §2.6` requires every Fraction movement categorized and `17` requires every variant to map to a declared Source or Sink, so a client filing an unknown reason under "other" has broken the audit property. `Visibility`, `MembershipState`, `SocietyStatus`, and `Finality` are also closed.

**Consequence:** adding a Source or Sink to the economy is an API major-version event, or it ships behind a new field. Expensive and correct. `02 §5` budgets two new Sources per phase, so the cost is bounded and visible where the decision is made.

### 3.4 Deprecation

Error sets carry the same distinction: an operation declaring `fn.error_set = OPEN` may gain codes within a major, and clients must default-branch on `retryable`. Operations whose callers branch exhaustively — `POST /v1/transfers` — declare `CLOSED`.

**Notice periods, minimums, no exceptions granted informally:**

| Scope | Notice | Additional requirement |
|---|---|---|
| A field | 180 days | Successor field live before the notice starts |
| An endpoint | 180 days | Successor endpoint live; migration note in the changelog |
| A resource family | 365 days | ADR; named successor; migration guide with runnable examples |
| A major version | 730 days from successor GA | All of the above, plus per-principal migration telemetry |

**Signalling on the wire**, from the first day of the notice period, on every response from the affected route:

```http
HTTP/1.1 200 OK
Deprecation: @1798761600
Sunset: Sat, 31 Jan 2026 00:00:00 GMT
Link: <https://docs.fractalnode.org/v1/migrations/transfers-v2>; rel="deprecation"; type="text/html"
Link: </v2/transfers>; rel="successor-version"
Fn-Warning: 299 fractal-node "Endpoint deprecated. Sunset 2026-01-31. Successor: POST /v2/transfers."
```

`Deprecation` is an RFC 9745 date; `Sunset` is RFC 8594. Both appear together or neither appears. The CLI prints `Fn-Warning` on stderr (`33 §7.2`); SDKs emit it once per route per process, because a warning printed on every call is a warning nobody reads.

**After sunset**, the route answers `410 Gone` with a problem document naming the successor. It never answers `404`: a caller that gets `404` debugs their own URL construction, and a caller that gets `410` reads the migration note.

### 3.5 The compatibility test in CI

```
 PR ──► 1. buf breaking --against 'main'          contract-level structural diff
        2. fn-apidiff (classifier)                applies the §3.2 table, incl. enum/error kinds
        3. golden corpus replay                   ~1,200 recorded (request, response) pairs from
                                                  the last release, replayed against the new build;
                                                  response diffs classified, not eyeballed
        4. SDK compile gate                       6 pinned example programs, one per SDK, must
                                                  compile and pass against the new contract
        5. CLI surface diff                       every removed/renamed command is a break
        6. capability diff                        a newly-required capability on an existing
                                                  operation is a break (§3.2)
              │
              ├── all clear ─────────────────────► merge
              └── any BREAKING ─────────────────► build fails unless the PR carries an
                                                   `api-major` label AND an ADR link AND
                                                   the major version was bumped in the contract
```

Step 3 is the one that catches semantic changes a type diff cannot see. The corpus is captured from the staging environment's real traffic, scrubbed, and pinned per release; a response whose bytes change without a declared reason is a finding a human must classify.

---

## 4. The Resource Map

### 4.1 Conventions

From `01 §9`, restated as rules the generator enforces:

- Paths are plural canonical nouns: `/v1/societies/{society_id}/chambers/{chamber_id}`. Never `channel`, never `room`, never `group`.
- Identifier path segments use the domain's identifier scheme (`11 §6`): `soc_…`, `fct_…`, `lst_…`, `fn1…` for FNIDs, ULIDs for messages and events.
- Every Society-owned resource is nested under `/v1/societies/{society_id}/`. This is P1 in the URL structure: a path that cannot answer "which Society owns you?" is either on the Global Registry (`01 §6`) or is a defect.
- `me` is a legal alias for the calling Citizen's FNID: `/v1/citizens/me`.
- **No endpoint is named with a forbidden verb.** There is no `/update`, no `/manage`, no `/process`, no `/sync`. `PATCH` on a resource maps to a *named domain command*; the HTTP method is transport, the command is the truth, and the command's name comes from `01 §8`.

**Custom methods.** Some operations are state transitions, not CRUD. The rule:

> If the operation produces a durable addressable record, it is a `POST` to a sub-collection (`/exports`, `/purchases`, `/versions`). If it is a state transition on the parent that produces only Domain Events, it is a custom method: `POST /v1/societies/{society_id}:fracture`. The verb after the colon must exist in `01 §8`.

Inventing a `/fractures` collection to avoid a colon would lie about the model — there is no Fracture object, there is a Society that fractured — and the lie would then propagate into the SDKs and the CLI.

### 4.2 Resource families and the complexity budget

`02 §5` allows **three new public API resource families per phase**, and this chapter's map has far more than three phases' worth of paths. The reconciliation is a definition, and it is the honest one:

> **A resource family is a maximal set of resources that share an owning boundary from `10 §3`, are versioned together, and are governed by one capability domain.** Chambers, Threads, and Messages are one family (Discourse, S3). Wallets, Transfers, and Postings are one family (Ledger, S5).

This chapter claims **one exemption**, stated rather than smuggled: the *platform meta-resources* — `/v1/operations`, `/v1/webhooks`, `/v1/schema`, `/v1/capabilities`, `/v1/batch` — do not count. They carry no domain state and are the mechanism by which every other family is delivered. If one ever carries domain state, it counts.

| Family | Boundary | Path root(s) | Phase |
|---|---|---|---|
| Identity | S1 | `/v1/citizens`, `/v1/handles`, `/v1/devices`, `/v1/sessions` | 0 |
| Society | S2 | `/v1/societies`, `…/memberships`, `…/charter` | 0 |
| Discourse | S3 | `…/chambers`, `…/threads`, `…/messages`, `/v1/convergences` | 0 |
| Vault | S4 | `…/vault/objects`, `…/vault/manifests` | 1 |
| Ledger | S5 | `/v1/wallets`, `/v1/transfers`, `…/postings` | 1 |
| Agent | S10 | `…/agents`, `…/envelopes`, `…/policies`, `…/workflows`, `…/runs` | 1 |
| Governance | S9 | `…/charter/versions`, `…/proposals`, `…/votes`, `…/moderation` | 2 |
| Progression | S7 | `/v1/citizens/{id}/progression`, `…/achievements`, `…/unlocks`, `…/standing` | 2 |
| Economy | S6 | `…/sources`, `…/settlements`, `…/contribution` | 2 |
| Asset | S8 | `…/facets`, `/v1/facet-standards` | 3 |
| Extension | S11 | `/v1/extensions`, `…/installs` | 3 |
| Discovery | S13 | `/v1/search`, `/v1/discovery`, `/v1/interests` | 3 |
| Market | S12 | `/v1/listings`, `…/purchases`, `…/payouts` | 6 |
| Signals | S14 | `wss://…/v1/signals`, `/v1/subscriptions` | 0 (protocol) |
| *Platform meta* | — | `/v1/operations`, `/v1/webhooks`, `/v1/schema`, `/v1/capabilities`, `/v1/batch` | 0 (exempt) |

Fourteen domain families across four budgeted phases plus later work. The budget is not decoration: it is why Market is Phase 6 and not Phase 2, and it is what an engineer points at when a fifteenth family is proposed.

### 4.3 The endpoint table

Capability strings use the `domain.resource.verb` form of `15 §4.1`; `12 §7.1` specifies the same lattice. Every string below exists in the generated capability registry (§12.2) — the API never invents one. `—` means an authenticated principal acting on itself: a principal always holds read over its own records, which is P8's minimum rather than an exception to it.

| # | Method | Path | Capability | Idempotency |
|---|---|---|---|---|
| 1 | GET | `/v1/citizens/me` | — | Safe |
| 2 | PATCH | `/v1/citizens/me` | — | `If-Match` required |
| 3 | POST | `/v1/handles` | `society.handle.claim` | Key required |
| 4 | GET | `/v1/capabilities` | — | Safe |
| 5 | GET | `/v1/societies` | `society.read` | Safe |
| 6 | POST | `/v1/societies` | `society.create` (Level ≥ 3) | Key required |
| 7 | GET | `/v1/societies/{sid}` | `society.read` | Safe, ETag |
| 8 | GET | `/v1/societies/{sid}/charter` | `governance.charter.read` | Safe, ETag |
| 9 | POST | `/v1/societies/{sid}/charter/versions` | `governance.charter.propose` | Key required |
| 10 | POST | `/v1/societies/{sid}/charter/versions/{v}:enact` | `governance.charter.enact` | Key required |
| 11 | GET | `/v1/societies/{sid}/memberships` | `society.member.read` | Safe |
| 12 | POST | `/v1/societies/{sid}/memberships` | `society.member.join` | Key required |
| 13 | DELETE | `/v1/societies/{sid}/memberships/{fnid}` | `society.member.leave` \| `society.member.remove` | Naturally idempotent |
| 14 | POST | `/v1/societies/{sid}/chambers` | `chamber.create` | Key required |
| 15 | GET | `/v1/societies/{sid}/chambers/{cid}/threads` | `chamber.thread.read` | Safe |
| 16 | POST | `/v1/societies/{sid}/chambers/{cid}/threads` | `chamber.thread.create` | Key required |
| 17 | GET | `…/threads/{tid}/messages` | `chamber.message.read` | Safe, ETag |
| 18 | POST | `…/threads/{tid}/messages` | `chamber.message.post` | **Key required** |
| 19 | PATCH | `…/messages/{mid}` | `chamber.message.revise` | `If-Match` required |
| 20 | DELETE | `…/messages/{mid}` | `moderation.message.redact` | Naturally idempotent |
| 21 | POST | `/v1/convergences` | `society.convergence.create` (Level ≥ 2) | Key required |
| 22 | POST | `/v1/convergences/{cvid}:crystallize` | `society.convergence.crystallize` | Key required → Operation |
| 23 | POST | `/v1/societies/{sid}:fracture` | `society.fracture` (Society Level ≥ 5) | Key required → Operation; dry run mandatory |
| 24 | GET | `/v1/wallets/{wid}` | `wallet.read` | Safe |
| 25 | POST | `/v1/transfers` | `wallet.transfer<=…>` | **Key required** |
| 26 | GET | `/v1/societies/{sid}/postings` | `wallet.posting.read` | Safe |
| 27 | PUT | `/v1/societies/{sid}/vault/objects/{path}` | `vault.object.write<path:…>` | Naturally idempotent (content-addressed) |
| 28 | GET | `/v1/societies/{sid}/vault/objects/{path}` | `vault.object.read<path:…>` | Safe, ETag, Range |
| 29 | POST | `/v1/societies/{sid}/facets` | `facet.mint` (Level ≥ 6) | Key required |
| 30 | POST | `/v1/facets/{fid}:transfer` | `facet.transfer` | Key required |
| 31 | POST | `/v1/societies/{sid}/agents` | `agent.enroll` (Level ≥ 4) | Key required |
| 32 | POST | `/v1/societies/{sid}/envelopes` | `agent.envelope.grant` | Key required; elevated context |
| 33 | DELETE | `/v1/societies/{sid}/envelopes/{eid}` | `agent.envelope.revoke` | Naturally idempotent |
| 34 | POST | `/v1/societies/{sid}/workflows/{wid}:invoke` | `agent.workflow.invoke` | Key required → Operation |
| 35 | POST | `/v1/societies/{sid}/installs` | `society.extension.install` (Level ≥ 7) | Key required → Operation |
| 36 | POST | `/v1/listings/{lid}/purchases` | `market.listing.purchase` | **Key required** |
| 37 | GET | `/v1/citizens/{fnid}/progression` | — (self) \| `society.member.read` | Safe |
| 38 | GET | `/v1/societies/{sid}/events?since={seq}` | `audit.read` \| `society.read` | Safe |
| 39 | POST | `/v1/societies/{sid}/exports` | `society.export` | Key required → Operation |
| 40 | POST | `/v1/societies/{sid}/webhooks` | `society.webhook.subscribe` | Key required |
| 41 | GET | `/v1/search?q=…` | `society.read` (scoped) | Safe |
| 42 | GET | `/v1/operations/{oid}` | — (creator-scoped) | Safe |

Four things here are worth reading twice. Endpoints 18, 25, and 36 refuse a request without an `Idempotency-Key` — a duplicate there is a message posted twice, money moved twice, or a purchase charged twice, and client discipline is not a control. Endpoint 23 will not execute without a prior dry run in the same correlation (`11 §3.2`). Endpoint 13 is one path for leaving and for being removed, distinguished by capability, because they are one state transition with different authority. Endpoint 4 is the one an Agent calls first (§14.2).

---

## 5. Request and Response Conventions

### 5.1 Envelope shape

A single resource is returned at the top level with no wrapper. A collection is:

```json
{
  "items": [ { "…": "…" } ],
  "page": {
    "next_cursor": "eyJzIjoiNDQ3MSIsImkiOiIwMUhZ…",
    "has_more": true,
    "page_size": 50
  }
}
```

**Why no universal wrapper.** A `{ "data": …, "meta": … }` envelope costs a level of indirection on every read, in every language, forever, for a need only collections and warnings have. Collections get `page`; warnings ride in `Fn-Warning`; operation metadata is an `Operation`. The honest cost: no single place for a future cross-cutting response field, and if we need one it will be a header, since a body change would be breaking (§3.2).

**Field naming** is `snake_case` in JSON, matching the protobuf field names exactly so the two encodings share one mental model and one golden corpus.

### 5.2 Pagination is cursor-based

Every collection paginates with `?cursor=…&limit=…`. Default limit 50, maximum 200, advertised in `page.page_size`. Offset pagination is not offered anywhere.

**Why.** Three reasons, in order of how much they matter here.

*Correctness under concurrent insertion.* Our collections are append-heavy — messages, events, and postings all grow at the head while a client pages. An insert during offset paging silently duplicates or skips a row, and for a client reconciling a local replica (P2) a skipped Posting is a wrong balance. A cursor encodes a position in a total order, not a count from the start.

*Determinism as an agent requirement.* §14.4 makes this contractual: paging a collection twice must yield the same sequence. A cursor pins the sort key, the tiebreaker, a hash of the active filters, and the snapshot `seq`. Changing filters mid-page is refused, not quietly honoured.

*Cost.* `OFFSET n` in Postgres is O(n). At 200,000 messages in an active Chamber, deep offset paging is a scan; a keyset cursor is an index seek regardless of depth.

**Cursor format.** Opaque, base64url of a signed `{v, sort_key, tiebreaker_id, filter_hash, snapshot_seq}`. Signed, so a tampered cursor is rejected rather than executed. Opaque, so its structure can change without a breaking change; a client that decodes one is outside the contract.

**Honest cost.** No page numbers and no exact totals. Where a count is genuinely required (members, listing results) the resource carries a denormalized count with its staleness stated. We prefer an honest approximate count to an exact one that costs a full scan per page view.

### 5.3 Filtering and sorting

Filters are **typed, named, closed parameters per collection** — `?state=active`, `?since=…`, `?author=fn1…` — never a query DSL in a string. A general filter language is an unbounded query planner exposed to unauthenticated callers, and the first Agent to find a full-scan predicate will find it before we do. Every filter must name the index that serves it; a filter with no index does not ship.

Sorting is a closed enum per collection (`?sort=created_at.desc`), with one default per collection that is part of the contract (§3.2: changing it is breaking). Arbitrary multi-column sorting is not offered, for the same index-discipline reason.

### 5.4 Sparse fieldsets, expansion, partial responses

- `?fields=handle,display_name,level` restricts the response to a whitelisted subset. Requesting a field that does not exist is a `422`, never a silent omission — a typo that silently returns less data is a defect generator.
- `?expand=society,author.persona` embeds related resources, to a maximum depth of 2 and a maximum of 4 expansions per request, both advertised in the schema. This is the GraphQL round-trip saving without the GraphQL cost surface (§2.5).
- `Prefer: return=minimal` on a write returns `204` with a `Location` header instead of the representation. Agents doing bulk writes use this; it removes a meaningful share of egress.

`ETag` varies with `fields` and `expand`, which fragments the cache. That is the honest cost of both features and the reason expansion is capped.

### 5.5 Conditional requests

Every addressable resource carries a **strong** `ETag` derived from `(society_id, aggregate_id, aggregate_version)` — not a body hash, because a body hash changes when the serializer changes and an aggregate version does not.

- `If-None-Match` on `GET` → `304 Not Modified`. The local-first clients (P2) use this on every replica reconciliation, which is where the bandwidth saving actually lands.
- `If-Match` on `PATCH`/`PUT`/`DELETE` → optimistic concurrency. A mismatch is `412 Precondition Failed` with a problem document naming the current version. `If-Match` is **required** on every field-level amendment (endpoints 2 and 19 above); lost-update-by-default is not a thing we ship.

### 5.6 The `Idempotency-Key` contract

```http
POST /v1/transfers HTTP/1.1
Idempotency-Key: 01HYQ3M8Z4N7VJ2K9C6TQ8XB0P
```

| Rule | Behaviour |
|---|---|
| Key format | ULID or UUIDv4, client-generated, 16–64 chars |
| Dedupe scope | `(principal, idempotency_key)`, per `10 §5` |
| Window | 24 hours from first receipt |
| Replay, same request fingerprint | The **original** response, byte-identical, with `Idempotency-Replayed: true` |
| Replay, different fingerprint | `422 idempotency_key_reused`, naming the field that differs |
| In flight | `409 idempotency_in_flight` with `Retry-After: 1` — never a second execution, never a block |
| Missing on a required endpoint | `400 idempotency_key_required` |
| Fingerprint | Hash of method, path, and the canonicalized body — deliberately not the headers, so a retry with a refreshed token still deduplicates |

The last row matters more than it looks. Access tokens live 10 minutes (`12 §5.2`); a retry after a network stall frequently carries a new token. A fingerprint over headers would treat that as a different request and execute the transfer twice.

**A key is also honoured across protocols.** The same key sent over gRPC metadata dedupes against an HTTP call. There is one command path, so there is one dedupe table.

### 5.7 The standard header set

| Header | Direction | Purpose |
|---|---|---|
| `Authorization: DPoP <token>` + `DPoP: <proof>` | → | Device-bound auth (§7.1) |
| `Idempotency-Key` | → | §5.6 |
| `If-Match` / `If-None-Match` | → | §5.5 |
| `Prefer: return=minimal\|representation` | → | §5.4 |
| `Fn-Society: soc_…` | → | Optional disambiguation when a resource is reachable from more than one Society context |
| `Fn-Correlation-Id: <ulid>` | ↔ | Client-supplied; becomes `correlation_id` on every emitted event (`10 §5`) and every trace span |
| `Fn-Dry-Run: true` | ← | Confirms the response describes effects that were not applied (§14.5) |
| `Idempotency-Replayed: true` | ← | This is a replay, not a new execution |
| `ETag`, `Last-Modified` | ← | §5.5 |
| `RateLimit`, `RateLimit-Policy`, `Retry-After` | ← | §8.2 |
| `Deprecation`, `Sunset`, `Link` | ← | §3.4 |
| `Fn-Warning` | ← | Non-semantic advisories; never carries meaning a client must act on |
| `Fn-Request-Id` | ← | The Runtime's identifier for this call; quoted in support and in every log line |

`Fn-Correlation-Id` is the field that makes an incident tractable: a Citizen reporting "my transfer vanished" gives one identifier that resolves to a trace, a command, an event, and a Posting.

---

## 6. The Error Model

### 6.1 Shape

Every non-2xx response is `application/problem+json` per RFC 9457, with a fixed set of Fractal Node extensions.

```http
HTTP/1.1 403 Forbidden
Content-Type: application/problem+json
Fn-Capability-Required: wallet.transfer<=100FRC/day
Fn-Request-Id: 01HYQ3MB1F8T0J4W5Z2Q7R9N3D

{
  "type":     "https://spec.fractalnode.org/problems/capability_denied",
  "title":    "Capability not granted",
  "status":   403,
  "code":     "capability_denied",
  "detail":   "Transfer refused — Envelope env_01HY… permits wallet.transfer<=100FRC/day; 140 FRC requested in the last 24h window. Raise the limit in Policy, or transfer 60 FRC.",
  "instance": "/v1/transfers",
  "retryable": false,
  "cause": {
    "pep_reason":          "LimitExceeded",
    "required_capability": "wallet.transfer<=140FRC/day",
    "held_capability":     "wallet.transfer<=100FRC/day",
    "envelope":            "env_01HY…",
    "society":             "soc_01H8Z…",
    "window_resets_at":    "2026-09-04T11:02:17Z"
  },
  "remedy": {
    "action":   "amend_policy",
    "uri":      "/v1/societies/soc_01H8Z…/policies/pol_01HY…",
    "cli":      "fn policy limit set wallet.transfer 200FRC/day --society soc_01H8Z…",
    "requires": ["governance.policy.enact", "elevated_context"]
  },
  "correlation_id": "01HYQ3M8Z4N7VJ2K9C6TQ8XB0P"
}
```

`type` is a stable URI; `code` is the registry key clients branch on, because branching on a URI means branching on a URL that may move. `detail` is human-facing prose in the house voice. `cause` and `remedy` are machine-facing and never contain prose a client must parse.

### 6.2 Cause, then remedy, never an apology

`33 §7.3` sets the register and makes a lint of it: *"Oops! Something went wrong" is a banned string, enforced by lint.* The API-side rules:

1. **Every `detail` states what happened, why, and the next action.** The example above is verbatim the shape `33` gives: *"Transfer refused — Envelope permits 100 FRC/day, 140 requested. Raise the limit in Policy, or send 60 FRC."*
2. **No apology, no blame, no exclamation.** "Sorry", "unfortunately", "please try again later", "something went wrong", "an unexpected error occurred" — all lint failures in CI against the contract's example corpus and against production error strings sampled from telemetry.
3. **`remedy` is structured and actionable.** Where a remedy exists it names an action, a URI, a CLI invocation, and the capabilities the remedy itself requires. A remedy the caller cannot perform says so, and names who can.
4. **`internal_fault` is the one code with no remedy.** Its `detail` states that the Runtime failed, gives the `Fn-Request-Id`, and states whether the operation may have been applied. It never speculates: a caller's next move depends on knowing whether a write landed.

### 6.3 Retryability

`retryable` is a boolean in every problem document, and it is a contract, not a hint (§3.2: changing it is breaking).

- `retryable: true` — the same request with the same `Idempotency-Key` may be retried after `Retry-After` if present, otherwise on the SDK's default backoff.
- `retryable: false` — retrying will fail identically. SDKs do not retry these, and an SDK that does is a defect.
- **Never absent.** A missing flag would push the decision back onto per-code client tables, which is exactly the coupling the registry exists to remove.

### 6.4 The closed error-code registry

The registry is append-only within a major, codes are never recycled (§3.2), and every code appears here or does not exist.

| Code | Status | Retryable | Cause and remedy |
|---|---|---|---|
| `validation_failed` | 400 | no | A field failed a stated constraint. `cause.fields[]` names each. |
| `idempotency_key_required` | 400 | no | The endpoint requires one. Generate a ULID and resend. |
| `unauthenticated` | 401 | no | No credential, or one that failed binding. See `WWW-Authenticate`. |
| `token_expired` | 401 | yes | Refresh and resend (§7.4). |
| `dpop_proof_invalid` | 401 | no | Proof missing, replayed, or not bound to the token's key. |
| `capability_denied` | 403 | no | `cause.required_capability` names what was lacking. |
| `envelope_expired` | 403 | no | The Envelope's `expires_at` has passed. Renewal is a new Envelope (`15 §4.4`). |
| `envelope_revoked` | 403 | no | Revocation is retroactive to in-flight work (`15 §4.4`). |
| `unlock_required` | 403 | no | A Level, Trust, Standing, or Achievement gate is unmet (`18 §5`). `cause.gate` states it. |
| `charter_forbids` | 403 | no | The Society's Charter denies this to this role. |
| `confirm_required` | 403 | no | The action is in `confirm_classes`. `remedy.uri` is the confirmation request. |
| `blast_radius_exceeded` | 403 | no | The action would reach more principals than the Envelope permits (`15 §4.3`). |
| `agent_halted` | 403 | no | The Agent is halted. Only its Operator can resume it. |
| `society_kill_switch` | 403 | no | Society-wide halt engaged. |
| `insufficient_stake` | 403 | no | The action class requires a Stake this principal has not posted. |
| `not_found` | 404 | no | No such resource, **or** the caller may not know it exists. Deliberately conflated (§6.5). |
| `gone` | 410 | no | Sunset endpoint or dissolved resource. `remedy` names the successor. |
| `conflict` | 409 | no | The command contradicts current state; `cause.expected`/`cause.actual`. |
| `idempotency_in_flight` | 409 | yes | The first execution is running. `Retry-After: 1`. |
| `precondition_failed` | 412 | no | `If-Match` mismatch. `cause.current_etag` is given; re-read and re-apply. |
| `payload_too_large` | 413 | no | `cause.limit_bytes`. Use the Vault upload path for object bytes. |
| `unsupported_media_type` | 415 | no | |
| `idempotency_key_reused` | 422 | no | Same key, different request. `cause.differing_fields[]`. |
| `cursor_invalid` | 422 | no | Malformed, tampered, or filter-mismatched cursor. Restart paging. |
| `cursor_expired` | 422 | no | Snapshot aged out. Restart paging; `cause.max_age_s`. |
| `insufficient_funds` | 422 | no | `cause.available`/`cause.required` in quanta. |
| `dry_run_required` | 422 | no | This operation refuses execution without a prior dry run in the same correlation. |
| `rate_limited` | 429 | yes | Per-principal bucket exhausted. `Retry-After` in seconds. |
| `quota_exceeded` | 429 | no | A per-Society quota, not a rate. Retrying does not help; `remedy` names the quota. |
| `internal_fault` | 500 | yes | The Runtime failed. `cause.write_state` is `applied`, `not_applied`, or `unknown`. |
| `dependency_unavailable` | 503 | yes | A port (`10 §7`) is down. `Retry-After` present. |
| `shed` | 503 | yes | Load shed at the gateway. Backoff with full jitter (`10 §10`). |

### 6.5 Two deliberate design points

**`404` is overloaded on purpose.** A distinct `403` for "this exists but you may not see it" is an existence oracle: an adversary enumerates private Society handles by status code. A caller lacking read authority on the *parent* gets `404`; a caller who can read the parent but not act gets `403`, because there the existence is already known and the missing capability is what the caller needs. The trade is harder diagnosis of legitimate permission problems; `Fn-Request-Id` resolves to a log line stating the true reason.

**Denials are Domain Events, not log lines.** `10 §8` and `15 §6` require it: a denied Agent action emits `AgentActionBlocked` and counts against the Agent's Trust. The HTTP error renders that event rather than replacing it. An error surface that only logs cannot be audited.

---

## 7. Authentication and Authorization on the Wire

`12 §5` fixes the model; this section fixes its HTTP rendering. Nothing here relaxes anything there.

### 7.1 Sessions are passkey-derived and device-bound

```
  passkey assertion ──► device key proves possession ──► SESSION
                                     │
              ┌──────────────────────┴──────────────────────┐
              ▼                                             ▼
     access token, 10 min                        refresh token, 30 d
     DPoP-bound to device_id + session_id        rotating, reuse-detected,
     audience = gateway                          DPoP-bound to the device key
```

**Bearer tokens are refused everywhere** (`12 §5.2`). Every call carries `Authorization: DPoP <access_token>` plus a `DPoP` proof header (RFC 9449) signed by the device key over the method, the URI, a nonce, and a timestamp. A stolen token alone is inert, which is the entire point.

```http
POST /v1/societies/soc_01H8Z.../chambers/cham_01H9A.../threads/thr_01H9B.../messages HTTP/1.1
Host: api.fractalnode.org
Authorization: DPoP eyJhbGciOiJFZERTQSIsInR5cCI6ImF0K2p3dCJ9...
DPoP: eyJ0eXAiOiJkcG9wK2p3dCIsImFsZyI6IkVkRFNBIiwiandrIjp7...
Idempotency-Key: 01HYQ3M8Z4N7VJ2K9C6TQ8XB0P
Fn-Correlation-Id: 01HYQ3M8Z4N7VJ2K9C6TQ8XB0P
Content-Type: application/json

{ "body": { "text": "Charter v7 supersedes v6 at block 44812." }, "reply_to": null }
```

```http
HTTP/1.1 201 Created
Location: /v1/societies/soc_01H8Z.../chambers/cham_01H9A.../threads/thr_01H9B.../messages/01HYQ3MC…
ETag: "aggr:thr_01H9B:4472"
RateLimit: limit=300, remaining=287, reset=41
Fn-Request-Id: 01HYQ3MC2P8V1K5X6A3R8S0T4E
```

**Elevated context** (`12 §5.2`) is a 5-minute credential minted by a fresh passkey assertion, required for adding a device, key rotation, changing recovery, granting an Envelope, and Transfers above the confirm threshold. On the wire it is a distinct token audience; a call needing it without one fails `403 confirm_required` with `remedy.uri` pointing at the elevation flow.

### 7.2 The device-code flow for the CLI

The CLI is a first-class front end (N3), so it enrolls as a device rather than holding a pasted secret. No secret crosses the wire in either direction.

```
  fn login
    │
    ├─► POST /v1/device-authorizations
    │      { "client": "fn-cli/1.4.2", "host": "workstation-01",
    │        "requested_capabilities": ["society.read", "chamber.message.post", …] }
    │
    │   ◄── 201 { "device_code": "…", "user_code": "FRAC-8Q2M",
    │             "verification_uri": "https://fractalnode.org/link",
    │             "interval": 5, "expires_in": 600 }
    │
    ├─► Terminal prints FRAC-8Q2M; opens the browser where it can
    │
    ├─► Citizen approves in an authenticated GUI session, seeing exactly which
    │   capabilities this CLI requests, on which host, and for how long
    │
    ├─► CLI generates its OWN device keypair, enrolls it as a DeviceRecord
    │   (platform = Cli), and proves possession in the token exchange
    │
    └─► POST /v1/device-authorizations/{device_code}:exchange   (polled at `interval`)
           ◄── 428 authorization_pending   (retryable, Retry-After: 5)
           ◄── 429 slow_down               (interval doubled)
           ◄── 200 { access_token, refresh_token, device_id, granted_capabilities }
```

`granted_capabilities` is the intersection the Runtime computed, never the set the CLI requested (`15 §4.2`). A CLI that asked for more than the Citizen holds is told what it actually got, in the capability grammar, at login — not on its first denied call.

### 7.3 API keys are pointers to Envelopes

For headless and CI contexts, `12 §5.3` is categorical: **an API key is the credential for an Envelope, not a bearer god-token.** It is Society-scoped, capability-limited, rate-limited, mandatorily expiring within 90 days, individually revocable, and audited via `envelope_ref` on every event it produces.

```http
Authorization: FN-Key fnk_01HYQ3.../<secret>
```

| Property | Value |
|---|---|
| `key_id` | Appears in every audit line; the secret never does |
| Authority | Exactly the referenced Envelope's `CapabilitySet` — no more, ever |
| Scope | One Society (P1) |
| Lifetime | Mandatory `expires_at` ≤ 90 days |
| Revocation | Immediate, retroactive to in-flight actions (`15 §4.4`) |
| Rotation | Two keys may reference one Envelope during a rotation window |
| Presentation | The secret is displayed exactly once, at creation |

**I-12.6 restated as an API invariant:** no credential this API issues grants unscoped authority. A `**` CapabilitySet cannot be created because attenuation requires a grantor who holds `**` and no such principal exists — including our own operations tooling.

### 7.4 Token lifetimes

| Token | Lifetime | Binding | Revocation | Refresh |
|---|---|---|---|---|
| Access | 10 min | DPoP to device key | Expiry | Via refresh token |
| Refresh | 30 d, rotating | DPoP to device key | Immediate; reuse kills the session family | Single-use; returns a new pair |
| Elevated | 5 min | Fresh passkey assertion | Expiry | Not refreshable — re-assert |
| Device-code | 10 min | — | Consumed on exchange | Not refreshable |
| Agent session | ≤ Envelope `expires_at`, max 90 d | Agent device key | Immediate on Envelope revocation | Not refreshable past the Envelope |
| API key | ≤ 90 d | Key secret + source constraints | Immediate | Rotation, not refresh |

Refresh is `POST /v1/sessions:refresh`, single-use, returning a new pair. Presenting a consumed refresh token is treated as compromise: the entire session family is killed and a security Signal is delivered to every device (`14 §11`, an `Urgent` class).

### 7.5 The exact denial shapes

**No credential, or a credential that failed binding — 401.**

```http
HTTP/1.1 401 Unauthorized
WWW-Authenticate: DPoP algs="EdDSA", realm="fractal-node",
                  error="invalid_token",
                  error_description="Access token expired at 2026-09-03T10:41:02Z."
DPoP-Nonce: 8Q2MvR3kT1
Content-Type: application/problem+json

{ "type":"https://spec.fractalnode.org/problems/token_expired",
  "code":"token_expired", "status":401, "retryable":true,
  "detail":"Token expired 2026-09-03T10:41:02Z. Refresh at POST /v1/sessions:refresh and resend." }
```

**Authenticated but not authorized — 403, and it names the capability.**

```http
HTTP/1.1 403 Forbidden
Fn-Capability-Required: chamber.message.post<in:cham_01H9A>
Content-Type: application/problem+json

{ "type":"https://spec.fractalnode.org/problems/capability_denied",
  "code":"capability_denied", "status":403, "retryable":false,
  "detail":"Post refused — Envelope env_01HY… holds chamber.message.read in cham_01H9A, not chamber.message.post. Request the capability from the Operator, or post in a Chamber the Envelope covers.",
  "cause": {
    "pep_reason":"NotGranted",
    "required_capability":"chamber.message.post<in:cham_01H9A>",
    "held_capabilities":["chamber.message.read<in:*>","moderation.flag<in:*,<=200/day>"],
    "envelope":"env_01HY…","society":"soc_01H8Z…"
  },
  "remedy": {
    "action":"request_capability",
    "uri":"/v1/societies/soc_01H8Z…/envelopes/env_01HY…/requests",
    "cli":"fn envelope request chamber.message.post --in cham_01H9A --envelope env_01HY…",
    "requires":["agent.envelope.grant"],
    "approver":"operator"
  } }
```

Two rules govern this shape and both are deliberate.

**`WWW-Authenticate` appears on 401 only.** RFC 9110 permits it on 403; we decline, because a challenge on an authorization failure invites a re-authentication loop that cannot succeed — the credential was fine, the authority was not. The 403 carries `Fn-Capability-Required`, one header a shell pipeline reads without parsing JSON.

**`held_capabilities` is disclosed to the calling principal only**, never another's. `pep_reason` mirrors the `15 §6` deny enum exactly — `AgentHalted`, `SocietyKillSwitch`, `NoAuthority`, `NotGranted`, `Forbidden`, `LimitExceeded`, `BlastRadius`, `ConfirmRequired`, `InsufficientStake` — so failure handling and the audit trail speak one vocabulary.

---

## 8. Rate Limiting and Quotas

### 8.1 Two mechanisms that are frequently confused

**Rate limits** are per-principal token buckets protecting the Runtime from burst; they refill, so retrying works. **Quotas** are per-Society ceilings — Vault bytes, Agent runs, Extension installs — that do not refill on a timer, so retrying does not help and the remedy is a governance or economic action. Hence two codes with different `retryable` values.

```
   request
      │
      ▼
 ┌──────────────────┐  no   ┌──────────────────┐  no   ┌──────────────────┐
 │ principal bucket │──────►│ Agent bucket     │──────►│ Society quota    │
 │ (token bucket)   │       │ (tighter; per    │       │ (counter, no     │
 │                  │       │  Envelope too)   │       │  refill)         │
 └────────┬─────────┘       └────────┬─────────┘       └────────┬─────────┘
     exhausted                  exhausted                  exhausted
          │                          │                          │
          ▼                          ▼                          ▼
   429 rate_limited          429 rate_limited           429 quota_exceeded
   Retry-After: <s>          Retry-After: <s>           retryable: false
```

Gateway buckets protect the Runtime. The PEP's limits (`15 §4.3` — per-capability rate, daily totals, spend caps) are evaluated in the command transaction and protect the Society. Neither substitutes for the other, and `15 §6` is explicit that the gateway does not authorize.

### 8.2 Response headers

RFC 9331 `RateLimit` fields, on every response — not only on 429, because a client that only learns its budget at exhaustion cannot pace itself.

```http
RateLimit: limit=300, remaining=41, reset=27
RateLimit-Policy: 300;w=60;burst=60;comment="citizen-l6", 20;w=1
Retry-After: 27
```

Two windows are advertised: a sustained per-minute rate and a per-second burst ceiling. `comment` names the tier, which is how a caller diagnoses "why is my budget 300" without a support conversation.

### 8.3 429 semantics

`Retry-After` on `rate_limited` is a whole number of seconds computed from the bucket's actual refill, never a constant. SDKs honour it and add full jitter on top — a fleet of Agents retrying precisely at `reset` is a self-inflicted thundering herd. On `quota_exceeded` it is absent, `retryable` is false, and `remedy` names the quota, its consumption, and the action that raises it.

`10 §10` is categorical about the alternative: shed load at the gateway with `429` and `Retry-After`, never by silently dropping. A dropped request is indistinguishable from a lost one, and a client cannot reason about a difference it cannot observe.

### 8.4 Tiering by Level, without arbitrariness

`18 §5.1` sets the numbers and this chapter invents none: a programmatic credential unlocks at **Level 3, 60 req/min**, rising to **300 at Level 6** and **600 at Level 9**. The question is how to apply that without the API feeling capricious.

**The rule that removes the arbitrariness:** *a session-authenticated Citizen always gets the baseline interactive budget regardless of Level; Level gates the issuance of programmatic credentials and their sustained rate.* A Level 0 Citizen's GUI, desktop, and mobile all run at full interactive speed on day one. Level 3 unlocks pointing a *program* at the API; Levels 6 and 9 raise how hard it may push. That is what the gate is for: `18 §1`'s job J1 makes progression the schedule on which deny-by-default relaxes, and a ninety-second-old account with a 600 req/min programmatic credential is a spam engine. The gate is on automation volume, not on being new.

| Tier | Sustained | Burst | Gate |
|---|---|---|---|
| Interactive session (any Level) | 120 req/min | 30 req/s | Authenticated session |
| Programmatic, Level 3 | 60 req/min | 10 req/s | `18 §5.1` |
| Programmatic, Level 6 | 300 req/min | 20 req/s | `18 §5.1` |
| Programmatic, Level 9 | 600 req/min | 40 req/s | `18 §5.1` |
| Agent (per Envelope) | 25% of the Operator's tier, floor 30 req/min | 5 req/s | `10 §10`, `15 §4.3` |
| Unauthenticated (public reads) | 30 req/min per source | 5 req/s | — |

Agents get a tighter bucket than their Operator by decree (`10 §10`). The 25% figure is a published starting parameter, tuned against measurement and announced when it changes; the *structure* — an Agent is a fraction of its Operator, never a peer — does not.

**Every tier is published in `GET /v1/capabilities`** alongside the caller's current consumption. A limit a caller cannot discover before hitting it is a limit that feels arbitrary; one they can query is a budget.

### 8.5 Society quotas

Per-Society counters — Vault bytes, media per day, Agent runs, Extension installs, Chamber count, member count — are set by Society Level (`11 §2.3`) and by economic settlement (`17`). `13 §6` fixes the rule that matters most: **writes are refused with a typed error; reads and exports are never blocked.** Data is never held hostage — a P2 and P9 commitment expressed as an API behaviour, so `quota_exceeded` is unreachable on any read or export route.

---

## 9. The Realtime API

`14 §2` specifies the Signal protocol, its subscription rules, its backpressure ladder, and its replay ring. This section is the client-facing contract only, and adds nothing to that specification.

### 9.1 Connection and authentication

```
wss://api.fractalnode.org/v1/signals
```

The socket authenticates with the same DPoP-bound access token as HTTP, presented in the first `Hello` frame rather than a query string — a token in a URL lands in proxy logs and browser history. The handshake fails closed: an invalid token gets `Bye{Protocol}` before any subscription is accepted.

Frames are binary CBOR behind a fixed 8-byte header (`14 §2`). Every frame kind is registered in the same schema registry as Domain Events, and the JSON rendering below is the debugging projection the CLI prints under `--trace`, not a second wire format.

### 9.2 Frames

Client → Relay: `Hello`, `Subscribe`, `Resume`, `Ack`, `Presence`, `Typing`, `Read`.
Relay → Client: `Welcome`, `Signal`, `Gap`, `PresenceSet`, `Shed`, `Bye`.

```
 ┌──────────┬──────────┬──────────┬──────────────────────────────────────┐
 │ ver (u8) │ kind(u8) │ flags(u16)│ len (u32)  │  CBOR payload           │
 └──────────┴──────────┴──────────┴──────────────────────────────────────┘
```

A `Signal` frame carries `{ society, seq, kind, body }`, where `kind` is the registered event kind and `body` is validated against that kind's generated JSON Schema (§12.2). A client that can read the schema can decode a Signal it has never seen.

### 9.3 Subscribe, resume, heartbeat, backpressure

**Subscribe.** Scopes are `Society`, `Chamber`, `Thread`, `Convergence`, or `Self`. Every scope resolves to exactly one Society or to the Citizen's own inbox (P1). There is no wildcard above a Society for any principal, including first-party operations tooling. The cap is 200 scopes per connection.

**Resume.** The client persists the highest **contiguous** applied `seq` per Society; contiguity is what makes a cursor a safe resume point. On reconnect it sends `Resume{session, cursors[]}` after full-jitter backoff `U(0, min(30s, 0.5s·2^attempt))`. Cursors inside the replay ring (4,096 events or 15 minutes, whichever is less) yield `Welcome` plus missed Signals; anything older yields a `Gap`, and the client pulls `GET /v1/societies/{sid}/events?since={seq}` then resumes live. A `Gap` is the protocol returning the client to the authoritative path, not a failure.

**Heartbeat.** Relay ping every 15 s; a client missing two consecutive pongs is closed. Clients send `Ack{society, seq}` at least every 10 s or every 32 frames. `Ack` is flow control, not read state — conflating the two is how read receipts end up wrong.

**Backpressure.** The outbound queue is bounded (256 frames or 1 MiB). On fill, in this fixed order: coalesce latest-per-subject (presence, typing, read marks) → `Shed{class}` with notice → `Gap` the durable stream → close after 30 s undrained. P0 Signals — `ChamberMessagePosted`, `TransferSettled`, `EnvelopeRevoked` — are never dropped, only gapped.

### 9.4 A worked session

```
C ──► Hello      { proto: 1, device: "dev_01HY…", caps: { cbor: true, zstd: true } }
S ◄── Welcome    { session: "ses_01HY…", replay_window: 4096, limits: { scopes: 200,
                   queue_frames: 256, ack_interval_ms: 10000 } }

C ──► Subscribe  { scopes: [ Society("soc_01H8Z…"), Thread("soc_01H8Z…","thr_01H9B…"), Self ],
                   since:  [ { society: "soc_01H8Z…", seq: "4471" } ] }
S ◄── Signal     { society: "soc_01H8Z…", seq: "4472",
                   kind: "discourse.message.posted.v1",
                   body: { message_id: "01HYQ3MC…", thread_id: "thr_01H9B…",
                           author: "fn1qz…", envelope_ref: null } }
C ──► Ack        { society: "soc_01H8Z…", seq: "4472" }

     ── 40 s of silence; Relay pings, client pongs ──

S ◄── Signal     { society: "soc_01H8Z…", seq: "4473",
                   kind: "ledger.transfer.settled.v1", body: { … } }

     ── network partition; client backs off U(0, 0.5s), U(0, 1s), U(0, 2s) ──

C ──► Resume     { session: "ses_01HY…", cursors: [ { society: "soc_01H8Z…", seq: "4473" } ] }
S ◄── Welcome    { session: "ses_01HY…", replay_window: 4096, limits: { … } }
S ◄── Gap        { society: "soc_01H8Z…", from_seq: "4473", to_seq: "9310",
                   reason: "RingOverrun" }

     ── client pulls the authoritative path ──
C ──► GET /v1/societies/soc_01H8Z…/events?since=4473&limit=200   (repeat until has_more=false)
C ──► Ack        { society: "soc_01H8Z…", seq: "9310" }
S ◄── Signal     { society: "soc_01H8Z…", seq: "9311", … }        ← live again

     ── an Envelope is revoked while an Agent's socket is open ──
S ◄── Signal     { society: "soc_01H8Z…", seq: "9312",
                   kind: "agent.envelope.revoked.v1", body: { envelope: "env_01HY…" } }
S ◄── Bye        { reason: "Revoked" }
```

The last two frames are the pattern worth noting: revocation is retroactive and immediate (`15 §4.4`), so the Relay does not wait for the Agent to notice. It delivers the fact and closes the socket.

---

## 10. Bulk, Long-Running, and Streaming

### 10.1 Batch

`POST /v1/batch` accepts up to 50 sub-requests sharing one authentication context. Each carries its own path, body, and `Idempotency-Key`; each gets its own status and problem document.

```json
{ "requests": [
    { "id": "a", "method": "POST", "path": "/v1/societies/soc_…/chambers/cham_…/threads/thr_…/messages",
      "idempotency_key": "01HY…A", "body": { "…": "…" } },
    { "id": "b", "method": "PATCH", "path": "/v1/citizens/me", "if_match": "\"aggr:cit_…:88\"",
      "body": { "display_name": "…" } } ] }
```

**Batch is explicitly not atomic**, and every result says so. There is no generic transactional batch, because a multi-write transaction across aggregates is a distributed transaction wearing an API costume; `11 §5` answers that with named sagas and explicit compensation. Where atomicity is genuinely required — a Fracture's treasury division, a purchase — a domain operation provides it. Batch amortizes round trips and claims nothing more.

### 10.2 Long-running operations

Anything that cannot complete inside the p99 request budget returns `202 Accepted` with an `Operation` resource.

```http
HTTP/1.1 202 Accepted
Location: /v1/operations/op_01HYQ3N4…
Retry-After: 2
```

```json
{ "operation_id": "op_01HYQ3N4…",
  "kind": "society.fracture",
  "state": "running",
  "society": "soc_01H8Z…",
  "created_at": "2026-09-03T10:44:02Z",
  "progress": { "phase": "treasury_division", "completed": 3, "total": 7 },
  "correlation_id": "01HYQ3M8…",
  "result": null,
  "error": null }
```

States: `pending → running → succeeded | failed | cancelled`, terminal states immutable and retained 30 days. `result` is an inline document or a `Location`. `error` is a full problem document using the same codes as the synchronous path — one error vocabulary, not two.

Completion reaches clients three ways, in preference order: a Signal on the `Self` scope, a webhook (§11), or polling `GET /v1/operations/{id}` honouring `Retry-After`. Safely abandonable operations expose `POST /v1/operations/{id}:cancel`; Fracture does not, because `11 §3.2` makes it resumable-forward only once the parent's log is sealed. An operation that cannot be cancelled says so in `cancellable: false` rather than failing the cancel call.
Completion reaches clients three ways, in preference order: a Signal on the `Self` scope, a webhook (§11), or polling `GET /v1/operations/{id}` honouring `Retry-After`. Abandonable operations expose `POST /v1/operations/{id}:cancel`; Fracture does not, because `11 §3.2` makes it resumable-forward only once the parent's log is sealed. An uncancellable operation says so in `cancellable: false` rather than failing the cancel call.
**Operations used:** Fracture, Crystallization, Society export and import, Extension install, Workflow invocation, bulk moderation, media transcode, settlement report generation.

### 10.3 Streaming

Three streaming shapes, each for a distinct need:

| Need | Mechanism | Why not the others |
|---|---|---|
| Live state at a connected front end | Signal WebSocket | Bidirectional, multiplexed, resumable |
| A headless consumer tailing one Society's events | SSE: `GET /v1/societies/{sid}/events:stream` (`Accept: text/event-stream`) | One-way, plain HTTP, survives proxies, trivially consumed by a shell or an agent framework |
| Bulk history or high-volume agent traffic | gRPC server-streaming | Flow control and typed frames |

SSE carries the event `id` as the per-Society `seq`, so `Last-Event-ID` is the same mechanism as `since(seq)` — one cursor concept across three transports. It exists because a caller who wants `curl -N` to print events should not need a WebSocket client, and because most agent frameworks consume SSE natively.

### 10.4 Export and import

P2 makes this a sovereignty requirement, not a feature: *"Take your Society and leave" is an export, not a negotiation* (`10 §4`). P9 makes the personal equivalent mandatory: personal data is exportable and deletable by its owner.

```
POST /v1/societies/{sid}/exports        → Operation
  { "scope": ["events","charter","memberships","vault_manifests",
              "treasury_statement","facets","anchors"],
    "vault_bytes": "manifest" | "inline",
    "format": "fnbundle/1" }
```

The bundle is a signed, content-addressed archive: the complete event log with its hash chain, every Charter version, memberships with Standing and tenure, Vault Manifests (Shard bytes by reference by default — exporting a 10 TB Society must not move 10 TB unless asked), the Treasury statement with every Posting, Facet records with provenance, and the Anchor chain with its proofs. It verifies offline — `fn verify export.fnbundle` recomputes the hash chain against the Anchors without contacting the Runtime. An export whose integrity only we can attest to is not sovereignty.

`POST /v1/citizens/me/exports` is the P9 personal export: authored messages, Memberships, Standing, wallet history, declared interests, Facets, and the complete telemetry record about the Citizen (`02 §4` bans silent telemetry, which means it must be inspectable).

`POST /v1/imports` reconstitutes a bundle into a self-hosted Node (Phase 6). It is an Operation, verifies signatures and hash chains before writing anything, and fails closed on the first inconsistency with the offending `seq` named. Import is what makes the export honest: a format nothing can read is a backup nobody has tested.

---

## 11. Webhooks

Signals are for connected front ends. Webhooks are for systems that are not connected and cannot hold a socket.

**Subscription.** `POST /v1/societies/{sid}/webhooks` with a target URI, an event-kind filter, and a description. Subscriptions are Society-scoped (P1) and bounded by the subscribing Envelope — a webhook cannot deliver an event its Envelope could not read. Filters are exact kinds or one trailing wildcard within a boundary (`ledger.*`), never `*`.

**Delivery guarantee: at-least-once, ordered per Society, best-effort across Societies.** Exactly-once is not promised because nobody can deliver it; every delivery carries `event_id` and consumers deduplicate on it. Ordering follows `seq`, preserved by a single in-flight delivery per subscription, so a slow consumer delays only its own stream.

**Signing.** Ed25519 detached signature over `timestamp + "." + raw_body`:

```http
POST /hooks/fractal HTTP/1.1
Fn-Signature: t=1788345842,v1=MEUCIQD…,kid=whk_01HY…
Fn-Event-Id: 01HYQ3ND…
Fn-Event-Kind: ledger.transfer.settled.v1
Fn-Society: soc_01H8Z…
Fn-Delivery-Attempt: 1
Content-Type: application/json
```

The public key is at `GET /v1/societies/{sid}/webhooks/{id}/keys`, two keys active during rotation so rotation never drops a delivery. **Replay protection** is what the signature alone does not give: reject a timestamp outside a 300-second window, and deduplicate on `Fn-Event-Id`. SDK verifiers check both and offer no way to skip either.

**Retry.** Exponential backoff with full jitter — 10 s, 30 s, 2 m, 10 m, 1 h, 6 h, 24 h — then dead-letter. A subscription failing for 7 days is disabled and a Signal notifies the Society's governance role. Dead-lettered deliveries are inspectable at `GET …/webhooks/{id}/deliveries` and individually replayable. `2xx` is success; a `410 Gone` from the consumer disables the subscription immediately, because that is the consumer saying it is finished.

**Honest cost.** Webhooks add a delivery queue, a retry scheduler, a signing-key lifecycle, and a dead-letter surface to the operational footprint, and they are a request-forgery vector requiring defence (no private address ranges, DNS re-resolution per attempt, no redirects followed). We accept it because the alternative — telling headless integrators to poll — converts our rate limits into their architecture.

---

## 12. Machine-Readable Everything

### 12.1 The single source of truth

**Decision.** The contract is a set of **protobuf files under `contract/`**, managed with Buf, annotated with `google.api.http` for the HTTP mapping and a small closed set of Fractal Node options. It is the only place an API surface is defined. Everything else is generated.

```protobuf
// contract/fractal/v1/discourse.proto
service DiscourseService {
  rpc PostChamberMessage(PostChamberMessageRequest) returns (Message) {
    option (google.api.http) = {
      post: "/v1/societies/{society_id}/chambers/{chamber_id}/threads/{thread_id}/messages"
      body: "*"
    };
    option (fn.capability)      = "chamber.message.post<in:{chamber_id}>";
    option (fn.idempotency)     = REQUIRED;
    option (fn.dry_run)         = SUPPORTED;
    option (fn.emits)           = "discourse.message.posted.v1";
    option (fn.signal_scope)    = THREAD;
    option (fn.cli)             = "message post";     // → fn message post
    option (fn.error_set)       = OPEN;
    option (fn.phase)           = 0;
  }
}
```

**Why protobuf as the source rather than OpenAPI, Rust types, or a bespoke IDL.**

*Against OpenAPI-as-source.* OpenAPI is the better *output* and a poor source: it cannot express gRPC services, its breaking-change tooling is materially weaker than `buf breaking`, and hand-maintained specifications drift from the implementation with near-physical reliability. We generate it, so it cannot drift.

*Against Rust-types-as-source.* Tempting, and rejected on two grounds. It couples the public contract to one language's type system when the front ends are peers, not clients of a Rust program (P13). And it makes the contract *movable by a refactor* — a rename in a domain crate becomes a silent breaking change, the exact failure §3 exists to prevent. The contract must be a thing a human deliberately edits, in a directory whose changes get a different review.

*Against a bespoke IDL.* We would own a parser, a type system, a breaking-change classifier, and six code generators. `00 §3` requires an exit cost, and the exit cost of a bespoke IDL is a rewrite of the entire toolchain.

**Honest costs, stated.** Protobuf's JSON mapping is not ours — `oneof` naming, int64-as-string, unset-versus-default, `google.protobuf.Timestamp` — so §12.3 pins a profile. Protobuf has no native ETag, conditional request, or header concept, so those live in custom options and gateway behaviour. And its enums are open by default, which is exactly wrong for `PostingReason` — hence the mandatory `fn.enum_kind` annotation (§3.3).

### 12.2 What is generated

```
                          contract/*.proto
                                 │
   ┌──────────┬──────────┬───────┴────┬───────────┬──────────┬────────────┐
   ▼          ▼          ▼            ▼           ▼          ▼            ▼
OpenAPI    gRPC      JSON Schema   SDK cores   CLI cmd    capability   error-code
  3.1     services    2020-12      (6 langs)   surface     registry     registry
   │          │           │            │          │           │            │
   │          │           │            │          │           └─► checked against
   │          │           │            │          │               12 §7.1 grammar;
   │          │           │            │          │               an unparseable
   │          │           │            │          │               capability fails
   │          │           │            │          └─► fn <noun> <verb> tree +
   │          │           │            │              shell completions + help
   │          │           │            └─► transport, models, pagination iterators,
   │          │           │                retry policy, error types
   │          │           └─► every event kind's payload schema; the Signal decoder;
   │          │               webhook payload validation
   │          └─► internal boundary stubs + public high-throughput services
   └─► the reference documentation site; the Postman/Bruno collection;
       the mock Runtime used by SDK tests and by the docs' runnable examples
```

Nine artifacts, one edit. The generator is a CI job, its outputs are committed (so that a diff is reviewable and a build is reproducible), and a hand-edit to a generated file fails the build.

### 12.3 The JSON mapping profile

Pinned in the contract's README and tested by one golden corpus shared across both encodings:

| Concern | Rule |
|---|---|
| 64-bit integers | Decimal **string** in JSON, always, including `seq` and every `quanta` amount |
| Timestamps | RFC 3339 UTC with explicit precision (`10 §10`); never epoch numbers |
| Field names | `snake_case`, identical to the proto field name |
| Unset vs default | Explicit `optional`; absent means absent, and `0` never means "unset" |
| `oneof` | Rendered as a tagged object `{ "kind": "text", "text": {…} }`, never as sibling nullable fields |
| Enums | `SCREAMING_SNAKE_CASE` strings, never ordinals; open enums documented with their unknown-member rule |
| Bytes | base64url without padding |
| Unknown fields in a request | Rejected with `validation_failed` — a silently ignored typo is a defect generator |
| Unknown fields in a response | Consumers must ignore; every SDK enforces this |

### 12.4 How this makes P13 mechanical

P13's falsification test — *any feature in the GUI but absent from the CLI, or the reverse, blocks the release tag* — is a promise as a checklist and a fact as a build gate. Four parts, all reading the same contract:

```
 ┌─ GATE 1 ─ contract completeness ──────────────────────────────────────────┐
 │ Every RPC declares fn.capability, fn.idempotency, fn.cli, fn.phase.       │
 │ A missing annotation fails the lint. No RPC ships undeclared.             │
 ├─ GATE 2 ─ CLI surface parity ─────────────────────────────────────────────┤
 │ The CLI's command tree is GENERATED from fn.cli. It cannot lack a         │
 │ command, because the command is not hand-written. An RPC without an       │
 │ fn.cli value fails GATE 1.                                                │
 ├─ GATE 3 ─ GUI provenance ─────────────────────────────────────────────────┤
 │ Every network call in every client is issued through the generated SDK.   │
 │ A lint denies raw fetch/reqwest/URLSession against the API origin. The    │
 │ set of operations a GUI can invoke is therefore a subset of the contract, │
 │ by construction — which is P3's grep test, automated.                     │
 ├─ GATE 4 ─ parity suite ───────────────────────────────────────────────────┤
 │ For each operation, a test executes it twice against an ephemeral         │
 │ Runtime — once via the CLI, once via the TypeScript SDK — and asserts the │
 │ resulting event streams are identical modulo ids and timestamps.          │
 │ Divergence fails the release tag.                                         │
 └───────────────────────────────────────────────────────────────────────────┘
```

Gate 4 is the expensive one and the one that catches things: identical event streams is a far stronger claim than identical HTTP responses, and it is the claim P13 actually makes. Its cost is a slow suite — roughly one ephemeral Runtime per operation family — so it runs on release candidates and nightly, not per commit.

The composite effect is that a capability which is not in the contract does not exist (P3), and a capability in the contract is in the CLI whether anyone remembered or not (P13). Neither principle depends on discipline.

---

## 13. The SDKs

### 13.1 The two-layer rule

Every SDK is exactly two layers, and the boundary between them is load-bearing.

```
 ┌──────────────────────────────────────────────────────────────────┐
 │ ERGONOMIC LAYER — hand-written, ~15% of the code, ~all of the UX │
 │ auth + DPoP · token refresh · retry + jitter · idempotency        │
 │ defaults · pagination iterators · realtime client · problem →     │
 │ typed error mapping · dry-run helpers · offline replica (where    │
 │ offered) · logging hooks                                          │
 ├──────────────────────────────────────────────────────────────────┤
 │ GENERATED LAYER — 100% machine-produced, never hand-edited       │
 │ models · request/response types · route table · error codes ·     │
 │ capability strings · event payload types · Signal frame decoder   │
 └──────────────────────────────────────────────────────────────────┘
```

Generated code carries no cleverness; hand-written code carries no schema knowledge. A contract change regenerates the lower layer and cannot silently alter the upper one. This prevents the SDK that accumulates hand-patched models and drifts from the API it wraps — P3's second-implementation problem one layer up.

### 13.2 Cross-cutting behaviour every SDK must implement identically

| Behaviour | Contract |
|---|---|
| Auth | Device key held in the platform keystore; DPoP proof per call; automatic single-use refresh; refresh-reuse → hard session kill and a raised error, never a silent retry |
| Retry | Only when `retryable: true`. Full jitter. Honours `Retry-After` exactly, then jitters on top. Default 3 attempts, capped at 30 s total |
| Idempotency | A key is generated automatically for every unsafe request and **reused across retries of the same logical call**. An SDK that generates a fresh key per attempt has defeated the mechanism |
| Pagination | A lazy iterator/stream; the raw cursor is reachable but never required |
| Errors | Problem documents map to typed errors carrying `code`, `cause`, `remedy`, `retryable`, `request_id`. `remedy.cli` is printed in developer builds |
| Unknown fields | Ignored on responses; open enums surface an `Unknown(String)` variant; closed enums that fail to parse are an error, not a default |
| Deprecation | `Fn-Warning` logged once per route per process |
| Clock | Configurable, so tests are deterministic (`10 §7`) |

### 13.3 The six

| SDK | Phase | Generated layer | Ergonomics | Offline (P2) | Realtime | Notes |
|---|---|---|---|---|---|---|
| **TypeScript** | 1 | Zod-validated models, fetch transport | Browser + Node; WebCrypto DPoP; React hooks in a separate package so the core stays framework-free | Full: IndexedDB replica + outbox, shared with the web GUI | WebSocket + SSE | The reference SDK. The web GUI is its first consumer, which is what keeps it honest |
| **Rust** | 1 | Same contract crate the Runtime uses for wire types | `async` on Tokio; typed capability errors; used by the CLI and by Tauri | Full: SQLite replica + outbox — this is the P2 implementation the other clients embed | WebSocket + gRPC | The CLI being an SDK consumer is N3 made structural |
| **Python** | 2 | Pydantic models | Sync and async clients; first-class `dry_run`; pandas-free by design | **None.** Online-only, documented as such | SSE + WebSocket | The agent-author's SDK. Optimized for `15`'s Workflow authors |
| **Go** | 3 | Generated structs, `net/http` transport | Context-first; no reflection; single dependency | None | SSE + gRPC | Operational integrations, custodian tooling, webhook consumers |
| **Swift** | 5 | Codable models | Structured concurrency; Keychain + Secure Enclave device key | Full, **via the Rust core over UniFFI** | WebSocket | Ships with native iOS (`34`, Phase 5) |
| **Kotlin** | 5 | kotlinx.serialization models | Coroutines + Flow; Keystore-backed device key | Full, **via the Rust core over JNI** | WebSocket | Ships with native Android |

**The mobile decision is the one worth defending.** Swift and Kotlin do not reimplement the replica, the outbox, `10 §6`'s conflict policy, or the MLS client; they bind the Rust core. Three independent event-log replicas with per-data-class conflict resolution would produce three subtly different notions of what a Society's state is, with bugs unreproducible across platforms. The cost — an FFI boundary, larger binaries, a harder debugging story, a build needing the Rust toolchain — is smaller than the cost of divergence. N2 exists partly so this option is available when Phase 5 arrives.

**Python and Go are online-only, and we say so in the first paragraph of their READMEs.** Pretending otherwise would be worse than the limitation. Their callers are integrations and agents that run beside the Runtime, not on a Citizen's laptop in a tunnel.

### 13.4 Release and versioning

- SDK **major** tracks the API major. `@fractalnode/sdk@2.x` speaks `/v2/` and only `/v2/`.
- SDK **minor** is independent and additive: a regenerated layer for new endpoints is a minor; an ergonomic addition is a minor.
- SDK **patch** is bug fixes only, never a regeneration that adds surface.
- Every SDK release names the contract commit it was generated from; `fn version --contract` prints it. A mismatch against the Runtime's advertised contract produces a startup warning, not a failure — an older client must keep working, which is what additive-only means.
- Release cadence follows the contract, not a calendar: a merged contract change publishes all six SDKs in one automated release. Six SDKs on six schedules is how parity dies.

---

## 14. Designing for an Agent Caller

An agent is a caller that cannot ask a colleague, cannot read a guide written after its training cutoff, and cannot safely learn by trying a write. Five properties follow, and all five are cheap when designed in from the start.

### 14.1 Discoverability

`GET /v1/.well-known/fractal-node` returns the entry document: current majors, the contract commit, and links to the OpenAPI document, the JSON Schema index, the error-code and capability registries, the rate-limit policy, and the Signal endpoint. `GET /v1/schema` returns the artifacts themselves. An agent starting from the base URI alone reaches a complete, current description of the surface.

### 14.2 Self-description of the caller's own authority

```http
GET /v1/capabilities?society=soc_01H8Z…
```

```json
{ "principal": { "kind": "agent", "fnid": "fn1ag…", "operator": "fn1qz…" },
  "envelope": { "id": "env_01HY…", "expires_at": "2026-10-01T00:00:00Z" },
  "capabilities": [
    { "capability": "chamber.message.read<in:*>",              "operations": ["GetMessages","ListThreads"] },
    { "capability": "moderation.flag<in:*,<=200/day>",         "operations": ["FlagMessage"],
      "consumed": { "window": "24h", "used": 37, "limit": 200 } } ],
  "denied_operations": [
    { "operation": "PostChamberMessage", "missing": "chamber.message.post<in:*>",
      "request_uri": "/v1/societies/soc_01H8Z…/envelopes/env_01HY…/requests" } ],
  "limits": { "tier": "agent", "sustained_rpm": 75, "burst_rps": 5 },
  "confirm_classes": ["wallet.transfer", "external.publish"] }
```

An agent that reads this plans; one that does not, probes. Probing means attempting writes to discover authority, which produces denial events counting against Trust (`15 §6`) — so the API that answers honestly is also the one that does not punish an agent for our design flaw. `denied_operations` is bounded to operations the Envelope *nearly* reaches, not an enumeration of the platform.

### 14.3 Errors that carry the remedy

§6.1's `cause` and `remedy` exist for this caller. The missing capability is a string in the exact grammar of `12 §7.1` — not prose, not an internal identifier — so an agent can put it verbatim into a capability request its Operator reviews. The loop closes: *denied → request the precise capability → human approves the computed intersection → retry*, all in the vocabulary the audit trail will use.

### 14.4 Deterministic pagination and stable ordering

Two guarantees, both contractual:

1. Paging a collection twice from the same cursor returns the same items in the same order, unless the underlying resources changed. The cursor pins the snapshot `seq`, so concurrent writes do not perturb an in-progress traversal.
2. Every collection's default ordering is part of the contract; changing it is breaking (§3.2).

This matters more for agents than for humans because an agent frequently re-reads a collection to detect change. Non-deterministic ordering turns "what changed?" into a set comparison over the whole collection, and at that point the agent is spending a rate-limit budget on our design defect.

### 14.5 `dry_run`, on everything consequential

Every operation annotated `fn.dry_run = SUPPORTED` accepts `?dry_run=true`. The rule:

> A dry run runs the **full** PEP evaluation and the **full** domain decision, computes the effects, then discards them. It emits no Domain Event, moves no Fraction, and mutates nothing. It returns `200` with `Fn-Dry-Run: true`, the body the real call would produce, plus an `effects` block.

```http
POST /v1/societies/soc_01H8Z…:fracture?dry_run=true
```

```json
{ "dry_run": true,
  "would_succeed": true,
  "effects": {
    "events":   [ { "kind": "society.fractured.v1", "count": 1 },
                  { "kind": "society.created.v1",   "count": 2 } ],
    "postings": [ { "debit": "wal_treasury_S", "credit": "wal_treasury_A", "amount": "412000000000" },
                  { "debit": "wal_treasury_S", "credit": "wal_treasury_B", "amount": "188000000000" } ],
    "memberships": { "to_child_a": 84, "to_child_b": 39, "unassigned": 0 },
    "vault":       { "manifests_rereferenced": 2841, "bytes_moved": "0" },
    "invariant_checks": [ { "check": "sum_debits_eq_sum_credits", "result": "pass" },
                          { "check": "no_citizen_loses_history",   "result": "pass" },
                          { "check": "total_facets_preserved",     "result": "pass" } ] },
  "capabilities_used": ["society.fracture"],
  "confirm_required": ["society.fracture"],
  "dry_run_token": "dry_01HYQ3P2…" }
```

Three properties make this useful rather than decorative. **The PEP runs**, so one call answers "would I be allowed?" and "what would happen?" with no side effect. **`invariant_checks` are the real property tests** from `11 §7`, evaluated against the simulated result; a violation returns `would_succeed: false` naming the failing check. And **`dry_run_token`** satisfies §4.3's `dry_run_required` — Fracture will not execute without a token from a dry run in the same correlation, which is `11 §3.2` enforced at the wire rather than trusted to a client.

Dry run is mandatory on Fracture, Dissolution, Charter enactment, bulk moderation, and any Transfer above a Society's confirm threshold. It is available on everything else that writes.

**Honest cost.** Every dry-run-supporting handler needs its effect computation separable from its commit — a real constraint, and one `10 §1`'s pure-domain rule already imposes. Where a handler cannot be made separable, `fn.dry_run = UNSUPPORTED` is declared with a documented reason; silence is not an option the linter permits.

### 14.6 Structured output, everywhere

No machine field carries prose. `detail` is for humans; `cause` and `remedy` are for machines. Enumerations are enumerations, not strings that happen to have a few values. Quantities carry their unit in the field name (`amount_quanta`, `limit_bytes`, `window_seconds`). An agent should never need a regular expression to consume a response; if it does, that is a contract defect to file.

---

## 15. Documentation Standards

`00 §5` makes documentation part of the definition of done: an API reference entry, a CLI help entry, and a changelog line. This chapter adds the mechanism and one rule.

> **Invariant I-30.2 — an endpoint without documentation is not shipped.** The publish pipeline refuses a contract change whose new or altered operations lack a description, at least one runnable example, and an entry in the changelog. This is enforced at the generator, not at review.

**Reference is generated; guides are written.** The reference — every operation, field, enum member, error code, capability, and tier — comes from the contract, so it cannot drift. Guides, concepts, and migration notes are hand-written, because a generator cannot explain why Fracture requires a dry run.

**Every operation carries three examples, and all three are executed in CI:**

```
  curl                     the universal one; copy-pasteable; no client required
  one SDK                  TypeScript by default; the SDK a caller most likely holds
  the CLI                  fn <noun> <verb> — proof the P13 binding exists and works
```

Examples run against the generated mock Runtime on every docs build. An example that no longer executes fails the build. Documentation that has never been run is a wish, and the industry's default state is a corpus of wishes.

**House rules for reference prose** (`33 §7`): terse, declarative, no apology, second person as operator, every error stating cause and remedy, no exclamation marks, the vocabulary of `01` and nothing else. A page that says "channel" instead of "Chamber" is a defect with a lint rule attached.

---

## 16. API Governance

### 16.1 Adding a resource family

A new family is a permanent versioning commitment and consumes one of the three per phase (`02 §5`). The proposal must answer, in this order:

1. **Which principle does it serve?** (`02 §6` question 1.)
2. **Which phase places it?** If it is not the current phase, the output is a proposal document, not a contract change.
3. **Which boundary from `10 §3` owns it?** A family with no owning boundary is either misplaced or a new boundary, and a new boundary is a separate, larger decision.
4. **Which capability domain governs it?** (`15 §4.1`'s closed domain list.) A family needing a new domain needs an ADR.
5. **What is the CLI surface?** Named up front, because it is generated and must read well as `fn <noun> <verb>`.
6. **What Signals does it emit, or why none?**
7. **What does it cost forever?** Support, versioning, security surface, and the deprecation obligation.
8. **What is being cut to stay inside the budget?** `02 §8`: nothing enters a phase without something leaving it.

Adding an *endpoint* to an existing family is ordinary work under the additive-only rule and needs no ADR — only the §15 documentation gate and the §3.5 compatibility test.

### 16.2 Review

| Change | Review |
|---|---|
| New endpoint in an existing family | One reviewer; automated gates |
| New field, open enum member, or error code | One reviewer; automated gates |
| New resource family | ADR + named approver; consumes budget |
| Any change classified BREAKING | ADR + named approver + major bump; no exceptions |
| New capability string | ADR — it changes the authorization lattice (`12 §7`) |
| Deprecation | §16.3 |

### 16.3 The deprecation committee of one

Deprecation is where APIs quietly betray their consumers, so the process is narrow and personal. **A single named approver signs every deprecation** — currently the project's owner. Not a committee: a committee diffuses the accountability that makes a person hesitate before breaking someone's integration.

The submission must contain the successor, live and documented; a migration guide with runnable examples in all three forms; usage telemetry naming who calls the deprecated surface and how much; a notice period no shorter than §3.4's minimum; and the `Deprecation`/`Sunset`/`Link` headers already live on the route.

Two rules the approver applies without discretion. **A deprecation whose successor does not exist is rejected** — "we will build the replacement during the notice period" is how a notice period becomes a shutdown. And **the clock starts when the headers ship, not when the decision is made**, because a deprecation nobody was told about has not been announced.

---

## 17. Trade-offs, Failure Modes, and Rejected Alternatives

### 17.1 What we accepted

| Choice | Cost accepted | Why |
|---|---|---|
| Three protocols | Three conformance suites, two JSON mappings | Each carries traffic the others carry badly; one generated contract keeps them from diverging |
| Protobuf as source of truth | A mapping profile, custom options, no native ETag concept | `buf breaking` is the strongest compatibility gate available, and it makes §3.5 real |
| Cursor-only pagination | No page numbers, no exact totals | Correctness under concurrent insertion; determinism for agents; index-seek cost |
| Closed enums for economic types | Adding a Source or Sink can require a major | `11 §7.15` and `17` require exhaustive categorization; a silent "other" bucket destroys the audit property |
| Conflated `404` | Harder permission diagnosis | An existence oracle is an enumeration attack on private Societies (P9) |
| Non-atomic batch | No generic multi-write transaction | `11 §5` answers cross-aggregate consistency with sagas; a generic transaction API would be a distributed transaction in disguise |
| Mandatory `Idempotency-Key` on three endpoints | A slightly ruder API | Duplicate money, duplicate purchase, duplicate message. Client discipline is not a control |
| No socket writes | One round trip on send | One authorization path (P8); `14 §3` shows the perceived cost is zero |
| Six SDKs | Six release surfaces | P13 is a release gate; an unsupported language is a second-class front end |
| Mobile SDKs bind the Rust core | FFI complexity, larger binaries | Three replica implementations would be three definitions of Society state |

### 17.2 Rejected alternatives

| Alternative | Fair case for it | Why rejected |
|---|---|---|
| **GraphQL** | Genuinely graph-shaped domain; one round trip per screen; introspection | Unbounded edge cost against per-principal buckets; capability denial does not compose into partial errors; loses `Idempotency-Key`/`ETag`/`Retry-After`; makes the P13 parity gate unenumerable. `expand` and `fields` recover most of the benefit (§2.5) |
| **JSON:API** | A real specification with an ecosystem; solves envelopes, sparse fieldsets, and relationships | Its document envelope and relationship-object indirection cost every consumer a mental model we would have to teach, to standardize things we needed three conventions for. Adopted its `fields` idea; declined the rest |
| **gRPC as the primary public surface** | Faster, typed, streaming, one encoding | Excludes browsers without a proxy, excludes casual inspection, excludes an agent writing a call from a description. P3's audit test depends on a readable wire |
| **OpenAPI as the source of truth** | The obvious choice; everyone reads it | Cannot express gRPC; weaker breaking-change tooling; hand-maintained specs drift. Generated instead, so it cannot |
| **Rust types as the source of truth** | Zero mapping layer; one language | Contract becomes movable by refactor; couples the peer-front-end contract to one runtime's type system |
| **Header-negotiated versioning** | RESTfully pure; no URL churn | Version invisible in logs, traces, proxy rules, and incident timelines |
| **Bearer tokens with long lifetimes** | Simpler clients | `12 §5.2` refuses bearer tokens outright; a stolen token must be inert |
| **Offset pagination alongside cursors** | Familiar; enables page numbers | Two paginations means two correctness stories, and the wrong one gets used for the replica reconciliation path |
| **A generic query/filter DSL** | Fewer endpoints; flexible callers | An unbounded query planner exposed to Agents; no way to guarantee an index exists |
| **Webhooks only, no Signal socket** | One delivery mechanism to operate | Interactive latency (`14 §3`) is unreachable through webhooks |
| **Signal socket accepts writes** | One connection, fewer round trips | Two authorization implementations, one of which will be wrong (I-30.1) |

### 17.3 Failure modes we expect

| Failure | Symptom | Response |
|---|---|---|
| Contract becomes a bottleneck | Every change waits on a contract review | Split the contract by boundary with per-boundary ownership; keep one linter and one classifier |
| Generated SDK ergonomics are poor in one language | Callers hand-roll HTTP instead | The ergonomic layer is the release gate for that SDK; a hand-rolled client in our own examples is the alarm |
| Golden corpus rots | Compatibility test passes while real clients break | Corpus regenerated per release from scrubbed staging traffic; a stale corpus fails its own freshness check |
| `dry_run` diverges from execution | A dry run says pass and the real call fails | Both paths share one handler with a commit flag; a property test asserts the dry run's `would_succeed` matches the real outcome on a generated corpus |
| Rate limits push callers to parallel credentials | Many keys per principal to multiply budget | Buckets are per-**principal**, not per-credential. Keys attenuate a principal's budget; they never add to it |
| Enum widening lands anyway | A closed enum gains a member in a patch | `fn.enum_kind` is mandatory; `buf breaking` plus the classifier fails the build. If it still lands, it is an incident with an ADR, not a hotfix |
| Webhook consumers become an outage amplifier | Slow consumers back up delivery | One in-flight delivery per subscription; per-subscription queues; auto-disable at 7 days |
| The parity suite becomes too slow to run | Gate 4 gets disabled "temporarily" | It runs on release candidates and nightly, never per commit — designed for the pressure it will be under |

---

## 18. Invariants This Chapter Adds

Each becomes a test that runs on every change.

| # | Invariant |
|---|---|
| I-30.1 | The Signal socket never writes domain state. Every state change enters via HTTP or gRPC with an `Idempotency-Key`. |
| I-30.2 | An operation without a description, a runnable example, and a changelog entry does not publish. |
| I-30.3 | Every RPC in the contract declares `fn.capability`, `fn.idempotency`, `fn.cli`, `fn.phase`, `fn.error_set`; every enum declares `fn.enum_kind`. Missing annotations fail the lint. |
| I-30.4 | Every capability string in the contract parses under the `12 §7.1` grammar and exists in the generated registry. |
| I-30.5 | Every path to a Society-owned resource contains a `society_id` (P1). |
| I-30.6 | No endpoint name, CLI command, or event kind uses a forbidden verb from `01 §8`. |
| I-30.7 | Every error response is `application/problem+json` with a `code` from the registry and an explicit `retryable`. |
| I-30.8 | No error string matches the banned-apology lint (`33 §7.3`). |
| I-30.9 | Every 403 authorization denial names the missing capability in `cause.required_capability` and in `Fn-Capability-Required`. |
| I-30.10 | Every collection paginates by cursor; no endpoint accepts an `offset` parameter. |
| I-30.11 | Every 64-bit integer is encoded as a decimal string in JSON, in both encodings. |
| I-30.12 | Every operation with `fn.dry_run = SUPPORTED` emits zero Domain Events when invoked with `dry_run=true`. |
| I-30.13 | Every generated SDK reuses one `Idempotency-Key` across all retries of one logical call. |
| I-30.14 | No credential issued by this API grants a CapabilitySet its issuing principal does not hold (I-12.6, I-12.7). |
| I-30.15 | Reads and exports are never refused for `quota_exceeded`. |
| I-30.16 | Every operation in the contract has a CLI binding, and the parity suite proves both produce identical event streams (P13). |

---

## 19. What Would Make Us Change This

Stated in advance so the signal is recognised rather than rationalized.

- **Mobile round trips exceed four per screen with `expand` in use.** → Ship named Runtime-defined composite reads. Reconsider GraphQL only if those exceed roughly thirty distinct compositions, at which point we are maintaining a query language badly.
- **The protobuf JSON mapping profile accumulates more than a handful of exceptions.** → The mapping is fighting us. Move the source of truth to a small bespoke schema that emits protobuf as one target, accepting the toolchain cost we rejected in §12.1.
- **`buf breaking` proves insufficient for semantic changes at volume.** → Promote the golden corpus from a gate to the primary classifier and invest in response-diff classification.
- **Third-party gRPC adoption stays near zero after two phases.** → Keep gRPC internal only, and stop generating public gRPC documentation. A public surface nobody uses is a versioning commitment we are paying for twice.
- **Agents dominate traffic and HTTP/JSON overhead becomes a measured cost centre.** → Offer CBOR on the HTTP surface via content negotiation, with the same schemas. Not a new API; a second encoding of the existing one.
- **The parity suite's runtime exceeds the release window.** → Shard it by resource family and run families in parallel. Disabling it is not on the list.
- **Two API majors prove operationally unsustainable.** → Extend the v1 support window and slow major cadence further, rather than shortening it. The support window exists to be honoured; a window we break is worse than one we never offered.
