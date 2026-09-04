# 60 — Self-Critique

> **Prerequisites:** the entire blueprint. This chapter is read last and acted on first.
> **Governs:** the standing list of known structural weaknesses, the scaling limits and the numbers at which they bind, the cross-chapter contradictions that must be reconciled, the unstated assumptions the design rests on, the prioritized remediation set, and the evidence that would justify abandoning this architecture rather than patching it.

---

## 1. Method

### 1.1 What was evaluated

The Canon (`00`, `01`, `02`), then `10`, `11`, `50`, then `12`, `13`, `14`, `15`, `16`, `17`, `18`, `19`, `20`, `21`, `32` in full; `30`, `31`, `33`, `34`, `40`, `41`, `42` at section granularity with full reads of every section carrying numbers, phase claims or invariants; ADRs 0003, 0004, 0005, 0006, 0009, 0010, 0011, 0014 in full.

Four techniques produced the findings.

1. **Invariant tracing.** Every numbered invariant (`11 §7`, `12 §13`, `13 §14`, `15 §14.1`, `16 §19`, `17 §16`, `18 §13`, `19 §16`, `20 §16`, `21 §16`) was checked against the chapters that must *satisfy* it, not the chapter that declares it. Several are declared in one place and contradicted in another.
2. **Numeric reconciliation.** Every figure appearing in two chapters was compared: storage rates, replication factors, emission shares, fee splits, performance budgets, binary sizes, Level gates, phase assignments. This is where most real defects are. Prose can be vague and survive; numbers cannot.
3. **Arithmetic on the stated models.** Durability, repair bandwidth, emission schedule, sink coverage, attestation sampling, XP curves, fan-out cost and Postgres throughput were re-run. Several models are correct in isolation and inconsistent with the model they feed.
4. **Failure-path construction.** For each subsystem, a concrete sequence of inputs and state producing a bad outcome, then a check of whether the design prevents it, detects it, or merely disapproves of it.

### 1.2 What was deliberately not evaluated

Product-market fit (`50 §5` already owns the stop condition). Legal correctness — every chapter touching law flags counsel, and I treat those flags as unresolved risk rather than error. Aesthetic judgement; `32` and `33` are internally coherent and are critiqued only where they constrain engineering or commerce. Code quality of an implementation that does not exist. Cryptographic primitives — MLS, BLAKE3, Reed-Solomon, Shamir, XChaCha20-Poly1305 and Ed25519 are taken as sound; only their composition and operation are examined. And I do not relitigate a rejected alternative unless the arithmetic that justified rejecting it is wrong.

### 1.3 The standard applied

A finding names the chapter and section, describes a failure with specific inputs and state, and proposes a remedy with a cost. "This might be hard" is not a finding. Where I say a number is wrong, I show the arithmetic.

---

## 2. The Strongest Things About This Architecture

A critique that cannot identify the load-bearing good decisions is not calibrated.

1. **Per-Society ordering with no global consensus** (`10 §4`, ADR-0004). The best decision in the blueprint. It removes consensus from the write path, makes Fracture a single-partition operation, makes export a file rather than a negotiation, and makes test isolation free. The cost is stated honestly and the escape hatch — a narrow global sequencer, never a global log — is pre-designed.
2. **The Ledger is the event log** (`16 §3`). Making `PostingRecorded` a domain event and the Posting table a projection eliminates the dual-write problem rather than treating it. The reconciler-plus-freeze policy in `16 §4.5` is the correct posture: a ledger that keeps accepting writes while known-inconsistent converts a bug into a loss.
3. **Attenuation-only grants** (`12 §7.2`, `15 §4.2`, `20 §5`). Computing the grant as an intersection with the grantor's own set, in the constructor, makes escalation unrepresentable rather than policed. Reusing identical Envelope machinery for Agents and Extension Installs — with two phases of adversarial exposure before third-party code runs — is sophisticated sequencing.
4. **Absent imports rather than denied calls** (`15 §7`, `20 §4.1`). "An ungranted capability is not denied, it is unnameable" is the correct answer to both prompt injection and API-confusion sandbox escape. It converts an unbounded problem into a bounded one.
5. **Encryption before chunking, with the dedupe loss priced** (`13 §3.1`). The confirmation-of-file attack is real, the keyed CDC gear table closes the boundary-fingerprint channel, and the residual leak is stated. The 6%-of-gross-capacity cost is computed, not asserted.
6. **Sink-first Class A settlement with a forfeited ceiling** (`17 §3.3`, `17 §5.1`). Bounded above by a published curve, bounded below by verified work, never banked. `17 §5.4`'s observation — once the ceiling binds, adversarial volume can only steal share and cannot inflate supply — is the sharpest economic insight in the corpus.
7. **Deterministic simulation as the primary correctness gate, and `sim-mutation` in particular** (`40 §7.5`, ADR-0014). Weekly inversion of each invariant to prove the harness can still fail is the test of the test. Very few projects do this, and it is what makes high-throughput agent-written code safe to accept.
8. **Honest statements of what cannot be done** — `13 §10.3`, `12 §10.3`, `14 §4.6`, `21 §7.2`, `16 §13`, `19 §7`. This posture is worth more than any single technical decision, because it is what makes the rest credible.

---

## 3. Structural Weaknesses

Ranked by probability × cost when it occurs × cost of fixing it late.

### 3.1 W1 — There is no authoritative phase table, and eight chapters disagree

**Implicated:** `50`, `41 §5`, `30 §4.2`, `17 §3.2`, `19 §17`, `20 §2`, `21 §14`, `16 §20`, `02 §5`.

An agent assigned M2.6 loads `13` and `21`. `21 §14` places `ContributionReceipt` rendering "(XP and S4 only)" in Phase 2, justified by "`17` S4 is Phase 2". `17 §3.2` does place S4 and S6 at Phase 2. `50 PH2`'s complexity budget says **0 Sources**. The agent implements a Source, meets its acceptance criteria, and silently breaches both `02 §5` and `50`. Nothing catches it, because `cargo xtask budgets` checks against a table authored from one of these chapters.

Six further divergences: `41 §5.3` puts `fractal-domain-progression` at Phase 2 (`50 M1.6` ships it in PH1), `fractal-domain-agent` and `-governance` at Phase 2 (`50` PH3); `41 §5.5` puts `fractal-adapter-ffmpeg` at Phase 4 (`50 M2.7` ships transcoding in PH2 and calls it the first extraction) and `fractal-adapter-rail-internal` at Phase 2 (`16 §9` says Phase 1); `30 §4.2` puts Agent and Vault families at Phase 1 (`50`: PH3 and PH2); `19 §17` puts listings and the review pipeline at "4–5" (`50 PH6`); `20 §2` puts `workflow` and `automation-pack` at Phase 4 (`50 M3.5`/`M3.7` ship both in PH3).

`02 §6` requires an agent to answer "which phase does the roadmap place it in?" — but there are eight roadmaps, and none of the other seven phase columns is generated from `50`.

**Remedy.** `docs/phases.toml` as the single source: per phase, its milestones, resource families, Sources, crates, services and dependency count. `50` renders from it; every other phase column becomes generated; `cargo xtask lint-phases` fails the build on a hand-authored claim — the mechanism `40 §4` already uses for terminology.

**Cost.** 3–5 days: one xtask, one TOML, one CI job, a mechanical pass over eight chapters. **Buys:** removes an entire class of agent misdirection, and makes the complexity budget mechanically enforceable.

---

### 3.2 W2 — Storage economics do not close, and the two chapters that own them differ by 6× to 18×

**Implicated:** `13 §8.2`, `13 §6.1`, `13 §14 V5`, `17 §1.2`, `17 §3.2`, `17 §3.3`, `17 §4 K8`, `18 §5.1`, `18 §5.2`, `50 PH2`.

**Rates.** `17 §1.2` fixes the anchor at **1 FRC ≡ 1 GB-month** and K8 charges **1.00 FRC/GB-month**. `13 §8.2` derives **46.8 FRC/TiB-month** paid to Custodians and **58.5 FRC/TiB-month** charged to Societies:

```
  13 §8.2 Society charge:   58.5 FRC/TiB-mo  =  0.0571 FRC/GB-mo
  17 K8    Society charge:                       1.00   FRC/GB-mo     -> 17.5x apart
  13 §8.2 Custodian pay:    46.8 FRC/TiB-mo  =  0.0457 FRC/GB-mo
  17 S1    Custodian pay:                        0.28   FRC/replica-GB-mo -> 6.1x apart
```

Under `13`'s numbers a gigabyte-month costs 0.057 FRC, so the definitional anchor — by `17 §1.2`'s own statement the single most important economic decision in the document — is violated 17.5× inside the chapter that implements it.

**Replication.** `13 §6.1` fixes RS(10,16) at **1.60×**. `17 §3.3` multiplies logical bytes by **3**:

```
  as written:      470,000 GB x 3.0 = 1,410,000 replica-GB x 0.28 = 394,800 FRC/mo
  with RS(10,16):  470,000 GB x 1.6 =   752,000 replica-GB x 0.28 = 210,560 FRC/mo
```

The model overstates the Custodian bill by 1.87×, propagating into every S1-derived figure in `17 §5.2`.

**Share.** `13 §8.2` states an annual bound of 1,825,000 FRC and asserts storage-and-bandwidth claims "at most **20%** of the platform's annual emission ceiling". `17 §3.2` gives S1 22% and S2 10% — 32% — and at `17 §5.1`'s Y1 ceiling of 200 M FRC, 22% is **44,000,000 FRC/yr**: 24× `13`'s bound.

**Free quota — the one that actually breaks the model.** `17 §3.3` models a flat 5 GB/Citizen and 25 GB/Society. `18 §5.1` grants 1 GB at L0 rising to 250 GB at L12; `18 §5.2` grants 10 GB at SL0, 100 GB at SL1, 1 TB at SL2, 10 TB at SL3 and **"unmetered Vault (cost-settled)" at SL4** — which also directly contradicts `50 PH2`'s mitigation, "no unmetered storage, ever". Re-running with `18`'s grants over 4,000 Societies at 60/30/9/1% across SL0–SL3:

```
  2400x10 + 1200x100 + 360x1000 + 40x10000  =  904,000 GB free
  17 §3.3 assumed                           =  100,000 GB     -> 9.0x understated
```

Custody is paid on every stored byte; the Settlement Pool collects only above quota. Since the free grant scales with population *and* with monotonic Society Level (`18 I-9`), storage emission does not converge to zero as `17 §3.3` claims — it grows.

