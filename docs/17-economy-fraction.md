# 17 — The Fraction Economy

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `10-system-architecture.md`, `11-domain-model.md`.
> **Governs:** Fraction supply policy, the complete Source and Sink registry, emission scheduling, Contribution Score definitions, anti-abuse economics, internal market mechanisms, wallet economic UX, the economy simulation gate, and parameter governance.
> **Does not govern:** the `Ledger` trait, `Posting` mechanics, or the Facet Standard — see `16-ledger-and-assets.md`. XP, Level, Trust, and Standing mechanics — see `18-progression-and-reputation.md`. This chapter *consumes* Trust and Standing as inputs; it never defines or writes them (Invariant 8, `11 §7`).

---

## 1. Economic Thesis

### 1.1 What Fraction is for

Fraction (FRC) exists to solve exactly two problems that a social platform with real resource costs cannot solve without it:

1. **Settlement between peers.** In the Custodian mesh (`13`), the party paying for storage and the party providing it are both Citizens or Nodes. There is no merchant and no customer — there are peers exchanging a metered commodity in continuous, sub-cent increments. A fiat rail cannot do this: the per-transaction cost exceeds the transaction.
2. **A unit of account for contribution.** A Society needs to say "this Chamber cost 40 GB-months and Ana's curation carried it" in one number, comparably, across Societies, across time, without an off-platform intermediary.

Fraction is **not** a speculative instrument, and this document treats any design pressure toward making it one as a defect. It is internal-only until Phase 9 (`02 §3`), non-redeemable, has no external market, and the platform quotes no external price.

### 1.2 The anchor — the single most important economic decision in this document

**Invariant E1 (the Anchor).** One Fraction is defined as the administered tariff for **one gigabyte-month of Vault storage**.

```
   1 FRC  ≡  1 GB · month of Vault storage at the reference tariff
```

FRC is therefore a **commodity-anchored internal unit of account**, not a floating claim on nothing. Every other price in the system is stated relative to a physical quantity the platform actually delivers. This is what makes purchasing power a measurable, defensible property instead of a hope.

**Honest cost.** The anchor commits us to administering a price. If real storage cost falls 15%/year and we do not re-base the tariff, FRC quietly becomes overpriced relative to the resource it names, the storage Sink weakens, and the economy drifts. The re-basing rule in §13 is not optional maintenance — it is the mechanism that keeps the anchor honest.

### 1.3 The design goal as measurable properties

P12 requires falsifiable claims. The economy is "working" if and only if all five hold. Each is emitted as a platform metric and asserted in the simulation harness (§12).

| # | Property | Definition | Target band | Failure meaning |
|---|---|---|---|---|
| M1 | **Sink Coverage Ratio** (SCR) | (burns + net stake locked + reclamations) ÷ emission, trailing 90 days | 0.85 – 1.15 from Y4; ≥ 0.95 steady state | < 0.85 supply overhang; > 1.15 deflationary squeeze |
| M2 | **Basket price stability** | Cost of the Fraction Reference Basket (§1.4) | ±25% over any rolling 12 months | The unit of account is not a unit of account |
| M3 | **Velocity** | Annual Transfer volume ÷ mean circulating supply | 4 – 12 | < 4 hoarding; > 12 churn (investigate wash activity) |
| M4 | **Concentration** | Share of circulating supply held by the top 100 Wallets | < 25% | Capture |
| M5 | **Emission intensity** | Annual emission ÷ circulating supply | < 5% by Y8 | The economy is still a faucet, not a market |

### 1.4 The Fraction Reference Basket (FRB)

A fixed bundle of platform resources, priced every epoch. It is the CPI of this economy.

| Component | Quantity | Y1 unit price | Y1 cost |
|---|---|---|---|
| Vault storage | 100 GB-month | 1.00 FRC | 100.00 FRC |
| Egress bandwidth | 50 GB | 0.10 FRC | 5.00 FRC |
| Compute | 1,000 CU | 0.020 FRC | 20.00 FRC |
| Facet mint (8 KB state) | 1 | 5 + 0.5×8 | 9.00 FRC |
| Premium handle | 1 year | 25.00 FRC | 25.00 FRC |
| **FRB total** | | | **159.00 FRC** |

M2 asserts the FRB stays inside `[119.25, 198.75]` FRC over any rolling 12 months.

---

## 2. Units, Precision, and Rounding

### 2.1 Why 1e-9

The smallest unit is **1 quantum = 1e-9 FRC**. Two independent reasons, both arithmetic.

**Reason 1 — the smallest real payment must carry five significant digits.** The smallest economically meaningful event in the system is one Custodian holding one 256 KB Shard for one hour:

```
  0.28 FRC / replica-GB-month          (custody rate, §3 S1)
    × 0.25 GB                          (256 KB shard)
    ÷ 730 hours/month
  = 0.00009589 FRC  =  95,890 quanta

  worst-case rounding loss = 1 quantum = 1.043e-5 of the payment = 0.0010%
```

At 1e-6 precision this payment would be 96 units and rounding loss would be ~1%, compounding across ~10^9 attestations per year into a visible, unaccountable drift. At 1e-12 we lose the second property:

**Reason 2 — total supply must fit in a signed 64-bit integer with headroom.**

```
  hard supply cap    1,000,000,000 FRC
  × 1e9 quanta/FRC = 1.000e18 quanta
  i64::MAX         = 9.223e18 quanta
  headroom         = 9.22×
```

The entire supply, and any single balance, is a native `i64` on every platform we target — including `wasm32` (`10 §11`), where 128-bit arithmetic is emulated and slow. Intermediate arithmetic uses `i128`; persisted balances are `i64`. The headroom absorbs the `EmissionAccount`'s negative mirror of total supply (`11 §2.6`) and every plausible cap amendment.

### 2.2 The Conservative Rounding Rule

**Invariant E2.** Rounding always moves in the direction that **shrinks circulating supply**. Concretely:

| Operation | Rule | Rationale |
|---|---|---|
| Emission to a Wallet | **Floor** (toward zero) | The protocol never over-emits. Residual quanta are simply not emitted. |
| A charge levied on a payer (fee, tariff, mint cost) | **Ceiling** | The payer never underpays; the sub-quantum difference becomes a Sink. |
| Splitting a whole into shares (fee splits, Treasury division, Fracture) | **Largest-remainder apportionment**, ties broken by ascending `WalletId` | Σ parts == whole, exactly, deterministically. No dust is created or destroyed. |
| Stake release | **Floor** to the staker; remainder burned | Locking is never profitable through rounding. |

**Banker's rounding (round-half-to-even) is rejected.** It is statistically unbiased, which is precisely the problem: unbiased means half of all rounding events *create* Fraction. Fraction created by a rounding rule has no named Source, which violates P12 directly and makes Invariant 4 (`total supply == -EmissionAccount.balance`) false by construction. We prefer a rule that is *biased against us* and provably conservative over one that is elegant and leaky.

### 2.3 No floating point, ever

`f32` and `f64` are forbidden in `fractal-domain-ledger` and `fractal-domain-economy`, enforced by `#![deny(clippy::float_arithmetic)]` plus a CI grep. Consequences that must be designed around, not worked around:

- Ratios and shares are `u32` **parts-per-million** (ppm). `14%` is `140_000`. Multiplication is `(amount as i128 * ppm as i128) / 1_000_000`.
- Exponents and concave transforms (the `^0.6` in §7.3) are computed by **integer-domain fixed-point log/exp tables** with published, pinned lookup tables and a property test asserting monotonicity and determinism across all three target architectures. A non-deterministic Contribution Score is an unreplayable event log (P6).
- Percentages shown in a UI are formatted at the edge, never computed in the domain.

---

## 3. The Source Registry

A **Source** is a named, rate-limited, phase-gated mechanism that emits Fraction. This list is closed. Every `PostingReason` emission variant maps to exactly one row (`11 §7`, Invariant 15). Adding a Source requires an ADR, a simulation run, and a slot in the phase's complexity budget — `02 §5` caps new Sources at **two per phase**, which is exactly why the phase column below reads as it does.

### 3.1 The two Source classes

