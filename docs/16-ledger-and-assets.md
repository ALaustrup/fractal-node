# 16 — Ledger and Assets

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `10-system-architecture.md`, `11-domain-model.md`.
> **Governs:** the `Ledger` port and its Phase 1 implementation, determinism and replay, Anchoring, the migration path to a future Fractal Node L1, the `Chain` and `Rail` ports, and the `FN-ASSET/1` Facet Standard — identity, schema, evolution, ownership, custody, licensing, provenance, transfer, and external bindings.
>
> **Does not govern:** Sources, Sinks, emission policy, Contribution Score, fee schedules, or any economic parameter. Those are `17-economy-fraction.md`. This document defines the *machine*; `17` defines what is *fed into it*. Where a number is needed here it is illustrative and marked as such.

---

## 1. The Two Commitments

This chapter delivers two things that P11 demands and that most platforms get wrong in the same way — by letting a vendor's data model become the domain model.

1. **The Ledger is ours, internal, deterministic, and double-entry, behind a trait, from commit #1** (N4). It is not a chain, not a database table people write to directly, and not a balance column. It is an append-only sequence of balanced Postings that happens to be stored in Postgres in Phase 1 and may be stored somewhere entirely different in Phase 8 without a single line of domain code changing.

2. **Facets are a native asset primitive designed around evolution, not around token transfer.** We are not modelling ERC-721 or ERC-1155 and then adding features. We are specifying what a digital object owned inside a Society actually needs to be, and treating any external chain representation as a lossy projection of the canonical record we hold.

Both have the same shape: **the abstraction is created before its second implementation exists, because that implementation is known to be coming** (`02 §7`, anti-gold-plating exception).

---

# PART A — THE LEDGER

## 2. The `Ledger` Port

### 2.1 The trait

```rust
/// fractal-ports::ledger
///
/// The ONLY way any domain or application code touches Fraction.
/// No domain crate may reference a ledger implementation type (P5 lint).
#[async_trait]
pub trait Ledger: Send + Sync + 'static {

    /// Apply one atomic, balanced group of Postings.
    /// Either every Posting in the batch commits, or none does.
    async fn post(&self, batch: PostingBatch) -> Result<PostingReceipt, LedgerError>;

    /// Balance of a Wallet as of the ledger's current committed frontier.
    async fn balance(&self, wallet: WalletId) -> Result<Balance, LedgerError>;

    /// Balances of many Wallets at one consistent frontier. Not a loop over
    /// `balance` — a caller comparing two wallets read at different frontiers
    /// would observe a false imbalance.
    async fn balances(&self, wallets: &[WalletId]) -> Result<BalanceSet, LedgerError>;

    /// Lock funds against a future settlement (escrow, stake, in-flight transfer).
    /// Moves `amount` from available into `locked` WITHOUT changing `balance`.
    async fn lock(&self, req: LockRequest) -> Result<LockId, LedgerError>;

    /// Release a lock without moving funds.
    async fn release(&self, lock: LockId) -> Result<(), LedgerError>;

    /// Consume a lock by posting it. Atomic with the posting.
    async fn settle(&self, lock: LockId, batch: PostingBatch)
        -> Result<PostingReceipt, LedgerError>;

    /// Open a Wallet. Idempotent on `(owner, society, purpose)`.
    async fn open_wallet(&self, spec: WalletSpec) -> Result<WalletId, LedgerError>;

    /// Ordered, paginated Postings touching a Wallet. The auditable history.
    async fn statement(&self, q: StatementQuery) -> Result<Statement, LedgerError>;

    /// Deterministic state root over all Wallets in a Society at a frontier.
    /// Input to Anchoring (§6).
    async fn state_root(&self, scope: LedgerScope, at: Frontier)
        -> Result<StateRoot, LedgerError>;

    /// Assert the global invariants. Called by CI, by the reconciler, and
    /// before every Anchor. Returns the witness on failure, not a bool.
    async fn audit(&self, scope: LedgerScope) -> Result<AuditReport, LedgerError>;
}

pub struct PostingBatch {
    pub batch_id:       Ulid,          // idempotency key — see §2.3
    pub society_id:     SocietyId,     // P1: every Posting is society-scoped
    pub postings:       Vec<Posting>,  // >= 1, must balance as a SET
    pub caused_by:      EventId,       // the domain event that justifies this
    pub correlation_id: Ulid,
}

pub struct Posting {
    pub debit:  WalletId,
    pub credit: WalletId,
    pub amount: Quanta,                // i64 quanta, checked; never floating point (17 §2.1)
    pub reason: PostingReason,         // closed enum, defined in `17`
}

pub struct PostingReceipt {
    pub batch_id:  Ulid,
    pub frontier:  Frontier,           // (society_id, ledger_seq) after commit
    pub postings:  Vec<PostingId>,
    pub applied:   Applied,            // Fresh | AlreadyApplied
}
```

### 2.2 What the trait guarantees

| # | Guarantee | Statement |
|---|---|---|
| L1 | **Atomicity** | A `PostingBatch` commits entirely or not at all. There is no partially applied batch, at any frontier, under any failure. |
| L2 | **Balance invariant** | For every committed batch, `Σ amounts debited == Σ amounts credited`. A batch that does not balance is rejected before any write. Globally: `Σ all wallet balances == 0`. |
| L3 | **Idempotency** | `post` with a previously committed `batch_id` returns the original receipt with `Applied::AlreadyApplied` and writes nothing. Retries are always safe — from the CLI, a flaky mobile network, or a saga resuming (`11 §5`). |
| L4 | **Deterministic replay** | Replaying the Posting log from genesis on a clean store yields byte-identical balances and an identical `state_root` at every frontier. |
| L5 | **Non-negativity** | No committed batch may leave `balance >= locked >= 0` false for any Wallet, except the `EmissionAccount` (§4.2). Checked inside the transaction, not after. |
| L6 | **Ordering** | Postings are totally ordered within a Society (`ledger_seq`) and only causally ordered between Societies. This is the same ordering decision as the event log (`10 §4`), for the same reason. |
| L7 | **Immutability** | No Posting is ever updated or deleted. A correction is a new, opposite Posting with `reason = Correction{original}`. There is no `UPDATE` statement against the Posting table in the codebase, and a lint enforces that. |

### 2.3 What the trait deliberately does NOT expose

Every omission below is a place where a leaked concept would later force a rewrite. This list is as load-bearing as the trait itself.

| Not exposed | Why |
|---|---|
| Any notion of a *block*, *confirmation depth*, or *chain height* | These are properties of one family of implementations. A caller that branches on confirmation depth cannot be moved to a ledger without one. Finality is expressed only as `Frontier` + `FinalityClass` (§7.3) — an opaque, comparable token. |
| A fee parameter on `post` | Fees are Postings, authored by the domain against a named `PostingReason` from `17`. A ledger-level fee knob would smuggle a chain's cost model into the domain and violate P12's "every Fraction has a named source and sink". We never write the word that begins with "g". |
| Raw SQL, a transaction handle, or a connection | Would make the Postgres implementation load-bearing. Multi-aggregate atomicity is achieved by the application layer passing a whole `PostingBatch`, not by sharing a transaction. |
| Signing keys or address formats | Ownership is a `Principal`/`WalletId` in our identity system (`12`). External address formats live in the `Chain` adapter and never enter the domain. |
| Wall-clock time as an input | `Clock` is a separate port (`10 §7`). Postings receive the domain event's `occurred_at`. See §5. |
| Mutation of history | See L7. |
| Currency selection | There is exactly one unit of account: Fraction, in integer quanta. Multi-asset accounting would be a different trait, and `17` explicitly does not need one. |

**Falsification test for P11 in this chapter:** implement `Ledger` as a stub backed by an in-memory map plus a local test chain anchor. If any crate under `fractal-domain-*`, any API contract in `30-api-and-sdk.md`, or any client requires a change, the abstraction has failed and the PR is rejected.

---

## 3. The Ledger is the Event Log

The classic dual-write problem is: write the business event, then write the money movement, and pray the process does not die between them. Outbox tables, two-phase commit, and reconciliation jobs are all treatments for a disease we simply do not contract.

**A Posting is an event.** `PostingRecorded` is a Domain Event in the Society's Log (`10 §5`, boundary S5). The Ledger's Posting table is a **projection** of that Log, exactly like the Discourse read model — disposable by definition (P6), rebuildable by replay.

```
   command: TransferFraction
        │
        ▼
  ┌──────────────────────┐   ONE transaction, ONE log, ONE append
  │  DOMAIN decides:     │
  │  balance sufficient? │
  │  policy satisfied?   │
  │  produce Postings    │
  └──────────┬───────────┘
             │ emits
             ▼
  ┌──────────────────────────────────────────────────────────────┐
  │  SOCIETY LOG                                                 │
  │  seq 4471  TransferRequested   { from, to, amount, corr }    │
  │  seq 4472  PostingRecorded     { batch, postings[], reason } │  ◄── the money
  │  seq 4473  TransferSettled     { transfer_id, frontier }     │
  └──────────┬───────────────────────────────────────────────────┘
             │ fan-out (all derived, all rebuildable)
   ┌─────────┼──────────┬──────────────┬─────────────┐
   ▼         ▼          ▼              ▼             ▼
┌────────┐┌────────┐┌──────────┐┌────────────┐┌──────────┐
│ Wallet ││Statem- ││ Treasury ││  Economy   ││ Signal   │
│balances││ent proj││ dashboard││  metering  ││ fan-out  │
└────────┘└────────┘└──────────┘└────────────┘└──────────┘
```

Consequences, all of them the point:

- **No dual write.** The event and the money are the same append. There is no window.
- **The economy is replayable.** P12's "every Fraction that ever moved is a replayable fact" is literally true, not aspirational.
- **A corrupt balance is a projection bug, not a loss.** Drop the wallet projection, replay, and it is correct again. This is the single strongest argument for event-sourcing money, and it only holds if the fold is genuinely deterministic (§5).
- **`Correction` is legible.** A reversal is two events far apart in the log, both permanent. Auditors see what happened *and* what was done about it.

**Invariant LE1:** the Posting projection at frontier F is bit-identical to the fold of all `PostingRecorded` events up to F. The reconciler (§4.5) asserts this continuously; CI asserts it on every generated history (`11 §7.2`).

---

## 4. Phase 1 Implementation — Double-Entry over Postgres

### 4.1 Why Postgres and not something clever

Serves P12 (correctness of the economy) and operability. Postgres gives us: real ACID transactions across the event append and the balance projection; `SERIALIZABLE`/`REPEATABLE READ` when we want it and row locks when we do not; `numeric`/`bigint` integer arithmetic with no floating point anywhere; logical decoding for CDC; and the largest operational knowledge base of any database (`10 §11`). Exit cost is bounded precisely because everything sits behind `Ledger`.

*Alternatives rejected:* a dedicated ledger product (Formance, TigerBeetle) — TigerBeetle is genuinely excellent at this shape and is the most credible Phase 4+ swap, but adding a second stateful system before the spine sentence is true costs a full complexity-budget slot (`02 §5`) and buys throughput we do not yet need; a chain, now — see §11.

### 4.2 The account tree

Every account is a Wallet (`11 §2.6`). There is no second kind of thing. System accounts are Wallets owned by the `System` principal, distinguished only by `purpose`.

```
                        ┌────────────────────────────────┐
                        │        LEDGER ROOT             │
                        │   Σ all balances == 0 (L2)     │
                        └───────────────┬────────────────┘
                                        │
        ┌───────────────────┬───────────┴───────┬────────────────────┐
        ▼                   ▼                   ▼                    ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐   ┌────────────────┐
│ EMISSION      │   │ CIRCULATING   │   │ SYSTEM        │   │ BURN           │
│ (global)      │   │               │   │ (per society  │   │ (global)       │
│               │   │               │   │  + global)    │   │                │
│ balance is    │   │               │   │               │   │ balance only   │
│ NEGATIVE.     │   │               │   │               │   │ ever increases │
│ -balance ==   │   │               │   │               │   │                │
│ total emitted │   │               │   │               │   │                │
└───────────────┘   └───────┬───────┘   └───────┬───────┘   └────────────────┘
                            │                   │
              ┌─────────────┼─────────┐   ┌─────┴───────┬──────────────┐
              ▼             ▼         ▼   ▼             ▼              ▼
       ┌────────────┐┌───────────┐┌────────────┐┌────────────┐┌──────────────┐
       │ Citizen    ││ Society   ││ Agent      ││ ESCROW     ││ FEE          │
       │ Wallet     ││ TREASURY  ││ Wallet     ││ (per soc.) ││ (per soc.)   │
       │ (global +  ││           ││            ││ holds      ││ collects,    │
       │  per-soc.) ││           ││            ││ locked     ││ then splits  │
       └────────────┘└───────────┘└────────────┘│ funds mid- ││ per Charter  │
                                                │ settlement ││ + platform   │
                                                └────────────┘└──────────────┘
                                                ┌────────────┐
                                                │ STAKE      │
                                                │ (per soc.) │
                                                │ bonds;     │
                                                │ slashes to │
                                                │ BURN or FEE│
                                                └────────────┘
```

| Account | Scope | Sign | Role |
|---|---|---|---|
| `EmissionAccount` | Global | Always ≤ 0 | The only account permitted a negative balance. Every Emission is a Posting debiting Emission and crediting a Wallet. **Total supply is `-EmissionAccount.balance`** — a directly queryable number, never an estimate (`11 §2.6`). The exemption to L5 is *one account*, and it is the account whose negativity is the definition of supply. |
| `BurnAccount` | Global | Monotonically ≥ 0 | Terminal sink. Fraction credited here is out of circulation forever. `circulating == -Emission.balance - Burn.balance`. |
| Citizen / Society Treasury / Agent Wallets | Per Principal (Treasury per Society) | ≥ 0 | Ordinary holders. |
| `EscrowAccount` | Per Society | ≥ 0 | Holds funds during multi-step settlement — Facet sales, marketplace purchases, conditional transfers (§14.3). Escrow always has a named counterparty and a deadline. |
| `FeeAccount` | Per Society + global | ≥ 0 | Transient. Fees land here and are split to Treasury, creators, and the platform by rules in `17`. A non-zero `FeeAccount` balance at the end of a settlement window is an alert, not a feature. |
| `StakeAccount` | Per Society | ≥ 0 | Holds Stakes. A slash is a Posting from Stake to Burn or Fee, never to a person's Wallet — nobody profits from another's slash, because that creates an incentive to manufacture slashes. |

**Invariant LA1:** exactly one account in the entire system may hold a negative balance, and it is `EmissionAccount`. Any other negative balance is a P0 incident.

### 4.3 Balances: running balance, not read-time fold — and why

**Decision: a materialized running balance, written in the same transaction as the Posting, with the event-sourced fold as the authority and a continuous reconciler proving they agree.**

The two candidates:

| | Read-time fold | Materialized running balance |
|---|---|---|
| Read cost | O(postings for that wallet) — unbounded and growing | O(1) |
| Correctness authority | Trivially the truth | Derived; can drift |
| Wallet surface at p75 | Fails P10's 100ms interaction budget within months | Meets it indefinitely |
| Concurrency | Needs a serialization point anyway to check sufficiency | Row lock, natural |
| Auditability | Perfect | Requires the reconciler to be perfect |

Neither pure option is acceptable, so we take the hybrid and name the authority explicitly:

- The **fold over `PostingRecorded` is the definition** of a balance. If the projection and the fold disagree, the fold wins and the projection is rebuilt. This is what makes LE1 meaningful.
- The **projection is a `wallet_balance` row** carrying `(wallet_id, balance, locked, ledger_seq, running_hash)`, updated in the same Postgres transaction as the Posting insert and the event append. Because it is one transaction, the projection can never be *stale relative to a committed Posting* — only wrong if the code is wrong, which is exactly what the reconciler catches.
- `running_hash = BLAKE3(prev_running_hash || posting_id || amount || direction)` gives each Wallet a per-wallet hash chain. Any single-row tamper is detectable without replaying the whole Society.
- **Checkpoints** every 4096 Postings per Society store `(ledger_seq, state_root, per-wallet balances digest)`, bounding rebuild time to one checkpoint interval rather than to genesis.

Honest cost: we maintain two representations of the same fact and a job to prove they agree. That job is not optional, and its failure is a page. We accept that cost because a wallet balance is read thousands of times more often than it is written, and because P10's budget is a hard gate within its own domain.

### 4.4 Concurrency model

```
  post(batch)
     │
     ├─ 1. VALIDATE (pure, no I/O)
     │      • batch balances: Σ debit == Σ credit          ← reject, don't lock
     │      • every reason ∈ PostingReason (closed enum)
     │      • amounts > 0; Σ accumulated in i128, narrowed by i64::try_from —
     │        checked, never wrapping, never saturating (61 X4)
     │
     ├─ 2. IDEMPOTENCY probe: INSERT batch_id INTO posting_batch
     │      ON CONFLICT DO NOTHING RETURNING …
     │      • conflict ⇒ return original receipt, Applied::AlreadyApplied   (L3)
     │
     ├─ 3. LOCK all touched wallets in CANONICAL ORDER (WalletId ascending)
     │      SELECT … FROM wallet_balance WHERE wallet_id = ANY($1)
     │      ORDER BY wallet_id FOR UPDATE
     │      • canonical order ⇒ deadlock is structurally impossible
     │
     ├─ 4. CHECK sufficiency: balance - locked >= debit, per wallet          (L5)
     │
     ├─ 5. APPEND PostingRecorded to the society log (seq = nextval)
     │      INSERT postings
     │      UPDATE wallet_balance (balance, locked, ledger_seq, running_hash)
     │      all in ONE transaction                                          (L1)
     │
     └─ 6. COMMIT ─► receipt { frontier }
```

- **Isolation level `READ COMMITTED` plus explicit `FOR UPDATE` in canonical wallet order.** Not `SERIALIZABLE`: serialization failures under contention on a hot Treasury would produce retry storms and a latency profile we cannot reason about. Explicit ordered row locks give us the same safety for this access pattern with predictable behaviour and no aborts.
- **Deadlock freedom is structural**, not probabilistic: every transaction acquires wallet locks in ascending `WalletId` order, so no cycle can form. A lint rejects any ledger code path that locks a wallet outside `lock_wallets_ordered()`.
- **Contention is bounded by design.** The hot rows are per-Society Treasury, Fee, and Escrow — not global. This is the sharding decision of `10 §4` paying for itself again: a busy Society contends with itself, never with the platform.
- **The `EmissionAccount` is the one global hot row.** Emission is therefore *batched by the settlement run* (`17`), not posted per reward: one Posting group per settlement window per Society, not one per XP award. This is a design constraint on `17`, stated here because it is a property of the ledger's concurrency model.
- **No optimistic writes, ever** (`10 §6`). A Transfer is `PENDING` in every client until the receipt returns. Money does not do eventual consistency.

### 4.5 Holding `Σ debits == Σ credits` under load

Four independent mechanisms, because one is a hope and two is a coincidence:

1. **Construction.** `PostingBatch::new` is the only constructor and it returns `Err` unless the set balances. An unbalanced batch cannot be *represented*, so it cannot be posted.
2. **Transaction.** Balance mutations and the Posting insert share one transaction. A crash between them is not reachable.
3. **Database constraint.** A deferred `CHECK` on the batch aggregate plus a trigger asserting `Σ signed_amount == 0` per `batch_id` at commit. Belt and braces — the trigger has caught exactly the class of bug where a new code path forgets the constructor.
4. **The reconciler.** A continuous job per Society: sum all balances (must be 0), verify each wallet against its `running_hash` chain, and replay the last checkpoint interval from the event log and compare (LE1). Divergence emits `LedgerDivergenceDetected`, freezes Postings for that Society, and pages. **We freeze rather than continue** — a ledger that keeps accepting writes while known-inconsistent converts a bug into a loss.

Property tests (`11 §7`, items 2–4) generate adversarial histories — concurrent transfers, interleaved locks, settlement during Fracture, crash injection at every step of §4.4 — and assert L1–L7 after every command.

---

## 5. Determinism and Replay

Replay is not a debugging convenience. It is the mechanism by which P6 and P12 are *verifiable* rather than asserted, and it is the mechanism by which the Phase 8 migration (§7) is a swap rather than a rewrite. If the fold is not deterministic, none of it holds.

**What replay must produce:** given the same Log, a clean store, and the same code version, the resulting Posting projection, every wallet balance, and every `state_root` at every frontier are bit-identical.

**Therefore the ledger and every domain path that produces Postings forbids:**

| Forbidden | Why | Enforcement |
|---|---|---|
| Floating point in any monetary path | `f64` addition is not associative; replay order differences produce different totals. Fraction is **`i64` quanta persisted and on the wire, `i128` intermediate**, throughout. | `#![deny]` lint on `f32`/`f64` in `fractal-domain-ledger`; a type-level `Quanta(i64)` newtype with no `From<f64>`, no `From<u128>` and no `as` conversion in the tree; `const _: () = assert!(SUPPLY_CAP_QUANTA <= i64::MAX / 9);` so a cap amendment that eats the 9.22× headroom fails the build rather than production. |
| Unsigned money, or 128-bit money in storage | `u128` cannot represent the mandatory negative `EmissionAccount` balance (§19 LA1, `11 §7.4`), which alone is dispositive; and 128-bit arithmetic is emulated and slow on `wasm32`, which the domain targets from PH0 (N2). Balances are `i64`; `i128` is permitted in expressions and forbidden in storage and on the wire. On the wire `Quanta` is a **JSON string of the decimal quanta count**, never a JSON number — IEEE-754 is exact only to 2^53 ≈ 9.0e15 and the cap is 1e18. gRPC `sint64`; Postgres `bigint`; canonical encoding for `state_root` is 8-byte big-endian two's complement. | `Quanta` is a newtype in `fractal-types`; the serde impl emits a string; a codegen test asserts the OpenAPI schema is `type: string, format: quanta` (`61 X4`). |
| Wall-clock reads (`SystemTime::now`) | The same replay at a different time would produce different output. | `Clock` is a port (`10 §7`). The domain receives `occurred_at` from the event. `recorded_at` is metadata and is **never an input to a decision**. |
| Non-deterministic iteration | `HashMap` ordering varies by seed; Posting order within a batch would vary, changing `running_hash` and `state_root`. | `BTreeMap`/`BTreeSet` only in the domain; a lint denies `std::collections::HashMap` in `fractal-domain-*`. |
| Ambient randomness | Same reason. | `Rng` is a port, seeded from the event. |
| Uncontrolled ID generation | A replayed Posting must keep its identity. | `IdGen` port; Posting IDs are derived deterministically as `Ulid(batch_id, index)`. |
| Reading a projection to make a decision | Projections are rebuildable and may lag; a decision derived from one is not reproducible from the Log alone. | Domain reads only aggregate state and event inputs. Reviewed at the Policy Enforcement Point. |
| Locale-, platform-, or version-dependent serialization | Byte-identical roots require byte-identical encoding. | Canonical deterministic encoding (§6.2), pinned; a change is a new `schema_version` with an upcaster. |
| Unbounded external calls in the posting path | A model provider or HTTP call in the money path is non-replayable by construction. | Adapter-layer only; results enter as events before they can influence a Posting. |

**Upcasting.** Old events are never rewritten (`10 §5`). Replay applies registered upcasters `v1 → v2`. Upcasters are pure functions with their own golden-vector tests: a fixed corpus of historical events must produce a fixed state root forever. That corpus is the regression suite that makes long-horizon replay safe.

---

## 6. Anchoring

### 6.1 What an Anchor is and why

An **Anchor** (`01 §6`) is a periodic cryptographic commitment of a Society's Log state root and Ledger state root. Its purpose is to make history *provably* immutable to a third party, rather than immutable because we say so.

In Phase 1 an Anchor is written internally: to the global Anchor log, cross-signed by the Node, and published. That is weaker than a public chain — an operator with total database control could in principle rewrite both the history and the anchors. We say so plainly. What internal anchoring buys immediately is: tamper *evidence* against partial compromise, a portable proof format, an export any Citizen can independently verify, and — the strategic point — **the exact procedure that later anchors to a chain, exercised in production for years before it matters.** When the target changes in Phase 8, nothing above the adapter changes.

### 6.2 What is committed

```
                     SOCIETY ANCHOR at (society_id, seq = N)
                                    │
                         ┌──────────▼──────────┐
                         │    ANCHOR RECORD    │
                         │  anchor_root = H(   │
                         │    log_root         │
                         │  ‖ ledger_root      │
                         │  ‖ charter_hash     │
                         │  ‖ facet_root       │
                         │  ‖ prev_anchor_root │
                         │  ‖ society_id ‖ N   │
                         │  ‖ occurred_at )    │
                         └──────────┬──────────┘
        ┌────────────────┬──────────┴───────┬──────────────────┐
        ▼                ▼                  ▼                  ▼
 ┌─────────────┐  ┌─────────────┐   ┌─────────────┐   ┌──────────────┐
 │  log_root   │  │ ledger_root │   │ facet_root  │   │charter_hash  │
 │ merkle over │  │ merkle over │   │ merkle over │   │ hash of the  │
 │ event       │  │ (wallet_id, │   │ (facet_id,  │   │ enacted      │
 │ integrity   │  │  balance,   │   │  state_hash,│   │ Charter      │
 │ hashes      │  │  locked,    │   │  owner,     │   │ version      │
 │ 1..N        │  │  run_hash)  │   │  prov_head) │   │              │
 │             │  │ sorted by   │   │ sorted by   │   │              │
 │             │  │ wallet_id   │   │ facet_id    │   │              │
 └─────────────┘  └─────────────┘   └─────────────┘   └──────────────┘
```

- **Hash:** BLAKE3, matching content addressing elsewhere (`11 §6`) — parallelizable over large trees, verified streaming.
- **Tree:** binary Merkle, leaves sorted by the key shown, domain-separated leaf/node prefixes (`0x00`/`0x01`) to prevent second-preimage attacks, duplicate-last-node promotion for odd widths, and an explicit empty-tree constant. Encoding is canonical and pinned; changing it is a versioned event.
- **`prev_anchor_root`** chains anchors, so a forged anchor must forge every subsequent one.
- The Anchor is itself a Domain Event (`AnchorCommitted`) — which means the anchoring history is replayable like everything else.

### 6.3 Schedule

| Trigger | Illustrative Phase 1 value | Rationale |
|---|---|---|
| Every N events | 4096 | Bounds the unanchored window by work, aligning with the ledger checkpoint interval so both roots are cheap to compute. |
| Every T seconds | 600 | Bounds it by time for quiet Societies. |
| Before a sealing operation | mandatory | Fracture (`11 §3.2`), Dissolution, and Seal each anchor *before* the seal. The fracture point must be independently provable. |
| On Charter enactment | mandatory | Governance changes must be pinned to a provable moment. |
| On demand | Level 3+ Societies | A Society may pay a fee (`17`) to anchor immediately — useful before a high-value Facet sale. |

Anchoring is per-Society. There is no global anchor of all Societies in Phase 1; a global aggregation anchor over per-Society roots is a Phase 8 concern that changes only the adapter.

### 6.4 How a third party verifies an Anchor

The verifier needs no access to our infrastructure beyond a published Anchor and a proof bundle a Citizen can export from the CLI (`fn society anchor verify`, `fn facet prove`).

```
  Given: an Anchor record A, a claim C (e.g. "Facet X was owned by
         Citizen Y at seq N", or "this Message exists at seq 4471")

  1. Fetch A from the published Anchor log (or from the chain, Phase 8+).
  2. Verify A's signature chain and that A.prev_anchor_root matches A-1.
  3. Obtain the inclusion proof for C: the sibling hashes from C's leaf
     to the relevant sub-root, plus the three sibling sub-roots.
  4. Recompute: leaf → sub-root → anchor_root.
  5. Assert recomputed anchor_root == A.anchor_root.
  6. For a ledger claim, additionally verify the wallet's running_hash
     chain across the interval — this proves not just the balance but
     the sequence of Postings that produced it.

  Cost: O(log n) hashes. No trust in the Runtime. Offline-verifiable.
```

The verifier is published as a standalone tool with no Fractal Node dependencies. **A verification tool that only we can run is not verification.**

**What an Anchor does NOT prove:** that we showed you *all* events (an omission attack). Inclusion proofs prove presence, not completeness. Completeness is addressed by the append-only `seq` with no gaps, by the per-event `integrity` hash chain (`10 §5`), and — honestly — by the fact that a Citizen's local replica (P2) is an independent copy that would diverge if we omitted. Phase 8 anchoring to a public chain plus multi-party Anchor witnessing narrows this further. It does not vanish. See §17 (T8).

