# 14 — Realtime and Social

> **Prerequisites:** the Canon (`00`, `01`, `02`), `10-system-architecture.md`, `11-domain-model.md`, `12-identity-and-trust.md`.
> **Governs:** the Signal transport and the Relay boundary; the realtime latency budget; end-to-end encryption of Chambers and private conversation (MLS, RFC 9420); the exact line between encrypted and Runtime-readable content; moderation under encryption; voice, video, and the Stage (Phase 4); discovery without surveillance (Phase 5); the Convergence → Crystallization → Society → Fracture funnel; anti-abuse on social surfaces; the notification model.

---

## 0. Two sentences this chapter must make true

> **A Message written in a Chamber reaches a peer fast enough that the conversation feels like one shared space — and if that Chamber is private, no part of Fractal Node's infrastructure can read it. Including us. Including under compulsion.**

> **A Citizen finds people worth talking to because of what they *declared*, never because of what was silently measured about them.**

The first is P8/P10. The second is P9. Same chapter, because they are the same product: a social layer whose speed is not paid for with surveillance, and whose privacy is not paid for with unusability.

---

## 1. Why the Relay is a distinct boundary

`10 §3` names the Relay **S14** and places it second in the extraction order. It owns no domain state: it is a subscription multiplexer and fan-out engine that consumes the bus, matches events against live subscriptions, and writes frames to sockets.

| Property | Runtime | Relay |
|---|---|---|
| Scaling axis | commands/sec (CPU, DB) | connections × fan-out (memory, egress) |
| Working set | aggregates, projections | socket buffers, subscription index, replay ring |
| Failure impact | platform down | realtime degrades to polling |
| Deploy cadence | weekly | daily (framing and buffer tuning) |
| State durability | durable | wholly reconstructible |

That is criteria 1 and 2 of the `10 §2` extraction rule, met measurably. It ships inside the modular monolith behind the `Relay` port and is extracted when connection memory or egress contends with command throughput.

**Relay invariants — structural, not conventional.**

- **R1.** No command path exists from a socket frame to the Log. Realtime input traverses the same gateway and application layer as everything else (P3). A socket is an output device with a control path.
- **R2.** Every durable Signal carries `(society_id, seq)` from an already-persisted Domain Event. A Signal notifies that an event exists; it is never the event's authority. Delivery is at-least-once; clients dedupe on `(society_id, seq)`.
- **R3.** The Relay holds no plaintext it could not otherwise read. In an `EndToEnd` Chamber it moves opaque ciphertext (N6).
- **R4.** A subscription confers no authority. It is re-evaluated against the subscriber's Envelope and Membership on every capability-affecting event; revocation severs live subscriptions within one bus hop, not at next reconnect.
- **R5.** A durable Signal is never silently dropped. The Relay coalesces, sheds declared-ephemeral classes, or emits an explicit `Gap` (`10 §10`).

---

## 2. The Signal protocol

**Transport.** WebSocket over TLS 1.3, binary frames, CBOR payloads behind a fixed 8-byte header, every frame kind registered in the same schema registry as Domain Events. JSON was rejected: ~2.5× the serialization cost and egress at 30k connections, for a debugging convenience the CLI provides with a `--trace` flag.

```rust
enum SignalFrame {
    // client → relay
    Hello       { proto: u16, device: DeviceId, caps: ClientCaps },
    Subscribe   { scopes: Vec<Scope>, since: Vec<Cursor> },
    Resume      { session: SessionId, cursors: Vec<Cursor> },
    Ack         { society: SocietyId, seq: u64 },     // flow control, not read state
    Presence    { state: PresenceState, scope: Scope },
    Typing      { thread: ThreadId, ttl_ms: u16 },
    Read        { thread: ThreadId, upto: Ulid },
    // relay → client
    Welcome     { session: SessionId, replay_window: u32, limits: ConnLimits },
    Signal      { society: SocietyId, seq: u64, kind: &'static str, body: Bytes },
    Gap         { society: SocietyId, from_seq: u64, to_seq: u64, reason: GapReason },
    PresenceSet { scope: Scope, deltas: Vec<PresenceDelta> },
    Shed        { class: SignalClass, retry_after_ms: u32 },
    Bye         { reason: CloseReason },  // Revoked | Superseded | Draining | Protocol
}

enum Scope { Society(SocietyId), Chamber(SocietyId, ChamberId),
             Thread(SocietyId, ThreadId), Convergence(ConvergenceId), Self_ }
```

`Bye{Draining}` makes Relay deploys invisible: the instance stops accepting, hands each client a jittered delay, and clients resume elsewhere with cursors intact.

**Subscription rules.** Every scope resolves to exactly one Society or to the Citizen's own inbox (P1). Subscribing to `Society` yields Signals only for Chambers the subscriber may *currently* read — the filter runs at fan-out against the live access decision, not against a subscribe-time snapshot (R4). **No wildcard above a Society exists for any principal, including first-party operations tooling**; operational visibility is metrics and traces, never a plaintext tap. An Agent subscribes under its Envelope. Cap: 200 scopes per connection — a Citizen in 400 Societies subscribes to `Self` plus what is on screen and pulls the rest from the local replica (P2).

