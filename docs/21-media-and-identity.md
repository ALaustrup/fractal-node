# 21 — Media and Identity

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `11-domain-model.md`, `12-identity-and-trust.md`, `13-data-and-storage.md`, `32-design-system.md`.
> **Governs:** the Profile as a composed surface, the Persona presentation layer, Gallery Chambers and personal Collections, the media experience layer, Vault sharing and versioning UX, media search, and the integration points where media and identity meet the economy and the permission system.
> **Does not govern:** the storage substrate — chunking, encryption order, erasure coding, Custodian protocol, attestation, settlement arithmetic and the transcode ladder are `13`. Progression mechanics — XP formulas, Level curves, Unlock gates, Trust bands are `18`. The Facet Standard is `16`. Marketplace listings, pricing and payouts are `19`. This chapter consumes all four and re-specifies none of them.

---

## 1. Thesis: the Profile Is a Home, Not a Settings Page

Every social product eventually decides what a profile is *for*. The two common answers are both failures. The first treats the profile as a **record** — a name, an avatar, a follower count, some counters — which makes it a settings page with a public URL and gives a person no reason to return to their own. The second treats it as a **stage** — an engagement surface measured by views, optimized for reach — which converts the one place a Citizen owns into a place they perform.

Fractal Node takes the third answer. A Profile is a **first-class digital home**: a composed, persistent, sovereign space that a Citizen arranges, keeps things in, works out of, and receives people into. Concretely, and this is a checklist rather than a metaphor, it means all seven of the following are true:

1. **It is composed, not filled in.** The Citizen chooses which Modules exist, what they contain, where they sit, and who sees each one. There is no fixed field list.
2. **It holds real objects.** Galleries, documents, writing, code, Facets and Insignia are the actual Vault Objects (`11 §2.7`), rendered in place, not links to somewhere else.
3. **It is a place of work.** Files are shared from it, versions are reviewed in it, media is pinned for offline from it, Fraction is received into it. It is a surface with verbs, not just nouns.
4. **It is addressable and portable.** `FN://@kaya` resolves on every front end including the CLI (P13), renders offline from the local replica (P2), and exports completely (P9).
5. **It is context-aware.** The same Citizen presents a different Persona in each Society without becoming a different person (§2).
6. **It is disclosure-controlled per Module.** Visibility is not one global switch; it is a property of each thing the Citizen chose to put there, defaulting to the most private setting that leaves the Module meaningful (P9).
7. **It is quiet.** The economy runs through it and never shouts at it (§10.4).

**What it must never become**, stated so that a reviewer can cite a line number when refusing a proposal:

| Anti-goal | What it looks like in a PR | Why it is refused |
|---|---|---|
| **A vanity metrics dashboard** | Profile view counts, gallery view counts, follower totals, "trending" ordering, a visible earnings ticker | Requires persisting behavioural observation P9 forbids; converts a home into a performance; manufactures engagement rather than value (P12). There is no view counter anywhere in this chapter, by construction rather than by policy. |
| **A CSS free-for-all** | Arbitrary stylesheets, injected HTML, per-Profile fonts, absolute positioning, third-party embeds | Destroys every accessibility guarantee (N8), every performance budget (`32 §8`), and the design system's ability to make ten thousand Profiles feel like one product. It also imports XSS and tracking-pixel surface into the most-visited page in the product. |
| **An ad surface** | Sponsored Modules, paid placement in discovery, an ad-supported free media tier | On the Never list (`02 §4`). Advertising is refused as a funding model, not merely as a feature. |

> **Invariant M0.** No surface specified in this chapter persists, displays, or ranks by a count of views, impressions, dwell, or scroll depth. Absence is the mechanism; there is no setting to turn it off because there is nothing to turn off.

---

## 2. The Identity Layer: Citizen ↔ Persona ↔ Profile

### 2.1 Three objects, three jobs

Canon already names all three (`01 §2`). Their separation is what makes contextual presentation possible without fragmenting accountability.

```
   CITIZEN                 the person. One FNID, one Handle, one XP,
   (global)                one Trust, one global Wallet, one Vault.
      │                    Immutable identity. (11 §2.1)
      │
      │  presents as
      ▼
   PERSONA                 one per Society (01 §2). Display name, avatar,
   (per Society)           pronunciation, role chips, and a set of
      │                    Module overrides. Presentation only.
      │
      │  arranges
      ▼
   PROFILE                 the composed home surface. ONE per Citizen,
   (global, one)           with per-Society overrides supplied by Personas.
                           Not N documents — one document, N views.
```

**Why one Profile with overrides rather than N Profiles.** Two documents drift. A Citizen maintaining a Profile per Society maintains none of them after the third. The Profile is a single `ProfileLayout` (§3.3); a Persona carries a bounded `PersonaOverride` — display name, avatar, accent, and a per-Module `disclosure` and `hidden` flag. A Persona may hide a Module, re-scope who sees it, and rename the Citizen; it may not add a Module the Profile does not contain, and it may not widen a Module's disclosure beyond what the Profile sets. Overrides narrow. This asymmetry is what stops a Society from becoming a place where a Citizen's identity is more exposed than they chose.

```rust
struct Profile {
    profile_id: ProfileId,
    citizen:    Fnid,                       // 1:1 with Citizen, never reassigned
    layout:     ProfileLayout,              // §3.3
    cover:      Option<CoverRef>,
    sigil:      SigilVariant,               // procedural by default; re-roll at L9 (18 §5.3)
    default_disclosure: Disclosure,         // applied to any Module that declares none
    revision:   u32,
}

struct PersonaOverride {
    society_id:   SocietyId,
    display_name: Option<String>,           // Unicode; the Handle is never overridable
    avatar:       Option<ObjectRef>,
    accent:       Option<AccentStop>,       // must be the Society's accent or the Citizen's
    module_view:  BTreeMap<SlotId, ModuleView>,
}

struct ModuleView { hidden: bool, disclosure: Option<Disclosure> }  // narrowing only
```

### 2.2 Pseudonymity: what is linkable, what is not, and the honest statement

`12 §10.2` settles the underlying question and this chapter does not reopen it: **one FNID everywhere, Personas change presentation and not identity.** Cross-Society linkage is therefore possible for anyone who can see two memberships. The Profile is the surface where that fact becomes visible, so it is the surface that must state it accurately.

| Fact | Always linkable | Under Citizen control | Never built |
|---|---|---|---|
| FNID across Societies | **Yes** — same key, same Citizen | — | — |
| Handle | **Yes** — global, unique, immutable after 14 days | — | — |
| Level | **Yes** — exact, to everyone (`18 §10.4`) | — | — |
| Trust | Band only (`ESTABLISHED`/`NEUTRAL`/`RESTRICTED`) | — | Numeric Trust is never public |
| Which Societies a Citizen is in | No | Per Society, **default hidden** (`12 §10.2`) | — |
| Standing in a Society | No | Per Society disclosure setting | — |
| Insignia and Achievements | No | Default: shared Societies only | — |
| Gallery contents, writing, code, Collections | No | Per Module disclosure | — |
| Cross-Society activity graph | — | — | **Not built.** No endpoint returns it; no projection computes it |
| What a Citizen viewed, when, for how long | — | — | **Not persisted** (M0, `13 V10`) |

**The honest privacy statement**, rendered verbatim in the Profile's disclosure surface and in `fn me disclosure`:

> Your Handle and Level are public. Your Trust is shown only as a band. Everything else on this Profile is shown to exactly the audience you chose per Module, and you can enumerate and revoke every disclosure you have ever made. Two people who can both see your membership in two Societies can tell it is the same you — that is the cost of one accountable identity, and we do not pretend otherwise. If you need presentations that cannot be linked, create a second Citizen. We neither forbid it nor detect it; it starts at Level 0 with no Trust, and that is the whole price.

That paragraph is product copy and it is also the design constraint. A privacy claim we cannot enforce is not shipped as UI text.

---

## 3. The Profile Composition Model

### 3.1 The grid

A Profile is a **12-column grid of fixed-height rows**, not a canvas. Modules occupy whole cells from a closed set of footprints. There is no overlap, no z-order, no absolute positioning, and no free-form drag to arbitrary pixels.

```
  standard (1440–1919)      12 columns · gutter space-5 (16px) · row unit 96px
  compact  (1101–1439)       8 columns · same gutter and row unit
  handheld (≤1100)           4 columns · modules reflow, never shrink below W4
  wide/ultra (≥1920)        12 columns; the grid gains max-width, not more columns

  FOOTPRINTS (closed set — a Module declares which of these it supports)
    W4·H1   W4·H2   W6·H1   W6·H2   W8·H2   W12·H1   W12·H2   W12·H3
```

**Layout constraints, all enforced at save time:**

| # | Constraint | Why |
|---|---|---|
| L1 | Cells may not overlap; a Module occupies a contiguous rectangle | Overlap has no meaning at `handheld` and no accessible reading order |
| L2 | No row may be partially empty at `standard` except the last | Prevents the ragged, half-broken look that reads as unfinished rather than sparse |
| L3 | ≤ 12 Modules total, regardless of Level | Complexity budget (`02 §7`). A 40-Module Profile is not expressive, it is unreadable |
| L4 | Module *order* is authored once and is the reflow order | One authored layout, deterministic on every breakpoint. There is no second mobile layout to maintain and therefore none to rot |
| L5 | At `handheld` every Module becomes W4 and stacks in order | A phone gets the same content in the same sequence, never a different Profile |
| L6 | Slot count is Level-gated: 3 at L2, 6 at L5, free ordering at L7, 12 at L10 (`18 §5.3`) | Customization capability is earned; capacity is not purchasable here (`18 §5.3`) |
| L7 | Exactly one Module may declare `hero: true` and it must sit in row 1 | A single focal point; two heroes is zero heroes |