| Class | Definition | Payment posture | Payout factor |
|---|---|---|---|
| **Class A — Verifiable** | The contribution is cryptographically or computationally provable without a human in the loop (bytes attested, egress receipted, compute re-executed) | **Sink-first**: the Settlement Pool (tariffs collected) pays first; emission covers only the residual shortfall | Typically π ≈ 1.0 — verifiable work is paid nearly in full |
| **Class B — Attested** | The contribution is a human judgement (this post was useful, this moderation was correct) | **Pool-rationed**: a fixed share of the epoch budget is divided among claimants | π ≪ 1 at scale — attested work is rationed, never guaranteed |

This split is the economy's central asymmetry and it is deliberate: **we pay confidently for things we can prove and cautiously for things we must believe.**

### 3.2 Source table

Daily budget references below use the Year-1 schedule (§5): **547,945 FRC/day**.

| # | Source | Pays for | Contribution Score input | Rate / formula | Per-window cap | Anti-farm defense | Share | Phase |
|---|---|---|---|---|---|---|---|---|
| **S1** | **Storage Custody** | Holding Shards for other Societies | `C = Σ_s (bytes_s × attested_hours_s × pass_s)`, `pass ∈ {0,1}` | `0.28 FRC per replica-GB-month`, paid sink-first from the Storage Settlement Pool | No Custodian > **5%** of the epoch's custody pool | Shards assigned by XOR distance `h(shard) ⊕ FNID` — a Custodian cannot choose what it holds; random challenge at λ=4/shard/day; a missed challenge zeroes the epoch for that Shard and slashes 3× the epoch's earnings | 22% | 4 |
| **S2** | **Bandwidth Service** | Serving Shards and media to requesters | `C = Σ bytes in recipient-signed delivery receipts` | `0.10 FRC/GB × congestion(1.0–2.0)`, sink-first | 3% of pool per Node; 500 GB/day/Node absolute | Receipts must be signed by a **distinct paying Principal** with positive Trust; self-service pairs (same Operator on both ends) are excluded by FNID lineage check | 10% | 4 |
| **S3** | **Compute Contribution** | Inference and transcode capacity | `C = Σ CU × verify_pass`, CU normalized to a reference GPU-second | Uniform auction clearing price × CU, sink-first | 5% of pool per provider | 2% of jobs re-executed deterministically against a committed result hash; mismatch slashes the provider's bond and voids the epoch | 8% | 5 |
| **S4** | **Content Creation** | Original Messages, media, and documents judged useful | Quadratic peer-attestation score (§7.3) | `pay = pool × C_a / ΣC`, `C_a = R_a^0.6` | `4 + 4×Level` FRC/day/Citizen (max 24 at L5) | Quadratic breadth-over-depth; relationship saturation; Trust-weighted attesters; Standing gate on attesters; 40% of award vests over 30 days | 14% | 2 |
| **S5** | **Curation** | Organizing, tagging, summarizing, connecting existing work | Same shape as S4, but attestation weight requires the attester to have *consumed* the curated object (read receipt ≥ 30s dwell, locally computed, never transmitted as behavioural data — P9) | `pay = pool × C_a / ΣC` | 8 FRC/day/Citizen | As S4, plus: a curator earns nothing from objects they authored (self-dealing check on `Message.author`) | 6% | 3 |
| **S6** | **Moderation Work** | Reviewing reports, taking Moderation Actions that survive Appeal | `C = Σ_actions (severity_weight × upheld)`, `upheld ∈ {0,1}` resolved at Appeal window close (14 days) | `2.0 FRC × severity_weight`, severity ∈ {1, 2, 5} | 30 FRC/day/Citizen; no Citizen may action > 20% of one Society's reports in an epoch | **Payment is deferred 14 days and paid only on `upheld = 1`.** An overturned action pays zero and costs 1.0 FRC from the actioner's Stake. Reviewers are assigned, never self-selected | 8% | 2 |
| **S7** | **Governance Participation** | Voting and proposal authorship in Charter governance | `C = enacted_proposals_participated`, counted only where the vote was **committed before reveal** | 0.5 FRC per enacted proposal participated in; 3.0 FRC for authoring an enacted proposal | 12 proposals/month/Citizen (max 6 FRC/month base) | Commit-reveal (a vote cast after the outcome is knowable pays nothing); pays only on **enactment**, not on proposal volume; proposal authorship costs the 10 FRC amendment Sink (K12) up front | 5% | 3 |
| **S8** | **Agent Development** | Agents whose actions are adopted and not reverted | `C = Σ (adopted_actions − 3×reverted_actions)`, floored at 0, weighted by the Operator's Trust | `0.05 FRC per net adopted action` | 40 FRC/day per **Operator** (not per Agent) | The cap is per-Operator, so running 100 Agents multiplies cost and not reward; every action carries `envelope_ref` (`10 §5`); a blocked action (`AgentActionBlocked`) counts as −1 | 6% | 5 |
| **S9** | **Extension Development** | Extensions installed and retained | `C = Σ_installs (retained_days ≥ 30) × distinct_societies^0.5` | `1.2 FRC per retained install-society` | 2% of pool per Extension per epoch | Retention gate (30 days) defeats install-churn; `distinct_societies^0.5` defeats one Operator installing into 500 shell Societies; Societies must be ≥ Level 1 with ≥ 5 members | 8% | 6 |
| **S10** | **Onboarding & Vouching** | Bringing Citizens who become real participants | `C = Σ_invitees [ Level(invitee) ≥ 2 ∧ tenure ≥ 90d ∧ Trust > 0 ]` | 5.0 FRC per qualifying invitee, paid once at day 90 | 20 qualifying invitees/year/Citizen (100 FRC/yr) | Voucher stakes **50 FRC** per vouch, locked 90 days, slashed in full if the invitee is confirmed Sybil; payment gated on a 90-day, Level-2, positive-Trust outcome the attacker must actually manufacture | 4% | 6 |
| — | **Unallocated headroom** | — | — | Drawn pro-rata by any Source whose verified claim exceeds its share, to a **hard ceiling of 1.5× that Source's share**; otherwise **forfeited** | — | Rule-based, not discretionary — no human or Agent can direct it (P12: no discretionary emission) | 9% | — |

**Total allocated: 91%. Headroom: 9%. Sum: 100%.**

### 3.3 Sink-first settlement, worked

This is the mechanism that makes Class A emission *self-extinguishing* as the platform matures. Month 12 of the 10-year model, storage only:

```
  Citizens 50,000 × 5 GB free              =   250,000 GB
  Societies 4,000 × 25 GB free             =   100,000 GB
  Paid storage above quota                 =   120,000 GB
  ───────────────────────────────────────────────────────
  Logical storage                          =   470,000 GB
  × replication factor 3                   = 1,410,000 replica-GB

  Custodian owed  = 1,410,000 × 0.28 FRC   =   394,800 FRC / month
  Settlement Pool = 120,000 × 1.00 FRC     =   120,000 FRC / month   (K8, recycled)
  ───────────────────────────────────────────────────────
  EMISSION (residual only)                 =   274,800 FRC / month
                                           =     9,041 FRC / day

  S1 daily share ceiling (Y1)              =   120,548 FRC / day
  Utilization                              =         7.5%
  FORFEITED, permanently                   =   111,507 FRC / day
```

Read the last line carefully. The schedule is a **ceiling that is mostly not reached**, and unreached budget is destroyed, not banked. When paid storage grows past the free quota — which is the definition of the platform succeeding — the Settlement Pool covers the Custodian bill outright and storage emission converges to **zero without any parameter change**.

---

## 4. The Sink Registry

A **Sink** removes Fraction from a Wallet. Three dispositions, and the distinction matters enormously:

- **Burn** — posted to the `BurnAccount`. Permanently removes supply. Reverses emission.
- **Redistribute** — moves to another Wallet (a Treasury, a Settlement Pool, a harmed party). Supply is unchanged; only ownership moves. **A redistribution is not an anti-inflation mechanism** and is never counted in SCR.
- **Reclaim** — returns to the Emission Reserve, reducing *future* emission budget rather than current supply.