---

## 7. The Migration Path to a Fractal Node L1

**The commitment (P11):** migrating to a Fractal Node L1 is a swap of implementation behind an unchanged trait plus a state-root anchoring procedure. Never a rewrite. Stated as a procedure so it can be audited now, years before it runs.

### 7.1 What changes and what must not

| | Changes | Must NOT change |
|---|---|---|
| **Code** | One adapter crate: `fractal-adapter-ledger-l1` implementing `Ledger` | The `Ledger` trait; every `fractal-domain-*` crate; the application layer; sagas |
| **Contracts** | Nothing | `30-api-and-sdk.md` request/response shapes; CLI verbs; event kinds and payloads |
| **Clients** | Nothing except how a `FinalityClass` is *rendered* | Every client code path |
| **Anchoring** | The target: internal Anchor log → L1 | The Anchor record format, the Merkle construction, the verification procedure |
| **Semantics** | Finality becomes probabilistic-then-final rather than immediate-on-commit | L1–L7; the account tree; `PostingReason` |
| **Operations** | Validator operation, key management, chain monitoring | Per-Society sharding, the event log |

### 7.2 The procedure

```
 STAGE 0  PRECONDITION
   Ledger trait unchanged for ≥ 2 phases. Reconciler green. Replay from
   genesis reproduces every anchor. If replay does not reproduce history
   TODAY, migration is not attemptable — this is why §5 is not optional.

 STAGE 1  ADAPTER
   Implement Ledger for the L1. Same trait, same tests: the ENTIRE ledger
   conformance suite (L1–L7, property tests, adversarial histories) runs
   against both implementations. A trait with one implementation was never
   tested; a trait with two is.

 STAGE 2  SHADOW  (read-only, no user impact)
   Replay the full Posting history into the L1 in a shadow instance.
   Compare state roots at every checkpoint. Any divergence, anywhere,
   in any Society, stops the migration. Duration: ≥ 1 full phase.

 STAGE 3  DUAL-WRITE  (Postgres authoritative, L1 shadow)
   Every post() writes to BOTH. Postgres returns the receipt; L1 writes
   are async and monitored. Divergence detector runs per batch:
       assert postgres.state_root(F) == l1.state_root(F)  ∀ F
   Exit criterion: ZERO divergences for 30 consecutive days at production
   volume, INCLUDING at least one Fracture, one Dissolution, and one
   settlement run. Not a sample — all of them.

 STAGE 4  DUAL-WRITE, INVERTED  (L1 authoritative, Postgres shadow)
   Same writes, authority flipped. Reads served from L1. Postgres continues
   as a live shadow and remains a one-config-flag rollback for ≥ 90 days.
   Anchor target switches to the L1 here — the same anchor_root, a new
   destination.

 STAGE 5  CUTOVER
   Postgres ledger demoted to archive. Adapter selection is a config value
   (FRACTAL_LEDGER_BACKEND). The trait never changed. No domain code was
   touched at any stage.
```

### 7.3 Finality, honestly

The one semantic that genuinely changes. Phase 1 finality is immediate on transaction commit. An L1's is not. We surface this without leaking chain concepts into the domain:

```rust
pub enum FinalityClass {
    Committed,   // durably recorded by the ledger; internal Phase 1 stops here
    Anchored,    // included in a committed Anchor; third-party verifiable
    Settled,     // irreversible under the ledger's own finality rule
}
```

Clients already render `PENDING` → settled today (`10 §6`), because we built for this before it existed. The receipt gains a `FinalityClass`; no client acquires a concept of confirmation depth. If an implementation has reorgs, the adapter absorbs them below `Committed` and never reports a `Committed` frontier it might retract. **An adapter that cannot make that promise is not an acceptable `Ledger` implementation** — that is the acceptance criterion, stated now.

### 7.4 Rollback

Rollback is a config flag at Stages 3–4 because Postgres remains a live, verified shadow. After Stage 5, rollback is a Stage 2–4 run in reverse, and the honest statement is that **it is not a fast operation** — days, not minutes. That asymmetry is why Stage 4 lasts 90 days and why Stage 3's exit criterion is zero divergences rather than a threshold. The cheap moment to discover a problem is before authority moves.

---

## 8. The `Chain` Port

```rust
#[async_trait]
pub trait Chain: Send + Sync + 'static {
    fn chain_id(&self) -> ChainId;
    fn capabilities(&self) -> ChainCapabilities;   // anchoring? bindings? transfer?

    /// Publish an Anchor. The ONLY method Phase 1–7 uses.
    async fn commit_anchor(&self, anchor: AnchorRecord) -> Result<ChainRef, ChainError>;

    /// Verify a previously published Anchor from chain state.
    async fn verify_anchor(&self, r: ChainRef) -> Result<AnchorProof, ChainError>;

    /// Project a Facet as an external representation (§15). Lossy by definition.
    async fn bind_facet(&self, b: FacetBindingRequest) -> Result<ChainBinding, ChainError>;

    /// Observe external state relevant to a binding. Read-only.
    async fn observe(&self, q: ChainQuery) -> Result<Observation, ChainError>;
}
```

Phase 1 ships `NullChain`, which anchors to the internal Anchor log and reports `capabilities = { anchoring: true, bindings: false, transfer: false }`. This is the second implementation P5 requires at boundary creation, and it is the one that runs in production.

**Bridging (Phase 8+), stated honestly.** Bridges are the single most-exploited component in this industry. The largest losses in the history of the space have been bridge compromises, not consensus failures. Any bridge is a trusted multi-party custody system wearing the costume of a protocol, and its security is the security of its weakest signer.

Therefore, if bridging is ever enabled:

- **Never for Fraction before Phase 9**, and never before the `17` conditions on external exchangeability are met. A bridge is an external-liquidity mechanism, and premature external liquidity is on the Not-Yet list (`02 §3`) for economic reasons independent of security.
- **Facet bindings are one-way projections by default** (§15): the canonical record stays with us; the external artifact is a mirror. A mirror compromise does not move the canonical asset.
- **Any two-way bridge requires:** a dedicated ADR overturning nothing but adding this capability; an independent audit; per-window value caps; a mandatory delay on outbound value above a threshold; an emergency pause any Guardian may trigger and only a human may release (P4); and a published, honest statement of the trust assumption in the UI at the moment of use — not in a footnote.
- **The residual risk is not eliminated by any of that.** If bridging exists, bridge exploit is a live risk, and the correct mitigation for the value we cannot afford to lose is not to bridge it.

---

## 9. The `Rail` Port

A **Rail** is a payment/settlement adapter — a way value enters or leaves the system.

```rust
#[async_trait]
pub trait Rail: Send + Sync + 'static {
    fn rail_id(&self) -> RailId;
    fn direction(&self) -> RailDirection;          // In | Out | Both
    fn constraints(&self) -> RailConstraints;      // limits, jurisdictions, KYC tier

    async fn quote(&self, r: RailQuoteRequest) -> Result<RailQuote, RailError>;
    async fn initiate(&self, r: RailTransferRequest) -> Result<RailHandle, RailError>;
    async fn poll(&self, h: RailHandle) -> Result<RailStatus, RailError>;
    async fn reverse(&self, h: RailHandle, reason: ReversalReason)
        -> Result<RailHandle, RailError>;          // chargebacks are real
}
```

Phase 1 ships exactly one Rail: `InternalRail`, which moves Fraction between Wallets and is a thin wrapper over `Ledger`. That is not a placeholder — it is the Rail that all internal economy uses.

**Fiat is Phase 9+ and gated on legal counsel** (`02 §3`), not on engineering. It requires a licensed entity, KYC/AML, sanctions screening, chargeback handling, tax reporting, and jurisdiction-by-jurisdiction analysis. None of that is code we can write our way around.

**Keeping the abstraction honest in the meantime.** An abstraction with one implementation is a guess. Three disciplines keep this one from rotting:

1. `reverse` exists in the trait **now**, because fiat rails have chargebacks and a design that discovers reversibility in Phase 9 will discover it as a rewrite. The internal Rail implements it as a `Correction` Posting (§2.2 L7), which is the correct semantics anyway.
2. `quote` exists **now** because every external rail has fees, spreads, and expiry. Internal quotes are exact and instantly expiring; the shape is exercised from day one.
3. A `MockFiatRail` — settlement delay, random failures, async reversals, KYC rejections — runs in the test suite from Phase 1. It is the second implementation P5 requires, and it is what proves the sagas in `11 §5` tolerate a rail that is slow, fallible, and reversible.

We do not build fiat. We build so that fiat is an adapter.

---

# PART B — THE FACET STANDARD (FN-ASSET/1)

## 10. Design Thesis: Why Facets Evolve

### 10.1 The problem with static token standards

ERC-721 and ERC-1155 encode one idea well: *a scarce, transferable pointer to an identifier, whose meaning lives somewhere else.* Everything interesting about the asset — what it looks like, what it means, what it does — sits behind a URI the standard does not govern. The consequences are structural, not incidental:

| Static-standard property | Consequence |
|---|---|
| State is an ID plus an owner | Anything richer lives off-standard, so every project reinvents it incompatibly |
| Metadata is a URI | The asset is only as durable as someone's hosting bill. "Ownership" of a dead link is a common outcome |
| Mutation is not modelled | Changing an asset means a custom contract nobody else understands, or minting a replacement and abandoning the original's provenance |
| Ownership is the entire relationship | Licensing, custody, delegation, and use-rights are all bolted on |
| Royalties are a hint | An honest standard would say enforcement is a marketplace courtesy. Most do not |
| Provenance is transfer history | *Who owned it* is recorded; *what happened to it* is not |
| Composition is per-project | "This contains that" has no standard meaning |