Reflow is deterministic and derived, not authored: at `compact`, a W12 stays W12 (of 8 columns), a W6 stays W6, a W8 becomes W8-of-8, and a W4 becomes W4-of-8 — Modules never reorder, only re-measure. This is the entire argument against free positioning: a layout with overlap and z-order cannot reflow deterministically, which forces either a second authored mobile layout (which nobody maintains) or a broken phone experience (which everybody ships).

### 3.2 The Module manifest

Every Module — first-party and Extension-supplied — is described by the same manifest. First-party Modules use the identical registration path an Extension uses (P7); `fn.standing-explainer` in `20 §2` is already an example.

```rust
struct ModuleManifest {
    kind:          ModuleKind,               // closed enum + Extension(InstallId, Name)
    footprints:    &'static [Footprint],     // which of the eight it supports
    hero_capable:  bool,
    unlock:        Gate,                     // 18 §5 gate grammar, verbatim
    requires:      &'static [Requirement],   // Collection | ObjectRef | Connector | Wallet | …
    reads:         &'static [DataScope],     // exactly what it may read; nothing ambient
    disclosure_default: Disclosure,          // the most private setting that still works (P9)
    surface:       SurfaceDescriptorRef,     // 20 §7 — design-system primitives only
    offline:       OfflineBehavior,          // Full | LastKnownGood | Unavailable  (P2)
    budget:        RenderBudget,             // nodes, bytes, and a wall-clock ceiling
}

enum Disclosure { Nobody, Society(SocietyId), SharedSocieties, Everyone }
```

`reads` is the load-bearing field. A Module is handed a **narrowed context** containing exactly the scopes it declared, never the Profile aggregate. A `gallery` Module that declared `Collection(id)` cannot see the Citizen's Wallet, memberships, or other Collections — not because it is asked not to, but because those values are absent from its input (`20 §6`, the same absent-not-stubbed rule).

### 3.3 `ProfileLayout`

```rust
struct ProfileLayout {
    profile_id: ProfileId,
    theme:      ThemeRef,            // validated set only (§4)
    accent:     AccentStop,          // 1 of the 12-stop wheel (32 §9)
    density:    Density,             // Comfortable | Compact | Dense — gated at L3
    modules:    Vec<ModulePlacement>,// ≤ 12; index IS the reflow order (L4)
    validated:  ValidationStamp,     // §4.4 — no stamp, no render
    revision:   u32,
}

struct ModulePlacement {
    slot_id:    Ulid,
    kind:       ModuleKind,
    footprint:  Footprint,           // must be in manifest.footprints
    origin:     GridCell,            // (col, row), snapped, validated against L1/L2
    config:     ModuleConfig,        // schema-validated per kind
    disclosure: Disclosure,
    hero:       bool,
}

struct ValidationStamp {
    validator_version: u16,
    contrast_pass:     bool,
    motion_pass:       bool,
    budget_pass:       bool,
    computed_scrim:    f32,          // derived, never authored (§4.4)
    at:                Timestamp,
    sig:               Signature,    // Runtime-signed; clients refuse an unsigned layout
}
```

### 3.4 The first-party Module catalog

Twenty-one Module kinds ship first-party. Every one uses `ui.profile.module` (`20 §5`, hook 35). Gates are expressed in the `18 §5` gate grammar.

| # | `ModuleKind` | Shows | Requires | Unlock gate | Default disclosure | Footprints |
|---|---|---|---|---|---|---|
| 1 | `bio` | Free text (Manrope, ≤ 600 chars), pronunciation, one line of role chips | — | `Level ≥ 0` | `Everyone` | W6·H1, W12·H1 |
| 2 | `gallery` | A Collection rendered as a grid, with cover, count, and ordering mode | `Collection` in the Citizen Vault or a granted Society Vault path | `Level ≥ 2` | `SharedSocieties` | W6·H2, W12·H2, W12·H3 |
| 3 | `pinned_media` | Up to 8 hand-chosen Objects, any media kind, version-pinned | `[ObjectRef]` | `Level ≥ 2` | `SharedSocieties` | W6·H1, W12·H1 |
| 4 | `now_playing` | Current or recent listening, as **data**: title, artist, art re-hosted as a Vault Object | A Citizen-authorized, revocable, read-only `Connector` with an egress allowlist entry | `Level ≥ 5` | `SharedSocieties` | W4·H1 |
| 5 | `shelf` | Transferable Collectible Facets, with evolution state and provenance depth (`32 §5.5` `FacetTile`) | Held Facets | `Level ≥ 5` (hold/transfer Facets, `18 §5.1`) | `SharedSocieties` | W4·H2, W6·H2 |
| 6 | `insignia_case` | Insignia and Badges, current tiers, earned dates | Earned Insignia | `Level ≥ 0` | `SharedSocieties` | W4·H2, W6·H2 |
| 7 | `memberships` | Societies, sigils, roles, tenure — one row per Society, each independently disclosable | Active Memberships | `Level ≥ 0` | **`Nobody`** (`12 §10.2`) | W4·H2, W6·H1 |
| 8 | `contribution` | The Citizen's own Contribution readout: awarded, withheld, escrowed, per Source, per window | Own progression record | `Level ≥ 0` | **`Nobody`** | W6·H2, W12·H2 |
| 9 | `custody` | Bytes held, bytes served, attestations passed, FRC accrued this window (`32 §5.5` `CustodianPanel`) | Custodian eligibility | `Level ≥ 8 ∧ Trust ≥ 200 ∧ Stake(500)` (`18 §5.5`) | `SharedSocieties` | W4·H2, W6·H2 |
| 10 | `agents` | Enrolled Agents: kind, Operator, live Envelope summary, TTL, revoke control | ≥ 1 enrolled Agent | `Level ≥ 4` | `SharedSocieties` | W6·H2 |
| 11 | `guestbook` | Signed entries from other Citizens, review-queued before render | Moderation queue (Phase 3) | `Level ≥ 3` | `SharedSocieties`, entries **held for approval** | W6·H2, W12·H2 |
| 12 | `writing` | Long-form Objects with an in-place reader, ordered manually or by publication date | Vault Objects of a text kind | `Level ≥ 2` | `Everyone` | W8·H2, W12·H2 |
| 13 | `code` | Repository-shaped Objects: tree, file view, syntax highlighting via `CodeBlock`, no execution | Vault Objects; a `Collection` for a tree | `Level ≥ 3` | `SharedSocieties` | W6·H2, W12·H2 |
| 14 | `links` | Outbound links, ≤ 8, `rel="noopener noreferrer"`, no favicons fetched from third parties | — | `Level ≥ 0` | `Everyone` | W4·H1 |
| 15 | `availability` | Declared availability windows and contact posture — "open to Convergence", "not taking work" | — | `Level ≥ 2` | `SharedSocieties` | W4·H1 |
| 16 | `lineage` | The Citizen's Society lineage graph: Crystallizations witnessed, Fractures survived, Forks founded | Lineage records (`11 §2.2`) | `Level ≥ 3` | `SharedSocieties` | W6·H2 |
| 17 | `season` | The Season Chronicle: objectives met, Season accent, permanent record (`18 §8`) | A closed Season | `Season participation` | `SharedSocieties` | W4·H1, W6·H1 |
| 18 | `receive` | A transfer target: Handle, FNID prefix, Trust band, and the host-rendered `TransferSheet` entry point | Global Wallet | `Level ≥ 0` (receiving is ungated) | `Everyone` | W4·H1 |
| 19 | `storefront` | The Citizen's marketplace Listings, rendered as `ListingCard`s; purchase chrome is host-rendered | Published Listings (`19`) | `Level ≥ 7` | `Everyone` | W6·H2, W12·H2 |
| 20 | `signal` | The Citizen's own recent public domain events, chronological, no ranking | — | `Level ≥ 0` | `SharedSocieties` | W4·H2, W6·H2 |
| 21 | `extension` | Whatever a third-party Extension's `ui.profile.module` hook returns, as a Surface Descriptor | An installed Extension with the hook and a granted Envelope | `Level ≥ 7` (install Extensions) | `Nobody` until the Citizen sets it | W4·H1, W4·H2, W6·H2 |

Three catalog rules worth stating. **First**, `contribution` and `memberships` default to `Nobody` — the two Modules most tempting to make public are the two whose exposure is least reversible, so the default is the private one and the Citizen opts out of privacy rather than into it (P9). **Second**, `now_playing` is data, not an embed: there is no third-party iframe, the metadata is fetched through the Runtime's egress allowlist, and the album art becomes a Vault Object so the Module does not 404 and does not beacon (§15). **Third**, `custody` is the only Module gated on Trust and Stake rather than Level, because it displays a claim about infrastructure reliability and an unearned claim there is worse than no claim.

