# 61 — Reconciliation

> **Prerequisites:** `60-self-critique.md`, and the Canon (`00`, `01`, `02`, `03`).
> **Governs:** the authoritative resolution of every cross-chapter contradiction and every high-severity structural weakness `60` identified, plus the ones this pass found while resolving them. Where `60` states a problem, this chapter states what is now true.
> **Reading note:** `60` is a critique and is deliberately non-directive. **This chapter is directive.** A ruling here overrides the chapters it names, whether or not the edit has landed yet — the Status column says which.

---

## 1. How to Read This Ledger

Every entry has the same six parts, and the ruling is the load-bearing one.

| Part | What it is |
|---|---|
| **ID** | `X<n>` a contradiction from `60 §5`; `W<n>` a weakness from `60 §3`; `N<n>` found during this pass; `D<n>` an open decision for Andrew |
| **Conflict** | One sentence. If it takes two, it is two entries |
| **Ruling** | What is now true. Decisive — one side, or a third answer better than both |
| **Rationale** | Tied to a principle number, a Canon section, or arithmetic |
| **Changes** | The chapters and sections that must move |
| **Status** | `APPLIED` this pass · `DEFERRED → PH<n>` with the exact change specified in §5 · `OPEN → D<n>` needs Andrew |

**Ledger size.** 49 rulings — 18 cross-chapter contradictions from `60 §5`, 15 found while resolving them, and 16 structural weaknesses from `60 §3`. Fifteen are fully applied; twenty-six are applied in part with the remainder scheduled in `03 §2`; eight are deferred with the exact change written out in §5; six open decisions await Andrew in §6. Every edit made in this pass is enumerated in §7.

A ruling that is `DEFERRED` is still a ruling. Deferral means the *edit* is too large to make safely in a mechanical pass, not that the question is still open. §5 states the exact change so the deferred edit is transcription, not rethinking.

---

## 2. Cross-Chapter Contradictions

### X1 — The worked ADR example is numbered 0009, which is the Facet standard

**Conflict.** `40 §6.4` reproduces a worked ADR for deterministic simulation numbered **ADR-0009** and dated 2025-11-14; the corpus assigns 0009 to the native Facet asset standard and **0014** to deterministic simulation, both dated 2026-09-03. The reproduced text also cites "integration tests (§6.6)", which are `40 §7.6`.

**Ruling.** The worked example is **ADR-0014**, dated **2026-09-03**, and its internal reference is **§7.6**. The block is additionally marked as a transclusion point so that the copy and `adr/0014-deterministic-simulation-testing.md` cannot diverge again: `cargo xtask lint-docs` renders the fenced block from the ADR file and fails on a diff.

**Rationale.** `40 §5` forbids documentation drift and `40 §4` already enforces terminology by lint; a duplicated document with no generator is drift with a mechanical fix. This is the corpus's most visible self-contradiction — a chapter that teaches ADR discipline containing a mis-numbered ADR.

**Changes.** `40 §6.4`.  **Status: APPLIED.**

---

### X2 — The Policy Enforcement Point is placed inside the wasm core

**Conflict.** `34 §2.1` draws `fractal-app` — explicitly "commands, queries, **policy PEP**" — inside THE CORE, "identical bytes of logic on every target", where the targets include the browser. `41 §8.1` marks `fractal-app-*` **✖ by design** for `wasm32` precisely because "someone will eventually run policy enforcement in the browser, which `10 §8` forbids".

**Ruling.** **`41` is right.** `fractal-app-*` and `fractal-api-*` never compile to `wasm32`. The wasm core contains exactly:

```
  fractal-types · fractal-macros · fractal-schema · fractal-ports (traits only,
  no impls linked) · fractal-domain-*  ·  fractal-core (feature "wasm")  ·
  fractal-core-wasm · fractal-sync · fractal-store · fractal-crypto ·
  generated tokens · generated API client
```

and does **not** contain `fractal-app-*`, `fractal-api-*`, any `fractal-adapter-*`, `fractal-node`, or `fractal-cli`.

The nuance the diagram was reaching for, stated properly: the client legitimately needs to know whether an action is likely to be permitted, so it can grey out a control rather than offer it and fail. That is served by `fractal-domain-agent`'s pure decision function, which *does* compile to wasm — but its output is an **advisory affordance hint**, never a decision. Every command is re-evaluated at the authoritative PEP in `fractal-app-kernel`, server-side, on the path every front end takes. A hint that disagrees with the PEP is a UI defect, never an authorization outcome.

**Rationale.** P8 (deny by default; the enforcement point must be inside the trust boundary) over P13 (one core, many front ends). `10 §8`'s architectural commitment is unambiguous: "not in the agent, not in the gateway, not in the front end". The conflict order in `00 §2` puts P8 first and P13 eleventh.

**Changes.** `34 §2.1` core box and its caption; the diagram joins the doc-drift detector.  **Status: APPLIED.**

---

### X3 — Redis appears in three chapters and in neither the port table nor the topology

**Conflict.** Redis is in `10 §1`'s system diagram, `14 §2`'s presence table and `40 §7.6`'s integration dependencies, and in neither `10 §7`'s port table — declared "the complete swappable list" — nor `10 §9`'s deployment topology.

**Ruling.** **Redis is removed.** Presence is held in **Relay-process memory** at a 45-second TTL with a 15-second heartbeat, and gossiped between Relay instances over the existing NATS subject `presence.<society_id>`. No `EphemeralStore` port is added. A Relay restart loses presence, which self-heals within one heartbeat and is invisible to a Citizen.

**Rationale.** P5 requires two implementations at a boundary's creation; presence has no second implementation worth having and no durability requirement, so a port here is abstraction without a caller (`02 §7`). `10 §2`'s operational-surface argument is decisive: a fourth stateful system costs a permanent operational burden and a `02 §5` dependency slot to store data whose loss is undetectable.

**Changes.** `10 §1` diagram, `10 §11` (the incidental Redis Streams mention is a rejected-alternative and stands), `14 §2` presence row, `40 §7.6` integration dependency list.  **Status: APPLIED.**

---

### X4 — `Quanta` is `u128` in one place and `i64` in another

**Conflict.** `16 §2.1`, `16 §4.4`, `16 §5` and ADR-0006 §2 say `u128`. `17 §2.1` says `i64` persisted with `i128` intermediate, and gives arithmetic for it.

**Ruling.** **`Quanta` is `i64`.**

| Property | Ruling |
|---|---|
| **Type** | `pub struct Quanta(i64);` — a newtype in `fractal-types`, with no `From<f64>`, no `From<u128>`, and no `as` conversion anywhere in the tree |
| **Signedness** | **Signed.** Non-negotiable, and the dispositive argument: `11 §2.6` and `16 §19` LA1 require the `EmissionAccount` to hold a **negative** balance mirroring total supply, and `11 §7.4` makes total supply directly queryable as `-EmissionAccount.balance`. An unsigned type cannot represent the one row the whole supply invariant is built on |
| **Width** | **64-bit.** `1e9 FRC × 1e9 quanta = 1.000e18`; `i64::MAX = 9.223e18`; headroom **9.22×**, which absorbs the emission mirror, every plausible cap amendment, and the K = 64 `EmissionAccount` shards |
| **Intermediate** | `i128` for every multiply-then-divide, every batch sum, and every ppm computation (`17 §2.3`). `i128` is permitted in expressions and **forbidden in storage and on the wire** |
| **Overflow policy** | **Checked, never wrapping, never saturating.** `checked_add`/`checked_sub`/`checked_mul` throughout; an overflow returns `LedgerError::Overflow` and aborts the whole batch. A saturating ledger silently invents or destroys Fraction, which is a P12 violation dressed as robustness. The only permitted narrowing site is `i64::try_from(i128)` at the boundary of a completed computation, and it is `?`-propagated, never `unwrap`ed |
| **Compile-time assertion** | `const _: () = assert!(SUPPLY_CAP_QUANTA <= i64::MAX / 9);` in `fractal-domain-ledger`, so a cap amendment that eats the headroom fails the build rather than production |
| **Wire format** | **A JSON string of the decimal integer quanta count** (`"12345678901234567"`), never a JSON number. IEEE-754 doubles are exact only to 2^53 ≈ 9.0e15; the supply cap is 1e18, three orders past that, so a JSON number would silently round balances in every JavaScript client. gRPC: `sint64`. Canonical deterministic encoding for `state_root` (`16 §6.2`): fixed-width **8-byte big-endian two's complement** |
| **Storage** | Postgres `bigint`. Not `numeric` — `numeric` is arbitrary-precision, slower, and invites the `f64` conversion the lint exists to prevent |

**Rationale.** P12 first: the negative `EmissionAccount` is how "total supply is a directly queryable number, not an estimate" is true, and unsigned makes it false by construction. P10 and N2 second: the domain compiles to `wasm32` from PH0, where 128-bit arithmetic is emulated and slow, and the ledger is on the hot path of every economic operation. `60` I2 is right that this becomes unfixable after the first Posting, which is why it is PH1 and not later.

**Changes.** `16 §2.1` (`Posting.amount` comment), `16 §4.4` (validate step), `16 §5` (the forbidden-practices table), ADR-0006 §2 and §7.4, `17 §2.1` (unchanged, now the source).  **Status: APPLIED.**

---

### X5 — The marketplace fee is 12% in one chapter and 5% in another

**Conflict.** `19 §6.1` sets 12% standard / 10% services / 4% launch with a 70% creator floor, split 6 pp burn / 4 pp Operations / 2 pp Assurance. `17` K7 sets 5% split 40% burn / 40% seller's Society Treasury / 20% Platform Reserve. `50 M6.3` follows `19`. Separately, `19 §13` T6 says a detected self-purchase "pays 100% platform fee", which makes `19 §16` invariant 4 (creator ≥ 70% on every settled purchase) false.

**Ruling.** **12%, per `19 §6.1`.** `17` K7 is regenerated from `19 §6.1` and stops being an independent claim. Concretely:

```
  K7  Marketplace fee — Sink::MarketplaceFee
      amount       12% of gross  (10% for C9 Services; 4% launch rate on a
                   verified creator's first 10,000 FRC lifetime gross)
      disposition  50.0% burn  ·  33.3% Operations Account  ·  16.7% Assurance
                   Reserve      (i.e. 19 §6.1's 6 / 4 / 2 pp of gross)
      phase        PH6
```

The Treasury leg is **not** part of K7. `17` gave the seller's Society 40% of the fee; `19` gives the hosting Society 0–10% **of gross**, and only on a Shelf-originated sale. These are different mechanisms with different triggers and different payers, and only one of them can be K7. Ruling: the Shelf share is a separate row, **K7b `Sink::ShelfShare`**, disposition **Redistribute** — and therefore never counted in the Sink Coverage Ratio (`17 §1.3` M1), which is the whole point of `17 §4`'s three-disposition taxonomy. The affiliate/curator share is not a Sink at all: it is a Posting from the creator's leg to the affiliate inside the same balanced batch, with `PostingReason::AffiliateShare`.

**Self-purchase is void, not fee'd.** A purchase the settlement code detects as a self-purchase (shared device key, funding-graph proximity, reciprocal pattern per `19 §13` T6) is **refused at the saga**; if detected after settlement it is **reversed in full** under R2, including the burn reversal. A 100% platform fee is a punishment mechanism that makes an invariant false in order to punish, and `19 §16` invariant 4 is worth more than the punishment.

**Rationale.** P12 (every Fraction has a named source and a named sink, and a redistribution is not an anti-inflation mechanism) plus `19`'s ownership: `19` owns the market, `17` owns the emission ceiling and the Sink taxonomy. Cross-chapter ownership is settled by which chapter's model breaks if the number changes — `19 §6.3` builds an eight-row competitive justification on 12%, and nothing in `17` depends on 5%.

**Changes.** `17 §4` K7 row plus a new K7b row; `19 §13` T6 mitigation text; `19 §16` invariant 4 (unchanged, now satisfied).  **Status: APPLIED.**

---

### X6 — Storage rates, replication factor, free quotas and emission share disagree across `13`, `17` and `18`

**Conflict.** Five numeric disagreements, each independently load-bearing: the tariff (17.5×), the custody rate (6.1×), the replication factor (1.6× vs 3×), the free quota (9×), and the storage share of the emission ceiling (20% vs 32%).

**Ruling.** Six decisions, and then the corrected arithmetic.

**(a) The anchor is administered and stands.** `17 §1.2`'s E1 — 1 FRC ≡ 1 GB-month of Vault storage at the reference tariff — is the definitional invariant of the currency and does not move. K8 charges **1.00 FRC per logical GB-month above quota**.

**(b) The Custodian payout price is derived, not administered.** `13 §8.2`'s mechanism is correct: cap first, divide pro rata. What was wrong is that its worked example **assumed** `cap_w = 5,000 FRC/day` and then reported the resulting 46.8 FRC/TiB-month as if it were a rate. It is not a rate; it is a quotient. The cap is the free variable, and it comes from `17 §3.2`'s S1 share of the emission schedule, not from `13`.

**(c) The replication factor in every economic model is 1.60×.** `13 §6.1` fixes RS(10,16) at 1.60×; `17 §3.3`'s 3× is a modelling error that overstates the Custodian bill by 1.875× and propagates into every S1-derived figure in `17 §5.2`.

**(d) S1 pays 0.28 FRC per replica-GB-month, unchanged.** This number is load-bearing in two other places — `17 §2.1`'s five-significant-digits argument for 1e-9 precision, and `13 §12`'s peg constraint — and changing it would ripple further than the error it fixes.

**(e) σ becomes a floor, not a target.** `13`'s Invariant V5 says the Sink rate per byte-hour exceeds the Source rate per byte-hour by a platform constant σ > 1, "initially 1.25". At the corrected rates σ is 2.23, which satisfies V5 comfortably. V5 is restated as **σ ≥ 1.25**, a floor the settlement code asserts, rather than a constant it is supposed to equal.

**(f) The storage share of the ceiling is `17 §3.2`'s: S1 22%, S2 10%.** `13 §8.2`'s "at most 20%" and its absolute annual bound of 1,825,000 FRC are stale by 24× and are deleted; `13 §8.2` now derives `cap_w` from `17`'s share, which is what its own text already says `17` owns.

**The corrected arithmetic, end to end.**