The deepest problem: **these standards model a certificate, and the interesting digital objects in a social platform are not certificates — they are things that accumulate history.** An Insignia earned across three years of a Society. An instrument that records who played it. A creator's work that gains a movement each year. Modelling those as an immutable pointer plus an off-standard side-channel is modelling them wrong, and every workaround makes the asset less portable, not more.

### 10.2 The thesis

**A Facet is a stateful object with declared evolution rules, whose entire history is provenance.** Static assets are the degenerate case (`evolution: Immutable`), not the default (`11 §2.9`).

We can make this choice because we own the ledger and the runtime that enforces it. A mutation need not be an externally priced transaction; it is a Domain Event in a Society's Log — replayable, anchored, provable. That is strictly more expressive and strictly cheaper, at the honest cost of a trust assumption (§15) we make explicit rather than hide.

We are also not competing with those standards. A Facet may *project* onto one (§15). It is never *defined* by one.

---

## 11. FN-ASSET/1 — Specification Sketch

```rust
pub struct Facet {
    // ── IDENTITY (immutable for life) ───────────────────────────────
    pub facet_id:    FacetId,       // fct_<ULID>_<society prefix>  (11 §6)
    pub standard:    StandardId,    // "FN-ASSET/1"
    pub society_id:  SocietyId,     // minting Society. NEVER changes. (P1)
    pub schema:      SchemaRef,     // (schema_id, version) — global registry
    pub creator:     Fnid,          // NEVER changes, even on transfer
    pub minted_at:   Timestamp,
    pub genesis:     Hash,          // H(identity ‖ schema ‖ initial_state ‖ rules)

    // ── STATE (mutable under rules — the point of the standard) ─────
    pub state:       FacetState,    // schema-validated, canonically encoded
    pub state_hash:  Hash,
    pub revision:    u32,           // monotonic; increments on every Evolved
    pub lifecycle:   Lifecycle,     // Minting|Active|Evolving|Locked|Transferring|Retired

    // ── RULES (mutable only under their own amendment clause) ───────
    pub evolution:   EvolutionRules,
    pub composition: CompositionRules,
    pub binding_mode: BindingMode,  // Free | SocietyBound | PrincipalBound

    // ── OWNERSHIP AND CUSTODY (distinct concepts) ───────────────────
    pub owner:       Principal,     // who holds the property right
    pub custodian:   Option<Custody>, // who may act, without owning
    pub transfer:    TransferPolicy,

    // ── COMMERCE ────────────────────────────────────────────────────
    pub license:     LicenseSet,    // what holders may DO, orthogonal to ownership
    pub royalty:     RoyaltyTerms,

    // ── HISTORY (append-only, never rewritten) ──────────────────────
    pub provenance:  ProvenanceChain,

    // ── EXTERNAL (optional, lossy, non-authoritative) ───────────────
    pub bindings:    Vec<ChainBinding>,
}
```

### 11.1 Identity

`facet_id` embeds its minting Society, so origin is legible without a lookup, satisfying P1's "which Society owns you?" test at the identifier level. `creator` never changes — it is an authorship fact, not a property right, and conflating the two is how creator attribution gets laundered on resale. `genesis` is the hash a counterfeit cannot reproduce (§17, T6).

### 11.2 Schema

Facet Standards are one of the nine Global Registry entries (`01 §6`) — *schemas are global, instances are society-scoped*. A schema declares typed fields, constraints, defaults, and which fields are `evolvable`, `owner_scoped`, or `frozen`.

```rust
pub struct FacetSchema {
    pub schema_id:  SchemaId,
    pub version:    u16,
    pub fields:     BTreeMap<FieldName, FieldSpec>,   // BTree: deterministic (§5)
    pub required:   BTreeSet<FieldName>,
    pub renderers:  Vec<RendererHint>,                // how clients display it
    pub media:      Vec<MediaSlot>,                   // Vault ObjectIds, not URLs
}

pub struct FieldSpec {
    pub ty:         FieldType,        // U64|I64|Text|Bool|Timestamp|Enum|Ref|MediaSlot|List|Map
    pub mutability: Mutability,       // Frozen | Evolvable | OwnerScoped
    pub constraint: Option<Constraint>,   // range, length, regex, enum membership
}
```

Schema evolution is additive within a version (new optional fields); a breaking change is a new version plus a registered migration, exactly as with events (`10 §5`). Instances pin their version and migrate explicitly. **Media lives in the Vault as content-addressed Objects (`11 §2.7`), never as an external URL.** This is the single largest durability difference from a URI-based standard: a Facet's media is content-addressed, erasure-coded, replicated, and reconstructible from its Manifest. It cannot 404.

### 11.3 Evolution rules

```rust
pub struct EvolutionRules {
    pub mode:        EvolutionMode,
    pub triggers:    Vec<EvolutionTrigger>,
    pub on_transfer: TransferBehavior,
    pub amendment:   AmendmentClause,   // how these rules may themselves change
    pub max_revision: Option<u32>,      // an evolution budget, if declared
}

pub enum EvolutionMode {
    Immutable,                          // the degenerate case: a static asset
    Deterministic,                      // pure function of ledger/log facts
    Attested { attesters: Vec<Fnid>, threshold: u8 },   // signed external input
    Governed { charter_rule: RuleRef }, // Society governance decides
    Hybrid   { ... },
}

pub enum EvolutionTrigger {
    Threshold  { metric: MetricRef, op: CmpOp, value: i128 },  // XP, tenure, Standing
    Event      { kind: EventKind, filter: Filter },            // a Domain Event
    Schedule   { every: Duration, until: Option<Timestamp> },  // domain time only
    Owner      { action: ActionName, cost: Option<Quanta> },   // owner-invoked
    Attestation{ oracle: OracleRef, claim: ClaimSpec },        // external fact
    Composition{ on: CompositionEvent },                       // contained Facet changed
}

pub struct EvolutionRule {
    pub trigger:   EvolutionTrigger,
    pub guard:     Guard,        // pure predicate over state + trigger payload
    pub effect:    Effect,       // declarative field mutations, no arbitrary code
    pub authority: Authority,    // Creator | Owner | Society | System | Attester
    pub cost:      Option<Quanta>,
}
```

**`effect` is declarative, not code.** Set, increment (bounded), append to a list (bounded), unlock a media slot, transition an enum. No Turing-complete evolution in FN-ASSET/1. Arbitrary logic is a Phase 7 Experience Runtime concern with a sandbox and a metering story (`20`); putting it in the asset standard would make every Facet a program with an unbounded audit surface, and would make replay dependent on an execution engine we do not yet have. The cost is that some evolutions are inexpressible; we accept that in exchange for a standard that is verifiable by inspection.

**Deterministic vs oracle-driven.**

| | Deterministic | Attested (oracle) |
|---|---|---|
| Input | Facts already in the Log or Ledger | A signed claim from a declared attester |
| Replayable | Yes, from the Log alone | Yes — because the **attestation is recorded as an event first**, then evaluated |
| Trust | None beyond the Runtime | The attester set, explicitly named in the Facet |
| Failure mode | Bug | Attester compromise or lie |
| Disclosure | — | Attesters and threshold are visible in every client, always |

The critical rule: **an oracle never mutates state directly.** It emits `AttestationRecorded`; the rule engine folds that event deterministically. This keeps L4/§5 intact — replay never calls out to the world, because the world's input is already in the log.

**Who may trigger.** `authority` is enforced at the Policy Enforcement Point in the application layer (`10 §8`), not in the asset. An Agent evolving a Facet carries `envelope_ref` (P4), and irreversible evolutions (`max_revision` exhaustion, media-slot burn, Retirement) default into `confirm_classes` — a human confirms.

**Evolution history is provenance.** Every `FacetEvolved` event appends a `ProvenanceEntry`. There is no compaction, no pruning, and no "current state only" mode. The state is a fold; the history is the asset.

### 11.4 The transfer question: does a Facet reset when sold?

**Decision: No. A Facet never resets on transfer. Its default is to carry its full state and provenance to the new owner. Fields explicitly marked `OwnerScoped` are *sealed and archived* — not erased — and a fresh per-owner accumulator begins.**

```rust
pub enum TransferBehavior {
    Carry,                  // DEFAULT — all state persists
    SealOwnerScoped,        // OwnerScoped fields snapshot into provenance, reset to default
    Freeze,                 // the Facet becomes Immutable on first transfer
    Forbid,                 // non-transferable (Insignia, §12.3)
}
```

Justification:

1. **Resetting destroys the thing that makes a Facet worth more than a certificate.** A twelve-year Insignia whose history vanishes on sale is a JPEG with extra steps.
2. **Provenance is append-only by invariant** (`11 §2.9`). A reset that deleted history would violate it. Sealing does not: the previous owner's tally becomes a permanent, attributed entry.
3. **It is honest about what was earned by whom.** "Held by @a for 400 days, who reached tier 3" is a *better* record than a wiped counter, for both buyer and seller.
4. **Where a reset is genuinely correct — a per-owner streak, a use quota, a personal-use license grant — `SealOwnerScoped` expresses it precisely**, at field granularity, declared at mint time and visible before purchase.

The honest cost: a buyer inherits history they did not create, and some Facets will carry a previous owner's reputation with them. We consider that a feature (it is what provenance means in every other market for durable goods) but it is a real constraint on creators, and it is why `on_transfer` is declared at mint and immutable thereafter under most amendment clauses.

---

## 12. Composability

### 12.1 Facets containing Facets