### 3.5 A Profile at `standard`

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ⌁ FN // @kaya                   ⌘K            ● LIVE    1,204 FRC     @kaya  │ 44
├──────────────────────────────────────────────────────────────────────────────┤
│ ░░░░░░░░░░░░░░░░░░░░░░░  COVER 16:5  ·  scrim computed at save  ░░░░░░░░░░░░ │
│ ░░                                                                        ░░ │
│ ░░  ◈   KAYA OKONKWO                                     [ 07 / PROFILE ] ░░ │
│ ░░      @kaya · fn1qz4…8h2c · L7 · TRUST ESTABLISHED                      ░░ │
│ ░░      ▌ archivist  ▌ custodian                                          ░░ │
│ └────────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   1    2    3    4    5    6    7    8    9   10   11   12   ← 12 columns    │
│ ┌───────────────────────────────┐┌───────────────────────────────┐           │
│ │ [ 01 / BIO ]           W6·H1  ││ [ 02 / PINNED ]        W6·H1  │           │
│ │ Builds archives. Custodies    ││ ┌────┐┌────┐┌────┐┌────┐      │           │
│ │ 4 TiB. Writes on lineage.     ││ │▓▓▓▓││▓▓▓▓││▓▓▓▓││▓▓▓▓│      │           │
│ └───────────────────────────────┘└───────────────────────────────┘           │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ [ 03 / GALLERY ]  field-notes · 214 objects · SHARED SOCIETIES   W12·H2  │ │
│ │ ┌───────┐┌───────┐┌───────┐┌───────┐┌───────┐┌───────┐┌───────┐┌───────┐ │ │
│ │ │▓▓▓▓▓▓▓││▓▓▓▓▓▓▓││░░░░░░░││▓▓▓▓▓▓▓││▓▓▓▓▓▓▓││░░░░░░░││▓▓▓▓▓▓▓││▓▓▓▓▓▓▓│ │ │
│ │ └───────┘└───────┘└───────┘└───────┘└───────┘└───────┘└───────┘└───────┘ │ │
│ │   ░ = blurhash placeholder, resolved from the Manifest before any fetch   │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│ ┌───────────────────┐┌───────────────────┐┌───────────────────────────────┐  │
│ │ [ 04 / INSIGNIA ] ││ [ 05 / CUSTODY ]  ││ [ 06 / RECEIVE ]       W4·H2  │  │
│ │ ◈ TEN_NINES  t.3  ││ HELD    4.11 TiB  ││ @kaya · fn1qz4…8h2c           │  │
│ │ ◈ ARCHIVIST  t.2  ││ SERVED  812 GiB   ││ TRUST  ESTABLISHED            │  │
│ │ ◈ GENESIS    —    ││ ATTEST  1,204 OK  ││ ┌───────────────────────────┐ │  │
│ │       W4·H2       ││ ACCRUED 12.400    ││ │  TRANSFER FRACTION     ►  │ │  │
│ │                   ││   PENDING  W4·H2  ││ └───────────────────────────┘ │  │
│ └───────────────────┘└───────────────────┘└───────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│ FN://@KAYA · SYNCED 2s AGO · 12 MODULES · 3 HIDDEN TO YOU · LAYOUT r41       │ 24
└──────────────────────────────────────────────────────────────────────────────┘
```

Two details in that wireframe are deliberate. The status bar reports `3 HIDDEN TO YOU` — a viewer is told that Modules exist which they cannot see, because a Profile that silently omits things teaches viewers to distrust what they do see. And the `custody` Module shows `ACCRUED 12.400 PENDING` in mono, electric, tabular, static: that is the entire visual footprint of the economy on a Profile (§10.4).

---

## 4. The Customization Contract

This section is the hard line. It is the single most-argued and most-important decision in the chapter.

### 4.1 What a Citizen may change

| Axis | Range | Gate |
|---|---|---|
| Theme | `void`, `daylight`, `contrast`, plus any installed Theme Extension that passed install-time validation (`32 §9`) | L0 / L3 / L12 per `18 §5.3` |
| Accent | One stop from the curated 12-stop wheel at matched chroma | L5 |
| Density | Comfortable / Compact / Dense | L3 |
| Module selection | Which of the 21 kinds exist, and their config | L2 (3), L5 (6), L10 (12) |
| Module arrangement | Footprint from the declared set, origin cell, order | Fixed order below L7; free ordering at L7 |
| Per-Module disclosure | The four-value `Disclosure` enum, narrowable per Persona | L0 |
| Cover media | One image or ≤ 10s silent loop, 16:5, from the Vault | L2 |
| Avatar | Procedural at L0; uploaded at L2 | L0 / L2 |
| Sigil variant | Deterministic re-roll of the procedural seed, 3 lifetime | L9 |

### 4.2 What a Citizen may never change

Typography family and scale. Spacing scale. Radius. Contrast ratios and the AA floor. The semantic accent assignments — **`--fn-accent-agent` violet is permanent and may not be reassigned by any theme, Extension, or Profile**. Component structure and anatomy. Motion durations and easings. The status bar. Focus indicators. The grid itself. Any injection of HTML, CSS, JavaScript, fonts, or third-party network origins.

This is `32 §9`'s theming contract applied to Profiles without exception: *a theme may override the field and accent colors and one background texture; it may not override semantic assignments, contrast, spacing, radius, type scale, motion, or component structure.*

### 4.3 Why the line is here and not somewhere else

The argument for arbitrary CSS is that expressiveness is the point of a home, and constraint is paternalism. The argument is wrong on the facts, and the reason is worth stating precisely rather than asserting.

**What arbitrary CSS actually buys is variance, not expression.** Look at what people actually did with unbounded profile styling in the products that allowed it: a small number of skilled authors made a handful of genuinely striking pages, and everyone else produced something unreadable, or copy-pasted one of six templates, or produced a page that was fine on the author's monitor and broken everywhere else. The distribution of outcomes was bimodal and the mode was "worse."

**What it costs is every guarantee this product makes.** Contrast becomes unverifiable, so N8 becomes aspirational. Payload becomes unbounded, so `32 §8`'s budgets become suggestions. Layout becomes non-reflowable, so the phone experience is a second thing to maintain. Arbitrary origins become reachable, so a Profile becomes a tracking beacon on a platform whose premise is P9. Arbitrary markup becomes renderable, so a Profile can paint a convincing transfer confirmation — the exact spoofing primitive `20 §7` refuses for Extensions and which we would be reintroducing on the most-visited page in the product. And the design system stops being a system, which means the ten-thousandth Profile no longer looks like it belongs to the same platform as the first.

**What the constrained model delivers instead** is the part people actually wanted: *content* (which Modules, holding what), *arrangement* (where, how big, in what order), *palette* (theme plus accent), and *identity* (cover, avatar, sigil). Those four axes multiply to a very large space — 21 Module kinds, 8 footprints, 12 accents, 3+ themes, arbitrary content — while every point in that space is accessible, fast, reflowable, and recognizably Fractal Node. The constraint is not a tax on expression; it is the thing that makes a stranger's Profile legible at a glance and yours distinctly yours.

**The honest cost.** Some layouts are genuinely impossible: a full-bleed asymmetric photographic composition, a hand-set typographic essay, a page whose entire concept is a broken grid. Those Citizens are correctly frustrated, and the answer is not an escape hatch. It is (a) grow the Module catalog and the footprint set through `32-design-system.md` when a real need is demonstrated, first-party and third-party at the same moment (P7), and (b) route genuinely custom rendering to an Experience (`20 §12`, Phase 7), where the sandbox is stronger and the Citizen deliberately entered. There is no third path.

### 4.4 Save-time validation

A layout is not saved until it passes. The validator is one implementation in the Runtime, called identically by the GUI and the CLI (P3, P13).

```
  SAVE ProfileLayout
    │
    ├─ 1. STRUCTURE   footprints ∈ manifest set · L1 no overlap · L2 no gaps
    │                 L3 ≤12 modules · L6 slot count ≤ Level grant · L7 one hero
    │                 config schema-valid per kind · disclosure ⊆ Profile default
    │
    ├─ 2. TOKENS      resolve theme × accent × density to the full semantic set
    │                 reject any Extension theme lacking a valid install stamp
    │
    ├─ 3. CONTRAST    for every text/background pair REACHABLE in this layout:
    │                 body text ≥ 7:1 (AAA, default theme) · UI text ≥ 4.5:1
    │                 non-text indicators ≥ 3:1 · focus ring ≥ 3:1 vs both sides
    │                 cover overlay: mean + p95 luminance sampled over the
    │                 title's bounding box → SOLVE for scrim opacity that
    │                 guarantees 7:1. The scrim is COMPUTED, never authored.
    │                 FAIL → reject, naming the failing pair and its ratio
    │
    ├─ 4. MOTION      cover loop ≤10s · no flash >3Hz (WCAG 2.3.1) ·
    │                 a static frame is derived and is what reduced-motion serves
    │
    ├─ 5. BUDGET      cover ≤400KB delivered · Surface Descriptor ≤128KB/500 nodes
    │                 (20 §7) · estimated route payload ≤60KB gzip (32 §8)
    │
    └─ 6. STAMP       ValidationStamp signed by the Runtime, pinned to
                      validator_version. Clients REFUSE to render an unstamped
                      or version-stale layout and fall back to the default
                      layout with a visible notice — never to unvalidated output.
```

Rejection copy follows `33 §7.3` — cause then remedy, no apology: `Layout refused — accent "ember" on surface-2 measures 3.9:1 for body text; 7:1 required. Choose another accent, or switch this Module to a raised surface.`

Step 3's cover handling is the case that matters. Every product that lets people put a photo behind their name eventually ships unreadable names. Solving for the scrim rather than offering an opacity slider means the Citizen chooses the picture and the system guarantees the reading — which is the customization contract in one mechanism.

---

## 5. Galleries and Personal Media

### 5.1 Two kinds of Gallery, and a Canon clarification

A **Gallery Chamber** is a Society space (`ChamberKind::Gallery`, `01 §3`): many contributors, access through Charter roles, Objects in the Society Vault, moderation under the Society's policy. It is a shared room.

A **personal Gallery** is a Collection in the Citizen's own Vault, surfaced by a `gallery` Module. It is not a Chamber, because Chambers belong to Societies (P1) and this media belongs to a person.

That raises a Canon question the earlier chapters left implicit: `18 §5.1` grants every Citizen a Vault at Level 0 and `13 §10.4` names "Citizen private Vault keys", but `01 §1` scopes Vault under Society. **The resolution is the one already used for Wallets**: `Wallet` carries `society: Option<SocietyId>`, where `None` denotes the global Citizen wallet (`11 §2.6`). `Vault` takes the identical shape — `society: Option<SocietyId>`, `None` meaning the Citizen Vault, which hangs off Global Registry entry 1 (`Citizen`) exactly as the global Wallet does. No new Global Registry entry, no new escape hatch, one-line change to `11 §2.7`, listed in §17 as a required Canon amendment shipped in the same PR as this chapter.

A Citizen may surface a Society Gallery's Collection on their Profile **by reference**. The grant is re-checked at render time against the viewer, never copied and never cached across viewers. A Module is a window, not a grant (M1).

### 5.2 Collections, albums, ordering, covers

```rust
struct Collection {
    collection_id: CollectionId,
    vault:         VaultId,                  // Citizen Vault or Society Vault
    title:         String,
    items:         Vec<CollectionItem>,      // ordered
    ordering:      Ordering,
    cover:         Option<CoverChoice>,
    acl:           Acl,                      // 13 §10.1 — Collections are ACL'd objects
}

