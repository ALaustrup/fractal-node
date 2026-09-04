# 32 — Design System: LATTICE

> **Prerequisites:** `00-foundational-principles.md`, `01-canonical-terminology.md`, `33-brand-identity.md`.
> **Governs:** every pixel and every character cell on every surface — Web GUI, Desktop, Mobile, CLI, Terminal, marketplace, documentation, and marketing.
> **Non-negotiable N7:** one token source, compiled to every target. A color, size, duration, or easing that is not a token cannot ship.

---

## 1. The System in One Paragraph

LATTICE is a **near-monochrome, square, mono-labelled, physically-lit** interface system. Structure is drawn with hairlines and corner registration marks rather than filled boxes. Color is spent almost entirely on meaning: mint for human action, blue for data and value, violet for agents. Type does the hierarchy work — Manrope for human language, DM Mono for machine truth — so the interface can be dense without being loud. Motion is scarce and always states a fact. The result reads as an instrument: quiet at rest, precise under use, and unmistakably not another consumer social app.

---

## 2. Token Architecture

```
   tokens/
   ├── core.json          primitives: raw color, size, duration, easing values
   ├── semantic.json      meaning: --fn-text-primary, --fn-accent-agent, --fn-dur-base
   ├── component.json     component-scoped: --fn-btn-height, --fn-panel-pad
   └── theme/
       ├── void.json      dark (default, flagship)
       ├── daylight.json  light
       └── contrast.json  high-contrast (AAA everywhere, no glows)
                 │
                 │  cargo xtask tokens      (Rust — no Node.js dependency,
                 ▼                           so the CLI never needs a JS toolchain)
   ┌─────────────┴──────────────────────────────────────────────┐
   ▼           ▼             ▼            ▼           ▼          ▼
 tokens.css  tokens.rs   Tokens.swift  Tokens.kt  ansi.rs   tokens.d.ts
 (web,       (CLI,       (iOS)         (Android)  (terminal (types for
  desktop)    native)                              palette)  authoring)
                 │
                 └──► CI gate: `git diff --exit-code` after regeneration.
                      Drift between targets is impossible, not discouraged.
```

**Three tiers, strictly.** Components reference semantic tokens; semantic tokens reference core tokens; nothing skips a tier. A component that reaches for a core token is a review rejection — it means a semantic meaning is missing and should be named.

**Naming:** `--fn-<category>-<role>-<variant?>-<state?>`, e.g. `--fn-text-primary`, `--fn-accent-agent`, `--fn-surface-2`, `--fn-btn-bg-hover`. Lowercase, hyphenated, no abbreviations except the `fn` prefix.

---

## 3. Core Tokens

### 3.1 Color primitives

```
/* field */
--fn-c-void        #040608     --fn-c-surface-1   #080c10
--fn-c-surface-2   #0c1116     --fn-c-surface-3   #121820
--fn-c-surface-4   #1a222c

/* ink */
--fn-c-ink-100     #e8edf0     --fn-c-ink-80      #a8b4bb
--fn-c-ink-60      #7b868d     --fn-c-ink-40      #586267
--fn-c-ink-20      #394246     --fn-c-ink-inv     #03100f

/* accents (brand-locked, see 33 §2.1) */
--fn-c-signal      #8ce8df     --fn-c-signal-deep #4d9e9a
--fn-c-electric    #55b9ff     --fn-c-field       #9d6cff
--fn-c-ember       #ff9d5c     --fn-c-rupture     #ff6b7a

/* lines */
--fn-c-line-subtle rgba(173,225,220,.07)
--fn-c-line        rgba(173,225,220,.14)
--fn-c-line-strong rgba(173,225,220,.26)
--fn-c-line-signal rgba(140,232,223,.45)
```

### 3.2 Semantic color

