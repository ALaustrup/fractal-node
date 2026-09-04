# 13 — Data and Storage

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `10-system-architecture.md`, `11-domain-model.md`.
> **Governs:** the Vault; the Object → Version → Manifest → Shard stack; encryption, chunking, erasure coding and content addressing; the Custodian protocol and its Attestations; storage and bandwidth settlement in Fraction; the media pipeline; retention, tombstones and takedown; media search; the `BlobStore` port and the Phase 4 migration to the Custodian mesh.

---

## 1. Position

The Vault (boundary **S4**, `10 §3`) owns every byte a Society persists that is not a Domain Event. The Log holds *what happened*; the Vault holds *what was made*. A Message referencing a video stores an `ObjectRef`, never bytes — which is what keeps the Log small enough to replay (P6) and the Vault dumb enough to distribute (P5).

Three commitments fix everything below.

1. **A Custodian is untrusted.** It sees ciphertext and hashes. Every efficiency this chapter gives up buys that property (P8, N6).
2. **Distribution is an implementation detail.** Phases 1–3 write to an S3-compatible store behind `BlobStore`; Phase 4 swaps in the Custodian mesh with no domain-code change (P5, `02 §3`).
3. **Payment follows proof, never claim** (P12, `11 §2.7`).

**Vocabulary.** The stored, distributed, rewarded artifact is a **Shard**. "Chunking" names a pipeline stage; "chunk" appears below only for BLAKE3's internal 1 KiB leaves.

---

## 2. The Stack

```
 OBJECT   object_id · society_id · VaultPath · Acl · MediaMeta · versions[] (append-only)
    │
    ▼  1..N immutable snapshots
 VERSION  version_id · author · at · parent · tombstone?
    │
    ▼  exactly one
 MANIFEST size · content_key(WrappedKey) · erasure RS(10,16) · merkle_root
          shards: Vec<ShardRef> (ordered) · segment table (offsets, nonces, pad buckets)
    │
    ▼  ordered, content-addressed
 SHARD ~4 MiB ──► 16 RS fragments ──► 16 Custodians, ≥ 6 failure domains, VRF-assigned
```

An Object is a name with a history; a Version is an immutable byte sequence; a Manifest is the recipe; a Shard is the unit of distribution and reward. Only the Manifest layer knows the mapping, and only the ACL's key holders can decrypt it.

---

## 3. The Write Pipeline

```
 plaintext
   │ ① SEGMENT  keyed FastCDC, gear table = HKDF(vault_key,"cdc")
   │            min 256 KiB · target 1 MiB · max 4 MiB
   │ ② ENCRYPT  each segment sealed alone: XChaCha20-Poly1305,
   │            key = HKDF(content_key,"seg",seg_tag); padded to {256K,512K,1M,2M,4M}
   │ ③ CHUNK    coalesce sealed segments into ~4 MiB Shards.
   │            Shard boundaries never coincide with segment boundaries.
   │ ④ ERASURE  Reed-Solomon(10,16) over GF(2^8), per Shard
   │ ⑤ ADDRESS  BLAKE3-256 multihash per fragment; Merkle root over ShardRefs
   │ ⑥ DISTRIBUTE  VRF assignment → 16 Custodians across ≥ 6 failure domains
   ▼
 ObjectStored { object_id, version_id, merkle_root, shard_count, bytes }
```

### 3.1 Why Encryption Precedes Chunking

`11 §2.7` states it as an invariant. The reasons, ordered by cost of getting them wrong:

1. **Convergent encryption leaks membership.** Chunk-then-encrypt with content-derived keys makes identical plaintext produce identical ciphertext platform-wide, enabling the confirmation-of-file attack. That is a P9 violation, unfixable once ciphertext is published.
2. **A plaintext artifact must not exist in the code path.** N6 requires the *absence* of a server-side plaintext path, not its disuse. Encrypt-first means the only thing ever named, hashed, spooled or handed to an adapter is ciphertext.
3. **Unkeyed boundary sequences are a fingerprint.** Content-defined boundaries over plaintext with a public gear table yield a length sequence that near-uniquely identifies a file — a practical attack on CDC backup systems. Keying the gear table (step ①) puts the boundary decision inside the key domain.
4. **Parity over plaintext leaks linear relations.** RS fragments are linear combinations of their inputs; coding must follow the seal.

**Invariant V1.** No Shard boundary, length, or fragment index is derivable from plaintext without the Vault key. Padding buckets, segment coalescing and the keyed gear table make this true. Residual leak: a coarse size-bucket distribution. Accepted, and stated.

### 3.2 The Honest Cost

Dedupe is scoped to a key domain. Two Societies storing the same video store it twice.

Social payloads are dominated by already-compressed media, where measured cross-tenant dedupe runs 5–15%. Against 1.6× erasure overhead, forgoing 10% costs ~6% of gross capacity. Buying it back costs the confirmation-of-file attack; `00 §2` ranks P8 first and P10 twelfth. Not close.

What we keep is the case that matters: *within* a key domain, appending 30 seconds to a 40-minute recording re-seals only the touched segments and reuses every other segment's ciphertext verbatim, because segment keys derive from a content tag rather than a position.

---

## 4. Chunking

| Parameter | Value | Reason |
|---|---|---|
| Algorithm | FastCDC, 64-bit keyed gear | ~3× Rabin throughput at equal boundary quality |
| Segment min/target/max | 256 KiB / 1 MiB / 4 MiB | Bounds re-seal after an edit; caps Manifest length |
| Shard target | 4 MiB | See below |
| Inline threshold | < 64 KiB | Carried encrypted inside the Manifest; no Shard, no settlement |
| Normalization | level 2 | ~85% of segments land within 0.5–2× target |

**Why 4 MiB.** Per-Shard coordination overhead is ~1.5 KiB (Manifest entry, Attestation slot, 16 placement records): 0.036% at 4 MiB, 0.57% at 256 KiB, 0.002% at 64 MiB. But a 64 MiB Shard forces a 640 MiB repair read (§6.4) and wastes egress on range reads. 4 MiB also keeps one RS(10,16) decode inside cache on commodity hardware.