struct CollectionItem { object: ObjectRef, pin: VersionPin, caption: Option<String> }
enum  VersionPin { Floating, Pinned(VersionId) }
enum  Ordering { Manual, CapturedAt, AddedAt, Title }
enum  CoverChoice { Item { object: ObjectRef, crop: NormRect }, FirstItem }
```

An **album** is a Collection with `Ordering::Manual` and an explicit cover — a narrative sequence rather than a bag. The distinction is a config value, not a type, because two types would mean two code paths and a migration the first time somebody wants to reorder a bag.

`VersionPin` matters more than it looks. A Gallery pinned to `Floating` follows the Object's latest Version, so re-editing a photo updates it everywhere. A Gallery pinned to a `VersionId` is a fixed exhibit that a later edit cannot silently rewrite. Default is `Floating` for personal Galleries and `Pinned` for anything referenced by a Facet or a Listing, because an asset whose media can change under the buyer is a different product.

### 5.3 Metadata and EXIF — the GPS question

Ingest splits embedded metadata into three planes, on the client, **before encryption** (`13 §3`).

| Plane | Fields | Disposition |
|---|---|---|
| **Preserved** | Dimensions, orientation, bit depth, ICC profile, HDR transfer function, color primaries | Kept and used for rendering; present in derived renditions. Without these, images render wrong. |
| **Retained-private** | Capture time, camera make/model, lens, exposure, focal length | Stored in the Manifest's encrypted metadata plane. Visible to principals with `READ`. **Never present in derived public renditions.** |
| **Stripped always** | Embedded thumbnails, maker-note blobs, serial numbers, owner/artist name tags, editing-software history | Removed unconditionally. Embedded thumbnails in particular are a real leak: they are frequently a pre-crop, pre-redaction copy of a different image. |
| **Stripped by default** | GPS latitude, longitude, altitude, heading, and any location name field | Removed on the client before encryption. |

**The decision on GPS (P9).** Location is removed by default. On the first upload carrying coordinates, the client shows exactly what it found — including a rendered map of the point — and offers retention **per Collection**, never globally and never as a remembered account-wide preference. Reasoning: a sticky global "keep location" toggle is precisely how home addresses leak, because the setting is made once in a context where it is harmless and then applies in a context where it is not. Per-Collection scoping means the decision is made where the consequence lives. Where coordinates are retained, the Gallery renders a persistent `LOCATION` chip on the item so retention is never invisible, and the coordinates live in the encrypted plane — a viewer without `READ` cannot obtain them from a public rendition under any circumstance, because they were never encoded into one.

The honest cost: a Citizen who wants a map view of their travel photos must opt in per Collection, which is friction on a feature people like. We take the friction. The asymmetry of harm is not close.

### 5.4 The viewer

Full-bleed, dark by default regardless of theme (the surface is the media, not the chrome), with the chrome auto-hiding after 2s of pointer inactivity and returning instantly on any keyboard event.

| Key | Action | Key | Action |
|---|---|---|---|
| `←` `→` | Previous / next item | `Space` | Play / pause (AV) |
| `Home` `End` | First / last | `,` `.` | Frame step back / forward |
| `+` `−` `0` `1` | Zoom in / out / fit / 100% | `J` `L` | −10s / +10s |
| `F` | Full-bleed toggle | `M` | Mute |
| `I` | Info panel (metadata, versions, who can see this) | `C` | Comments and annotations |
| `P` | Pin for offline (§6.3) | `D` | Download original (only if the right is held) |
| `Esc` | Close, restoring scroll position and focus to the originating tile | | |

The filmstrip is one tab stop with roving `tabindex`; focus is trapped in the viewer and restored precisely on exit. Zoom is pointer-anchored and momentum-free. Color is managed end-to-end: the ICC profile from the Preserved plane is honored, Display P3 is served where the display reports support, and HDR sources are tone-mapped to SDR with a declared curve rather than clipped — a clipped highlight is a silent data loss, and this product does not do silent.

---

## 6. The Media Experience Layer

### 6.1 Verified progressive playback

`13 §5` specifies BLAKE3 tree addressing and verified streaming; this chapter specifies the experience obligation on top of it. **Every byte that reaches a decoder is verified against a hash derived from an independently resolved Manifest** (`13 V2`), at 1 KiB leaf granularity, *while playing* rather than after downloading. A leaf that fails re-fetches from a different Custodian, emits `ReplicaCorrupt`, and — this is the experience requirement — does not stall playback unless the refetch exceeds the buffer, in which case the player shows `VERIFYING` in the status bar rather than an unexplained pause. Verification overhead is budgeted at ≤ 4% of decode wall time; above that it fails CI as a performance regression, not as a correctness one.

### 6.2 Placeholders and prefetch

Every image and video Object carries, inlined in its Manifest (`13 §9.3` already inlines sub-64 KiB thumbnails, so this costs no additional round trip), a ~28-byte blurhash-class gradient signature and its dominant color. The grid paints the gradient before any fetch resolves, so a Gallery has structure and color at first paint and never flashes from grey to image.

Prefetch is bounded and declared, because unbounded prefetch is how a media product quietly becomes expensive for the Citizen and for the mesh:

- The 160 px tier for everything in the viewport plus one screen.
- The next two items in the Collection's ordering, only while the viewer is open.
- Nothing beyond the current item on a metered connection.
- Nothing at all for a Profile the Citizen has not scrolled to.
- **A prefetch never emits a delivery receipt** — it must not, because S2 pays against recipient-signed receipts (`17 §3.2`) and prefetch-generated receipts would be a Fraction-farming primitive that costs the attacker nothing.
- A prefetch is never a view for any purpose, which is trivially true here because views are not recorded at all (M0).

### 6.3 Offline availability (P2)

P2 is not deferrable, so offline media ships in Phase 2 rather than as a later optimization. The canonical verb is already `pin` (`01 §8`), and it is reused rather than duplicated — with an explicit class:

```rust
enum PinClass {
    Custody,   // a Node commits to hold Shards for OTHERS. Attested, paid (S1).
    Offline,   // a Node holds Shards for its OWN Citizen's reading. Unattested, unpaid.
}
```

An Offline pin earns nothing, and the reason is structural rather than stingy: S2 requires a receipt signed by a *distinct paying Principal* with no shared Operator lineage (`17 §3.2`), and you are not distinct from yourself. Offline pins are per-device, budgeted against a Citizen-set cache ceiling, LRU-evicted with pinned items exempt, and surfaced in the status bar as `OFFLINE 4.2 GiB / 20 GiB`. A pinned Collection stays fully browsable, zoomable, and annotatable offline; annotations queue and render at 70% opacity with a pending mark until settled (`32 §6`).

### 6.4 Performance budgets (P10, CI-enforced alongside `32 §8`)

| Metric | Target | Measured by |
|---|---|---|
| Gallery grid first paint, warm local cache | 120 ms p95 | instrumented trace |
| Gallery grid first paint, cold | 600 ms p75 | Lighthouse CI / instrumented |
| Time to first frame, video, warm placement | 400 ms p75 · 800 ms p95 | player instrumentation |
| Time to first frame, cold, including placement resolution | 1 200 ms p75 | player instrumentation |
| Scrub → new frame rendered | 250 ms p95 | frame trace |
| Gallery scroll, 2 000-item virtualized list | no frame > 33 ms; p95 ≤ 16.6 ms | frame trace |
| Image decode | off the main thread, always | lint + trace |
| Verification overhead | ≤ 4% of decode wall time | benchmark |
| Profile route chunk | ≤ 60 KB gzip (`32 §8`) | bundle analyzer |
| Cover payload delivered | ≤ 400 KB | save-time validator |

---

## 7. File Sharing

### 7.1 The sharing model

Sharing grants `Rights` (`13 §10.1`: `READ · LIST · APPEND · WRITE · SHARE · DELETE · TRANSCODE`) to a `Subject`. Five share shapes, one mechanism:

| Shape | Subject | Typical use | Expiry |
|---|---|---|---|
| Direct | `Citizen(Fnid)` | One person | Optional; default none |
| Role | `Role(RoleId)` | "Everyone with the archivist role" — resolves through the Charter, so an amendment re-scopes access without touching the ACL | Optional |
| Society | Society's role set | A whole Society | Optional |
| Link | `ShareLink` | Someone without an account, or a channel we do not control | **Mandatory**, default 7d, max 365d |
| Public | `Public` | Genuinely open publication | Optional |
| Extension | `ExtensionInstall(InstallId)` | A collaboration tool | Bounded by the Envelope's own TTL |

### 7.2 View versus download, and the watermarking decision

`READ` is a single right. "View-only" is a *presentation* of `READ` in which the client hides download affordances and the gateway serves only derived renditions, never the original Version. We are explicit in the UI about what that is worth: **view-only raises the cost of copying from one click to a screen recording; it is a courtesy, not a control.** The share sheet says so in those terms. A product that implies otherwise is making a DRM claim it cannot honor, and once bytes reach a decoder the claim is false.

**Watermarking: decided.** Invisible forensic per-recipient watermarking is **rejected**. Three independent reasons, any one sufficient: it requires generating a unique rendition per recipient, which destroys content-addressed dedupe and the deterministic `derived_version_id` in `13 §9.3` (every recipient's copy becomes a distinct stored object, multiplying storage by the audience size); it requires server-side plaintext, which is impossible for `EndToEnd` content and would make N6 false; and it embeds a covert per-recipient tracking artifact in a file, which is exactly the surveillance posture P9 exists to forbid — the fact that it tracks *leakers* rather than *readers* does not change what it is.

**Visible watermarking is permitted**, opt-in per share, and rendered client-side at view time from the share grant — recipient Handle and timestamp composited over the rendition, never baked into a stored object. It is honest: the recipient can see they are marked. It is also weak, and the UI says so.

### 7.3 Versioning UX

`Object.versions` is append-only and a revert is a new Version, never a truncation (`13 §10.1`). The UX makes that legible rather than merely true.

```
  ┌─ HISTORY ────────────────────────────────────────────────────────────────┐
  │ ▌ v7   @kaya      2026-09-03 11:40   4.2 MB   "final crop"     ● CURRENT │
  │   v6   @tobi      2026-09-02 17:02   4.4 MB   "levels"                   │
  │   v5   @kaya      2026-09-01 09:14   4.4 MB   "—"           ↺ RESTORE    │
  │   v4   @kaya      2026-08-28 22:10   9.1 MB   "raw import"   ⇅ COMPARE   │
  └──────────────────────────────────────────────────────────────────────────┘