```
  ┌────────────────────────────────────┐
  │  Facet A  (container)              │   Rules:
  │  composition: Container{max: 8}    │   • A contained Facet's OWNER must
  │  ┌──────────┐  ┌──────────┐        │     equal the container's owner at
  │  │ Facet B  │  │ Facet C  │        │     insertion, and follows it on
  │  │ (Locked) │  │ (Locked) │        │     transfer (atomically, one batch)
  │  └──────────┘  └──────────┘        │   • Contained Facets are Locked:
  │                                     │     not independently transferable
  │  DEPTH LIMIT 4 · NO CYCLES          │   • Removal requires the container's
  │  (checked on insert, both ways)     │     TransferPolicy to permit it
  └────────────────────────────────────┘   • Composition changes emit events
                                             on BOTH Facets' provenance
```

Depth is capped at 4 and cycles are rejected on insert (a Facet may not contain an ancestor). Unbounded nesting makes ownership resolution, state-root computation, and transfer atomicity unbounded — all three are invariants we refuse to make O(unknown). A container's `facet_root` leaf includes its children's `state_hash`es, so a contained Facet cannot be swapped without changing the container's anchor.

### 12.2 Facets bound to Societies

`BindingMode::SocietyBound` means the Facet cannot leave its minting Society. Society artifacts, Charter seals, and Chamber decorations are society-bound. On Fracture (`11 §3.2`) society-bound Facets follow the split specification; on Dissolution they are released to their owners rather than destroyed — history is not ours to erase.

### 12.3 Facets bound to Citizens — Insignia

`BindingMode::PrincipalBound` with `TransferBehavior::Forbid` is the soulbound-like case: an **Insignia** (`01 §7`), a Facet subtype earned through progression. It is non-transferable, non-purchasable, and its evolution triggers may read XP, Level, Trust, Standing, and Achievements.

Three hard rules, all enforcing existing Canon:

1. **An Insignia may never be sold, and no mechanism may convert Fraction into one.** Pay-to-win is on the Never list (`02 §4`).
2. **An Insignia may be *retired* by its holder but never revoked by a Society**, except as a recorded, appealable Moderation Action under a Charter (`01 §7`). Silent removal of earned recognition is not available to anyone.
3. **An Insignia's evolution may read Trust but may never write it.** Trust is written only by processes whose inputs exclude XP and Fraction (`11 §7.8`).

---

## 13. Licensing and Creator Monetization

**Ownership and licence are orthogonal.** Owning a Facet is holding the property right in the instance. A licence is what a *holder* may DO with the work. Conflating them is the ambiguity that made "what did I actually buy?" the defining question of the last asset cycle.

```rust
pub struct License {
    pub kind:       LicenseKind,
    pub grantee:    Principal,
    pub scope:      LicenseScope,      // fields/media slots covered
    pub territory:  Territory,         // InPlatform | Global
    pub term:       Term,              // Perpetual | Until(ts) | Renewable{period}
    pub sublicense: bool,
    pub revocation: RevocationTerms,   // ONLY for breach, defined at grant
    pub consideration: Option<Quanta>,
}

pub enum LicenseKind {
    Personal,           // display and personal use; no commercial exploitation
    Commercial,         // commercial exploitation of the work as-is
    DerivativeAllowed { attribution: bool, share_alike: bool },
    TimeBoxed { until: Timestamp },     // e.g. an exhibition or event licence
    Exclusive { field: ExclusivityField }, // creator may not grant overlapping
}
```

A licence grant is a Domain Event (`LicenseGranted`) with its own provenance entry. Licences may be granted to non-owners: a Society may licence display rights to a work it does not own; a creator may retain ownership and sell only commercial rights.

**Royalty enforcement — what is and is not enforceable.**

| Context | Enforceable? | Mechanism / honest limit |
|---|---|---|
| Transfer inside Fractal Node | **Yes, fully** | The Ledger is the settlement layer and the Facet registry is the ownership record. A transfer that does not include the royalty Posting **does not commit** — ownership does not change. This is not a marketplace policy; it is a ledger invariant. |
| Transfer via a Fractal Node marketplace | Yes | Same mechanism; the marketplace is a caller, not an authority (`10 §3`, S12). |
| An off-platform payment plus an on-platform transfer at zero price | **Partially** | We cannot see the side payment. Mitigations: the transfer is still recorded with a stated price of zero, permanently and publicly, in provenance; `RoyaltyTerms` may set a floor or require an appraisal attestation for gift transfers; wash patterns are detectable. We do not claim to prevent it. |
| A projection on an external chain (§15) | **No** | Once an artifact exists on a chain we do not control, that chain's marketplaces enforce whatever they choose — which is, empirically, nothing. This is a decisive reason the canonical record stays with us. |
| Off-platform use of the underlying work | **No** | That is copyright law, not software. The platform provides evidence — timestamped, anchored provenance and licence records that are strong evidence of authorship and terms — and nothing more. Anyone claiming a technical system enforces copyright is selling something. |

**Revenue settlement.** Every commercial event compiles to one atomic `PostingBatch` (L1). Splits are computed by the domain from `RoyaltyTerms` and the Charter's `economy` parameters; the *rates* live in `17`.

```
  Sale of Facet F by @seller to @buyer for P
  ─────────────────────────────────────────────────────────────
  1. lock(buyer, P)                              → LockId
  2. ONE PostingBatch, atomic with the ownership change:
        buyer.escrow  → seller       P - royalty - fee
        buyer.escrow  → creator      royalty        (RoyaltyPaid)
        buyer.escrow  → society.fee  fee            (MarketFee)
        (fee is later split Treasury/platform per 17 — separate batch)
  3. FacetTransferred { F, seller → buyer, price P, batch_id }
     appended in the SAME transaction as the Postings.
  ─────────────────────────────────────────────────────────────
  There is no state in which the money moved and the Facet did not,
  or vice versa. That is the whole argument for owning the ledger.
```

---

## 14. Transfer Semantics

### 14.1 Atomic swap with Fraction

Facet ownership and Fraction movement are the same transaction, in the same Society log. No two-phase protocol, no counterparty risk, no settlement window — a direct consequence of §3 (the Ledger *is* the log).

### 14.2 The `Locked` state

`Lifecycle::Locked` (`11 §4`) means: owner unchanged, transfer forbidden, evolution permitted or forbidden per rule. A Facet is Locked while escrowed, staked as a bond, contained in another Facet (§12.1), or under an active Moderation hold. **Every lock has a holder, a reason, and a deadline.** A lock without a deadline is a bug: the reconciler flags any lock older than its declared term and emits `FacetLockExpired`, auto-releasing to the owner. Locks that can be forgotten become assets that can be stolen by inaction.

### 14.3 Escrow and conditional transfer

```
  SELLER                    ESCROW (per-society)                   BUYER
    │                             │                                  │
    │ list(F, price, expiry)      │                                  │
    ├────────────────────────────►│  F → Locked{reason: Listed}      │
    │                             │                                  │
    │                             │◄─────────────────────────────────┤ accept
    │                             │  lock(buyer, P) → EscrowAccount  │
    │                             │                                  │
    │                     ┌───────▼────────┐                         │
    │                     │  CONDITIONS?   │  all must hold:         │
    │                     │  • licence ack │  • not expired          │
    │                     │  • Level gate  │  • buyer solvent        │
    │                     │  • attestation │  • F still Locked{Listed}│
    │                     └───────┬────────┘                         │
    │        ┌────────────────────┴─────────────────────┐            │
    │        ▼ ALL HOLD                                 ▼ ANY FAILS  │
    │   settle(lock, batch):                      release(lock):     │
    │   Postings + FacetTransferred               funds returned,    │
    │   ONE transaction                           F → Active,        │
    │◄──────────────────────────────────────────  nothing partial ───┤
```

Conditional transfer is the general form: a transfer whose commit is guarded by a set of declarative conditions evaluated at settlement. Deadlines are mandatory. Every escrow either settles or releases; **there is no third outcome**, and the reconciler proves that the per-Society `EscrowAccount` balance equals the sum of open locks at every checkpoint.

---

## 15. Interoperability and External Projection

A Facet may be projected onto an external chain (Phase 8+) as a `ChainBinding`. The projection is a **mirror, not a migration.**

```
   ┌──────────────────────────────────────┐        ┌────────────────────────┐
   │  FRACTAL NODE — CANONICAL RECORD     │        │  EXTERNAL CHAIN        │
   │                                      │        │                        │
   │  full state · evolution rules        │  proj. │  facet_id              │
   │  full provenance · licences          │ ─────► │  state_hash (at rev R) │
   │  composition · custody · royalties   │        │  owner (mapped addr)   │
   │  media in the Vault                  │        │  anchor reference      │
   │                                      │        │  media pointer         │
   │  ANCHORED (§6) — third-party          │        │                        │
   │  verifiable without trusting us       │        │  LOSSY. FROZEN AT R.   │
   └──────────────────────────────────────┘        └────────────────────────┘
                      ▲                                        │
                      └───── external transfer observed ───────┘
                            (Phase 8+; recorded as an event,
                             never as an authoritative mutation)
```

**What is lost in projection:** evolution rules, full provenance (only a root fits), licence terms and their enforceability, composition, the custody-vs-ownership distinction, `OwnerScoped` semantics, and — most importantly — *future state*. A projection is a snapshot at revision R with a hash. The living object stays here.

**Why the canonical record stays with us.** Because the properties that make a Facet worth having — evolution, provenance depth, enforceable royalties, durable Vault media, licence semantics — are precisely the properties that do not survive the crossing. Exporting the canonical record to gain "portability" would trade every distinguishing feature for a wider list of marketplaces that do not enforce our terms.