| # | Sink | Trigger event | Amount | Disposition | Phase |
|---|---|---|---|---|---|
| K1 | Society creation | `SocietyCreated` | **0 FRC** for a Citizen's first Society at Founder tier and for any Crystallization; **250 FRC** for each additional | 100% burn | 2 |
| K2 | Chamber beyond quota | `ChamberCreated` above the Society Level quota (`11 §2.3`) | 40 FRC | 100% burn | 2 |
| K3 | Society tier upgrade | ceiling purchase (members / Chambers / Vault) | 120 / 480 / 1,200 / 2,400 FRC by tier | 100% burn | 3 |
| K4 | Premium handle reservation | `HandleClaimed` where length ≤ 5 | 25 FRC/year | 100% burn | 2 |
| K5 | Facet mint | `FacetMinted` | `5 FRC + 0.5 FRC per KB of state` | 70% burn / 30% Society Treasury | 3 |
| K6 | Facet schema evolution | `FacetEvolved` where the schema changes | 1 FRC | 100% burn | 3 |
| K7 | Marketplace fee | `PurchaseCompleted` | **12% of gross**; 10% for C9 Services; 4% launch rate on a verified creator's first 10,000 FRC lifetime gross (`19 §6.1`) | **50.0% burn / 33.3% Operations Account / 16.7% Assurance Reserve** — i.e. `19 §6.1`'s 6 / 4 / 2 pp of gross | 6 |
| K7b | Society Shelf share | `PurchaseCompleted` on a Shelf-originated sale | 0–10% of gross, default 0, hard-capped at 10% by the hosting Society's Charter (`19 §6.1`) | **Redistribute** to the hosting Treasury. Supply is unchanged; **never counted in the Sink Coverage Ratio** (§4) | 6 |
| K8 | Storage tariff | monthly Vault settlement above quota | 1.00 FRC per GB-month | → Storage Settlement Pool; **surplus above Custodian owed is burned** | 4 |
| K9 | Bandwidth tariff | egress settlement above quota | 0.10 FRC/GB × congestion(1.0–2.0) | → Bandwidth Settlement Pool; surplus burned | 4 |
| K10 | Compute purchase | auction clear | clearing price × CU | 90% → provider; **10% of clear burned** | 5 |
| K11 | Priority service | expedited transcode / restore / compute slot | 2 FRC per job; 0.5 FRC/GB restore; 1.5× clearing price for a priority slot | premium portion 100% burn | 3 |
| K11b | In-Society Chamber pin | `MessagePinned` beyond the free pin quota | 5 FRC / 24h | 100% burn | 3 |
| K12 | Charter amendment | `CharterEnacted` | 10 FRC | 100% burn | 3 |
| K13 | Stake slash | `StakeSlashed` | the slashed amount | 50% burn / 50% harmed-party compensation | 4 |
| K14 | **Transfer fee** | — | **0 FRC** | — (see §4.1) | — |
| K15 | Dormant Treasury reclamation | Society `Dormant` ≥ 24 months (`11 §4`) | 100% of Treasury balance | **Reclaim** to Emission Reserve; fully claimable for 12 months on reactivation | 5 |
| K16 | Vesting forfeiture | Membership departs in bad standing, or Trust collapses, with unvested emission outstanding | unvested remainder | 100% burn | 4 |
| K17 | Extension listing bond | `ListingPublished` | 100 FRC, refundable at delisting | locked (net supply reduction while locked); slashed on confirmed fraud | 6 |

### 4.1 Why there is no transfer fee

A transfer fee taxes velocity — the exact behaviour M3 wants. It also falls hardest on the smallest transfers, which are the ones that make peer settlement interesting.

**Decision: zero transfer fee.** Spam is controlled by per-Principal token buckets at the gateway (`10 §10`), not by price.

**Honest cost, stated plainly:** free transfers make wash-trading loops cost nothing. We accept this because we have removed the *reason* to wash: **no Source pays for transfer volume, sale volume, or balance.** Wash trading in this economy buys nothing but a bigger number in a ledger nobody scores. The defense is the absence of the incentive, not the presence of a toll.

### 4.2 Why paid discovery placement does not exist

The obvious high-volume Sink for a social platform is paid reach: pay FRC to appear in more people's discovery results. It would be the single largest burn in this table.

**Rejected.** Paid ranking is advertising, and advertising and engagement-optimized ranking are on the Never list (`02 §4`, serving P9 and P12). We do not get to reintroduce it because the sink math is convenient. K11 (priority *service*) and K11b (in-Society pin) survive because they price **scarce compute and a Society's own attention surface**, not other Citizens' attention. The revenue we forgo is real; the principle is not negotiable.

---

## 5. The Emission Model

### 5.1 Supply policy

**Chosen: a hard lifetime cap with a geometric decay schedule, gated by productivity coupling, with no carry-forward.**

Three rules, all simultaneously binding:

```
  R1  CAP        Cumulative lifetime emission ≤ 1,000,000,000 FRC. Immutable (§13 Tier 0).
  R2  SCHEDULE   Year-n scheduled budget  B(n) = 200,000,000 × 0.80^(n-1) FRC
                 Σ B(n) for n → ∞  =  200,000,000 / 0.20  =  1,000,000,000 FRC  ✔ consistent with R1
                 Emission ceases entirely when B(n) < 100,000 FRC  (year 36).
  R3  COUPLING   Actual epoch emission = Σ_sources min( share_s × B(n)/365 × 1.5 ,  verified_claim_s )
                 Unclaimed budget is FORFEITED. There is no carry-forward, no reserve accumulation,
                 no catch-up, and no discretionary release. (P12)
```

**Why this and not the alternatives.**

*A fixed pre-mined supply with no emission* cannot pay Custodians, which is the one thing the token exists to do. It also concentrates the entire supply in whoever holds it at genesis — pay-to-win by wealth, `02 §4`.

*Pure productivity coupling with no schedule* (emit whatever verified work claims) sounds maximally honest and is the trap. Verified work volume is a function of how much hardware someone points at the network. Without a ceiling, a well-capitalized actor sets the emission rate. The schedule is what makes the supply knowable in advance to a Citizen deciding whether to participate.

*Pure schedule with no coupling* emits the full budget regardless of whether anything was delivered. That is a faucet, and faucets are farmed.

**Requiring both** means: emission is bounded above by a published curve that nobody can raise, and bounded below by whether real work happened. The forfeiture rule is what removes the overhang — an unspent budget that accumulates is a future dump wearing a cap.

### 5.2 The schedule, ten years

```
FRC/yr (millions)
 200 ┤██████████████████████████████ 200.0                             ← scheduled ceiling
 180 ┤
 160 ┤████████████████████████ 160.0
 140 ┤
 128 ┤███████████████████ 128.0
 120 ┤
 102 ┤███████████████ 102.4
  82 ┤████████████ 81.9
  66 ┤█████████ 65.5
  52 ┤███████ 52.4
  42 ┤██████ 41.9
  34 ┤█████ 33.6
  27 ┤████ 26.8
     └──┬────┬────┬────┬────┬────┬────┬────┬────┬────┬──
       Y1   Y2   Y3   Y4   Y5   Y6   Y7   Y8   Y9  Y10

  ░░ = ACTUAL emission (productivity-coupled), overlaid:
 200 ┤░░ 3.3   ← Y1: only S4–S7 live (Phase 2–3). 1.65% of ceiling.
 160 ┤░░░░ 31.0
 128 ┤░░░░░░░░░░░░░░░░░ 118.0
 102 ┤███████████████ 102.4  ← Y4 onward the CEILING BINDS. Claim ≫ budget.
```

