# 18 — Progression and Reputation

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `10-system-architecture.md` §3 (boundary S7 — Progression), `11-domain-model.md` §2.1/§2.4 (Citizen, Membership, Standing), `33-brand-identity.md` §4 (motion) and §7 (voice).
> **Governs:** XP, Level, Trust, Standing, Unlocks, Achievements, Milestones, Seasons, Society reputation, anti-farming policy, and every surface on which progression is displayed.
> **Owns boundary:** S7. Emits `XpAwarded`, `LevelReached`, `TrustAdjusted`, `StandingChanged`, `UnlockGranted`, `AchievementGranted`, `MilestoneCrossed`, `SeasonOpened`, `SeasonClosed`. Reads Fraction balances **never** (invariant `11 §7.8`).

---

## 1. Thesis, and the Anti-Goals

### 1.1 What progression is for

Progression in Fractal Node does exactly three jobs. Nothing else is a legitimate reason to add a mechanic here.

**J1 — Gate capability responsibly.** A Level 0 Citizen — an account created ninety seconds ago — must not be able to DM strangers, mint Facets, publish an Extension, enroll an Agent, or take custody of another Society's Shards. Not for moral reasons: those are precisely the capabilities an adversary wants on day zero, and P8 (deny by default) has to mean something *after* registration, not only before it. Progression is the schedule on which the deny-by-default posture relaxes.

**J2 — Teach the platform gradually.** Societies, Chambers, Charters, Envelopes, Facets, Vaults, Custodianship, a Treasury, and a CLI that does all of it. Presenting that surface at once is not generous; it is unusable. Levels are a paced curriculum: each unlock arrives shortly after the concept it depends on has become familiar.

**J3 — Make contribution legible.** A Society choosing a moderator, a Federation choosing a delegate, a Citizen choosing whose Facet to acquire — all need evidence. XP, Trust, and Standing are that evidence, in three registers answering three different questions. Legibility is the feature; the numbers are the implementation.

### 1.2 The anti-goals, stated so they can be cited in review