**Why content-defined, not fixed-size.** Insert one byte at the head of a 1 GiB Object. Fixed 4 MiB boundaries shift all 256 Shards — 1 GiB rewritten, 1.6 GiB re-stored, 4 096 placement decisions. Content-defined boundaries change one or two Shards — ~8 MiB. For versioned media, collaborative canvases and append-heavy recordings, that is two orders of magnitude in write amplification and settlement churn.

**The cost of CDC.** The rolling hash is a full plaintext pass: SIMD FastCDC sustains 2–4 GB/s per core, so 1 GiB spends ~300 ms on boundary detection before encryption (~1.5 GB/s) and RS coding (~1 GB/s) — roughly 2 core-seconds per GiB at ingest. Shard sizes become variable, forcing the padding-bucket scheme. Fixed-size chunking has none of these costs and is the right answer for append-only data — which is why the Log does not use this pipeline.

---

## 5. Content Addressing and Verified Streaming

BLAKE3-256, multihash-encoded, everywhere: `ObjectId`, `VersionId`, `ShardRef.hash`, fragment addresses (`11 §6`).

Throughput (5–10 GB/s multi-threaded vs 1–2 GB/s for SHA-256) and parallelism matter. **Verified streaming is decisive.** BLAKE3 is a Merkle tree over 1 KiB leaves, so any byte range authenticates against the root *before* delivery to the decoder. A Citizen validates each leaf as it arrives; a substituted fragment fails in milliseconds, not after a 900 MB download and a disagreeing whole-file digest. This is what makes an untrusted Custodian tolerable.

```
   ROOT = ShardRef.hash (from the Manifest)
     └─ 12 levels ─ 4096 leaves of 1 KiB over a 4 MiB Shard
   verify one leaf with 12 sibling CVs = 384 B → authenticated range read
```

**Outboard cost.** N−1 internal nodes of 64 B: 4 095 × 64 B = 256 KiB per 4 MiB Shard, **6.25%**. Stored only for classes needing authenticated range reads (media, Derived). A sequential full read recomputes the tree and pays zero.

`Manifest.merkle_root` is a BLAKE3 tree over the ordered `ShardRef` list — it authenticates the recipe, so reordering, dropping or substituting a Shard fails. `11 §2.7` requires verification on every reconstruction; failure emits `ReplicaCorrupt` and slashes (§7.6).

**Invariant V2.** Every byte delivered to a decoder is authenticated against a hash from an independently resolved Manifest. There is no unverified read path.

---

## 6. Erasure Coding

### 6.1 Scheme

**Reed-Solomon(10,16) over GF(2^8)** for Primary data: any 10 of 16 fragments reconstruct.

| Scheme | Overhead | Tolerated losses | Repair read amplification |
|---|---|---|---|
| 3× replication | 3.00× | 2 | 1× |
| RS(4,6) | 1.50× | 2 | 4× |
| **RS(10,16)** | **1.60×** | **6** | **10×** |
| RS(20,30) | 1.50× | 10 | 20× |
| RS(6,9) — Derived | 1.50× | 3 | 6× |

RS(10,16) is the knee: three times the loss tolerance of 3× replication at 53% of the storage. RS(20,30) is nominally better on both axes and rejected because repair reads 20 fragments, doubling the dominant recurring cost (§6.4), and 30 distinct failure domains is not a Phase 4 population. Classic RS beats locally-repairable codes here on implementation maturity and analysability; LRC is the first optimization to revisit if §6.4 binds, as an ADR.

### 6.2 Durability Arithmetic

With *q* the probability a fragment's Custodian is permanently lost within one repair window, the Object survives unless 7 of 16 are lost:

```
  P(loss/window) = Σ(k=7..16) C(16,k) · q^k · (1−q)^(16−k)

  A — consumer mesh: 5%/month attrition, 72 h repair window
      h = −ln(0.95)/30 = 1.710e-3/day ;  q = 1 − e^(−3h) = 5.117e-3
      C(16,7)·q^7·(1−q)^9 = 11440 · 9.23e-17 · 0.9549 = 1.008e-12
      (k ≥ 8 terms sum < 1e-14)
      × 121.7 windows/yr = 1.23e-10  →  ANNUAL DURABILITY 0.99999999988 (~10 nines)

  B — datacentre: 25%/yr attrition, 24 h window
      q = 1 − e^(−(−ln 0.75)/365) = 7.88e-4
      11440 · (7.88e-4)^7 = 2.20e-18 ; × 365 = 8.0e-16  (~15 nines)
```

**Do not believe Scenario B.** Both assume independence, and independence always breaks. We publish durability two ways:

- **Independent-failure durability:** 10 nines (A) — the number the code is designed against.
- **Correlated-failure tolerance:** with 16 fragments over 8 failure domains at 2 each, the scheme survives total simultaneous loss of **any 3 of 8 domains** (6 lost, 10 remain = exactly *k*). This is a placement property, not a coding property, and it is the number that actually protects the platform.

**Durability is a property of the repair loop, not of the code.** If repair stalls 30 days at A's attrition, *q* = 1 − e^(−30h) = 5.0e-2 and per-window loss becomes 11440 · 7.81e-10 · 0.63 ≈ 5.6e-6 — 5.6 million times worse. Hence `ReplicaLost` is a Domain Event with a page, not a metric.

### 6.3 Repair Triggers

| Trigger | Condition | Priority |
|---|---|---|
| Attestation failure | 3 consecutive missed/invalid for one fragment | Normal |
| Availability floor | live fragments < 13 | Normal |
| Urgency floor | live fragments < 12 | Urgent — pre-empts settlement |
| Critical floor | live fragments ≤ 11 | Critical — page |
| Corruption | hash mismatch on any read | Immediate + slash |
| Graceful drain | Custodian `Draining` | Background, spread over the window |
| Domain concentration | placement < 6 distinct domains | Background rebalance |