| Year | Scheduled `B(n)` | Cumulative scheduled | Uncapped claim | **Actual emission** | Aggregate π̄ | Sinks (burn+lock+reclaim) | SCR | Circulating supply, end |
|---|---|---|---|---|---|---|---|---|
| Y1 | 200,000,000 | 200,000,000 | 3.3 M | **3.3 M** | 1.00 | 1.2 M | 0.36 | 2.1 M |
| Y2 | 160,000,000 | 360,000,000 | 31 M | **31.0 M** | 1.00 | 9 M | 0.29 | 24.1 M |
| Y3 | 128,000,000 | 488,000,000 | 118 M | **118.0 M** | 1.00 | 48 M | 0.41 | 94.1 M |
| Y4 | 102,400,000 | 590,400,000 | 268 M | **102.4 M** | 0.382 | 89 M | 0.87 | 107.5 M |
| Y5 | 81,920,000 | 672,320,000 | 470 M | **81.9 M** | 0.174 | 80 M | 0.98 | 109.4 M |
| Y6 | 65,536,000 | 737,856,000 | 690 M | **65.5 M** | 0.095 | 66 M | 1.01 | 108.9 M |
| Y7 | 52,428,800 | 790,284,800 | 845 M | **52.4 M** | 0.062 | 53 M | 1.01 | 108.3 M |
| Y8 | 41,943,040 | 832,227,840 | 975 M | **41.9 M** | 0.043 | 43 M | 1.03 | 107.2 M |
| Y9 | 33,554,432 | 865,782,272 | 1,085 M | **33.6 M** | 0.031 | 35 M | 1.04 | 105.8 M |
| Y10 | 26,843,546 | 892,625,818 | 1,220 M | **26.8 M** | 0.022 | 28 M | 1.04 | 104.6 M |

```
  Cumulative emitted, Y1–Y10        =  556.8 M FRC
  Cumulative scheduled, Y1–Y10      =  892.6 M FRC
  PERMANENTLY FORFEITED             =  335.8 M FRC   (37.6% of the schedule, never created)
  Cumulative removed (burn+lock)    =  452.2 M FRC
  Circulating supply, end of Y10    =  104.6 M FRC
  Peak circulating supply           =  109.4 M FRC   (Y5)
```

Circulating supply **peaks in Year 5 and declines thereafter**. That is the shape a bounded economy is supposed to have, and it is produced by the interaction of a decaying ceiling with sinks that scale with population — not by anyone's discretion.

### 5.3 When contribution outgrows the budget

From Y4 the ceiling binds, and the payout factor collapses:

```
  π_s(epoch)  =  min(1, budget_s / Σ_a C_a)         per Source s

  Y3   claim 118 M  vs  budget 128 M   →  π̄ = 1.000
  Y4   claim 268 M  vs  budget 102.4 M →  π̄ = 0.382
  Y5   claim 470 M  vs  budget  81.9 M →  π̄ = 0.174
  Y10  claim 1,220M vs  budget  26.8 M →  π̄ = 0.022
```

A Citizen performing *identical work* earns **45× less nominal FRC in Y10 than in Y3**. This must be stated to Citizens plainly and early, because pretending otherwise is how token economies acquire an angry cohort.

Three things make it survivable, and only the first two are ours to control:

1. **Class A work is not diluted the same way.** Verifiable Sources are sink-first and paid near-fully (π ≈ 1.0); the collapse falls on Class B attested work. In Y5, resource Sources claimed ~34 M and were paid 34 M (π = 1.0), while attested Sources claimed ~436 M and were paid ~47.9 M (π ≈ 0.11). Custodians are running a business; creators are sharing a prize pool. Those are different promises and the UI must not present them identically.
2. **Purchasing power rises as the tariff re-bases.** Under the §13 re-basing rule the storage tariff falls with measured reference cost, roughly 15%/yr, so 1 FRC buys ~5× more storage in Y10 than Y1. Net real decline for identical attested work is therefore ~9×, not 45×.
3. **Income shifts from emission to exchange.** This is the intended maturation.

| Year | Share of Citizen FRC income from **emission** | from **exchange** (marketplace, services, Society payments) |
|---|---|---|
| Y2 | 92% | 8% |
| Y4 | 61% | 39% |
| Y5 | 48% | 52% |
| Y7 | 28% | 72% |
| Y10 | 14% | 86% |

**The falsification test for this claim:** if the exchange share has not crossed 50% by the end of Y5 in production telemetry, the economy has failed to become an economy and is still a subsidy program. That triggers the §15.2 fallback review.

### 5.4 The structural reason the P12 test passes

P12's falsification test demands bounded circulating supply under **100× adversarial farming**. Note what the model above makes true:

> From the epoch the ceiling binds, emission is a **fixed pool divided among claims**. Adversarial volume cannot increase the pool. A 100× farm does not inflate supply by one quantum — it can only *steal share* from honest claimants.

This converts the hardest problem in token design (bounded supply under attack) into a much more tractable one (bounded *share capture* under attack), which §7 and §8 solve with caps, quadratic aggregation, and negative-margin attack economics. Before the ceiling binds (Y1–Y3), supply is bounded by the schedule anyway. There is no window in which farming inflates the currency.

---

## 6. The Balance Equation

### 6.1 Statement

```
  ΔS(t)  =  E(t)  −  B(t)  −  L(t)  −  R(t)

     S    circulating supply
     E    emission        (§3, bounded by §5)
     B    burns           (Sinks with burn disposition)
     L    net stake locked (removed from circulation while locked; returns on release)
     R    reclamations    (K15 → Emission Reserve)

  Redistributions do NOT appear. Moving Fraction between Wallets changes nothing about supply.
  This is the single most common modelling error in token design and it is excluded by construction.

  Steady state:   E  =  B + ΔL + R      ⟺      SCR = 1.00
```

```
      SOURCES                                          SINKS
  ┌──────────────┐                                ┌──────────────┐
  │ Emission     │                                │ Burn         │──► supply destroyed
  │ Account      │──► emission ──┐         ┌──────│ Stake lock   │──► supply immobilized
  │ (negative    │               │         │      │ Reclaim      │──► future budget reduced
  │  mirror of   │               ▼         │      └──────────────┘
  │  total supply│        ╔═══════════════════╗          ▲
  └──────────────┘        ║   CIRCULATING     ║──────────┘
         ▲                ║     SUPPLY  S     ║
         │                ╚═══════════════════╝
         │                        │   ▲
         │  reclaim (K15)         │   │  redistribution — internal, supply-neutral
         └────────────────────────┘   └─────────── Treasuries ◄──► Wallets ◄──► Pools
```

### 6.2 Three scenarios, with arithmetic

**Scenario U — Undershoot (sinks ≪ sources).** Our own Y2: E = 31.0 M, B+L+R = 9.0 M, SCR = 0.29.

```
  ΔS = 31.0 − 9.0 = +22.0 M on a mean base of 13.1 M  →  +168% supply growth
```

Because tariffs are administered, the first symptom is *not* a visible price rise — it is an **overhang**: a growing pile of FRC whose holders have nothing they need to spend it on. Second-order effects: hoarding (M3 velocity falls below 4), concentration (M4 rises as early contributors accumulate), and a latent dump risk at the moment external liquidity ever appears. Undershoot in the first three years is **expected and accepted** — a young economy must distribute before it can circulate. It becomes a defect if SCR has not reached 0.85 by end of Y4.

**Scenario O — Overshoot (sinks ≫ sources).** Suppose Y6 with fees unchanged but a 40% population contraction: E = 40 M (claims fall), B+L+R = 64 M, SCR = 1.60.

```
  ΔS = 40 − 64 = −24 M on a base of 108.9 M  →  −22% supply in one year
```

Symptoms: a new Citizen needs 250 FRC to found a second Society and can now earn it in weeks rather than days; Societies defer storage purchases to conserve Treasury; velocity rises briefly (panic spending) then collapses (hoarding what remains). This is the worse failure, because deflation punishes participation and the punishment compounds.

**The circuit breaker (rule-based, published in advance, not discretionary):**

```
  IF  SCR > 1.35 for two consecutive monthly epochs
  THEN all Tier-1 fixed fees (K1, K2, K3, K4, K5, K11, K11b, K12) reduce by 10%,
       automatically, at the next epoch boundary, to a floor of 40% of their base value.
  IF  SCR < 0.60 for four consecutive monthly epochs
  THEN the same fees increase by 10%, to a ceiling of 200% of base.
  Both directions log a `EconomicParameterAdjusted` event with the triggering measurement.
```

The breaker is asymmetric on purpose — fast to relieve deflation (2 epochs), slow to tighten (4 epochs). Raising costs on Citizens is the change that most deserves deliberation.