**Backpressure.** Each connection has a bounded outbound queue (256 frames or 1 MiB). On fill, in this order, never reordered: **coalesce** latest-per-subject (presence, typing, read marks) → **shed with notice** (`Shed{class}`) → **gap** the durable stream → **close** if undrained after 30 s.

| Class | Examples | Droppable |
|---|---|---|
| P0 | `ChamberMessagePosted`, `TransferSettled`, `EnvelopeRevoked` | never dropped — only gapped |
| P1 | read marks, reaction deltas | yes (CRDT converges on pull) |
| P2 / P3 | typing / presence | yes (ephemeral) |

A `Gap` is not a failure. It is the protocol admitting the truth and returning the client to the authoritative path: pull `GET /v1/societies/{id}/events?since={seq}`, reconcile, resume. That is the Sync Engine path (`10 §6`), so slow clients cost no extra machinery.

**Reconnection.** The client persists the highest **contiguous** applied `seq` per Society — contiguity is what makes a cursor a safe resume point.

```
  disconnect ──► full-jitter backoff  U(0, min(30s, 0.5s·2^attempt))
       │
       └─► Resume{ session, cursors:[{soc_A,4471},{soc_B,902}] }
              ├── cursors inside the replay ring ──► Welcome + missed Signals + live
              └── any cursor older than the ring ─► Welcome + Gap(soc_A,4471,9310)
                                                        └─► client pulls via API, then live
```

The ring is bounded per Society at the lesser of **4,096 events or 15 minutes**. It is memory, not durability. Sizing it larger is a false economy: a client offline longer is better served by a bulk pull that can be paged, compressed and rate-limited than by a socket firehose.

**Presence, typing, read receipts.**

| Feature | Storage | Lifetime | Default visibility |
|---|---|---|---|
| Presence | **Relay-process memory**, ephemeral, gossiped between Relay instances over the NATS subject `presence.<society_id>` | 45 s TTL, 15 s heartbeat | co-members of that Society only |
| Typing | never persisted, never an event | 4 s TTL, ≤1 emit / 3 s / Thread | Thread participants; **off by default in E2EE Chambers** (it is content metadata) |
| Read receipts | CRDT LWW-Register per `(citizen, thread)` | durable | per-Society; Charter may disable display |

Presence is `Active | Idle | Focused(scope) | Invisible | Offline`, scoped per Society — a Citizen can be Active in a work Society and Invisible in a personal one, which falls directly out of P1. **`Invisible` is available at Level 0 and is indistinguishable from `Offline`.** Privacy is never an unlock (`02 §4`).

---

## 3. The latency budget

Same region, warm TLS, Society not cold.

```
 SENDER            EDGE        RUNTIME       BUS      RELAY          PEER
   ├─sign+encrypt 3ms─►│          │           │         │              │
   │                   ├─authz 2ms►│          │         │              │
   │  (local optimistic apply <16ms, P2)      │         │              │
   │                   │          ├─decide 1ms│         │              │
   │                   │          ├─persist 8ms─►publish 2ms►fan-out 3ms►
   │                   │          │           │         ├──12ms───────►├─decrypt+paint 8ms
   │◄────12ms ack──────┤          │           │         │              │
```

| Hop | p50 | p95 | Note |
|---|---:|---:|---|
| Client sign + MLS seal | 3 ms | 9 ms | one AEAD seal, one Ed25519 sign; device count costs at Commit, not per Message |
| Client → Edge (RTT/2) | 12 ms | 45 ms | physics and radio wake-up; TLS 1.3 0-RTT on resume |
| Edge authz, limits, idempotency | 2 ms | 6 ms | Envelope decision cached per request, never per session |
| Runtime domain decision | 1 ms | 3 ms | pure, no I/O |
| Persist event + projection | 8 ms | 30 ms | group-commit fsync — the largest controllable cost |
| Bus publish | 2 ms | 8 ms | NATS JetStream |
| Relay match + fan-out | 3 ms | 10 ms | one shared encoded frame per scope |
| Relay → Peer (RTT/2) | 12 ms | 45 ms | |
| Peer decrypt, apply, paint | 8 ms | 24 ms | inside P10's 100 ms interaction-to-paint |
| **End to end** | **51 ms** | **180 ms** | |

**Targets, enforced as SLOs on `fractal_relay_signal_latency_ms`:**

| | p50 | p95 | p99 |
|---|---:|---:|---:|
| Same region | ≤ 120 ms | ≤ 250 ms | ≤ 500 ms |
| Cross region | ≤ 220 ms | ≤ 450 ms | ≤ 900 ms |
| Sender-perceived (local echo) | ≤ 16 ms | ≤ 33 ms | ≤ 50 ms |

The gap between the 51 ms model and the 120 ms target is deliberate: a budget set at the measured mean fails on the first bad day.

**Where the budget goes:** 47% network (two legs, shortened only by edge presence), 16% persistence, 22% client crypto and paint, 15% everything else. Optimization order follows: edge PoPs and 0-RTT, then persistence group-commit, then the client render path. **The domain layer is not on the list** — it is 2% of the budget, and `02 §7` forbids optimizing without a profile.