| Token | Void theme | Meaning |
|---|---|---|
| `--fn-text-primary` | `ink-100` | Body and headings |
| `--fn-text-secondary` | `ink-80` | Supporting prose |
| `--fn-text-tertiary` | `ink-60` | Metadata, timestamps |
| `--fn-text-quaternary` | `ink-40` | Mono micro-labels only |
| `--fn-accent-action` | `signal` | Primary action, live, success, present |
| `--fn-accent-data` | `electric` | Fraction, metrics, links, focus |
| `--fn-accent-agent` | `field` | Non-human origin — **never reassigned** |
| `--fn-accent-attention` | `ember` | Pending, degraded, stale, staked |
| `--fn-accent-danger` | `rupture` | Error, destructive, Fracture, slash |
| `--fn-accent-dormant` | `ink-40` | Disabled, archived, offline |
| `--fn-bg-page` | `void` | |
| `--fn-bg-sunken` | `surface-1` | Input wells, code, gutters |
| `--fn-bg-raised` | `surface-2` | Panels, cards, sidebars |
| `--fn-bg-floating` | `surface-3` | Menus, popovers |
| `--fn-bg-overlay` | `surface-4` | Modals, command palette |

**The one-glance rules, enforced by lint and review:**
1. `--fn-accent-agent` appears on non-human-authored content and nowhere else.
2. Every Fraction amount uses `--fn-accent-data`. Money is never green or gold.
3. At most three accent hues visible in one viewport.
4. No literal hex outside `tokens/`. Stylelint `color-no-hex` with an allowlist of one file.

### 3.3 Space, size, radius

```
--fn-space-0  0     --fn-space-1  2px   --fn-space-2   4px   --fn-space-3   8px
--fn-space-4  12px  --fn-space-5  16px  --fn-space-6   20px  --fn-space-7   24px
--fn-space-8  32px  --fn-space-9  40px  --fn-space-10  48px  --fn-space-11  64px
--fn-space-12 80px  --fn-space-13 96px  --fn-space-14 128px

--fn-radius-0 0      /* default — panels, inputs, buttons, tables */
--fn-radius-1 2px    /* chips, small controls */
--fn-radius-2 4px    /* thumbnails, list avatars */
--fn-radius-pill 999px  /* status pills only */
--fn-radius-full 50%    /* avatars, dots, orbit nodes */

--fn-hairline 1px    /* never 2px for structure; 2px is reserved for the active rail */
```

### 3.4 Elevation

Elevation is **light**, not shadow-as-decoration. Four levels only.

| Level | Composition |
|---|---|
| `e0` flat | none |
| `e1` raised | `0 1px 0 var(--fn-c-line-subtle)` inset top |
| `e2` panel | `0 28px 90px rgba(0,0,0,.40)` + `0 0 38px rgba(91,211,201,.09)` + `inset 0 0 28px rgba(91,211,201,.025)` |
| `e3` floating | `0 34px 110px rgba(0,0,0,.55)` + dual-tone rim (`-4px -4px 18px rgba(93,195,255,.20)`, `4px 4px 18px rgba(161,109,255,.20)`) |
| `e4` overlay | `0 38px 120px rgba(0,0,0,.60)` + dual-tone rim + `backdrop-filter: blur(16px)` on the scrim |

### 3.5 Motion (from `33 §4`)

```
--fn-ease-out    cubic-bezier(.16,.7,.2,1)
--fn-ease-inout  cubic-bezier(.2,.8,.2,1)
--fn-ease-exit   cubic-bezier(.4,0,1,1)

--fn-dur-instant  90ms   --fn-dur-fast   180ms   --fn-dur-base   280ms
--fn-dur-slow    420ms   --fn-dur-reveal 650ms   --fn-dur-ambient  4s
```

### 3.6 Z-index (a closed scale — no arbitrary values)

```
0 content · 10 sticky headers · 20 active rail · 30 dropdown · 40 popover
50 tooltip · 60 drawer · 70 modal · 80 command palette · 90 toast · 100 boot/splash
```

---

## 4. Layout

### 4.1 The Society shell