**Scenario B — Balance.** Y6 as modelled: E = 65.5 M, B+L+R = 66.0 M, SCR = 1.01, ΔS = −0.5 M on 108.9 M (−0.5%). Velocity 8.3. FRB at 151 FRC (−5.0% vs Y1's 159, inside the ±25% band). All five M-metrics green.

### 6.3 Control levers

| Lever | Direction | Max change per adjustment | Latency to effect | Side effect to accept |
|---|---|---|---|---|
| Source share vector (§3.2) | ± emission | ±20% relative, one change per parameter per 90 days | 1 epoch | Redistributes between contributor classes; creates winners and losers immediately |
| Schedule decay factor `0.80` | ± emission | **Tier 0 — immutable** | — | — |
| Per-window caps (S1–S10) | ± emission | ±25% | 1 epoch | Tightening caps hits the most productive contributors first |
| Fixed fee schedule (K1–K12) | ± sinks | ±10% per breaker step, ±20% by proposal | Immediate | Directly changes the cost of participating |
| Storage tariff (the anchor) | ± sinks, ± FRB | ±15% per year, indexed (§13) | 1 month | Moves the unit of account itself — the most consequential lever in the table |
| Free quotas (5 GB / 25 GB) | ∓ sinks | ±20% | 1 month | The most humane lever and the most expensive |
| Burn/redistribute split on K5, K7, K13 | ± burns | ±20 percentage points | 1 epoch | Shifts value from Society Treasuries to supply reduction |
| Vesting fraction (40%/30d) | ± effective supply | ±15 pp | 30 days | Long vesting reads as a lock-up and depresses participation |

**Governance process for pulling a lever:** §13. Every adjustment requires a published simulation run showing the projected 24-month M1–M5 trajectory under the change, a 30-day notice period, and a human signature (P4). No Agent may propose or enact an economic parameter change.

---

## 7. Contribution Metrics

P12 requires metrics that are "defined, measurable, and resistant to being farmed." Those are three separate properties and most systems achieve one.

### 7.1 The classification

| Class | Examples | Verification | Paid how |
|---|---|---|---|
| **Verifiable** | bytes attested, egress receipted, compute re-executed, uptime | Cryptographic proof or deterministic re-execution. No human judgement. | Directly, near-fully, sink-first |
| **Gameable but measurable** | messages posted, reactions received, follower count, session time | Counting is trivial; the count means nothing | **Never paid directly. Not an input to any Source.** May inform XP (`18`), which buys no Fraction. |
| **Judgemental** | "this was useful", "this moderation was correct" | Peer attestation, adversarially aggregated | Pool-rationed via §7.3, with §7.4 defenses |

**Invariant E3.** No raw activity count — message count, reaction count, follower count, dwell time, session length — may appear as a term in any Source formula. Enforced by a schema check on the `ContributionScored` event: every input field must be declared `Verifiable` or `Attested` in the Source registry.

### 7.2 Verifiable metrics

```
  STORAGE   C_storage(n, epoch) = Σ_{s ∈ Shards(n)} bytes_s · hours_s · pass_s · need_s
            pass_s ∈ {0,1}  — 1 iff every random challenge in the epoch was answered
                              within the SLA. Challenge rate λ = 4/shard/day (Poisson).
            need_s = 1 if live_replicas(s) ≤ target; 0.25 if over-replicated.
                     (We pay for redundancy we asked for, not redundancy someone chose.)

            P(evade | withholding fraction f) = e^(−λ·f·epoch_days)
            Withholding 10% of an epoch:  e^(−4·0.1·30) = e^(−12) = 6.1e−6
            Detection is effectively certain, and detection slashes 3× epoch earnings.

  BANDWIDTH C_bw(n, epoch) = Σ_r bytes_r · [signer(r) ≠ n] · [lineage(signer) ≠ lineage(n)]
            Receipts are signed by the *recipient*. Self-service is excluded by Operator
            lineage, not by heuristics.

  COMPUTE   C_cu(n, epoch) = Σ_j CU_j · verify_j
            verify_j = 1 unless j was sampled (p = 0.02) and the recomputed result hash
            differs from the committed hash, in which case the provider's bond is slashed
            and the entire epoch is voided.
            Expected cost of cheating on a fraction q of jobs:
              P(caught) = 1 − (1−0.02)^(q·N).  At q·N = 50 jobs: 1 − 0.98^50 = 63.6%.
              At q·N = 200: 98.2%. Voiding the epoch makes any positive q strictly negative EV.
```

### 7.3 Attested metrics: quadratic aggregation

For Sources S4, S5, and the judgemental part of S6:

```
  Raw signal          R_a = ( Σ_{j ∈ D(a)} sqrt(v_j) )²
  Diminishing return  C_a = R_a^0.6
  Award               pay_a = min( cap(Level_a),  pool · C_a / Σ_b C_b )

  where v_j, the weight of attester j on subject a, is:

      v_j = clamp(Trust_j, 0, T_max) · G_j · 1/(1 + k·A_ja)

      G_j  ∈ {0,1}   Standing gate: Level ≥ 2 ∧ tenure ≥ 30d ∧ Trust > 0
      A_ja           count of j's prior attestations to a in the trailing 30 days
      k    = 0.5     relationship saturation constant
      T_max = 4      no single attester can be worth more than four ordinary ones
```

**Why quadratic.** Breadth beats depth by construction:

```
  10 distinct attesters, v = 1 each  →  R = (10 · 1)²   = 100
   1 attester, v = 10                →  R = (√10)²      =  10
  Breadth is worth 10× depth for the same total weight.
```

**Why relationship saturation.** A collusion ring of size n attesting each other daily decays fast:

```
  Effective value of j's m-th attestation to a:  1/(1 + 0.5·m)
  Σ_{m=0}^{29} 1/(1+0.5m) = 2·(H_31 − H_1) = 2·(4.0273 − 1) = 6.05

  30 mutual attestations over 30 days are worth 6.05, not 30 — a 5.0× haircut.
  A ring must keep recruiting *new, gated, Trust-positive* identities to sustain output.
  That is the expensive thing, and it is exactly what we made expensive (§8).
```

**Why the `^0.6` concave transform.** Quadratic aggregation alone is superlinear in attester count, which rewards a coordinated bloc. Composing with `x^0.6` makes the end-to-end response to attester count `n^1.2` — mildly superlinear (breadth still pays) rather than quadratic (blocs dominate). The exponent is a Tier-1 parameter with a ±0.1 adjustment bound.

### 7.4 The four gates every attested award passes

```
  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
  │ 1. STANDING  │──►│ 2. QUADRATIC │──►│ 3. PER-LEVEL │──►│ 4. VESTING   │
  │    GATE      │   │  AGGREGATION │   │     CAP      │   │              │
  │ attester must│   │ breadth ≫    │   │ 4+4×Level    │   │ 60% now,     │
  │ be L≥2, 30d, │   │ depth; ring  │   │ FRC/day; and │   │ 40% over 30d;│
  │ Trust > 0    │   │ saturation   │   │ no principal │   │ forfeit (K16)│
  │              │   │              │   │ > 0.5% of an │   │ on bad-faith │
  │              │   │              │   │ epoch's total│   │ departure    │
  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘
```

Gate 3's absolute clause — **no Principal may receive more than 0.5% of any epoch's total emission** — is the backstop that holds even if every other gate is defeated by an attack we did not anticipate. It is asserted as invariant I-E4 in the simulation harness.

---

## 8. Anti-Abuse and Sybil Economics

The test is not "can this be gamed" — everything can. The test is **whether the attacker's expected margin is negative**. Each row below computes it.

Assumptions used throughout: detection probability `P_d` is the harness-measured value for that attack class over a 30-day window; slashed stake is lost in full; time cost is priced at 0 (the attacker is assumed patient and automated, which is the conservative assumption).

| Attack | Mechanism | Cost to attack (30d) | Reward from attack (30d) | Margin | Why it fails |
|---|---|---|---|---|---|
| **Sybil content farm** | 100 fake Citizens attesting one earner | 100 vouch stakes × 50 FRC = **5,000 FRC** at risk + 100 × 30d aging + 100 device attestations | Capped at 12 FRC/day (L2) × 30 = **360 FRC**, of which 40% unvested | `+360 − 0.90×5,000 = −4,140 FRC` | Attestation weight requires a Standing gate the attacker must *buy* with real stake; saturation kills the ring by ~day 6; `P_d ≈ 0.90` |
| **Self-dealing storage** | Store own data, custody own Shards | Tariff 1.00 FRC/GB-month paid; self-assignment probability = attacker's share of custodian capacity | 0.28 FRC per replica-GB-month, only on Shards the XOR-distance assignment happens to give them | At 0.1% capacity share: earn 1 FRC requires paying **3.57 FRC** → `−2.57 per FRC earned` | Custodians cannot choose their Shards. The margin is negative *by construction*, at every capacity share below 100% |
| **Wash trading** | Fake marketplace sales between controlled Wallets | 5% fee on GMV, 40% burned. 1,000 FRC washed = **50 FRC** cost, 20 burned | **0 FRC** — no Source pays for sale volume, GMV, or balance | `−50 per 1,000 FRC washed` | The incentive does not exist. Removing the reason beats policing the behaviour |
| **Collusion ring (curation)** | 10 Citizens cross-attesting daily | 10 gated identities (real cost: 10 × 90d + Trust maintenance) | 30 mutual attestations yield 6.05 effective, ÷ concave transform, ÷ per-day cap | Yield is **5.0× lower** than naive; per-capita output falls below honest baseline | Saturation + quadratic breadth requirement + `^0.6` |
| **Engagement farming** | High-volume posting/reacting | Free | **0 FRC** — activity counts are not Source inputs (Invariant E3) | `0` reward for any cost | Structural: the metric is not in the formula |
| **Agent-run farm** | Operator runs 200 Agents to multiply S8 income | Model inference cost for 200 Agents, real and continuous | Capped at **40 FRC/day per Operator** regardless of Agent count | Negative at any nontrivial inference cost | The cap is on the accountable human (`01 §2`: every Agent has exactly one Operator) |
| **Reward stripping** | Earn, extract, abandon in bad standing | Requires genuine contribution to earn at all | 60% immediate; **40% forfeited** (K16) on bad-faith departure | Strips at most 60% of an award that required real work | Vesting converts hit-and-run into a 40% haircut on genuine value delivered — an acceptable outcome |
| **Vouch farm** | Vouch 100 fakes for S10 payouts | 100 × 50 FRC stake = **5,000 FRC** locked 90 days | 5 FRC × 100, paid **only** at day 90 with Level ≥ 2 ∧ Trust > 0 = at most **500 FRC**, and only if each fake independently earns Level 2 | `+500 − 5,000 = −4,500` before detection | Payment is gated on the invitee doing 90 days of real, Trust-positive work — i.e. on not being a Sybil |
| **Moderation farm** | Mass-action reports for S6 | 1.0 FRC Stake cost per overturned action | Paid only on `upheld` after a 14-day Appeal window | Aggressive over-actioning is net negative as soon as the overturn rate exceeds 33% | Deferred, outcome-conditional payment; reviewers assigned not self-selected |
| **Governance spam** | Flood proposals for S7 | 10 FRC per amendment (K12), paid at submission | 3.0 FRC, only on **enactment** | `−10 + 3.0·P(enact)`; negative unless `P(enact) > 0.33` | An attacker who enacts a third of their proposals is a governance participant, not an attacker |

**The one honest gap.** A well-resourced attacker who is willing to make 100 accounts *genuinely* pass the Standing gate — 90 days of real activity, real Trust, real vouches — can capture attested-Source share. We do not claim to prevent this. We claim it costs more than it yields (the margins above), that it is indistinguishable from 100 people participating, and that the 0.5%-of-epoch backstop caps the damage. If an actor pays that price to look exactly like a healthy cohort, the economy has been attacked by contribution.

---

## 9. Anti-Inflation Mechanisms

| Mechanism | How it bounds supply | Strength |
|---|---|---|
| **Hard cap (R1)** | 1e9 FRC lifetime, Tier 0 immutable | Absolute |
| **Geometric decay (R2)** | 20%/yr; year-36 termination | Absolute |
| **Productivity coupling (R3)** | Emission ≤ verified claim | Strong; ties supply to delivered value |
| **No carry-forward** | 335.8 M FRC forfeited in the 10-year model | Strong; eliminates overhang and dump risk |
| **Sink-first Class A settlement** | Storage/bandwidth/compute emission → 0 as paid usage grows | Self-extinguishing |
| **Burn Sinks** | K1–K7, K10–K12, K13, K16: 452 M FRC removed over 10 years | Primary steady-state mechanism |
| **Velocity sinks** | Recurring consumption (storage, compute, egress) prices holding against using | Moderate; scales with real usage |
| **Stake locking** | S10 vouches, K17 bonds, Custodian bonds immobilize supply | Moderate; reversible by design |
| **Treasury reclamation (K15)** | Dormant Society Treasuries reduce future emission | Small but structurally important |
| **Circuit breaker (§6.2)** | Rule-based fee adjustment on SCR excursions | Corrective, not preventive |

### 9.1 The demurrage question

Demurrage — a decay on idle balances — is the classical cure for hoarding and it directly targets our M3 risk.

**Decision: rejected for Citizen and Agent Wallets. Adopted narrowly for dormant Society Treasuries (K15), as reclamation rather than decay.**

Reasoning, honestly:

- Demurrage is **confiscatory**, and confiscating from the Citizens most invested in the platform to fix a metric is not a trade we will make.
- It is **regressive**: a 2%/month decay is a rounding error to a large holder and a real loss to a Citizen with 40 FRC.
- It creates **"use it or lose it" pressure**, which is a dark pattern by another name and is on the Never list (`02 §4`).
- It substitutes for the harder correct work: if velocity is low, the answer is that **there is not enough worth buying**, and the fix is supply-side (better marketplace, cheaper compute, more useful Extensions), not a tax on patience.

K15 is a different thing and survives because a Society that has emitted no events for 24 months has no owner making a decision, the funds are recoverable for a further 12 months on reactivation, and the disposition is *reclaim* (reducing future emission) rather than *burn* — so nobody's value is destroyed, only deferred.

**If M3 velocity sits below 4 for four consecutive quarters,** the response is, in order: (1) audit what Citizens *cannot* buy; (2) reduce fixed fees via the breaker; (3) expand the marketplace surface. Demurrage may be reconsidered only by an ADR overturning this section.

---

## 10. Market Dynamics

Different goods need different price discovery. Using one mechanism everywhere is the mistake.

| Market | Mechanism | Why this one | Rejected here |
|---|---|---|---|
| **Vault storage** | **Administered fixed tariff**, re-based annually by rule (§13) | It *is* the anchor (E1). A floating price would make the unit of account float against itself. Predictability is the product. | Auction (destabilizes the anchor); AMM (implies a pool and a quoted price — exchange-shaped) |
| **Bandwidth** | Fixed tariff × **rule-computed congestion multiplier** (1.0–2.0, from measured regional saturation) | Egress cost genuinely varies by time and region; a published function of a measured quantity gives responsiveness without bidding | Real-time auction (latency-sensitive traffic cannot wait for a clear) |
| **GPU-class compute** | **Sealed-bid uniform-price auction**, 5-minute slots; all winners pay the lowest accepted bid | Genuinely scarce, genuinely heterogeneous, bursty. Uniform pricing makes truthful bidding near-dominant, so the clearing price is informative | Pay-as-bid (rewards bid-shading, price becomes noise); fixed tariff (cannot allocate scarcity) |
| **CPU-class compute** | Fixed tariff | Abundant; auction overhead exceeds the value cleared | — |
| **Extensions, services** | **Seller-set price**, public price history, no bidding | The seller knows their cost; buyers need comparability, not a market microstructure | Order book, AMM |
| **Facet secondary sale** | **Fixed price or timed ascending auction with anti-sniping extension** (any bid in the final 5 minutes extends by 5 minutes), seller's choice | Unique goods; ascending auctions are well-understood and sniping-resistant with the extension rule | AMM/bonding curve (manufactures a price where none exists and invites speculation) |

### 10.1 Why this is not an exchange

Stated as an engineering position, **not as legal advice**. Competent counsel is required in every target jurisdiction before Phase 9 and before any public statement on this topic.

The architectural facts we hold true while FRC is internal-only:

1. **There is no FRC trading pair.** FRC is never quoted against another asset, internal or external. Every market above prices a *consumable resource or good* in FRC. There is no venue where FRC is the thing being bought.
2. **FRC is not redeemable.** There is no path — none in the code, not merely none exposed (`00 §4` posture) — by which FRC becomes fiat or an external asset. The `Rail` port has exactly one implementation (`10 §7`) and no other is compiled.
3. **FRC is not transferable off-platform.** No bridge, no withdrawal, no bearer instrument.
4. **The platform quotes no external price and operates no order book, AMM, or price oracle referencing an external market.**
5. **No profit expectation is promised anywhere**, in product copy, documentation, or marketing. FRC is described consistently as a unit of account for platform resources.

Facts 1–5 are the position. They are also the reason §14's preconditions are conservative: every one of them is a fact we would be *giving up*.

---

## 11. Wallet UX Requirements (derived from the economics)

These are requirements on `31`/`32`, stated here because they follow from economic facts rather than design taste.

| Requirement | Economic origin |
|---|---|
| **Pending vs Settled is always visible.** Balances render as `settled` with a separate `pending` line. No optimistic write, ever (`10 §6`). | Wallet writes are server-authoritative. Showing a speculative balance as real is lying about money. |
| **Locked is a third number, never folded into balance.** `available = balance − locked`. | `11 §2.6` invariant `balance >= locked >= 0`. Stakes and bonds are not spendable and must not look spendable. |
| **Accrual is ambient.** A slow-updating counter in the Wallet surface. **No** celebration animation, **no** daily-claim button, **no** streak, **no** countdown, **no** push notification per accrual. One settlement summary per epoch, on the Wallet surface only. | `02 §4` bans dark patterns; P12 bans mechanics whose primary function is manufacturing engagement. A reward system that shouts becomes the product. |
| **Every Posting has a receipt.** Named Source or Sink, the formula inputs used, the epoch, the `π` applied, and the causing event id. Reachable in GUI and as `fn wallet receipt <posting_id>`. | P12's "named source and named sink" is unverifiable by a Citizen without this. P13 requires CLI parity. |
| **Vesting is shown as a schedule, not a surprise.** Unvested amounts appear with their release dates and forfeiture conditions from the moment they are awarded. | K16 forfeits value. Undisclosed forfeiture is a dark pattern. |
| **π is disclosed.** When a Source's payout factor is below 1.0, the receipt states the claim, the budget, and the resulting factor. | §5.3's dilution is the single most likely source of contributor anger. It is defensible; concealment is not. |
| **Society Treasury has a monthly Economic Statement.** Sources, Sinks, net, per-Chamber cost attribution, and a diff against the prior month. Signed, exportable, CLI-reachable. | A Society governs a Treasury under its Charter. Governing without a statement of accounts is theatre. |
| **Transfers show the destination's identity, not just its `WalletId`.** | Address-substitution is the oldest attack in payments. |
| **Offline shows last-known-good with an explicit staleness stamp.** | P2's falsification test names the Wallet surface specifically. |

---

## 12. The Simulation Harness

P12's falsification test is a CI job. It is named `econ-sim` and it lives in `fractal-econ-sim`.

**Shape.** Deterministic agent-based model over the `Clock`, `Rng`, and `IdGen` ports (`10 §7`) so every run is replayable from a seed. Agent archetypes: `HonestCreator`, `HonestCustodian`, `HonestCurator`, `Lurker`, `Whale`, `SybilFarmer(n)`, `CollusionRing(n)`, `SelfDealer`, `WashTrader`, `AgentFarmOperator(n)`, `RewardStripper`. Simulation tick = one settlement epoch (24h); horizon = 3,650 ticks (10 years).

**Scenario catalogue (every one runs on every economy PR):**

| Scenario | Population shape | Asserts |
|---|---|---|
| `baseline` | The §5.2 growth curve | M1–M5 land in band by Y4; supply peaks and declines |
| `adversarial_100x` | Attacker volume at **100× normal actor volume** across all farm archetypes simultaneously | **I-E3** — the P12 test |
| `whale_hoard` | 5% of Citizens hold 60% and never spend | Velocity floor behaviour; breaker does not oscillate |
| `mass_exodus` | 60% population loss over 4 epochs | Overshoot handling; breaker relieves within 2 epochs |
| `collusion_ring_scaling` | Rings of n = 5, 20, 100, 500 | Per-capita ring yield stays below honest baseline at every n |
| `storage_self_deal` | Attacker at 0.1%, 1%, 10%, 40% custodian capacity share | Margin stays negative up to 40% capacity share |
| `cap_transition` | Claim crosses budget at Y4 | π̄ falls smoothly; no discontinuity; no source starves below its share |
| `tariff_rebase` | Anchor re-based −15%/yr for 10 years | FRB stays in the ±25% band throughout |
| `parameter_stress` | Every Tier-1 parameter at ±20% simultaneously, 1,000 combinations | No combination produces unbounded supply or SCR outside [0.4, 2.0] |

**Asserted invariants:**

```
  I-E1  Σ debits == Σ credits, every tick, every scenario.                       (11 §7.2)
  I-E2  cumulative_emission(t) ≤ Σ_{n≤year(t)} B(n), for all t.
  I-E3  circulating_supply(t) ≤ 200,000,000 FRC over the 10-year horizon
        under adversarial_100x.                                 ← P12 falsification test
  I-E4  no Principal receives > 0.5% of any epoch's total emission.
  I-E5  attacker_ROI < 0 for every archetype in the adversary catalogue.
  I-E6  SCR ∈ [0.85, 1.15] from Y4 onward in `baseline`.
  I-E7  FRB ∈ [0.75, 1.25] × its 12-month trailing mean, always.
  I-E8  balance ≥ locked ≥ 0 for every Wallet, every tick.
  I-E9  total_supply == −EmissionAccount.balance, every tick.     (11 §7.4)
  I-E10 no Source formula reads a field classified `Gameable`.               (E3, §7.1)
```

**The gate (this is the enforcement teeth of P12):**

```
  PR to any economy crate        →  50 seeds, all invariants, must be green to merge.
  Nightly                        →  1,000 seeds, all scenarios.
  New or modified Source/Sink    →  ships behind ff.economy.<name>, DEFAULT OFF,
                                    and may not be enabled until 1,000 seeds pass.
  Any Tier-1 parameter proposal  →  must attach a simulation artifact (seed set, commit
                                    hash, 24-month M1–M5 projection). No artifact, no vote.
  I-E3 failure                   →  the mechanic ships disabled. Non-negotiable (P12).
```

---

## 13. Governance of Parameters

| Tier | Contents | Who may change | Process |
|---|---|---|---|
| **Tier 0 — Immutable** | Quantum size (1e-9); the double-entry invariant; the 1e9 FRC hard cap; the 0.80 decay factor; "no discretionary emission"; "Societies cannot mint Fraction"; the Conservative Rounding Rule; the Never list (`02 §4`) | **Nobody.** These are not parameters. | Changing any of them is a new currency, not an amendment. It would require overturning P12 by ADR and a supply migration with Citizen consent. |
| **Tier 1 — Platform-global** | Source share vector; per-Source caps and rates; fixed fee schedule (K1–K12); burn/redistribute splits; storage and bandwidth tariffs; free quotas; vesting fraction; quadratic constants `k`, `T_max`, and the `^0.6` exponent; breaker thresholds | Platform Governance, human-signed only (P4). **No Agent may propose or enact.** | Published proposal → simulation artifact attached → **30-day notice** → human signature. Bounds: ≤ ±20% relative per change (±25% for caps, ±15%/yr for the anchor), ≤ 1 change per parameter per 90 days. Every enactment emits `EconomicParameterAdjusted` with the prior value, new value, and simulation commit hash. |
| **Tier 1a — Indexed** | The storage tariff (the anchor) | Rule, not vote | Re-based annually to the measured 12-month blended reference cost of a GB-month across the Custodian mesh and the `BlobStore` fallback, clamped to ±15%/yr. Published input, published output, no discretion. |
| **Tier 2 — Society-local** | Charter `EconomyParams` (`11 §2.3`): internal fee splits, Treasury spending rules, local contribution weights, join Stake amounts | The Society, under its Charter's `AmendmentRule` | Ordinary governance. **Hard-bounded by Tier 1**: a Society may divide what it receives and set what it charges its own members; it may not alter emission, mint Fraction, or exceed platform caps. Enforced at the domain layer, not by convention. |

---

## 14. The Path to External Exchangeability (Phase 9+)

### 14.1 Preconditions — all seven, none optional

```
  1.  SCR ∈ [0.85, 1.15] for 8 consecutive quarters (2 years), production data.
  2.  Circulating supply YoY variance < 10% for 8 consecutive quarters.
  3.  FRB within ±25% band for 8 consecutive quarters.
  4.  Top-100 Wallet concentration (M4) < 25% of circulating supply.
  5.  Emission intensity (M5) < 5% of circulating supply annually.
  6.  econ-sim green on 10,000 seeds INCLUDING external-liquidity scenarios
      (price shock ±10×, arbitrage against the administered tariff, exit runs).
  7.  Legal opinion obtained per target jurisdiction covering: instrument
      classification, money-transmission and e-money licensing, KYC/AML and
      sanctions screening, consumer-protection and disclosure obligations,
      and tax reporting for recipients of emission.
```

**Precondition 7 is where this document stops.** Nothing in this chapter is legal or financial advice, and no engineer or Agent may treat it as such. The listed items are the *questions counsel must answer*, not answers. Phase 9 does not open on an engineering judgement.

### 14.2 Why premature external liquidity would destroy the internal economy

Not a preference — arithmetic. Suppose FRC lists externally in Y3 and the price rises 10× against whatever it lists into.

```
  The anchor (E1) says 1 FRC ≡ 1 GB-month. That tariff is administered and does not move.

  → A Society holding 10,000 FRC now perceives its storage as 10× more "expensive"
    in external terms and stops buying above the free quota.
  → K8, the largest single Sink in the mature model, collapses toward zero.
  → SCR falls from ~1.0 toward ~0.3. Emission now outruns sinks by 3×.
  → Supply grows without bound relative to the utility available to absorb it.
  → The price rise destroyed the mechanism that gave the price a floor.

  And in the other direction — a 90% price fall:
  → Custodians earning 0.28 FRC/replica-GB-month now earn 0.028 in external terms.
  → Custodian capacity exits; replication targets fail; ReplicaLost events cascade;
    the storage product degrades; paid storage demand falls; K8 collapses anyway.
```

Both directions terminate at the same failure. An externally-priced FRC is no longer a unit of account for platform resources — it becomes a claim whose value is set elsewhere and imported into every internal price. Additionally: the moment a market price exists, contribution becomes mining, the most efficient farm wins, and every §8 margin must be recomputed against an attacker whose reward is externally fungible.

The preconditions exist to ensure that when external liquidity arrives, the internal sinks are large enough and diverse enough that internal demand — not the external market — remains the marginal source of FRC value.

---

## 15. Trade-offs and Rejected Alternatives

### 15.1 Rejected designs

| Alternative | Honest case for it | Why rejected |
|---|---|---|
| **Fixed supply, no emission** (all Fraction pre-created and distributed at genesis) | Simplest possible supply story. Zero inflation risk. No emission code, no settlement runs, no farm surface. | It cannot pay Custodians — the one job the token exists for. And whoever holds the genesis allocation holds permanent economic power over every future contributor: pay-to-win by wealth (`02 §4`). |
| **Pure fee-share, no token at all** (fiat rails; contributors get a revenue share) | Genuinely simpler and lower-risk. No supply policy, no emission, no farming, no Phase-9 regulatory cliff. Real money is a better incentive than internal money. | It cannot price the Custodian mesh: sub-cent, continuous, peer-to-peer settlement is uneconomic on fiat rails, and it requires us to be the merchant of record for every peer transaction — the licensing burden we deferred to Phase 9 arrives in Phase 4 instead. It also cannot represent a Society Treasury as a first-class governed object. **This remains the named fallback (§15.2).** |
| **Points with no ledger** (a reputation number, no double-entry) | No economy to get wrong. Trivially safe. | Unauditable and arbitrary; violates P6 (not reconstructible) and P12 (no named source or sink); and it is precisely "karma", banned in `01 §10`. It also cannot settle real resource costs, so the storage problem is unsolved and returns as a billing system. |
| **Launch with an external listing** | Immediate liquidity, immediate contributor value, immediate attention. | §14.2. Also inverts the incentive on day one: with a price, the rational strategy is farming, and every §8 margin flips positive. |
| **Demurrage on Citizen Wallets** | Direct, effective velocity control. Well-understood. | §9.1. Confiscatory, regressive, and a "use it or lose it" dark pattern. |
| **Transfer fee** | Easy, high-volume, always-on Sink. | §4.1. Taxes the behaviour M3 wants and falls hardest on the smallest transfers. |
| **Stake-weighted governance** | Aligns decision power with economic exposure. Standard in the space. | Pay-to-win on governance weight — explicitly on the Never list (`02 §4`). Governance weight derives from Standing, never from balance. |
| **Paid discovery placement** | Would be the largest burn Sink available to us. | §4.2. It is advertising. |
| **Carry-forward of unspent emission budget** | Smooths reward volatility across lean epochs; feels fair to contributors in a slow quarter. | Accumulates a 335.8 M FRC overhang in our own 10-year model. A stockpile with a release rule is a dump with a schedule. |

### 15.2 The named fallback

If, at the end of Year 5, production telemetry shows **SCR still below 0.85** and **exchange income below 50%** of Citizen FRC income, the correct conclusion is that Fraction is functioning as a subsidy program rather than an economy. The response is not another parameter adjustment. It is to open the ADR that converts Fraction into an internal accounting unit for resource settlement only — Custodian, bandwidth, and compute payments — and to move contribution rewards to fee-share on real marketplace revenue.

Writing that down now, before there is anything to defend, is the cheapest thing in this document.

---

## 16. Invariants (the test suite for this chapter)

Each becomes a property test or a simulation assertion. This list extends `11 §7`.

1. **E1** — the storage tariff is the definitional anchor of FRC; changing it is an anchor re-base with a published, indexed rule (§13 Tier 1a), never an ordinary fee change.
2. **E2** — all rounding moves in the direction that shrinks circulating supply (§2.2).
3. **E3** — no raw activity count appears as a term in any Source formula (§7.1).
4. **E4** — every emission Posting names exactly one Source from §3.2; every non-emission Posting whose destination is `BurnAccount` names exactly one Sink from §4. `PostingReason` has no `Other` (`11 §2.6`).
5. **E5** — cumulative emission never exceeds the cumulative schedule; unspent budget is forfeited, never banked.
6. **E6** — no Principal receives more than 0.5% of an epoch's total emission.
7. **E7** — no Society, Charter, Agent, or Extension can cause an emission Posting. Emission originates only from the settlement run, only from the `EmissionAccount`.
8. **E8** — no Agent may propose or enact a Tier-0 or Tier-1 parameter change (P4).
9. **E9** — no float appears in `fractal-domain-ledger` or `fractal-domain-economy`; all ratios are ppm integers.
10. **E10** — a Source with a failing `econ-sim` invariant is unreachable in production: the feature flag defaults off and cannot be enabled by configuration alone.
11. **E11** — `Rail` has exactly one compiled implementation before Phase 9; there is no code path from a Wallet to an external asset (`00 §4` posture — absent, not merely unused).
12. **E12** — every economic parameter in effect is published at a stable, machine-readable endpoint with its effective date, prior value, and the simulation commit hash that justified it. An unpublished parameter is a violation of P12's "bounded, measured, and published".