**Sender perception is off this path entirely.** Local-first applies the Message optimistically and paints in under 16 ms; the round trip only moves the state indicator from *pending* to *committed*. This is why the honest latency conversation is about peer delivery, not sender delivery.

---

## 4. End-to-end encryption: MLS (RFC 9420)

### 4.1 Why MLS

A Society is a group with continuous churn, several devices per member, and sizes from 3 to 5,000.

| Scheme | Add/remove cost | PCS | Fits? |
|---|---|---|---|
| Double Ratchet, pairwise groups | O(N) per sender, O(N²) group | strong, pairwise | No — 500 members × 3 devices = 1,500 sessions per sender |
| Olm/Megolm sender keys | cheap add, **weak remove** — a removed device keeps the key until every sender rotates | weak | No — removal is what a Society does most |
| **MLS (RFC 9420)** | O(log N) tree op for add, remove, update | strong; one Commit heals | Yes |

Removal being cheap *and cryptographically effective* is the deciding property. Ban a member Tuesday; Wednesday's Messages must be unreadable to them without a coordinated rekey across every sender.

### 4.2 Groups, leaves, and the delivery role

One MLS group per `EndToEnd` Chamber, per DM pair, and per Convergence. **Leaves are devices, not Citizens** — that is what makes multi-device work without sharing keys between devices. The Runtime is the MLS Delivery and Authentication Service: it stores KeyPackages, orders and broadcasts handshake messages, and enforces epoch consistency. It is untrusted for confidentiality and trusted only for availability and ordering — exactly the trust model MLS assumes and exactly the one we can honestly claim. Credentials are the FNID key chain from `12`, so a peer verifies "this device belongs to `@handle`" without trusting the directory.

### 4.3 Key schedule and epochs

```
 epoch N ─────────────────────────────► Commit ─────────► epoch N+1
  init_secret_N ─┬─ commit_secret (ratchet-tree path update)
                 ▼
  epoch_secret_N ─┬─ sender_data_secret   header protection
                  ├─ encryption_secret ─► secret tree ─► per-leaf, per-generation keys
                  ├─ exporter_secret ───► SFrame media keys (§7), franking key (§6)
                  ├─ membership_key / confirmation_key
                  └─ init_secret_{N+1}      (init_secret_N is deleted here)
```

**Forward secrecy** at two granularities: the secret tree ratchets per message within an epoch; `init_secret_N` is destroyed on epoch advance. **Post-compromise security** is the property that matters for long-lived communities — a device compromised at epoch N is locked out at N+2 once its path key is replaced by its own Update or its removal, with no one needing to notice the compromise. A quiet group never exercises PCS, so we force it: **every device Updates at least every 7 days or 500 group messages**, whichever comes first.

### 4.4 Churn and Commit batching

Proposals are cheap; Commits are the serialized, expensive operation. One Commit per join would be pathological at Society scale, so proposals batch and commit at **64 proposals or 10 seconds**, whichever first.

**The honest cost:** a removed member can decrypt for up to that window. Irrelevant for a voluntary `Left`, unacceptable for a moderation removal — so `Charter.moderation.immediate_commit_on_removal` defaults **true**, forcing a Commit within one round trip at the price of an epoch storm during a mass ban. Both behaviours are correct in their own context, so the Charter chooses (P1).

Scale limit, already flagged in `10 §12`: beyond **~1,000 leaves**, KeyPackage churn and Commit size bind. **Every MLS figure in the corpus is stated in leaves**, because leaves are what the mechanism consumes; at the published assumption of **2.5 devices per Citizen** (§4.2), the 1,000-leaf ceiling is ≈400 Citizens, and the PH2 operating limit is 500 leaves ≈ 200 Citizens. `14 §4.1`'s "3 to 5,000" describes *Society* size and not group size; a Society may hold 5,000 Citizens and still cannot make one Chamber E2EE for all of them (`61 X10`). Above that, a Chamber requesting E2EE is told so in the client and offered a smaller Chamber or transport encryption — **never silently downgraded**.

**Device fan-out** is one ciphertext per group per Message regardless of device count. Per-device work is a `Welcome` at join and a path secret per Commit: O(log N), not O(N).

### 4.5 Multi-device and history — the hard part

**MLS provides no history.** A device joining at epoch 90 cannot read epoch 12. That is a security property colliding with the expectation that a new phone shows your conversations. Three mechanisms, increasing in risk, each an explicit choice:

| Mechanism | How | Decided by | Risk | Default |
|---|---|---|---|---|
| Device-to-device transfer | new device authenticates to an existing device (QR + passkey), history streams over an ephemeral X25519 session | the Citizen | none beyond those two devices | on |
| Vault-backed archive | client encrypts history under a Citizen-held `HistoryKey`, stores it as ordinary Vault Objects; a new device gets the key device-to-device or via social recovery (`12`) | the Citizen | long-lived key; compromise defeats FS **for the archive**, not the live group | on for DMs, off for Chambers |
| Chamber history handoff | an existing member re-encrypts a bounded window to a new joiner; emits `ChamberHistoryHandedOff` naming them | the **Charter** | a member can always leak what they can read — this makes it attributable rather than pretending it is preventable | off |