| # | Anti-goal | Falsification test |
|---|---|---|
| A1 | **Not an engagement treadmill.** No mechanic exists whose primary output is time-on-platform. | For every XP source, name the durable artifact or service it produced. A source whose artifact is "the Citizen was here" is a violation (P12). |
| A2 | **No time-pressure mechanics.** No streaks, no daily login rewards, no countdown timers on progress, no expiring earnable content. | Grep the UI for a countdown bound to a progression value. Any hit is a violation (`02 §4`, dark patterns). |
| A3 | **No purchasable progression.** Fraction buys cosmetics, storage, compute, and Extensions. It never buys XP, Trust, Standing, governance weight, or an Achievement. | Trace every `XpAwarded` and `TrustAdjusted` event's causation chain. Any chain containing a `Posting` whose reason is a purchase is a violation. |
| A4 | **No loot boxes, no randomized rewards.** Every Unlock has a stated, deterministic condition. | Any grant path invoking `Rng` is a violation. |
| A5 | **No leaderboard as a primary surface.** Ranking Citizens against each other converts contribution into competition and competition into farming. | A default-visible surface that sorts Citizens by XP is a violation. |
| A6 | **No conflation.** XP and Trust are never the same number, never derived from each other, never displayed as one value (`01 §7`, hard rule). | Invariant `11 §7.8` as a property test, plus a UI lint (§10.4). |
| A7 | **No silent denial.** Withheld XP, held XP, and Trust decrements are always visible to the Citizen with a stated cause. | Any progression state change with no reader-visible cause row is a violation (P9's inspectability, `02 §4` silent telemetry). |

### 1.3 The one-sentence rule

> Progression raises the ceiling on what you may do. It never raises the floor on how often you must show up.

---

## 2. The Three-Axis Model

Three orthogonal quantities. Different units, different domains, different update rules, different lifetimes, different UI. This separation is the single most important design decision in the chapter, and §12.1 explains what happens to systems that skip it.

```
                    WHAT IT ANSWERS            SHAPE             SCOPE        LIFETIME
  ┌──────────┐   "How much have you       monotonic ↑         GLOBAL       permanent
  │    XP    │    contributed?"           never decreases     (Citizen)    never decays
  └────┬─────┘                                                             never resets
       │ pure function                          
       ▼                                        
  ┌──────────┐   "What may you do?"       derived, monotonic  GLOBAL       permanent
  │  LEVEL   │                            L = f(XP) only
  └──────────┘

  ┌──────────┐   "Can you be relied      bidirectional ±      GLOBAL       decays toward
  │  TRUST   │    upon?"                  bounded ±1000       (Citizen,    neutral
  └──────────┘                            saturating          Agent, Node)

  ┌──────────┐   "What are you to        4 independent dims   PER-SOCIETY  windowed;
  │ STANDING │    THIS Society?"          never summed        (Membership)  decays on
  └──────────┘                                                              inactivity

   ═══════════════════════════════════════════════════════════════════════════════
   FORBIDDEN EDGES  (each is a CI-enforced property test)
     XP ──✗──► TRUST        volume never buys reliability
     TRUST ──✗──► XP        reliability is not a contribution multiplier on the earner
     FRACTION ──✗──► any    money buys capacity, never progression   (02 §4)
     STANDING ──✗──► one scalar   no weighted sum exists anywhere in the codebase
   ═══════════════════════════════════════════════════════════════════════════════
```

### 2.1 XP — volume of meaningful contribution

**Definition.** `Xp: u64`, global, monotonic, per Citizen. The sum of all `XpAwarded` events. There is no decrement path in the domain; the type exposes only `add`.

```
XP(t) = Σ  award(e)      over all XpAwarded events e with occurred_at ≤ t
award(e) ≥ 0 for all e.      No event ever emits a negative award.
```

**Why monotonic.** An XP figure that can fall is a Trust figure wearing the wrong label. Retroactive removal also makes Levels non-monotonic, which makes Unlocks revocable, which destabilises the capability model — a Citizen would lose an Agent they already enrolled. The correct response to fraudulent XP is not subtraction but (a) not awarding it, via the escrow in §9.6, and (b) a Trust decrement, which is the axis built to move down.

**Honest cost.** A Citizen who farmed successfully before a defence shipped keeps the XP. Accepted: a retroactively mutable progression history costs more than the fraud does, and contradicts P6's premise that the Log is fact.

### 2.2 Trust — reliability and good faith

**Owning document.** `12-identity-and-trust.md` §8 defines the Trust type — range `-1000..=1000` with neutral `0`, the admitted input set, the `head^1.5` saturating update function, the decay half-lives (180 days positive, 365 days negative, floored at the value the last 90 days of evidence justifies), the flow-capped vouch graph, and the type-level exclusion of XP, Level, Fraction, and volume (I-12.8). **This chapter does not redefine any of it and must not.** What follows is the progression-side contract only.

Three properties of `12 §8` carry consequences the rest of this document depends on:

- **Saturation.** The `head^1.5` term means the last two hundred points of Trust cost far more than the first two hundred. Trust is cheap to establish and expensive to perfect — which matches what it measures. Every Trust gate in §5 is therefore set at a value a diligent Citizen reaches in months, not years.
- **Positive Trust is present-tense.** It decays on a 180-day half-life whether or not the Citizen misbehaves, because a dormant principal's reliability is unknown rather than proven. Trust-gated Unlocks are consequently **conditional**, not permanent (§5). Level-gated Unlocks are permanent. This asymmetry is the entire practical difference between the two axes, expressed in the capability model.
- **Negative Trust fades.** A 365-day half-life, slower than positive but not indefinite, is what makes the redemption path in §11.4 real rather than decorative.

**Progression-specific settlement rules** (these are this chapter's additions to `12 §8.2`, not replacements):

| Rule | Statement |
|---|---|
| Deferred settlement | A moderation action's Trust credit settles at **T+14 days**, on upholding — never at the moment the action is taken. A moderator cannot bank reputation for a decision that has not survived appeal |
| Window granularity | Custody Trust settles per settlement window, not per Shard, so a Custodian with a million Shards and one bad night is not annihilated |
| Symmetric appeal | A successful appeal credits the appellant and debits the actor in the same transaction. There is no outcome in which being wrong is free |
| Onboarding neutrality | Onboarding a Citizen who later proves hostile costs the inviter vouch budget (`12 §8.4`) but never Trust directly. Recruitment is not a reliability claim |

**Why Trust is never derived from XP.** The moment reliability is a function of volume, the cheapest route to being trusted is to be loud. Every reputation system that has collapsed — forum karma, marketplace star inflation, aggregated "reputation points" — collapsed through this one edge. Volume is trivially manufacturable by an adversary with time or scripts; reliability is only manufacturable by being reliable across counterparties who could have penalised you instead. Separating them keeps the expensive signal expensive.

### 2.3 Standing — Society-scoped, four-dimensional

`Standing` is defined in `11 §2.4` as a four-field struct. This document specifies the semantics of each field and forbids their collapse.

| Field | Type | Semantics | Window | Decay |
|---|---|---|---|---|
| `contribution` | `u32` | XP earned **within this Society**, summed over a rolling 180-day window | 180d | Rolls off naturally |
| `trust` | `i32` | Society-local reliability on the same `-1000..=1000` scale, updated by the function in `12 §8.3` with Charter-tunable weights inside platform bounds | — | Same half-lives as global Trust |
| `tenure_days` | `u32` | Days since `joined_at`, backdated for Crystallization (`11 §3.1`) | — | **Never decays** |
| `governance` | `u32` | Quorum-reaching votes cast + enacted proposals authored, over a 180-day window | 180d | Rolls off naturally |

Lifetime contribution is not a Standing field. It is replay-derivable from the Log (P6) and is exposed in the audit record (§10.3), but gates never read it — because a gate on lifetime contribution is a gate that a Citizen who left three years ago still passes.

**The conjunction rule — an invariant, not a guideline.**

```
A Standing gate is a CONJUNCTION of per-dimension thresholds. It is never a weighted sum.

    gate ::= contribution ≥ a  ∧  trust ≥ b  ∧  tenure_days ≥ c  ∧  governance ≥ d

FORBIDDEN, and absent from the codebase, not merely unused:
    score = w1·contribution + w2·trust + w3·tenure + w4·governance
```

Why: a weighted sum lets a Citizen buy their way past a low Trust threshold with high contribution. That is the exact failure the three-axis model exists to prevent, reintroduced one abstraction layer down. `StandingRecord` therefore has no `Ord` implementation and no `total()` method, and adding one fails the dependency lint.

**Global Trust vs Society Trust.** A Citizen's global Trust seeds their Society Trust at join time as `T_society = round(0.5 · T_global)`, and thereafter the two evolve independently. A Society may be more forgiving or more demanding than the platform; it may not export its judgement onto the platform. Global Trust moves only on platform-scoped events (custody, Sybil findings, Envelope violations, platform moderation). This keeps one hostile Society from destroying a Citizen's platform-wide standing (P1: Societies are sovereign *within* their boundary, not beyond it).

---

## 3. XP: Sources, Formulas, and Caps

### 3.1 The single award formula

Every XP source in the platform computes its award through one function. There are no bespoke award paths.

```
XP_s(n) = ⌊ B_s · φ_s(n) · R · Q ⌋          and any result < 1 is 0

  B_s   base award for source s                     (table §3.3)
  φ_s(n) = δ_s^(n−1)   diminishing returns; n is the 1-based index of this
                       qualifying event within the rolling 24h window for s
  R     counterparty trust weight (§3.2); 1.0 for sources with no counterparty
  Q     quality gate ∈ {0, 1}; see §3.4. Not a continuous multiplier.

CAPS, applied in order after the above:
  1. per-source daily cap      C_s^day
  2. per-source weekly cap     C_s^week
  3. GLOBAL CEILING            4,000 XP per Citizen per rolling 7 days
```

The global ceiling is the load-bearing constraint. It bounds the fastest possible Level curve regardless of how many sources an adversary combines, which is what makes the time-to-level figures in §4.3 a guarantee rather than an estimate.

### 3.2 Counterparty trust weighting

XP that depends on another principal's action is weighted by *that principal's* Trust — never by the earner's (which would be the forbidden `TRUST → XP` edge).

```
w(c) = clamp( (T_c + 1000) / 2000 , 0 , 1 )      neutral counterparty ⇒ 0.5
age(c) = 0.25  if account_age(c) < 14 days,  else 1.0
R      = ( Σ_{c ∈ distinct counterparties} w(c) · age(c) ) / |distinct counterparties|
```

A brand-new zero-Trust account contributes `0.5 × 0.25 = 0.125` of a weight unit. Eight of them are worth one established Citizen. This is the primary economic defence against alt-account boosting, and it is deliberately not a binary block — a real new Citizen's engagement still counts, just less.

### 3.3 The XP table

`n` counts within a rolling 24h window per source unless the row says otherwise.

| Code | Source | `B` | `δ` | Day cap | Week cap | Anti-farm defence |
|---|---|---|---|---|---|---|
| `XP.MSG` | Chamber message posted | 4 | 0.72 | 12 | 60 | 6th message of the day earns 0 (§3.5). `Q` requires ≥ 80 chars or media, non-duplicate |
| `XP.THREAD` | Thread you opened reaches ≥ 3 distinct substantive repliers | 25 | 0.85 | 100 | 400 | Settles at T+24h; repliers must be distinct and Trust-weighted |
| `XP.ENGAGE` | Substantive engagement received (reply, quote, citation, save) | 6 / engager | 0.90 | 150 | 700 | **Raw reactions award zero XP, always** (§9.2). One award per distinct engager per artefact, ever |
| `XP.CURATE` | Curation adopted (pin, collection, or digest accepted by a Chamber) | 30 | 0.80 | 120 | 500 | Requires `Standing.governance ≥ 20`; acceptance is by a different principal |
| `XP.MOD` | Moderation action upheld | 60 | 0.90 | 300 | 1,200 | Settles at T+14d on upholding. An overturned action awards 0 **and** costs Trust |
| `XP.APPEAL` | Appeal you filed succeeds | 120 | 0.85 | — | 360 | Reversal is the gate |
| `XP.GOV.VOTE` | Vote cast on a Proposal that reaches quorum | 20 | 0.85 | 80 | 300 | No XP for voting on a proposal you authored. Quorum-gated, so votes on dead proposals pay nothing |
| `XP.GOV.PROP` | Proposal you authored is enacted | 250 | 0.75 | — | 1,000 | Enactment-gated, not authorship-gated |
| `XP.CHARTER` | Charter amendment you authored is enacted | 300 | 0.75 | — | 900 | As above; human signatures required (P4) |
| `XP.STORE` | Storage custody settled | 1 / GB-month | linear | 200 | 900 | Paid against **Attestations**, never claimed capacity (`11 §2.7`). Self-custody pays 0 |
| `XP.BAND` | Bandwidth served | 1 / 5 GB egress | linear | 100 | 450 | Receiver-signed receipts; FNID-pair loops excluded |
| `XP.COMPUTE` | Compute contributed (transcode, index, inference) | 1 / 10 credit-units | linear | 150 | 600 | Redundant re-execution sampling at 3%; mismatch voids the window and slashes |
| `XP.AGENT` | Agent or Workflow you published is installed by another Society | 150 | 0.85 | — | 1,200 | Distinct Society **and** distinct Operator; install must survive 14 days |
| `XP.EXT` | Extension you published is installed by another Society | 400 | 0.85 | — | 2,000 | As above, plus security review pass |
| `XP.ONBOARD` | A Citizen you onboarded independently reaches Level 2 | 200 | 0.90 | — | 800 | Invitee's Level 2 must be reached via counterparties other than the inviter |
| `XP.CODE` | Merged contribution to a platform or Society repository | 80 / 200 / 600 | 0.85 | — | 2,000 | Review class assigned by a human maintainer |
| `XP.DOC` | Documentation or translation merged | 60 | 0.85 | — | 600 | Human maintainer sign-off |
| `XP.FACET` | A Facet you minted is acquired by a distinct Citizen | 40 | 0.80 | — | 400 | Paid on third-party acquisition, never on mint. Mint costs a fee (a Sink) |
| `XP.ATTEST` | Peer attestation later corroborated | 15 | 0.88 | 90 | 350 | Corroboration by an independent attestor is the gate |

**Sources that award zero XP, permanently, by design:** logging in; opening the app; reading; reacting; following; profile edits; joining a Society; being online; message length beyond the `Q` threshold; any action taken by an Agent on the Operator's behalf (§9.5).

### 3.4 `Q` is a gate, not a dial

`Q ∈ {0, 1}`. It is a deterministic, inspectable admissibility check — *not* a model-scored quality rating.

```
Q = 1  iff  ¬duplicate(body, author, 30d)
       ∧    length ≥ threshold_kind  ∨  carries media/Facet/code
       ∧    ¬ (author == sole participant in thread)
       ∧    the Chamber is not marked xp_exempt by its Charter
```

A continuous model-derived score was rejected: non-deterministic (breaks P6 replay), non-inspectable (breaks A7), and it makes XP a function of a `ModelProvider` adapter — a vendor inside the domain layer (P5). A binary gate is weaker and honest. Nuanced value judgement is what `XP.ENGAGE` is for: other Citizens do it, weighted by their Trust.

### 3.5 What repetitive clicking is worth

`XP.MSG` with `B = 4`, `δ = 0.72`, and a Trust-neutral solo author (`R = 1`, no counterparty):

```
  n:      1     2     3     4     5     6     7     8     …    50
  raw:  4.00  2.88  2.07  1.49  1.08  0.78  0.56  0.40         0.0000004
  awd:     4     2     2     1     1     0     0     0    …        0

  cumulative for the day: 10 XP.  Marginal XP of message #6 onward: ZERO.
```

Ten XP is 0.25% of the weekly ceiling. Two hundred messages a day earns exactly what five earns. The XP is in `XP.ENGAGE` — whether anyone with Trust replied — which is not a function the author controls by clicking. The same shape holds for every source. The design rule, stated so it is checkable at review:

> **Cost-asymmetry rule.** For every XP source, the adversary's marginal cost of producing one more unit of XP must exceed the marginal Fraction value of that XP at the current emission rate (`17 §7`). A source that fails this ships disabled (P12 falsification test).

---

## 4. The Level Curve

### 4.1 The formula

```
increment(L) = min( round10( 100 · L^1.5 ) , 9000 )        XP to go from L−1 to L
threshold(L) = Σ_{i=1..L} increment(i)                     cumulative XP to hold Level L
Level(XP)    = max { L : threshold(L) ≤ XP }               pure function; never set directly
```

The cap engages at `L = 21`. Below it the curve is superlinear; above it, strictly linear. That is the whole shape argument: **superlinear early so each of the first dozen Levels is a real accomplishment; linear late so no Level is ever unreachable.** A permanently superlinear curve eventually prices the next Level beyond a human lifetime — a level cap that lies about being one.

### 4.2 Thresholds, Levels 0–30

| L | Δ | Cumulative | L | Δ | Cumulative | L | Δ | Cumulative |
|---|---|---|---|---|---|---|---|---|
| 0 | — | 0 | 11 | 3,650 | 17,910 | 21 | 9,000 | 85,080 |
| 1 | 100 | 100 | 12 | 4,160 | 22,070 | 22 | 9,000 | 94,080 |
| 2 | 280 | 380 | 13 | 4,690 | 26,760 | 23 | 9,000 | 103,080 |
| 3 | 520 | 900 | 14 | 5,240 | 32,000 | 24 | 9,000 | 112,080 |
| 4 | 800 | 1,700 | 15 | 5,810 | 37,810 | 25 | 9,000 | 121,080 |
| 5 | 1,120 | 2,820 | 16 | 6,400 | 44,210 | 26 | 9,000 | 130,080 |
| 6 | 1,470 | 4,290 | 17 | 7,010 | 51,220 | 27 | 9,000 | 139,080 |
| 7 | 1,850 | 6,140 | 18 | 7,640 | 58,860 | 28 | 9,000 | 148,080 |
| 8 | 2,260 | 8,400 | 19 | 8,280 | 67,140 | 29 | 9,000 | 157,080 |
| 9 | 2,700 | 11,100 | 20 | 8,940 | 76,080 | 30 | 9,000 | 166,080 |
| 10 | 3,160 | 14,260 | | | | | | |

Levels above 30 continue at 9,000 XP each, indefinitely. There is no level cap and there is no prestige reset (§12.5).

### 4.3 Time to Level

Three profiles, defined by sustained weekly XP against the 4,000 ceiling.

| Profile | XP/wk | Behaviour |
|---|---|---|
| **Light** | 400 | A few substantive posts a week that people reply to; occasional vote |
| **Moderate** | 1,100 | Regular participation in 2–3 Societies, some curation or moderation, votes |
| **Heavy** | 2,800 | Daily substantive participation, custody or compute contribution, governance, publishing |

| Reach | Light | Moderate | Heavy |
|---|---|---|---|
| L1 | day 1 | day 1 | day 1 |
| L3 | 2 wk | 5 d | 2 d |
| L5 | 7 wk | 3 wk | 1 wk |
| L8 | 21 wk | 8 wk | 3 wk |
| **L12 — capability ceiling** | **13 mo** | **5 mo** | **8 wk** |
| L20 | 3.7 yr | 1.3 yr | 6 mo |
| L30 | 8 yr | 2.9 yr | 1.1 yr |

**The capability ceiling is Level 12.** Every capability Unlock in §5 lands at or below it. Levels 13+ grant recognition, cosmetics, and eligibility for *opt-in* roles only.

> **Invariant I-7:** No Unlock at Level > 12 confers authority over another principal, a higher rate limit, a larger quota, or any economic advantage.

This is why an eight-year Light path to Level 30 is acceptable rather than hopeless: nothing a Citizen *needs* sits at the far end. The far end records a long life on the platform, and that legitimately takes a long time. Putting real capability at Level 30 would be a treadmill wearing a curve (A1).

---

## 5. The Unlock Catalog

An **Unlock** is granted by a `UnlockGranted` event when its gate first evaluates true. Unlocks are permanent with one exception, stated explicitly: **Trust-gated and Standing-gated Unlocks are conditional and are suspended while their gate is false.** Level-gated Unlocks are permanent, because Level is monotonic. The distinction is visible in the UI (§10.2) and in `fn me unlocks`.

```
GATE GRAMMAR
  gate ::= Level ≥ n
         | Trust ≥ t                     (conditional — suspends if violated)
         | Standing(society){ conjunction of §2.3 dimensions }   (conditional)
         | Achievement(id)               (permanent)
         | SocietyLevel ≥ n              (society-scoped)
         | Stake(amount)                 (conditional — released on unstake)
         | conjunction of the above
Envelopes (11 §2.8) still apply on top of every Unlock. An Unlock says
"you MAY hold this capability"; an Envelope says "you DO, until <expiry>".
```

### 5.1 Features (Citizen Level)

| L | Unlocked |
|---|---|
| **0** | Read public Societies; join up to 3; post (10 msg/hr); react; receive Fraction; receive DMs from Citizens in a shared Society; 5 MB/day media; 1 GB Citizen Vault; procedural avatar; **found the first hearth — exactly one Society, free (see below)** |
| **1** | Initiate DMs within shared Societies; Transfer ≤ 10 FRC/day; 50 MB/day media; join 10 Societies |
| **2** | Open a Convergence; custom avatar; 3 Profile Modules; Transfer ≤ 100 FRC/day; 250 MB/day media |
| **3** | **Found a second and subsequent Society** (250 FRC each, `17` K1); API token, 60 req/min; full CLI parity locally; 1 GB/day media; 5 GB Citizen Vault |
| **4** | DM Citizens outside shared Societies (subject to their privacy setting); enroll 1 Agent; Transfer ≤ 500 FRC/day |
| **5** | Create Chambers where a role permits; hold and transfer Facets; 6 Profile Modules; theme selection |
| **6** | **Mint Facets** (in a Society at Society Level ≥ 3); API 300 req/min; 3 Agents; author Workflows |
| **7** | Install Extensions where the Charter permits; 25 GB Vault; publish a Workflow to the marketplace |
| **8** | **Custodian eligibility**; Relay participation; Transfer ≤ 2,000 FRC/day |
| **9** | **Register a self-hosted Node**; API 600 req/min; custom sigil variant (3 lifetime re-rolls) |
| **10** | 5 Agents; 100 GB Vault; publish Extensions to the marketplace; Transfer ≤ 5,000 FRC/day |
| **11** | Federation delegate eligibility; Season objective authorship (proposal only) |
| **12** | **Capability ceiling.** API 1,200 req/min; 10 Agents; 250 GB Vault; no Level-imposed transfer limit (Envelope and Charter limits remain); sandboxed Theme Extension authoring |
| 13–30 | Recognition only. Insignia tiers, layout variants, archival tools, Chronicle export, marketplace featured-slot *eligibility* (never placement). See I-7 |

Rate limits and quotas above are ceilings the gateway enforces per principal (`10 §10`). Vault quota is *additionally* purchasable with Fraction — capacity is a commodity. **Scheduling precedence is not** (§5.5).

> **The first-hearth exemption.** Every Citizen may found **exactly one** Society at Level 0, at no Fraction cost (`17` K1 already prices a Citizen's first Society at 0 FRC). This is a deliberate exemption from J1's Sybil argument (§1.1), and it does not weaken it: **the farmable quantity is Society *volume*, not the first Society.** An attacker who can create one Society per identity is bounded by the identity system, which is where that defence belongs (`12 §9`); an attacker who can create a thousand per identity is bounded by nothing. Four details make it a gate rather than a loophole.
>
> 1. The allowance is **one-time, non-transferable and per-FNID**, consumed at `SocietyCreated`.
> 2. It is **not restored** if that Society is later Dissolved, Archived, or the founder departs. A renewable first-hearth allowance is a renewable Sybil resource.
> 3. A **Crystallization does not consume it** (`11 §3.1`): a Convergence that crystallizes has already cleared ≥ 3 participants, ≥ 48 h or ≥ 100 messages, and ≥ 2 participants at Level ≥ 1 — a stronger gate applied to the group than Level 3 is to the individual.
> 4. The second and every subsequent Society requires `Level ≥ 3` **and** the 250 FRC K1 charge.
>
> Without this exemption, `02 §2`'s spine sentence — "A Citizen can create a Society" — and `50 PH1` AC-1 — registration to Society creation to first message in under three minutes, unassisted — are both unreachable, because §4.3 puts time-to-Level-3 at two weeks on the Light path. There is a second-order benefit: every recovery failure today produces a Level-0 Citizen who cannot found anything, which manufactures precisely the population profile the Sybil defences are tuned to suppress (`61 W13`). Ruled in `61 X7`; `11 §2.3` carries the same note.

### 5.2 Society capabilities

The table in `11 §2.3` is Canon and is reproduced here unchanged in its left two columns, with this document's additions in the third.

| SL | Canon unlocks (`11 §2.3`) | Expansion specified here |
|---|---|---|
| **0** | Founder governance, 1 Chamber, 25 members | Text Chambers only; **5 GB Vault**; no Extension installs; no Treasury spend; Agent mode `Forbidden` or `OnMention` |
| **1** | Roles, 5 Chambers, 100 members, Treasury spending | Gallery + Board Chambers; **25 GB Vault**; Society accent selection (`33 §2.5`); Agent mode `Participant`; 1 enrolled Agent |
| **2** | Council governance, custom Charter clauses, 500 members | Voice + Stage Chambers; **100 GB Vault**; Charter amendment process; 5 Agents; moderation role delegation; Convergence sponsorship |
| **3** | Direct voting, custom sigil, Extension installs, Facet minting, **2,000 members** | Custom sigil **upload** (`33 §6`); **500 GB Vault**; Facet Standard selection; Society-scoped Achievements; own Season Challenge authorship |
| **4** | Delegated governance, Federation, Experience hosting, **10,000 members** | Canvas + Experience Chambers; **2 TB Vault**; cross-Society Envelopes; treasury-funded Sources within platform bounds (`17 §7`) |
| **5** | **Fracture**, self-hosted Node, custom economic parameters, **unbounded members** | **5 TB Vault**; Charter-defined Standing weights (within platform bounds); Society-issued Insignia; Federation founding; Custodian pool operation |

> **There is no unmetered grant, at any Level.** SL4's former "unmetered Vault (cost-settled)" is deleted: it contradicted `50 PH2`'s mitigation — "no unmetered storage, ever" — and `02 §4`'s prohibition on infinite or discretionary emission, since a grant funded from emission with no ceiling is exactly that with an extra step. Capacity beyond the grant is purchasable with Fraction (§5.5); the grant itself is bounded.
>
> **The ladder is generated, not authored.** Both quota tables in this section — Citizen (§5.1) and Society (above) — are rendered from `economy/rates.toml`, which `17` owns, and are additionally bound in aggregate by **Invariant E14**:
>
> ```
>    SUM(free_GB) x replication(1.60) x S1_rate(0.28)  <=  0.60 x S1_share_of_B(n)
> ```
>
> Free storage scales with population **and** with monotonically increasing Society Level (I-9), so without an aggregate ceiling storage emission grows rather than converging to zero as `17 §3.3` claims. When the left side reaches the right, the ladder is re-derived downward at the next epoch with 90 days' published notice (`17 §13`). The grant is a Level-indexed allowance denominated in the anchor unit, not a number of gigabytes fixed forever (`61 X6`).

**Fracture at Society Level 5 is honoured exactly as `11 §2.3` states, and additionally gated on reputation floors (§6.3).** A Society that has reached Level 5 and then lost moderation quality keeps the Level and loses the ability to exercise Fracture until the floor is restored. Attainment and fitness are different questions (§6.4).

### 5.3 Themes and profile customization

| Gate | Unlocked |
|---|---|
| L0 | Void theme; procedural avatar; procedural Society sigil |
| L2 | Custom avatar; 3 Profile Modules |
| L3 | Daylight theme; density mode (Comfortable / Compact / Dense, `33 §5.5`) |
| L5 | Curated theme set (6 accents from the 12-stop wheel); 6 Modules |
| L7 | Free Module ordering and layout grid |
| L9 | Custom sigil variant — deterministic re-roll of the procedural seed, 3 lifetime |
| L12 | Theme Extension authoring — **token overrides only**, validated against the contrast floors in `33 §2` at install time. A theme that fails WCAG AA cannot be installed (N8) |
| Achievement `GENESIS` | Genesis colourway (permanent, non-transferable) |
| Season | Season accent pack; permanent once installed (§8.4) |

Fraction may purchase cosmetic packs. It may not purchase a Level-gated customization slot, because slots are complexity budget, not decoration.

### 5.4 Insignia, Badges, and Collectibles (Facets — see `16`)

All three are `FN-ASSET/1` Facets minted under the platform Society or an issuing Society. The distinction is the license, and it is the pay-to-win firewall.

| Kind | License | Evolution | Transferable | Purchasable |
|---|---|---|---|---|
| **Insignia** | `NonTransferable` | `Tiered` — gains tiers as the underlying Milestone advances | **Never** | **Never** |
| **Badge** | `NonTransferable` | `Immutable` — a fixed record of one Achievement | **Never** | **Never** |
| **Collectible** | `Transferable` | Varies (`16`) | Yes | Yes, with Fraction |

> **Invariant I-8:** No Facet whose acquisition path includes a Transfer may be an input to any Level, Trust, Standing, or Unlock gate. Collectibles are ornament and market; Insignia are record.

Because Insignia are `Tiered` Facets, a Milestone crossing *evolves* the existing Facet rather than minting a new one. A Citizen's custody Insignia at 10,000 GB-months is the same `FacetId` it was at 1 GB-month, with a provenance chain showing every tier crossing. This is the Facet model doing the work it was designed for (`11 §2.9`).

### 5.5 Infrastructure privileges

| Privilege | Gate |
|---|---|
| **Custodian eligibility** | `Trust ≥ 100 ∧ proof-of-capacity at registration ∧ Stake(bond_rate × committed_bytes, floor 500 FRC) ∧ 30-day probation at reduced assignment`. **No Level gate.** `13 §7.4` owns custody economics and is the sole source for the bond; a flat figure cannot make slashing costlier than misbehaviour across three orders of magnitude of committed capacity — at 50 TiB, 500 FRC is one day's earnings. The Level gate is deleted rather than lowered: Level 8 is 21 weeks on the Light path (§4.3) and Trust ≥ 200 needs months of evidenced events whose highest-frequency admitted input is *Custodian attestation streak*, which is unavailable to someone who is not yet a Custodian. It gated nothing that proof-of-capacity plus a byte-scaled bond does not gate better, and it excluded exactly the population that would supply the first petabyte (`61 X13`) |
| **Relay participation** | `Level ≥ 8 ∧ Trust ≥ 300 ∧ 30d uptime attestation ∧ Stake(1,000 FRC)` |
| **Self-hosted Node registration** | `Level ≥ 9 ∧ Trust ≥ 100` |
| **Society self-hosted Node** | `SocietyLevel = 5` (Canon, `11 §2.3`) |
| **Custodian pool operation** | `SocietyLevel = 5 ∧ Achievement(TEN_NINES)` |
| **Vault quota** | Level-granted base (§5.1); additional capacity purchasable with Fraction |
| **Scheduling precedence** | `weight = f(Trust, custody success rate, Attestation history)`. **Never a function of XP, Level, or Fraction.** |
| **Security disclosure channel** | **Ungated.** No Level, Trust, or Standing requirement. A Level 0 Citizen at negative Trust must be able to report a vulnerability |

> **Capacity is purchasable. Precedence is earned.** This one line resolves every "is it pay-to-win?" argument in the infrastructure tier. Fraction buys you more disk. It never buys you ahead of someone in the queue.

---

## 6. Society Level and Reputation

### 6.1 Society XP

```
SXP_week =  Σ_{m ∈ qualifying members} min( xp_earned_in_society(m, week) , 500 )
          + 0.5 · custody_gb_months_served_for_others
          + governance_health_bonus            (§6.2, 0–400)
          + retention_bonus                    (§6.2, 0–400)
          − moderation_debt                    (overturned actions × 150)

qualifying member ::= tenure_days ≥ 14  ∧  Standing.trust ≥ 0
SXP is monotonic: SXP_week is floored at 0 before accumulation.
```

The per-member cap of 500 is deliberate: one prolific founder cannot level a Society alone. Ten moderately active members outrank one extremely active one, which is the correct incentive for a container whose value is collective.

### 6.2 `SocietyReputation` — six independent dimensions, each 0–100

| Dimension | Measure | Window |
|---|---|---|
| `vitality` | Distinct members posting substantively / member count | 30 d |
| `retention` | Members active at t−90d still active at t | 90 d |
| `governance_health` | Quorum attainment rate × proposal participation breadth | 180 d |
| `moderation_quality` | 1 − (overturned actions / total actions), floored at 0 | 180 d |
| `economic_activity` | Distinct Wallet pairs transacting / member count, log-scaled | 90 d |
| `custody` | Attestation success rate for Shards this Society serves | 30 d |

Never summed. Same conjunction rule as §2.3.

### 6.3 Society Level thresholds

| SL | SXP | Reputation floors (all must hold) | Age |
|---|---|---|---|
| 0 | 0 | — | — |
| 1 | 2,500 | `retention ≥ 30` | 14 d |
| 2 | 12,000 | `retention ≥ 40 ∧ governance_health ≥ 30` | 60 d |
| 3 | 45,000 | `+ moderation_quality ≥ 50 ∧ governance_health ≥ 45` | 120 d |
| 4 | 140,000 | `+ economic_activity ≥ 40 ∧ retention ≥ 55` | 240 d |
| 5 | 400,000 | `all ≥ 50 ∧ moderation_quality ≥ 65` | 365 d |

### 6.4 Attainment vs fitness

> **Invariant I-9:** `Society.level` is monotonic and never decreases. Capabilities gated on a reputation floor are **suspended** while the floor is unmet and restored when it is met, without a Level change.

A Society is not demoted for a bad quarter — demotion would destroy the meaning of Lineage and would make Fracture eligibility a moving target during a Fracture proposal. Instead, the Level records what the Society achieved and the reputation floors record whether it is currently fit to exercise the most consequential of those capabilities. A suspended capability shows its unmet floor and its current value in the Society's Charter surface. Nothing is hidden (A7).

A Society entering `Dormant` (`11 §4`) freezes all reputation windows rather than draining them, and resumes on the first member action. Dormancy is not a punishment.

---

## 7. Achievements and Milestones

### 7.1 The distinction

| | **Achievement** | **Milestone** |
|---|---|---|
| Nature | A named recognition of a *specific accomplishment* | A *threshold crossing* on a counter |
| Repeatable | No. Granted once, ever | Yes, in tiers |
| Criteria | Qualitative but deterministic and enumerable | A single numeric predicate |
| Facet form | A `Badge` — `Immutable` | An `Insignia` — `Tiered`, evolving in place (§5.4) |
| May grant an Unlock | Yes, if declared at introduction | No, except a cosmetic tier |
| Event | `AchievementGranted` | `MilestoneCrossed` |

Both are permanent, non-transferable, and never decay (§11.1).

### 7.2 Achievement catalog (initial set)

| Category | Id | Condition |
|---|---|---|
| Social | `FIRST_WORD` | First message posted in any Chamber |
| Social | `CONVERGENCE` | Participate in a Convergence that Crystallizes |
| Social | `QUORUM` | Be one of the founding participants named in a Crystallization |
| Social | `INTERLOCUTOR` | 100 distinct Citizens have replied substantively to you |
| Social | `BRIDGE` | Active Membership in 5 Societies simultaneously for 90 days |
| Creation | `SIGNAL` | Author a Thread with ≥ 25 distinct substantive repliers |
| Creation | `CORPUS` | 500 authored artefacts that each received `XP.ENGAGE` |
| Creation | `CARTOGRAPHER` | Author a document a Chamber adopts as its canonical reference |
| Creation | `ATELIER` | Mint 10 Facets subsequently acquired by distinct Citizens |
| Governance | `FRAMER` | Author an enacted Charter amendment |
| Governance | `ASSEMBLY` | Vote in 50 Proposals that reached quorum |
| Governance | `ARBITER` | 25 upheld Moderation Actions with zero overturned in the same 180-day window |
| Governance | `RECUSAL` | Formally recuse from a Proposal after declaring a conflict. **Awards 0 XP**, grants `Standing.governance` |
| Governance | `QUIET_HAND` | ≥ 50 Moderation Actions, none appealed, over 365 days |
| Infrastructure | `CUSTODIAN` | First settled custody window |
| Infrastructure | `TEN_NINES` | 365 consecutive days of custody with zero failed Attestations |
| Infrastructure | `MESH` | Serve Shards for 25 distinct Societies |
| Infrastructure | `NODE` | Operate a registered self-hosted Node for 90 days |
| Infrastructure | `RELAY` | Serve 1,000,000 Signals |
| Economy | `FIRST_POSTING` | First Transfer settled |
| Economy | `SETTLED` | 100 Transfers completed with zero disputes |
| Economy | `TREASURER` | Steward a Treasury across a full governance cycle without an unreconciled Posting |
| Economy | `UNDERWRITER` | Stake ≥ 1,000 FRC for 180 days without slashing |
| Agent | `OPERATOR` | Enroll your first Agent |
| Agent | `ENVELOPE` | Operate an Agent for 90 days with zero `AgentActionBlocked` |
| Agent | `RESTRAINT` | Revoke an Agent's Envelope in response to your own audit finding, before any incident |
| Agent | `AUTHOR` | Publish a Workflow installed by 10 distinct Societies |
| Longevity | `TENURE_I / II / III` | 1 / 3 / 5 years since `CitizenRegistered` |
| Longevity | `RETURNED` | Return to Active after ≥ 180 days Dormant |
| Longevity | `LINEAGE` | Hold Membership in a Society and in both children of its Fracture |
| Rare | `GENESIS` | Among the first 1,000 registered Citizens. Enumerable by nature; the only quota-limited Achievement |
| Rare | `ANTIBODY` | Report a Sybil cluster subsequently confirmed (§9.4) |
| Rare | `ZERO_DAY` | Report a verified security vulnerability |
| Rare | `DRY_RUN` | Identify an invariant violation that blocks a Fracture dry run (`11 §3.2`) |
| Rare | `UNMOVED` | Hold a Facet continuously from mint through three Evolutions |

### 7.3 Milestone counters (tiers at 1 / 10 / 100 / 1k / 10k unless stated)

Artefacts that received engagement · distinct substantive repliers · GB-months served · Signals relayed · quorum-reaching votes cast · Societies co-founded (1/3/10) · Facets minted and held by others · days of tenure (30/365/1095/1825) · Attestations settled · Citizens onboarded to Level 2 · merged code contributions · translated strings · compute credit-units contributed.

### 7.4 Rules for extending the catalog

> **Invariant I-10:** The Achievement and Milestone catalog is append-only. A grant is revoked only for proven fraud, and the revocation is itself a recorded event naming its cause.

1. A new Achievement must be **retroactively evaluable from the Log** (P6) or carry an explicit `effective_from` date. Silently prospective criteria are forbidden — they punish the people who did the thing before you named it.
2. Criteria are **versioned by identity**. Changing a condition mints a new `AchievementId`; the old id keeps its holders and its original criteria text forever. Nothing is ever made harder for existing holders.
3. A deprecated Achievement stops being grantable. It never stops being held or displayed.
4. An Achievement that grants an Unlock must declare it at introduction, and can never later lose it.
5. **Rarity is an outcome, never a target.** No Achievement is quota-limited except those enumerable by nature (`GENESIS`).
6. Society-issued Achievements (Society Level 3+) are namespaced `soc_<id>/<name>`, are visible as Society-issued, and can never gate a platform capability (P1: a Society's authority ends at its boundary).

---

## 8. Seasons

Seasons are Phase 6 (`02 §3`) and require a stable progression baseline to be additive to.

### 8.1 What a Season is

A 90-day named period with an index, carrying: **objectives** (≈ 20, all satisfiable by ordinary contribution — a Season never introduces a parallel grind), a **themed Insignia tier**, a set of **transferable Collectible Facets**, **Society Challenges** (co-operative, Society-scoped, scored on the same reputation dimensions as §6.2), a **Season accent pack** (`33 §2.5` wheel), and a **Chronicle** — a generated permanent record of what each participating Citizen and Society actually did.

### 8.2 The additive rule, and the mechanism that makes FOMO impossible

> **Invariant I-11:** No Season expires progress, resets a permanent value, or renders an Insignia or Achievement permanently unobtainable.

Season objectives remain completable **forever**, including after the Season closes. The Insignia granted records *when* the objective was met:

```
Insignia { objective: "S3/CUSTODY_WINDOWS_12",
           season_defined: 3,
           season_completed: 7 }        ← a Tiered Facet field, not a separate asset
```

Completing Season 3's objective during Season 7 grants the same Insignia with `season_completed: 7`. Nothing is lost by not being present; the record is simply truthful about time. This is the entire FOMO defence, and it costs nothing: an "I was there" signal survives, because `season_defined == season_completed` is a visible, unfakeable fact — while the reward itself stays permanently reachable.

### 8.3 What a Season may never contain

| Forbidden | Why |
|---|---|
| A countdown timer on any progression surface | A2, `02 §4` dark patterns |
| Daily or weekly login objectives, streaks, or recency requirements inside the window | A1, A2 |
| A purchasable pass that grants XP, Trust, Standing, an Achievement, or an Unlock | A3 |
| Randomized rewards of any kind | A4 |
| An objective requiring another Citizen to lose | A5 |
| Any objective not satisfiable through §3.3 sources | A1 — a Season adds naming and cosmetics, never a new XP economy |

Fraction may buy Season **Collectibles** and accent packs. It buys nothing else (I-8).

### 8.4 Closing a Season gracefully

```
  T−14d   Season content and objectives published in full, unchanged since open
  T+0     SeasonClosed emitted
            ├─ Chronicles written to each participating Log — permanent, exportable
            ├─ objectives migrate to the permanent catalog, still completable
            ├─ Season Insignia tier freezes its `season_defined`, not its availability
            ├─ accent packs stay installed for everyone who has them, forever
            └─ Society Challenge results recorded in each Society's reputation history
  T+1d    the next Season may open. There is no gap requirement and no reset.
```

A Season that closes changes nothing a Citizen holds. It only stops being the current one.

---

## 9. Anti-Abuse

### 9.1 The threat model, and the countermeasure for each

| # | Attack | Countermeasures |
|---|---|---|
| F1 | **Message-volume farming** | `δ = 0.72` geometric decay (§3.5), `Q` gate, per-source daily cap, 4,000/week global ceiling |
| F2 | **Reaction rings** | Raw reactions award **zero XP, permanently** (§9.2). There is no ring to run |
| F3 | **Reciprocal-engagement collusion** | Distinct-counterparty requirement, counterparty Trust weighting (§3.2), clustering discount (§9.3), one `XP.ENGAGE` award per engager per artefact ever |
| F4 | **Alt-account boosting** | `age(c) = 0.25` for accounts under 14 days, `w(c) = 0.5` at neutral Trust, `XP.ONBOARD` payable only when the invitee reaches L2 via *other* counterparties, invite-subtree accountability (`12 §9`) |
| F5 | **Agent grinding** | §9.5 |
| F6 | **Sybil custody** | Payment against Attestations only, erasure-verified challenge–response, self-custody pays 0, FNID-pair egress loops excluded |
| F7 | **Moderation farming** (taking easy actions for `XP.MOD`) | T+14 settlement on upholding, overturned actions pay 0 and cost Trust, `Standing.governance` gate to become eligible |
| F8 | **Governance spam** (proposal flooding) | `XP.GOV.PROP` is enactment-gated; authoring costs a refundable bond; voting on your own proposal pays 0 |
| F9 | **Compute fraud** | 3% redundant re-execution sampling; a mismatch voids the window and slashes the Stake |
| F10 | **Cross-source stacking** | The 4,000/week global ceiling binds regardless of source count |

### 9.2 Why reactions are worth zero

A reaction costs one click, proves nothing was read, and is the cheapest unit of manufacturable approval in social software. Worth even 1 XP, reaction-ring farming becomes rational at scale. Reactions stay in the product — good affordance, and they feed within-Society Discovery — but they are **not** an XP input and never will be. `XP.ENGAGE` requires a reply, quote, citation, or save: acts costing authorship or commitment, from a counterparty whose Trust weights the award.

### 9.3 Clustering discount

For a Citizen `a`, build the reciprocal-engagement graph over the last 90 days and compute the local clustering coefficient `C(a) ∈ [0,1]` of `a`'s neighbourhood.

```
R' = R · (1 − C(a))^1.5
```

A Citizen engaged with by a dense clique has `C → 1` and earns near zero from it; one engaged with by strangers who do not know each other has `C → 0` and is unaffected. This penalises the *shape* of collusion instead of requiring proof of intent, applies to `R` only — never to a Level, Trust, or an Achievement — and leaves outside engagement undiscounted.

**Honest cost:** a small, genuinely insular Society of real people earns less `XP.ENGAGE` internally than a comparable open one. Accepted. XP measures contribution *to the ecosystem*; an insular group contributes to itself, which is fine, and which is what Standing (§2.3) measures.

### 9.4 Sybil clusters

Detection is specified in `12 §9`; this chapter defines only the progression consequence. On confirmation, every identity in the cluster receives the `w = 1.00` negative Trust event (`12 §8.2`), all *pending* XP in escrow (§9.6) is voided, and all *settled* XP is retained (§2.1). The Achievement `ANTIBODY` is granted to the reporter.

### 9.5 Agents earn no XP for anyone

> **Invariant I-12:** An action taken by an Agent awards zero XP to the Agent, to its Operator, and to any Citizen whose artefact the Agent engaged with.

Agents are throughput (P4); letting throughput generate XP makes XP a measure of scripting capacity. Concretely: Agent-authored messages are not `XP.ENGAGE` counterparties; Agent replies do not count toward `XP.THREAD`'s distinct-replier threshold; Agent actions consume the Operator's gateway rate budget; Agents accrue only their own Trust (`12 §8.5`). What earns XP is *building* the Agent — `XP.AGENT`, paid when another Society installs it and the install survives 14 days. The contribution is the tool, not its output.

### 9.6 Anomaly escrow, not silent denial

```
z = (weekly_XP(citizen) − μ_cohort) / σ_cohort
z > 4  ⇒  the excess above the cohort 99.9th percentile is placed in ESCROW
          ├─ visible to the Citizen immediately, with the reason and the amount
          ├─ released automatically at T+14d if no adverse finding
          └─ voided by a reviewed finding, which names its cause
```

Escrowed XP is disclosed, not hidden (A7). A false positive costs fourteen days on part of one week's XP and nothing else: no Level is lost, because escrowed XP was never awarded and Levels are monotonic over awarded XP only.

---

## 10. The Presentation Rule

Progression feedback in this product is **ambient**. The brand is an instrument (`33 §1`), and instruments report state; they do not celebrate at you.

### 10.1 The interruption budget

| Class | Surfaced how | Interrupts? |
|---|---|---|
| XP award | The Progress Rail advances. No number, no toast, no sound | **Never** |
| Fraction accrual | Mono, Electric, tabular figures, `PENDING` until settled (`33 §3.4`) | **Never** |
| Milestone crossed | One line in the Signal stream; the Insignia Facet evolves in place | **Never** |
| Achievement granted | One line in the Signal stream; a Badge appears in the profile | **Never** |
| Season objective met | One line in the Signal stream | **Never** |
| Level-up, **cosmetic only** | Rail resets; the Level readout increments | **Never** |
| Level-up **granting a capability** | A single dismissible, non-blocking panel naming exactly what is now permitted | **Once**, at most once per session |

That last row is the only interruption in the entire progression system. It exists because the Citizen's affordances changed and they must know what they may now do. It states the capability, not the accomplishment — a status change, not a reward.

### 10.2 Exactly what animates

- **The Progress Rail** is the active-rail idiom (`33 §5.3`): a 2px inset rail on the leading edge of the profile chip, `--fn-signal` at 65%. It advances with a `280ms --fn-ease-out` width transition. That is the whole animation.
- **On level-up**, the sigil's diamond core performs its existing `45° → 225°` rotation once (`33 §4.4`) and the rail resets. No overlay, no particles, no confetti, no sound.
- **On a Trust change**, nothing animates. Trust changes are consequential and are read, not watched.
- **Reduced motion** (`33 §4.5`): the rail becomes a static fill, the rotation is omitted. Every state remains distinguishable without motion.

Banned outright, enforced by design-system review: confetti, coin showers, loot-box reveals, full-screen takeovers, default-on sound, progress bars that fill past the viewport, "almost there!" copy, any string containing an exclamation mark (`33 §7.3`).

### 10.3 The audit trail — a hard requirement

> **Invariant I-13:** Every XP quantum, Trust delta, and Standing change is traceable to exactly one domain `event_id`, and that trace is readable by the Citizen it concerns, on every front end.

```
$ fn me xp --since 7d
[ PROGRESSION ] @andrew // XP RECORD // 7d
  TS                  SOURCE       BASE   φ(n)   R      Q   AWARD  EVENT
  2026-09-01T09:14Z   XP.ENGAGE      6    0.90   0.81   1      4   01K4...9F
  2026-09-01T09:14Z   XP.MSG         4    0.52   1.00   1      2   01K4...9F
  2026-09-02T17:02Z   XP.MOD        60    1.00   1.00   1     60   01K4...J2   settled T+14
  2026-09-03T11:40Z   XP.MSG         4    0.19   1.00   0      0   01K4...QA   Q=0 duplicate
  ─────────────────────────────────────────────────────────────────────────
  AWARDED 66 · WITHHELD 8 · ESCROW 0 · WEEK 1,240 / 4,000 CEILING
```

Withheld XP appears with its reason. A Citizen can always answer "why did I earn that, and why not this" without asking anyone. The GUI renders the same rows; parity is a release gate (P13, N3).

### 10.4 What other Citizens see

| Value | To its owner | To others |
|---|---|---|
| XP | Exact | Level only |
| Level | Exact | Exact |
| Trust | Exact number | A three-state band: `ESTABLISHED` / `NEUTRAL` / `RESTRICTED` |
| Standing | Four separate readouts | Four separate readouts, per that Society's disclosure setting (`12 §10`) |
| Achievements / Insignia | All | Per disclosure setting, default: shared Societies only |

A public numeric Trust score is refused twice over: it manufactures the leaderboard the system rejects (A5), and a visible number invites campaigns to move it. The band carries everything a counterparty needs before transacting.

> **UI lint:** no component may render an XP value and a Trust value in the same visual group, and no component may compute a value from both. Violations fail the build (A6).

---

## 11. Decay, Restriction, and Forgiveness

### 11.1 The ledger of permanence

| Never decays, never resets | Decays |
|---|---|
| XP · Level · Achievements · Milestones · Insignia · Badges · Season Chronicles · `tenure_days` · Lineage | Trust (toward 0, `12 §8.3`) · `Standing.contribution` (180d window) · `Standing.governance` (180d window) · Society reputation dimensions (§6.2 windows) · scheduling precedence weight |

There is no prestige reset, no seasonal ladder reset, no inactivity penalty on XP or Level, and no mechanism anywhere that reduces a `u64` XP value.

### 11.2 Trust bands and their effects

| Trust | Band | Effect |
|---|---|---|
| `+400 … +1000` | `ESTABLISHED` | Full eligibility: custody, Relay, moderation roles, vouching budget (`12 §8.4`) |
| `0 … +399` | `NEUTRAL` | Default posture. Every Level-gated capability available |
| `−1 … −199` | `WATCHED` | High-leverage rate limits halved; no new Envelope grants above class B; moderation eligibility suspended |
| `−200 … −599` | `RESTRICTED` | No DM initiation; no Facet minting; no custody or Relay; no governance *authorship* — **voting is retained** |
| `−600 … −1000` | `UNDER REVIEW` | Charter-scoped suspension available to Societies; platform review opened |

**The crucial property:** a Level 18 Citizen at Trust −400 is still Level 18. They keep every Level-gated feature, Achievement, Insignia, and their whole history. They lose only the Trust-gated capabilities — the ones where other people's assets, safety, or Shards are at risk. This is the three-axis model paying for itself: the consequence is scoped exactly to what was violated, with no retroactive erasure.

Voting is retained at `RESTRICTED` deliberately. Disenfranchisement is a governance act, not a reputation consequence; a Society wanting it must say so in its Charter and enact it (P4).

### 11.3 Standing on inactivity

`Standing.contribution` and `Standing.governance` roll off their 180-day windows and reach zero after 180 days of inactivity in that Society. `tenure_days` and Society-local Trust do not. A returning Citizen therefore returns with long tenure, intact Trust, and no current contribution — an accurate description of a returning member, conveyed precisely *because* the four readouts were never summed. `Departed` zeroes forward-looking Standing and retains authored history (`11 §4`).

### 11.4 The redemption path

```
  ①  CAUSE IS NAMED         every Trust decrement carries its event_id and reason;
                            readable at fn me trust --log
                  ▼
  ②  TIME                   negative Trust decays on a 365-day half-life,
                            suspended while any negative event lands within 30 days
                  ▼
  ③  REMEDIATION            Charter-defined restorative acts — an appeal-mandated
                            action, a restitution Posting, a supervised custody
                            window — emit positive Trust events directly
                  ▼
  ④  BOUNDED VOUCHING       up to 3 active vouches from the established core;
                            each costs the voucher budget + a slashable bond,
                            and a reoffence within 180 days cascades to them (12 §8.4)
                  ▼
     RECOVERED               with one honest cost: a Citizen who has been below
                            −600 may not exceed +400 for 365 days after returning
                            to 0. The ceiling is disclosed, dated, and visible.
```

Stage ④ makes the path social rather than merely temporal: someone with a record must stake their own reputation on you. Expensive by design, and flow-capped so it cannot become a Sybil laundering channel.

---

## 12. Trade-offs and Rejected Alternatives

| Rejected | Why it is attractive | Why it is refused |
|---|---|---|
| **A single karma / reputation score** | One number, one API, one UI element, trivially sortable | The historical failure mode of every social reputation system. One score must answer "how much did you do" and "can you be relied on" at once, and those have opposite gaming profiles: volume is cheap to manufacture, reliability is not. Merging them prices reliability at the cost of volume. Banned by `01 §7` |
| **Streaks and daily objectives** | The strongest known retention mechanic in consumer software | Manufactures anxiety and punishes absence — converts a platform into an obligation. On the Never list (`02 §4`), and incompatible with P2: an offline week must cost nothing |
| **Leaderboards as a primary surface** | Cheap motivation; visible social proof | Ranking turns contribution into competition and competition into farming, and makes the top of the board a harassment target. Comparative views are opt-in, Society-scoped, never on XP alone (A5) |
| **Purchasable XP or Trust** | Direct revenue; trivial to implement | Pay-to-win, on the Never list (`02 §4`). It also destroys the only thing XP is for: bought XP is not evidence, and every gate built on it becomes a price list |
| **Prestige / seasonal resets** | Extends the curve indefinitely; re-monetises the early game | Deleting earned progress to sell it back is the purest treadmill. Also breaks I-7 (a reset Citizen loses capability they relied on) and P6 (a projection contradicting its Log) |
| **Model-scored quality as an XP multiplier** | Genuinely better signal than a binary gate | Non-deterministic (P6), non-inspectable (A7), and puts a `ModelProvider` in the domain layer (P5). Rejected for the binary `Q` gate plus Trust-weighted peer engagement (§3.4) |
| **Downvotes as a Trust input** | Fast, cheap negative signal | Trivially weaponisable and indistinguishable from disagreement. Trust moves only on evidenced, appealable, replayable events (`12 §8.2`) |
| **Public numeric Trust** | Maximum legibility | Leaderboard dynamics plus a harassment vector. The three-state band (§10.4) carries the decision-relevant information (§10.4) |
| **XP decay for inactivity** | Keeps Levels "current"; discourages coasting | XP is historical fact; erasing it is dishonest. Currency is already measured by Trust and windowed Standing. Doing it twice just makes XP a worse Trust |
| **Society Level demotion** | Symmetry with reputation floors | Demotion makes Lineage meaningless and moves Fracture eligibility underneath an in-flight Fracture proposal. Suspending the affected capability gets the same safety with none of the instability (§6.4) |

### 12.1 Known failure modes of this design, stated honestly

1. **The curve is guessed.** The `100·L^1.5` shape, the 4,000/week ceiling, and every `B` and `δ` in §3.3 are informed estimates and will be wrong. All are constants in one versioned policy table over a replayable history (P6), so recalibration is a re-projection, not a migration — and it may never *lower* a Citizen's Level: curve changes apply to future awards only.
2. **`XP.ENGAGE` under-rewards excellent work in small Societies.** A brilliant post seen by nine people earns little. Mitigated by Standing (Society-scoped, scale-blind) and `XP.CURATE`. Not solved.
3. **The clustering discount penalises genuine insularity** (§9.3, stated cost).
4. **Level 12 as the capability ceiling makes Levels 13–30 optional.** Some Citizens will stop caring at 12. Acceptable — that is what "not a treadmill" looks like in practice.
5. **Trust's 180-day positive half-life penalises seasonal contributors.** A Custodian running a node six months a year drifts toward neutral off-season. Accepted: Trust is a present-tense claim (`12 §8.3`).

---

## 13. Invariants

These become property tests in `fractal-domain-progression` and run on every PR.

| # | Invariant |
|---|---|
| I-1 | `Xp` is monotonic. No code path decrements it. The type exposes no `sub` |
| I-2 | `Level` is a pure function of `Xp` and is never assigned directly (`11 §2.1`) |
| I-3 | No `TrustAdjusted` event has a causation chain containing XP, Level, Fraction, or a purchase (`11 §7.8`, `12` I-12.8) |
| I-4 | No `Standing` value is ever reduced to a single scalar. `StandingRecord` implements neither `Ord` nor `total()` |
| I-5 | Every `XpAwarded` names exactly one source code from §3.3 and one causing `event_id` |
| I-6 | Weekly XP per Citizen never exceeds 4,000, across all sources, in any generated history |
| I-7 | No Unlock above Level 12 confers authority over another principal, a rate, a quota, or an economic advantage |
| I-8 | No Facet acquirable by Transfer is an input to any gate |
| I-9 | `Society.level` is monotonic; reputation floors suspend capabilities, never Levels |
| I-10 | The Achievement catalog is append-only; grants are revoked only for proven fraud, with a recorded cause |
| I-11 | No Season expires progress or makes an Insignia permanently unobtainable |
| I-12 | Agent actions award zero XP to every principal |
| I-13 | Every progression change is traceable to one `event_id` and readable by its subject on every front end |
| I-14 | Every progression projection is reproducible by replaying the Log from its most recent verified checkpoint, and from zero within the published rebuild SLO (P6, `11 §7.10`, `40 §9.4`) |
| I-15 | Trust-gated and Standing-gated Unlocks suspend when their gate is false; Level-gated Unlocks never suspend |