The flagship layout. Four regions, stable across every screen size; what changes is which regions are resident vs summoned.

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ⌁ FN // ORACLE-HALL          ⌘K              ● LIVE   1,204 FRC    @kaya │ 44px topbar
├────┬─────────────────┬──────────────────────────────────┬────────────────┤
│    │                 │                                  │                │
│ S  │   CHAMBERS      │           STAGE                  │   CONTEXT      │
│ O  │                 │                                  │                │
│ C  │ ▌# general      │  the active surface:             │  members       │
│ I  │   # signal      │  chamber · wallet · vault ·      │  agents        │
│ E  │   # treasury    │  profile · market · charter      │  pinned        │
│ T  │                 │                                  │  activity      │
│ Y  │   VOICE         │                                  │                │
│    │   ◇ commons     │                                  │                │
│ R  │                 │                                  │                │
│ A  │   AGENTS        │                                  │                │
│ I  │   ◈ archivist   │                                  │                │
│ L  │                 │                                  │                │
│    ├─────────────────┤                                  │                │
│ 56 │  XP ▓▓▓▓▓░░ L7  │                                  │                │
├────┴─────────────────┴──────────────────────────────────┴────────────────┤
│ FN://ORACLE-HALL/GENERAL · SYNCED 2s AGO · 14 CUSTODIANS · BLOCK 44812   │ 24px status
└──────────────────────────────────────────────────────────────────────────┘
   56px      240px              fluid                        320px
```

- **Society Rail** (56px): vertical strip of Society sigils. The only always-visible cross-Society navigation. Unread is a 4px signal dot, not a count badge — counts create anxiety, presence does not.
- **Chambers** (240px): the current Society's spaces, grouped by kind. Selection uses the active rail (`inset 2px 0 0`).
- **Stage** (fluid): the working surface. One at a time. Never a nested scroll region inside another scroll region.
- **Context** (320px): collapsible. Contextual to the Stage.
- **Status bar** (24px): DM Mono 8px, `.18em` tracking, `--fn-text-quaternary`. Path, sync state, custodian count, anchor block. This is the single most brand-defining piece of chrome in the product — it is the site's `terminal-readout` promoted to a permanent fixture.

### 4.2 Breakpoints

| Name | Width | Behavior |
|---|---|---|
| `handheld` | ≤ 1100 | Rail collapses to a summonable sheet; Context becomes a drawer; Stage full-bleed. Touch and gamepad targets ≥ 44px. |
| `compact` | 1101–1439 | Rail + Chambers + Stage. Context on demand. |
| `standard` | 1440–1919 | All four regions. Default. |
| `wide` | 1920–2559 | Context expands to 380px; Stage gains a max measure of 88ch and centers. |
| `ultra` | 2560–3439 | Stage may split into two panes (e.g. Chamber + Vault). |
| `panorama` | ≥ 3440 at ≥ 21:9 | Three-pane Stage. Surplus width buys **panes, never longer lines.** |

**The ultrawide invariant:** reading measure never exceeds 88 characters at any width. A 5120px monitor gets more surfaces, not wider paragraphs. This single rule is what separates a considered ultrawide experience from a stretched one.

### 4.3 Density

| Mode | Row height | Vertical padding | Default for |
|---|---|---|---|
| Comfortable | 40px | `space-4` | Everything, by default |
| Compact | 32px | `space-3` | Power users; auto-selected at `wide`+ |
| Dense | 26px | `space-2` | Tables, logs, ledger, audit trails only |

Density is a token multiplier, not a separate stylesheet.

---

## 5. Component Inventory

Every component below has: an anatomy, all states (`rest · hover · active · focus-visible · disabled · loading · error · empty`), a keyboard contract, an ARIA contract, a token map, a CLI/terminal equivalent, and a visual regression test. A component without all nine does not exist.

### 5.1 Foundation

| Component | Notes |
|---|---|
| `Panel` | Three intensities: Tick (corner marks), Framed (+ border), Sealed (+ dual-tone rim). Corner marks are 14px, diagonal opposition. |
| `Rail` | The 2px inset selection indicator. Used by nav, list rows, tabs, and the CLI (`▌`). |
| `Hairline` | 1px divider at `--fn-c-line`. Never 2px. |
| `Grain` | One viewport-level noise layer at 3–4%, 8s cycle. Never per-element. |
| `FieldGlow` | Cursor-tracked radial glow inside large panels. Disabled under reduced motion. |
| `Sigil` | The diamond mark. Sizes 12/16/24/32/64. Procedural variant for Societies. |

### 5.2 Input

`Button` (primary / secondary / ghost / danger — height 40 comfortable, 32 compact; magnetic hover on primary only), `IconButton`, `TextField`, `TextArea`, `Select`, `Combobox`, `Checkbox`, `Radio`, `Switch`, `Slider`, `SegmentedControl`, `SearchField`, `AmountField` (Fraction-specific: tabular figures, quantum precision, unit suffix, max/half shortcuts), `FilePicker`, `DatePicker`.

**Field anatomy** (from the site's `label` pattern):
```
  01 / SOCIETY NAME                    ← DM Mono 9px, .20em, --fn-text-quaternary
  ┌────────────────────────────────┐
  │ Oracle Hall                    │  ← DM Mono 14px, .06em, --fn-text-primary
  └────────────────────────────────┘  ← 1px --fn-c-line, becomes --fn-c-line-signal on focus
  Changeable once, within 14 days.    ← Manrope 12px, --fn-text-tertiary