```
  RATE RECONCILIATION
    Sink charged to a Society (K8, E1)          1.00   FRC / logical GB-month
    Replication (13 §6.1, RS(10,16))            1.60   x
    Source paid to Custodians (17 S1)           0.28   FRC / replica GB-month
    Source per LOGICAL GB-month  0.28 x 1.60  = 0.448  FRC
    sigma = 1.00 / 0.448                      = 2.23   >= 1.25  ✔ V5 holds

  DAILY CAP, derived from 17 §3.2 and not assumed
    B(1)  (17 §5.1 R2)                          200,000,000 FRC / yr
    S1 share 22%                                 44,000,000 FRC / yr
    cap_w = 44,000,000 / 365                        120,548 FRC / day
    (identical to the figure 17 §3.3 already prints — the two chapters
     agree the moment 13 stops assuming its own cap)

  PEG CONSTRAINT, recomputed (13 §12)
    Custody revenue  0.28 x 1024 GB/TiB       =     286.7 FRC / TiB-month held
    Custodian marginal cost (13 §12)          =      $2.04 / TiB-month
    2x margin requirement                     =      $4.08
    Required FRC utility value  4.08 / 286.7  =    $0.0142
    (13 §8.2 previously required $0.087 — the corrected model needs
     6.1x LESS purchasing power to recruit honest capacity)

  FREE QUOTA, re-scaled (18 §5.2 ladder replaced)
    OLD  SL0 10 GB · SL1 100 GB · SL2 1 TB · SL3 10 TB · SL4 UNMETERED
    NEW  SL0  5 GB · SL1  25 GB · SL2 100 GB · SL3 500 GB · SL4 2 TB · SL5 5 TB
    4,000 Societies at 60 / 30 / 9 / 1 % over SL0..SL3:
       2400x5 + 1200x25 + 360x100 + 40x500        =    98,000 GB
       (18's old ladder gave 904,000 GB — 9.2x the modelled 100,000)

  TEN-YEAR MODEL, MONTH 12, RE-RUN  (replaces 17 §3.3)
    Citizens 50,000, Level-weighted per 18 §5.1
       35,000 x 1 GB + 10,000 x 5 + 4,000 x 25 + 1,000 x 100
                                                 =   285,000 GB
    Societies 4,000, Level-weighted per the NEW ladder
                                                 =    98,000 GB
    Paid storage above quota                     =   120,000 GB
    ------------------------------------------------------------------
    Logical storage                              =   503,000 GB
    x replication 1.60                           =   804,800 replica-GB

    Custodian owed  804,800 x 0.28               =   225,344 FRC / month
    Settlement Pool 120,000 x 1.00 (K8)          =   120,000 FRC / month
    ------------------------------------------------------------------
    EMISSION (residual only)                     =   105,344 FRC / month
                                                 =     3,464 FRC / day
    S1 daily ceiling (Y1)                        =   120,548 FRC / day
    Utilization                                  =       2.9 %
    FORFEITED, permanently                       =   117,084 FRC / day
```

**Do the economics close?** Not on their own, and this is the parameter change that makes them close. `60 §3.2`'s sharpest observation is that the free grant scales with population **and** with monotonically increasing Society Level (`18` I-9), so storage emission grows rather than converging to zero as `17 §3.3` claims. Re-scaling the ladder fixes the size of the error, not its sign. The fix is a **binding aggregate constraint**:

```
  Invariant E14 (the free-grant ceiling)

     SUM(free_GB) x replication x S1_rate   <=   0.60 x S1_share_of_B(n)

  At Y1:   (285,000 + 98,000) x 1.60 x 0.28  =    171,584 FRC / month
           0.60 x 44,000,000 / 12            =  2,200,000 FRC / month
           utilization                       =        7.8 %

  When the left side reaches the right side, the Level-indexed free ladder is
  re-derived downward at the next epoch with 90 days' published notice (17 §13's
  re-basing procedure). The grant is a Level-indexed allowance denominated in the
  anchor unit, not a number of gigabytes fixed forever.
```

That constraint is what makes "storage emission converges to zero" a mechanism instead of a hope, and it is what stops `60 A11` — "free storage quotas are affordable for a pre-revenue platform" — from being an unstated assumption.

**Also ruled:** `18 §5.2`'s SL4 grant of **"unmetered Vault (cost-settled)" is deleted.** It contradicts `50 PH2`'s mitigation "no unmetered storage, ever", it makes E14 unenforceable by construction, and an unmetered grant funded from emission is the mechanism by which a well-designed economy still runs out of money.

**Rationale.** P12 throughout: bounded, measured, published emission, and a contribution metric that cannot be farmed. The free grant was the unbounded term.