Repair is elected, not volunteered: the coordinator picks a Repairer with spare committed capacity, adequate Trust, and a failure domain the Shard does not already occupy.

### 6.4 Repair Bandwidth

Regenerating one lost 4 MiB fragment reads *k* = 10 fragments: 40 MiB read per 4 MiB written. **10× amplification is the largest recurring cost in the mesh.**

```
  1 PiB logical → 1.6 PiB stored. 5%/month attrition:
    lost   = 0.05 × 1.6 PiB              =  80 TiB/month
    reads  = 10 × 80 TiB                 = 800 TiB/month
    egress = 800 × 1.1e12 × 8 ÷ 2.592e6  = 2.72 Gbps, continuous, forever
```

Consequences designed for: repair is throttled to a configured fraction of each Custodian's committed egress; repair bounties are paid from the *same* emission cap as custody (§8.2), so churn dilutes the rate rather than inflating supply; and graceful exit (§7.7) converts unplanned 10× repair into planned 1× hand-off.

---

## 7. Replication Policy and the Custodian Protocol

### 7.1 Data Classes and Floors

| Class | Contents | Scheme | Min domains | Extra full replicas | Tier |
|---|---|---|---|---|---|
| **Sovereign** | Charters, wrapped Vault keys, MLS state, Anchors | RS(10,16) | 8 | 4 | Hot |
| **Log** | Event segments, checkpoints | RS(10,16) | 8 | 2 | Hot |
| **Primary** | Uploaded originals, Facet media, canvas checkpoints | RS(10,16) | 6 | 0 | Hot → Warm |
| **Derived** | Ladder rungs, thumbnails, previews | RS(6,9) | 3 | 0 | Hot |
| **Archived** | Sealed Societies, Dissolution archives | RS(10,16) | 6 | 0 | Cold |
| **Ephemeral** | Drafts, staging, scratch | 1 replica | 1 | 0 | Hot |

Derived data is weaker on purpose: it is regenerable, so losing a 720p rendition costs CPU, not information. Sovereign data gets belt and braces because a lost Vault key is unrecoverable by construction (§10.5). Tier assignment uses per-Object aggregate access recency — never per-Citizen access history (P9).

### 7.2 Placement

```rust
fn assign(shard: &ShardRef, epoch: EpochId, set: &CustodianSet) -> Placement {
    let seed = vrf_output(coordinator_key, shard.hash, epoch);   // publicly verifiable
    set.iter()
       .filter(|c| c.state == Active && c.free_bytes >= shard.size)
       .filter(|c| charter_policy.permits(c))       // jurisdiction / tier / min-Trust
       .map(|c| (blake3(seed, c.fnid), c))
       .sorted_by_key(|(h, _)| *h)
       .take_diverse(16, /* min_domains */ 6)       // domain = (ASN, /24 or /48, geo cell)
       .collect()
}
```

Neither the Custodian nor the Society chooses. A Society may *constrain* through its Charter and pays for the narrower pool in a higher settlement price. It may not name individuals — that reintroduces self-dealing (§8.3).

### 7.3 Charter-Raised Policy

**Invariant V3.** `effective = max(platform_floor, charter_policy)`, field by field.

```rust
struct CharterStoragePolicy {
    min_failure_domains: Option<u8>,      // clamped upward only
    extra_full_replicas: Option<u8>,      // additive only
    jurisdictions:       Option<Vec<Iso3166>>,
    min_custodian_trust: Option<Trust>,
    tier_floor:          Option<Tier>,
    retention_days:      Option<u32>,     // clamped: max(platform_min, requested)
    key_escrow:          KeyEscrow,       // Shamir M-of-N over Charter roles
}
```

P1 sovereignty inside a P8/P12 envelope: a Society governs its durability upward at its Treasury's expense, and nobody can vote themselves below the floor. Raising the policy raises the bill immediately and visibly.

### 7.4 Custodian Lifecycle

```
 Registered ──► Bonded ──► Provisioning ──► Active ──┬──► Draining ──► Exited
                                                     ├──► Probation ──► Active
                                                     └──► Probation ──► Slashed ──► Exited
```

**Registration.** `CustodianRegistered { fnid, committed_bytes, tier, region, asn, endpoints, storage_class }`. Nothing is trusted yet.

**Capacity commitment.** Before `Active` the coordinator issues a seed; the Custodian writes `committed_bytes` of `BLAKE3-XOF(seed ‖ offset)` filler and answers random-offset reads under a deadline derived from its declared throughput. This catches a Node committing 100 TiB while owning 1 TiB. It does **not** catch fast on-demand regeneration — mitigated by a deadline tighter than XOF regeneration at scale, and by displacing filler with real Shards quickly, after which regeneration is impossible.

