# 19 — Marketplace

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `10-system-architecture.md` (boundary **S12 Market**), `11-domain-model.md`.
> **Governs:** Listings, discovery and ranking, pricing and licensing, the purchase saga, revenue share, payouts, ratings, the review and recall pipeline, creator tooling, Society-hosted shelves, and marketplace anti-abuse.
> **Does not govern:** the execution sandbox, the host API surface, resource metering, or extension lifecycle inside a Society — all of that is `20-plugin-and-extension-model.md`. Emission, Sources, Sinks, and supply policy are `17-economy-fraction.md`. This document is about the **market**, not the runtime.

---

## 1. Thesis

The marketplace is not a feature of Fractal Node. It is the mechanism by which the platform stops being a product and becomes an **economy** — where individual creators, small studios, and organizations build businesses whose revenue is denominated in Fraction and whose customers are Societies.

A marketplace succeeds on exactly four things. Everything in this document is an attempt to design for one of them.

| Pillar | What it means concretely | Where it is addressed |
|---|---|---|
| **Distribution** | A creator who publishes reaches real Societies without buying attention. Install is one action from the point of discovery. | §8 Discovery, §12 Society shelves |
| **Discovery** | A buyer with a need finds the thing that solves it, and the ranking that surfaces it is explainable and not surveillance-derived (P9). | §8 |
| **Trust** | A buyer can predict what an Extension will do *before* paying, and the platform removes bad actors fast without collateral damage. | §3.4 capability manifest, §10 review pipeline, §13 anti-abuse |
| **Payout reliability** | A creator gets paid on a published schedule, in an amount they can compute in advance, with a stated policy for what is held back and why. | §6 revenue share, §7 payouts |

Marketplaces die of the inverse of each: no distribution (a graveyard of unfound listings), no discovery (ranking captured by whoever games it hardest), no trust (one supply-chain incident and installs stop forever), or no payout reliability (creators leave and never come back — this is the one that is unrecoverable, because it is a reputation loss in the labor market, not in the product).

**Phase reality, stated up front (`02 §3`).** First-party Extensions and **free** third-party Extensions ship earlier. **Third-party paid Extensions and all marketplace payments are Phase 6.** Fiat off-ramp is **Phase 9+** and gated on counsel. Everything in §4–§7 is designed now and built at Phase 6; §3, §8, §9, §10 and §11 are needed as soon as a free third-party Listing exists, because a free Extension still requires a Listing, a capability manifest, a review pipeline, and a recall procedure. Trust infrastructure is not a payments feature.

**Terminology additions proposed by this document** (per `01 §8`, a term is proposed in the same PR that uses it): `publish`/`Published`, `purchase`/`Purchased`, `refund`/`Refunded`, `license`/`Licensed`, `payout`/`PaidOut`, `recall`/`Recalled`, `review`/`Reviewed`, `curate`/`Curated`. `ListingPublished`, `PurchaseCompleted` and `PayoutIssued` are already named in `10 §3` S12.

---

## 2. Category Taxonomy

Nine categories, closed set. A Listing has exactly one `category`; a Listing that "is really two things" is two Listings. The categories differ in what is delivered, how it is priced, how it is versioned, and — most importantly — what review it requires (§10).

| # | Category | What it is | Delivery | Typical pricing | Versioning | Review class |
|---|---|---|---|---|---|---|
| C1 | **Plugin** | An Extension of kind `plugin`: code that runs against the host API, holding an Envelope | Signed WASM component + manifest, resolved at Install | One-time, subscription, or seat | SemVer, compatibility range against host API | R2, or R3 if elevated Envelope |
| C2 | **Theme** | Extension of kind `theme`: design-token overrides and layout variants, no executable code | Token bundle + assets, content-addressed | One-time or free | SemVer against `32-design-system.md` token schema | R0 (R1 if it ships any script) |
| C3 | **Society Template** | Extension of kind `template`: a Charter draft, Chamber layout, role set, starter Envelopes, seeded content | Declarative bundle applied at Society creation or Fork | One-time or free | SemVer; a template version is frozen once applied | R1 (R2 if it pre-grants any capability) |
| C4 | **Agent Workflow** | Extension of kind `workflow`: a declarative automation graph an Agent executes (`15-agent-runtime.md`) | Signed workflow graph + declared capability requirements | One-time, subscription, or usage-metered | SemVer; graph hash pinned per Install | R2; R3 if it touches wallet, vault write, or moderation |
| C5 | **Automation Pack** | Extension of kind `automation-pack`: a curated set of Workflows, triggers, and default Policies solving one operational job | Bundle of C4 units + configuration schema | Subscription (most natural) | SemVer at pack level; members pinned | R2/R3, taking the maximum of members |
| C6 | **SDK / Developer Tool** | Client libraries, codegen, local test harnesses, CI actions. Consumed *outside* the Runtime | Source or binary artifact + signed checksum; no Envelope, because nothing installs into a Society | Free (strongly preferred), one-time, or commercial-seat | SemVer against the public API version | R1 (supply-chain scan; no Envelope to audit) |
| C7 | **Digital Asset (Facet)** | A Facet or a mint-right to a Facet: Insignia, instrument, artifact, template asset (`16-ledger-and-assets.md`) | `FacetMinted` / `FacetTransferred` to the buyer's Wallet at settlement | One-time, edition-limited, or pay-what-you-want | Facets evolve; editions are immutable in count | R0 + IP screen |
| C8 | **Media** | Images, audio, video, fonts, 3D, document sets. Not executable | Vault Object + Manifest, license-gated read | One-time, PWYW, or subscription bundle | Content-addressed; a new version is a new Manifest | R0 + IP screen |
| C9 | **Service** | Human labor: design, moderation shifts, development, curation, translation, audit | An engagement with milestones; escrowed (§5.4) | Fixed-price per milestone, retainer, or hourly-with-cap | Not versioned; scoped by a Statement of Work | R1 on the Listing + creator identity verification |

**Why "services" is in the same market as code.** A Society that needs a moderation rota needs *either* a moderation Workflow *or* three humans on a schedule, and frequently both. Splitting these into two marketplaces means the buyer has to know which kind of solution their problem takes before they can search. It also means the labor half never inherits the trust, escrow, dispute, and reputation machinery that the software half gets. The cost is real: services need escrow, milestones, and dispute resolution that software purchases do not (§5.4, §5.5), and they cannot be automatically recalled. We accept that cost.

**Why C6 (SDKs) has no Envelope.** Developer tools run on the creator's own machine, not inside a Society. They therefore get no capability audit — only supply-chain scanning and signature verification. This asymmetry must be stated in the Listing UI, because a buyer's intuition ("the platform reviewed it") is otherwise wrong in the one category where it matters most for their laptop.

---

## 3. The Listing Model

A **Listing** is the marketplace's public record of a purchasable or installable thing. It lives on the **Global Registry** (`01 §6`, entry 6: `Extension` — marketplace listings, pre-install) because a Listing must be discoverable before any Society owns anything. An **Extension Install** is Society-scoped (P1); the Listing is not.