```
Focus adds `filter: drop-shadow(0 0 7px rgba(140,232,223,.4))` (extracted) **plus** a 2px offset outline. The glow alone is not a sufficient focus indicator — WCAG requires the outline, and reduced-motion/high-contrast users need it.

### 5.3 Navigation

`SocietyRail`, `ChamberList`, `Tabs`, `Breadcrumb` (renders as `FN://SOCIETY/CHAMBER`), `CommandPalette` (⌘K/Ctrl-K — every action in the product is reachable here; it is the GUI's expression of P13), `ContextMenu`, `Drawer`, `Sheet` (handheld).

### 5.4 Display

`Avatar` (Citizen: circle. Society: diamond-inscribed sigil. Agent: circle with a violet rim — the shape itself encodes principal class), `StatusPill` (closed vocabulary from `33 §7.2`), `SignalDot` (the pulse), `Badge`, `InsigniaTile`, `MetricTile`, `Sparkline`, `Chart` (per the dataviz rules — categorical series draw from a 12-stop wheel at fixed chroma, never the semantic accents), `Table` (dense mode, tabular figures, sticky header, column sort, virtualized), `Timeline`, `CodeBlock`, `LogStream`, `EmptyState` (uses the outlined-word display treatment and a static orbit), `Skeleton`.

### 5.5 Domain components

These are where the design system earns its keep. Each is specified once and used identically on every surface.

| Component | Purpose | Signature detail |
|---|---|---|
| `MessageRow` | A Chamber message | Human = default. **Agent = violet left rail, violet handle, `⟡` prefix, and an "acting under Envelope X" affordance.** Never merely a small label. |
| `Composer` | Message input | Mono input, ⌘Enter to send, field-ripple on commit, agent-mention picker |
| `WalletCard` | Balance + recent | Electric, tabular, `--fn-accent-attention` for pending, always shows settled vs pending separately |
| `TransferSheet` | Send Fraction | Amount, recipient, memo, fee breakdown, **irreversibility notice**, field-ripple on commit |
| `LedgerTable` | Postings | Dense, both sides of every Posting visible, reason chip, anchor block reference |
| `XpMeter` | Progression | 5-segment bar, ambient. Level-up is a 650ms segment fill + a status-bar line. **Never a modal, never confetti.** |
| `ContributionReceipt` | Why you earned | Source, measured input, formula, amount, window. The audit trail P12 requires, rendered. |
| `EnvelopeCard` | An Agent grant | Capabilities as chips, limits as meters, TTL countdown, one-click revoke |
| `PolicyEditor` | Human authoring | Plain-language rendering above the machine form; the human text is generated *from* the policy, never the reverse |
| `CharterView` | Governance | Diff-first: what changed, who signed, when enacted |
| `FacetTile` | An asset | Shows evolution state and provenance depth, not just an image |
| `CustodianPanel` | Storage service | Bytes held, bytes served, attestations passed, FRC accrued this window |
| `SocietyCard` | Discovery | Sigil, name, member count, Level, lineage badge (Crystallized / Fractured / Forked) |
| `ListingCard` | Marketplace | Price in FRC, creator Trust, **requested capabilities shown before purchase** (P8) |
| `FractureWizard` | The signature op | Multi-step, mandatory dry-run diff, explicit confirmation of what each child receives |