**Stake.** `bond = bond_rate × committed_bytes`, set at roughly two months of expected earnings (~100 FRC/TiB at §8.2's reference rate). Below one month, slashing is cheaper than compliance; far above two, capital cost prices out honest Custodians. The bond is `locked` in the Custodian's Wallet (`11 §2.6`) and released 30 days after `Exited`.

### 7.5 The Attestation Scheme

A Custodian must prove it holds specific bytes at a specific time — cheaply, without the verifier storing those bytes, without precomputation, and with no option cheaper than actually storing them.

**Beacon.** `B_e = BLAKE3(anchor_root_at_e ‖ e)`. Unpredictable before epoch *e* because the Anchor (`01 §6`) commits to events not yet written.

**Sampling.** Exhaustive challenge is unaffordable: a 10 TiB Custodian holds ~2.6M fragments at ~11 KiB proof each = 28.8 GiB per epoch. Instead sample *s* fragments using `B_e`:

```
  Detect withholding of fraction f at confidence 1−δ:  s ≥ ln(δ)/ln(1−f)
     f = 1%,   δ = 1%  →  s = ln(0.01)/ln(0.99)  =   459
     f = 0.1%, δ = 1%  →  s = ln(0.01)/ln(0.999) = 4 603
```

We sample **s = 460 per Custodian per epoch**, scaled up with bond at risk. Proof traffic: 460 × 11 KiB ≈ **5.1 MiB per epoch** — four orders of magnitude better, detecting 1% withholding at 99% confidence in one epoch and 0.1% within ten.

**Per-fragment challenge**, for sampled fragment *F* whose root the verifier already holds:

```
  m = 8 leaves:  idx_j = BLAKE3(B_e ‖ custodian_fnid ‖ F.hash ‖ j) mod leaf_count
  response      = { leaf bytes[idx_j], sibling path[idx_j] }  for j in 0..8
  aggregate     = BLAKE3(concat of all leaves and paths across all s fragments)
  Attestation   = ( e, custodian_fnid, aggregate, Sig_custodian(B_e ‖ aggregate) )
```

The verifier recomputes paths against `F.hash` and **stores nothing**.

| Attack | Why it fails |
|---|---|
| Precompute | `B_e` depends on an Anchor root that does not exist until epoch *e* |
| Keep hashes, drop bytes | Leaf indices are chosen after commitment; the response must carry real leaf bytes |
| Store a digest | BLAKE3 is not invertible; forging authenticated leaves requires the bytes |
| Fetch from a peer on challenge | Deadline `max(500 ms, 3 × p50 RTT + size term)`; all 16 replicas are challenged in the same instant, so no peer is free |
| Replay | The signature binds `B_e`; a stale beacon is rejected |
| One answer, many identities | Each Attestation binds `custodian_fnid`; sample sets differ per identity |

**Verifier cost:** 460 × 8 × 12 = 44 160 BLAKE3 compressions plus one signature check per Custodian per epoch. For 10 000 Custodians, under a minute of one core per day. Cheap on the side that must scale.

**Cadence:** 24 h, offset per Custodian by `BLAKE3(fnid) mod 86400` so challenges spread. Hot-tier Custodians additionally face unannounced **retrieval audits** through the normal serving path. Attestation proves possession; a retrieval audit proves service. They are different failures.

### 7.6 Slashing

| Violation | Response |
|---|---|
| 1 missed Attestation | Warning event, no penalty — networks blink |
| 3 consecutive missed | `Probation`; assignments frozen; earnings withheld pending cure |
| 7 consecutive missed | Repair; slash = repair cost + 5% of bond; Trust decrement |
| Invalid Attestation | Slash 20% of bond; `Probation` |
| Fragment hash mismatch on read | `ReplicaCorrupt`; slash 20%; reassign |
| Retrieval SLO breach ×5 | Tier demotion; bandwidth multiplier reduced |
| Ungraceful exit holding N fragments | Slash ∝ `N × repair_cost`, capped at bond |

Slashed Fraction is a Sink: part funds the repair bounty, the remainder burns. It is never credited to the coordinator or the platform — that would create an incentive to slash.

### 7.7 Graceful Exit

`CustodianDrainRequested` → re-replication scheduled across a drain window (default 14 days, minimum 72 h) → the Custodian keeps serving and **keeps earning at full rate** → at zero assignments, `CustodianExited`; bond releases after a 30-day clawback.

Full rate during drain is deliberate. A graceful exit costs the network 1× hand-off; a vanishing one costs 10× repair. Making the honest exit the profitable one is cheaper than any penalty.

---

## 8. Compensation

### 8.1 Metering

Two quantities, both **verified-only** — claimed bytes are worth zero.

| Quantity | Unit | Evidence | Counted when |
|---|---|---|---|
| Custody | TiB-hour held | Valid Attestation for the epoch | Fragment assigned, live, attested |
| Service | TiB served | Retrieval receipt signed by the *retrieving* principal, naming `(fragment_hash, bytes, epoch)` | Receipt resolves to a Manifest whose ACL admits that principal |

Sampling generalises: passing the 460-fragment sample credits the full attested assignment set; failing credits zero. Partial credit would incentivise holding only a sampled subset.

```
  S(c,w) = α · custody_TiB_hours + β · served_TiB
  α = 1,  β = 720   ("serving a TiB pays like holding a TiB for a month")
  clamp:  served_TiB ≤ γ · custody_TiB,  γ = 30
```

The γ clamp is load-bearing: without it, a Custodian holding 1 TiB could claim 10 000 TiB served and take nearly the whole pool (§8.3).

### 8.2 Settlement Window, Cap, and Price

**Window: 24 hours**, closing 00:00 UTC, settled by the `StorageSettlement` saga (`11 §5`), idempotent per window, posted under a dedicated `PostingReason` (`11 §2.6` — closed enum, no `Other`).

Price is **derived, not fixed**: cap first, divide pro rata. This is the only construction that bounds emission under adversarial growth (P12).

```
  emission(w) = min( cap_w , Σ S(c,w) × reference_rate )
  price(w)    = emission(w) / Σ S(c,w)
  pay(c,w)    = S(c,w) × price(w)

  Worked example, cap_w = 5 000 FRC/day
    2 PiB held = 2 048 TiB × 24 h              =  49 152 TiB-hours
    40 TiB served/day × β=720                  =  28 800
    Σ S(c,w)                                   =  77 952 units
    price = 5 000 / 77 952                     =   0.06414 FRC/unit
    10 TiB Custodian: 240 TiB-h × 0.06414      =  15.39 FRC/day ≈ 468 FRC/month
    per TiB-month of custody: 730 × 0.06414    =  46.8 FRC
```

Doubling network capacity halves the price; it does not double emission. **Invariant V4: `emission(w) ≤ cap_w`, always, with no discretionary override.** Repair bounties draw from the same cap.

**Annual bound:** 5 000 × 365 = **1 825 000 FRC/yr**. `13` fixes the rule that the storage-and-bandwidth Source claims at most **20% of the platform's annual emission ceiling**; `17-economy-fraction.md` owns that ceiling's absolute value and the taper. Design intent: Society storage fees (a Sink) grow to exceed Custodian payments (a Source), so net storage emission trends to zero at maturity. The cap holds whether or not that happens.

**Peg constraint, stated for `17`:** custody must beat a Custodian's ~$2.04/TiB-month marginal cost (§12). At 46.8 FRC/TiB-month and a 2× margin requirement, FRC's utility value must reach ≈ **$0.087** for the mesh to recruit honest capacity. If it does not, Phase 4 keeps S3 as a paid replica and says so publicly (§13).

### 8.3 Anti-Fraud

**Invariant V5.** The per-byte-hour Sink rate charged to a Society exceeds the per-byte-hour Source rate paid to Custodians by `σ`, where **`σ ≥ 1.25` is a floor**, not a target. Reconciled against `17`'s administered tariff and `§6.1`'s 1.60× replication factor, the observed value is **2.23** (`61 X6`); a settlement window in which it would fall below 1.25 is refused, not repriced.

```
  Society pays       58.5 FRC/TiB-month   (46.8 × 1.25)
  Custodians receive 46.8 FRC/TiB-month
  Round-tripping your own data nets      −11.7 FRC/TiB-month
```

| Attack | Mitigation | Residual |
|---|---|---|
| **Self-dealing** — pay yourself to store your own data | V5 makes it strictly loss-making; VRF assignment means you cannot target your own Shards; settlement excludes fragments whose Custodian is the uploading principal or shares its Operator | A whale could eat the σ loss to farm Standing — but Standing never derives from custody, so there is nothing to farm |
| **Sybil Custodians** — many identities, one disk | Bond scales with committed *bytes*, not identities, so splitting changes nothing; proof-of-capacity at registration; simultaneous challenge burst saturates a shared disk and blows the deadline; failure domains key on (ASN, subnet, geo cell) so co-located identities collapse to one domain | Resistance is **economic, not cryptographic**. A well-capitalised adversary with genuinely distributed hardware is a large Custodian, which is fine |
| **Lazy Custodian** — fetch on challenge | Deadline below cross-network fetch of 4 MiB; all 16 replicas challenged at once; leaf indices chosen after the beacon commits | A colluding pair on a 1 ms link is not lazy — it is two Custodians in one failure domain, caught by placement |
| **Bandwidth wash trading** | γ clamp (`served ≤ 30 × held`); receipts count only when signed by a principal the ACL admits against a live Version; per-pair receipt cap per window; β bounded so bandwidth cannot dominate | Colluders with real Memberships manufacture some traffic; caps bound the yield to a rounding error |
| **Withholding** — hold but refuse to serve | Unannounced retrieval audits with tier-specific SLOs; breach demotes tier and cuts the bandwidth multiplier | Cold-tier Custodians legitimately answer slowly |
| **Capacity overcommit** | Throughput-derived proof-of-capacity deadline; re-verification above 80% commitment | Fast XOF regeneration, narrowing as filler is displaced |

---

## 9. The Media Pipeline

### 9.1 Ingest and the E2EE Boundary

Transcoding requires plaintext. We do not pretend otherwise.

| Chamber `EncryptionMode` | Transcoding |
|---|---|
| `EndToEnd(MlsGroupId)` | **Client-side only.** The Node encodes the ladder locally, then runs each rendition through §3. A weak device uploads the original un-laddered. |
| `Transport` | Server-side under a time-boxed per-Object key release, recorded as `TranscodeKeyReleased { object_id, transcoder_fnid, expires_at, granted_by }` — a Domain Event on the Society's audit surface. |

There is no third option and no server-side plaintext path for `EndToEnd` content (N6). The honest cost: the most private Chambers get the worst adaptive streaming on low-powered devices. The UI says so rather than quietly downgrading encryption.

### 9.2 Codecs

| Media | Primary | Fallback | Why |
|---|---|---|---|
| Video | **AV1** (SVT-AV1 preset 6–8) | H.264 High, 720p rung only | 30–50% lower bitrate at equal quality. Egress is the mesh's dominant marginal cost (§6.4), so bitrate is money — for Custodians, for metered Citizens, and for the emission cap. Royalty-free removes a licensing surface. |
| Audio | **Opus** | AAC-LC | Best across the bitrate range, royalty-free, and the same codec the Relay uses for voice (`14`) — one decoder path, not two. |
| Image | **AVIF** | JPEG (progressive) | ~50% smaller at equal quality; alpha and HDR without a second format. |

**Honest cost.** SVT-AV1 preset 6 costs ~5–10× H.264 encode CPU; preset 8 narrows to ~2–3× at modest quality loss. AV1 decode is hardware-accelerated on 2023+ silicon but not universal, so the dual ladder costs ~**1.4×** the storage of AV1 alone. We spend it rather than exclude devices (P10, N8). The fallback is deliberately *one* rung: a device that cannot decode AV1 adapts by network, not by resolution.

**Rejected:** HEVC (patent pools untenable for a self-hostable platform), VP9 (dominated by AV1 with no licensing advantage), VVC (immature encoders, worse licensing than HEVC).

### 9.3 Ladder, Renditions, and Cost

| Rung | Resolution | fps | kbps | Codec |
|---|---|---|---|---|
| 240p | 426×240 | 30 | 145 | AV1 |
| 360p | 640×360 | 30 | 300 | AV1 |
| 480p | 854×480 | 30 | 560 | AV1 |
| 720p | 1280×720 | 30 | 1 100 | AV1 |
| 1080p | 1920×1080 | 30 | 2 000 | AV1 |
| 1440p | 2560×1440 | 60 | 4 500 | AV1 |
| 2160p | 3840×2160 | 60 | 9 000 | AV1 |
| 720p-compat | 1280×720 | 30 | 2 400 | H.264 |

```
  Ten minutes of 1080p source, ladder to 1080p + compat:
    AV1 240p..1080p = 4 105 kbps × 600 s = 307.9 MB
    H.264 720p compat = 2 400 × 600      = 180.0 MB
    Derived 487.9 MB ≈ 465 MiB, at RS(6,9) 1.5×  ≈ 698 MiB
    Primary original (8 Mbps, 600 s) 600 MB ≈ 572 MiB, at 1.6× ≈ 915 MiB
    TOTAL STORED per 10-minute upload            ≈ 1.58 GiB
```

Thumbnails: AVIF + JPEG at 160/480/1280 px long edge — usually under 64 KiB and therefore inlined into the Manifest, never reaching a Custodian.

**Adaptive streaming:** CMAF fMP4, 4-second segments, one HLS and one DASH playlist over the same segments. Rendition identity is deterministic:

```
  derived_version_id = BLAKE3( source_version_id ‖ rung ‖ codec ‖ profile_version )
```

Three consequences: re-transcode is idempotent and stores nothing new; a profile bump changes the address so ladders roll forward lazily; and a lost Derived rendition is *regenerated* by a transcode job, not repaired by a 10× read. That asymmetry is why Derived gets RS(6,9) and three domains.

---

## 10. Lifecycle

### 10.1 Versions and ACLs

`Object.versions` is append-only. An edit writes a new Version with `parent = previous`, reusing unchanged segment ciphertext (§3.2). A revert is a new Version, never a truncation.

```rust
struct Acl {
    owner:      Principal,
    grants:     Vec<AclGrant>,        // (Subject, Rights, expires_at)
    inherit:    Option<VaultPath>,
    key_policy: KeyPolicy,            // which group wraps content_key
}
enum Subject { Citizen(Fnid), Role(RoleId), Agent(Fnid), ExtensionInstall(InstallId), Public }
bitflags Rights { READ | LIST | APPEND | WRITE | SHARE | DELETE | TRANSCODE }
```

Enforcement is **intersected, never unioned**:

```
 effective(principal, object) = Acl.rights ∩ Envelope.capabilities (P4/P8)
                              ∩ Charter.role_capabilities (P1)
                              ∩ cryptographic reach (holds a wrapped content_key)
```

`Role` subjects resolve through the Charter, so an amendment re-scopes Vault access without touching an ACL. An Agent's `vault.object.read` can only narrow, never widen (`11 §2.8`).

**Revocation is forward-effective.** Removing `READ` rotates the content key and re-wraps future Versions. It does not un-read ciphertext already fetched and does not re-encrypt existing Shards — that would rewrite immutable content-addressed data. The UI says this plainly.

### 10.2 Tombstones and GC

```
 ObjectTombstoned{object_id, version_id?, reason, at}   ← Domain Event, permanent
    │ retention window (Chamber RetentionPolicy / CharterStoragePolicy, clamped)
    ▼
 MARK   coordinator publishes an unassign set → settlement for those fragments
    │   stops IMMEDIATELY (at mark, not at sweep)
    │   7-day grace — a mistaken tombstone is reversible here and only here
    ▼
 SWEEP  Custodians delete; ShardUnassigned emitted; capacity returns
```

A fragment is collectable only when no live Manifest in any Society references its hash, all referencing tombstones have passed retention, and no legal hold is set. The reference count is a Projection over Manifests, rebuildable from the Log (P6) — deliberately, because an under-counting refcount destroys data.

**Invariant V6.** Fracture and Crystallization re-reference Manifests; they never re-upload and never decrement to zero (`11 §3.1–3.2`). Fracturing a 10 TiB Society moves zero bytes.
**Invariant V7.** An Object referenced by a live Facet carries an uncollectable pin for the Facet's life. Facets are never destroyed (`11 §4`).

### 10.3 Takedown — What Is and Is Not Possible

**Implemented.** *Unresolve*: remove the Manifest from the Society's index — the Object becomes unreachable through every Fractal Node surface, and the index is the choke point we control. *Unassign*: publish fragment hashes to the coordinator's unassign set; Custodians delete and stop earning. *Key revocation*: rotate and re-wrap. *Blocklist*: fragment hashes, plus perceptual hashes for publicly-served plaintext renditions, refused at ingest and gateway.

**Not possible, stated plainly:**

- **We cannot prove global deletion.** Anyone who held the content key and fetched the fragments holds plaintext, and content addressing makes re-upload trivial. A takedown is eviction from this platform's custody and index — not erasure from the world. A product claiming otherwise is lying.
- **We cannot scan E2EE content,** and we do not build client-side scanning; it is a P8/P9 non-starter that would make N6 false. Abuse response in `EndToEnd` Chambers runs on report: the reporting Citizen's client decrypts with its own key and attaches signed evidence under its own signature. That path is real and it is the only one compatible with the guarantee.
- **A Custodian cannot comply with a content-specific order,** because it cannot identify content. Orders become shard-hash unassignment lists. A Custodian's honest answer to "do you store X?" is "I do not know, and I could not find out."
- **Jurisdiction is a Charter choice, not a platform guarantee** (§7.3), and it costs more.

### 10.4 Key Loss

Losing a content key destroys the data, by construction.

| Key class | Escrow default | Mechanism |
|---|---|---|
| Society Vault keys | **On** | Shamir M-of-N over Charter role holders; recovery is a quorum governance event, logged and announced |
| Citizen private Vault keys | **Off** | Social recovery per `12-identity-and-trust.md`, opt-in |

A Society losing its Vault is catastrophic and collective, so the default protects the collective. A Citizen's private Vault is theirs to lose; defaulting escrow on would be a custody claim we refuse to make (`02 §4`).

---

## 11. Search, Collaboration, and the Port

### 11.1 Search Without Covert Inference

Indexed sources, each opt-in per Object, VaultPath or Society, defaulting **off** except where the content was authored to be found (P9):

| Source | Default | Computed where |
|---|---|---|
| Citizen-authored title, description, tags, alt text | On | Runtime |
| Vault path, Chamber, Society taxonomy | On | Runtime |
| OCR, speech transcript, image caption, embedding | **Off** | Below |

```
  E2EE / private content                 Public / Transport-encrypted
  ──────────────────────                 ───────────────────────────
  computed on the Citizen's Node over    computed by the Runtime via the
  plaintext it already holds             Search port (Postgres FTS → Tantivy;
        ▼                                pgvector → ANN). Content is already
  index encrypted with a Vault-scoped    readable by its audience, so indexing
  index key, stored as an Object         reveals nothing new
        ▼
  queries run LOCALLY against the streamed encrypted index
```

Sizing the local path: int8-quantized 384-dimension embeddings are 384 B per Object; 100 000 Objects is a 38 MB index, living in the local store P2 already requires.

**P9 compliance as a test, not a promise.** No index entry may derive from behaviour — nothing about what a Citizen viewed, for how long, or in what order enters any index. Inference is declared, opt-in, run on the owner's device for private data, and inspectable: `GET /v1/societies/{id}/objects/{id}/index` returns exactly what was indexed and from which source. `11 §7.13` holds absolutely — search never writes `Citizen.interests`.

### 11.2 Collaboration Hooks

| Surface | Integration | Rule |
|---|---|---|
| **Chamber** (`Gallery`, `Canvas`, `Board`) | `MessageBody::Media` carries `ObjectRef{object_id, version_id}`, never bytes; posting grants the Chamber's role set `READ` | Deleting a Message tombstones the *reference*; the Object survives if anything else references it |
| **Canvas** | CRDT updates live in the Log; checkpoints written to the Vault as Versions every N ops or T seconds | Checkpoints bound replay; the Log stays the source of truth (P6) |
| **Facet** | `Facet.state` may reference Objects; transfer moves the ACL grant, not bytes | V7 — the pin is uncollectable for the Facet's life |
| **Extension** | Owns `/ext/<install_id>/` with a per-install quota; anything else needs a grant intersected with its Envelope | Capabilities `vault.object.{read,write,append,transcode}`, `vault.manifest.read` (P7) |

### 11.3 The `BlobStore` Port

```rust
#[async_trait]
pub trait BlobStore: Send + Sync {
    async fn put_fragment(&self, hash: &Multihash, bytes: Bytes) -> Result<Receipt>;
    async fn get_fragment(&self, hash: &Multihash) -> Result<Bytes>;
    async fn get_range(&self, hash: &Multihash, r: Range<u64>) -> Result<Bytes>;
    async fn stat(&self, hash: &Multihash) -> Result<Option<FragmentStat>>;
    async fn unassign(&self, hashes: &[Multihash]) -> Result<UnassignReceipt>;
    async fn placement(&self, hash: &Multihash) -> Result<Placement>;
}
```

**The trait is shaped to the weaker backend.** Note the deliberate absences: no `list()` (a mesh has no cheap enumeration), no `delete()` (mesh deletion is eventual; the trait must not promise synchrony the mesh cannot keep), no buckets or prefixes (S3 concepts that would leak a vendor model into the domain, P5). Every method takes a content hash and nothing else. Implementations: `MemBlobStore` (deterministic tests), `S3BlobStore` (Phase 1–3), `MeshBlobStore` (Phase 4), `TieredBlobStore` (the migration vehicle).

### 11.4 Phase 1–3, and the Migration

The entire §3 pipeline runs from Phase 1 — encryption, keyed CDC, RS(10,16), BLAKE3 addressing, Manifests, Merkle roots, ACLs, tombstones, GC. Only *placement* differs: all 16 fragments go to an S3-compatible store, spread across regions where the vendor allows. Attestations run degraded — the coordinator challenges itself against the store, emitting real `ShardAttested` events marked `SelfCustody` — so the settlement path is exercised and observable for two phases before it ever emits Fraction. `ff.economy.storage_settlement` stays off.

This matters more than it looks. The hard part of Phase 4 is not the mesh; it is discovering in production that the Manifest schema, the repair loop, or the GC refcount was wrong. Running the whole stack against a boring backend for three phases surfaces those defects while they are cheap.

| Step | State | Reads | Writes | Settlement |
|---|---|---|---|---|
| 4.0 | Coordinator up; `TieredBlobStore` | S3 authoritative | Both; mesh shadow-verified byte-for-byte | Off |
| 4.1 | Mesh trusted for regenerable data | Mesh-first for **Derived** | Both | Off |
| 4.2 | Mesh trusted for originals | Mesh-first for **Primary** | Both | Shadow-computed, not posted |
| 4.3 | Mesh authoritative | Mesh; S3 is 1 of the 16 | Mesh | **On** |
| 5.0 | S3 optional | Mesh | Mesh | On; S3 as a Charter-purchasable insured tier |

**Zero domain-code change, with the test that proves it:** swap `S3BlobStore` for `MeshBlobStore` in the composition root. `fractal-domain-vault` must compile unchanged and its full property-test suite must pass unchanged. If it does not, the abstraction failed and the migration stops (P5 falsification test).

---

## 12. Failure Modes and Cost

| Failure | Detection | Automatic response | Residual risk |
|---|---|---|---|
| Custodian offline < 1 h | Missed heartbeat | Serve from the other 15 | Latency blip |
| Custodian permanently lost | 3 missed Attestations | `ReplicaLost` → repair (10× read) | Repair bandwidth |
| Correlated domain loss | ≥ 2 losses sharing a domain in one epoch | Urgent repair; domain quarantined from placement | 3-of-8 domains is the ceiling (§6.2) |
| Bit rot | Hash mismatch on read or Attestation | `ReplicaCorrupt`, slash, reassign | Detected on access outside the sample, not proactively |
| Manifest lost, fragments intact | Object unresolvable | Rebuild by replaying `ObjectStored` from the Log | None — this is why P6 matters |
| Content key lost | Decrypt failure | Society escrow recovery (§10.4) | **Unrecoverable** without escrow |
| Coordinator outage | Health check | Reads/writes continue on cached placement; no new assignment, no settlement | Repair pauses; durability degrades with outage length |
| Repair storm | Queue depth | Throttle to committed egress; prioritise by fragment count | Extended window raises loss probability superlinearly |
| Transcoder backlog | Queue age | Serve the original un-laddered — degrade, never fail | Poor mobile playback meanwhile |
| Object-store outage (Ph. 1–3) | Error rate | Serve from the local Node replica (P2); queue writes in the outbox | New uploads stall |
| Malicious Custodian, wrong bytes | Verified streaming fails at a 1 KiB leaf | Slash, reassign, refetch a peer fragment | Refetch latency only |
| Society over quota | Settlement projection | Writes refused with a typed error; reads and exports never blocked | None — never hold data hostage |

| Backend | $/TiB-month | Egress | Note |
|---|---|---|---|
| S3 Standard | 23.55 | $0.09/GB | Egress makes a media product structurally unprofitable |
| Cloudflare R2 | 15.36 | $0 | Phase 1–3 default, for exactly that reason |
| Backblaze B2 | 6.14 | free to partners | Cold-tier candidate |
| Custodian marginal | **~0.30** | metered | HDD $12/TB ÷ 60 mo = $0.20; 0.625 W/TB → 0.456 kWh × $0.15 = $0.068 |

```
  True mesh cost per logical TiB per month:
    custody  1.6 × $0.30                                  = $0.48
    repair   0.78 TiB read / logical TiB / mo (§6.4) × $2/TiB     = $1.56
    coordinator, audit, settlement (amortised)             ≈ $0.10
    TOTAL ≈ $2.04  vs R2 $15.36  →  ~7.5×, not the 50×
    a naive capital comparison suggests.
```

That 7.5× is the honest thesis, and it is contingent: hotter repair traffic or a recruitment premium closes the gap. The `BlobStore` port exists so that outcome is a business decision rather than an architectural crisis.

---

## 13. Rejected Alternatives

| Alternative | Why rejected | Kept |
|---|---|---|
| **IPFS / libp2p** | Pinning is a promise, not a contract — no durability guarantee, no slashing, no payment. Public CIDs publish the *existence* of content (P9). DHT latency is irreconcilable with P10. | Multihash, Merkle DAG addressing |
| **Filecoin** | Settlement in FIL contradicts P11/P12 — emission must be ours to bound. Deal latency in hours. 32/64 GiB sectors are the wrong granularity by five orders of magnitude for a 200 KiB image. | Their PoRep/PoSt research is better than ours; §7.5 borrows its shape |
| **Storj** | The nearest sibling — erasure coding, node reputation, coordinated placement. Rejected as a *foundation* because the coordinator is a party we do not control, payment is not in Fraction, and the trust model is theirs rather than the Charter's. | Coding parameters and audit sampling; we will likely ship an adapter |
| **Arweave** | Pay-once-store-forever is wrong for a platform that must honour retention, tombstones and takedown. Permanence as a feature conflicts with P9's deletability directive. | A candidate `Chain` adapter for Anchors, which should be permanent and are tiny |
| **Plain S3 forever** | One vendor can end a Society, so "sovereign" becomes false; egress pricing makes media structurally unprofitable; and there is no honest way to pay a Citizen for contribution when the contribution is a credit card (P12). | It is the right Phase 1–3 answer and we ship it. If mesh operating cost never beats R2, keeping it is the correct outcome |
| **Self-hosted only, no mesh** | A single enthusiast NAS delivers about two nines. A sovereignty story that loses data is worse than none. | Self-hosting stays first-class *inside* the mesh: a Node may custody its own Society under the same floors |

---

## 14. Invariants

1. **V1** No Shard boundary or length is derivable from plaintext without the Vault key.
2. **V2** Every byte delivered to a decoder is authenticated against a hash from an independently resolved Manifest.
3. **V3** `effective = max(platform_floor, charter_policy)`, field by field. A Charter never lowers a floor.
4. **V4** `emission(window) ≤ cap_window`, always, no discretionary override.
5. **V5** Sink rate per byte-hour ≥ σ × Source rate per byte-hour, with **σ ≥ 1.25 as a floor asserted by the settlement code**, not a constant the rates are expected to equal. At the reconciled rates (`61 X6`) the observed σ is 2.23; it is published in the supply dashboard as a health figure.
6. **V6** Fracture and Crystallization re-reference Manifests; they never re-upload bytes.
7. **V7** An Object referenced by a live Facet is uncollectable.
8. **V8** A Custodian is paid only against a valid, unexpired, non-replayed Attestation.
9. **V9** `Manifest.merkle_root` verifies on every reconstruction; failure emits `ReplicaCorrupt` and slashes.
10. **V10** No index entry derives from behavioural observation.
11. **V11** Every Shard resolves to at least one `society_id` (P1); an orphan is a GC candidate, never a resident.
12. **V12** No domain crate names S3, a Custodian implementation, or a codec library.

Each becomes a property test under `40-engineering-standards.md`.

---

## 15. What Would Make Us Change This

- **Repair bandwidth becomes the binding cost** before Phase 5 → adopt a locally-repairable code to halve repair reads, as an ADR with the full correctness argument.
- **Custodian recruitment fails at the modelled price** → keep S3 authoritative past 4.3, publish that the mesh is a supplement rather than the substrate, and do not pretend otherwise.
- **Observed correlated failures exceed 3-of-8 domains** → move to RS(12,20) over 10 domains and accept 1.67× overhead.
- **Client-side transcoding proves unusable on median mobile hardware** → offer server-side transcoding for `EndToEnd` content only under an explicit, revocable, per-Object key release with a governance-visible event, and publish the reduced guarantee. Never silently.
- **Sampling at s = 460 misses a real withholding attack** → scale *s* with bond at risk and shorten the epoch on Probation, before touching the challenge construction.