**Changes.** `13 §8.2` (worked example, annual bound, peg constraint), `13 §14` V5, `17 §1.2` (unchanged), `17 §3.3` (worked model), `17 §12` (new invariants I-E11 storage rate agreement, I-E12 modelled quota equals `18`'s ladder over the real Level distribution, I-E13 S1+S2 share within `17 §3.2`, **I-E14** the free-grant ceiling), `18 §5.1`, `18 §5.2`, and a new `economy/rates.toml` owned by `17` from which `13 §8.2` and `18`'s quota tables are generated.  **Status: `18 §5.2` ladder and SL4 deletion APPLIED; the rest DEFERRED → PH1, specified in §5.2.**

---

### X7 — Society creation requires Level 3, and the spine sentence requires it on day one

**Conflict.** `18 §5.1` places "Found a Society" at Level 3 and `30 §4.3` #6 encodes `society.create (Level ≥ 3)`; `18 §4.3` gives time-to-L3 as 2 weeks / 5 days / 2 days. `02 §2`'s spine sentence is "**A Citizen can create a Society**…" and `50 PH1` AC-1 requires registration → Society creation → first message in under 3 minutes, unassisted.

**Ruling — the first-hearth exemption.** Every Citizen may found **exactly one** Society at **Level 0**, at no Fraction cost (`17` K1 already prices the first Society at 0 FRC). Founding a second and every subsequent Society requires **Level ≥ 3** and costs **250 FRC** (`17` K1). Four details that make it a gate rather than a loophole:

1. The allowance is a **one-time, non-transferable, per-FNID** grant, consumed at `SocietyCreated`.
2. It is **not restored** if that Society is later Dissolved, Archived, or the founder departs. A renewable first-hearth allowance is a renewable Sybil resource; a consumed one is not.
3. A **Crystallization does not consume it** (`11 §3.1`). A Convergence that crystallizes required ≥ 3 participants, ≥ 48 h or ≥ 100 messages, and ≥ 2 participants at Level ≥ 1 — that is a stronger Sybil gate than Level 3, applied to the group rather than the individual.
4. The capability string becomes `society.create` for the first and `society.create` with a `Level ≥ 3` predicate thereafter; the API returns `403` with a problem document naming the remaining requirement, per `30 §6`.

**Rationale.** `18 §1.1`'s J1 Sybil argument is correct and so is `02 §2`, and both survive because **the farmable quantity is Society *volume*, not the first Society**. An attacker who can create one Society per identity is limited by the identity system, which is where that defence belongs (`12 §9`); an attacker who can create a thousand per identity is limited by nothing. Placing the gate on volume puts it where the adversarial capability actually is. Conflict order: P1 (the Society is the atomic container and must be reachable) and P12 (the gate must not be farmable) are both served; nothing is traded.

Secondary rationale, from `60 §3.13`: every recovery failure today creates a Level-0 Citizen who cannot found a Society, inside the 72-hour new-Citizen envelope — which manufactures exactly the population profile the Sybil defences are tuned to suppress. The first-hearth exemption removes that second-order harm.

**Changes.** `18 §5.1` (L0 and L3 rows), `11 §2.3` (note), `30 §4.3` #6 capability predicate, `51 §16` Q1 (resolved), `17` K1 (unchanged, already correct), `50 PH1` AC-1 (now reachable).  **Status: APPLIED for `18` and `11`; `30 §4.3` DEFERRED → PH1 (§5.4).**

---

### X-GA — PH1 has no Fraction source but needs a working Wallet

**Conflict.** `50 PH1`'s complexity budget is explicit — "0 economic Sources (Fraction exists and moves; nothing emits it yet)" — while M1.5 ships Wallets, Transfers and Postings, M1.6 ships `ContributionReceipt`, and `51 §5.5`–`§5.6` ship a Wallet surface and a TransferSheet. A wallet with no possible balance is a screenshot.

**Ruling — `PostingReason::GenesisAllocation`.** A bounded, published, non-Source allocation, specified completely:

| Property | Value |
|---|---|
| **Grant** | **100 FRC** to each new Citizen's global Wallet at `CitizenRegistered`; **250 FRC** to each new Society Treasury at `SocietyCreated` |
| **Idempotency** | Once per principal, idempotent on `(principal, GenesisAllocation)`. A Society that Fractures does not re-grant to its children |
| **Aggregate cap** | **50,000,000 FRC** — 5% of the R1 lifetime cap — enforced as a hard stop. When the pool is exhausted new principals receive zero and the client says so plainly. "Bounded in principle" is not bounded |
| **Accounting** | Posted **from the `EmissionAccount`**, exactly like emission. `11 §7.4` (`total supply == -Σ EmissionAccount[i].balance`) therefore holds **unchanged**, with no special case. This is the improvement over `51` Q2's separate `GenesisAccount`, which would have required a second negative-balance exception and weakened the corpus's cleanest invariant |
| **Schedule** | Drawn against `B(1)` (`17 §5.1` R2), so it cannot inflate beyond the published curve. At 50,000 Citizens and 4,000 Societies the draw is 6,000,000 FRC — 3% of `B(1)` |
| **Anti-Sybil** | The Citizen grant is credited to `wallet.locked` and unlocks at **Level 1**. A registration farm cannot sweep it, and Level 1 requires actual participation (`18 §4.2`). The Society grant is unlocked, because the Society Treasury is Charter-governed and the first-hearth allowance already bounds Society count |
| **Disclosure** | Its own line in the public supply dashboard (`17 §11`) from the day the dashboard exists, and its own row in the Source/Sink registry marked **not a Source** |
| **Retirement** | Issuance **stops at the PH4 exit gate**, when S1 and S2 are live. The `PostingReason` variant remains in the closed enum forever, marked `retired` — old events are historical fact and are never rewritten (`10 §5`) |
| **Budget** | **Consumes no `02 §5` Source slot.** It has no rate, no formula, no Contribution Score input, no recurrence, and no per-epoch budget. Its only adversarial surface is registration Sybil, which is `12 §9`'s problem and is not made worse by 100 locked FRC |

**Rationale.** P12 requires every Fraction to have a named source and a named sink, and this has one — a named, capped, published, retired allocation is more economically honest than a Source that exists only to make a screen non-empty. `02 §5`'s Source budget exists because "each Source is an attack surface requiring its own simulation"; an allocation with no formula has nothing to simulate beyond its cap, which is a single assertion. `00 §3`'s standing bias — correctness over velocity — is served by keeping `11 §7.4` free of exceptions.

**Changes.** `17 §4` (a new pre-registry row), `11 §2.6` (`PostingReason` note), `16 §20`, `51 §16` Q2 (superseded — the grant is larger, capped, and Citizen-inclusive), `03 §2.2`.  **Status: DEFERRED → PH1, specified in §5.3. The ruling is binding now.**

---

### X8 — Phase assignments contradict across eight chapters

**Conflict.** `50` versus `41 §5.3`/`§5.4`/`§5.5`, `30 §4.2`, `17 §3.2`, `19 §17`, `20 §2`, `21 §14` and `16 §20`, on progression, governance, agent, economy, discovery, transcoder, rail, vault, listings, workflows, S4 accrual and internal anchoring.

**Ruling.** **`03-phase-authority.md` and `docs/phases.toml` are created as the fourth Canon file and its machine-readable companion**, carrying 203 capability rows across PH0–PH9. Every phase column in every other chapter becomes a generated, non-normative convenience view; `cargo xtask lint-phases` fails the build on a hand-authored phase claim, and `cargo xtask phase-check` fails a Work Unit that targets an out-of-phase capability. Both xtask commands land in **M0.2** with the other Canon lints, because a mechanism that arrives after the drift is a post-mortem.

Every individual disagreement is resolved in `03 §2` and summarized in `03 §3`. The eleven cases where `50` itself was overridden are marked `[overrides 50]` in `03 §2` with the reason.

**Rationale.** `02 §6` question 2 is unanswerable with eight roadmaps, which converts `00 §6`'s "never expand scope beyond the current phase" from a rule into a coin flip. `40 §4` already solved exactly this for terminology, `41 §7` for dependency direction and `32 §2` for design tokens; this is the same treatment applied to the third thing the corpus states in more than one place.

**Changes.** New `docs/03-phase-authority.md`, new `docs/phases.toml`, `00 §6` (three files → four), `02` header and `§3` deferral table, plus the eight generated phase columns.  **Status: APPLIED for `03`, `phases.toml`, `00 §6` and `02`; the eight generated columns are DEFERRED → PH0 with the generator (§5.1).**

---

### X9 — Performance budgets conflict across four chapters, each claiming ownership

**Conflict.** `32 §8` states the budgets and is declared to own them; `40 §13.1` restates them; `34 §11` states a third set and says the source is `perf/budgets.json`; `31 §10` states a fourth for the CLI. Eight numbers disagree, and `50 PH2` AC-2 has already encoded one of them as a gate.

**Ruling.** **`32 §8` is authoritative.** `perf/budgets.json` is **generated from `32 §8`**, not the other way round — `34 §11` has the direction of authority backwards. `cargo xtask budgets --render` regenerates the tables in `40 §13.1`, `34 §11` and `31 §10`, and `cargo xtask lint-docs` fails on a hand-authored budget number, the mechanism `40 §4.2` already applies to colour literals.

Ownership is then partitioned so that "authoritative" does not mean "`32` must state a battery figure for iOS":

| Table | Owns | May never |
|---|---|---|
| `32 §8` | The cross-platform client rows: cold start, warm start, interaction→paint, frame budget, initial JS, route chunk, memory ceiling, font payload — for **web** and **desktop** | — |
| `34 §11` | Per-target **extensions** `32` does not state: Android, iOS, Linux, macOS, PWA, battery, binary/installer size, IPC/FFI cost, and the measurement mechanism for each | Restate a `32 §8` row with a different value |
| `31 §10` | The CLI rows | Restate a `32 §8` row |
| `40 §13.1` | The **server-side** rows: API read/write p99, event append, ledger posting, Envelope evaluation, Signal end-to-end, projection lag, Runtime memory, and the enforcement mechanism for all of them | Restate a client row at all — it references, it does not copy |

**The eight conflicting numbers, resolved.** Three of them were never conflicts — they were two different measurements sharing a label, which is the more useful finding:

| Metric | Ruling | Note |
|---|---|---|
| Desktop warm start | **400 ms** | `32`. `34`'s 600 ms corrected |
| Web warm start | **800 ms** | `32`. `34`'s 900 ms corrected |
| Desktop memory | **450 MB RSS at 5 Societies, 10 k cached Messages** | `34`'s figure — stricter *and* better specified. `32`'s 600 MB corrected. **`50 PH2` AC-2 must change from "≤ 600MB under a 4-Society soak"** |
| Web memory | **350 MB** | `34`. `32`'s 400 MB corrected |
| Initial JS, first route | **180 KB gz** | `32` and `40` agree. `34`'s 220 KB was the *total initial payload* under a JS label |
| Total initial payload | **400 KB gz** | `40`. Not a conflict once the two are named separately |
| CLI binary | **12 MB compressed artifact, 18 MB on disk** | `31` measured compressed, `34` measured on disk. Both are true; both are stated |
| CLI memory | **80 MB RSS at 1 Society, 120 MB at 4 Societies** | `34` omitted the Society count, `31` stated it. Both are true; both are stated |

**Rationale.** P10 is a hard gate within its own domain (`00 §2`), and a gate whose number depends on which chapter the reader opened is not a gate. `32` owns the numbers because it is the design system and N7 already makes it single-source for the token pipeline; `40` owns enforcement because it owns CI.

**Changes.** `32 §8` (desktop and web memory rows), `34 §11` (warm-start and JS rows, plus a pointer), `31 §10` (a pointer), `40 §13.1` (a pointer), `50 PH2` AC-2.  **Status: APPLIED for `32 §8`, `34 §11` and `50 PH2` AC-2; the render pipeline DEFERRED → PH0 (§5.1).**

---

### X10 — MLS scale is stated three incompatible ways

**Conflict.** `14 §4.1` says "sizes from 3 to 5,000"; `14 §4.4` and ADR-0010 bind "beyond ~1,000 leaves"; `50 PH2` limits E2EE to "≤ 200 members"; `10 §12` triggers its fallback at ">1,000 members". Leaves are devices (`14 §4.2`), so 5,000 members is ~12,500 leaves — an order of magnitude past the bind point, and the figure that triggers a pre-committed architectural fallback is stated in the wrong unit in three places.

**Ruling.** **Every MLS figure is stated in leaves**, with the devices-per-Citizen assumption published alongside it.

```
  Published assumption:  2.5 devices per Citizen   (60 §4, restated in 14 §4.2)

  Mechanism ceiling      1,000 leaves     KeyPackage churn and Commit size bind
  PH2 operating limit      500 leaves     ~200 Citizens  — preserves 50 PH2's intent
  PH4 onward             1,000 leaves     ~400 Citizens  — after the PH1 spike
  Above the ceiling      E2EE is REFUSED, never silently downgraded (14 §4.4)
```

`14 §4.1`'s "3 to 5,000" describes **Society** size, not group size, and is corrected to say so: a Society may hold 5,000 Citizens; any single E2EE Chamber within it may hold at most 1,000 leaves (≈400 Citizens). `10 §12`'s trigger and ADR-0010 §9's review trigger both become ">1,000 leaves".

The PH1 OpenMLS spike at 1,000 leaves on real hardware (`60` I16) is what turns the operating limit from an estimate into a measurement, and it is scheduled in PH1 precisely so the `10 §12` fallback decision is not on PH2's critical path.

**Rationale.** P8/N6: E2EE is non-negotiable, so the number at which it stops working must be stated in the unit the mechanism actually consumes. A member-denominated ceiling silently becomes wrong the moment device count changes, and device count is a product decision nobody would think to check against a crypto limit.

**Changes.** `14 §4.1`, `14 §4.2`, `14 §4.4`, `10 §12`, ADR-0010 §3 and §9, `50 PH2` risk row.  **Status: APPLIED for `10 §12` and `14 §4.4`; `14 §4.1`, ADR-0010 and `50` DEFERRED → PH1 (§5.5).**

---

### X11 — The Citizen Vault has no home under P1

**Conflict.** `01 §1` scopes Vault under Society; `01 §6`'s Global Registry is a closed nine-entry list that excludes it; `11 §7` invariant 1 requires every persisted row to have a `society_id` or appear on that list. Yet `18 §5.1` grants every Citizen a Vault at Level 0, `13 §10.4` names Citizen private Vault keys, and `21 §3.4` builds eight Profile Modules on it. `21 §17` proposes the fix and it never landed.

**Ruling.** **Land `21 §17`'s amendment, and fix the invariant it exposes.**

```rust
struct Vault {
    vault_id: VaultId,
    society:  Option<SocietyId>,   // None => the Citizen Vault
    owner:    Principal,           // Citizen | Society — always exactly one
    // …
}

struct Object {
    object_id:  ObjectId,
    vault:      VaultId,
    society_id: Option<SocietyId>, // mirrors its Vault; None => Citizen Vault
    // …
}
```

`11 §7` invariant 1 becomes a **three-clause** test:

> Every persisted row either (a) carries a `society_id`, or (b) carries a null `society_id` **and** a resolvable owning reference to a Global Registry entry, or (c) is itself a Global Registry entry. Anything else is a violation.

Clause (b) is what the Citizen Vault uses: `society = None`, `owner = Citizen(fnid)`, and `Citizen` is Global Registry entry 1. **No Global Registry entry is added**, and no new P1 escape hatch is opened — `01 §6`'s list stays at nine, with entry 1 annotated to name the Citizen Vault and the global Wallet as the objects that hang off it.

**Alternative considered and rejected.** A `VaultScope { Society(SocietyId), Citizen(Fnid) }` enum is strictly better typed — it makes the second case *named* rather than *absent*, so invariant 1 becomes structural rather than conventional. Rejected because `11 §2.6`'s `Wallet` already uses `Option<SocietyId>` for exactly this, and two spellings of one idea inside one domain model costs more in agent confusion than the type buys in rigour. If `Wallet` is ever revisited, both should move to `VaultScope`'s shape together.

**Rationale.** P1's falsification test is the invariant, and an invariant that a shipped feature makes false is worse than no invariant — it teaches the team that a red property test is normal. `60` I3 is right that this becomes a failing property test the moment the first Citizen Vault Object is written, which is PH2; the Canon amendment is PH1 so it is never true.

**Changes.** `11 §2.7`, `11 §7.1`, `01 §1` hierarchy diagram, `01 §6` entry 1, `21 §17` (marked landed).  **Status: APPLIED.**

---

### X12 — Offline Agent invocation is answered two ways

**Conflict.** `15 §6` rule 5 says an Agent on a disconnected replica "queues commands in the outbox and they are authorized on arrival". `34 §12.1` says "Invoke an Agent — **Refused** unless a local `ModelProvider` is configured", on every target.

**Ruling.** Both are true and they answer different questions; state both, in both chapters, in the same sentence shape.

1. **Inference** is refused offline unless a local `ModelProvider` is configured. Nothing queues a prompt.
2. **Action authorization** is never granted offline. An Agent action produced by a local model enters the outbox **unauthorized** and is evaluated at the PEP on arrival.
3. **New, and the part neither chapter said:** an outboxed Agent command whose Envelope has **expired or been revoked** between enqueue and arrival **fails at the PEP** and surfaces to the Operator as `AgentActionBlocked`. It is never authorized retroactively against the grant that existed at enqueue time. Anything else would make `12 §7.3`'s "revocation is immediate and retroactive to in-flight actions" false for exactly the population that is hardest to reach.

**Rationale.** P4 (an Agent may never widen its own envelope, and revocation must bite) over P2 (local-first). `00 §2` puts P4 fourth and P2 seventh.

**Changes.** `15 §6` rule 5, `34 §12.1`.  **Status: DEFERRED → PH3 (§5.6).**

---

### X13 — The Custodian bond is a flat 500 FRC and a per-byte rate

**Conflict.** `18 §5.5` gates Custodian eligibility at `Level ≥ 8 ∧ Trust ≥ 200 ∧ Stake(500 FRC)`. `13 §7.4` sets `bond = bond_rate × committed_bytes` at ~100 FRC/TiB. A 50 TiB Custodian bonds 5,000 FRC under `13` and 500 under `18`.

**Ruling.** `13` owns custody economics. `18 §5.5`'s Custodian row becomes:

```
  Custodian eligibility
      Trust >= 100
    ∧ proof-of-capacity at registration (13 §7.4)
    ∧ Stake( bond_rate x committed_bytes ), floor 500 FRC
    ∧ 30-day probation at reduced assignment
```

**The Level gate is deleted entirely**, not merely lowered. This goes further than X13 as `60` states it, and folds in `60 §3.14`: Level 8 requires 21 weeks on the Light path, and Trust ≥ 200 requires months of evidenced events whose highest-frequency admitted input — "Custodian Attestation streak honored" — is unavailable to someone who is not yet a Custodian. **Level 8 gates nothing that proof-of-capacity plus a byte-scaled bond does not gate better**, and it excludes precisely the population that would supply the first petabyte: datacentre operators, NAS enthusiasts, small hosting providers, none of whom will spend five months posting in Chambers to earn the right to sell disk.

Two supporting changes: a **pre-recruited first cohort** under an explicit published arrangement so `13 §11.4` step 4.3 does not depend on organic supply, and a **measured PH4 entry precondition** — committed capacity ≥ 3× current logical storage across ≥ 8 failure domains at the modelled price, before step 4.1.

**Rationale.** P12: a bond exists to make slashing costlier than misbehaviour, and a flat figure cannot do that across three orders of magnitude of committed capacity — at 50 TiB, 500 FRC is one day's earnings. `18 §5.5`'s own principle, "capacity is purchasable, precedence is earned", is satisfied: the bond scales with capacity and scheduling precedence remains a function of Trust and attestation history only.

**Changes.** `18 §5.5` Custodian row, `13 §7.4` (unchanged, now the sole source), `13 §11.4` step 4.1.  **Status: APPLIED for `18 §5.5`; the cohort and the entry precondition DEFERRED → PH4 (§5.7).**

---

### X14 — The simulation seed count at a phase gate is 500,000 in one place and 5,000,000 in another

**Conflict.** `50 PH1` AC-5 requires 500,000 simulated histories at the gate; `40 §7.5` and ADR-0014 specify 5,000,000 pre-phase-gate. `60 §4` row 12 shows the two published *rates* are themselves inconsistent by 6.25× (5.6 seeds/s per-PR versus 34.7/s nightly), and that 5,000,000 is 40–250 core-hours either way.

**Ruling.** Three tiers, each with a stated cost, and the gate figure is **500,000**.

| Tier | Seeds | Budget | Where |
|---|---|---|---|
| Per PR | 10,000 fresh + the full regression corpus | ≤ 6 min on 8 cores | Fast lane. ADR-0014 §7's `--seeds 10000` stands, and the per-PR rate discrepancy is explained: the 6-minute figure **includes the regression corpus**, which is stated rather than implied |
| Nightly | 500,000, histories to 100 k steps, wide fault bands | 4 h | Full lane, nightly |
| **Phase gate** | **500,000** against the phase's feature set | 4 h, one job | `50`'s figure wins. `40 §7.5`'s 5,000,000 is corrected |
| Annual soak | 5,000,000 | ~40 min on a 64-machine burst fleet, or 250 core-hours serial | Retained as an annual exercise, funded explicitly, and **not a gate** |

**Rationale.** A gate nobody can afford to run is not a gate — it is a step that gets waived, and a waived gate is worse than a smaller one because it teaches that gates are negotiable (`40 §8.3` has no bypass list for exactly this reason). 500,000 seeds at a 4-hour nightly budget is a real gate that a small team actually runs. `sim-mutation` (ADR-0014 §7) is the assurance that the 500,000 are meaningful, and it is unaffected by the count.

**Changes.** `40 §7.5`, ADR-0014 §7 and §9, `50 PH1` AC-5 (unchanged).  **Status: APPLIED for `40 §7.5`.**

---

### X15 — Four broken document references

**Conflict.** `16 §2.3`/`§7.1` cite `30-api-contract.md` (actual: `30-api-and-sdk.md`); `19 §5` and `20 §5` cite `19-marketplace-and-commerce.md` (actual: `19-marketplace.md`); `50 M1.8`/`50 PH1` cite a chapter `51` that `60` believed did not exist; `00 §1` P12 cites "`19` / `17-economy-fraction.md`" for the economy simulation harness, which lives only in `17 §12`.

**Ruling.** All four corrected. `51-phase-1-web-gui.md` **does** exist — `60` was written against an earlier tree — so that reference stands and `60 §5` X15's third item is withdrawn. A **link checker** joins the fast lane: every `NN-name.md`, `§n.n` and `adr/NNNN` reference in `docs/` must resolve to an existing file and an existing heading.

**Rationale.** `40 §5` already forbids documentation drift; this is drift with a mechanical fix, and it is the cheapest one in the corpus.

**Changes.** `16 §2.3`, `16 §7.1`, `19 §5`, `20 §5`, `00 §1` P12.  **Status: APPLIED.**

---

### X-TH — Theme names and token file layout disagree

**Conflict.** `32 §9` names three built-in themes `void` / `daylight` / `contrast` and `32 §2` places the token source at `tokens/`. `41 §10.2` names them `default` / `high-contrast` / `terminal-amber` and places the source at `packages/tokens/src/`.

**Ruling — split, because each chapter is right about its own subject.**

- **Names: `32` wins.** `void`, `daylight`, `contrast`. `32` owns the design system and its names are load-bearing in `18 §5.3`'s theme unlock table, `33`'s brand language and `51 §9`'s copy strings. `41 §10.2`'s `terminal-amber` is not a theme at all — it is a CLI palette variant, and it is renamed accordingly.
- **File layout: `41` wins.** `packages/tokens/src/theme/{void,daylight,contrast}.json`. `41` owns repository structure; `32 §2`'s bare `tokens/` is a sketch written before the monorepo layout existed, and `32 §5`'s "no literal hex outside `tokens/`" lint allowlist must name the real path or it allows the wrong directory.
- **Phasing:** PH1 ships `void` and `contrast`; `daylight` is PH2 (`51` Q5).

**Rationale.** N7 requires one source and five targets, and "one source" is a path, not a concept. P10/N8: `contrast` is the accessibility theme and its name appears in a user-facing setting, so it is a product decision (`32`/`33`) rather than a build-layout decision (`41`).

**Changes.** `41 §10.2` theme filenames, `32 §2` token path, `32 §5` lint allowlist path.  **Status: DEFERRED → PH0 (§5.8), because both edits are inside generated-pipeline descriptions that land with M0.5.**

---

### X-GW — Grace windows are stated three ways

**Conflict.** `01 §2` says a Handle is "immutable after a grace window" without stating it; `11 §2.1` says 14 days; `12 §2.3` says 14 days after first claim; `32 §5.2`'s field example says "changeable once, within 14 days"; `33 §7.3` says of a Society name only "you can change it once", with no window.

**Ruling.** **14 days, once, for both Handle and Society name.** One change, inside 14 days of first claim, after which both are immutable. The old value is reserved to the same principal for 12 months (matching `11 §2.1`'s `Departed` reservation) so the window cannot be used to squat-and-release. `51 §16` Q6 already proposed this and it is now binding.

**Rationale.** P9 and product honesty: a Handle is an identity surface other Citizens learn, and an unbounded rename window makes every mention stale. One change inside 14 days covers the real case (a typo, or second thoughts on day two) and nothing else.

**Changes.** `01 §2` Handle row, `33 §7.3`, `32 §5.2` (already correct), `11 §2.1` and `12 §2.3` (already correct).  **Status: APPLIED for `01 §2`.**

---

## 3. Contradictions Found During This Pass

### N1 — PH1 breaches the resource-family budget under `30 §4.2`'s own definition

**Conflict.** `30 §4.2` defines a resource family as a maximal set of resources sharing an owning `10 §3` boundary. Under that definition PH1 delivers Identity, Society, Discourse, Ledger, Progression and Signals — **six** families against `02 §5`'s cap of three. `50 PH1`'s budget line claims three by counting under a looser definition ("chambers and messages are sub-resources of societies"), which is true but does not make Discourse stop being its own boundary.

**Ruling.** `30 §4.2`'s definition governs — it is the one with an owner, a test and a stated exemption. Three moves bring PH1 to exactly three:

1. **Society is claimed in PH0** by the walking skeleton (`50 M0.7` ships `POST /v1/societies`). PH0 uses 1 of its 3.
2. **Progression is a sub-resource of `citizens`**, not a family. `30 §4.2` already paths it that way: `/v1/citizens/{id}/progression`. It shares no boundary-level versioning surface of its own.
3. **Signals is a protocol, not a family.** Its one REST resource, `/v1/subscriptions`, moves to **PH2** with the Relay's durable subscription store. The WebSocket surface is a transport contract governed by `30 §7`, and S14 has no domain crate for the same reason (`41 §5.3`).

PH1 then claims Identity, Discourse and Ledger — **exactly at budget**. `03 §4.3` records the arithmetic.

**And S15 Atlas costs PH1 nothing.** `60 §3.9`'s remedy proposed spending a PH1 family slot on Atlas and moving Discovery to Phase 4 to pay for it. That trade is unnecessary: Atlas's inbox is served under `/v1/citizens/me/inbox` (Identity family), its search under the Discourse family (`51` Q8), and its Shard refcount has no public API at all. Atlas is a **crate and a boundary, not a resource family and not a service**, and it does not become a service until PH5.

**Rationale.** `02 §5`'s rationale for the family cap is "each is a permanent versioning commitment", which attaches to a versioned public contract, not to a Rust boundary. Progression and Signals fail that test; Discourse passes it.

**Changes.** `30 §4.2` phase column and the Signals row, `50 PH1` and `50 PH2` budget lines, `03 §4`.  **Status: APPLIED in `03`; the chapter edits DEFERRED → PH0 with the generator (§5.1).**

---

### N2 — `02 §5`'s dependency budget is undefined for a polyglot monorepo

**Conflict.** `02 §5` allows "5 new third-party runtime dependencies per phase". `51 §2.3` claims all five PH1 slots for `apps/web` alone — React, TanStack Router, TanStack Query, Zustand, `idb-keyval` — and says "that is the whole list and it is the whole budget". PH1 must also ship passkeys (`webauthn-rs`), a WebSocket surface (`tokio-tungstenite`), content addressing (`blake3`), sortable IDs (`ulid`) and signatures (`ed25519-dalek`) in the Runtime, and a TUI in the CLI. The budget is breached by construction before any Work Unit opens.

**Ruling.** **Five new third-party runtime dependencies per phase *per deployable artifact*** — the Runtime binary, the web client, the CLI binary, the desktop shell, and each mobile shell counted separately.

**Rationale.** `02 §5` states the reason for the limit: "each is a supply-chain surface and a maintenance horizon". A React dependency is not in the Runtime's supply chain and cannot compromise the ledger; making it compete with `webauthn-rs` for the same slot measures nothing real. The per-artifact reading preserves the rule's purpose — no single shipped binary accretes more than five new external maintainers per phase — while making it satisfiable. `03 §4.2` records the per-artifact draw for every phase.

**Changes.** `02 §5` (dependency row), `03 §4.1`.  **Status: APPLIED for `03 §4.1`; `02 §5` DEFERRED → PH0 pending the ADR named in §6, because amending Canon requires one (`00 §7`).**

---

### N3 — `17 §3.2` activates Sources in phases where `50` budgets zero

**Conflict.** `17 §3.2` places S4 and S6 at "Phase 2" and S5 and S7 at "Phase 3". `50 PH2` and `50 PH3` both budget **0 economic Sources**. `21 §14` compounds it by scheduling `ContributionReceipt` accrual rendering "(XP and S4 only)" in Phase 2, justified by "`17` S4 is Phase 2".

**Ruling.** **No Source emits before PH4.** `50` is right and `17 §3.2`'s phase column is wrong, for a reason `17` itself supplies: a Source cannot ship before the emission ceiling machinery (`17 §5`), the settlement saga (`11 §5`), the public supply dashboard (`17 §11`) and the economic simulation gate — all of which are PH4 deliverables, and the last of which is PH4's *entry criterion*. Shipping S4 in PH2 would mean emitting Fraction against a model nobody has yet been required to pass.

The full ladder is in `03 §4.4`: PH4 S1+S2, PH5 S4+S6, PH6 S5+S9, PH7 S3+S8, PH8 S7+S10. `21 §14`'s `ContributionReceipt` row renders **XP only** in PH2 and gains its S4 leg at PH5.

**Rationale.** P12: emission must be bounded, measured and published, and none of those three verbs has a subject before PH4.

**Changes.** `17 §3.2` phase column, `21 §14`, `03 §2`.  **Status: APPLIED in `03`; chapter columns DEFERRED → PH0 with the generator (§5.1).**

---

### N4 — `16 §20` places the Facet standard in a phase with no asset milestone

**Conflict.** `16 §20` puts `FN-ASSET/1` core and the evolution engine at "Phase 3" and `30 §4.2` puts the Asset family at Phase 3. `50 PH3` contains no asset milestone — M3.1 to M3.9 are Envelopes, PEP, Agent runtime, ModelProvider, Workflows, Extension host, first-party Extensions, Governance v1 and Agent surfaces. Meanwhile `18 §5.4` says Insignia and Badges *are* Facets, and `50 M1.6` ships Achievements in PH1.

**Ruling.** **The Asset family and `FN-ASSET/1` core move to PH4** as a new milestone **M4.11**, using one of PH4's two free family slots. PH3's family budget is exactly consumed by Agent, Extension and Governance (N1), so Asset could not stay there in any case.

The Insignia question resolves cleanly using the Facet model's own property: **PH1–PH3 Achievements, Badges and Insignia are progression records** in `fractal-domain-progression`, with no Facet representation. At PH4 they are **minted retroactively as `Tiered` Facets carrying full tier provenance reconstructed from the progression log** — which is exactly what `18 §5.4` describes an Insignia doing ("the same `FacetId` it was at 1 GB-month, with a provenance chain showing every tier crossing"). Nothing is lost and nothing is re-created, which is the same guarantee Crystallization makes (`11 §3.1`).

**Rationale.** P6: the progression log is the source of truth, so a Facet minted from it later is not a reconstruction, it is a projection with an identity. `02 §5`'s family cap forces the move; the Facet evolution model makes it free.

**Changes.** `16 §20`, `30 §4.2`, `18 §5.4` (a note), `50 PH4` (new M4.11).  **Status: APPLIED in `03`; chapter edits DEFERRED → PH0 with the generator (§5.1).**

---

### N5 — `41 §5.4` places a PH0 crate on a PH2 dependency

**Conflict.** `41 §5.4` puts `fractal-app-kernel` at Phase 0 and lists `fractal-domain-agent` among its permitted dependencies, because the kernel owns "the Policy Enforcement Point invocation". `41 §5.3` puts `fractal-domain-agent` at Phase 2. A crate cannot depend on a crate that does not exist yet.

**Ruling.** `fractal-domain-agent` is created in **PH0**, holding only the `CapabilitySet` lattice — the type, the `meet` operation, the attenuation constructor and their property tests. Two callers need it before PH3: `fractal-app-kernel` at PH0, and the Charter's `capabilities: BTreeMap<RoleId, CapabilitySet>` at PH1 (`11 §2.3`). `Agent`, `Envelope`, `Policy` and `Workflow` types land in **PH3** with M3.1.

`41 §5.3`'s single "Phase" cell cannot express a crate that arrives in two pieces, and is replaced by two rows — one per stage — which is also true of `fractal-domain-governance` (Charter and roles at PH1; Proposal, Vote and moderation at PH3; quorum and delegation at PH5).

**Rationale.** P5's two-implementations-at-introduction rule and `02 §7`'s "no abstraction without two callers" are both satisfied at PH0: the lattice has the kernel and the testkit double. Building the lattice early is also what makes `50 PH3` AC-1 — no Envelope in 1,000,000 generated grant histories confers a capability its grantor lacked — testable from PH0 rather than from PH3.

**Changes.** `41 §5.3` (split rows for `-agent` and `-governance`), `41 §5.4`, `03 §2.1`.  **Status: APPLIED in `03`; `41` edits DEFERRED → PH0 with the generator (§5.1).**

---

### N6 — `18 §5.2` grants unmetered storage, which two other documents forbid

**Conflict.** `18 §5.2` grants Society Level 4 "unmetered Vault (cost-settled)". `50 PH2`'s mitigation is "no unmetered storage, ever". `02 §4` forbids infinite or discretionary emission, and an unmetered grant funded from emission is exactly that with an extra step.

**Ruling.** **Deleted.** The Society quota ladder becomes `SL0 5 GB · SL1 25 GB · SL2 100 GB · SL3 500 GB · SL4 2 TB · SL5 5 TB`, all metered, all purchasable beyond the grant with Fraction (`18 §5.5`: capacity is a commodity). The aggregate free-grant ceiling E14 (X6) applies on top.

**Rationale.** P12. Also `60 A11`: the platform must not subsidize storage in a unit it prints against a real dollar bill with no revenue.

**Changes.** `18 §5.2`.  **Status: APPLIED.**

---

### N7 — `11 §2.3` specifies member caps only to Society Level 2

**Conflict.** `11 §2.3` gives 25 members at SL0, 100 at SL1, 500 at SL2, and is silent from SL3 up — while `14 §4.1` casually describes Societies of 5,000 and `60 §4` row 3 shows the E2EE ceiling binds at ~400 Citizens.

**Ruling.** The ladder is completed: **SL3 2,000 members · SL4 10,000 · SL5 unbounded.** The E2EE ceiling is *orthogonal and stated separately*: any single E2EE Chamber holds at most 1,000 leaves (≈400 Citizens) regardless of Society size (X10), and a Society above that size simply cannot make one Chamber E2EE for everybody — which the client says at Chamber creation, never at message-send time.

**Rationale.** P1: the Society is the unit whose limits must be knowable, and a table that stops halfway invites each chapter to invent the rest. Separating the membership cap from the crypto ceiling is what stops a product decision (bigger Societies) from silently becoming a security decision.

**Changes.** `11 §2.3`, `18 §5.2`.  **Status: APPLIED.**

---

### N8 — `19 §17` ships listings and the review pipeline at "4–5"

**Conflict.** `19 §17` places the listing model, capability manifest, review pipeline R0–R2, recall, ratings, discovery and free third-party Extensions at "Phase 4–5". `50 PH6` owns the marketplace, and `02 §3` defers third-party paid Extensions to Phase 6 because "nothing executes untrusted third-party code until the Envelope system has been adversarially tested for two phases".

**Ruling.** **PH6.** All of it. `19 §17`'s "4–5" is corrected, and the argument that a *free* Extension can ship earlier is rejected: `19 §17`'s own closing sentence — "a free Extension holds an Envelope, and an Envelope is what actually needs governing" — is the reason it cannot. The external security audit is PH3's exit gate; two phases of adversarial exposure after it are PH4 and PH5; PH6 is the earliest phase in which `02 §3`'s condition is met.

A range is also not a phase (`03 §1`). "4–5" is an unmade decision wearing a number, and this is the decision.

**Rationale.** P8 over P7 — `10 §12` already states the standing commitment that P8 outranks P7 when third-party code execution is at stake, and `00 §2` puts P8 first and P7 thirteenth.

**Changes.** `19 §17`.  **Status: APPLIED in `03`; `19 §17` DEFERRED → PH0 with the generator (§5.1).**

---

### N9 — `20 §2` places `workflow` and `automation-pack` a phase late

**Conflict.** `20 §2` puts extension kinds `workflow` and `automation-pack` at Phase 4. `50 M3.5` ships the Workflow graph, triggers, steps, conditions, compensation and versioning plus three first-party Workflows in PH3, and `50 M3.7` ships ten first-party Extensions in PH3.

**Ruling — split, because the two kinds are not the same risk.**

- **`workflow` is PH3.** M3.5 ships the graph and M3.7 ships Extensions built on it; a `workflow` Extension is that graph in a distributable wrapper and adds no new authority surface.
- **`automation-pack` is PH4.** It bundles multiple Workflows *and Policy proposals*, and a Policy proposal is a consent surface a human signs. That needs Governance v1 (M3.8) to have been in production for a phase before it becomes a distributable artifact, because a pack that proposes Policy against a governance model still settling is a consent screen nobody can evaluate.

**Rationale.** P4: only Citizens author Policy, and a pack that ships Policy drafts is a mechanism for putting drafts in front of humans at scale. That mechanism should not launch in the same phase as the governance model it drafts against.

**Changes.** `20 §2` kind table.  **Status: APPLIED in `03`; `20 §2` DEFERRED → PH0 with the generator (§5.1).**

---

### N10 — `30 §4.2` places the Agent and Vault families a phase or two early

**Conflict.** `30 §4.2` puts the Agent family (`agents`, `envelopes`, `policies`, `workflows`, `runs`) at Phase 1 and the Vault family at Phase 1; `50` ships Agents in PH3 and the Vault in PH2. It also marks Identity, Society, Discourse and Signals as Phase 0, which is four families against `50 PH0`'s stated one.

**Ruling.** Agent → **PH3**, Vault → **PH2**, Discovery → **PH5** (`51` Q8), Asset → **PH4** (N4), Economy → **PH4**, Progression → not a family (N1), Signals → protocol, `/v1/subscriptions` at PH2 (N1). PH0 declares **only** `societies`. `51 §16` Q11 already ruled that `50` governs sequencing and that `30 §4.2`'s column must be restated in `PH<n>` notation; this completes it with the actual values.

**Rationale.** P3 is not weakened by a later family: API-first means no capability exists until it exists as an API, not that every family is declared early. A declared-but-empty family is a permanent versioning commitment bought with nothing (`02 §5`).

**Changes.** `30 §4.2`.  **Status: APPLIED in `03`; `30 §4.2` DEFERRED → PH0 with the generator (§5.1).**

---

### N11 — `13 §14` V5 states σ as a constant the corrected rates do not equal

**Conflict.** V5 requires the Sink rate per byte-hour to exceed the Source rate by "a platform constant σ > 1 (initially **1.25**)". At the reconciled rates (X6) σ is 2.23.

**Ruling.** V5 becomes **σ ≥ 1.25**, a floor asserted by the settlement code and by `econ-sim`, not a target the rates are expected to hit. The observed σ is published in the supply dashboard as a health figure.

**Rationale.** P12: the invariant's purpose is that the platform never pays out more per byte-hour than it takes in. A floor states that; an equality states a pricing policy that no chapter owns.

**Changes.** `13 §14` V5, `13 §8.3`.  **Status: APPLIED.**

---

### N12 — Three chapters each claim to be the single source of performance budgets

**Conflict.** Folded into **X9**. Recorded separately because the *ownership* claim and the *number* conflict are different defects: even with identical numbers, three chapters claiming single-source ownership guarantees the next divergence.

**Ruling.** See X9. `32 §8` is the source; `perf/budgets.json` is generated from it; `34 §11`, `31 §10` and `40 §13.1` own disjoint extensions and may not restate a `32 §8` row.

**Status: APPLIED (see X9).**

---

### N13 — `16 §4.4`'s single `EmissionAccount` row is contradicted by its own concurrency argument

**Conflict.** `16 §4.4` identifies the `EmissionAccount` as "the one global hot row" and mitigates by batching per settlement window **per Society** — which still funnels every Society into one row. `13 §8.2` then fixes the window at 24 hours closing 00:00 UTC, converting distributed load into a synchronized spike: 100,000 Societies × 5 ms serialized is 8.3 minutes of held lock, daily, during which `40 §9.4`'s ledger-settle p99 < 1 s SLO is violated and `16 §4.5`'s reconciler cannot get a consistent read.

**Ruling.** Two changes, neither of which alters a single economic quantity.

1. **Shard `EmissionAccount` into K = 64 sub-accounts.** `EmissionAccount[i]` where `i = BLAKE3(society_id) mod 64`, each pre-allocated a deterministic `1/64` slice of the epoch budget. Total supply becomes `-Σ EmissionAccount[i].balance` — still exact, still directly queryable, no longer one row. `16 §19` invariant 2 becomes "exactly K accounts may be negative, and they are the `EmissionAccount` shards"; `11 §7.4` becomes `total supply == -Σ EmissionAccount[i].balance`.
2. **Offset the settlement close** by `BLAKE3(society_id) mod 86400` seconds. The cap is per-epoch, not per-instant, so nothing changes economically. `13 §7.5` already applies exactly this technique to attestation cadence and did not apply it here.

K = 64 is chosen to match the `ShardRouter`'s hash function and modulus (W4), so a Society's settlement Posting is **always on the same primary as the Society**. That is the detail that makes the future shard cheap: without it, every settlement becomes a cross-shard write the moment step 3 of the ladder executes.

**Rationale.** P12 (total supply stays exactly queryable) and P6 (the fold stays deterministic — K is a constant, the slice is deterministic, and replay reproduces it). The architecture's central claim is that it has no global write contention; this was the one place it did.

**Changes.** `11 §7.4`, `16 §4.2`, `16 §4.4`, `16 §19.2`, `13 §8.2`.  **Status: APPLIED for `11 §7.4` and `16 §19.2`; the rest DEFERRED → PH1 (§5.9).**

---

### N14 — `30 §4.3` #6 encodes the Society-creation gate that X7 overturns

**Conflict.** `30 §4.3` endpoint 6 encodes `society.create (Level ≥ 3)`, which X7 replaces.

**Ruling.** The capability string is `society.create`, unconditional for a Citizen who has not consumed their first-hearth allowance, and predicated on `Level ≥ 3` thereafter. The generated capability registry (`30 §12.2`) carries the predicate; the endpoint table states both cases.

**Status: DEFERRED → PH1 (§5.4).**

---

### N15 — Adding S15 leaves `10 §3` and `41 §5.3` structurally out of step

**Conflict.** `10 §3` has fourteen boundaries and `41 §5.3` thirteen domain crates, with S14 Relay explicitly having no domain crate because it is transport. Adding S15 Atlas creates the same question again.

**Ruling.** **Atlas has no domain crate.** It owns no invariants and no decisions — only projections — so a domain crate would invite exactly what `41 §5.3` warns about for Relay: persistence and read-path logic drifting into a place the purity rules protect. Atlas is `fractal-app-atlas` in the application layer, hosted by `fractal-app-projection`'s runner until it extracts as a service at PH5. `10 §3` gains S15; `41 §5.3` gains nothing; `41 §5.4` gains one row.

**Rationale.** P6: a projection is disposable by definition, and a crate whose contents are all disposable does not belong in the layer whose contents are all invariants.

**Changes.** `10 §3`, `41 §5.4`.  **Status: APPLIED.**

---

## 4. Structural Weaknesses

`60 §3` ranks sixteen weaknesses by probability × cost × cost-of-fixing-late. Each gets a ruling. Where a weakness is fully discharged by a contradiction ruling above, the entry says so and does not restate it — a ruling stated twice is a ruling that can drift.

### W1 — There is no authoritative phase table, and eight chapters disagree

**Ruling.** Discharged by **X8**: `03-phase-authority.md` is created as the fourth Canon file, `docs/phases.toml` is its machine-readable companion, and `cargo xtask phase-check` / `lint-phases` land in M0.2. `60`'s remedy proposed `docs/phases.toml` alone; a TOML file with no prose is not something an agent loads before an implementation task, which is why the authority is a Canon *document* with the TOML generated beside it.
**Rationale.** `02 §6` question 2. **Status: APPLIED.**

### W2 — Storage economics do not close, and the two owning chapters differ by 6× to 18×

**Ruling.** Discharged by **X6**, with corrected arithmetic, a re-scaled quota ladder, σ as a floor, the deletion of SL4's unmetered grant, and the new binding constraint **E14** (aggregate free-grant custody cost ≤ 60% of the S1 share) that makes "storage emission converges to zero" a mechanism rather than a claim. The corrected model also **reduces** the FRC utility value needed to recruit honest Custodian capacity from $0.087 to **$0.0142**, which materially weakens W14 and `60 A1`.
**Rationale.** P12. **Status: partially APPLIED (`18 §5.2`); rest DEFERRED → PH1 (§5.2).**

### W3 — The emission schedule is denominated in calendar years; the roadmap is not

**Conflict.** `17 §5.1` R2 fixes `B(n) = 200,000,000 × 0.80^(n-1)` with *n* a calendar year and R3 forfeits unclaimed budget permanently. No Source emits before PH4 (N3), which at `60 §3.11`'s realistic multiple completes near month 35. Y1–Y3 then elapse with 488 M FRC of schedule — 48.8% of the lifetime cap — forfeited against almost no claim, invalidating `17 §5.2`'s ten-year model, `17 §5.3`'s π collapse and the M1–M5 target bands in `17 §1.3` that define "the economy is working".

**Ruling.** **Redefine *n* as epochs since the first Source was enabled in production**, where "enabled" is the `SourceActivated` domain event for **S1** specifically, emitted once, at PH4. Concretely:

```
  emission_epoch_origin : Timestamp   set exactly once, by the S1 SourceActivated event
  n(t) = floor( (t - emission_epoch_origin) / 365 days ) + 1
  B(n) = 200,000,000 x 0.80^(n-1)     unchanged
  R1, R3                              unchanged
```

The decay shape, the forfeiture rule and the lifetime cap are all preserved; only the clock's zero moves. `17 §13`'s parameter tiers record **the shape as Tier 0** (immutable) and **the origin as Tier 1a** (set once, by event, never by a human). `GenesisAllocation` (X-GA) draws against `B(1)` and is therefore issued *before* the origin is set — which is correct, because it is not emission from a Source and its cap is absolute rather than scheduled.

**Rationale.** P12 requires the emission model to be measurable and published; a model whose validity depends on the roadmap not slipping is neither, because `50 §1` explicitly says the durations are "estimates for sequencing, not commitments". `00 §3`'s standing bias — correctness over velocity — applies to models as much as to code.

**Changes.** `17 §5.1` R2, `17 §5.2` (the ten-year table re-based), `17 §13` (tier assignment), `50 §5` (a new cancel/reorder row: "PH4 slips past month 30 → re-run `17 §5.2` against the actual origin before the PH4 gate").
**Status: DEFERRED → PH1 (§5.10). The ruling is binding now.**

### W4 — Postgres is the event store, and the stated scaling remedy adds no write capacity

**Conflict.** `10 §12` prescribes "partition by `society_id` first" and ADR-0006 §9 triggers on "p99 write latency … **after partitioning by `society_id`**", as the remedy for a write bottleneck. Declarative partitioning within one instance adds **no write throughput** — one instance, one WAL, one writer. ADR-0006 §3 states the truth two sections earlier and §9 contradicts it.

**Ruling — the real ladder, with measurable triggers.**

```
  1  PARTITION           declarative hash partitioning of the event and projection
                         tables by society_id, 64 partitions
     BUYS                index depth, vacuum cost, locality, per-Society DROP
     DOES NOT BUY        write throughput. One instance, one WAL, one writer
     TRIGGER             the event table exceeds the primary's buffer-cache working
                         set, OR autovacuum duty cycle on the event table > 25%

  2  OFFLOAD READS       projections to physical replicas and logical-decoding CDC
     BUYS                read capacity
     DOES NOT BUY        write throughput
     TRIGGER             replica share of total reads < 60%

  3  SHARD ACROSS PRIMARIES
                         N independent Postgres primaries; Societies mapped by
                         rendezvous hash on society_id; routing in the composition
                         root (fractal-node) — NOT in the domain, NOT in the app layer
     BUYS                write throughput, linearly in N
     TRIGGER (measured)  sustained posting-path write transactions/second exceeds
                         60% of the MEASURED single-primary ceiling for 7 consecutive
                         days, OR p99 posting latency > 50 ms at that load
                         (the ceiling is measured in PH1, not assumed from 60 §4)

  4  REPLACE THE ENGINE  EventStore -> FoundationDB or per-Society segment files
     TRIGGER             only after step 3 is executed and measured
```

**Consequences for the single-transaction property, stated rather than discovered.**

- **The property survives, because it was always per-Society.** ADR-0006 exists to guarantee that the event append and the projection update commit together; a Society lives wholly inside one shard, so that transaction never spans primaries.
- **No transaction may span shards.** Exactly one command in the corpus touches two Societies: **Fracture**. It is already specified as "single transaction **per child**, idempotent, resumable" with the parent's log sealed before any child write (`11 §3.2`), so it is shard-safe by construction. Every other cross-Society operation is a saga with compensation (`11 §5`).
- **The Global Registry moves to its own primary** at step 3, hash-partitioned by FNID prefix. It has no ordering requirement, so unlike the log it is genuinely partitionable (`60 §4` row 1). This is scheduled at PH5 in `03 §2.6`.
- **Cross-shard reads go through S15 Atlas**, never through a join. That is the second reason Atlas exists.

**What must be true in the code beforehand, so the shard is cheap.** All six are PH1 or PH2 obligations:

1. No SQL outside `fractal-adapter-postgres` — already lint-enforced (ADR-0006 §7.1).
2. **Every `EventStore` and `Ledger` port method carries a routable key** — a `society_id`, or a `WalletId` that resolves to one — in its signature. No method may imply a global scan. **Audit this at PH1**, because a single keyless method is what forces a scatter-gather later.
3. **No query joins across `society_id`.** Enforced by the same lint that enforces dependency direction.
4. **`EmissionAccount` shards use the same hash and modulus as the router** (N13, K = 64), so a settlement Posting is always same-primary as its Society. Without this, every settlement becomes a cross-shard write on the day step 3 runs.
5. **A `ShardRouter` exists in the composition root from PH1 with one shard configured**, so the seam is exercised continuously — the discipline `13 §11.4` already applies to `BlobStore`.
6. **PH2 runs the full suite against a two-primary composition root** (`60` I14).

**Rationale.** P5: the port boundary is what makes this a swap rather than a rewrite, and the port is only a swap if every method is routable. The failure this prevents is specific and expensive — the team executes the partitioning project, measures no throughput improvement, and discovers mid-incident that the real fix is a topology change nobody designed.

**Changes.** `10 §12` bullet 2, ADR-0006 §3 (unchanged, now consistent), ADR-0006 §9 trigger (a), `40 §11` backup story, `03 §2.2`/`§2.3`.  **Status: APPLIED for `10 §12` and ADR-0006 §9; the PH2 two-primary proof scheduled in `03 §2.3`.**

### W5 — Projection rebuild time has no bound and no SLO

**Ruling.** Three parts, all binding.
1. **Generalize `16 §4.3`'s checkpoints to every projection**, every 4,096 events per projection per Society, so rebuild is snapshot-load plus tail-replay rather than replay-from-zero. Scheduled **PH1**, because retrofitting checkpoints to eight projections later costs more than building one checkpoint mechanism once.
2. **Publish a rebuild SLO in `40 §9.4`: any single projection for any Society rebuilds in ≤ 15 minutes at p99**, as a **PH2 phase-gate criterion**, measured against a large-Society fixture in `41 §15` (100 M events minimum — `fractal-sim`'s 100,000-step histories are four orders of magnitude below the case that matters).
3. **Amend `11 §7` invariant 10** to: *"Every projection is reproducible by replaying the log from the most recent verified checkpoint, and from zero within the published rebuild SLO."* Full replay from zero is retained as a **periodic audit**, not as the recovery path.

**Rationale.** P6's falsification test is "delete every projection and rebuild from the event log" — a test the corpus treats as a guarantee and never as a cost. `40 §9.6` requires every alert to have a runbook, and "drop and rebuild" as a runbook step with a 5-to-37-hour execution time is not a runbook, it is an outage with paperwork.

**Changes.** `11 §7.10`, `16 §4.3`, `40 §9.4`, `41 §15`, `50 PH2` acceptance criteria, `03 §2.2`/`§2.3`.  **Status: APPLIED for `11 §7.10`; the rest scheduled in `03` and DEFERRED → PH1/PH2 (§5.11).**

### W6 — The single `EmissionAccount` row is a global serialization point

**Ruling.** Discharged by **N13**: K = 64 shards keyed by the same hash as the `ShardRouter`, plus a per-Society settlement-close offset of `BLAKE3(society_id) mod 86400`.
**Rationale.** P12 and P6. **Status: invariants APPLIED; implementation DEFERRED → PH1 (§5.9).**

### W7 — The history archive key silently becomes the weakest link in E2EE

**Conflict.** `14 §4.5` defaults the Vault-backed DM archive **on**, encrypted under a `HistoryKey` obtainable "device-to-device **or via social recovery**". `12 §6.3` lists Vault objects whose content keys are wrapped to the recovery key as recoverable, and E2EE history as not. The DM archive is simultaneously both. If `HistoryKey` is recoverable, `t` guardians can read the Citizen's entire DM history after 72 hours, and `12 §6.2`'s protections all depend on an active device that this population by definition lacks. If it is not recoverable, `14 §4.5`'s "or via social recovery" is false.

**Ruling.** **`HistoryKey` is wrapped separately from the identity recovery key, behind its own separately-signed opt-in, off by default.**

- Default recovery restores identity, wallet, memberships, Level, Trust and Standing — **and not DM history**.
- The disclosure string becomes: *"Your guardians can restore your account. They cannot read your saved conversations unless you turn this on."* `14 §4.5`'s current sentence — "if that key is stolen, saved history can be read" — is true and does not say the thing that matters.
- Enabling escrow requires a separate signature over a `HistoryEscrowEnabled` grant naming the guardian set, and is revocable, which re-wraps forward.
- **New invariant I-12.13:** no recovery flow reconstructs a `HistoryKey` absent a live `HistoryEscrowEnabled` grant.
- **Companion rule in `34 §12.2`:** a Citizen whose only enrolled device is on evictable storage (PWA-on-iOS) is prompted for a second factor before holding Fraction or a Membership above Level 1. This closes the compounding case — a Citizen whose sole device key is evicted by the OS, with no second device, whose only path is a recovery that either exposes their history or does not restore it.

**Rationale.** P8 and P9 over P2. `00 §2` puts P8 first and P9 second; convenience of restore is P2 at position seven. The composition of three locally-correct chapters produced a guarantee nobody wrote and nobody owns, and the user experience *is* the composition.

**Changes.** `12 §6.2`, `12 §6.3`, `12 §13` (new I-12.13), `14 §4.5`, `34 §12.2`.  **Status: DEFERRED → PH2 (§5.12).**

### W8 — A confused-deputy path from Extension to Agent that the threat models name and do not close

**Conflict.** `20 §6` hook 31 lets an Extension return `list<tool-def>` to an Agent. `15 §13.1` wraps untrusted *content* in a structurally distinct region — but a tool's name, description and parameter schema are **structure, not content**: they enter the model as part of the action space. An Extension offering `approve_transfer_safe`, described as "use this whenever the user mentions a payment", supplies an instruction channel the data/instruction boundary does not cover. `20 §13` T9 names the residual and does not mitigate it. A second path: hook 18 permits fee annotation "clamped by the Charter's `economy` parameters", with no per-Extension revenue cap, and `ExtensionQuoteClamped` fires only when a quote *exceeds* the cap, never when it equals it.

**Ruling.** Four changes, all PH3.

1. **Extension-supplied tool definitions are untrusted content.** Host-namespaced as `ext.<install_id>.<name>`; the name, description and parameter documentation are rendered **inside** the `UNTRUSTED` region; the system region states explicitly which tools are host-native and that everything in the untrusted region is a claim made by a named Install.
2. **New assertion A11:** no Extension-supplied tool may be invoked for a `confirm_class` action unless the confirmation dialog names the supplying Install.
3. **Hook 18 fee disclosure:** any non-zero quoted fee is rendered in the **host-drawn** `TransferSheet` with the Install named. A fee the host does not draw is not charged.
4. **Per-Install fee-revenue ceiling** in `20 §11`, and `ExtensionQuoteClamped` fires at the cap as well as above it — an Extension installed across 4,000 Societies quoting the Charter maximum on every transfer extracts a continuous unsurfaced rent, and the event that would surface it is the one that never fires.

**Rationale.** P4: an Agent may execute within a granted envelope, but the value of the envelope depends on the Agent's action selection not being steerable by a third party at scale. P8's "deny by default" covers authority; this covers influence, which the threat tables identify and leave open.

**Changes.** `15 §13.1`, `15 §14.1` (new A11), `20 §6` hooks 18/31, `20 §11`, `20 §13` T9, `50 PH3` AC-3 injection suite.  **Status: DEFERRED → PH3 (§5.13).**

### W9 — The cross-Society read path is named as a cost three times and designed zero times

**Ruling — boundary S15, Atlas.** Full specification:

| Property | Ruling |
|---|---|
| **Name** | **S15 — Atlas.** Joins `10 §3`'s boundary table as the fifteenth |
| **Owns** | The Citizen unified inbox; cross-Society search; marketplace statistics; and the **Shard reference count** that `13 §10.2` says destroys data if under-counted |
| **Crate** | `fractal-app-atlas` in the application layer. **No domain crate** (N15) — it owns projections, not invariants, exactly as S14 Relay owns transport and has none. Hosted by `fractal-app-projection`'s runner until extraction |
| **Consumes** | From every Society log over `society.*.>`: `SocietyCreated`, `SocietyFractured`, `MemberJoined`, `MemberDeparted`, `ChamberMessagePosted`, `ThreadResolved`, `ObjectStored`, `ObjectTombstoned`, `VersionCommitted`, `ShardUnassigned`, `ListingPublished`, `PurchaseCompleted` |
| **Consistency** | **Eventually consistent, monotonic per reader.** A reader never observes Atlas at a lower frontier than one they have already observed, enforced by an `atlas_frontier` vector carried on every Atlas request and response. Staleness bound: **≤ 5 s at p99, ≤ 30 s at p99.9**, published as an SLO beside `40 §9.4`'s 2 s projection lag. Every response carries `as_of` (the per-Society frontier vector for the Societies in scope) and `stale: bool` |
| **Authority** | **READ-ONLY and never authoritative.** Atlas emits no domain event, holds no Wallet, takes no lock, and is never read by a command handler — `16 §5`'s "no reading a projection to make a decision" applies to it absolutely. If Atlas and a Society's log disagree, the log wins and Atlas is rebuilt |
| **Refcount posture** | **Monotone-safe.** Over-count permitted; under-count **forbidden**. GC requires **positive confirmation from every Society that has ever referenced the hash**, at that Society's current `seq`, that the hash is unreferenced. The *absence* of a reference is never sufficient. A Society whose log Atlas cannot read at its current `seq` **blocks the sweep** for every hash it has ever referenced |
| **New invariant V13** (`13 §14`) | No `ShardUnassigned` for garbage collection is emitted unless every Society in the hash's referencing set has positively confirmed non-reference at a frontier ≥ the tombstone's frontier |
| **Rebuild** | From the union of the referencing Societies' logs, with its own checkpoints (W5) |
| **Phasing** | PH1 inbox + in-Society search feed + refcount skeleton · PH2 refcount live with the Vault · PH5 extraction as a service and cross-Society search · PH6 marketplace statistics |
| **Budget** | **No new resource family** — the inbox is `/v1/citizens/me/inbox` (Identity family) and search is Discourse-scoped in PH1 (`51` Q8). **No new service until PH5.** This is a strictly better trade than `60 §3.9` proposed, which spent a PH1 family slot and moved Discovery to Phase 4 to pay for it |

**Rationale.** P1 pays for itself with per-Society partitioning and charges for it at exactly one place: reads that span partitions. Naming that cost three times in three chapters and designing it zero times is how the most-used surface in the product gets invented under deadline. The refcount is the urgent half — `13 §10.2` states plainly that an under-counting refcount destroys data, and after a Fracture `13 V6` guarantees two Societies reference the same Shards, so the projection whose under-count is data loss is also the one that is cross-partition.

**Changes.** `10 §3` (S15), `10 §4` (the cost paragraph now names its owner), `13 §10.2`, `13 §14` (V13), `14 §2` (`Scope::Self_` names Atlas), `30 §4.3` #41, `41 §5.4`, `03 §2.2`.  **Status: APPLIED for `10 §3`, `10 §4` and `41 §5.4`; `13`, `14` and `30` DEFERRED → PH1 (§5.14).**

### W10 — Fracture divides ownership but not cryptographic reach

**Conflict.** After an acrimonious split, `11 §3.2` executes with **no key-rotation step**. `13 §10.1` makes revocation forward-effective — removing READ "rotates the content key and re-wraps *future* Versions" — but nothing in the Fracture procedure removes READ, so every member of child B who held a wrapped `content_key` for a path assigned to child A still holds it, and the Shards stay live because A still references them (`13 V6`). A member of B with a local replica can decrypt A's finance archive **indefinitely, including future Versions**. The five Fracture invariants preserve Fraction, Facets, members, history and resumability. Confidentiality is not among them.

**Ruling.** A **mandatory execution step**, inserted after Vault re-referencing and before any child transitions to `Active`:

```
  For every Vault path assigned to exactly ONE child:
      rotate the content key
      re-wrap it to that child's key holders ONLY
      emit VaultKeyRotatedOnFracture { path, child, key_epoch }

  Paths with disposition `shared-custody` keep their key and are listed in the
  dry-run diff as DELIBERATELY SHARED, named path by path, so a human sees them.

  Existing Shards are NOT re-encrypted. That is impossible for content-addressed
  data and is already stated in 13 §10.1's residual register.
```

**New Fracture invariant — `11 §7` invariant 16:**

> After a Fracture, no principal assigned solely to child B holds a live wrapped key for any Vault path assigned solely to child A. Rotation is forward-effective: Versions written before the fracture point remain decryptable by anyone who held the key at that time, and this residual is disclosed in the dry-run diff.

**Property test**, added to `50 PH5` AC-1's 100,000 generated splits: for every generated split, for every path assigned solely to one child, assert that the post-fracture key-holder set for that path is a subset of that child's members.

**The cost, honestly.** Re-wrapping is proportional to **key holders, not bytes** — an 800-member Society with 2.5 devices each is 2,000 re-wraps per rotated path, milliseconds of work. `13 V6`'s "fracturing a 10 TB Society moves zero bytes" is fully preserved. What is **not** fixed, and is stated rather than papered over: a member of B who already downloaded and decrypted A's historical Versions keeps that plaintext forever. Content addressing makes that unavoidable (`13 §10.3`), and the rotation stops the *future*, which is the half that can be stopped.

**Rationale.** P8 over P1: the Fracture invariants were written from the ledger's perspective, where invariants are conservation properties, and conservation says nothing about who can read. `14 §9` already handles the Chamber equivalent honestly; the Vault case differs because the *future* is also unprotected without rotation. `50 PH5` AC-1 tests preservation over 100,000 splits and confidentiality over none.

**Changes.** `11 §3.2` (execution step), `11 §7` (invariant 16), `13 §10.1` (residual register), `50 PH5` AC-1.  **Status: APPLIED for `11 §3.2` and `11 §7.16`; `13 §10.1` and `50 PH5` DEFERRED → PH5 (§5.15).**

### W11 — The scope is not buildable in the stated timeframes; the realistic multiple is 2.0–2.6×

**Ruling.** The **sequencing is not changed** — `60` is explicit that the order is right. Three changes, one of which is a staffing decision and is therefore **open decision D1**:
1. `50 §1`'s durations are restated as **ranges with the realistic multiple stated**, and `03 §1` already carries both columns so that no reader sees only the optimistic figure.
2. A **cut candidate is designated per phase**, as `50 PH5` already does for Seasons. Recorded in `phases.toml` as `cut_candidate` on every `[[phase]]`: PH1 density modes and half the component inventory · PH2 responsive ultrawide · PH3 Governance v1 · PH4 video and the Stage · PH5 Seasons · PH6 Society storefronts · PH7 the third Experience.
3. A **"this phase exceeded 1.5× its estimate" row is added to `50 §5`** for every phase, not only PH2.
**Rationale.** `00 §5`: seven of eight is not done. A plan that cannot report is a plan that fails silently.
**Changes.** `50 §1`, `50 §5`, `phases.toml`.  **Status: `phases.toml` APPLIED; `50` DEFERRED → PH0 (§5.16); staffing OPEN → D1.**

### W12 — The hard line on Extension UI will not hold commercially, and the escape hatch is years away

**Ruling.** `20 §7`'s Surface Descriptor line is **correct engineering** and is not overturned here. But the decision about the escape hatch is made **before PH6**, not at its gate, because hard lines break under commercial pressure at gates. The recommended answer is `60`'s Option A — a **host-owned, host-chromed, input-mediated canvas** with no ambient I/O, no persistence import, a fixed frame budget and no ability to draw outside its own rectangle, which is a strict subset of `20 §12.1`'s Experience client and therefore costs sandbox work but not governance, tick-loop, session or economy work. **This is open decision D5**, because it costs a PH6 budget slot and an ADR, and because Option B (rewrite `19 §1`/`19 §14` for a marketplace of automations and templates, and re-justify the 12% fee against it) is a legitimate different product rather than a failure.
**Status: OPEN → D5.**

### W13 — Recovery is the real churn source, and the guardian model has a cold-start failure

**Ruling.** Three changes, all PH1, all binding.
1. **A platform-operated guardian of last resort** holding exactly **one** sub-threshold Shamir share, with a hard rule that `t` strictly exceeds the number of platform-held shares. This is **not custody under `02 §4`**: `02 §4` forbids custody of user private keys "without an explicit, revocable, user-initiated delegation", and this is a single sub-threshold share under exactly such a delegation, which cannot reconstruct anything alone by construction.
2. **The second device becomes the primary recovery story**, asked for before guardians. A second device is a mechanism people understand; a guardian set is one they will not complete. At launch there is literally nobody to nominate.
3. **Recovery-set formation rate becomes a `50 PH1` acceptance criterion** with a stated floor, alongside `12 §14`'s existing lagging triggers.
**Rationale.** P8 (the recovery path is a security surface and must actually work) and P12 (`60 §3.13`'s second-order effect: every recovery failure manufactures exactly the Level-0, no-history, new-Citizen profile the Sybil defences are tuned to suppress — and `12 §9` accepts "a patient adversary wins at small scale" on the assumption that honest Citizens are not repeatedly re-registering).
**Changes.** `12 §6.1`, `12 §3.3`, `50 PH1` acceptance criteria and risk table, `03 §2.2`.  **Status: scheduled in `03 §2.2`; chapter edits DEFERRED → PH1 (§5.17).**

### W14 — Custodian supply is gated behind Level 8 and paid in a unit with no external value

**Ruling.** The eligibility half is discharged by **X13** (Level gate deleted; bond scaled by committed bytes; 30-day probation). The economics half is materially improved by **X6**: the corrected replication factor and rate reconciliation drop the FRC utility value needed to recruit honest capacity from **$0.087 to $0.0142**, a 6.1× reduction, because `13 §8.2`'s figure was computed against a cap it had assumed rather than derived. Two additions stand: **pre-recruit a first cohort** under an explicit published arrangement so `13 §11.4` step 4.3 does not depend on organic supply, and add a **measured PH4 entry precondition** — committed capacity ≥ 3× current logical storage across ≥ 8 failure domains at the modelled price — before step 4.1.
**Rationale.** P12 and `13 §15`'s own stated fallback. The mesh must recruit or fail **before** the `BlobStore` migration, not during it.
**Status: X13 APPLIED; cohort and precondition DEFERRED → PH4 (§5.7).**

### W15 — Moderation under E2EE: the limits are honest, the exposure is not costed

**Ruling.** The technical position is not overturned — refusing client-side scanning is correct, and `E2`'s crate-graph lint is the right enforcement. Two things are added, and one is open.
1. **A safety-operations model joins `50 §3`'s continuous tracks** alongside Security, Accessibility, Performance and Documentation: report-volume projection, review staffing, latency and appeal SLOs, reviewer welfare, and an escalation ladder mirroring `19 §5.5`. It is a **PH1 exit criterion**, because PH1 is when external Citizens arrive. At 100,000 active Citizens and a conservative 0.5% monthly report rate that is 500 reports/month requiring a human to view disclosed plaintext; `19 §10.3` sizes the *marketplace* review pipeline and the *safety* pipeline has no equivalent. **Scheduled in `03 §2.2`.**
2. **A jurisdictional posture section is added to `14 §6`**: which regimes the platform intends to operate under, which are compatible with N6 as written, and the pre-committed response to an incompatible order — geoblock, exit, or overturn N6 by ADR. `12 §14` handles the analogous identity case in exactly the right shape ("if that is unacceptable to the regulator, the Rail does not ship there") and it is not applied to E2EE. **This requires counsel and is open decision D2.**
**Rationale.** P8/N6 is a non-negotiable, which means the *business* response to an order that conflicts with it must be pre-decided rather than improvised under legal pressure.
**Status: track APPLIED to `03`; `14 §6` OPEN → D2.**

### W16 — Performance budgets have three claimed single sources of truth

**Ruling.** Discharged by **X9** and **N12**. `32 §8` is the source; `perf/budgets.json` is generated from it; `34 §11`, `31 §10` and `40 §13.1` own disjoint extensions; three of the eight "conflicts" resolve as two different measurements sharing a label and both figures are now stated.
**Status: partially APPLIED; render pipeline DEFERRED → PH0 (§5.1).**

---

## 5. Deferred Fixes, Specified Exactly

Each entry states the change precisely enough that applying it is transcription. **The ruling is binding from today**; only the edit is deferred, and only because the edit is large enough that a mechanical pass would risk losing content.

### 5.1 The generated phase columns → PH0, with M0.2

`cargo xtask lint-phases` renders the phase column of eight tables from `docs/phases.toml` and fails on a diff. Until it lands, each table below is stale and `03 §2` governs.

| Table | Change |
|---|---|
| `41 §5.3` | `-progression` 2→PH1 · `-governance` 2→**split rows**: Charter/roles PH1, Proposal/Vote/moderation PH3 · `-agent` 2→**split rows**: CapabilitySet PH0, Agent/Envelope/Workflow PH3 · `-economy` 2→PH2 (crate) with no Source before PH4 · `-discovery` 2→PH5 |
| `41 §5.4` | Add `fractal-app-atlas`, PH1, hosted by `fractal-app-projection` until PH5 |
| `41 §5.5` | `-ffmpeg` 4→PH2 · `-rail-internal` 2→PH1 · `-tantivy` 4→PH4 |
| `30 §4.2` | Identity/Discourse 0→PH1 · Society stays PH0 · Vault 1→PH2 · Ledger 1→PH1 · Agent 1→PH3 · Governance 2→PH3 · Progression 2→**not a family**, a `citizens` sub-resource · Economy 2→PH4 · Asset 3→PH4 · Extension 3→PH3 · Discovery 3→PH5 · Market 6→PH6 · Signals→**protocol**, `/v1/subscriptions` PH2 |
| `17 §3.2` | S4 2→PH5 · S6 2→PH5 · S5 3→PH6 · S7 3→PH8 · S3 5→PH7 · S8 5→PH7 · S9 6→PH6 · S10 6→PH8 · S1/S2 4→PH4 unchanged |
| `17 §4` | K1/K2/K4 2→PH2 (the first-hearth Society is free at PH1 so no Sink fires) · K7 6→PH6 with the 12% figure (X5) · others unchanged |
| `19 §17` | "4–5"→**PH6** for every row in that band |
| `20 §2` | `workflow` 4→PH3 · `automation-pack` 4→PH4 · others unchanged |
| `21 §14` | `ContributionReceipt` row: PH2 renders **XP only**; the S4 leg arrives PH5 |
| `16 §20` | `FN-ASSET/1` core and the evolution engine 3→**PH4** (M4.11) · internal anchoring 2→PH2 unchanged |
| `50` | PH0/PH1/PH2/PH3/PH4 budget lines restated per `03 §4.2`; PH4 gains M4.11; PH7 ships two first-party Experiences (pending D3) |

Also in PH0, with M0.5: the `perf/budgets.json` render pipeline (X9) and the doc link checker (X15).

### 5.2 `economy/rates.toml` and the corrected storage model → PH1

Create `economy/rates.toml`, owned by `17`, as the single source for: the anchor, K8 and K9 tariffs, S1 and S2 rates, the replication factor (imported from `13 §6.1`), σ's floor, the Citizen and Society free-quota ladders, and the S1/S2 emission shares. `13 §8.2`'s worked example and `18 §5.1`/`§5.2`'s quota tables are generated from it.

Edits: `13 §8.2` — delete the assumed `cap_w = 5,000 FRC/day`, derive it as `0.22 × B(n) / 365`; delete "at most 20% of the platform's annual emission ceiling" and the 1,825,000 FRC/yr bound; restate the peg constraint at **$0.0142**. `17 §3.3` — replace the worked model with X6's corrected block. `17 §12` — add **I-E11** (the modelled storage rate equals `13`'s), **I-E12** (modelled quota equals `18`'s ladder over the actual Level distribution), **I-E13** (S1+S2 share within `17 §3.2`), **I-E14** (the free-grant ceiling: `Σ free_GB × 1.60 × 0.28 ≤ 0.60 × S1_share_of_B(n)`).

### 5.3 `GenesisAllocation` → PH1

`17 §4` gains a pre-registry row above K1, marked **not a Source and not a Sink**, carrying every field from X-GA's table. `11 §2.6` gains one sentence on the `PostingReason` closed enum noting the retired-variant rule. `16 §20` gains a row: "`GenesisAllocation` issuance — PH1, retired at the PH4 exit gate". `51 §16` Q2 is marked **superseded by `61 X-GA`** — the grant is larger, capped in aggregate, Citizen-inclusive, and posted from `EmissionAccount` rather than a separate `GenesisAccount`.

### 5.4 The first-hearth capability string → PH1

`30 §4.3` #6: capability becomes `society.create`, with the endpoint note "unconditional while the Citizen's first-hearth allowance is unconsumed; `Level ≥ 3` and the K1 250 FRC charge thereafter". `30 §12.2`'s generated capability registry carries the predicate. `51 §16` Q1 marked **resolved by `61 X7`**.

### 5.5 MLS figures in leaves → PH1

`14 §4.1`: "sizes from 3 to 5,000" becomes "Societies from 3 to 5,000 Citizens; any single E2EE Chamber holds at most **1,000 leaves** (≈400 Citizens at the published 2.5 devices/Citizen)". `14 §4.2` states the 2.5 assumption explicitly and cites it. ADR-0010 §3 and §9: "~1,000 members" → "~1,000 leaves". `50 PH2` risk row: "≤ 200 members" → "≤ 500 leaves (≈200 Citizens)".

### 5.6 Offline Agent semantics → PH3

`15 §6` rule 5 gains: "Inference is refused offline unless a local `ModelProvider` is configured (`34 §12.1`); an outboxed Agent command whose Envelope has expired or been revoked before arrival **fails at the PEP** and is surfaced as `AgentActionBlocked`. No command is authorized retroactively against the grant that existed at enqueue." `34 §12.1`'s "Refused" row gains: "— inference only. Action authorization is always deferred to reconnect (`15 §6`)."

### 5.7 Custodian cohort and the PH4 entry precondition → PH4

`13 §11.4`: a new step 4.0 — "committed capacity ≥ 3× current logical storage across ≥ 8 failure domains at the modelled price, measured, before step 4.1". `13 §12` gains a pre-recruitment paragraph naming the arrangement as published and explicit. `50 PH4` entry criteria gain the same precondition.

### 5.8 Theme names and token paths → PH0, with M0.5

`41 §10.2`: `default.json`/`high-contrast.json`/`terminal-amber.json` → `void.json`/`daylight.json`/`contrast.json`, with the CLI palette variant named `cli-amber` and moved out of `theme/`. `32 §2` and `32 §5`: `tokens/` → `packages/tokens/src/`, including the `color-no-hex` allowlist path.

### 5.9 `EmissionAccount` sharding and settlement offset → PH1

`16 §4.2` and `§4.4`: K = 64 shards, `i = BLAKE3(society_id) mod 64`, each pre-allocated `1/64` of the epoch budget; the same hash and modulus as the `ShardRouter`. `13 §8.2`: settlement window closes at `BLAKE3(society_id) mod 86400` seconds past 00:00 UTC rather than at 00:00 UTC.

### 5.10 The emission epoch origin → PH1

`17 §5.1` R2: *n* becomes epochs since `emission_epoch_origin`, set once by S1's `SourceActivated` event. `17 §5.2`: the ten-year table is re-based on the origin and its narrative no longer refers to calendar years. `17 §13`: shape → Tier 0, origin → Tier 1a (set once, by event). `50 §5`: new row — "PH4 slips past month 30 → re-run `17 §5.2` against the actual origin before the PH4 gate".

### 5.11 Projection checkpoints and the rebuild SLO → PH1 / PH2

PH1: generalize `16 §4.3`'s checkpoint mechanism to every projection at 4,096 events per projection per Society. PH2: `40 §9.4` gains the rebuild SLO (≤ 15 min p99, any projection, any Society); `41 §15` gains a ≥ 100 M-event large-Society fixture; `50 PH2` gains the acceptance criterion.

### 5.12 `HistoryKey` separation → PH2

`14 §4.5`: the Vault-backed archive row's Default becomes **off**, and "or via social recovery" becomes "or, if `HistoryEscrowEnabled` is granted, via social recovery". The disclosure string becomes the sentence in W7. `12 §6.3`: DM archive keys move to the not-recoverable list unless `HistoryEscrowEnabled` is live. `12 §13`: new **I-12.13**. `34 §12.2`: evictable-storage-only Citizens are prompted for a second factor before holding Fraction or a Membership above Level 1.

### 5.13 Extension tool definitions as untrusted content → PH3

`15 §13.1`: tool definitions supplied by an Extension are rendered inside the `UNTRUSTED` region and namespaced `ext.<install_id>.<name>`; the system region enumerates host-native tools. `15 §14.1`: new **A11**. `20 §6`: hook 31 documents the namespacing; hook 18 requires host-drawn fee disclosure naming the Install, and `ExtensionQuoteClamped` fires at the cap as well as above it. `20 §11`: per-Install fee-revenue ceiling. `50 PH3` AC-3: the injection suite gains tool-definition cases.

### 5.14 Atlas consumers → PH1

`13 §10.2`: the refcount Projection is owned by S15 and is monotone-safe; GC requires positive confirmation from every referencing Society at its current `seq`. `13 §14`: new **V13**. `14 §2`: `Scope::Self_` is populated by S15 Atlas with the stated staleness bound. `30 §4.3` #41: `GET /v1/search` states that it is served by Atlas, is index-only, and carries `as_of` and `stale`.

### 5.15 Fracture rekey downstream → PH5

`13 §10.1`: the residual register gains the Fracture case — pre-fracture Versions stay decryptable by pre-fracture key holders; post-fracture Versions do not. `50 PH5` AC-1: the confidentiality property test over the same 100,000 generated splits.

### 5.16 Roadmap durations and cut candidates → PH0

`50 §1`: durations restated as ranges with `60 §3.11`'s multiple stated. `50 §5`: a "phase exceeded 1.5× its estimate" row for every phase. Cut candidates are already in `phases.toml`.

### 5.17 Recovery cold start → PH1

`12 §6.1`: the platform guardian of last resort, with the sub-threshold rule and the `02 §4` argument stated. `12 §3.3`: the second device is prompted before guardians. `50 PH1`: recovery-set formation rate as an acceptance criterion with a floor, and a risk row naming recovery-set formation.

---

## 6. Open Decisions — for Andrew

Five decisions this pass could not make, each with a recommended default. **Work is not blocked on any of them**: an agent adopts the default, cites the decision id in the PR, and the default becomes an ADR at the phase gate if it is not overturned first — the same protocol `51 §16` uses.

### D1 — Timeline, staffing, and the single human approver

**The question.** `60 §3.11` puts the realistic multiple at 2.0–2.6×, and `60 A10` identifies the only true single point of failure in the design: one human approves every Milestone, signs every release and reviews every Work Unit, with no delegation path, no second approver and no degraded mode. At 1,000–2,000 Work Units for PH0–PH3 and fifteen reviews per productive day, that is **70–130 reviewer-days of pure review** before that reviewer does anything else.

**Recommended default.** Fund a **second approver before PH1 begins**, with a written scope — they may approve Work Units and Milestones, and may not accept an ADR or sign a release tag, which stay with Andrew. If a second approver is not funded, `42` must state plainly that **human review is the throughput ceiling** and `50 §1`'s durations must be re-derived from review capacity rather than from engineering effort. Both answers are acceptable; only leaving it unstated is not.

**Blocks:** nothing. **Decide by:** PH1 entry.

### D2 — Jurisdictional posture on E2EE

**The question.** `14 §6` and `13 §10.3` foreclose scanning of E2EE content by construction, enforced as a crate-graph lint. Several jurisdictions have enacted or are enacting duties a report-only posture does not satisfy. The blueprint's response exists in three chapters as a technical statement and nowhere as a business decision: no list of jurisdictions served, no stated response to an order the architecture cannot satisfy, no gate at which the question is answered.

**Recommended default.** Adopt the shape `12 §14` already uses for identity Rails: **publish the list of jurisdictions the platform intends to serve, and pre-commit to geoblocking any jurisdiction that mandates proactive scanning of E2EE content**, rather than overturning N6. Overturning N6 changes what the product is, and that decision should require an ADR against a live regulatory order, not against a forecast. **Requires counsel** — this is not a legal opinion.

**Blocks:** `14 §6`'s new posture section. **Decide by:** PH1 exit.

### D3 — Two Sources in PH7, against `50 PH7`'s stated zero

**The question.** `17 §3.2` defines ten Sources; `02 §5` allows two per phase; no Source may activate before PH4 (N3). Two per phase fills PH4, PH5 and PH6 and leaves four. Placing S3 (Compute Contribution) and S8 (Agent Development) in PH7 overrides `50 PH7`'s stated 0-Sources budget line and requires something to leave PH7 (`02 §8`).

**Recommended default.** **Accept the override, and cut the third first-party Experience** — `50 PH7` ships two. Rationale: S3's compute auction (K10) already exists from PH5 with no matching Source, which means compute providers are paid entirely by redistribution for two phases; and the Experience host is the largest consumer of purchased compute in the corpus, so pairing S3 with PH7 is where it belongs on the merits, not only on the budget. The alternative is to push S3 and S8 to PH8, which breaches PH8's Source budget instead and leaves the compute market unfunded for four phases.

**Blocks:** `50 PH7`'s milestone list. **Decide by:** PH6 exit.

### D4 — S10 Onboarding & Vouching does not activate until PH8

**The question.** Holding `02 §5`'s two-Sources-per-phase line places S10 — the corpus's **only designed growth mechanism** — at PH8. `60 A5` already identifies that the platform bans advertising (`02 §4`), paid placement (`19 §8`) and every behavioural signal (`14 §8`), leaving declared-interest discovery and Serendipity at 2/Citizen/week, and that "growth must then come from invitation, community seeding or integrations — none of which is designed or budgeted anywhere in the corpus". Deferring the invitation Source to PH8 makes that gap worse.

**Recommended default.** **Leave S10 at PH8 and fund invitation separately, without a Source.** Invitation does not need to *pay* to work: the mechanism (an invite that carries a vouch, a 90-day qualification, a slashable 50 FRC stake) can ship at PH5 with the reward set to zero and the Source activated later. That preserves the budget, exercises the anti-Sybil machinery for three phases before any money flows, and separates the growth mechanism from the growth incentive — which is the safer order.

**Blocks:** nothing immediately. **Decide by:** PH4 exit, because the invite mechanism must be designed before it is needed.

### D5 — The Extension UI escape hatch

**The question.** `20 §7`'s Surface Descriptor cannot express a diagram editor, a spreadsheet grid, a waveform scrubber or a game — the categories that generate marketplace revenue everywhere else. The stated escape hatch is the Experience Runtime at PH7, realistically three to four years out, behind seven gates. Meanwhile `19 §14` targets an earnings Gini ≤ 0.75 and time-to-first-sale ≤ 21 days at PH6, and `19 §1`'s four ways a marketplace dies omits the fifth: nothing worth selling.

**Recommended default.** **Option A** — pull one tightly-scoped primitive forward into PH6: a **host-owned, host-chromed, input-mediated canvas** with no ambient I/O, no persistence import, a fixed frame budget, and no ability to draw outside its own rectangle. It is a strict subset of `20 §12.1`'s Experience client, so it costs sandbox work but not governance, tick-loop, session or economy work — roughly six engineer-weeks plus an ADR plus a PH6 budget slot, paid for by cutting Society storefronts (PH6's designated cut candidate). Option B — accept the constraint and rewrite `19 §1` and `19 §14` for a marketplace of automations and templates — is a legitimate product and costs a day, but the 12% fee (X5) should then be re-justified against it.

**Blocks:** `19 §1`, `19 §14`, `20 §7`. **Decide by:** PH5 exit — *before* PH6 opens, not at its gate, because a hard line under commercial pressure at a gate is a hard line that breaks.

### D6 — Two Canon amendments require ADRs

**The question.** Two rulings in this ledger amend `00`/`02`, and `00 §7` requires an ADR for a Canon change. Neither is contentious; both need the paperwork before the PH0 gate, where `02 §5` requires zero open ADRs.

1. **ADR-0015 — `03-phase-authority.md` joins the Canon.** Amends `00 §6` (three files → four) and `02`'s header. Consequence list: every generated phase column, `cargo xtask phase-check`, `docs/phases.toml`.
2. **ADR-0016 — the dependency budget is per deployable artifact.** Amends `02 §5`'s dependency row (N2). Consequence list: `03 §4.1`, `51 §2.3`, every phase's budget line in `50`.

**Recommended default.** Draft both in PH0 alongside M0.2, accept both at the PH0 gate. An agent may draft them; only Andrew may accept them (`40 §6.5`, P4).

**Blocks:** the PH0 exit gate. **Decide by:** PH0 exit.

---

## 7. What Changed in This Pass

Every edit is listed. Line counts are before → after; **no chapter shrank**, which is the check that a mechanical pass did not destroy content.

| File | Lines | Words | What changed |
|---|---|---|---|
| **`03-phase-authority.md`** | *new*, 469 | 8,200 | The fourth Canon file. Precedence statement · PH0–PH9 with `50`'s goals and `60`'s realistic ranges · the 203-row master capability→phase table · 21 rulings carried · the six-axis complexity-budget ledger with three rebalanced phases · the `phases.toml` schema and the six xtask assertions · the amendment rule |
| **`docs/phases.toml`** | *new*, 1,962 | 7,700 | Ten `[[phase]]` blocks with budgets and cut candidates; 203 `[[capability]]` rows with `id`, `name`, `phase`, `owner`, `milestone`, `kind` and `consumes`. Validates as TOML; its derived budget sums reproduce `03 §4.2` exactly |
| **`61-reconciliation.md`** | *new*, 1,110 | 18,000 | This ledger |
| `00-foundational-principles.md` | 220 → 220 | 2,540 → 2,585 | §6 three-file Canon → four, with `03`'s precedence stated · P12's broken harness reference → `17 §12` · §6 scope rule now points at `03 §2` and `phases.toml` |
| `01-canonical-terminology.md` | 228 → 249 | 2,354 → 2,832 | Four-file Canon in the header · §1's P1 invariant becomes the three-clause form with the Citizen Wallet/Vault diagram · Handle grace = 14 days, once · **`Genesis Allocation`** added to §4 · **`Atlas`** added to §6 · Global Registry entry 1 annotated · `Vault` redefined for both owners · **eight verbs added to §8**: `publish`, `purchase`, `refund`, `license`, `payout`, `recall`, `review`, `curate` |
| `02-scope-guardrails.md` | 130 → 135 | 1,319 → 1,534 | Four-file Canon · phase-authority pointer in the header · §3's Not-Yet table restated in `PH<n>` notation and **marked non-normative and generated** · §5's dependency row amended to per-deployable-artifact · budget-summation note · §6 question 2 rewritten to point at `03` |
| `10-system-architecture.md` | 387 → 431 | 3,599 → 4,441 | §1 Redis removed from the topology diagram · **§3 gains S15 Atlas** with its consistency model, staleness bound, read-only rule and monotone-safe refcount posture · §4's cost paragraph now names its owner · **§12's Postgres remedy replaced** with the four-step ladder, measurable triggers, the single-transaction consequences and the six code preconditions · §12's MLS trigger restated in leaves |
| `11-domain-model.md` | 589 → 621 | 3,428 → 4,296 | §2.3 member ladder completed to SL5 with the E2EE ceiling separated · first-hearth note · §2.7 gains `Vault` with `society: Option<SocietyId>` and `owner`, `Object` gains `society_id: Option<…>`, plus the Citizen Vault paragraph and the rejected `VaultScope` alternative · **§3.2 gains the mandatory Fracture rekey step** and a sixth Fracture invariant in prose · §7 invariants 1, 4, 10 and 15 rewritten; **invariant 16 added** |
| `13-data-and-storage.md` | 658 → 658 | 7,068 → 7,142 | §8.3 and §14 V5: σ becomes a floor of 1.25, observed 2.23, with the refusal rule |
| `14-realtime-and-social.md` | 486 → 486 | 6,791 → 6,876 | §2 presence moves from Redis to Relay-process memory gossiped over NATS · §4.4 states the leaf/Citizen conversion and corrects §4.1's scope |
| `16-ledger-and-assets.md` | 1,083 → 1,085 | 11,857 → 12,077 | §2.1 `Posting.amount` → `i64` · §4.4 batch sum in `i128`, checked · **§5 gains the full `Quanta` ruling row** (signedness, width, overflow, wire format, storage) · §19 invariants 2 and 3 restated for the K = 64 `EmissionAccount` shards · broken `30-api-contract.md` reference fixed (2 sites) |
| `17-economy-fraction.md` | 772 → 773 | 10,341 → 10,418 | **K7 restated at 12%** with `19 §6.1`'s 6/4/2 pp split · **K7b `Sink::ShelfShare`** added as a Redistribute, excluded from SCR |
| `18-progression-and-reputation.md` | 779 → 798 | 9,509 → 10,156 | **§5.1 first-hearth exemption** at L0 with the four anti-farming details, and L3 restated as second-and-subsequent · **§5.2 quota ladder re-scaled** SL0–SL5, **SL4's unmetered grant deleted**, member ceilings added, E14 stated · **§5.5 Custodian eligibility rewritten** — Level gate deleted, bond scaled by committed bytes, probation added |
| `19-marketplace.md` | 728 → 728 | 10,480 → 10,531 | §13 T6: a detected self-purchase is **void**, not 100%-fee'd, preserving §16 invariant 4 |
| `20-plugin-and-extension-model.md` | 647 → 647 | — | Broken `19-marketplace-and-commerce.md` reference fixed |
| `21-media-and-identity.md` | 895 → 895 | 12,885 → 12,938 | §17's Canon amendment marked **LANDED** with the fuller shape actually applied |
| `32-design-system.md` | 357 → 359 | 3,131 → 3,284 | §8 desktop and web memory rows corrected to `34`'s stricter figures · **the ownership partition stated**: `32 §8` is the source, `perf/budgets.json` is generated from it, and `34`/`31`/`40` own disjoint extensions |
| `33-brand-identity.md` | 396 → 396 | 3,625 → 3,628 | §7.3 onboarding copy: "You can change it once, **within 14 days**" |
| `34-client-platform-strategy.md` | 787 → 800 | 12,484 → 12,806 | **§2.1 core box redrawn** — `fractal-app` out, `fractal-core` in, with a dashed server-only box naming the PEP and the advisory-hint rule · §11 warm-start, JS and CLI rows corrected; the generation direction reversed and the ownership boundary stated |
| `40-engineering-standards.md` | 1,278 → 1,283 | 15,496 → 15,797 | **§6.4's worked ADR renumbered 0009 → 0014**, dated 2026-09-03, §6.6 → §7.6, and marked a transclusion point · §7.5 pre-phase-gate seeds 5,000,000 → **500,000** with the rate discrepancy explained and the 5 M soak retained as an annual, non-gate exercise · §7.6 Redis removed from integration dependencies · §13.1's ownership claim corrected |
| `41-repo-and-crate-structure.md` | 1,302 → 1,306 | 13,394 → 13,567 | §5.4 gains **`fractal-app-atlas`** with its dependency set and hosting arrangement, plus the statement that S15 has no domain crate and why |
| `42-source-control-automation.md` | 1,141 → 1,141 | — | Broken `30-api-contract.md` reference fixed |
| `50-roadmap-phases.md` | 499 → 502 | 5,565 → 5,859 | Header: `03` owns phase assignment, `60`'s multiple stated · PH0 entry criteria include `03` and `phases.toml` · **PH2 AC-2 memory 600 MB/4 Societies → 450 MB/5** · PH2 risk table gains the projection-rebuild row and restates MLS in leaves · **§3 gains a fifth continuous track, Safety Operations**, gated first at PH1 exit · Documentation track gains the three new lints |
| `51-phase-1-web-gui.md` | 1,034 → 1,034 | 15,041 → 15,191 | §16 Q1, Q2, Q5, Q6 and Q11 marked **RESOLVED** or **SUPERSEDED** against their `61` ruling, with the rulings that differ from the proposed defaults stated inline |
| `adr/0006-postgres-as-primary-store.md` | 69 → 71 | — | §2 `Quanta` → `i64`/`i128` with the reasoning · §7.4 falsification test extended with the headroom assertion and the wire-format test · **§9 review trigger (a) corrected**: partitioning does not add write throughput, with a pointer to `10 §12`'s real ladder |

**Not edited, deliberately.** `12`, `15`, `30`, `31`, `adr/0010` and `adr/0014` carry rulings whose edits are specified in §5 and deferred, because each is a section rewrite rather than a surgical replacement and a mechanical pass over them would risk losing content. `60-self-critique.md` is **not edited at all**: it is a dated artifact of a review, and amending a critique to match its resolutions destroys the record of what was found. Read `60` for the findings and this chapter for what is true.