---

## 6. States, Loading, and Emptiness

**The four states every data surface must define**: `loading`, `empty`, `error`, `stale`. A screen that defines only the happy path is not done (`00 §5`).

- **Loading** — skeletons matched to final layout, never spinners over content. Above 800ms, the status bar shows the operation. Never a blocking overlay for a read.
- **Empty** — the outlined-word display treatment, a static orbit, one sentence of what this is for, one primary action. Empty states are the best onboarding surface in the product; treat them as designed screens, not fallbacks.
- **Error** — cause then remedy, no apology (`33 §7.3`). Inline where the failure is scoped; the status bar for global. Never a modal for a recoverable error.
- **Stale (P2)** — this is unique to a local-first product and must be visible without being alarming: the status bar shows `SYNCED 2s AGO`; beyond 60s it shows `STALE 4m`; offline shows `OFFLINE · 3 QUEUED` in `--fn-accent-attention`. Content stays fully readable and interactive. Queued writes render at 70% opacity with a pending mark until settled.

---

## 7. Accessibility (N8 — non-negotiable)

| Requirement | Standard |
|---|---|
| Contrast | AA floor everywhere; **AAA for body text in the default theme** (`ink-100` on `void` = 17.4:1) |
| `ink-40` | Decorative and mono micro-labels only. Lint-enforced: never on prose. |
| Focus | Always visible, 2px outline + 2px offset, never removed. Glow is additive, never the sole indicator. |
| Keyboard | Every action reachable. Documented shortcut map. ⌘K reaches everything. Full focus-trap discipline in modals. |
| Screen readers | Every interactive element labelled. Live regions for incoming messages and balance changes, `polite` not `assertive`. |
| Motion | `prefers-reduced-motion` disables pulse, orbits, ripple, magnetic hover, and field glow. Every state remains distinguishable without motion. |
| Color | Never the sole carrier of meaning. Agent origin also carries the `⟡` glyph and a text label. Status pills carry words, not just hues. |
| Targets | ≥ 44×44 at `handheld`; ≥ 32×32 elsewhere with ≥ 8px spacing. |
| Text | Zoom to 200% without loss of function; no fixed-height text containers. |
| Contrast theme | A third theme with all glows removed and every pair ≥ 7:1. |
| Gate | axe-core in CI blocks merge. Manual audit at each phase gate. |

---

## 8. Performance Budgets (P10 — CI-enforced)

| Metric | Web | Desktop | Measured by |
|---|---|---|---|
| Cold start → interactive | 2.5s p75 | 1.5s p75 | Lighthouse CI / instrumented boot |
| Warm start | 800ms | 400ms | instrumented |
| Interaction → paint | 100ms p95 | 100ms p95 | INP / frame trace |
| Frame budget | 16.6ms (8.3 where display allows) | same | trace |
| Initial JS | ≤ 180KB gzip | n/a | bundle analyzer |
| Route chunk | ≤ 60KB gzip | n/a | bundle analyzer |
| Memory ceiling | **350MB** | **450MB RSS** (5 Societies, 10k cached Messages) | soak test |
| Font payload | ≤ 90KB (subset WOFF2, 2 families) | same | build |