**Why nothing catches it.** `17 §12`'s `econ-sim` asserts I-E1 through I-E10, none of which is "the storage rate in `17` equals the rate in `13`", "the modelled replication factor equals `13 §6.1`", or "modelled free quota equals `18`'s grants".

**Remedy.** One machine-readable rate table owned by `17` at `economy/rates.toml`, from which `13 §8.2`'s worked example and `18`'s quota tables are generated. Add **I-E11** (replication factor matches `13 §6.1`), **I-E12** (modelled quota matches `18`'s grants over the actual Level distribution), **I-E13** (S1+S2 share within `13 §8.2`'s claimed ceiling). Delete SL4's "unmetered" grant per `50 PH2`.

**Cost.** One week of modelling plus three simulation assertions. **Buys:** the PH4 exit gate becomes a test of the system rather than of a model that does not describe it.

---

### 3.3 W3 — The emission schedule is denominated in calendar years; the roadmap is not

**Implicated:** `17 §5.1 R2`, `17 §5.2`, `17 §5.3`, `50 §1`.

`R2` fixes `B(n) = 200,000,000 × 0.80^(n-1)` with *n* a year, and `R3` forfeits unclaimed budget permanently. `17 §5.2` shows Y1 emission of 3.3 M because "only S4–S7 live (Phase 2–3)".

`50` sequences PH0–PH3 at 33 weeks and PH4 — the first phase in which any resource Source emits — at weeks 47–61, on estimates `50 §1` calls "estimates for sequencing, not commitments". At the realistic multiple of §3.11, PH4 completes near month 35. Y1–Y3 have then elapsed with schedule budgets of 200 M + 160 M + 128 M = **488 M FRC, 48.8% of the lifetime cap**, forfeited against almost no claim because the Sources did not exist.

The consequence is not less FRC; it is that `17 §5.2` is invalidated. Supply never reaches the modelled 109 M peak, the π collapse in `17 §5.3` never occurs on schedule, and the M1–M5 target bands in `17 §1.3` — the definition of "the economy is working" — are calibrated to a trajectory that cannot happen. `50 §5` has no cancel/reorder row for it.

**Remedy.** Redefine *n* as **epochs since the first Source was enabled in production**. This preserves the decay shape, the forfeiture rule and the lifetime cap while making the model robust to the single most likely event in the project. Record the shape as Tier 0 and the origin as Tier 1a, set once by event.

**Cost.** Two days. **Buys:** an economic model that survives a late launch rather than being falsified by it.

---

### 3.4 W4 — Postgres is the event store, and the stated scaling remedy does not add write capacity

**Implicated:** `10 §11`, `10 §12`, `16 §4.1`, `16 §4.4`, ADR-0006 §3 and §9, `40 §13.1`.

Every command takes one Postgres transaction that appends an event, updates projections and — on the money path — takes ordered `FOR UPDATE` locks. A single primary with `synchronous_commit = on` and group commit on NVMe sustains roughly **5,000–15,000 small write transactions/second**. With messages, reactions, read marks, presence and settlement, the platform reaches that band at a few tens of thousands of active Citizens.

`10 §12` prescribes: "**Partition by `society_id` first**; move the event log to FoundationDB or per-society segment files second." ADR-0006 §9 triggers on "p99 write latency on the posting path exceeds 50 ms **after partitioning by `society_id`**."

**Partitioning does not do what it is described as doing.** Declarative partitioning reduces index depth, vacuum cost and improves locality. It adds no write throughput, because every partition shares one instance, one WAL and one writer. ADR-0006 §3 states the truth — "A single primary is a single write bottleneck. Read replicas help projections; they do not help the posting path" — and §9 then prescribes partitioning as the remedy for exactly that bottleneck.

The real remedy is **sharding across primaries**, a different and much larger project: it changes `10 §9`'s topology, `40 §11`'s backup story, and the one-transaction property ADR-0006 exists to preserve. The team will execute the partitioning project, measure no throughput improvement, and discover mid-incident that the fix is a topology change nobody designed.

**Remedy.** Rewrite `10 §12` and ADR-0006 §9 to name the true ladder: (1) partition for vacuum, index depth and locality — buys latency, not throughput; (2) move projections to replicas and CDC consumers — buys read capacity; (3) **shard Societies across N primaries by consistent hash on `society_id`**, routed at the composition root, with no transaction spanning shards; (4) then consider FoundationDB. Prove step 3 by running the suite against a two-primary composition root in PH2 — the discipline `13 §11.4` already applies to `BlobStore`.

**Cost.** Two days of guidance, ~2 engineer-weeks in PH2. **Buys:** a planned migration instead of an emergency one.

---

### 3.5 W5 — Projection rebuild time has no bound and no SLO

**Implicated:** `11 §7.10`, `16 §4.3`, `40 §11.5`, `40 §9.4`.

`11 §7.10` requires every projection to be reproducible by replaying from zero; `40 §11.5` names replay as "the ultimate mechanism". `16 §4.3` bounds *ledger* rebuild with checkpoints every 4,096 Postings. **No other projection has a checkpoint.**

A Society with 500 M events — 100,000 members over eight years at ~1.7 events/member/day — holds ~500 GB in one log at a 1 KB mean envelope. Replay is single-threaded per Society because the fold is order-dependent and `16 §5` forbids non-deterministic iteration. At ~30,000 events/second including projection writes:

```
  500,000,000 / 30,000 = 16,667 s = 4.6 hours per projection pass
  8 projections, shared pass 4.6 h; independent passes ~37 h
```

A projection bug ships and is detected at 09:00. The remedy per `40 §11.5` is "drop and rebuild". The Society is unreadable for five to thirty-seven hours. `40 §9.4` sets projection *freshness* lag at 2 s with a 99.5% target and sets no rebuild SLO at all — while `40 §9.6` requires every alert to have a runbook.

The blueprint treats replay-from-zero as a guarantee and never as a cost. `fractal-sim` asserts the invariant on histories of at most 100,000 steps, four orders of magnitude below the case that matters.

**Remedy.** (1) Generalize `16 §4.3`'s checkpoints to every projection, so rebuild is snapshot-load plus tail-replay. (2) State a rebuild SLO in `40 §9.4` — e.g. any single projection for any Society rebuilds in ≤ 15 minutes at p99 — as a phase-gate criterion with a large-Society fixture in `41 §15`. (3) Amend `11 §7.10` to "reproducible from the most recent verified checkpoint, and from zero within the published rebuild SLO", retaining full replay as a periodic audit rather than the recovery path.

**Cost.** ~3 engineer-weeks in PH2 plus a fixture. **Buys:** removes a multi-hour outage from the top of the incident distribution and makes P6's falsification test executable at production scale.

---

### 3.6 W6 — The single `EmissionAccount` row is a global serialization point the settlement design maximizes

**Implicated:** `11 §2.6`, `16 §4.2`, `16 §4.4`, `13 §8.2`, `40 §9.4`.

`16 §4.4` identifies it: "The `EmissionAccount` is the one global hot row. Emission is therefore batched by the settlement run, not posted per reward: one Posting group per settlement window **per Society**." Per-Society batching still funnels every Society into one row. `13 §8.2` then fixes the window at 24 hours **closing 00:00 UTC**, converting distributed load into a synchronized spike:

```
  100,000 Societies x 5 ms serialized on one row = 500 s  = 8.3 minutes
  1,000,000 Societies                            = 83 minutes
```

Throughout which the row is locked, other emission paths queue, `16 §4.5`'s reconciler cannot get a consistent read, and `40 §9.4`'s ledger-settle p99 < 1 s SLO (99.9% target) is violated daily. The spike coincides with the attestation epoch and the repair-bounty draw from the same cap, so the highest-contention moment is also the most safety-critical arithmetic.

`13 §7.5` already applies the right technique elsewhere — attestation cadence is "offset per Custodian by `BLAKE3(fnid) mod 86400`" — and does not apply it here.

**Remedy.** (1) **Shard `EmissionAccount` into K sub-accounts**, each pre-allocated a deterministic slice of the epoch budget. Total supply becomes `-Σ EmissionAccount[i].balance` — still exact, still directly queryable, no longer one row; `16 §19` invariant 2 becomes "exactly K accounts may be negative." (2) **Offset settlement close** by `BLAKE3(society_id) mod 86400`. The cap is per-epoch, not per-instant, so nothing changes economically.

**Cost.** One day of design, ~1 engineer-week, one property test. **Buys:** removes the only unavoidable global write contention in a design whose central claim is that it has none.

---

### 3.7 W7 — The history archive key silently becomes the weakest link in E2EE

**Implicated:** `14 §4.5`, `12 §6.1`, `12 §6.2`, `12 §6.3`, `34 §12.2`, `50 M2.10`.

`14 §4.5` defaults the Vault-backed archive **on for DMs**, encrypted under a Citizen-held `HistoryKey` obtained "device-to-device or via social recovery". `12 §6.3` lists Vault objects "whose content keys are wrapped to the recovery key" as recoverable and E2EE history as not. The DM archive is simultaneously both, and no chapter says which.

If `HistoryKey` is recoverable, then `t` guardians can, after 72 hours, read the Citizen's entire DM history. `12 §6.2`'s protections — wallet freeze, Envelope suspension, unilateral device veto — depend on an active device, and the population that most needs recovery is the one with none, and therefore no veto. `14 §4.5`'s disclosure — *"If that key is stolen, saved history can be read"* — does not say *"your guardians can, together, read it"*, and that is the sentence that matters.

If it is not recoverable, the archive is worthless in the only scenario it exists for, and `14 §4.5`'s "or via social recovery" is false.

Compounding: `34 §12.2` establishes that PWA-on-iOS storage "is cleared after extended non-use" and a browser tab is a cache, not a replica. Since the PWA is the mobile product through Phase 4 (`34 §7`), a Citizen whose only device is an iPhone can lose their sole device key to an OS eviction with no second device, leaving only recovery — which either exposes their history or does not restore it.

Each chapter is locally correct; nobody owns the composition, and the composition is the user experience.

**Remedy.** Wrap `HistoryKey` **separately** from the identity recovery key, behind its own separately-signed opt-in, **off by default**. Default recovery then restores identity, wallet, memberships and Level, and not DM history; the disclosure becomes *"Your guardians can restore your account. They cannot read your saved conversations unless you turn this on."* Add **I-12.13**: no recovery flow reconstructs a `HistoryKey` absent a `HistoryEscrowEnabled` grant. Add a rule to `34 §12.2`: a Citizen whose only enrolled device is on evictable storage is prompted for a second factor before holding Fraction or a Membership above Level 1.