```

Diff is offered where it can be rendered honestly and refused where it cannot:

| Content | Diff |
|---|---|
| Text, code, structured data | Unified line diff, computed **client-side over decrypted plaintext**; the Runtime never needs a plaintext path to render a diff |
| Images | Three modes: side-by-side, swipe divider, and a difference-blend with a delta heatmap and a numeric changed-pixel percentage |
| Audio / video | Waveform or keyframe-strip alignment plus a metadata diff (duration, codec, bitrate, dimensions) |
| Everything else | Metadata diff only: size, MIME, dimensions, duration. **No fabricated binary diff view** — a hex delta that nobody can read is decoration |

Restore writes a new Version whose parent is current and whose content is the restored one, with an automatic comment naming the source Version. History never shortens.

### 7.4 Leases (checkout) for collaborative files

Concurrent binary editing has no merge, so the mechanism is advisory and its job is preventing *surprise*, not preventing writes.

```rust
struct ObjectLease { object_id: ObjectId, holder: Principal, reason: String,
                     acquired_at: Timestamp, expires_at: Timestamp }   // MANDATORY
```

Default 4 h, maximum 72 h, renewable while held, auto-released on expiry. This mirrors `16 §14.2`'s rule for Facet locks verbatim and for the same reason: **every lease has a holder, a reason, and a deadline; a lease that can be forgotten becomes a file that can be stolen by inaction.** Breaking another principal's lease requires `WRITE ∧ SHARE` or a Charter role, is a recorded domain event, and sends a Signal to the holder. Two holders of `WRITE` can always both write; when they do, the result is two Versions with a common parent and a visible `FORKED` marker in history — never a lost update, never a silent last-writer-wins.

### 7.5 Annotations anchored to a Version

```rust
struct Annotation {
    annotation_id: Ulid,
    object_id: ObjectId,
    version_id: VersionId,        // anchored to a VERSION, never to an Object
    anchor: Anchor,
    author: Principal,            // Agent authorship renders violet (11 §2.5)
    body: MessageBody,
    resolved_by: Option<Principal>,
}

enum Anchor {
    Whole,
    Rect { x: f32, y: f32, w: f32, h: f32 },     // normalized to the source
    TimeRange { start_ms: u64, end_ms: u64 },
    TextRange { start: u64, end: u64, context_hash: Hash },
}
```

When a new Version lands, anchors are re-resolved: text anchors by `context_hash` search, rect anchors by transform if the crop/resize is derivable from the version metadata. Anchors that cannot be re-resolved are marked `ORPHANED` and shown against the Version they were made on. They are never silently dropped and never silently moved to a plausible-looking new location, because a comment relocated to the wrong part of an image is worse than a comment marked orphaned.

### 7.6 Collaboration hooks for Extensions

`13 §11.2` fixes the Vault-side rules; `20 §5` fixes the hook catalog. The Profile and Gallery surfaces consume hooks 27 (`media.rendition.requesting`), 28 (`media.preview.rendering`), and 35 (`ui.profile.module`) unchanged. Two additions are proposed for the Host API and listed in §17: `vault.object.versioned` (post-commit, observational — the hook a review tool or a changelog Extension needs) and `vault.annotation.anchoring` (pre-commit, influential — the hook a domain-specific annotator needs to supply its own anchor resolver). Both go through the Reserved Hook Register process; neither ships without the 5% surface check at the phase gate (`20 §2`).

---

## 8. Permissions

### 8.1 Composition — nothing new, stated once

Effective rights are the intersection defined in `13 §10.1` and are not re-derived here:

```
  effective(principal, object) = Acl.rights
                               ∩ Envelope.capabilities        (P4/P8)
                               ∩ Charter.role_capabilities    (P1)
                               ∩ cryptographic reach          (holds a wrapped key)
```

This chapter adds exactly one filter, and only for Profile rendering:

```
  visible(module, viewer) = ModulePlacement.disclosure
                          ∩ PersonaOverride.disclosure       (narrowing only, §2.1)
                          ∩ effective(viewer, underlying objects)
```

> **Invariant M1 — a Module is a window, never a grant.** Placing an Object in a Module never widens anyone's effective rights to it. A `gallery` Module set to `Everyone` over a Collection whose Objects grant `READ` only to a Society renders the title and the item count and nothing else, to everyone else. The Module cannot be used as a permission laundering device, and this is a property test, not a review checklist.

### 8.2 Inheritance down Vault paths

`Acl.inherit: Option<VaultPath>` already exists (`13 §10.1`). The rules:

- Resolution is `own_grants ∪ resolve(inherit)` — **union inside the ACL**, because folder sharing is what people expect and an intersection would make a shared folder useless.
- The union cannot escalate, because the result is still intersected with Envelope, Charter, and cryptographic reach at §8.1. Inheritance widens the ACL term only; it cannot widen the product.
- Depth is capped at 16; a cycle is a validation error at write time, not a traversal guard at read time.
- Breaking inheritance is explicit, is a recorded event, and the UI shows the before/after principal set — because "I detached this folder and quietly lost three people's access" is the classic silent failure of every inheritance system ever built.

### 8.3 The "who can see this" explainer — a hard UI requirement

> **Requirement.** Every Object, Collection, Module and share surface must expose, in **one action**, a panel that answers "who can see this" by listing the **resolved principal set** — not the rules that produce it — with a reason on every row. CLI parity is `fn vault who <path>` (P13, N3).

```
  ┌─ WHO CAN SEE  /vault/field-notes/ ───────────────────────────────────────┐
  │ @kaya            OWNER            READ LIST WRITE SHARE DELETE           │
  │ @tobi            DIRECT GRANT     READ LIST                    expires — │
  │ archivist (6)    ROLE · ORACLE-HALL                READ LIST             │
  │ ↳ inherited from /vault/                                                 │
  │ LINK a7f2…       SHARE LINK       READ            2 uses · expires 4d    │
  │ ⟡ fn.archivist   EXTENSION        READ LIST       envelope expires 12d   │
  │ ─────────────────────────────────────────────────────────────────────────│
  │ NOT VISIBLE TO: everyone else, including other members of ORACLE-HALL    │
  └──────────────────────────────────────────────────────────────────────────┘