**The honest counter-argument, stated plainly.** A canonical record we control is a canonical record we could corrupt. That is a real trust assumption and it is the strongest argument for the chain-native approach. Our answers are: Anchors are independently verifiable by a tool with no dependency on us (§6.4); Citizens hold local replicas (P2) that would diverge visibly; the Log is append-only with a hash chain; and Phase 8 anchoring to a public chain reduces the assumption further. It does not reduce it to zero, and we do not claim it does.

**Conflict resolution when a binding exists:** if an external transfer occurs on the mirror, we record `BindingDiverged` and mark the binding `Stale`. **The canonical owner does not change.** A binding is a projection; a projection cannot mutate its source. Two-way binding requires the full §8 bridge conditions and an ADR.

---

## 16. Worked Example — "Cantos", an Evolving Serialized Work

A creator, `@ilse`, mints a longform musical work inside the Society `soc_ARCHIVE`. It ships with one movement and gains a movement each time she completes one, up to seven. It is sold once; it keeps evolving afterwards; the buyer's listening tally is theirs and is sealed if they resell. This exercises schema, evolution, licence, royalty, transfer, escrow, and provenance end to end.

### 16.1 Schema (registered globally, `01 §6` entry 7)

```rust
FacetSchema {
    schema_id: "fn.media.serialized_work", version: 1,
    fields: {
      "title":            { ty: Text,            mutability: Frozen },
      "movements_total":  { ty: U64,             mutability: Frozen,
                            constraint: Range(1..=7) },
      "movements_out":    { ty: U64,             mutability: Evolvable,
                            constraint: Range(1..=7) },
      "movement_media":   { ty: List<MediaSlot>, mutability: Evolvable,
                            constraint: MaxLen(7) },   // Vault ObjectIds
      "cover":            { ty: MediaSlot,       mutability: Frozen },
      "listen_count":     { ty: U64,             mutability: OwnerScoped },
      "first_heard_at":   { ty: Timestamp,       mutability: OwnerScoped },
      "completed":        { ty: Bool,            mutability: Evolvable },
    },
    required: { "title", "movements_total", "cover" },
    renderers: [ Gallery, ChamberInline, ProfileModule ],
}
```

### 16.2 Mint

```
FacetMinted {
  facet_id:  fct_01J9…_soc_ARCHIVE      society_id: soc_ARCHIVE
  creator:   fn1ilse…                    owner: fn1ilse…      revision: 0
  schema:    ("fn.media.serialized_work", 1)
  state:     { title: "Cantos", movements_total: 7, movements_out: 1,
               movement_media: [obj_…a1], cover: obj_…c0,
               listen_count: 0, completed: false }
  genesis:   H(identity ‖ schema ‖ initial_state ‖ rules)
  binding_mode: Free       transfer: TransferPolicy::Open
  on_transfer:  SealOwnerScoped
}
```

The mint fee is a `PostingBatch` in the same transaction (`11 §5`, `MintFacet` saga). Movement audio is uploaded to the Society's Vault first; the schema references `ObjectId`s, so the media is erasure-coded and Custodian-attested (§11.2, T12) rather than a link.

### 16.3 Evolution rules

| # | Trigger | Guard | Effect | Authority | Cost |
|---|---|---|---|---|---|
| E1 | `Owner{action: "publish_movement"}` | `movements_out < movements_total` **and** actor `== creator` | `movements_out += 1`; append `ObjectId` to `movement_media` | `Creator` | mint-tier fee |
| E2 | `Threshold{metric: state.movements_out, op: Eq, value: 7}` | — | `completed = true` | `System` (deterministic) | 0 |
| E3 | `Event{kind: "media.playback.completed", filter: facet == self}` | actor `== owner` | `listen_count += 1`; set `first_heard_at` if unset | `System` | 0 |
| E4 | `Attestation{oracle: soc_ARCHIVE curators, claim: "archival_master_deposited"}` | 2-of-3 threshold | unlock media slot `master` | `Attester` | 0 |

E1 is the creator's authority and survives sale — **the work keeps growing for whoever holds it**, which is the entire premise. E2 is deterministic: it is a pure function of state and needs no actor. E3 is `OwnerScoped`, so it is the buyer's tally, not the seller's. E4 is the oracle case: the curators sign an attestation, `AttestationRecorded` lands in the Log, and only then does the rule engine fold it (§11.3) — replay never calls out to the world.

Note what E1 implies and why it is declared at mint: the creator retains a *permanent capability over an object she no longer owns*. That is unusual, it is visible in the Facet before purchase, and `AmendmentClause` for these rules is `CreatorAndOwner` — neither party can change the deal alone.

### 16.4 Licence and royalty

```rust
LicenseSet {
  default_holder_license: License {
    kind: DerivativeAllowed { attribution: true, share_alike: false },
    scope: Fields(["movement_media"]), territory: InPlatform,
    term: Perpetual, sublicense: false,
    revocation: OnBreachOnly,
  },
  grantable_by_creator: [ Commercial, TimeBoxed, Exclusive{ field: Sync } ],
}
RoyaltyTerms { creator_bps: 750, floor: Some(50 FRC), on_zero_price: RequireAttestation }
```

The holder may make attributed derivatives inside the platform; commercial exploitation is a separate grant `@ilse` sells independently of ownership (§13). 7.5% of every priced transfer goes to the creator as a Posting inside the transfer batch — not a marketplace policy, a ledger invariant. A zero-price transfer is permitted but requires a gift attestation and is recorded as such forever (T5).

### 16.5 Lifecycle

```
 t0  FacetMinted            revision 0   owner @ilse       movements_out 1
 t1  FacetEvolved  (E1)     revision 1   movements_out 2      + provenance
 t2  FacetListed            Locked{Listed, expires t2+7d}    price 400 FRC
 t3  FacetLockAcquired      buyer @rowan  lock(400) → EscrowAccount
 t4  ── ONE TRANSACTION ─────────────────────────────────────────────────
     PostingRecorded  escrow → @ilse        340 FRC   FacetSalePrincipal
                      escrow → @ilse         30 FRC   RoyaltyPaid   (7.5%)
                      escrow → soc.fee       30 FRC   MarketFee
     FacetTransferred @ilse → @rowan  price 400  behavior SealOwnerScoped
       provenance += { sealed: listen_count 61, first_heard_at t0+2d,
                       held_by @ilse t0..t4 }
       state.listen_count = 0   state.first_heard_at = null
     ────────────────────────────────────────────────────────────────────
 t5  FacetEvolved  (E3)     revision 3   @rowan's listen_count 1
 t6  FacetEvolved  (E1)     revision 4   movements_out 3   — by @ilse, post-sale
 t7  AttestationRecorded    curators 2-of-3
 t8  FacetEvolved  (E4)     revision 5   master slot unlocked
 t9  AnchorCommitted        facet_root includes state_hash at revision 5
```

At `t9` any third party can prove, without trusting us, that `@rowan` owned the Facet at revision 5, that `@ilse` created it, that 30 FRC of royalty was paid at `t4`, and that `@ilse` held it for the first 61 listens — from an inclusion proof and the published Anchor (§6.4).

**Contrast, an Insignia.** The same machinery with `binding_mode: PrincipalBound`, `transfer: Forbid`, no licence, no royalty, and evolution triggers reading Standing and tenure: tiers at 180/540/1095 days of Membership plus a governance-participation threshold. No sale path exists, no Fraction can produce it (`02 §4`), and §11.4's transfer question never arises — which is exactly why non-transferability is a declared mode of one standard rather than a second standard.

---

## 17. Threat Table

| # | Attack | Vector | Mitigation | Residual risk |
|---|---|---|---|---|
| T1 | **Double-spend** | Concurrent Transfers spending the same balance | Ordered `FOR UPDATE` wallet locks (§4.4); sufficiency checked inside the transaction; no optimistic writes anywhere in the money path | Near zero within one ledger. Reappears at bridges (T7), which is a reason not to have them |
| T2 | **Replay of a command or Posting** | Retried request, captured API call, malicious resubmission | `batch_id` idempotency (L3); `(principal, idempotency_key)` dedupe (`10 §5`); per-wallet `nonce`; signed commands | A replay outside the 24h dedupe window relies on `batch_id` uniqueness, which is enforced by a unique index — a client reusing a `batch_id` for different content gets the original receipt, which is correct but surprising. Documented. |
| T3 | **Ledger corruption** (bug, operator, storage fault) | Direct DB write, faulty migration, silent disk corruption | Event log is the authority (§3); per-wallet `running_hash` chains; checkpoints; continuous reconciler that **freezes Postings on divergence** (§4.5); Anchors make post-hoc rewriting detectable | A sufficiently privileged operator who rewrites the log, the anchors, and every replica before anyone verifies. Narrowed by Citizen replicas and Phase 8 external anchoring. Not eliminated in Phase 1 — stated in §6.1 |
| T4 | **Evolution abuse** — farming state | Automating a trigger; Sybil accounts feeding a Threshold rule | Rate limits per trigger; `max_revision` budgets; `Attested` mode for anything valuable; triggers read Sybil-resistant Contribution Score, never raw volume (P12); evolution costs are Postings (`17`) | A well-resourced attacker still out-farms an honest Citizen on volume-based triggers. Mitigation is a schema-design rule: **do not build valuable triggers on volume**. Reviewed at Standard registration |
| T5 | **Royalty evasion** | Off-platform payment + zero-price on-platform transfer | Royalty is a ledger invariant for priced transfers (§13); zero-price transfers are permanently recorded as such; floors and appraisal attestations available; wash patterns detectable | Real and unpreventable for side payments. Stated honestly rather than mitigated by claim |
| T6 | **Counterfeit Facet** | Cloned schema, imitated media, lookalike Society | `genesis` hash is unforgeable; `creator` FNID is immutable; `facet_id` embeds the minting Society; clients display creator FNID + Society + anchor status, never the name alone; Standards are globally registered | Social confusion persists — users who do not check provenance can be deceived by a visually identical Facet. This is a UI problem (`33`) as much as a protocol one |
| T7 | **Bridge exploit** | Compromise of bridge signers or contract (Phase 8+) | No bridge before Phase 8; Facet bindings one-way by default; value caps, delays, emergency pause (§8); independent audit required | **High, and irreducible if a two-way bridge exists.** The only complete mitigation is not bridging value we cannot afford to lose. Historically the most-exploited component in the industry |
| T8 | **Anchor forgery / omission** | Fabricated Anchor, or omitting events from proofs | `prev_anchor_root` chaining (forging one requires forging all subsequent); Node cross-signature; published Anchor log; independent verifier; gapless `seq` plus per-event integrity chain | Omission is not fully solved by inclusion proofs. Narrowed by local replicas (P2), gapless sequence, and Phase 8 multi-party witnessing. Named as a live limitation |
| T9 | **Unauthorized evolution or transfer by an Agent** | An Agent exceeding intent | Policy Enforcement Point in the application layer (`10 §8`); every action carries `envelope_ref`; irreversible classes require live human confirmation (P4); Envelopes expire | An Operator who grants an over-broad Envelope. Mitigated by defaults and UI, not by protocol |
| T10 | **Lock/escrow griefing** | Listing to lock a Facet indefinitely; abandoned escrow | Mandatory deadlines on every lock and escrow; auto-release on expiry; listing fees; reconciler asserts `EscrowAccount == Σ open locks` | Short-term denial of transfer within the deadline window. Bounded by policy, not eliminated |
| T11 | **Oracle/attester compromise** | A named attester signs a false claim | Threshold signatures; attester set is public in every client; attestations recorded as events before evaluation, so a false one is permanently attributable; Society may revoke an attester going forward | Past evolutions driven by a compromised attester stand. Reversal is a governed, recorded correction — never a silent rewrite |
| T12 | **Media rot / hostage** | Losing the referenced work | Media lives in the Vault as content-addressed, erasure-coded Objects with attested Custodians (`11 §2.7`), not as URLs | Custody economics must hold long term. If replication funding fails, media availability degrades — a `13`/`17` concern, and the reason media is never an external URL |