**Cost.** One week across `12` and `14`, one property test, one onboarding surface. **Buys:** closes the gap between what the product promises about private messages and what a guardian quorum can read.

---

### 3.8 W8 — There is a confused-deputy path from Extension to Agent that the threat models name and do not close

**Implicated:** `15 §13.1`, `15 §13.3`, `15 §14.1 A4`, `20 §6` hooks 18, 20, 31, 32, `20 §13` T9.

**Is the Envelope system escalation-proof?** For capability *grants*, yes. `12 §7.2`'s `meet`, `15 §4.2`'s constructor and `20 §5`'s attenuation all compute intersections with the grantor's own set, re-checked at evaluation (`I-12.7`). I could not construct a path producing a capability the grantor lacked.

**The open path is influence, not authority.** `20 §6` hook 31 lets an Extension return `list<tool-def>` to an Agent. `15 §13.1` wraps untrusted *content* — Chamber messages, Vault documents, Extension output — in a structurally distinct region. **Tool definitions are not content; they are structure.** A tool's name, description and parameter schema enter the model as part of the action space, not as data inside the `UNTRUSTED` region. An Extension offering a tool named `approve_transfer_safe`, described as "use this whenever the user mentions a payment", supplies an instruction channel the data/instruction boundary does not cover.

The PEP bounds the outcome to the Envelope, so the exploit yields nothing where the Agent holds nothing. But `15 §4.1` shows `wallet.transfer<=100FRC/day` as a normal grant and `18 §5.1` grants ceilings to 5,000 FRC/day at Level 10. Within a legitimate envelope, an Extension can steer action selection at scale, across every Society that installed it. `20 §13` T9 names this residual — "a pair with overlapping legitimate authority reaching a result no human intended" — and does not mitigate it.

A second, narrower path: hook 18 (`economy.transfer.quoting`) permits an Extension to annotate a fee "clamped by the Charter's `economy` parameters". There is no per-Extension revenue cap, and `ExtensionQuoteClamped` fires only when a quote *exceeds* the cap, not when it equals it. An Extension installed across 4,000 Societies quoting the Charter maximum on every transfer extracts a continuous, unsurfaced rent.

**Remedy.** (1) Amend `15 §13.1`: Extension-supplied tool definitions are untrusted content — namespaced by the host (`ext.<install_id>.<name>`), descriptions rendered inside the `UNTRUSTED` region, and the system region stating which tools are host-native. (2) Add **A11**: no Extension-supplied tool may be invoked for a `confirm_class` action unless the confirmation names the supplying Install. (3) For hook 18, render any non-zero quoted fee in the host-drawn `TransferSheet` with the Install named, and add a per-Install fee-revenue ceiling to `20 §11`.

**Cost.** One week across `15` and `20`, plus extension of `50 PH3` AC-3's injection suite. **Buys:** closes the only influence channel the architecture's own threat tables identify and leave open.

---

### 3.9 W9 — The cross-Society read path is named as a cost three times and designed zero times

**Implicated:** `10 §4`, ADR-0004 §3 and §9, `14 §2`, `30 §4.3` #41, `13 §10.2`, `10 §3`, `41 §5`.

`10 §4` names the cost: "cross-Society operations (Federation, global search, marketplace-wide statistics, **a Citizen's unified inbox**) are harder." ADR-0004's review trigger (b) is "p99 fan-out read for a Citizen's unified inbox exceeds 400 ms at the Phase 5 population."

Now look for the design. `10 §3`'s fourteen boundaries include no global projection. `41 §5.3`'s thirteen domain crates include none. `fractal-app-projection` is the projection *runner*, not the owner of a cross-partition read model. `14 §2` says a Citizen in 400 Societies "subscribes to `Self` plus what is on screen" — but nothing specifies what populates `Scope::Self_`, where it lives, or what it guarantees. `30 §4.3` #41 is `GET /v1/search?q=…` with no statement of how it queries N partitions.

| Projection | Required by | Owner | Consistency model |
|---|---|---|---|
| Citizen unified inbox | `14 §2`, `14 §11` | **none** | **unspecified** |
| Global search | `30 §4.3` #41, `13 §11.1`, `19 §8` | **none** | **unspecified** |
| Shard reference count for GC | `13 §10.2` | "a Projection over Manifests" | **unspecified, and cross-Society after Fracture (`13 V6`)** |

The third is dangerous. `13 §10.2` requires that "a fragment is collectable only when no live Manifest **in any Society** references its hash" and warns that "an under-counting refcount destroys data." After a Fracture, `13 V6` guarantees both children reference the same Shards. So a cross-partition projection whose under-count is data loss has no owner, no boundary, no crate and no consistency specification.

"Stated as a cost" was allowed to substitute for "designed as a subsystem", three times, in three chapters, each reasonably assuming another owned it.