```rust
struct Listing {
    listing_id:     ListingId,           // "lst_" + ULID
    category:       Category,            // C1..C9, closed enum
    creator:        Fnid,                // Citizen or Society — always exactly one seller of record
    slug:           ListingSlug,         // globally unique, confusable-normalized (§13)
    display:        ListingDisplay,      // name, summary, long description, media
    versions:       Vec<ListingVersion>, // append-only; never rewritten
    current:        VersionId,           // the version offered to new buyers
    pricing:        PricingModel,        // §4.1
    license_terms:  LicenseTerms,        // §4.2
    review_state:   ReviewState,         // Draft | InReview | Published | Quarantined
                                         // | Recalled | Delisted | Deprecated
    risk_class:     RiskClass,           // R0..R4, assigned by the pipeline, not the creator
    shelves:        Vec<ShelfRef>,       // Society-hosted shelves carrying this (§12)
    support:        SupportTerms,        // response SLO, contact route, EOL policy
    created_at:     Timestamp,
}

struct ListingVersion {
    version_id:     VersionId,
    semver:         SemVer,
    artifact:       ContentHash,         // BLAKE3 multihash; the bytes are immutable
    signature:      Signature,           // creator's release key, verified at install
    sbom:           ContentHash,         // required for C1, C4, C5, C6 (P8)
    manifest:       CapabilityManifest,  // §3.4 — the Envelope this version will request
    compatibility:  CompatRange,         // host API range, plus platform/runtime constraints
    changelog:      Changelog,           // required, structured, non-empty
    published_at:   Option<Timestamp>,
    deprecation:    Option<Deprecation>,
}
```

### 3.1 Invariants

1. A `ListingVersion.artifact` hash is immutable once `Published`. Republishing changed bytes under the same version is impossible by construction — the hash is the identity.
2. `versions` is append-only. A withdrawn version is marked, never deleted; existing Installs keep a resolvable artifact.
3. A Listing with `review_state = Published` has at least one version whose `manifest` has been audited at its `risk_class`.
4. `creator` is a single Principal. Co-creators are handled by the Society-as-creator path (a Society Listing pays into a Treasury and splits by Charter), never by a multi-owner Listing. **One seller of record, always** — this is what makes payout, liability, and takedown tractable.
5. A `Recalled` version can never return to `Published`. Recovery is a new version.

### 3.2 Metadata and media