The archive trade-off is the one usually glossed over. The client states it in one sentence at the moment of enabling: *"Your archive is protected by a key you hold. If that key is stolen, saved history can be read. Live conversation is unaffected."* A security property a Citizen cannot articulate is one they do not have.

### 4.6 What MLS does not hide

Visible to the Runtime: group membership (someone must route Welcomes), message timing, message size, epoch churn, and which connection sent a frame. Mitigation is padding to 256 B / 1 KiB / 4 KiB / 16 KiB / 64 KiB buckets. **We do not implement cover traffic or mixnet routing** — it would cost every mobile Citizen battery and data, degrade §3 by an order of magnitude, and defend against a global passive adversary this population does not face. Claiming metadata privacy we do not deliver is worse than not delivering it. Rejected openly, not overlooked.

---

## 5. The line: encrypted vs Runtime-readable

| Content | Mode | Runtime can read? |
|---|---|---|
| Direct messages between Citizens | **E2EE (MLS)** | **No** — N6, no decryption path in the code |
| Convergence Threads | **E2EE (MLS)** | **No** |
| `Private` / `Sealed` Society Chambers | **E2EE by default** | **No** |
| Voice and video media (Phase 4) | **E2EE (SFrame)** | **No** — N6 names them explicitly |
| Vault Objects | encrypted before chunking, key wrapped to the ACL | **No** — Custodians see neither content nor file boundaries |
| `Public` / `Discoverable` Chamber content | transport TLS + at-rest | **Yes** — see below |
| Reactions, read marks, presence, typing | metadata | Yes — ephemeral or trivially inferable |
| Ledger, Standing, XP, governance votes | not encrypted from the Runtime | Yes — P12: a private ledger is an unverifiable ledger |
| Membership, Charter, Society structure | readable | Yes — routing, authorization, and Fracture require it |

**Why public content is deliberately readable.** A `Public` Chamber is a *publication*. Encrypting a publication from the infrastructure that publishes it is theatre — the guarantee voids the moment one of thousands of members is hostile, which for a public space is always. What we would forfeit is real and all of it serves the Citizen: search (S13), Charter moderation including `Moderator` agents inside Envelopes (P4), the XP/Contribution/Standing projections, accessibility obligations (transcripts, alt text, reading-order repair — P10/N8), and a child Society's ability to verify and re-index inherited history after Fracture.

**Justified against P8/P9.** P8 requires E2EE for private messages, voice and video — those exact three, all delivered with no plaintext path in code. P9 requires the most private default *that still makes the feature work*; for a public Chamber, encryption does not make the feature work, it breaks it while providing no confidentiality. P9's falsification test — name the feature that breaks without the field and the control that governs it — is answered for every field of a public Message: search, moderation, progression; governed by `Society.visibility` and `Chamber.encryption`, both chosen by humans at creation.

- **Invariant E1.** `Chamber.encryption` is immutable after creation. `Transport → EndToEnd` leaves a plaintext prefix while implying protection; `EndToEnd → Transport` is a silent downgrade. Changing mode means a new Chamber and an archived old one — a visible, attributable, event-emitting act.
- **Invariant E2.** No Runtime code path accepts an MLS group secret, a `HistoryKey`, or an SFrame key. Enforced by a CI lint over the crate graph, not by review discipline. N6 says *absent from the code*, not *unused*.

---

## 6. Moderation under encryption

**What is not possible, stated first.** No proactive scanning of E2EE content — not by the Runtime and **not client-side**, because client-side scanning is a plaintext access path with a policy promise attached (N6, and the `02 §4` Never list). No retrieval of a Message the Runtime was never shown, for anyone, including a Society's own moderators. No content-based rate limiting on E2EE surfaces.

**What works: message franking.** A recipient can prove *this exact plaintext came from this sender in this group at this time*, while the Runtime sees plaintext only if a human chooses to report.

```rust
struct FrankingTag {          // sender computes commitment; Runtime signs; recipient verifies
    commitment:  [u8; 32],    // HMAC(k_frank, plaintext); k_frank is per-message and
                              // travels INSIDE the MLS ciphertext, never to the Runtime
    group: MlsGroupId, epoch: u64, sender: Fnid,
    message_id: Ulid, franked_at: Timestamp,
    runtime_sig: Signature,   // Ed25519 over all of the above
}

struct FrankedReport {        // reporter produces; Runtime verifies
    tag: FrankingTag, plaintext: Bytes,
    opening: [u8; 32],        // k_frank for THIS message only
    reason: ReportReason, reporter: Fnid,
}
```