**This table is the source.** `perf/budgets.json` is **generated from it**, and `40 §13.1`, `34 §11` and `31 §10` render from that file — `cargo xtask budgets --render` regenerates them and `cargo xtask lint-docs` fails on a hand-authored budget number, the mechanism `40 §4.2` already applies to colour literals. Ownership is partitioned so that "authoritative" does not mean "`32` must state an iOS battery figure": this table owns the cross-platform **web and desktop** rows; `34 §11` owns per-target extensions `32` does not state (Android, iOS, Linux, macOS, PWA, battery, binary size, IPC cost) and may never restate a row above with a different value; `31 §10` owns the CLI rows; `40 §13.1` owns the **server-side** rows and references rather than copies. Reconciled in `61 X9`; the desktop and web memory figures above are `34`'s, which were both stricter and better specified, and `50 PH2` AC-2 moves with them.

**Motion performance rules:** animate only `transform`, `opacity`, and `filter`. The grain and field-glow layers are `will-change`-hinted and GPU-composited. Any animation causing layout or paint fails review. Frame-time regression fails CI.

---

## 9. Theming

Three built-in themes (`void`, `daylight`, `contrast`) plus **Society accent** (one of 12 curated hues at matched chroma, applied only to sigil, active rail, selection, and treasury chart series) and **Profile themes** (marketplace-purchasable, but strictly bounded).

**The theming contract** — this is what prevents user themes from destroying the product:

- A theme may override: `--fn-c-void` through `--fn-c-surface-4`, `--fn-c-signal`, the 12-stop accent wheel, and one optional background texture.
- A theme may **not** override: semantic accent assignments (agent violet is permanent), contrast ratios, spacing, radius, type scale, motion durations, or component structure.
- Every theme is validated at install time: all token pairs must pass AA, or the theme is rejected with the failing pair named. A theme cannot ship an inaccessible interface.
- Themes are Extensions (P7) and go through the marketplace review pipeline (`19`).

---

## 10. The CLI as a First-Class Design Surface (N3, N7)

The design system compiles to a terminal palette. This is not decoration — it is what makes the CLI a peer rather than a fallback.

| GUI concept | Terminal equivalent |
|---|---|
| `--fn-c-signal` | truecolor `#8ce8df`; 256-color fallback `86`; 16-color `cyan` |
| `--fn-c-electric` | `#55b9ff` / `75` / `bright blue` |
| `--fn-c-field` (agent) | `#9d6cff` / `141` / `magenta` |
| Active rail | `▌` in signal, leading the line |
| Panel corner ticks | `┌`…`┘` fragments at the corners only |
| Section index | `[ 02 / TREASURY ]` in signal, `.23em` rendered as spaced caps |
| Status pill | ` LIVE ` inverse video in the semantic hue |
| Signal dot pulse | `●` cycling at 2s between full and dim |
| XP meter | `▓▓▓▓▓░░` |
| Agent message | violet `⟡` prefix — the same glyph as the GUI |
| Field ripple | a single-frame flash of the committed line |

**Degradation ladder:** truecolor → 256 → 16 → no color (glyphs and layout carry all meaning) → `--plain` (machine-readable, no ANSI at all). `NO_COLOR` and `FN_OUTPUT=json` are honored. Full specification in `31-cli-and-terminal.md`.

---

## 11. Documentation and Governance of the System

- **Storybook** (or an equivalent) hosts every component with all nine required artifacts. A component not in Storybook does not exist.
- **Visual regression** on every component × theme × density × state, on every PR.
- **Contribution:** a new component requires a design review, a token map (no new core tokens without justification), a CLI equivalent, and an a11y sign-off. A new *pattern* (a novel interaction) requires an ADR.
- **The extension rule:** Extensions compose from these primitives declaratively. They receive no arbitrary DOM or CSS (`20`). This is what guarantees that a marketplace of thousands of Extensions still looks and behaves like one product — and it is the design decision most likely to be argued with and most important to hold.
- **Ownership:** the token source and this document are the same commit. Changing an extracted brand value (`33 §2.1`, `33 §4.1`) requires an ADR.