Required: name, one-line summary (≤ 100 chars), long description (Markdown, no script), category, at least one screenshot or a 15–60s demo capture for C1–C5, a support route, and an EOL policy. Optional: locale variants, accessibility notes, a "works offline" declaration (which is verified, not asserted — an Extension claiming offline capability is tested against a network-denied harness, per P2's falsification test).

All Listing media lives in a platform-owned Vault namespace with the same Shard machinery as any other Object (`13`). Listing media is public by definition; nothing else about a creator is.

### 3.3 Versioning, compatibility, and deprecation

- **SemVer, enforced mechanically where possible.** A version that removes a declared extension point, narrows an output schema, or changes a configuration key is a major bump; the publish pipeline detects these from the manifest diff and rejects a mislabelled patch release.
- **Compatibility ranges** are expressed against the public API version (P3), not against a product version. `compat: ">=api/1.4, <api/2.0"`. Installs resolve to the highest compatible version.
- **Deprecation** is a first-class state with a mandatory notice period: 90 days for paid Listings, 30 days for free. During deprecation the Listing is unavailable to new buyers, remains installable and updatable for existing licensees, and displays the successor Listing if one is named. Subscriptions on a deprecated Listing stop renewing at the notice date; no subscription may outlive its Listing's support horizon.

### 3.4 The capability manifest — shown before purchase

This is a P8 requirement expressed as a commerce rule, and it is the single most load-bearing element of the Listing.

```rust
struct CapabilityManifest {
    requested:   CapabilitySet,        // exactly the Envelope this version will ask for
    rationale:   BTreeMap<Capability, String>, // one required sentence per capability
    optional:    CapabilitySet,        // degrades gracefully if denied — must actually degrade
    network:     Vec<NetworkTarget>,   // declared egress hosts; anything else is blocked
    data_flows:  Vec<DataFlow>,        // what leaves the Society, to where, why (P9)
    limits:      Limits,               // rate, spend caps, storage ceiling
    verified:    ManifestVerification, // static-analysis attestation from the pipeline
}
```

**Invariant M1 — no purchase without disclosure.** The full manifest is rendered on the Listing page, above the purchase control, in plain language, for every visitor, signed-out included. Purchase and install flows both require an explicit consent step that shows the same content. A Listing whose manifest is not renderable cannot be `Published`.

**Invariant M2 — the manifest is a ceiling, not a wish.** The runtime Envelope granted at Install is derived from the manifest and can only be narrower. An Extension cannot request at runtime what it did not declare at publish. Enforcement is in the Policy Enforcement Point (`10 §8`); the mechanics are `20`.

**Invariant M3 — capability growth requires re-consent.** If version *n+1* requests any capability, network target, or data flow not present in version *n*, the update does **not** auto-apply. It queues as a pending update, the Society sees a diff of exactly what is new and why, and a Citizen holding the granting capability must approve. Until then the Society continues to run version *n*. This closes the single most common app-store abuse pattern (§13, T3).

**Invariant M4 — the diff is the disclosure.** Re-consent shows the *delta*, not the whole manifest again. A full re-read is how consent fatigue is manufactured; a two-line diff is how consent stays meaningful.

---

## 4. Pricing and Licensing

### 4.1 Pricing models

All prices are denominated in Fraction (quanta internally). No fiat prices exist in the product before Phase 9+.

| Model | Shape | Best fit | Constraints |
|---|---|---|---|
| **Free** | 0 FRC | C2, C3, C6, first-party | Still requires Listing, manifest, review |
| **One-time** | Single purchase, perpetual license | C1, C2, C7, C8 | Refund window per §5.3 |
| **Subscription** | Recurring per period (monthly/annual) | C1, C5, C9 retainers | Renewal is a distinct consented Posting; auto-renew is opt-in, never default (`02 §4`: no dark patterns). Price increases require 30 days notice and re-consent, never silent |
| **Usage-metered** | Priced per declared unit (invocations, transcodes, tokens) | C4, C5 | Buyer sets a hard spend cap at purchase; the Envelope enforces it; overage is refused, never billed |
| **Pay-what-you-want** | Floor ≥ 0, suggested price, no ceiling | C7, C8, indie C2 | Suggested price is displayed neutrally; no shaming copy on low amounts |
| **Society-seat** | Priced per active Membership, banded | C1, C5 in large Societies | Seat count is computed from the Society's own membership projection; recount monthly; bands, not per-head, to avoid churn billing |
| **Revenue-share / affiliate** | A curator or referrer earns a share of sales they originate | any | Comes out of the creator's share, capped (§6); attribution requires explicit, disclosed referral — never inferred from behavior (P9) |

**Rejected:** trials that auto-convert without a second consent; regional price discrimination (there is one currency and no geography model); "free with in-Extension purchases" as an unbounded pattern — an Extension that can spend must declare a spend cap in its manifest and cannot exceed it.

### 4.2 The License model

```rust
struct LicenseTerms {
    scope:        LicenseScope,      // Personal | Society | Commercial
    seats:        Option<SeatPolicy>,
    derivative:   DerivativeRights,  // None | PrivateModification | Redistribute { attribution }
    duration:     Duration,          // Perpetual | Term(days) | Subscription
    transferable: Transferability,   // NonTransferable | OnceWithFee | Freely
    revocation:   RevocationTerms,   // what ends it besides expiry: recall, fraud, breach
    territory:    (),                // deliberately absent — see note
}
```

| Dimension | Options | Default | Note |
|---|---|---|---|
| Scope | Personal (one Citizen), Society (all members of one Society), Commercial (use in a revenue-generating Society or resold service) | Society | Society scope is the natural unit because P1 makes the Society the container that installs things |
| Derivative | none / private modification / redistribute with attribution | private modification | "Private modification" matters: a Society that forks a Template must not be in breach for editing its own Charter |
| Duration | perpetual, term-boxed, subscription-linked | perpetual for one-time | A perpetual license survives the Listing being delisted; it does not survive a Recall for malice |
| Transferability | non-transferable / once-with-fee / freely | non-transferable for C1–C6; freely for C7, C8 where the creator elects | Free transferability turns a license into a secondary market; that is correct for assets and wrong for seats |

There is deliberately **no territory field**. The platform has no geography model and no legal capacity to enforce one. Territorial licensing, export control, and sanctions screening are questions for counsel before any fiat rail exists (Phase 9+); pretending to model them now would be worse than not modelling them.

### 4.3 How a license is represented — decision

**Decision: a License is a signed Grant record in the Asset boundary (S8), anchored by a Ledger Posting, and mirrored as a Facet only when the terms make it an asset.**

```rust
struct LicenseGrant {
    grant_id:     GrantId,
    listing:      ListingId,
    version_at_purchase: VersionId,
    licensee:     Principal,          // Citizen | Society | Agent
    terms:        LicenseTerms,       // snapshot at purchase — later term changes do not apply
    posting_ref:  Ulid,               // the settling Posting; the economic proof
    issued_at:    Timestamp,
    expires_at:   Option<Timestamp>,
    state:        GrantState,         // Active | Expired | Revoked | Refunded | Transferred
    signature:    Signature,          // platform issuance key; verifiable offline (P2)
}
```

**Why not "every license is a Facet."** It is the tempting answer and it is wrong for the common case:

- Facets are **Society-scoped** (`11 §2.9`). A Personal-scope license held by a Citizen who belongs to no Society would have no valid `society_id` — a direct P1 violation requiring a Global Registry addition (an ADR, `01 §6`).
- Facets are **transferable and evolvable by default**. Most licenses must be neither. Building non-transferability as a per-Facet exception inverts the defaults of the asset system.
- Mint and evolution carry fees and provenance obligations that are correct for collectibles and absurd for a seat license renewed monthly.
- License checks happen on every Install and every offline start. A Grant is a small signed record a Node can verify with no network (P2). A Facet lookup drags in the whole asset subsystem for an authorization decision.

**Why not "a license is only a Ledger entry."** A Posting proves that Fraction moved. It does not carry terms, scope, seat counts, or derivative rights, and `PostingReason` is a closed enum that must not grow a payload (`11 §2.6`). Money movement and rights grants are different facts and are modelled separately, joined by `posting_ref`.

**Where the Facet mirror applies.** When `transferable = Freely` and the category is C7 or C8, the Grant is mirrored as a Facet under the creator's Society. Then the license *is* an asset: resellable, provable, collectible, evolvable (an edition badge that records its chain of custody). This is opt-in per Listing, and the Facet is the transfer mechanism while the Grant stays the authorization record — one truth, two representations, with the Grant always authoritative.

**Invariant L1.** Every `LicenseGrant` references exactly one settling Posting, or is marked `promotional` with a named grantor. There is no third way for a license to exist.
**Invariant L2.** Terms are snapshotted at purchase. A creator changing `license_terms` on a Listing affects future purchases only. Retroactive term changes are impossible by construction.

---

## 5. Commerce

### 5.1 The purchase saga

Extends `PurchaseExtension` in `11 §5`, which is the canonical saga name.

```
 BUYER                    MARKET (S12)              LEDGER (S5)        EXTENSION (S11)
   │                          │                          │                   │
   │ 1. intent(listing,ver)   │                          │                   │
   ├─────────────────────────►│                          │                   │
   │                          │ 2. PRECHECK              │                   │
   │                          │  · listing Published?    │                   │
   │                          │  · version compatible?   │                   │
   │                          │  · buyer eligible?       │                   │
   │                          │    (Level/Standing gate) │                   │
   │                          │  · duplicate license?    │                   │
   │ 3. CONSENT SCREEN        │                          │                   │
   │◄─────────────────────────┤  manifest + price +      │                   │
   │   explicit approve       │  terms + refund window   │                   │
   ├─────────────────────────►│                          │                   │
   │                          │ 4. RESERVE ─────────────►│ lock funds        │
   │                          │                          │ balance≥locked≥0  │
   │                          │◄─────────────────────────┤ reservation_id    │
   │                          │ 5. VERIFY                │                   │
   │                          │  · artifact hash         │                   │
   │                          │  · creator signature     │                   │
   │                          │  · not Quarantined       │                   │
   │                          │    since precheck        │                   │
   │                          │ 6. ISSUE LicenseGrant    │                   │
   │                          │    (state = Active)      │                   │
   │                          │ 7. INSTALL ──────────────┼──────────────────►│
   │                          │    (Envelope ⊆ manifest) │   ExtensionInstalled
   │                          │◄─────────────────────────┼───────────────────┤
   │                          │ 8. SETTLE ──────────────►│ split Postings:   │
   │                          │                          │  creator / platform│
   │                          │                          │  / society / affil │
   │                          │                          │  + burn (Sink)     │
   │ 9. RECEIPT               │◄─────────────────────────┤                   │
   │◄─────────────────────────┤  PurchaseCompleted        │                   │
```

**Saga rules.**
- Steps 4–8 are one saga with compensation defined at every step: fail before 6 → release reservation, no license, no charge. Fail at 7 → license stands, install retried; **a paid-for license is never destroyed by an install failure**, because the buyer bought rights, not a successful install.
- Step 5 re-verifies after consent. A Listing quarantined between browse and pay must not sell.
- Step 8 is the only step that moves Fraction to a creator, and it is a single balanced multi-leg Posting set. Partial splits cannot exist (`11 §7.2`).
- The whole saga carries one `idempotency_key` (`10 §5`). Retrying a purchase never double-charges.
- For C9 Services, step 7 is replaced by **escrow funding** (§5.4) and step 8 is deferred to milestone acceptance.

### 5.2 Receipts

Every purchase emits an immutable receipt: listing, version, price, split breakdown *including the exact platform fee and burn*, terms snapshot, refund deadline, and the Posting IDs. Receipts are exportable (P9: personal data is exportable) and queryable from the CLI (`fn market receipts`). The split is shown to the buyer, not only the creator — a buyer is entitled to know how much of their payment reached the person who made the thing. Most stores hide this. Showing it is a competitive statement and costs nothing.

### 5.3 Refunds

| Category | Window | Additional condition | Rationale |
|---|---|---|---|
| C1 Plugin, C4 Workflow, C5 Pack | 14 days | < 2 hours cumulative metered runtime, or < 20 invocations | Matches the Steam norm buyers already understand; runtime cap prevents "use the whole thing, then refund" |
| C2 Theme, C3 Template | 14 days | Template not yet applied to a Society; once applied, no refund | Application is irreversible in the buyer's favour |
| C6 SDK | 14 days | Unconditional | Nothing to meter |
| C7 Facet, C8 Media | 48 hours | Content not materialized (downloaded/decrypted) | Non-revocable goods; the honest answer is a short window, not a fake one |
| C9 Service | Per milestone | Escrow release governs (§5.4) | Labor already performed is not refundable |
| Subscription | Pro-rated on cancel | Current period refunded pro-rata if < 25% consumed | Standard and predictable |

Refunds beyond these windows are at creator discretion, one-click available to the creator, and count as a positive Trust signal for the creator rather than a neutral one.

**Refund-abuse control.** Each Citizen carries a rolling 12-month refund budget: `max(5 refunds, 25% of purchase count)`. Exceeding it does not ban refunds; it routes them to manual review and freezes new purchases at the same creator. Budget consumption is disclosed to the Citizen (P9: no hidden scoring).

### 5.4 Escrow for services (C9)

```
 Engagement created ──► buyer funds escrow (full or per-milestone)
        │                     │ Fraction moves buyer → EscrowAccount (locked)
        │                     ▼
        │            ┌────────────────────┐
        │            │ MILESTONE ACCEPTED │──► release: escrow → provider (minus fees)
        │            └─────────┬──────────┘
        │                      │ no response in acceptance_window (default 7d)
        │                      ├──► AUTO-RELEASE (disclosed at funding time)
        │                      │
        │            ┌─────────▼──────────┐
        │            │ MILESTONE DISPUTED │──► §5.5
        │            └────────────────────┘
        ▼
 Engagement cancelled before start ──► full escrow return, no fee
```

Escrow funds are `locked` in the buyer's Wallet accounting until release (`11 §2.6`: `balance >= locked >= 0` holds throughout). Both parties stake a small bond (2% of engagement value, min 10 FRC) that is slashed on a finding of bad faith — this is the only economically meaningful deterrent available in a closed economy against frivolous disputes on either side.

### 5.5 Chargebacks and dispute resolution

**There is no chargeback in a token economy, and pretending otherwise is dishonest.** No card issuer stands behind a Fraction transfer; nobody can reverse a settled Posting, because the Ledger is append-only (P6, `11 §7.2`). What exists instead:

1. **Reversal by forward Posting.** A refund is a new balanced Posting in the opposite direction, referencing the original. History is never rewritten.
2. **The Creator Assurance Reserve.** Funded from the platform fee (§6.2). If a creator's payout holdback is exhausted and a refund is owed, the Reserve pays the buyer and the creator's account carries a negative balance recovered from future settlements. The buyer is made whole; the creator's obligation persists. This is the platform absorbing timing risk, not credit risk.
3. **Dispute ladder** — each step has a stated clock, and the whole ladder is a public policy document, not case-by-case discretion:

```
 L0  Direct resolution        creator ↔ buyer, 7 days, in-thread
      │ unresolved
 L1  Platform mediation       structured evidence, 14 days, decision published
      │  ▼ ≤ 500 FRC → binding here (the vast majority of disputes)
      │ appealed, > 500 FRC
 L2  Arbitration panel        3 Citizens: high Trust, no relationship to either
      │                       party, category-competent; stake required to serve;
      │                       decision + reasoning recorded as a domain event
      │ > 5,000 FRC or systemic (fraud, IP, safety)
 L3  Platform security/legal  the only step where the platform acts unilaterally;
                              always disclosed, always logged, counsel-gated
```

**Flag for counsel:** whether L2 panel decisions constitute binding arbitration, what disclosures a marketplace operator owes, whether the Assurance Reserve is a regulated instrument, and the consumer-protection floor applicable to digital goods sold for a non-fiat unit are all legal questions. This document specifies mechanism only. **No fiat rail ships before those answers exist.**

---

## 6. Revenue Share

### 6.1 The split

Gross price is what the buyer pays. Every fee is a percentage of gross, computed at settlement, and disclosed on both the receipt and the creator statement.

| Party | Share | Set by | Notes |
|---|---|---|---|
| **Platform** | **12%** standard; **10%** for C9 Services; **4%** launch rate | Platform, published | Launch rate applies to a verified creator's first 10,000 FRC lifetime gross |
| **Society** (when a Society-hosted Shelf originated the sale) | **0–10%**, default 0 | The hosting Society's Charter | Hard-capped at 10%. Only paid when the sale is attributed to that Shelf (§12) |
| **Affiliate / Curator** | **0–10%**, default 0 | The creator, per Listing | Deducted from the creator's share, not added to the buyer's price |
| **Creator** | Residual | — | **Floor: 70% of gross, always** |

**Invariant R1.** `platform + society + affiliate ≤ 30%` of gross. If stacked fees would exceed 30%, the *Society* share is reduced first, then the affiliate share. The creator never receives less than 70% of gross for any transaction, in any configuration. This is a hard check in the settlement code, not a policy statement.

**Invariant R2.** Fees are never charged on a refunded transaction. A refund reverses every leg, including the burn — the burn is reissued from the Emission Account with an explicit `RefundBurnReversal` reason so total supply stays exactly computable (`11 §7.4`).

### 6.2 Where the platform fee goes — the Sink (P12, cross-ref `17`)

The platform fee is not revenue in the usual sense; it is a **named Sink** plus two named accounts, because P12 requires every Fraction to have a named source and a named sink.

```
   12 pp platform fee
      ├── 6 pp  ──►  BURNED           Sink::MarketplaceFee — removed from circulation
      ├── 4 pp  ──►  Operations Acct  funds review pipeline, security audits, infrastructure
      └── 2 pp  ──►  Assurance Reserve funds refunds/disputes when holdback is exhausted (§5.5)
```

Half the fee is destroyed. This matters: marketplace volume becomes the platform's largest deflationary Sink, which means economic health improves as commerce grows rather than depending on emission growth. The Operations and Assurance accounts are published Wallets with public balances — anyone can verify the split (`17 §Sinks`). The Assurance Reserve has a ceiling (proposed: 90 days of trailing refund volume); overflow is burned rather than accumulated, so the platform cannot quietly build a war chest out of a consumer-protection mechanism.

### 6.3 Comparison and justification

| Platform | Standard take | Reduced tier | Condition |
|---|---|---|---|
| Apple App Store | 30% | 15% | < $1M/yr, or year 2+ subscriptions |
| Google Play | 30% | 15% | First $1M/yr |
| Steam | 30% | 25% / 20% | > $10M / > $50M lifetime — *volume rewards scale* |
| Unity Asset Store | 30% | — | Flat |
| Figma Community | ~15% (+ processing) | — | Deliberately low to seed supply |
| WordPress.org | 0% | — | No commerce layer at all; monetization is off-platform |
| Upwork / Fiverr (services) | 10% / 20% | — | Labor marketplaces cluster near 10–20% |
| **Fractal Node** | **12%** | **4%** | Reduced tier for *small* creators, not large ones |

**Why 12% and not 30%.** The 30% figure is priced for a monopoly distribution channel — payment processing (~3%), hosting, review, and, decisively, exclusive access to a device's users. We have none of that leverage and should not price as if we do. Our actual per-transaction costs are the Ledger Posting (near zero — it is our own ledger), review amortized over versions, hosting of artifacts and Listing media, and the dispute/assurance apparatus. Twelve percent covers those with margin at modest volume and is low enough to be a recruiting argument to creators who are currently paying 30%.

**Why the reduced tier goes to small creators, not large ones.** Steam's declining schedule rewards the publishers who least need it and worsens earnings concentration. Our marketplace-health metric is the earnings Gini (§14); a schedule that concentrates earnings degrades the metric we claim to optimize. The launch rate (4% on the first 10,000 FRC lifetime gross) directly attacks the hardest problem in any new marketplace: time-to-first-sale and time-to-first-meaningful-revenue.

**Honest cost.** A flat low fee means the platform earns less from its biggest sellers, exactly where marginal support costs are highest. If review and dispute costs exceed the Operations allocation at scale, the correct response is a *published* fee change with 90 days notice applying to future purchases only — never a silent take-rate creep, which is the standard way marketplaces betray their creators.

**Gaming the launch rate.** A creator could split into many identities to stay under 10,000 FRC forever. Mitigations: the launch rate requires a **verified creator identity** (`12-identity-and-trust.md`), the allowance is bound to that identity rather than to the Listing, and reuse of the same signing key or artifact hashes across identities collapses them into one allowance. Residual risk: determined multi-identity abuse. Accepted — the maximum extractable value is 8 pp on 10,000 FRC per fabricated identity, which is small relative to the cost of maintaining a verified identity with Trust.

---

## 7. Payouts

**How a creator actually gets paid.** At settlement, Fraction moves from the buyer's Wallet to the creator's Wallet as a balanced Posting set. The creator's Wallet is the same Wallet they use for everything else (N5). There is no separate "merchant account", no external processor, and — inside the platform — no delay beyond clearing. This is the substantive advantage of an internal ledger: payouts are ledger writes, not wire transfers.

| Parameter | Value | Rationale |
|---|---|---|
| Settlement cadence | Weekly, Mondays 00:00 UTC, covering the prior week | Predictable; batching keeps statements legible |
| Clearing delay | T+7 days from purchase | Covers the bulk of the 14-day refund window without holding all funds |
| Refund holdback | 15% of each settlement, released after 30 days | Sized to ~2.5× the healthy refund rate (§14) |
| Holdback waiver | Creators with ≥ 12 months tenure, ≥ 500 settled sales, and refund rate < 3% | Earned trust reduces friction; the waiver is revoked on a rate excursion |
| Minimum payout threshold | **None** for internal FRC | An internal Posting has no per-transaction cost, so a threshold would be an artificial hold on someone else's money |
| Statement | Per settlement: gross, each fee leg, holdback movement, refunds, net; exportable; CLI-queryable | P3/P13 — the CLI shows what the GUI shows |
| Negative balance | Permitted only via Assurance Reserve recovery (§5.5); recovered from future settlements at ≤ 50% per run | A creator is never zeroed out in one run |

**Fiat off-ramp: Phase 9+, gated on counsel.** Stated plainly because creators will ask on day one: *until Phase 9 you can earn Fraction and spend it inside Fractal Node, and you cannot convert it to fiat.* The `Rail` port (`10 §7`) is where a fiat processor plugs in, and the abstraction is honest today, but building it requires a licensed entity, KYC/AML, tax reporting, money-transmission analysis per jurisdiction, and a sanctions programme. `02 §3` places it at Phase 9+ and `17` explains why premature external liquidity destroys a young internal economy. **Any creator-facing copy that implies imminent cash-out is a defect.** The recruiting pitch is a functioning internal economy with lower fees, not a cash-out promise the platform cannot legally make.

---

## 8. Discovery and Ranking

P9 bans engagement-optimized ranking and covert behavioral data. `02 §4` lists "engagement-optimized ranking" on the Never List. This constrains discovery more than any other design in the document, and the constraint is the point: an app store's ranking algorithm is the most captured surface in software distribution precisely because it optimizes a proxy metric that money can buy.

### 8.1 Allowed and banned signals

| Signal | Allowed? | Why |
|---|---|---|
| Declared category and tags (creator-set, audited) | **Yes** | Declared, inspectable, correctable |
| Explicit search query terms | **Yes** | The Citizen stated the intent this second |
| Verified compatibility with the querying Society's API version, Level, and installed set | **Yes** | Factual, computed at query time, not stored |
| Curated collections (named human curators, disclosed) | **Yes** | Editorial with an accountable name attached |
| Society-curated Shelf membership (§12) | **Yes** | The Society's own governance decided |
| Creator Trust (`01 §7`) | **Yes** | Not purchasable, not volume-derived |
| Review pipeline outcomes: risk class, audit recency, security-incident history | **Yes** | Objective quality facts |
| Support-SLO adherence, unresolved-defect age, deprecation status | **Yes** | Measured obligation, not popularity |
| Rating average with Trust weighting and a minimum-n threshold | **Yes** | Purchaser-only (§9) |
| Install count, shown as a coarse band (10+/100+/1k+/10k+) | **Yes, display only** | Useful context; banded so it cannot be micro-optimized |
| **Citizen's declared interests** (`11 §2.1`, written only by the Citizen) | **Yes** | P9's explicit carve-in |
| Paid placement / promoted listings / ad auctions | **No** | P9, `02 §4`. Not "not yet" — never |
| Dwell time, scroll depth, hover, click-through, session length | **No** | Covert behavioral surveillance |
| Cross-Society browsing history, inferred interest vectors, lookalike modelling | **No** | P9; `11 §7.13` forbids inferred interests as an invariant |
| Recency-of-engagement decay tuned to drive return visits | **No** | Engagement optimization by another name |
| Velocity/trending computed from raw install spikes | **No** | Trivially manufactured; the classic bot-farm vector |
| Personalized ordering derived from anything not declared | **No** | Same as above |

### 8.2 The ranking function

```
score(listing, query, context) =
      w1 · text_relevance(query, listing.searchable_text)      // BM25, transparent
    + w2 · compatibility_fit(listing, context.society)          // binary-ish, hard gate
    + w3 · trust_weighted_rating(listing)                       // min n = 5, else neutral
    + w4 · creator_trust(listing.creator)                       // sublinear: log-scaled
    + w5 · quality_signals(listing)                             // audit recency, SLO, defects
    + w6 · declared_interest_match(context.citizen.interests)   // OPT-IN, off by default
    - p1 · deprecation_penalty
    - p2 · unresolved_incident_penalty
```

Weights `w1..w6` are **published constants**, versioned, and changed only by ADR with a changelog entry. The Listing UI carries a "why am I seeing this?" control that names the contributing terms for that specific result. A ranking that cannot be explained to the creator it demotes is a ranking that will be captured.

### 8.3 Sorts the Citizen controls

Default sort is **Relevance** for a query and **Curated** for a browse. The Citizen can switch to: Newest, Recently Updated, Highest Rated (min n), Most Installed (banded), Lowest Price, Free Only, Creator Trust, Audit Recency. The choice is sticky per Citizen, stored locally (P2), and never overridden by the platform.

**Honest cost.** Without behavioral ranking, cold-start discovery is weaker. A genuinely excellent Extension with no reviews and a low-Trust creator will surface below a mediocre one with tenure. Mitigations: a **New and Unreviewed** shelf with guaranteed rotation (deterministic round-robin, not lottery), the launch fee rate (§6.1), and Society Shelves as the primary editorial engine. We accept measurably worse cold-start discovery in exchange for a ranking surface that cannot be bought. That is the trade P9 makes, stated in the open.

---

## 9. Ratings and Reviews

| Rule | Specification |
|---|---|
| Who may review | Holders of an `Active` LicenseGrant only, ≥ 72 hours after issuance. Free Listings count — a free install is a license |
| One per license | One review per (Citizen, Listing); editable, with edit history retained |
| Rating scale | 1–5 stars plus required structured facets: *works as described*, *support responsiveness*, *value* |
| Refunded purchases | Review is retained and **labelled "refunded"**. Deleting it would let creators buy silence with refunds |
| Trust weighting | Displayed average weights each review by `sqrt(reviewer_trust_normalized)`, floor 0.25. Sublinear on purpose: high-Trust Citizens get more weight, never veto power. Both raw and weighted averages are shown |
| Creator response | One public reply per review, unlimited edits, no ability to delete or hide the review |
| Recency | Reviews carry the version reviewed. Reviews of versions > 2 major behind are collapsed by default and clearly counted |
| Moderation | Reviews are moderated for abuse and doxxing only. **A negative but accurate review is never removable**, by anyone, including the platform |
| Incentivized reviews | Offering anything of value for a review — Fraction, discounts, Facets, Standing — is a Trust-slashing offence for both parties and a delisting offence on repeat |

**Brigading defenses.** Anomalous review velocity (rate, Trust distribution, or account-age distribution outside the norm for that Listing's install base) triggers a **display freeze**, not a deletion: the aggregate stops updating, a notice appears, and reviews are examined. Freezing rather than deleting is deliberate — deletion is indistinguishable from censorship and destroys the evidence needed to judge. Reviews from Citizens whose license was purchased by a third party (gifting) are labelled. Reviews from accounts under Level 1 are counted at the 0.25 floor.

**Residual risk.** A patient, well-funded adversary with aged, Trust-bearing accounts can still move a rating. No purchaser-gated system fully prevents this. The rating is one term of six in §8.2 for exactly this reason.

---

## 10. Quality, Safety, and the Review Pipeline

### 10.1 Risk classes

Risk class is assigned by the pipeline from the capability manifest and the artifact, **never declared by the creator**. Creators may see the derivation; they may not set it.

| Class | Trigger | Gates |
|---|---|---|
| **R0** | No executable code, no Envelope (C2 static, C7, C8) | Automated only: malware/format scan, IP screen, content policy |
| **R1** | Declarative only, no capability grant (C3, C6, C9 listing copy) | Automated + spot-check sampling (≥ 10% of first publishes) |
| **R2** | Executable, standard Envelope (read chamber, post, store within quota) | Automated + human functional review + manifest audit |
| **R3** | Elevated Envelope: wallet spend, vault write outside own namespace, external network egress, moderation action, agent control, key access | R2 + security review + **verified creator identity** + **published Stake** (slashable, min 1,000 FRC or 10% of trailing 90-day revenue) |
| **R4** | Experience Runtime (Phase 7), or anything requesting a capability with no prior precedent | R3 + adversarial review + phased rollout cap |

### 10.2 Automated checks (every publish, every class)

```
 SUBMIT ──► ① signature + key continuity (same release key as prior version, or
        │      an explicitly consented key rotation event)
        ├──► ② artifact determinism: reproducible build attestation where the
        │      toolchain supports it; hash recorded either way
        ├──► ③ static analysis: capability inference from the code vs the DECLARED
        │      manifest. An undeclared capability path is a hard reject
        ├──► ④ dependency scan + SBOM diff vs prior version; known-CVE gate;
        │      new transitive dependency from an unvetted source flags to human
        ├──► ⑤ license compliance: declared license vs dependency licenses;
        │      copyleft contamination check; attribution completeness
        ├──► ⑥ secret scan (P8: secrets never ship)
        ├──► ⑦ content/IP screen: perceptual hashing against known works,
        │      trademark string match on name and slug (§13 T1)
        └──► ⑧ resource profile: startup time and memory against the P10 budget
```

Check ③ is the one that matters. **The manifest is verified against the code, not trusted from the creator.** A declared-vs-inferred mismatch is a hard reject with the specific call sites named. Where static inference cannot decide (dynamic dispatch, indirect host calls), the pipeline escalates to human review rather than passing — inconclusive is not a pass.

### 10.3 Human review tiers and the fast path

| Path | Condition | Target latency |
|---|---|---|
| **Fast path** | Patch or minor version, **zero manifest delta**, same release key, all automated checks green, creator has no incident in 180 days | Auto-publish ≤ 1 hour; post-publish audit sampled at 20% |
| **Standard** | R2 first publish, or any minor with a non-capability manifest change | p50 ≤ 2 business days, p95 ≤ 7 |
| **Elevated** | R3, or any capability delta in any class | p50 ≤ 5 business days, p95 ≤ 14 |
| **Adversarial** | R4, or a creator returning from a Recall | No SLO; it takes what it takes |

**Any capability delta leaves the fast path.** There is no version-number trick that widens an Envelope quietly (§13 T3). This is the rule that makes the fast path safe to have at all.

### 10.4 Recall — the kill switch

The scenario: a Listing installed in 4,000 Societies is discovered to exfiltrate Chamber content. Four escalating responses:

| Level | Action | Effect on existing Installs | Refunds |
|---|---|---|---|
| **Advisory** | Notice on the Listing and in every affected Society's admin surface | None; keeps running | No |
| **Quarantine** | Listing unbuyable and uninstallable-new; version pinned | Keeps running; updates blocked | No |
| **Suspend** | Envelope revoked platform-wide; Install becomes inert; configuration and data retained | Stops executing within one heartbeat interval | Pro-rata on subscriptions |
| **Recall** | Suspend + forced uninstall + license revoked + `RecallIssued` on the Global Registry | Removed; Society data written by it is retained and exportable | Full, funded from creator holdback then Assurance Reserve |

```
 DETECTION (pipeline audit | Society report | researcher disclosure | telemetry anomaly)
        │
        ▼  triage ≤ 4h for suspected active exploitation
 ┌──────────────┐   2-of-N platform security signatures required for Suspend/Recall
 │ RECALL ORDER │   (no single operator can kill 4,000 installs)
 └──────┬───────┘
        │ signed order → Global Registry event → Signal fan-out (Relay)
        ├────────────► ONLINE NODES: Envelope revoked at the Policy Enforcement
        │              Point on the next command. Immediate. (10 §8)
        │
        └────────────► OFFLINE NODES: cannot receive the order.
                       Mitigation: every Install carries a signed capability
                       attestation with a TTL (default 24h, 1h for R3). An offline
                       Node stops honoring the Envelope when the attestation
                       expires — fail-closed, not fail-open.
        ▼
 POST-RECALL: public incident report within 7 days (what, when, what data,
              what to do). Creator Stake slashed on a malice finding. Affected
              Societies get an exportable list of every action the Extension took,
              reconstructed from the event log (P6 — this is why events matter).
```

**The offline gap is the honest residual risk of P2.** A Node that is offline past its attestation TTL is safe (fail-closed); a Node offline *within* the TTL keeps running a recalled Extension for up to that window. Shortening the TTL trades offline usability against recall latency. R3 gets 1 hour because elevated capability is where the damage is; ordinary Extensions get 24 hours because P2 is a product promise. The knob is per-risk-class precisely so the trade is made once, explicitly, rather than per incident.

**Invariant K1.** A Recall never deletes Society data. It removes the Extension's authority and presence; everything it created stays with the Society that owns it (P1).

---

## 11. Creator Tooling

P3 and P13 mean the developer portal is not the privileged surface — it is one front end over the same public API the CLI and any third party uses. If a creator can do it in the portal, `fn market` can do it, and vice versa. That is a release gate, not an aspiration.

### 11.1 The publishing flow

```
  fn market init --category plugin            scaffolds manifest + metadata
  fn market manifest check                    static analysis: declared vs inferred (§10.2 ③)
  fn market sandbox create --template dev     creates a disposable sandbox Society
  fn market install --sandbox <soc> --local   installs the local build; full Envelope trace
  fn market package                           builds, SBOMs, signs with the release key
  fn market publish --version 1.2.0 \         submits to the pipeline
      --channel beta|stable
  fn market status <listing>                  review state, queue position, SLO clock
  fn market recall request <version>          creator-initiated withdrawal (self-recall)
  fn market payouts list --since 2026-01      settlement statements
  fn market reviews list --unanswered         review response queue
```

`fn market manifest check` runs the *same* analyzer as the publish pipeline. A creator who is surprised by a rejection at publish time is a tooling failure; the local check and the server check must never disagree in outcome, and their shared implementation is the mechanism that guarantees it.

### 11.2 Sandbox Societies

A **sandbox Society** is a real Society with `visibility = Private`, a `sandbox` flag, synthetic Citizens and Agents, and a Treasury funded with test Fraction from a segregated `SandboxEmission` account whose balance is excluded from circulating supply (P12 — test Fraction must never contaminate the real supply figure). Sandboxes expire after 30 days of inactivity, can be reset to a snapshot, and can replay a captured event trace so a creator can reproduce a bug from a real Society without ever seeing that Society's content.

### 11.3 Creator telemetry — privacy-respecting by construction

| Available to creators | Not available, ever |
|---|---|
| Install / uninstall counts, daily granularity | Any Citizen identifier, Handle, or FNID |
| Version distribution across the install base | Which Society installed it, unless that Society opts in |
| Crash and error rates, with stack traces from the Extension's own frames only | Chamber content, message text, member lists, Vault contents |
| Invocation counts and p50/p95 latency of its own entry points | Any cross-Extension or cross-Society joined view |
| Purchase, refund, and conversion counts | Individual purchase records or buyer identity |
| Ratings and review text (already public) | Behavioral traces, session recordings, funnels |
| Aggregate configuration-option usage | Anything below the k-anonymity threshold |

**k ≥ 25.** Any breakdown, filter, or segment that would resolve to fewer than 25 Societies returns "insufficient data" rather than a number. This is the standard defense against reconstruction by repeated narrow queries, and it must be enforced server-side in the query planner, not in the portal UI. All telemetry is documented and Citizen-inspectable (`02 §4`: no silent telemetry) — a Society can see exactly what each installed Extension reports upward.

**Honest cost.** Creators used to product analytics will find this thin. They cannot build funnels, cannot cohort, cannot see who churned. Some genuinely useful product work becomes harder. The alternative is a surveillance layer inside a sovereignty platform, which is the contradiction P9 exists to prevent.

---

## 12. Society-Hosted Markets

A Society may run a **Shelf**: a curated storefront of Listings it stands behind. This is the platform's primary answer to discovery-without-surveillance (§8) — editorial judgment distributed to communities that have domain expertise, rather than centralized in one ranking team.

```rust
struct Shelf {
    shelf_id:     ShelfId,
    society_id:   SocietyId,          // P1 — a Shelf is owned by exactly one Society
    entries:      Vec<ShelfEntry>,    // curated Listings, ordered by the Society
    standards:    ShelfStandards,     // published: what this Shelf requires beyond platform review
    fee_bps:      u16,                // 0..=1000 (0–10%), set by Charter, capped by R1
    visibility:   Visibility,         // Public shelves are globally discoverable
    curators:     Vec<RoleId>,        // Charter roles permitted to curate — never an Agent alone
}
```

| Rule | Specification |
|---|---|
| Eligibility | Society Level ≥ 3 (Extension installs unlock at 3, `11 §2.3`) to run a Shelf; Level ≥ 4 plus an explicit Charter clause to take a fee |
| Attribution | The Society fee is paid only on sales **originated** by that Shelf — a disclosed referral link or an in-Society install flow. Never on a sale the buyer found elsewhere |
| Standards | A Shelf may impose stricter standards than the platform (audited-only, open-source-only, accessibility-verified). It may **never** lower the platform floor |
| Curation is governance | Adding, removing, or reordering a Shelf entry is a governance event under the Charter, attributable to a role, appealable through the Society's own Appeal process (`01 §7`) |
| Creator consent | A creator can opt out of any Shelf. A Shelf cannot carry a Listing whose creator has objected — curation is not conscription |
| Conflict of interest | A Shelf carrying a Listing whose creator is the hosting Society, or a member holding a curator role, must display that relationship on the entry. Undisclosed self-dealing is a delisting offence for the Shelf |
| Revenue destination | Shelf fees are paid to the **Treasury**, never to an individual curator. Distribution to curators is then a Charter matter, visible to members |

**Governance implication, stated plainly.** A Shelf gives a Society economic power over creators, which creates the incentive to extract. The 10% cap, the origination requirement, the Treasury destination, the creator opt-out, and the appeal path are five independent limits on that power. A Society that abuses its Shelf loses creators, and — because Shelf quality is a §8 ranking input — loses discovery weight platform-wide.

---

## 13. Anti-Abuse

| # | Threat | Mitigation | Residual risk |
|---|---|---|---|
| **T1** | **Clone / typosquat Listings** — copying a popular Listing's name, slug, icon, and description | Confusable normalization on slug and display name (same machinery as Handles, `11 §6`); perceptual hash on icon and screenshots; edit-distance block against the top 5,000 Listings; artifact-hash match to an existing Listing is an automatic reject; trademark claim route (T7) | Semantic clones ("Better X") that are not confusable but trade on reputation. Partially addressed by creator Trust in ranking |
| **T2** | **Fake ratings** — purchased or Sybil reviews | Purchaser-gate + 72h hold + Trust weighting + velocity freeze (§9); paid-review offers are Trust-slashing for both sides | Aged, high-Trust accounts operated at cost. Rating is one of six ranking terms by design |
| **T3** | **Capability creep** — v2 quietly requests a wider Envelope | **Invariant M3**: any capability delta blocks auto-update, exits the fast path, requires human review and per-Society re-consent showing the diff | Semantic creep *within* an already-granted capability (an Extension granted `chamber.read` for search that starts training on content). Addressed by declared `data_flows` and audit, not by the capability system alone. **This is the hardest unsolved problem in the chapter** |
| **T4** | **Malicious update / supply-chain attack** — compromised release key or dependency | Key continuity check (§10.2 ①); key rotation is an explicit consented event, never silent; SBOM diff with new-dependency escalation; reproducible builds where the toolchain allows; R3 requires a Stake that is slashable | A compromised key plus a semantically identical manifest passes the fast path. Mitigated by 20% post-publish audit sampling and by TTL-bounded attestations limiting exposure |
| **T5** | **Refund abuse** — buy, extract value, refund | Runtime/invocation caps per category (§5.3); rolling per-Citizen refund budget; non-revocable categories get a 48h window | Content already materialized cannot be un-had. Accepted; the window is short and the budget is finite |
| **T6** | **Payout fraud** — fabricated sales to launder or farm Fraction from a subsidy | Sales between related identities (shared device keys, funding graph proximity, reciprocal purchase patterns) are excluded from payout and from ranking; the launch rate needs a verified identity; a **detected self-purchase is void, not fee'd** — refused at the saga, and if detected after settlement, reversed in full under R2 including the burn reversal. A 100% platform fee would make invariant 4 (creator ≥ 70% on every settled purchase) false in order to punish, and the invariant is worth more than the punishment (`61 X5`) | Distributed collusion rings. Detection is graph-based, therefore probabilistic; suspected rings are held, not auto-slashed, pending review (§5.5 L3) |
| **T7** | **IP infringement** — stolen code, art, or trademark | Perceptual hashing at submission; a published notice-and-counter-notice process: claim → 48h creator notice → counter-notice window → Quarantine (not deletion) pending resolution; repeat-infringer policy with escalating consequences | Ownership disputes the platform cannot adjudicate. **Escalates to L3 and counsel; the platform is not a court** |
| **T8** | **Manifest lying** — declaring narrow capability, doing more | Static analysis is the source of truth, not the declaration (§10.2 ③); undeclared capability path = hard reject; runtime enforcement means an undeclared call *fails* rather than succeeding silently (`20`) | Capability laundering through a legitimately granted broad capability. Same root as T3 |
| **T9** | **Review-pipeline DoS** — flooding submissions to exhaust human review | Per-creator submission rate limits; a refundable submission bond for R3 first publishes; automated checks run before any human sees anything | Cost shifts to automation capacity, which is cheaper to scale than reviewers |
| **T10** | **Shelf capture** — a Society monetizing curation slots covertly | Curation is a governance event with an attributable role; conflict disclosure required; fee capped at 10% and paid to Treasury only | A Society taking off-platform payment for placement. Detectable only by report; treated as fraud at L3 |

**Flag for counsel:** the notice-and-takedown process (T7), safe-harbour posture, repeat-infringer policy, and the platform's liability for third-party Extensions all require jurisdiction-specific legal review before Phase 6. The mechanism above is designed to be *compatible* with a standard notice regime; it is not a legal opinion that it satisfies one.

---

## 14. Marketplace Health Metrics

What is measured determines what is optimized, so the metric set is chosen to make *capture* visible rather than to make growth look good. Every one of these is published quarterly (P12's transparency discipline applied to the market).

| Metric | Definition | Healthy | Warning | Capture / failure |
|---|---|---|---|---|
| **Creator retention (90d)** | Creators with ≥ 1 publish in a quarter who publish again the next quarter | ≥ 60% | 40–60% | < 40% — creators are not making a living here |
| **Time-to-first-sale** | Median days from first publish to first paid transaction | ≤ 21 days | 21–60 | > 60 — discovery is broken, not supply |
| **Earnings Gini** | Gini coefficient of creator earnings over trailing 90 days | ≤ 0.75 | 0.75–0.85 | > 0.85 — winner-take-all capture; the long tail is decorative |
| **Top-10 revenue share** | Share of gross taken by the top 10 creators | ≤ 25% | 25–40% | > 40% |
| **Refund rate** | Refunded gross ÷ gross | 2–6% | 6–10% | > 10% quality failure; **< 0.5% is also a warning** — it means refunds are too hard to obtain |
| **Review latency** | p50 / p95 by tier (§10.3) | Within SLO | p95 breach 2 consecutive weeks | Chronic breach — the pipeline is the bottleneck on supply |
| **Review integrity** | Verified-purchase ratio (100% by construction) + brigade-freeze rate | Freeze < 0.5% of Listings/quarter | 0.5–2% | > 2% — rating system is contested |
| **Recall rate** | Recalls per 1,000 published versions | < 1 | 1–3 | > 3 — the pipeline is not catching what it should |
| **Fast-path defect escape** | Defects found in post-publish audit ÷ fast-path publishes | < 1% | 1–3% | > 3% — narrow the fast path |
| **Payout reliability** | Settlements executed on schedule, in full | 100% | any miss | any repeated miss is an existential trust event |
| **Free-to-paid ratio** | Free Listings ÷ all Listings | 40–70% | — | < 30% suggests a paywalled ecosystem; > 85% suggests nobody can monetize |

The pairing of **Gini** with **time-to-first-sale** is the important one: a marketplace can post excellent gross revenue while being a closed shop for five incumbents. Concentration is the failure mode that looks like success on every other dashboard, which is why it is a published number rather than an internal one.

---

## 15. Trade-offs, Failure Modes, and Rejected Alternatives

### 15.1 Rejected alternatives

| Alternative | Why it is attractive | Why rejected |
|---|---|---|
| **No marketplace at all** (WordPress.org model: free plugins, monetize off-platform) | Zero commerce surface, zero payments, zero dispute machinery, no regulatory exposure. Genuinely the cheapest option | Pushes value capture off-platform, where we can enforce no capability disclosure, no review, no recall, and no refund. It also concedes the core thesis: P12 requires a real internal economy with real Sinks, and marketplace fees are the largest Sink available (§6.2). Free-only is what Phases 1–5 actually do; the rejection is of it as the *end state* |
| **App-store model: 30% take, closed curation, single ranking team** | Proven revenue. Simple. Buyers understand it | 30% is monopoly-rent pricing we have no leverage to charge and no cost basis to justify (§6.3). Closed curation concentrates the exact discretion §8 is built to decentralize. Both would make us a worse version of an incumbent |
| **Fully permissionless: publish anything, no review** | Maximum creator velocity; ideologically consistent with sovereignty | Directly contradicts P8. A Society installing an Extension grants it an Envelope over its Chambers, Vault, and possibly Treasury. "Caveat emptor" is not a security model when the buyer cannot read WASM. `10 §12` already commits to delaying third-party execution rather than weakening isolation |
| **External payment processors only** (Stripe et al., no FRC) | Solves fiat immediately; someone else owns compliance and chargebacks | Contradicts P12 and `02 §3` — FRC is the unit of account, and marketplace volume is a designed Sink. It also imports the chargeback model into a ledger that cannot reverse a Posting, creating a permanent impedance mismatch. The `Rail` port keeps this available as an *additional* path at Phase 9+, not as the foundation |
| **Licenses as Facets, universally** | Elegant; one asset primitive for everything; instantly tradeable | P1 violation for Personal-scope licenses; wrong defaults on transferability and evolution; heavyweight for an authorization check that must work offline. See §4.3 |
| **Auction-based placement with revenue to Societies** | Would fund Societies and monetize discovery | It is advertising. `02 §4`, Never List |

### 15.2 Failure modes we expect and how we would recognize them

| Failure mode | Early signal | Planned response |
|---|---|---|
| **Ghost town** — Listings exist, nobody installs | Install-per-Listing median → 0; time-to-first-sale > 60d | The problem is demand, not supply: stop recruiting creators, invest in Shelves and first-party exemplars |
| **Concentration capture** — five creators own the market | Gini > 0.85, top-10 share > 40% | Fee schedule already favours small creators; add Shelf rotation guarantees; do **not** add discretionary promotion, which is the reflex and is banned |
| **Review pipeline becomes the bottleneck** | p95 latency breach sustained; submission queue depth grows monotonically | Widen the fast path only where the escape rate justifies it; hire reviewers before loosening R3; last resort is a longer SLO, honestly published — never a silent quality reduction |
| **Trust collapse after an incident** | Install rate drop across *all* third-party Listings after one Recall | Public incident report within 7 days, full event-log reconstruction for affected Societies, tighten the class of capability that caused it. Recovery is procedural, not communicative |
| **Creator exodus over the fiat gap** | Creator retention < 40% with high satisfaction on everything else | This is the predicted one. There is no fix inside this document: the answer is Phase 9 or an honest statement that it is not coming. Do not paper over it with promises |
| **Fee creep** | Any proposal to raise the take rate without a published cost basis | The 70% creator floor (R1) is an invariant in code. Changing it requires an ADR and 90 days notice on future purchases only |

---

## 16. Invariants (test suite)

These join the list in `11 §7`. Each becomes a property test.

1. No Listing reaches `Published` without a rendered, verified `CapabilityManifest` (M1).
2. A runtime Envelope is always a subset of the installed version's declared manifest (M2).
3. A version whose manifest adds any capability, network target, or data flow never auto-applies to an existing Install (M3).
4. `platform + society + affiliate ≤ 30%` of gross; creator receives ≥ 70% on every settled purchase (R1).
5. Every settled purchase produces a balanced Posting set; refunds reverse every leg including the burn (R2).
6. Every `LicenseGrant` references exactly one Posting or is an attributed promotional grant (L1).
7. License terms are snapshotted at purchase and never change retroactively (L2).
8. A published `ListingVersion.artifact` hash is immutable; a `Recalled` version never returns to `Published`.
9. No ranking input outside the allowed set in §8.1 appears in the scoring function.
10. No review exists without an `Active` or `Refunded` LicenseGrant for that (Citizen, Listing).
11. A Recall revokes authority and never deletes Society-owned data (K1).
12. Sandbox Fraction never appears in circulating supply.
13. Creator telemetry queries return no segment resolving to fewer than 25 Societies.
14. Society Shelf fee ≤ 1000 bps and is paid only to a Treasury on an originated sale.
15. Every marketplace `PostingReason` maps to a Source or Sink declared in `17`.

---

## 17. Phase Placement

| Capability | Phase | Gate |
|---|---|---|
| Listing model, capability manifest, review pipeline R0–R2, recall, ratings, discovery, free third-party Extensions | 4–5 | Requires `20` sandbox and the Envelope system adversarially tested |
| Society Shelves (curation, no fee) | 5 | Society Level 3 exists |
| **Paid Listings, licensing, purchase saga, revenue share, payouts, escrow, disputes, R3** | **6** | `02 §3` — third-party paid Extensions and marketplace payments |
| Shelf fees, affiliate/curator share | 6 | After paid commerce is stable for one phase |
| C9 Services with escrow | 6 | Dispute ladder operational |
| R4 / Experience Runtime Listings | 7 | `20`, Experience Runtime |
| Fiat off-ramp for payouts | **9+** | Licensed entity, KYC/AML, tax, sanctions — **counsel-gated, no exceptions** |

Nothing in §4–§7 ships before Phase 6. Everything in §3, §8, §9, §10 ships when the first third-party Extension does, free or not — because a free Extension holds an Envelope, and an Envelope is what actually needs governing.