```

This is the highest-leverage interface in the chapter. Permission systems do not fail because their model is too weak; they fail because no human can answer this question about their own files, so they either over-share to make things work or under-share and route around the system. Naming the *reason* per row is what makes the panel actionable rather than merely informative.

### 8.4 Share-link security

```rust
struct ShareLink {
    link_id:     ShareLinkId,     // in the URL PATH — the Runtime resolves by this
    object:      ObjectRef,
    rights:      Rights,          // READ, optionally LIST; never WRITE via a link
    secret_hash: Hash,            // H(secret); the secret itself is never stored
    password:    Option<Argon2idParams>,
    expires_at:  Timestamp,       // MANDATORY
    max_uses:    Option<u32>,
    created_by:  Fnid,
    revoked_at:  Option<Timestamp>,
}
```

Five properties, each with a reason:

1. **Unguessable.** 160 bits of CSPRNG output, base32. Not a slug, not a sequence, not derived from the path.
2. **The secret lives in the URL fragment**, never the path or query. It therefore never appears in a request line, a Runtime access log, a proxy log, or a `Referer` header. The fragment secret unwraps the content key client-side, so **holding the link record does not let the Runtime read the content.** The link is not a server-side bearer credential; it is a client-side key delivery.
3. **Mandatory expiry.** Default 7 days, maximum 365, no perpetual links — the same rule and the same reasoning as Envelopes (`11 §2.8`): a credential that never expires is a credential nobody remembers to revoke.
4. **Optional password**, Argon2id, verified client-side as part of key derivation, so a wrong password fails locally and the Runtime never becomes a verification oracle.
5. **Instantly revocable**, and revocation is **forward-effective** exactly as `13 §10.1` states: it removes the link record and rotates the wrapped key for future Versions; it does not un-fetch bytes already delivered. The revoke confirmation says this in one sentence rather than implying a recall.

Every access emits `ShareLinkAccessed { link_id, at, coarse_region }`, visible to the link's creator, retained 30 days. Coarse region is country-level and exists for exactly one purpose — answering "has my link leaked" — which is the minimum viable signal for that question. Additionally, the platform never places a link secret in an email, webhook, or Signal payload it generates; the Citizen copies the link, and where they take it is theirs.

---

## 9. Intelligent Search Over Media

`13 §11.1` fixes the architecture: authored metadata on by default and indexed by the Runtime, derived signals off by default, private content indexed on the owner's Node with the index itself stored as an encrypted Object, and `V10` — no index entry derives from behavioural observation. This section specifies the signal table and is rigorous about where computation happens.

### 9.1 Indexed signals

| Signal | Source | Computed where | Default | Consent state |
|---|---|---|---|---|
| Title, filename | Citizen-authored | Runtime (public) / Node (private) | **On** | Implicit — authored to be found |
| Description, caption | Citizen-authored | as above | **On** | Implicit |
| Alt text | Citizen-authored | as above | **On** | Implicit |
| Tags | Citizen-authored | as above | **On** | Implicit |
| Vault path, Collection title | Structural | as above | **On** | Implicit |
| Society / Chamber taxonomy | Structural | Runtime | **On** | Implicit |
| MIME, codec, dimensions, duration | Technical | Node at ingest | **On** | Implicit |
| Capture time | Retained-private plane (§5.3) | Node | **On** where retained | Follows §5.3 |
| GPS | Retained-private plane | Node | **Off** | Per-Collection opt-in (§5.3) |
| Version comments, annotation text | Citizen-authored | Node / Runtime by encryption mode | **On** | Implicit |
| Contributor Handles | Structural | as above | **On** | Implicit |
| OCR text | Derived | **Node only** for private; Runtime for public | **Off** | Explicit per Object / path / Society |
| Speech transcript | Derived | as above | **Off** | Explicit |
| Auto caption | Derived | as above | **Off** | Explicit |
| Visual embedding (384-d, int8) | Derived | as above | **Off** | Explicit |
| Text embedding | Derived | as above | **Off** | Explicit |
| **Views, dwell, scroll depth, open order, hover** | — | — | — | **NOT INDEXED. Not persisted. Structurally absent** (M0, `13 V10`) |

### 9.2 Where it runs, precisely

For `EndToEnd` and private content: derived signals are computed **on the Citizen's own Node**, over plaintext it already holds, using local models. The resulting index is encrypted under a Vault-scoped index key and stored as an ordinary Object — it inherits the same erasure coding, replication, and ACL as any other Object, which means the index is as durable and as private as the content it describes. Queries execute **locally**, against the streamed encrypted index. The query string never leaves the device.

The rigorous part is the leak the naive description hides. If the client fetched only the minimal set of index segments needed for a query, the *pattern* of content-addressed segment fetches would leak coarse information about which terms were searched, to a Custodian who can correlate. Two mitigations, and an honest boundary:

- **Below 128 MB of index** — roughly 300 000 Objects at 384 B of embedding plus postings — the index is fully resident locally after the first sync, so queries generate **zero network traffic** and the leak does not exist. This covers the overwhelming majority of Citizens and is the designed operating point.
- **Above 128 MB**, the client fetches whole fixed-size index shards in padded batches rather than the minimal set. This costs bandwidth and is stated in the UI as `INDEX PARTIAL — QUERIES COST BANDWIDTH`.
- **What we do not claim:** we do not claim private-information-retrieval-grade query privacy for very large indexes. PIR at this scale is not practical today. We state the boundary rather than implying a guarantee.

For public and `Transport`-encrypted content, indexing runs in the Runtime through the `Search` port (`13 §11.1`). This reveals nothing new: the content is already readable by its audience.

### 9.3 Discovery without covert inference

Results are ranked by match strength over declared signals, recency, and the querying Citizen's **own** structures — their Collections, their Societies, their declared interests (`11 §2.1`, which no process may write). There is no collaborative filtering, no "Citizens like you", no co-view graph, because the behavioural substrate those require does not exist and will not be created. `GET /v1/…/objects/{id}/index` returns exactly what was indexed and from which source, so a Citizen can audit rather than trust. A zero-result query returns zero results and restates what was searched; it never substitutes filler recommendations, which is the mechanism by which search quietly becomes a recommendation feed.

---

## 10. Economy Integration

### 10.1 What media actually pays

| Activity | Pays | Source | Note |
|---|---|---|---|
| Uploading media | **Nothing** | — | No Source pays for volume, ever |
| Having a Profile | **Nothing** | — | |
| Profile or Gallery views | **Nothing** | — | Views are not recorded (M0). This is permanent, not a phase decision |
| Holding others' Shards | Yes | S1 Storage Custody | `13 §7`, attestation-gated |
| Serving bytes to others | Yes | S2 Bandwidth Service | Recipient-signed receipts from distinct paying Principals |
| Transcoding for others | Yes | S3 Compute Contribution | Phase 5 |
| Original media judged useful by peers | Yes | S4 Content Creation | Quadratic peer attestation, 40% vested over 30 days |
| Organizing, tagging, curating what others consume | Yes | S5 Curation | Never for self-authored objects |
| Storage above quota | Costs | K8 Storage tariff | Surplus above Custodian owed is burned |
| Egress above quota | Costs | K9 Bandwidth tariff | |
| Priority transcode / restore | Costs | K11 | |

> **Invariant M12.** No Source pays for uploads, Profile views, or Gallery views. A proposal to reward attention is a proposal to build advertising with a token attached, and is refused under `02 §4`.

### 10.2 Paid media

Three shapes, all built on existing machinery rather than a new one:

1. **One-time.** A `License` grant (`16 §13`) settled as one atomic `PostingBatch` in the same transaction as the grant. The buyer receives a durable `READ` grant plus a named license — display, commercial, derivative — and the purchase UI says which. What they do *not* receive is ambiguity about "what did I actually buy", which is the failure `16 §13` exists to prevent.
2. **Subscription.** `Term::Renewable { period }`. Renewal is an explicit Posting per period and never a silent charge above a Citizen-set ceiling. Lapse revokes forward (`13 §10.1`), and the purchase sheet states plainly that what was downloaded during the term stays downloaded. Promising otherwise would be a DRM claim.
3. **Society-gated.** Access is a function of Membership and role, so the counterparty is the Charter's `JoinPolicy` and the Treasury, not the creator directly. This is the correct shape for a Society that funds a shared archive.

The `storefront` Module renders `ListingCard`s (`32 §5.5`) from `19`. It is a *view* of listings, not a second marketplace: requested capabilities are shown before purchase (P8), and **all purchase and confirmation chrome is host-rendered outside any Extension or Module tree** (`20 §7`, T8). A Profile can advertise; it can never paint a payment dialog.

### 10.3 The ambient accrual presentation rule

This is `18 §10` applied to media, and it is a hard requirement rather than a style preference.

| Event | Where it appears | Animates | Interrupts |
|---|---|---|---|
| Bytes served (a delivery receipt) | **Nowhere.** No per-object, per-view, or per-hour readout exists | — | Never |
| Fraction accrued, unsettled | One aggregated status-bar line per settlement window: `+0.412 FRC PENDING`, DM Mono, `--fn-accent-data`, tabular figures | No | Never |
| Settlement completed | `PENDING` clears; the `WalletCard` settled figure updates | 280 ms opacity crossfade on the amount only | Never |
| Peer attestation received (S4/S5) | One line in the Signal stream | No | Never |
| Sale completed | One Signal line and one `WalletCard` row | Field-ripple on that row, once | Never |
| Storage quota crossed | Status bar in `--fn-accent-attention`; writes refused with a typed error; **reads and exports never blocked** (`13 §12`) | No | Never |

**What never happens, enforced by design-system review:** no earnings counter that increments in a viewport; no per-item earnings badge on a Gallery tile; no daily-earnings push; no sound; no toast; no confetti; no coin animation; no "you earned X today" summary card. The reasoning is one sentence: **a live-incrementing earnings number turns a Gallery into a slot machine**, and manufacturing engagement rather than value is exactly what P12 forbids.

**Aggregation is mandatory.** A thousand delivery receipts in a settlement window produce one status-bar line, not a thousand. The status bar shows at most one accrual line at a time.

### 10.4 The audit trail

> **Requirement, inherited from `18 §10.3` / I-13.** Every displayed accrual must resolve in **two actions or fewer**, from where it was displayed, to a `ContributionReceipt` (`32 §5.5`) naming the Source, the measured input, the formula, the window, the amount, and the originating `event_id`. Withheld and escrowed amounts appear with their reason. Parity across GUI and CLI is a release gate (P13).

```
$ fn me earnings --since 7d --source S2
[ ECONOMY ] @kaya // BANDWIDTH SERVICE // 7d
  WINDOW              GB      CONG   RATE     GROSS    CAP     PAID   EVENT
  2026-08-28 .. 29   41.2     1.00   0.100    4.120      —    4.120   01K4…7B
  2026-08-30 .. 31   96.8     1.40   0.100   13.552   3% pool 9.100   01K4…9C
  2026-09-01 .. 02   12.0     1.00   0.100    1.200      —    1.200   01K4…D1  PENDING
  ──────────────────────────────────────────────────────────────────────────
  PAID 14.420 FRC · WITHHELD 4.452 (per-Node pool cap) · PENDING 1.200