**Remedy.** Boundary **S15 — Global Projections** in `10 §3` and crate `fractal-app-global` in `41 §5.4`, owning three read models with stated guarantees: inbox (eventually consistent, bounded staleness, rebuilt from the union of that Citizen's Societies' logs); search (eventually consistent, index-only, no authority); Shard refcount (**monotone-safe** — over-count permitted, under-count forbidden, with GC requiring positive confirmation from every referencing Society's log at its current `seq` rather than the absence of a reference). This consumes one PH1 resource-family slot, which is exactly the trade `02 §8` demands; the thing that leaves is `30 §4.2`'s Discovery, moving to Phase 4.

**Cost.** ~3 weeks design, ~4 weeks implementation across PH1–PH2, one budget slot. **Buys:** the most-used surface in the product acquires an owner before it is invented under deadline, and the GC refcount acquires a correctness posture before it can destroy data.

---

### 3.10 W10 — Fracture divides ownership but not cryptographic reach

**Implicated:** `11 §3.2`, `13 §10.1`, `13 §14 V6`, `14 §9`, `50 PH5`.

A Society of 800 fractures acrimoniously into A and B. The split assigns `/finance/**` to A with disposition `move`. Execution proceeds: "Vault: Manifests re-referenced, **NOT re-uploaded**. Shards gain a second owning society" (`13 V6`).

What can a member of B read? `13 §10.1` specifies that a Manifest's `content_key` is wrapped to the ACL's key holders, and that **revocation is forward-effective**: removing `READ` "rotates the content key and re-wraps *future* Versions… does not re-encrypt existing Shards."

`11 §3.2` contains **no key-rotation step**. Every member of B who held a wrapped `content_key` for `/finance/**` still holds it, and the Shards stay live because A still references them. A member of B with a local replica can decrypt A's finance archive indefinitely. The five Fracture invariants preserve total Fraction, total Facets, total members, readable history and resumability. **Confidentiality is not among them.**

`14 §9` handles the equivalent for Chambers honestly — history a member could decrypt stays decryptable. But the Vault case differs in one crucial respect: the *future* is also unprotected, because the same wrapped key decrypts every existing Version and no rotation happens at the fracture point. `11 §3.2` was written from the ledger's perspective, where invariants are conservation properties; `13 §10.1` from the ACL's, where forward-effectiveness is correct. Neither asks what a Fracture means for the key hierarchy, and `50 PH5` AC-1 tests preservation over 100,000 generated splits without testing confidentiality on any.

**Remedy.** Add a mandatory execution step between log sealing and child genesis: **for every Vault path assigned to exactly one child, rotate the content key and re-wrap to that child's key holders only; emit `VaultKeyRotatedOnFracture`.** Existing Shards stay as they are — unavoidable — but every subsequent Version is unreadable to the other child, with the residual stated in `13 §10.1`'s register. Add invariant 16 to `11 §7`: after a Fracture, no principal assigned solely to child B holds a live wrapped key for a path assigned solely to child A. Add it to `50 PH5` AC-1.

**Cost.** ~2 engineer-weeks in PH5. Re-wrap cost is proportional to key-holders, not bytes, so `13 V6`'s "zero bytes moved" is preserved. **Buys:** the signature operation stops leaking the thing people fracture over.

---

### 3.11 W11 — The scope is not buildable in the stated timeframes; the realistic multiple is 2.5–4×

**Implicated:** `50`, `40 §7.5`, `32 §5`, `42 §2.2`, `40 §8.3`.

`50 §1` gives PH0–PH5 as 3 + 10 + 10 + 10 + 14 + 16 = **63 weeks** for "a small human team (1–3) directing a fleet of AI coding agents, with one human approving Milestones and releases".

M1.7 alone is "the 40 Phase-1 components, **all nine required artifacts each**" — 360 artifacts. Alongside it in the same ten weeks: FNID derivation and the device chain with fork detection; passkeys with a device-code fallback; Shamir recovery with delay-and-notify; Charter evaluation; the Relay with its full backpressure ladder and replay ring; a double-entry ledger with ordered locking, running hashes, checkpoints and a continuous reconciler; XP/Level/Trust/Standing with 19 sources; full CLI parity with a generated command tree and a TUI; OpenTelemetry with SLOs, dashboards and a runbook per alert; and "all fifteen invariants pass under 500,000 simulated histories". For scale, `40 §7.5` and ADR-0014 both cost the simulation harness alone at "roughly **6 engineer-weeks** to first value" — 60% of a phase, for one of eleven milestones, and it is a PH0 deliverable inside a three-week phase.

Agents are fastest at work that is well-specified, locally verifiable and independently reviewable — perhaps 60% of this corpus. The remaining 40% (MLS, sync conflict semantics, Fracture execution, the repair loop, the nine-artifact discipline, a11y audits, every ADR) is judgement-dense or requires physical verification. And `42 §2.2` sets a Work Unit at ≤400 changed lines while `40 §8.3` requires **one human approval per merge with no bypass list**. At a few hundred thousand lines for PH0–PH3, that is 1,000–2,000 Work Units, each needing human review; at 20 minutes each and five productive review-hours a day, one reviewer clears fifteen a day — **70–130 reviewer-days of pure review** before that reviewer does anything else.

PH0 at 3 weeks is roughly right. PH1 at 10 is realistically 24–30; PH2 at 10 is 20–25 (the sync engine is the classic swamp and `50` says so); PH3 at 10 is 20–24 plus an external audit whose calendar time is not the team's; PH4 at 14 is 30–40 (two extractions, an economy activation, an SFU, a substrate migration); PH5 at 16 is 30–40. **Total: 127–162 weeks against a stated 63 — 2.0–2.6×, or 3–4× if the audit or store reviews go badly.**

**Remedy.** Do not compress the phases; the sequencing is right. (1) Restate `50 §1`'s durations as ranges with a stated confidence, and add a "if this phase exceeds 1.5× estimate" row to `50 §5` for every phase, not only PH2. (2) Designate the **cut candidate** per phase as `50 PH5` already does for Seasons — PH1's is density modes and half the component inventory, PH3's is Governance v1, PH4's is video and the Stage. (3) Fund a second reviewer before PH1, or state in `42` that review is the throughput ceiling.

**Cost.** A day of rewriting and a real staffing decision. **Buys:** the difference between a plan that fails and a plan that reports.

---

### 3.12 W12 — The hard line on Extension UI will not hold commercially, and the escape hatch is years away

**Implicated:** `20 §7`, `20 §12.6`, `21 §4.3`, `32 §11`, `19 §1`, `19 §14`, `50 PH6`, `50 PH7`.

`20 §7` gives Extensions a Surface Descriptor — design-system primitives, a fixed filter list, no DOM, CSS, JavaScript, canvas or iframe. The reasoning (spoofing, contrast, payload, reflow, coherence) is correct and I would make the same call. The stated cost is honest: "A rich diagram editor, a spreadsheet grid, a waveform scrubber — none of these is expressible." The stated answers are to grow the vocabulary through `32`, and to route custom rendering to the Experience Runtime.

The Experience Runtime is PH7 — nominally 93 weeks, realistically three to four years (§3.11) — behind seven gates including "zero sandbox escapes in two phases". So the marketplace launches at PH6 with a vocabulary that expresses polls, digests, triage queues and dashboards, while `19 §14` targets an **earnings Gini ≤ 0.75**, **top-10 share ≤ 25%** and **time-to-first-sale ≤ 21 days**. `19 §1` lists four ways a marketplace dies and omits the fifth: nothing worth selling. The categories that generate marketplace revenue everywhere else — editors, design tools, analytics surfaces, games — are exactly the ones the descriptor cannot express, and the answer to a creator asking for pixels is "wait four years".

Growing the vocabulary is also weaker than it reads: `32 §11` requires a design review, token map, CLI equivalent and a11y sign-off per component and an ADR per new pattern, while `02 §5` caps open ADRs at zero per gate. It is rate-limited by the first-party design team, which §3.11 already establishes as the binding constraint.

**Remedy.** Decide before PH6 rather than at its gate. **Option A (recommended):** pull one tightly-scoped primitive forward — a **host-owned, host-chromed, input-mediated canvas** with no ambient I/O, no persistence import, a fixed frame budget and no ability to draw outside its own rectangle. This is a strict subset of `20 §12.1`'s Experience client, so it costs sandbox work but not governance, tick-loop, session or economy work. **Option B:** accept the constraint and rewrite `19 §1` and `19 §14` to describe a marketplace of automations and templates — a legitimate product, but a different one, whose 12% fee (`19 §6.3`) should be re-justified against it.

**Cost.** Option A ~6 engineer-weeks in PH6 plus an ADR and a budget slot; Option B one day and a lower revenue projection. **Buys:** the decision is made deliberately at design time rather than under commercial pressure at the gate, which is where hard lines actually break.

---

### 3.13 W13 — Recovery is the real churn source, and the guardian model has a cold-start failure

**Implicated:** `12 §5.1`, `12 §6.1`, `12 §13 I-12.4`, `12 §14`, `34 §12.2`, `50 PH1`.

`12 §6.1` requires 3–7 guardians with `t ≥ ceil(n/2)+1`, drawn from Citizens, Society roles, or self-custody. `12 §3.3` prompts at first device and re-prompts at Level 1.

At launch there is nobody to nominate. The first thousand Citizens have no contacts, belong to no Society with a role structure (SL0 grants Founder governance only), and the remaining option — self-custody paper or an HSM — is the seed-phrase model `12 §12` rejects because "Citizens lose them". The modal early Citizen has one device, no recovery, and per `I-12.4` cannot even revoke that device.

`12 §14`'s triggers (passkey abandonment > 25%, guardian completion < 60%) are the right ones and are lagging indicators measured after the population exists. `50 PH1`'s risk table names only passkey browser support and does not name recovery-set formation at all.

The second-order effect is worse than the churn. A Citizen who loses their only device loses their FNID, Handle (reserved 12 months), Level, Trust, Memberships and balance, and creates a new Citizen — starting at Level 0, unable to found a Society (`18 §5.1`), inside the 72-hour new-Citizen envelope (`14 §10`). **Every recovery failure manufactures exactly the population profile the Sybil defences are tuned to suppress**, and `12 §9` accepts "a patient adversary wins at small scale" on the assumption that honest Citizens are not repeatedly re-registering.

**Remedy.** Before PH1 ships externally: (1) a **platform-operated guardian of last resort** holding exactly one sub-threshold Shamir share, with a hard rule that `t` exceeds the number of platform-held shares — this is not custody under `02 §4`, since it is a single sub-threshold share under an explicit user-initiated delegation; (2) **make the second device the primary recovery story**, asked for before guardians, because a second device is a mechanism people understand and a guardian set is one they will not complete; (3) add **recovery-set formation rate** to `50 PH1`'s acceptance criteria with a stated floor.

**Cost.** 2–3 engineer-weeks plus an onboarding surface. **Buys:** removes the most likely cause of early churn and stops that churn poisoning the Sybil model.

---

### 3.14 W14 — Custodian supply is gated behind Level 8 and paid in a unit with no external value

**Implicated:** `18 §5.5`, `18 §4.3`, `13 §7.4`, `13 §8.2`, `13 §12`, `13 §15`, `17 §1.1`, `17 §10.1`, `50 PH4`.

`18 §5.5` gates Custodian eligibility at `Level ≥ 8 ∧ Trust ≥ 200 ∧ Stake(500 FRC)`. `18 §4.3` gives time-to-L8 as 21 weeks (Light) / 8 (Moderate) / 3 (Heavy), and Trust ≥ 200 requires months of evidenced events — of which the highest-frequency admitted input, "Custodian Attestation streak honored", is unavailable to someone who is not yet a Custodian.

Then the economics. `13 §12` prices Custodian marginal cost at ~$0.30/TiB-month; `13 §8.2` computes that FRC's utility value must reach **≈$0.087** to recruit honest capacity. `17 §10.1` establishes that FRC has no trading pair, is not redeemable and is not transferable off-platform. The proposition is: reach Level 8, build Trust 200 over months, bond FRC, buy disks with dollars, and be paid in a unit that by design cannot become dollars until Phase 9, which `50 PH9` says "may never open".

`13 §15` names the fallback honestly — keep S3 authoritative and publish that the mesh is a supplement. But if it fires, the first of the two reasons `17 §1.1` gives for Fraction's existence disappears, leaving only "a unit of account for contribution" — which `17 §15.2` already names as the condition under which Fraction should be demoted to an internal accounting unit.

Separately, `18 §5.5`'s flat `Stake(500 FRC)` contradicts `13 §7.4`'s `bond = bond_rate × committed_bytes` at ~100 FRC/TiB: a 50 TiB Custodian bonds 5,000 FRC under `13` and 500 under `18`, a 10× difference in the parameter that makes slashing meaningful.

**Remedy.** (1) **Decouple eligibility from Level** and gate on what predicts good custody: a bond scaled by committed bytes (`13 §7.4`), proof-of-capacity at registration (already specified), and a probation period at reduced assignment. Level 8 gates nothing proof-of-capacity plus a bond does not gate better, and it excludes exactly the population — datacentre operators, NAS enthusiasts, small hosting providers — that would supply the first petabyte. (2) **Pre-recruit a first cohort** under an explicit published arrangement so `13 §11.4` step 4.3 does not depend on organic supply. (3) Add a measured PH4 entry precondition: committed capacity ≥ 3× current logical storage across ≥ 8 failure domains, at the modelled price, before step 4.1.

**Cost.** A parameter change and a paragraph, plus real money for the cohort. **Buys:** the mesh recruits or fails **before** the `BlobStore` migration, not during it.

---

### 3.15 W15 — Moderation under E2EE: the limits are honest, the exposure is not costed

**Implicated:** `14 §6`, `13 §10.3`, `21 §13` T3, `12 §11` T12, `19 §13` T7, `02 §4`.

Refusing client-side scanning is correct and correctly reasoned: a scanning path with a policy promise attached is a plaintext access path, and `E2` makes its absence a CI lint over the crate graph rather than a review convention. Franking (`14 §6`) trades deniability for accountability with the trade stated. `13 §10.3` and `21 §13` T3 state the limits without softening.

What is missing is the cost of the position. Several jurisdictions have enacted or are enacting duties a report-only posture does not satisfy — proactive detection obligations, technology notices, traceability requirements — all incompatible with "no code path accepts an MLS group secret". The blueprint's response exists in three chapters as a technical statement ("we cannot") and nowhere as a business decision: no list of jurisdictions served, no stated response to an order the architecture cannot satisfy, no gate at which the question is answered. `12 §14` handles the analogous identity case in exactly the right shape — "scope it to that Rail and jurisdiction as an adapter; if that is unacceptable to the regulator, the Rail does not ship there" — and it is not applied to E2EE.

Second uncosted item: the entire abuse-response capacity on private surfaces is **human review of franked reports**, and no chapter sizes that function. At 100,000 active Citizens and a conservative 0.5% monthly report rate, that is 500 reports a month requiring a human to view disclosed plaintext, with an appeal path (`01 §7`), an SLO, a staffing model and a duty of care to reviewers. `19 §10.3` sizes the *marketplace* review pipeline with latency tiers and an escalation ladder; the *safety* pipeline has no equivalent.

**Remedy.** (1) Add a **jurisdictional posture** section to `14 §6`: which regimes the platform intends to operate under, which are compatible with `N6` as written, and the pre-committed response to an incompatible order — geoblock, exit, or overturn `N6` by ADR. (2) Add a **safety operations model** to `50`'s continuous tracks alongside Security, Accessibility, Performance and Documentation: report volume projection, review staffing, latency and appeal SLOs, reviewer welfare, escalation ladder mirroring `19 §5.5`. (3) Make it a PH1 exit criterion, since PH1 is when external Citizens arrive.

**Cost.** Two weeks of policy plus counsel time; ongoing staffing. **Buys:** the platform does not discover at 50,000 Citizens that its abuse response is one person and a queue.

---

### 3.16 W16 — Performance budgets have three claimed single sources of truth, with conflicting numbers

**Implicated:** `32 §8`, `40 §13.1`, `34 §11`, `31 §10`, `50 PH2`.

`32 §8` states the budgets and `32` is declared to own them. `40 §13.1` restates them and says "the design system owns the numbers, this document owns their enforcement." `34 §11` states a third set and says "Budgets live in `perf/budgets.json`." `31 §10` states a fourth for the CLI.

| Metric | `32 §8` | `40 §13.1` | `34 §11` | `31 §10` |
|---|---|---|---|---|
| Desktop warm start | 400 ms | — | **600 ms** | — |
| Web warm start | 800 ms | — | **900 ms** | — |
| Desktop memory | **600 MB** | — | **450 MB RSS** | — |
| Web memory | **400 MB** | — | **350 MB** | — |
| Initial JS, first route | **180 KB gz** | 180 KB gz | **220 KB gz** | — |
| Total initial payload | — | **400 KB gz** | 220 KB gz | — |
| CLI binary | — | — | **18 MB** | **12 MB compressed** |
| CLI memory | — | — | **80 MB RSS** | **120 MB, 4 Societies** |

`50 PH2` AC-2 then encodes "memory ≤ 600MB under a 4-Society soak", following `32`; `34 §11` would fail that same build at 450 MB with five Societies. Whichever number lands in `perf/budgets.json` silently becomes truth and two chapters become wrong, because no lint compares a prose table to a JSON file.

**Remedy.** `perf/budgets.json` is the source; all four tables render from it via `cargo xtask budgets --render`; `cargo xtask lint-docs` fails on a hand-authored budget number — the mechanism `40 §4.2` already applies to colour literals. On the rows above I would take `34`'s stricter memory figures, `32`'s stricter warm-start figures and `31`'s CLI figures, each being the chapter closest to the measurement.

**Cost.** Two days. **Buys:** removes eight silently-wrong numbers and prevents the next twenty.

---

## 4. Scaling Analysis

Assumptions, stated so they can be attacked: 1 KB mean event envelope including payload; 30,000 events/second single-threaded deterministic fold with projection writes; 5,000–15,000 small write transactions/second on one Postgres primary with `synchronous_commit = on` on NVMe; 2.5 devices per Citizen; 150 KB steady-state memory per live WebSocket connection including TLS and framing; 10 Gbps NIC per Relay or SFU node. Figures from `13 §6.4`, `13 §7.5`, `14 §7`, `20 §11`, `40 §7.5` and `40 §13.1` are taken as given.

| # | Axis | First bottleneck | Binds at | Remedy |
|---|---|---|---|---|
| 1 | **Citizens** | The Global Registry has no partition key. Handle skeleton uniqueness (`12 §2.3`), the Citizen row, the device chain, global XP/Trust and the global Wallet all sit outside `society_id` partitioning, on the one primary. | ~10–50 M Citizens on a single primary; earlier if device-chain verification is on a hot path | Hash-partition the Global Registry by FNID prefix. It has no ordering requirement, so unlike the log it is genuinely partitionable. Before PH5. |
| 2 | **Societies per Node** | Desktop replica memory and disk. `34 §11` budgets 450 MB RSS at 5 Societies and 10 k cached Messages; `50 PH2` soaks 4 at 600 MB. Linear in resident Societies. | **~10–20 Societies per desktop Node**, against `14 §2`'s casual reference to "a Citizen in 400 Societies" | Lazy replication with a hot set: full replica for the *N* most recent, header-only beyond, promoted on open. `34 §12.3`'s `sync_step(budget)` is the driver; it needs a residency policy. |
| 3 | **Members per Society** | MLS leaves are **devices, not Citizens** (`14 §4.2`); `14 §4.4` binds "beyond ~1,000 leaves". | **~400 Citizens per E2EE Chamber** (1,000 ÷ 2.5). `14 §4.1`'s "3 to 5,000" is wrong by 12× when converted to leaves. Non-E2EE Chambers bind later, on row 6. | State every MLS figure in leaves. Above threshold, `14 §14`'s pre-committed fallback applies. Also fix `11 §2.3`, which specifies member caps only to 500 at SL2 and is silent above. |
| 4 | **Messages per Chamber** | Discourse projection index and `since(seq)` catch-up. ULID identity makes time-ordering free; the thread and search indexes grow linearly. | ~10^8 per Chamber before index maintenance dominates; the client binds far earlier — `34 §11` benchmarks a 5,000-message projection | Partition the Discourse projection by `(society_id, chamber_id)`; cold-archive threads past a retention horizon into Vault Objects with the log retained. |
| 5 | **Events per Society** | Projection rebuild time (§3.5). Storage is manageable; rebuild is not. | **~50 M events** ≈ 28 min per projection pass — where "drop and rebuild" stops being an incident response. At 500 M: 4.6 h per pass, ~37 h for eight independent projections | Per-projection checkpoints generalized from `16 §4.3`, plus a published rebuild SLO. |
| 6 | **Concurrent Signal subscriptions** | Relay connection memory. `14 §2` caps the outbound queue at 256 frames or **1 MiB** per connection and itself cites "30 k connections". | **~30–50 k per Relay instance** at 150 KB steady state (4.5–7.5 GB). 1 M concurrent needs 20–35 instances | Consistent-hash Societies to Relay instances so the subscription index is per-instance and `Scope::Society` fan-out is local. This is `10 §2` extraction ②; the sizing needs stating. |
| 7 | **Postings per second** | One primary, ordered `FOR UPDATE` per wallet, plus the single `EmissionAccount` row at settlement (§3.6). | **~5,000/s** steady state. The settlement burst binds far earlier: 100 k Societies closing at 00:00 UTC gives 8.3 min of fully serialized emission posting, violating the ledger SLO daily | Shard `EmissionAccount` into K sub-accounts; offset settlement close by `BLAKE3(society_id) mod 86400`. Then shard Societies across primaries (§3.4). |
| 8 | **Shards under custody** | Coordinator placement metadata and the cross-Society GC refcount (§3.9). 1 PiB logical → 1.6 PiB stored → **419 M fragments** at 4 MiB; at ~200 B placement state each, **84 GB** of coordinator metadata, plus a refcount over every Manifest in every Society. | **~1–10 PiB logical**, and much earlier for the refcount, which is unowned and cross-partition after the first Fracture | Partition placement by Shard hash prefix; make the refcount monotone-safe and require positive confirmation from every referencing Society before sweep. |
| 9 | **Custodians** | Not verification — `13 §7.5`'s 44,160 compressions per Custodian per epoch is trivial even at 10^6. It is **placement**: `13 §7.2`'s `assign()` filters, hashes and *sorts the entire active set* per Shard. | **~10 k Custodians × sustained write rate.** At 10 k and 1,000 new Shards/s, 10^7 hash-and-sort ops/s | Rendezvous hashing over a bucketed index, or a consistent-hash ring with domain-diversity buckets. Same distribution, O(log N) instead of O(N log N). Two days now; a rewrite later. |
| 10 | **Concurrent voice participants** | SFU egress. `14 §7`: 12-person video ≈ 19 Mbps; a 3-speaker/2,000-listener Stage ≈ 250 Mbps. Forwarding sustains 2,000–4,000 streams/core, so CPU is not the constraint. | Per 10 Gbps node: **~500 concurrent 12-person video Chambers** (~6,000 participants), **~4,000 8-person audio Chambers** (~32,000), or **~40 2,000-listener Stages**. Add 8–15% TURN fallback doubling egress on those | Extend `14 §7`'s regional cascade from Stages ≥ 500 to Voice Chambers. The media Sink (`17` K9) must price egress at cost or the mesh subsidizes it. |
| 11 | **Extensions installed** | `20 §11`'s **25 ms aggregate pre-commit budget per command** against an 8 ms p95 per hook. | **3–4 pre-commit-hooking Extensions per Society.** The 5th onward is skipped with `proceed`, recorded `deferred` — silently degraded | The most under-appreciated limit in the blueprint, and it directly caps marketplace value per Society. Move the dominant pattern (moderation, content policy) to post-commit with compensation, reserving pre-commit for hooks that must block; publish the per-Society Extension budget in the Listing UI. |
| 12 | **Simulation-harness runtime** | The two published rates are inconsistent by 6.25×: `40 §7.5` and ADR-0014 give 2,000 seeds in ~6 min on 8 cores (5.6/s) and 500,000 nightly in 4 h (34.7/s). | Pre-phase-gate is **5,000,000 seeds**: **250 core-hours** at the per-PR rate, **40 hours** at the nightly rate. Either way a multi-day job, and a gate blocker | Reconcile the rates (the per-PR figure presumably includes the regression corpus; say so). Then fund a burst fleet — 5 M at 34.7/s across 64 machines is ~40 minutes — or reduce the pre-gate figure. A gate nobody can afford to run is not a gate. |
| 13 | **Repair bandwidth** | 10× read amplification per lost fragment at 5%/month attrition — `13 §6.4` calls it the largest recurring cost and is right. | `13 §6.4`'s own arithmetic: **2.72 Gbps continuous, forever, per PiB logical.** At 10 PiB, 27 Gbps of pure repair traffic funded from the same emission cap as custody | `13 §15` already names locally-repairable codes as the response. Give it a measured trigger: adopt LRC when repair egress exceeds 40% of total mesh egress. |

---

## 5. Cross-Chapter Inconsistencies

Contradictions, not ambiguities: places where two chapters state incompatible facts and an implementer must pick one.

| # | Contradiction | Chapters | Resolution |
|---|---|---|---|
| **X1** | `40 §6.4` reproduces a worked ADR numbered **ADR-0009** for deterministic simulation, dated 2025-11-14. The corpus assigns **0009** to the Facet standard and **0014** to deterministic simulation, both dated 2026-09-03. The reproduced text also cites "integration tests (§6.6)"; they are `40 §7.6`. | `40 §6.4`, `adr/0009`, `adr/0014` | Renumber to **ADR-0014**, fix the date and the internal reference, and transclude the real file at build time so the two cannot diverge again. |
| **X2** | `34 §2.1` places `fractal-app` — explicitly "commands, queries, **policy PEP**" — inside **THE CORE**, "identical bytes of logic on every target", where targets include the browser. `41 §8.1` marks `fractal-app-*` **✖ by design** for `wasm32`: "someone will eventually run policy enforcement in the browser, which `10 §8` forbids." | `34 §2.1`, `41 §8.1`, `10 §8` | `41` is right and `34`'s diagram is the more visible artifact. Redraw the core box with `fractal-domain-*`, `fractal-sync`, `fractal-store`, `fractal-crypto`, `fractal-core` inside and `fractal-app-*` outside, PEP annotated server-only. Add the diagram to the doc-drift detector. |
| **X3** | **Redis** appears in `10 §1`'s topology, `14 §2`'s presence table and `40 §7.6`'s integration dependencies — and in **neither** `10 §7`'s port table, declared "the complete swappable list", **nor** `10 §9`'s topology, which shows only Postgres, Object Store and NATS. | `10 §1`, `10 §7`, `10 §9`, `14 §2`, `40 §7.6` | Either add a `PresenceStore` port with two implementations, plus Redis in `10 §9` and in the phase dependency count; or delete it and hold presence in Relay memory. Recommend the latter: presence is 45-second TTL data with no durability requirement, and a fourth stateful system is a poor trade against `10 §2`'s operational-surface argument. |
| **X4** | **`Quanta` is `u128` in one place and `i64` in another.** `16 §2.1`, `16 §5` and ADR-0006 §2 all say `u128`. `17 §2.1` says `i64` persisted / `i128` intermediate, and gives the reason: 128-bit arithmetic is emulated and slow on `wasm32`, and 1e9 FRC × 1e9 quanta leaves 9.22× headroom in `i64`. | `16 §2.1`, `16 §5`, `17 §2.1`, ADR-0006 §2 | `17` has the better argument, and `u128` additionally cannot represent the mandatory negative `EmissionAccount` balance (`16 §4.2` LA1) — a signed type is required. Resolve to **`Quanta(i64)` persisted, `i128` intermediate**; fix `16` and ADR-0006; add a compile-time headroom assertion. This becomes unfixable after the first Posting. |
| **X5** | **The marketplace fee is 12% in one chapter and 5% in another, with different splits.** `19 §6.1` sets 12% (10% services, 4% launch) with a 70% creator floor and splits it 6 burn / 4 operations / 2 assurance; `17` K7 sets "5% of price, 40% burn / 40% seller's Society Treasury / 20% Platform Reserve"; `50 M6.3` follows `19`. `19 §13` T6 also says a self-purchase "pays 100% platform fee", contradicting `19 §16` invariant 4 (creator ≥ 70% on every settled purchase). | `17` K7, `19 §6.1`, `19 §6.2`, `19 §13`, `19 §16`, `50 M6.3` | `19` owns the market; generate K7 from `19 §6.1`. Reconcile the Treasury leg — `17` gives Treasury 40% of the fee while `19` gives the Society 0–10% of gross only on Shelf-originated sales; these are different mechanisms and both cannot be K7. Make a detected self-purchase **void** rather than 100%-fee'd, preserving invariant 4. |
| **X6** | **Storage rates, replication factor, free quotas and emission share disagree across `13`, `17` and `18`** — 17.5× on the tariff, 6.1× on custody, 1.6× vs 3× on replication, 9× on free quota, 20% vs 32% on Source share. Arithmetic in §3.2. | `13 §8.2`, `13 §6.1`, `17 §1.2`, `17 §3.2`, `17 §3.3`, `17` K8, `18 §5.1`, `18 §5.2` | One machine-readable rate table owned by `17`, generating `13`'s worked example and `18`'s quota tables; `econ-sim` invariants I-E11–I-E13 asserting the agreement. |
| **X7** | **Society creation requires Level 3, and the spine sentence requires it on day one.** `18 §5.1` places "Found a Society" at L3 and `30 §4.3` #6 encodes `society.create (Level ≥ 3)`; `18 §4.3` gives time-to-L3 as 2 weeks / 5 days / 2 days. `02 §2`'s spine sentence is "**A Citizen can create a Society**…" and `50 PH1` AC-1 requires registration → Society creation → first message in **under 3 minutes, unassisted**. | `02 §2`, `18 §5.1`, `18 §4.3`, `30 §4.3`, `50 PH1` | Both positions have merit; `18 §1.1`'s J1 Sybil argument is correct and so is `02 §2`. Resolve with a **first-Society allowance**: every Citizen may found exactly one Society at L0 — which `17` K1 already prices at 0 FRC — with additional Societies gated at L3 and priced at 250 FRC. The Sybil gate then sits on Society *volume*, which is the farmable thing. |
| **X8** | **Phase assignments contradict across eight chapters**: `50` vs `41 §5` (progression, governance, agent, economy, discovery, transcoder, rail), `30 §4.2` (agent, vault, discovery), `17 §3.2` (S4–S7 at Phases 2–3 against `50`'s "0 Sources" in PH2 and PH3), `19 §17` (listings and review at "4–5" vs PH6), `20 §2` (workflow and automation-pack at Phase 4 vs `50 M3.5`/`M3.7`), `21 §14` (S4 accrual at Phase 2), `16 §20` (internal anchoring at Phase 2, for which `50 PH2` has no milestone). | eight chapters | `docs/phases.toml` as the single source; every phase column generated; `cargo xtask lint-phases` in the fast lane. |
| **X9** | **Performance budgets conflict across four chapters**, each claiming or implying ownership: desktop warm start 400 vs 600 ms, desktop memory 600 vs 450 MB, web memory 400 vs 350 MB, initial JS 180 vs 220 KB, CLI binary 12 vs 18 MB, CLI memory 120 vs 80 MB. `50 PH2` AC-2 already encodes one of the conflicting values as a gate. | `32 §8`, `40 §13.1`, `34 §11`, `31 §10`, `50 PH2` | `perf/budgets.json` as the source; all four tables generated; a doc lint on hand-authored budget numbers. Table in §3.16. |
| **X10** | **MLS scale is stated three ways.** `14 §4.1`: "sizes from **3 to 5,000**". `14 §4.4`: binds "beyond **~1,000 leaves**". `50 PH2`: E2EE limited to "≤ 200 members" in PH2. `10 §12`: fallback triggers at ">1,000 members". Leaves are devices (`14 §4.2`), so 5,000 members is ~12,500 leaves — an order of magnitude past the bind point. | `14 §4.1`, `14 §4.2`, `14 §4.4`, `10 §12`, `50 PH2` | State everything in **leaves**, once, in `14 §4.4`, and derive the member figure from a stated devices-per-Citizen assumption. This matters because it triggers a pre-committed architectural fallback. |
| **X11** | **The Citizen Vault has no home under P1.** `01 §1` scopes Vault under Society; `01 §6`'s Global Registry is a closed nine-entry list that excludes it; `11 §7.1` requires every persisted row to have a `society_id` or be on that list. Yet `18 §5.1` grants every Citizen a Vault at L0, `13 §10.4` names Citizen private Vault keys, and `21 §3.4` builds eight Modules on it. `21 §5.1` identifies the gap and `21 §17` proposes the fix — **which is not in `01` or `11`**. | `01 §1`, `01 §6`, `11 §2.7`, `11 §7.1`, `13 §10.4`, `18 §5.1`, `21 §17` | Land `21 §17`'s amendment (`Vault` gains `society: Option<SocietyId>`, mirroring `Wallet`) in `11 §2.7` and `01 §1` **before PH1**, because `11 §7.1` becomes a failing property test the moment the first Citizen Vault Object is written. |
| **X12** | **Offline Agent invocation is answered two ways.** `15 §6` rule 5: an Agent on a disconnected replica "**queues commands in the outbox** and they are authorized on arrival." `34 §12.1`: "Invoke an Agent — **Refused** unless a local `ModelProvider` is configured", on every target. | `15 §6`, `34 §12.1` | Different questions (inference availability vs action authorization) reading as contradictory. State both: inference is refused without a local provider; action authorization is always deferred to reconnect and never granted offline. |
| **X13** | **The Custodian bond is a flat 500 FRC and a per-byte rate.** `18 §5.5`: `Stake(500 FRC)`. `13 §7.4`: `bond = bond_rate × committed_bytes` at ~100 FRC/TiB, calibrated to two months of earnings. A 50 TiB Custodian bonds 5,000 under `13` and 500 under `18`. | `13 §7.4`, `18 §5.5` | `13` owns custody economics; delete the flat figure from `18 §5.5`. A bond's purpose is to make slashing costlier than misbehaviour, which a flat figure cannot do across three orders of magnitude of capacity. |
| **X14** | `50 PH1` AC-5 requires **500,000** simulated histories at the gate; `40 §7.5` and ADR-0014 specify **5,000,000** pre-phase-gate. | `50 PH1`, `40 §7.5`, ADR-0014 | Pick one, generate the other, and check the choice against §4 row 12's arithmetic before writing it into a gate. |
| **X15** | **Four broken document references.** `16 §2.3`/`§7.1` cite `30-api-contract.md` (actual: `30-api-and-sdk.md`); `19 §5` and `20 §5` cite `19-marketplace-and-commerce.md` (actual: `19-marketplace.md`); `50 M1.8` and `50 PH1` cite a chapter `51` that does not exist; `00 §1 P12` cites "`19` / `17-economy-fraction.md`" for the economy simulation harness, which lives only in `17 §12`. | `16`, `19`, `20`, `50`, `00` | A link checker in the fast lane. `40 §5` already forbids documentation drift; this is drift with a mechanical fix. |

---

## 6. Unstated Assumptions

Each is load-bearing, none is written down as an assumption, and each has a specific consequence if false.

**A1 — Contributors will accept a non-redeemable internal unit as compensation for real-dollar costs.** `17 §10.1` establishes no trading pair, no redemption, no off-platform transfer; `13 §12` prices hardware in dollars; `19 §7` tells creators there is no cash-out before Phase 9, which "may never open". *If false:* the mesh never recruits (§3.14), the marketplace never acquires supply, and `17 §15.2`'s named fallback becomes the design. Cost that fallback now.

**A2 — Cross-Society activity is rare and read-mostly.** ADR-0004 §3 accepts the partitioning cost "because cross-Society operations are rarer, more tolerant of eventual consistency, and mostly read-only". *If false* — if the median Citizen belongs to twenty Societies and lives in a unified inbox, which is how comparable products actually distribute — the highest-traffic surface in the product is the one with no owner and the worst characteristics (§3.9). **The likeliest assumption to be false, and its failure is architectural rather than tunable.**

**A3 — OpenMLS is production-ready at PH2's scale and schedule.** `10 §11` concedes "young ecosystem"; `50 M2.10` puts it on the critical path. *If false:* PH2 slips, or E2EE ships reduced — and `N6` is a non-negotiable, not a target. `10 §12` pre-commits the fallback but not the *evaluation*, which belongs in a PH1 spike.

**A4 — Agents produce this corpus at a defect rate the invariant suite catches.** The whole timeline argument rests on it, and `40 §7.5`, `42 §7` and `40 §14.3` are unusually well-designed for it. *If false:* the failure mode is not obvious bugs — the harness catches those — but **specification-satisfying wrongness**: code that meets its acceptance criteria and its invariants while being the wrong thing. Only human review detects that, and human review is the throughput ceiling (§3.11).

**A5 — Declared-interest discovery bootstraps a network from zero.** `14 §8` bans every behavioural signal and offers declared interests plus Serendipity at 2/Citizen/week; `14 §14` pre-commits to not trading P9 for growth. *If false:* the platform has no growth mechanism at all, since advertising (`02 §4`) and paid placement (`19 §8`) are also banned. Growth must then come from invitation, community seeding or integrations — none of which is designed or budgeted anywhere in the corpus.

**A6 — The fifteen invariants of `11 §7` are the right fifteen.** The oracle asserts exactly them, and `sim-mutation` proves it can detect their inversion. Nothing proves the *set* is complete. Two gaps from this review: no invariant for post-Fracture confidentiality (§3.10), none for cross-Society refcount under-counting (§3.9). *If false:* five million seeds find nothing while a class of defect the oracle was never asked about ships. Every chapter adds invariants; the oracle's coverage of those lists should itself be a CI check.

**A7 — Passkeys plus social recovery are usable by a non-technical population.** `12 §5.1` removes passwords entirely; `12 §14` sets 25% signup abandonment as the trigger. *If false:* the trigger fires after the population exists, and `12 §14` forecloses the obvious remedy — "Do **not** add passwords; that trade is not available" — leaving two options both narrower than the failure (§3.13).

**A8 — No jurisdiction the platform must serve will mandate proactive scanning of E2EE content before PH6.** `14 §6` and `13 §10.3` foreclose scanning by construction, enforced as a crate-graph lint. *If false:* the response is geoblocking, market exit, or overturning `N6` by ADR — none of which is written down as available anywhere (§3.15).

**A9 — One Postgres primary suffices through PH5.** ADR-0006 §9's review trigger presumes partitioning adds write capacity; it does not (§3.4). *If false:* the remedy is a topology change touching the composition root, the backup story, and the one-transaction property the ADR exists to preserve.

**A10 — One human is available to approve every Milestone, sign every release and review every Work Unit, and never becomes the bottleneck.** `42 §2.1` makes Milestone closure the only event requiring a human signature; `40 §8.3` requires one human approval per merge with no bypass; `02 §8` routes every scope escalation through one named person. *If false* — illness, competing obligations, or simply §3.11's arithmetic — the pipeline stops. There is no delegation path, no second approver and no degraded mode. **This is the only true single point of failure in the design, and it is a person.**

**A11 — Free storage quotas are affordable for a pre-revenue platform.** `18 §5.1`/`§5.2` grant up to 250 GB per Citizen and 10 TB per Society before "unmetered" at SL4, funded from emission. *If false:* the platform subsidizes storage in a unit it prints against a real dollar bill (`13 §12`) with no revenue — the mechanism by which a well-designed economy still runs out of money.

**A12 — The Society is the boundary users experience, not merely the one engineering wants.** P1 is asserted as a first principle and never validated. It is an excellent engineering boundary; whether people organize around it rather than around "my people, who span Societies" is empirical with an architectural answer. Testable alongside A2.

---

## 7. The Improvements

### 7.1 Change before PH0 — cheap now, expensive later

| # | Change | Chapters | Cost | Buys |
|---|---|---|---|---|
| I1 | **`docs/phases.toml` as the single phase source**, every phase column generated, `cargo xtask lint-phases` in the fast lane | `50` + eight consumers | 3–5 days | Eliminates X8 and a class of agent misdirection; makes `02 §5` mechanically enforceable |
| I2 | **Settle `Quanta` as `i64` persisted / `i128` intermediate** per `17 §2.1`; correct `16 §2.1`, `16 §5`, ADR-0006; add a headroom assertion | `16`, `17`, ADR-0006 | 1 day | Prevents an unfixable ledger type migration after the first Posting; `u128` also cannot hold the negative `EmissionAccount` |
| I3 | **Land `21 §17`'s Canon amendment**: `Vault` gains `society: Option<SocietyId>` | `01 §1`, `11 §2.7` | 1 day | `11 §7.1` stops being a failing property test the moment a Citizen Vault Object exists (X11) |
| I4 | **`perf/budgets.json` as the single budget source**, four tables generated, doc lint on hand-authored numbers | `32`, `40`, `34`, `31` | 2 days | Removes eight conflicting numbers, one already a PH2 gate criterion (X9) |
| I5 | **Resolve the Society-creation gate**: first Society at L0 and free (already `17` K1); additional at L3 and 250 FRC | `18 §5.1`, `30 §4.3`, `02 §2` | half a day | Makes `50 PH1` AC-1 and the spine sentence achievable (X7) |
| I6 | **Resolve Redis**: a `PresenceStore` port with two implementations and topology/budget entries, or delete it and use Relay memory. Recommend the latter | `10`, `14 §2`, `40 §7.6` | 2 days | Removes a fourth stateful system, or makes it honest (X3) |
| I7 | **Renumber `40 §6.4`'s worked ADR to 0014**, fix its date and §6.6 reference, transclude rather than duplicate; add a link checker for X15 | `40 §6.4` | 1 day | Removes the corpus's most visible self-contradiction and prevents recurrence |
| I8 | **Redraw `34 §2.1`** to place `fractal-app-*` outside the shared-core box | `34 §2.1` | half a day | Removes the one diagram that, followed literally, puts the PEP in the browser (X2) |
| I9 | **Name owners for the three global projections**: boundary S15, crate `fractal-app-global`, with stated consistency models — inbox (bounded staleness), search (index-only), refcount (**monotone-safe**) | `10 §3`, `41 §5.4`, `13 §10.2`, `14 §2` | 3 weeks design | The most-used surface acquires an owner before it is invented under deadline; the GC refcount acquires a correctness posture before it can destroy data (§3.9) |
| I10 | **Re-base the emission schedule on epochs since first Source activation**, not calendar years | `17 §5.1`, `17 §13` | 2 days | The economic model survives schedule slip instead of being falsified by it (§3.3) |

### 7.2 Change before the phase where it bites

| # | Change | By | Cost | Buys |
|---|---|---|---|---|
| I11 | Shard `EmissionAccount` into K sub-accounts; offset settlement close by `BLAKE3(society_id) mod 86400` | PH1 | 1 week | Removes the only global write-contention point in a design claiming to have none (§3.6) |
| I12 | One machine-readable economy rate table owned by `17`, generating `13 §8.2` and `18`'s quotas; add I-E11–I-E13; delete SL4's "unmetered Vault" | PH1 | 1 week | The PH4 economic gate becomes a real test (§3.2, X6) |
| I13 | Generalize projection checkpointing; publish a rebuild SLO; add a large-history fixture | PH2 | 3 weeks | Removes a 5-to-37-hour outage from the top of the incident distribution (§3.5) |
| I14 | Prove the `EventStore` port survives a two-primary composition root; rewrite `10 §12`/ADR-0006 §9's ladder to name sharding, not partitioning | PH2 | 2 weeks | Converts an emergency migration into a planned one (§3.4) |
| I15 | Decide the `HistoryKey` recovery question; add I-12.13; require a second factor before an evictable-storage-only Citizen holds Fraction or a Membership above L1 | PH2 | 1 week | Closes the gap between what the product promises about DMs and what a guardian quorum can read (§3.7) |
| I16 | Spike OpenMLS at 1,000 leaves on real hardware; restate every MLS scale figure in leaves | PH1 | 1 week | Moves the `10 §12` fallback decision off PH2's critical path (X10, A3) |
| I17 | Treat Extension-supplied tool definitions as untrusted content; namespace at the host; add A11; extend `50 PH3` AC-3's injection suite; add fee disclosure and a per-Install fee ceiling for hook 18 | PH3 | 1 week | Closes the only influence channel the threat tables name and leave open (§3.8) |
| I18 | Restructure the pre-commit hook budget: moderation and policy hooks to post-commit with compensation; publish the per-Society Extension budget in the Listing UI | PH3 | 2 weeks | Raises the Extension-per-Society ceiling from 3–4, currently a hard cap on marketplace value (§4 row 11) |
| I19 | Replace `13 §7.2`'s full-set sort with rendezvous hashing over a bucketed index | PH4 | 2 days | Two days now versus a rewrite at 10 k Custodians (§4 row 9) |
| I20 | Decouple Custodian eligibility from Level; bond by committed bytes per `13 §7.4`; pre-recruit a first cohort; add a measured capacity precondition to PH4 entry | PH4 | parameter change + real money | The mesh recruits or fails before the `BlobStore` migration, not during it (§3.14, X13) |
| I21 | Add jurisdictional posture to `14 §6` and a safety-operations model to `50`'s continuous tracks, funded as a PH1 exit criterion | PH1 | 2 weeks + staffing | The platform does not discover at 50,000 Citizens that its abuse response is one person and a queue (§3.15) |
| I22 | Add content-key rotation to `11 §3.2`'s Fracture execution, plus invariant 16 and a property test in `50 PH5` AC-1 | PH5 | 2 weeks | The signature operation stops leaking the thing people fracture over (§3.10) |
| I23 | Decide the Extension UI escape hatch: pull a host-owned input-mediated canvas forward from PH7, or rewrite `19 §1`/`19 §14` for a marketplace of automations | PH6 | 6 weeks or 1 day | The decision is made at design time rather than under commercial pressure at the gate (§3.12) |
| I24 | Restate `50 §1`'s durations as ranges; designate a cut candidate per phase; fund a second reviewer or state that review is the ceiling | PH0 | 1 day + staffing | The difference between a plan that fails and a plan that reports (§3.11) |
| I25 | Add recovery-set formation rate to `50 PH1`'s acceptance criteria; make the second device the primary recovery story ahead of guardians | PH1 | 2 weeks | Removes the most likely cause of early churn and stops it poisoning the Sybil model (§3.13) |

### 7.3 Monitor, decide later

| # | Signal to instrument | Decision it informs |
|---|---|---|
| I26 | Societies-per-Citizen distribution, and share of session time in a cross-Society surface versus in-Society, published weekly from PH1 week one | Whether A2 and A12 hold, and therefore whether the per-Society partition is the right boundary. **The highest-value single measurement in this document.** |
| I27 | Exchange share of Citizen FRC income, from the first Source | `17 §5.3`'s falsification test and the `17 §15.2` fallback |
| I28 | Passkey signup abandonment; guardian recovery completion | `12 §14`'s two named abandonment triggers |
| I29 | Postings/second headroom against the single-primary ceiling; settlement window duration | When I14's sharding work must start |
| I30 | Repair egress as a share of total mesh egress | When `13 §15`'s locally-repairable-code ADR is written |
| I31 | Pre-commit budget consumption per Society; count of `deferred` hook executions | Whether I18 was sufficient |
| I32 | Earnings Gini and time-to-first-sale from the first paid Listing | Whether I23's Option A was necessary |

---

## 8. What Would Make Me Abandon This Architecture

Not patch — rethink. Each is evidence that a premise is wrong rather than a parameter mis-set.

1. **The median Citizen belongs to more than ten Societies and spends most of their time in a cross-Society surface.** This falsifies A2 and makes per-Society partitioning the wrong boundary — not slow, wrong. The system would need a Citizen-primary partition with Societies as secondary indices, which is a different architecture. Measurable in PH1 week two (I26). This is the one I would watch hardest.
2. **`sim-mutation` fails: the harness does not detect a deliberately inverted invariant within 200 seeds.** ADR-0014 §7 calls this "the test of the test". If the oracle has stopped asserting, every green build since it broke is a false statement, and the argument for accepting agent-written code at volume collapses. Stop feature work; do not raise the seed count.
3. **Custodian recruitment at PH4 delivers under 30% of modelled capacity at the modelled price, after I20 has been tried.** This kills the first of the two reasons `17 §1.1` gives for Fraction's existence. `17 §15.2`'s response — demote Fraction to an internal accounting unit and move rewards to fee-share — is a different economic architecture, not a tuned one.
4. **An Envelope escalation is found in production after the PH3 external audit passes.** Not a hook bug, not an over-granting Citizen: an actual capability produced that the grantor did not hold. The agent and extension model rests entirely on `I-12.7` and `A3` being structural. The correct response is to remove third-party code execution and ship a first-party platform, per `10 §12`'s standing commitment that P8 outranks P7.
5. **A jurisdiction the platform must serve mandates proactive scanning of E2EE content.** `N6` is a non-negotiable and `E2` makes it a lint. No version of this design complies. The response is exit or a Canon amendment, and a Canon amendment here changes what the product is.
6. **Two consecutive phases exceed 2× their estimate with no identifiable single cause.** One phase over is normal. Two, with the cause distributed across everything, means the corpus specifies more per unit of delivery than the team plus agents can build. Cut the architecture to the spine sentence and ship that; do not extend the schedule again.
7. **The ledger reconciler finds a divergence it cannot explain, twice.** `16 §21` names the single case. Twice means the "log is authority, projection is disposable" premise is not holding in practice, and the argument for event-sourcing money — which is the argument for most of `16` — is unproven where it matters.
8. **PH1 external Citizens do not return after week one.** Already in `50 §5`, and it belongs here too, because it is the only item that would make everything else moot.

---

## 9. Confidence Table

Confidence that the *design* is right, not that the implementation will be. "What would raise it" names the cheapest available evidence.

| Subsystem | Chapters | Confidence | Principal doubt | What would raise it |
|---|---|---|---|---|
| Canon and conflict order | `00`, `01`, `02` | **High** | P10 at position 12 will be cited to justify a slow thing that did not need to be slow | Nothing. The strongest part of the corpus. |
| Per-Society partitioning | `10 §4`, ADR-0004 | **High on mechanism, Medium on premise** | A2 — that cross-Society use is rare | I26, in PH1 week two |
| Ledger and double-entry | `16 §2–§5` | **High** | The `EmissionAccount` hot row (§3.6); the `Quanta` type contradiction (X4) | I2 and I11 landed; reconciler green through PH2 |
| Event sourcing and replay | `10 §5`, `11 §7.10`, `16 §3` | **Medium-High** | Rebuild time is unbounded and unmeasured (§3.5) | I13: a published rebuild SLO met against a 100 M-event fixture |
| Postgres as the store | `16 §4.1`, ADR-0006 | **Medium** | The stated scaling remedy does not add write capacity (§3.4) | I14: the port surviving a two-primary composition root in PH2 |
| Capability and Envelope model | `12 §7`, `15 §4`, `20 §5` | **High** | Influence, not authority — the Extension→Agent tool channel (§3.8) | The PH3 external audit plus I17's extended injection suite |
| Agent runtime and PEP | `15` | **High** | Confirmation fatigue is named and not measured; the 5 ms PEP budget under counter contention | Measured confirmation-response quality after 90 days of PH3 |
| Extension sandbox | `20 §4`, `20 §11` | **Medium-High** | The 25 ms aggregate pre-commit budget caps Extensions per Society at 3–4 (§4 row 11) | I18 plus real per-Society budget telemetry in PH3 |
| Storage substrate | `13 §3–§6` | **High** | Repair bandwidth is the dominant recurring cost and is honestly stated | I30's measurement; an LRC ADR when it crosses 40% |
| Custodian protocol and attestation | `13 §7` | **High on cryptography, Low on supply** | Nobody may show up (§3.14) | I20, and a measured capacity precondition at PH4 entry |
| Storage economics | `13 §8`, `17 §3–§5` | **Low** | Rates differ 6–18×; replication factor wrong; free quota 9× understated; wrong clock (§3.2, §3.3) | I10 and I12, then a re-run of the ten-year model with I-E11–I-E13 green |
| Emission and supply policy | `17 §5`, `17 §9` | **Medium-High** | The structural argument is excellent; the inputs are wrong | As above. The mechanism is sound; the parameters are not. |
| Contribution metrics and anti-farming | `17 §7–§8`, `18 §3`, `18 §9` | **High** | The one admitted gap — an adversary who genuinely passes the Standing gate — is correctly bounded by the 0.5% backstop | `econ-sim`'s `collusion_ring_scaling` green at n = 500 |
| Progression and reputation | `18` | **High** | Every curve constant is a guess and says so; the Level-12 ceiling is the right call | Nothing before real population data |
| Identity, FNID, key rotation | `12 §2–§4` | **High** | Offline verification degrades after rotation, honestly stated | Nothing |
| Recovery | `12 §6` | **Low** | Cold-start guardian formation; the `HistoryKey` ambiguity (§3.7, §3.13) | I15 and I25, plus formation rate as a PH1 criterion |
| E2EE (MLS) | `14 §4`, ADR-0010 | **Medium** | Scale stated three ways (X10); OpenMLS maturity (A3); the archive-key composition (§3.7) | I16's PH1 spike at 1,000 leaves on real hardware |
| Moderation under encryption | `14 §6` | **High on mechanism, Low on operations** | Franking is genuinely good. Nobody has sized the human review function or the legal posture (§3.15) | I21: a funded safety-operations model and a named jurisdictional posture |
| Realtime and Relay | `14 §1–§3` | **High** | The backpressure ladder and `Gap` semantics are the best-specified part of `14` | A load test at 50 k connections per instance |
| Voice, video, Stage | `14 §7` | **Medium-High** | Egress economics; the E2EE downgrade above 5,000 listeners is correctly surfaced | Measured media Sink coverage in PH4 |
| Discovery without surveillance | `14 §8` | **Medium** | It may simply not bootstrap (A5), and every alternative is banned | Serendipity acceptance and Convergence→Crystallization conversion in PH4 |
| Fracture | `11 §3.2` | **Medium** | Conservation invariants are excellent; confidentiality is absent (§3.10) | I22, plus the ten supervised fractures `50 PH5` already requires |
| Facet standard | `16 §10–§16`, ADR-0009 | **High** | The evolution model beats the alternatives; the trust assumption is stated | ADR-0009's own review trigger (b): whether >5% of Facets use non-`Immutable` evolution |
| Marketplace | `19` | **Medium** | Nothing worth selling until the UI constraint moves (§3.12); the fee contradiction (X5) | I23's decision, plus `19 §14`'s Gini and time-to-first-sale |
| Design system | `32`, `33` | **High** | Coherent, well-argued, and the hard line is correct engineering | Nothing. The doubt is commercial and lives in `19`. |
| Client platform strategy | `34` | **Medium-High** | The mobile decision is well-reasoned and expensive; the diagram error (X2); the budget conflicts (X9) | I4 and I8; the Phase 3 UniFFI conformance gate `34 §7` already specifies |
| CLI and Terminal | `31` | **High** | The generated command tree makes P13 a build error rather than a promise | Nothing |
| Engineering standards and testing | `40` | **High** | The pre-gate seed count may be unaffordable (§4 row 12); the ADR numbering error (X1) | I7 and X14's reconciliation |
| Repo and crate structure | `41` | **High** | The phase column contradicts `50` (X8); dependency-direction enforcement is otherwise exemplary | I1 |
| Source control and agent protocol | `42` | **High** | Human review is the throughput ceiling and `42` does not say so (§3.11, A10) | I24, and a named second approver |
| Roadmap | `50` | **Medium-Low** | Durations are 2.5–4× optimistic (§3.11); the phase table is not authoritative (X8) | I1 and I24 |

---

## 10. Closing

Most of this blueprint is right, and the parts that are wrong are wrong in ways that are checkable — a direct product of `00 §1`'s discipline that every principle carries a falsification test, and the reason this critique could be written from the documents rather than from a running system.

The three findings that matter most are not the security ones. **The phase table is not authoritative** (X8), so agents will build the wrong things in the wrong order and no gate will catch it. **The storage economics do not close** (§3.2), so the phase that activates the economy is gated on a model that does not describe the system. **The cross-Society read path is named as a cost three times and designed zero times** (§3.9), so the surface users spend most of their time in has no owner, no consistency model and no crate.

None of the three is architectural. All three are the same failure: a corpus large enough that a fact stated in one chapter and contradicted in another survives, because no mechanism compares them. `40 §4` already solved this for terminology, `41 §7` for dependency direction, `32 §2` for design tokens and `41 §12` for schemas. The remedy is to extend the same treatment to phases, rates and budgets — which is what I1, I4 and I12 are.

The one finding that is architectural is A2, and it is not yet a finding. It is a measurement to take in the second week of PH1.