---

## 18. Trade-offs and Rejected Alternatives

| Alternative | Superficial appeal | Why rejected |
|---|---|---|
| **Build on Ethereum or Solana now** | Instant liquidity, existing wallets, no ledger to build, credibility with a crypto audience | Every Posting becomes a public, priced, latency-bound external transaction — irreconcilable with P10's interaction budget and with a social platform's write volume. Fees make micro-rewards (the core of `17`) uneconomic. Every user needs external wallet custody, which contradicts P8's stance on key custody and adds a catastrophic onboarding tax. It hands the economy's regulatory posture to a third party at Phase 1. And it forces the asset model into a static standard (§10.1), which is precisely the thing we are rejecting on design grounds. The `Chain` port keeps this available as an *adapter* whenever it is genuinely useful |
| **Use an existing L2 / appchain** | Cheaper than L1, still "on-chain", faster to a token | Same fee and latency structure, one order of magnitude reduced — still wrong for per-message-scale accounting. Adds sequencer-trust and bridge risk (T7) *on top of* the trust assumption we already have. Fragmented tooling and a shorter maintenance horizon than Postgres (`00 §3.5`). It buys a narrative, not a capability, and pays for it in permanent architecture |
| **Use a generic token standard (ERC-721/1155-shaped) for Facets** | Interoperability with existing marketplaces and wallets | Interoperability with an ecosystem that cannot represent evolution, provenance depth, licences, composition, or enforceable royalties means interoperating by discarding every distinguishing property (§15). We would inherit the URI durability problem, the royalty-as-a-hint problem, and a certificate model for objects that are not certificates. Projection (§15) captures the genuine portability value without conceding the model |
| **No ledger abstraction — just use Postgres directly** | Less code, no trait indirection, faster Phase 1 | Directly violates P5 and N4. The exit cost is unbounded: SQL leaks into the domain, the domain learns table shapes, and Phase 8 becomes a rewrite of everything that touched money — which, in an economy-bearing platform, is everything. The trait costs roughly one crate and a test double; the rewrite costs a phase. This is the exact trade `00 §3.3` requires us to make |
| **A single mutable `balance` column, no double-entry** | Simplest possible thing | Loses the invariant that makes the economy auditable. `Σ debits == Σ credits` is what turns "we think the numbers are right" into a checkable statement. Without it, P12's falsification test cannot run |
| **Turing-complete evolution scripts in Facets** | Maximum expressiveness; every asset a program | Unbounded audit surface, unbounded execution cost, replay dependent on an execution engine, and an infinite security surface at the moment of asset creation. Deferred to the Phase 7 Experience Runtime with a real sandbox (`20`), where it belongs. FN-ASSET/1 stays declaratively verifiable |
| **TigerBeetle (or a dedicated ledger engine) at Phase 1** | Purpose-built, extremely fast, correct by construction | Genuinely the best technical fit for the posting engine and the most credible Phase 4+ swap. Rejected *for now* on complexity budget (`02 §5`): a second stateful system before the spine sentence is true, buying throughput we do not need. The `Ledger` trait makes adopting it a swap, which is the entire point |

---

## 19. Invariants Introduced by This Chapter

Each becomes a property test (`11 §7`) or a CI lint.

1. `Σ` all wallet balances `== 0` at every frontier (L2).
2. Exactly **K accounts** may be negative, and they are the `EmissionAccount` shards `EmissionAccount[0..K)`, K = 64, keyed `i = BLAKE3(society_id) mod K` — the same hash and modulus as the `ShardRouter` (`10 §12`), so a settlement Posting is always same-primary as its Society (LA1, `61 W6`/`N13`).
3. `balance >= locked >= 0` for every Wallet except the `EmissionAccount` shards (L5).
4. `post` with a repeated `batch_id` writes nothing and returns the original receipt (L3).
5. The Posting projection equals the fold of `PostingRecorded` events at every frontier (LE1).
6. Replay from genesis reproduces every historical `state_root` byte-identically (L4).
7. No `f32`/`f64`, `HashMap`, `SystemTime::now`, or ambient RNG in `fractal-domain-ledger` or `fractal-domain-asset` (§5).
8. No `UPDATE` or `DELETE` against the Posting table anywhere in the codebase (L7).
9. Every wallet lock in the ledger path is acquired through `lock_wallets_ordered()` (§4.4).
10. Every `PostingReason` maps to a declared Source or Sink in `17` (`11 §7.15`).
11. Every Facet resolves to exactly one minting `society_id`, and `facet_id` encodes it (P1).
12. `Facet.creator` and `Facet.genesis` are never written after mint.
13. `ProvenanceChain` is append-only; no path shortens or rewrites it.
14. Every `FacetEvolved` names an `EvolutionRule` whose `authority` the actor satisfied.
15. An Agent-authored evolution or transfer carries a valid, unexpired `envelope_ref` (P4).
16. Every Lock and every escrow has a deadline, and `EscrowAccount == Σ open locks` at every checkpoint.
17. A priced transfer commits only together with its royalty and fee Postings, in one batch.
18. `Insignia` Facets are never transferable and never purchasable with Fraction (`02 §4`).
19. Facet composition depth `<= 4` and the composition graph is acyclic.
20. A `ChainBinding` never mutates canonical Facet state.
21. Every Anchor chains to its predecessor, and every anchored claim verifies with the standalone external verifier.

---

## 20. Phase Placement

| Capability | Phase | Note |
|---|---|---|
| `Ledger` trait + Postgres double-entry + reconciler | 1 | N4: abstracted from commit #1 |
| Wallets for Citizens, Societies, Agents | 1 | N5 |
| Internal Anchoring + standalone verifier | 2 | Exercised for years before it matters |
| `FN-ASSET/1` core: mint, state, provenance, transfer | 3 | Facet minting unlocks at Society Level 3 (`11 §2.3`) |
| Evolution engine: Deterministic + Owner triggers | 3 | |
| Licensing, royalties, escrow, marketplace settlement | 6 | With S12; paid Extensions are Phase 6 (`02 §3`) |
| Attested (oracle) evolution | 6 | Needs a Trust baseline for attesters |
| Composition | 6 | |
| `Chain` adapters, external bindings, FN L1 migration | 8+ | §7 procedure |
| Bridges (if ever) | 8+ | §8 conditions, ADR required |
| Fiat `Rail` | 9+ | Gated on counsel, not engineering |

---

## 21. What Would Make Us Change This

Stated in advance so the signal is recognized rather than rationalized.

- **The reconciler finds divergences it cannot explain.** → Stop feature work on the economy. A ledger whose invariants are merely usually true is worse than no ledger, because it is trusted.
- **Postgres posting throughput becomes the binding constraint before Phase 5.** → Swap the adapter to a dedicated ledger engine. The trait is why this is a project, not a rewrite.
- **Declarative evolution proves too weak for real creators** — the same shape keeps being requested and cannot be expressed. → Add bounded, metered, sandboxed evolution as `FN-ASSET/2`, reusing the Experience Runtime sandbox (`20`). Never by adding an escape hatch to `/1`.
- **Facet evolution proves to be a farming vector in production despite §17 T4.** → Ship affected Standards with `Immutable` evolution and re-enable only after the simulation harness (`17`/`19`) clears them under 100x adversarial load. The mechanic ships disabled (P12).
- **Anchoring internally proves insufficient for a real counterparty** (an institution requires third-party-verifiable history before Phase 8). → Anchor to a public chain early, using the *unchanged* Anchor format, through the `Chain` port. This is the migration we most want to be boring, and it is designed to be.