Properties: **non-forgeability** (a fabricated Message fails commitment verification against a tag the Runtime signed), **non-repudiation** (the tag binds the sender's authenticated MLS credential), **zero standing exposure** (nothing readable is held until a human reports), and **minimal disclosure** (the opening reveals one Message, not the conversation around it; the client shows the reporter exactly what will be disclosed first).

**The cost we accept: deniability.** In a plain MLS group a recipient can produce a plaintext but cannot prove you wrote it. Franking removes that. We take the trade deliberately: in a platform with governance, Standing, and appeals, accountable messaging is worth more than deniable messaging, and the alternative is an E2EE surface where harassment reports are unverifiable assertions and false accusations cost the same as true ones. Deniable, OTR-style protocols are a rejected alternative recorded here so the choice is visible rather than accidental.

**The limits, without spin.** Only reported content is ever seen; a wholly abusive private group with no dissenting member is invisible to us, as it is in every honest E2EE system. Proactive signals on E2EE surfaces are metadata only — send rate, distinct-recipient fan-out, account age, upheld-report rate — which is blunt, false-positive-prone, and itself abusable via mass false reporting. Therefore **no metadata signal alone produces a permanent sanction**: it produces a time-boxed, appealable rate limit or hold (`11 §7`). Upheld reports raise reporter Trust; bad-faith reports lower it — Trust is bidirectional (`01 §7`) precisely so reporting has skin in the game. Public Chambers are moderated conventionally, every action a `ModerationActionTaken` event, every action appealable.

---

## 7. Voice, video, and the Stage (Phase 4)

Deferred to Phase 4 by `02 §3`: text realtime and E2EE must be correct first, and an SFU is a permanent cost centre.

**Topology.** WebRTC with a Selective Forwarding Unit. Full mesh breaks past ~5 peers; an MCU (server-side mixing) is architecturally excluded because mixing requires decrypting media (N6). The SFU forwards packets it cannot read.

```
 Speaker A ─┐   ┌──────────────────────┐   ┌─► Listener 1  720p
 Speaker B ─┼──►│  SFU: forwards       │──►├─► Listener 2  360p (poor link)
 Speaker C ─┘   │  SFrame ciphertext,  │   └─► Listener N  audio only
                │  selects layers/peer │   sees: RTP headers, sizes, timing
                └──────────────────────┘   cannot see: media content
```

| Aspect | Choice | Why |
|---|---|---|
| Audio | Opus 20 ms, DTX, in-band FEC, RED (2 redundant frames) | ~30% bitrate for intelligibility at 10% loss |
| Video | AV1 k-SVC where negotiated → VP9 SVC → VP8 3-layer simulcast | per-layer drop decisions from one encode; simulcast is the compatibility floor |
| Jitter buffer | audio adaptive, 40 ms nominal / 200 ms ceiling; video 3-frame, ≤150 ms | below 40 ms concealment degrades audibly; above 200 ms turn-taking breaks |
| AEC / AGC / NS | client-side APM in browser and native shells | server-side AEC would need decrypted audio |
| Transcoding | **never in realtime**; only asynchronously for recordings | see cost below |

**E2EE media.** Insertable streams (`RTCRtpScriptTransform` and its native equivalents) apply SFrame per frame before packetization, keyed from the Chamber's MLS `exporter_secret` — so media confidentiality follows group membership automatically with no separate key management. Honest limit: the SFU sees headers, frame sizes and timing; frame size correlates with motion, and timing plus DTX reveals **who is speaking and when**. Constant-bitrate padding would close that at roughly 3× bitrate for everyone; it is off by default, and we say so rather than omitting it.

**The Stage Chamber** (`ChamberKind::Stage`) is not a large `Voice` Chamber; the scaling shape differs and so does the path.

| Listeners | Path | E2EE | Mouth-to-ear |
|---:|---|---|---|
| ≤ 500 | single SFU | yes (SFrame) | ~120 ms |
| 500–5,000 | cascaded regional relay SFUs forwarding ciphertext | yes | ~180 ms |
| > 5,000 | broadcast egress (LL-HLS); hand-raise promotes back onto the SFU path | **no** — packaging needs plaintext | ~3 s |

**The threshold is a visible product state, not a silent downgrade.** Clients display the loss of end-to-end encryption before the transition, and a Charter may forbid the transition entirely (the Stage then caps and refuses further listeners). Quietly dropping a guarantee under load is the exact failure this document exists to prevent. Speaker slots and hand-raising are Charter roles, not ad-hoc host controls.

**Recording and consent.** Off by default; requires the `chamber.session.record` capability held by a named role; per-session consent from every participant, with decliners removed from the recording path rather than the session; an on-record indicator rendered from session state that no client setting or Extension can suppress; and a `ChamberSessionRecorded { initiator, consented, started_at }` Domain Event. **There is no Runtime-side recording path for an E2EE session** — a recording is produced client-side by a participant who already holds keys and stored in the Vault encrypted to a declared audience. We cannot prevent a participant from recording, so we make it explicit, consented and attributed instead of pretending otherwise.

**Operational cost, without euphemism.**

| Scenario | Streams | Egress | Per hour |
|---|---:|---:|---:|
| 8-person audio Chamber (~42 kbps with FEC) | 8 pub → 56 sub | ~2.4 Mbps | ~1.1 GB |
| 12-person video Chamber (720p simulcast) | 12 pub → 132 sub | ~19 Mbps | ~8.6 GB |
| Stage: 3 speakers / 2,000 listeners, audio | 3 pub → 6,000 sub | ~250 Mbps | ~113 GB |

Egress dominates. CPU is comparatively cheap — pure forwarding sustains roughly 2,000–4,000 concurrent streams per core, and **any realtime transcoding collapses that by one to two orders of magnitude**, which is why layer selection is structural rather than an optimization. Add 8–15% of connections falling back to TURN relay, doubling egress on those. This is why media minutes are a named economic **Sink** charged to the Society's Treasury (P12, `17`): a feature with unbounded infrastructure cost and no matching sink is how platforms become advertising companies, and we are not doing that (`02 §4`).

---

## 8. Discovery without surveillance (Phase 5)

Phases 1–4 ship declared interests plus search (`02 §3`). The model below is designed now so the data model is not retrofitted later.

`11` invariant 13 — *no inferred value ever appears in `Citizen.interests`* — is P9 encoded as a domain invariant with a property test. Discovery is built on it and on nothing else.

```rust
struct InterestDeclaration {
    citizen: Fnid, tag: InterestTag, weight: u8,      // 1..=5, set by the Citizen only
    declared_at: Timestamp,
    visibility: InterestVisibility,  // Public | SocietiesOnly | MatchingOnly | Private
}
```

`MatchingOnly` matters: an interest can be used to match without appearing on a Profile. Searchable interests and social interests are different needs, and collapsing them forces oversharing.

**The interest graph** is two bipartite edge sets over a community-curated ontology (Citizen→tag, Society→tag) plus explicit affinity edges (follows, saves, "more like this"). Matching is cosine similarity over declared weighted vectors, filtered by declared availability and Society capacity, ordered by overlap. There is **no engagement objective, no learned ranking over behaviour, no per-Citizen model**. Every result carries a machine-readable `why` naming the matched tags and weights: a recommendation a Citizen cannot interrogate is one they cannot correct.

| Used — declared | Used — opt-in, individually toggleable | **Banned** |
|---|---|---|
| interest tags and weights | publish participation in a chosen Chamber | dwell time, scroll depth, view duration |
| declared languages | coarse proximity (geohash precision 4, 1 h TTL) | read-without-reply, open rates, session length |
| declared availability windows | "recently active" flag | content analysis for interest inference |
| coarse timezone band | Serendipity eligibility | contact-list upload; device graph; cross-device linking |
| "open to Convergence" flag | cross-Society interest sharing | IP geolocation for matching (routing/abuse only) |
| explicit follows, saves, "more like this" | sharing Standing with matched Societies | typing cadence, interaction biometrics |
| Societies made public by their members | | wallet or purchase history as a matching input |
| Level / Trust / Standing **as capability gates** | | inferred demographics of any kind |
| search queries typed, for that search only | | third-party trackers, ad identifiers, pixels |
| | | **anything not in the first two columns** |

The last row is operative: the list is **closed**. Adding a signal requires an ADR answering P9's falsification test. Every Citizen has a **"What discovery knows"** surface listing each stored signal, its source, and a delete control — that is not a settings page, it is the falsification test rendered as a screen.

**Serendipity** is the bounded, opt-in answer to cold start that replaces the recommendation surface we refuse to build: off by default; at most **2 per Citizen per week**; groups of **3–5** sharing ≥2 declared interests with overlapping availability and no Society already in common; opens a `Convergence` with `seed: Serendipity` and a 72 h expiry; **declining costs nothing, is never recorded, and never affects future matching** — otherwise declining itself becomes surveillance. Scarcity makes it an event rather than a mechanic, and it delivers a real conversation on a real promotion path instead of a suggestion list.

---

## 9. The social funnel

```
  discovery / invitation / Serendipity
        ▼
  ┌──────────────┐  ephemeral 72h (extends with activity), E2EE MLS group,
  │ CONVERGENCE  │  accrues Fraction, media, reactions, a real Thread
  └──────┬───────┘
         │  ALL of: ≥3 participants · (≥48h activity OR ≥100 Messages) · ≥2 at Level ≥1
         ▼
  ┌──────────────┐  the client surfaces the option; nothing is automatic
  │  ELIGIBLE    │
  └──────┬───────┘
         │  a participant proposes name + Charter template; ≥2/3 accept
         ▼
  ┌──────────────┐  Thread MOVED, not copied. Message IDs, authorship, signatures,
  │ CRYSTALLIZED │  reactions preserved. Tenure backdated to the first word spoken.
  │ → SOCIETY L0 │  MLS group carried forward at its current epoch.
  └──────┬───────┘
         │  Levels 0→5 (11 §2.3): roles · Chambers · Council · Direct voting · Facets · Federation
         ▼
  ┌──────────────┐──► FORK       copy the Charter, divide nothing
  │ SOCIETY L5   │──► FRACTURE   split membership, treasury, vault, history (11 §3.2)
  └──────────────┘──► DISSOLUTION distribute, export, seal, archive
```

| Transition | Gate | Preserved | Lost |
|---|---|---|---|
| Discovery → Convergence | opt-in, rate-limited (§10) | — | — |
| Convergence → Eligible | ≥3 participants **and** (≥48 h **or** ≥100 Messages) **and** ≥2 at Citizen Level ≥1 | everything | — |
| Eligible → Crystallized | ≥2/3 of participants accept name + Charter template | Message IDs, authorship, device signatures, reactions, media by reference, accrued Fraction, backdated tenure, the MLS group | nothing |
| Fracture (L5) | Charter `fracture_rules` quorum + mandatory dry run | Standing, tenure, Facets, full readable parent history from every child | nothing; parent Archived, never deleted |
| Dissolution | Charter quorum | authored history, Facets released to owners, sealed Log | forward-looking Standing |

**Why these thresholds.** Three participants because two is a DM. Forty-eight hours *or* a hundred Messages because a slow week-long thread and an intense afternoon are both real communities. Two participants at Level ≥1 because Crystallization is a Sybil target — creating Societies at scale must cost accumulated participation, not a signup. Two-thirds acceptance rather than a founder's unilateral act because a Society whose members did not agree to exist will not.

**Encryption continuity.** On Crystallization the Chamber inherits the Convergence's MLS group id and current epoch: no rekey, no re-encryption, no moment where a participant loses access to what they already read. Fracture is harder, and we state the limit: each child Chamber advances to a new epoch with non-assigned members removed, partitioning *future* content correctly, but **history a member could already decrypt stays decryptable to them forever.** Cryptography cannot un-read a Message. Fracture divides ownership, governance, treasury and future access — not memory.

---

## 10. Anti-abuse on social surfaces

| Vector | Control |
|---|---|
| Spam | per-principal token buckets at the gateway, Trust-scaled |
| Mass-DM | rolling **distinct-recipient** budget scaled by Trust; new Citizens may DM only within a shared Society or after an invite |
| Brigading | cross-Society burst detection on join+post correlation; Charter **Shield Mode** (slow mode, new-member hold, invite-only) enabled by any moderator role in one action, auto-expiring in 24 h |
| Harassment | franked reporting (§6); block severs Signals bidirectionally; mute is client-side only |
| Sybil Society creation | Crystallization thresholds (§9); creation fee as a Sink |
| Link spam / mass mentions | external links gated until a Trust threshold or one vouch; `@everyone`-class mentions are a Charter capability, never a default |
| Report abuse | bad-faith reports lower reporter Trust; reports are attributed and reviewable |
| Automated posting | Agents get separate, tighter buckets (`10 §10`); every Agent Message is visually distinguished and carries `envelope_ref` (P4) |

**New-Citizen envelope** (first 72 h or until Level 1): DMs only to Citizens sharing a Society or who invited them; ≤5 Convergence joins/day; ≤3 Society joins/day; no `@everyone`-class mentions; no external links. Every restriction is displayed with the condition that lifts it — a limit whose exit is hidden is a dark pattern.

**Standing gates capability without creating a caste.** The risk is obvious: a reputation system that gates capability becomes a caste where a bad first week is permanent. Five structural properties prevent it, as requirements rather than aspirations. (1) **Every gate publishes its exit condition** — a gate with no visible path out is a ban in disguise. (2) **Gates are Society-scoped**; low Standing in one Society has no effect in another, and no global score follows a person. (3) **Trust decays toward neutral in both directions** — old sins expire, and so does old credit; a reputation that only accumulates is a hierarchy. (4) **Gates are never purchasable** (`02 §4`); a caste system with a paid exit is worse than one without. (5) **Gates restrict amplification, never participation** — mass-DM, external links, `@everyone`, creation rates, Facet minting are gateable; reading, posting in your own Society's ordinary Chambers, and Charter-entitled governance are not. Standing is rendered as capability state, never as a leaderboard of persons (`33`).

---

## 11. The notification model

**A Signal is transport. A notification is an interruption.** Conflating them is how a platform becomes noise.

```
 Domain Event ─► Relay fan-out ─► Signal (every subscribed connected front end)
                                      │
                                      ▼
        NOTIFICATION POLICY  per Citizen × Society × Chamber, Thread overrides
            level: All | Mentions | Nothing · quiet hours · class: Normal | Urgent
                                      │
                                      ▼
        DEDUPE REGISTER  notification_id = H(society, thread, trigger_event_id)
            per-Citizen ephemeral LWW, 3 s grace; focused device claims it
                                      │
                                      ▼
        DELIVERY  in-app │ desktop │ push │ email digest │ CLI Terminal
                                      │
                              READ ───┴──► Signal clears it on every device
```

**Defaults follow P9** — the least noisy setting that keeps the feature working: new Society → `Mentions`; new Chamber → inherit; DM → `All`; active Convergence → `All`; a governance vote you are eligible for → `All`; everything else → `Nothing`.

**Cross-device dedupe.** `notification_id` is deterministic, so every device computes the same one. The first device to display claims it within a 3-second grace window; if the Citizen is present and focused anywhere (presence + focus), push to other devices is suppressed entirely; reading anywhere clears everywhere via a P1 Signal. This is why read state is stored even when a Society disables receipt *display* — the mechanism and the social feature are separable.

**Quiet hours** are a per-Citizen local schedule, suggested at onboarding, off unless set. Only `Urgent` is delivered, and `Urgent` is a closed enum with exactly four variants: a governance vote closing within the hour, a security event on the Citizen's identity, a wallet action requiring live human confirmation (P4 `confirm_classes`), and a mention flagged urgent by a Charter role holding that capability. **No product surface can define a new urgent class** — that is the mechanism that stops urgency inflation.

**Agents notify without becoming noise** (P4): an Agent cannot emit a notification at all. It emits a Domain Event and the Citizen's policy decides. Agents have a per-Society budget (default 3 per Citizen per day); exceeding it throttles the Agent and counts against its Trust. No Agent may mark anything `Urgent`. Agent-originated notifications are visually distinguished on every surface including the CLI Terminal (`33 §2.3`).

**Batching.** Non-urgent notifications coalesce per Society on a 90-second window: twelve Messages in a busy Chamber are one notification.

**Permanently banned classes** (`02 §4`): "you haven't posted in a while", "N people are active now", streaks and streak-loss warnings, re-engagement prompts, artificial urgency timers — and generally any notification whose trigger is *absence of activity* rather than *occurrence of an event*. A notification must name a thing that happened.

---

## 12. Alternatives rejected

| Alternative | Offers | Why rejected |
|---|---|---|
| **Matrix** | mature federated protocol, real E2EE, existing clients | its state-resolution DAG duplicates the per-Society event Log we already own (`10 §4`) — two ordering systems; Megolm removal semantics are weakest exactly where a Society is strongest (§4.1); adopting Matrix means adopting a data model that is not P1-shaped. We reuse its lessons, not its stack. |
| **XMPP** | battle-tested, extensible, federated | the XEP surface is a compatibility maze; presence and MUC semantics predate multi-device; OMEMO is an extension rather than a foundation. MLS is strictly better for our shape. |
| **Signal protocol for groups** | best-scrutinized 1:1 security | O(N²) group fan-out and O(N) rekey per membership change. Correct for a messenger, wrong for a community. |
| **ActivityPub** | fediverse interoperability, real network effects | no E2EE, no group semantics, no capability model, inbox delivery with no ordering guarantee. It is a publishing protocol, not a conversation protocol. **Retained as a future one-way publishing adapter for `Public` content behind the P5 boundary** — that is where it genuinely fits. |
| **Socket.IO** | fast to build, automatic fallbacks | brings its own framing, reconnection semantics, and a runtime we do not need. Our resume model is `since(seq)` against the Log; a generic reconnection layer would sit above the mechanism that actually matters and hide it. |
| **Third-party chat SDK** | ships in a week, someone else operates it | fails N6 (the vendor holds the plaintext path), P5 (vendor types in the domain), and P2 (no home for a local-first replica). The most tempting shortcut in this document, and it forfeits the product's premise. |
| **MCU / server-side mixing** | cheaper client CPU | requires decrypted media. Non-starter under N6. |
| **Client-side content scanning** | proactive safety on E2EE surfaces | a plaintext access path with a policy promise attached. Never list. |
| **Cover traffic / mixnets** | metadata privacy | order-of-magnitude latency and battery cost against an adversary this population does not face. Declined openly rather than half-implemented. |
| **WebTransport over HTTP/3** | better head-of-line behaviour on lossy links | **deferred, not rejected** — same `SignalFrame`, different carrier, behind the `Relay` port; evaluated in Phase 5 alongside native mobile (`34`). |

---

## 13. Failure modes

| Failure | Degradation | Recovery |
|---|---|---|
| Relay instance loss | jittered reconnect; presence stale and **marked stale**, never shown fresh | `Resume` with cursors, ≤2 s typical |
| Relay fully unavailable | clients poll `since(seq)` every 10 s; typing and presence vanish; Messages still send and arrive (P2 outbox) | automatic on return |
| Bus lag | Signals late; Log unaffected; clients with stale cursors pull | drains |
| Replay ring exhausted | `Gap` → API pull | immediate |
| Slow client | coalesce → shed → gap → close (§2) | reconnect and pull |
| Relay restart herd | edge connect token bucket + `Bye{Draining, retry_after}` with full jitter | spread over 30 s |
| MLS Commit storm (mass join) | proposals batch; epoch advance capped at 1 per 10 s per group | drains |
| MLS epoch divergence | client re-Welcomes via external commit, shows "reconnecting securely"; **never a plaintext fallback** | one round trip |
| Device key lost | history unrecoverable absent a Vault archive or another device; **no Runtime-side recovery exists, because one cannot** | identity via social recovery (`12`); history per §4.5 |
| SFU node loss | renegotiate to another SFU; ~2 s audio gap; session survives because session state lives in the Runtime, not the SFU | automatic |
| Stage crosses broadcast threshold | explicit, visible E2EE downgrade notice — or refusal, if the Charter forbids it | — |

---

## 14. What would make us change this

- **MLS proves unworkable above ~1,000 leaves in production.** → Sender-key groups with forced periodic rekeying for large Chambers only; MLS retained for DMs, Convergences and small Chambers; the reduced guarantee published in the client. Never silently.
- **`Gap`-driven pulls dominate traffic**, meaning the replay ring is mis-sized for real mobile behaviour. → Enlarge the ring and add a compressed delta-pull endpoint before enlarging socket buffers.
- **Franking produces a measurable chilling effect** on private Chamber usage. → Re-open the deniability trade-off (§6) as an ADR. The decision is deliberate, not permanent.
- **SFU egress outruns the media Sink.** → Tighten default video layers, make video opt-in per Chamber, price the Sink honestly — never subsidize it into an advertising model.
- **Declared-interest discovery fails to bootstrap.** → Better ontology, better onboarding, Society-side invitations. **Not** behavioural inference: P9 sits at position 2 in the conflict order and is not traded for growth.