```

The withheld line is the point. A Citizen can always answer "why did I earn that, and why not this" without asking anyone — which is what makes P12's honesty claim checkable rather than rhetorical.

---

## 11. Peer-to-Peer Transfers and Society-Level Transactions

### 11.1 Direct transfer from a Profile

The `receive` Module renders the Citizen's global Wallet as a target and opens the canonical host-rendered `TransferSheet` (`32 §5.5`). Level-based daily limits apply unchanged (`18 §5.1`: ≤10 FRC/day at L1, ≤100 at L2, ≤500 at L4, ≤2 000 at L8, ≤5 000 at L10, no Level-imposed limit at L12 — Envelope and Charter limits always remain). There is **no transfer fee** (`17 §4.1`), so a 0.05 FRC tip costs 0.05 FRC.

Facet transfers run from the `shelf` Module using `16 §14.1` unchanged: ownership change and Fraction movement are the same transaction, and **a transfer that cannot pay its royalty Posting does not commit** — the ownership does not change. That is a ledger invariant, not a marketplace policy.

### 11.2 Requests and invoices

```rust
struct TransferRequest {
    request_id: Ulid,
    from: Fnid, to: Fnid,
    amount: Quanta, memo: String,
    reference: Option<ObjectRef>,     // "for this file / this Collection"
    expires_at: Timestamp,            // MANDATORY, max 30d
    state: RequestState,              // Open | Accepted(PostingId) | Declined | Expired
}
```

**A request is not a claim on funds.** It renders as a Signal with an explicit accept, it expires, and it never pre-authorizes anything. Abuse controls: rate-limited per `(requester, target)` pair; the target may mute a requester permanently; the requester's Handle, FNID prefix and Trust band are always shown; and a request from a Citizen outside any shared Society requires Level 4, mirroring the DM gate (`18 §5.1`) — the same reasoning applies, because an unsolicited payment request from a stranger is a phishing primitive.

### 11.3 Society treasury disbursement

Disbursement is a Charter act, not a Profile act. Below the Charter's threshold a role-holder disburses directly, recorded; above it, a Proposal. Every disbursement names a `PostingReason` from the closed enum (`11 §2.6`) and a beneficiary — **there is no discretionary treasury spend with an unnamed reason**, because `PostingReason` has no `Other`. A Profile's `receive` Module can be a disbursement *target*; it is never a disbursement *control*. The Treasury surface is where treasury money moves, and putting a treasury control on a personal page would be an authority-placement error regardless of how convenient it looks.

### 11.4 Escrowed exchange

Media-for-Fraction reuses `16 §14.3` conditional transfer with no new machinery. Conditions typical here: license acknowledged by the buyer; **delivery attested** — the buyer's Node confirms a full verified reconstruction against the Manifest's `merkle_root`, which is a real receipt rather than a click; and a mandatory deadline. Every escrow either settles or releases; there is no third outcome, and the reconciler proves the per-Society `EscrowAccount` balance equals the sum of open locks at every checkpoint.

### 11.5 The irreversibility notice

Required on every surface in this chapter that can move value. It shows: the exact amount to quantum precision in tabular DM Mono; the recipient's Handle **and** FNID prefix; their Trust band; whether this Citizen has transacted with them before; and the sentence *"This cannot be reversed."* The primary action is disabled for **800 ms** after the sheet opens.

That delay is the only enforced interaction delay in the product, and it needs its justification stated because delays are usually a dark pattern: it exists because transfer sheets are frequently opened by a click that lands on a button the Citizen did not read, and 800 ms is long enough to break click-through momentum while sitting below the threshold at which a delay is perceived as obstruction. It is not a confirmation step, not a checkbox, and not a second dialog — all three of which train people to dismiss without reading.

---

## 12. Is a Digital Home a Facet?

**Decision: a Profile layout is not a Facet and is not transferable. Themes, Module packs, cover artworks and sigil variants are.**

The reasoning is three-part and each part is independently sufficient.

**Identity is `PrincipalBound`.** `16 §12.3` already establishes the shape for things that are records of a person rather than property: `BindingMode::PrincipalBound` with `TransferBehavior::Forbid`. A Profile is the presentation of a Citizen's identity. A market in identity surfaces is a market in identities, one refactor removed.

**A tradeable layout creates a pay-to-win pressure on customization capability.** The moment layouts are assets, the incentive arrives to make layout capability scarce and sellable — more slots, exclusive footprints, purchasable grid access. `18 §5.3` already forbids the direct version of this ("Fraction may not purchase a Level-gated customization slot, because slots are complexity budget, not decoration"), and making the *output* tradeable is how that rule gets routed around.

**A layout references things the buyer does not own.** A `ProfileLayout` contains `ObjectRef`s, `CollectionId`s, Facet holdings, Insignia and disclosure settings. Transferred as-is it either breaks (references resolve to nothing) or leaks (references resolve to the seller's content). Neither is a product.

**What *is* tradeable, and how it attaches.** Themes and Module packs are Extensions and go through the marketplace review pipeline (`32 §9`, `19`). Cover artworks and sigil variants may be minted as Collectible Facets — transferable, purchasable, ornament, and therefore permitted under `18 I-8` (no Facet acquired by Transfer may gate any Level, Trust, Standing or Unlock). Attachment is by reference plus a render-time license check:

```rust
struct ProfileAttachment {
    profile_id: ProfileId,
    source:     AttachmentSource,     // ThemeExtension(InstallId) | Facet(FacetId)
    scope:      AttachmentScope,      // Theme | ModulePack | Cover | SigilVariant
    license:    LicenseRef,
    fallback:   FallbackTarget,       // deterministic, named at attach time
}
```

The Profile holds a reference, never a copy. If the license lapses or the Facet is transferred away, the Profile falls back to the named built-in **deterministically** and tells the Citizen why, in one status-bar line — it never renders broken and never silently keeps rendering something the Citizen no longer holds.

**Layouts are shareable as templates, and free.** A `template` Extension (`01 §5`, the kind already exists) carries footprints, theme, accent, density and Module *kinds*. It carries no content, no `ObjectRef`s, and no disclosure settings. Copying someone's arrangement is good for the ecosystem; selling someone's identity surface is not. The template is the design; the Profile is the person.

---

## 13. Anti-Abuse

| # | Threat | Vector | Mitigation | Residual risk |
|---|---|---|---|---|
| **T1** | **Profile impersonation** | Display name and cover art copying a known Citizen; Handles are ASCII and homograph-normalized, so display names are the real surface (`12 §4`) | Handle rendered in DM Mono adjacent to every display name in Profile chrome, never truncated; FNID prefix always shown; sigil seeds are FNID-derived and non-uploadable below L9; published, appealable Moderation Action, never silent reassignment | Semantic impersonation (`@acme_support_official`) is a policy problem with latency, not a character problem (`12` T3) |
| **T2** | **Malicious media** — polyglot files, decompression bombs, malformed codecs | Upload; share link; Gallery render | Magic-byte sniffing at ingest with declared-MIME mismatch refused; declared-vs-actual dimension check before allocation, refusing > 512 MP or > 4 GiB decoded; all untrusted decode in a sandboxed decoder with a memory ceiling and a wall-clock fuel budget mirroring `20 §9`; **every derived rendition is re-encoded**, so what other Citizens receive is never the attacker's bytes; originals served only to explicit right-holders, never auto-played | A zero-day in a hardware decode path on a device we do not control. Mitigated by preferring the re-encoded rendition by default |
| **T3** | **CSAM and illegal content** in an encrypted, content-addressed store | Any upload path | **Public / `Transport` renditions:** perceptual-hash matching against licensed hash sets at ingest and at the gateway; match refuses, records, and escalates — this works because plaintext already exists on that path (`13 §9.1`). **`EndToEnd` content: not scanned, and no client-side scanning is built** (`13 §10.3`); response is report-based, with the reporting Citizen's client decrypting under its own key and attaching signed evidence under its own signature. **On confirmation:** Unresolve (index removal — the choke point we control), Unassign (fragment hashes to the Custodian unassign set), key rotation, hash blocklist at ingest and gateway, suspension, jurisdictional referral | Stated plainly and not softened: we cannot prove global deletion; anyone who held the content key holds plaintext; exact-hash blocklists are defeated by a single re-encode, and perceptual hashing narrows this without closing it; a Custodian genuinely cannot answer whether it holds a given file. **We will not build a scanning path into E2EE**, because a guarantee with an exception is not a guarantee |
| **T4** | **Harassment via guestbook and annotations** | `guestbook` Module; annotations on shared Objects | Guestbook is opt-in (the Module is absent by default) and entries are **review-queued before render**; per-Module allow-lists (shared Societies only / Trust band / Level floor); rate limits per `(author, target)`; every entry carries Handle and Trust band; blocking is bidirectional and removes prior entries from render — not from the Log, because history is not ours to erase (`11 §3.3`) | A determined harasser creates new Citizens. The Level-0 posture (`18 §5.1`) caps what a fresh identity can do, and invite-tree accountability (`12 §9`) makes it costly rather than impossible |
| **T5** | **Share-link leakage** | Forwarded links, indexed pages, logs | Fragment-only 160-bit secret (never in a request line or log); mandatory expiry; optional `max_uses`; optional Argon2id password verified client-side; instant revocation; access log with coarse region visible to the creator; a prompt when access count exceeds the expected audience; `noindex` and `Referrer-Policy: no-referrer`; platform-generated messages never carry the secret | A leaked link is valid until noticed, and revocation is forward-effective — it cannot un-fetch delivered bytes. Said plainly in the revoke confirmation |
| **T6** | **Storage-quota abuse** | Free-CDN use of a Profile; inflating custody demand; **dedupe-oracle probing** (uploading a candidate file to learn whether someone else already stored it) | Level-granted quotas with paid overage (K8) and metered egress (K9); write refusal over quota **never blocks reads or exports** (`13 §12`); the dedupe oracle is structurally dead because content keys are Vault-scoped — identical plaintext in two Vaults yields different ciphertext and therefore different Shard hashes (`13 §3`), so cross-ACL dedupe does not exist to probe | Legitimate heavy media use looks identical to CDN abuse at the byte level; the difference is priced rather than policed |
| **T7** | **Extension Module exfiltration** | A `ui.profile.module` Module reading beyond its grant | Surface Descriptors only — no DOM, CSS, or JS (`20 §7`); narrowed context containing exactly the declared `reads` scopes, with everything else *absent* rather than stubbed; egress allowlist; host-rendered consent and payment chrome | A Module rendering data the Citizen granted it, to an audience the Citizen set — that is consent working, not a leak |
| **T8** | **Accessibility bypass via cover media** | A cover chosen to make text unreadable; a flashing loop | Save-time validator computes the scrim rather than accepting one; flash-rate analysis rejects > 3 Hz (WCAG 2.3.1); a static frame is derived and served under reduced motion; no stamp, no render | Content that passes validation and is still aesthetically hostile — a moderation matter, not a validation one |
| **T9** | **Fraction farming through media** | Self-serving to generate delivery receipts; upload-volume farming; curation self-dealing | Uploads pay nothing (M12); S2 requires receipts signed by a distinct paying Principal with positive Trust and no shared Operator lineage; prefetch emits no receipts (§6.2); S5 pays nothing for self-authored objects; Offline pins are unattested and unpaid (§6.3) | A genuine cluster of real Citizens serving each other. That is the clustering-discount problem and it is handled in `18 §9.3`, not here |

---

## 14. Phase Placement

Per `02 §3`, and constrained by the same complexity budget.

| Capability | Phase | Gate / dependency |
|---|---|---|
| Profile with Modules 1, 3, 6, 7, 8, 14, 18, 20; theme selection; save-time validator | **2** | Nothing beyond the identity and Vault spine |
| Personal Collections, Gallery Module, viewer, EXIF handling, offline pin | **2** | P2 is not deferrable |
| Vault ACL sharing, share links, version history and restore | **2** | `13` pipeline is Phase 1 |
| Search over authored metadata, path and taxonomy | **2** | Runtime `Search` port |
| Ambient accrual presentation, `ContributionReceipt` rendering (XP and S4 only) | **2** | `17` S4 is Phase 2 |
| Adaptive playback ladder, verified progressive streaming, blurhash placeholders | **3** | `13 §9`, server-side transcode on `S3BlobStore` |
| Client-side transcode for `EndToEnd` content, with the degraded path stated in UI | **3** | `13 §9.1` |
| Modules 2, 5, 10, 11, 12, 13, 15, 16; Persona overrides; guestbook with review queue | **3** | Guestbook needs `14`'s report pipeline |
| Image and text diff, escrowed media-for-Fraction exchange, transfer requests | **3** | `16 §14.3` |
| Object leases, annotations anchored to versions, local encrypted derived-signal index (OCR, transcript, embeddings) | **4** | Local index needs the P2 local store at scale |
| Storage and bandwidth accrual on a Profile (`custody` Module live numbers) | **4** | `ff.economy.storage_settlement`; `13 §11.4` step 4.3 |
| Visible watermark overlay | **5** | Not load-bearing; sequenced after the sharing model settles |
| Modules 4, 19, 21; paid media; Theme and Module-pack Extensions; template sharing | **6** | Marketplace payments and third-party paid Extensions (`02 §3`) |
| Cover and sigil artworks as Collectible Facets | **6** | Facet marketplace |
| **Never:** treasury disbursement controls on a Profile surface | — | Authority placement, not scheduling (§11.3) |
| **Never:** view counters, engagement ranking, ad-supported tiers | — | `02 §4` |

---

## 15. Trade-offs and Rejected Alternatives

| Alternative | Why rejected | What we kept |
|---|---|---|
| **Arbitrary CSS / HTML Profiles** | Forfeits every accessibility guarantee (N8), every performance budget (`32 §8`), deterministic reflow, and the design system's coherence; imports XSS, third-party tracking, and a wallet-spoofing primitive onto the most-visited page in the product. The observed outcome in products that allowed it was bimodal with the mode at "worse" (§4.3) | The constrained grid plus the theme contract, which deliver content, arrangement, palette and identity — the four axes people actually wanted |
| **No customization at all** | Coherent, cheap, and wrong. A Profile that cannot be shaped is a settings page with a public URL, and §1's thesis dies with it. It also concedes the one surface where a sovereignty product should feel most like ownership | Nothing. This one is simply refused |
| **Ad-supported free media tier** | On the Never list (`02 §4`), serving P9 and P12. **The honest cost is real:** advertising is the obvious way to fund egress, and egress is the dominant marginal cost of a media platform (`13 §6.4`). We fund it with tariffs (K8/K9) and paid capacity instead, and we accept a smaller free tier as the price of not having an advertising data model | The tariff-and-quota model, which prices the actual scarce resource |
| **Third-party embeds** (video, music, social iframes) | An iframe is a spoofing primitive (`20 §7`) and a third-party tracking beacon on a platform whose premise is P9. It also 404s, which makes a Profile decay | The `now_playing` Module: an authorized, revocable, read-only connector fetched through the egress allowlist, rendered in design-system primitives, with art re-hosted as a Vault Object. **Honest cost:** no in-Profile playback of third-party catalogues — only "what I am listening to", as data |
| **Separate Profiles per Society** | Two documents drift; N documents rot. A Citizen maintaining five Profiles maintains one | One Profile, N Personas, narrowing-only overrides (§2.1) |
| **Free-form drag positioning with overlap and z-order** | Cannot reflow deterministically, which forces either a second authored mobile layout nobody maintains or a broken phone experience | An 8-footprint grid where authored order *is* reflow order (L4) |
| **Public view counters on Galleries** | A vanity metric that converts a home into a performance, and it requires persisting exactly the behavioural data P9 forbids | Nothing. Absence is the mechanism (M0) |
| **Invisible forensic watermarking** | Destroys content-addressed dedupe and deterministic rendition identity (`13 §9.3`); requires server-side plaintext, making N6 false; embeds a covert per-recipient tracking artifact (§7.2) | Optional, visible, client-composited watermarking that the recipient can see |
| **Server-side search index over private content** | Would require plaintext or a searchable-encryption scheme with leakage we cannot bound honestly | Node-local indexing with an encrypted index Object, and a stated boundary above 128 MB (§9.2) |

---

## 16. Invariants Introduced by This Chapter

Each becomes a property test under `40-engineering-standards.md`.

1. **M0** No surface persists, displays, or ranks by views, impressions, dwell, or scroll depth.
2. **M1** A Module is a window, never a grant: rendering a Module never widens effective rights to any Object it displays.
3. **M2** No `ProfileLayout` renders without a current, Runtime-signed `ValidationStamp`; an unstamped or version-stale layout falls back to the default layout with a visible notice, never to unvalidated output.
4. **M3** No theme, Extension, or Profile may override the type scale, spacing scale, radius, contrast floors, motion durations, component structure, or any semantic accent assignment — `--fn-accent-agent` violet above all (`32 §9`).
5. **M4** A `PersonaOverride` may only narrow: it may hide a Module or reduce its disclosure, never add one or widen one.
6. **M5** GPS coordinates are removed on the client before encryption unless retained by explicit per-Collection opt-in, and never appear in any derived rendition under any setting.
7. **M6** No index entry derives from behavioural observation (`13 V10`); the derived-signal index for private content is computed on the owner's Node and stored encrypted.
8. **M7** Every `ShareLink` has an `expires_at`; its secret exists only in the URL fragment and never in a Runtime request line, log, notification, or platform-generated message.
9. **M8** Every `ObjectLease` has a holder, a reason, and a deadline; the reconciler auto-releases expired leases.
10. **M9** Every displayed accrual resolves in ≤ 2 actions to a `ContributionReceipt` naming Source, measured input, formula, window, amount, and `event_id` (`18` I-13).
11. **M10** No accrual is rendered as a real-time incrementing value in any viewport, and no accrual interrupts (`18 §10.1`).
12. **M11** Every byte delivered to a decoder from a Profile, Gallery, or viewer surface is verified against a Manifest-derived hash (`13 V2`).
13. **M12** No Source pays for uploads, Profile views, or Gallery views.
14. **M13** A `ProfileLayout` is `PrincipalBound` and non-transferable; no path converts Fraction into a Level-gated customization slot (`18 §5.3`, I-8).
15. **M14** Purchase, transfer, and confirmation chrome is host-rendered and never inside a Module or Extension tree (`20 §7`, T8).

---

## 17. Required Canon Additions

Merged in the same PR as this chapter, per `00 §6`.

**To `01-canonical-terminology.md`:** **Collection** (an ordered, ACL'd set of Object references in a Vault; an *album* is a Collection with manual ordering and an explicit cover); **Module** (a composable unit of a Profile, declared by a `ModuleManifest`, rendered from a Surface Descriptor); **Cover** (the Profile's header media, subject to a computed scrim); **ShareLink** (a revocable, expiring, fragment-secret capability over an Object); **Lease** (an advisory, expiring hold on an Object for editing); **Disclosure** (the four-value audience selector on a Module or a Standing dimension).

**To `11-domain-model.md §2.7`: LANDED.** `Vault` gains `society: Option<SocietyId>` and `owner: Principal`, and `Object` gains `society_id: Option<SocietyId>`, mirroring `Wallet` in §2.6 — `None` denotes the Citizen Vault, which hangs off Global Registry entry 1 (`Citizen`). This adds no Global Registry entry and opens no new P1 escape hatch (§5.1). `11 §7` invariant 1 is correspondingly a three-clause test, and `01 §1` and `01 §6` carry the same statement. Applied by `61 X11`; without it, invariant 1 becomes a failing property test the moment the first Citizen Vault Object is written, which is PH2.

**To `20-plugin-and-extension-model.md §5`:** two hooks — `vault.object.versioned` (post-commit, observational) and `vault.annotation.anchoring` (pre-commit, influential) — subject to the Reserved Hook Register process and the 5% surface check.

**ADRs required before Phase 2 implementation:** the Module manifest and footprint set as a formal schema owned by `32-design-system.md`; the save-time validator's contrast solver, including the cover-scrim derivation and its test corpus; and the local search index format, including the 128 MB residency threshold and the padded-batch fetch policy above it.
