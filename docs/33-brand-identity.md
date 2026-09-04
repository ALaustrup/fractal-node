# 33 — Brand Identity: The Lattice System

> **Status:** Canon for all visual and verbal output.
> **Derived from:** the live `fractalnode.net` Access Terminal — extracted computed styles, motion curves, and copy structure — then extended from a one-page teaser into a complete identity capable of carrying a product, a marketplace, a CLI, and a marketing package.

---

## 1. Brand Thesis

The existing site already knows what Fractal Node is. It reads as an **instrument, not an app** — a dark field with a signal moving through it, orbital geometry, monospace classification labels, and language that treats the visitor as an operator rather than a consumer. Copy like *"Not another platform. Not another tool. A signal becoming structure."* and section indices like `[ 02 / DEVELOPMENT RELAY ]` establish a voice of **restrained technical authority**.

The identity system below preserves that DNA exactly and gives it the range a product needs: hierarchy for dense interfaces, semantic color for state, a light mode that does not betray the aesthetic, and a motion language that scales from a hover to a boot sequence.

**Positioning line:**
> Fractal Node is the instrument you use to build a society — not the platform that runs one for you.

**Three words:** *Sovereign. Precise. Alive.*

**The core metaphor — The Lattice.** Everything is a node in a field. Structure emerges from signal. The visual system therefore never draws a "container with content"; it draws a **field with structure resolving out of it**. This is why the site's panels have corner ticks rather than closed borders, why orbits rotate at different rates, and why the display type has one word solid and one word outlined — *resolved* and *not yet resolved*.

---

## 2. Color

### 2.1 Extracted core (do not alter — these are the brand)

| Token | Value | Role |
|---|---|---|
| `--fn-void` | `#040608` | The field. Base background. Near-black with a blue bias. |
| `--fn-ink` | `#e8edf0` | Primary text. |
| `--fn-muted` | `#7b868d` | Secondary text. |
| `--fn-signal` | `#8ce8df` | **Primary accent.** Mint-aqua. The signal itself. |
| `--fn-signal-deep` | `#4d9e9a` | Pressed / secondary signal. |
| `--fn-electric` | `#55b9ff` | Secondary accent. Blue. Data, links, focus. |
| `--fn-field` | `#9d6cff` | Tertiary accent. Violet. Agents, automation, the non-human. |
| `--fn-line` | `rgba(173,225,220,.14)` | Hairline rules and dividers. |

### 2.2 The extended ramp

The teaser needed eight colors. A product needs a ramp. These are derived by holding the void's blue bias constant and stepping luminance on a perceptual curve.

**Surface ramp** — every surface is the void plus light, never a separate grey:

```
--fn-surface-0   #040608   the field (page background)
--fn-surface-1   #080c10   sunken (input wells, code blocks, timeline gutters)
--fn-surface-2   #0c1116   raised (panels, cards, sidebars)
--fn-surface-3   #121820   floating (menus, popovers, tooltips)
--fn-surface-4   #1a222c   overlay (modals, command palette)
--fn-scrim       rgba(2,4,6,.72)   modal backdrop, with blur(16px)
```

**Text ramp:**

```
--fn-text-primary    #e8edf0   17.4:1 on void — AAA
--fn-text-secondary  #a8b4bb    9.6:1 on void — AAA
--fn-text-tertiary   #7b868d    5.6:1 on void — AA
--fn-text-quaternary #586267    3.1:1 — decorative/mono micro-labels ONLY, never prose
--fn-text-inverse    #03100f   on signal-filled surfaces
```

**Line ramp:**

```
--fn-line-subtle  rgba(173,225,220,.07)   internal dividers
--fn-line         rgba(173,225,220,.14)   default (extracted)
--fn-line-strong  rgba(173,225,220,.26)   panel edges, focused fields
--fn-line-signal  rgba(140,232,223,.45)   active/selected edges
```

### 2.3 Semantic color

The brand's three accents are *assigned meaning* rather than used decoratively. This is the single most important discipline in the system — it is what lets a dense interface stay readable in near-monochrome.

| Semantic | Color | Meaning | Where |
|---|---|---|---|
| **Signal** | `#8ce8df` mint | Human action, success, live, present | Primary buttons, active nav, online, confirmed |
| **Electric** | `#55b9ff` blue | Data, value, links, focus ring | Fraction amounts, metrics, hyperlinks, focus |
| **Field** | `#9d6cff` violet | Agents, automation, the non-human | Agent messages, workflow runs, AI-generated |
| **Ember** | `#ff9d5c` amber | Attention, pending, degraded | Warnings, staleness, sync-pending, staking |
| **Rupture** | `#ff6b7a` coral | Destruction, error, irreversible | Errors, slashing, Fracture, delete |
| **Dormant** | `#586267` grey | Inactive, archived, offline | Disabled, archived Societies |

**Why violet for agents specifically:** users must be able to tell, at a glance and without reading, whether a human or an agent produced something. Assigning a permanent, non-negotiable hue to non-human origin does that pre-attentively. This is a P4 requirement expressed in color: *the boundary between human and agent is visible.*

**Fraction is always Electric.** Never green, never gold. Money in this system is *data*, not treasure. This one decision keeps the economy feeling like an accounting instrument rather than a casino — which is precisely what P12 requires.

### 2.4 Light mode — "Daylight"

Light mode is not an inversion. It is the same lattice viewed under illumination: the field becomes paper, the signal stays the signal, and the accents darken to hold contrast.

```
--fn-surface-0   #f4f6f7      --fn-text-primary    #0a1014
--fn-surface-1   #eaeef0      --fn-text-secondary  #3d484e
--fn-surface-2   #ffffff      --fn-text-tertiary   #5f6b71
--fn-surface-3   #ffffff      --fn-line            rgba(10,40,44,.12)
--fn-surface-4   #ffffff
--fn-signal      #0f7a70   (darkened for 4.6:1 on paper)
--fn-electric    #1668c7
--fn-field       #6b35d6
--fn-ember       #a35a12
--fn-rupture     #c2303f
```

Glows become soft shadows; the grain layer drops to 30% opacity; the outlined-word display treatment uses a 1px ink stroke. **Dark is the default and the flagship.** Daylight exists for accessibility, daytime work, and printed/marketing contexts — it must be *good*, not merely present.

### 2.5 Society accent

Each Society may pick one accent hue from a curated 12-stop wheel, all tuned to the same chroma and luminance as `--fn-signal`. It is applied only to: the Society's sigil, its active-nav rail, its Chamber selection state, and its Treasury chart series. It never overrides semantic color. This gives Societies identity without letting them break the system — the classic failure of user-themable products.

---

## 3. Typography

### 3.1 The two-family system (extracted, retained)

| Role | Family | Weights | Why |
|---|---|---|---|
| **Display & UI** | **Manrope** | 300, 400, 500, 600, 700 | Geometric-humanist sans with tight, near-monolinear curves. At weight 300 with negative tracking it reads architectural rather than corporate — which is the whole voice. |
| **Mono & Data** | **DM Mono** | 300, 400, 500 | Warm, low-contrast monospace. Carries the "instrument" register without the retro-terminal cliché of a pixel or IBM face. |

Both are open-licensed (SIL OFL), self-hosted as subset WOFF2, variable where available. **No third family. Ever.** A third family is how design systems die.

Fallbacks: `Manrope, "Inter var", system-ui, -apple-system, "Segoe UI", sans-serif` and `"DM Mono", "JetBrains Mono", ui-monospace, "Cascadia Mono", monospace`.

### 3.2 The signature display treatment

The site's defining typographic move, preserved verbatim as a system component:

```css
.fn-display {
  font-family: Manrope;
  font-weight: 300;
  letter-spacing: -0.065em;
  line-height: 0.88;
}
.fn-display em {                     /* the second word */
  font-style: normal;
  color: transparent;
  -webkit-text-stroke: 1px rgba(232,237,240,.72);
}
```

Two words. First solid, second outlined. *Fracture / the Node.* *Live / Signal.* *Request / Access.* The outline is the thing not yet resolved — structure emerging from field. Use it for: marketing headlines, empty states, onboarding, Season titles, phase-gate screens. **Never** in dense product UI; it does not survive below 34px.

### 3.3 Type scale

Two ladders. The display ladder is fluid (`clamp`) because it is compositional. The UI ladder is fixed because interfaces need predictable rhythm.

**Display (Manrope 300, tracking −0.065em → −0.02em as size drops):**

| Token | Size | Use |
|---|---|---|
| `display-1` | `clamp(65px, 9.2vw, 146px)` | Hero. Extracted value, unchanged. |
| `display-2` | `clamp(58px, 8vw, 120px)` | Section opener |
| `display-3` | `clamp(34px, 5.4vw, 78px)` | Pull quote / manifest |
| `display-4` | `clamp(28px, 3.2vw, 44px)` | Surface title |

**UI (Manrope):**

| Token | Size / LH / Weight | Use |
|---|---|---|
| `title-l` | 24 / 1.25 / 500 | Panel title |
| `title-m` | 18 / 1.35 / 500 | Card title |
| `title-s` | 15 / 1.4 / 500 | List heading |
| `body-l` | 15 / 1.65 / 400 | Reading text, messages |
| `body-m` | 14 / 1.6 / 400 | Default UI text |
| `body-s` | 13 / 1.55 / 400 | Dense UI |
| `caption` | 12 / 1.45 / 400 | Timestamps, helper |

**Mono (DM Mono) — the classification register:**

| Token | Size / Tracking / Case | Use |
|---|---|---|
| `label-l` | 11 / .20em / UPPER | Buttons, primary labels |
| `label-m` | 10 / .23em / UPPER | Eyebrows, section indices — extracted value |
| `label-s` | 9 / .20em / UPPER | Field labels, chips |
| `label-xs` | 8 / .18em / UPPER | Micro-readouts, status bars — extracted value |
| `data-m` | 14 / .06em / normal | Inputs, addresses, hashes, Fraction amounts |
| `data-s` | 12 / .04em / normal | Table cells, log lines |
| `code` | 13 / 0 / normal | Code blocks, CLI output |

**Rule:** anything that is *machine-truth* — an amount, an ID, a hash, a timestamp, a status, a count — is set in DM Mono. Anything that is *human-language* is set in Manrope. A reader can therefore tell fact from prose without reading either. This is the typographic expression of P12 (economic honesty): the numbers look like numbers.

### 3.4 Tabular figures

`font-variant-numeric: tabular-nums` is mandatory on every Fraction balance, XP value, member count, and table column. Non-tabular figures in a live-updating balance produce visible width jitter, which reads as instability — exactly the wrong signal for a wallet.

---

## 4. Motion

### 4.1 Extracted curves (canonical)

```
--fn-ease-out    cubic-bezier(.16, .7, .2, 1)    entrances, expansions, reveals
--fn-ease-inout  cubic-bezier(.2, .8, .2, 1)     movement, morphs, transitions
--fn-ease-exit   cubic-bezier(.4, 0, 1, 1)       exits (added — the site has no exits)
```

### 4.2 Duration scale

```
--fn-dur-instant  90ms    state flips: hover, checkbox, toggle
--fn-dur-fast    180ms    micro: tooltip, chip, focus ring
--fn-dur-base    280ms    default: extracted from site transitions
--fn-dur-slow    420ms    panels, drawers, sheets
--fn-dur-reveal  650ms    scroll reveals, surface entrances
--fn-dur-ambient  4s+     orbits, pulses, grain — never blocks interaction
```

### 4.3 The five signature behaviors

These are the interaction fingerprints. They appear on every surface, in every client, including the CLI where the medium allows.

**① Pulse.** A 5px signal dot, `2s` infinite, opacity 1 → .25, glow `0 0 12px` → `0 0 2px`. Means *this is live*. Extracted verbatim. Used for: connection status, live Chambers, active Agents, streaming responses.

**② Magnetic hover.** Cursor proximity pulls an element up to 6px toward the pointer and tilts it up to 3°, via `--mag-x`, `--mag-y`, `--mag-tilt-x/y`, spring-damped, released on exit. Extracted from `physics.css`. Applied to: primary buttons, Society cards, marketplace listings, Facet tiles. **Never** to list rows or dense table cells — magnetism in a dense list is nausea.

**③ Field glow.** A radial gradient follows the cursor inside a panel via `--glow-x` / `--glow-y`, ~180px radius, 6–10% signal opacity. Makes large dark surfaces feel responsive and lit rather than dead. Extracted. This single effect does more for "premium" perception than any other in the system.

**④ Letter-spacing breath.** On hover, mono labels expand tracking by ~0.02em with a text-shadow bloom over 280ms. Extracted (`transition: color .28s, text-shadow .28s, letter-spacing .28s`). Text *responds* without moving. Reserved for mono labels and links.

**⑤ Field ripple.** On click/commit, a ring expands from the point of contact — `scale(.15) → scale(3.2)`, opacity `.85 → 0`, 650ms `ease-out`. Extracted from `physics.css`. Reserved for consequential commits: send, transfer, mint, enact, install. Not for every click. Scarcity is what makes it mean something.

### 4.4 Ambient layers (always present, never distracting)

- **Grain.** A tiled noise overlay at 3–4% opacity, animated on a 4-step 8s translate cycle (extracted `@keyframes noise`). Applied above surfaces, below text. Kills banding in the dark gradients and gives the field physical texture. GPU-composited; one layer per viewport, not per element.
- **Orbits.** Three concentric rings, `30s` / `23s` reverse dashed / `16s`, with a 4px signal node on each, plus a rotating diamond core (`45° → 225°`, `scale 1.5`, `4s`). Extracted. Reserved for hero, boot sequence, loading, and empty states — never behind dense content.
- **Grade.** `filter: saturate(1.16) contrast(1.04)` on the root (extracted). Subtle, and the reason the palette photographs so well.

### 4.5 Reduced motion

The site already does this correctly and it is now a hard requirement (P10/N8):

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation: none !important;
    transition: none !important;
    scroll-behavior: auto !important;
  }
}
```

Plus: pulses become static filled dots, orbits become static rings, ripples become a 90ms opacity flash, magnetic hover and field glow disable entirely. Every state must remain distinguishable **without** motion — motion is never the sole carrier of meaning.

---

## 5. Form Language

### 5.1 Corner ticks (the signature frame)

Extracted from `.form-shell`. A panel is not a box with a border; it is a **region marked at its corners**.

```css
.fn-panel::before { top:-1px; left:-1px;  border-top:1px solid var(--fn-signal); border-left:1px solid var(--fn-signal); }
.fn-panel::after  { right:-1px; bottom:-1px; border-right:1px solid var(--fn-signal); border-bottom:1px solid var(--fn-signal); }
/* both: position:absolute; width:14px; height:14px; content:"" */
```

Diagonal opposition — top-left and bottom-right — not four corners. Four corners is a targeting reticle; two is a **registration mark**. Registration marks are the language of instruments and printing plates, which is exactly the register this brand occupies.

Three panel intensities: **Tick** (corner marks only, on `--fn-surface-2`), **Framed** (ticks + full `--fn-line` border, for inputs and modals), **Sealed** (ticks + border + the dual-tone rim light below, for the Wallet, Charter, and other high-consequence surfaces).

### 5.2 Dual-tone rim light

Extracted from `physics.css`. Elevated surfaces are lit from two directions in two hues:

```css
box-shadow:
  -4px -4px 18px rgba(93,195,255,.20),      /* electric, upper-left */
   4px  4px 18px rgba(161,109,255,.20),     /* field violet, lower-right */
   0 28px 90px rgba(0,0,0,.44);             /* the actual depth */
```

Two light sources of different temperature is what makes the surfaces look **physical rather than flat-dark**. It is the most expensive-looking three lines of CSS in the system.

### 5.3 The active rail

Extracted: `box-shadow: inset 2px 0 0 rgba(140,232,223,.65)`. Selection is a 2px inset rail on the leading edge, plus a 4% signal wash. Not a fill, not a pill, not a border. It scales identically from a nav item to a table row to a CLI list, and it works in a terminal (`▌`). This is the selection idiom everywhere.

### 5.4 Radius and geometry

```
--fn-radius-0   0px    panels, inputs, buttons, tables — the default
--fn-radius-1   2px    chips, small controls
--fn-radius-2   4px    avatars-in-lists, thumbnails
--fn-radius-pill 999px  status pills only
--fn-radius-full 50%    avatars, signal dots, orbit nodes
```

**The system is fundamentally square.** The site uses zero border radius on every structural element. Roundness signals friendliness and consumer software; squareness signals precision and instrumentation. Radius is spent only where roundness carries information (a face, a dot, a state pill).

The one recurring non-square form is the **diamond** — a square rotated 45°, extracted from `.core`. It is the Fractal Node primitive shape: node, sigil, marker, bullet, loading indicator.

### 5.5 Spacing

4px base unit; the scale is `0, 2, 4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96, 128`. Layout gutters use `vw` at marketing scale (extracted `4vw` / `8vw` / `12vw`) and fixed px inside product surfaces. Product density has three modes — **Comfortable** (default), **Compact** (−20% vertical, for power users and ultrawides), **Dense** (−35%, tables and logs only).

---

## 6. Iconography

- **Grid:** 24×24, 1.25px stroke, square caps, square joins, no fills except state dots.
- **Construction:** built from the lattice — straight lines, 45° diagonals, circles, and diamonds. No organic curves, no rounded corners, no duotone.
- **Weight match:** the stroke weight is tuned to sit beside Manrope 300 and DM Mono 400 without dominating.
- **The Sigil.** The site's `⌁` glyph and the diamond core are the two brand marks. The product sigil is the **rotating diamond with a signal node**: a diamond outline in `--fn-signal` with a 4px filled node on its upper-left vertex. It is the app icon, the favicon, the CLI boot glyph, the loading state, and the Society default avatar base.
- **Society sigils** are procedurally generated from the Society's FNID: a deterministic lattice figure (3–7 nodes connected by straight and 45° segments, inscribed in a diamond) in the Society's accent hue. Every Society gets a unique, recognizable, non-uploadable-by-default mark on creation. Custom uploads unlock at Society Level 3.

---

## 7. Voice

### 7.1 Principles

**Terse. Declarative. Never cute.** The extracted copy establishes the exact register:

> "Not another platform. Not another tool. A signal becoming structure."
> "The system is listening."
> "Leave a trace in the system. When the threshold opens, the signal will find you."

Three techniques are doing the work, and they are now house style:

1. **Negation → assertion.** Clear the space, then place the thing. *"Not a wallet. An instrument of record."*
2. **Second-person as operator, not customer.** The reader *does* things to the system. Never "we'll help you" — *"You define the policy. The agent executes."*
3. **Sentence fragments as structure.** Short. Load-bearing. Never breathless.

### 7.2 The classification register

Machine-facing text uses the site's structural grammar verbatim:

- Section indices: `[ 02 / DEVELOPMENT RELAY ]`
- Namespaced paths: `FN://DEV-RELAY`, `FN://SOCIETY/ORACLE-HALL`
- Qualified labels: `SECURE // 256`, `ACCESS TERMINAL // FRACTAL NODE`
- Numbered fields: `01 / NAME`, `02 / EMAIL`
- Status words, always mono uppercase, from a **closed vocabulary**:
  `LIVE · SAFE · PASS · OK · STANDBY · SANITIZED · PENDING · SEALED · DORMANT · SLASHED · FRACTURED · DEGRADED · OFFLINE`

Adding a status word requires adding it here. An open-ended status vocabulary is how interfaces become unreadable.

### 7.3 Register by surface

| Surface | Register | Example |
|---|---|---|
| Marketing | Manifesto | "Fracture the Node." |
| Onboarding | Direct, warm, still precise | "Name your Society. You can change it once, within 14 days." |
| Product UI | Plain, minimal, zero personality | "Transfer 40 FRC to @kaya" |
| Errors | Cause, then remedy, no apology | "Transfer refused — Envelope permits 100 FRC/day, 140 requested. Raise the limit in Policy, or send 60 FRC." |
| CLI / logs | Classification register | `[ WALLET ] transfer 40.000000000 FRC → @kaya · OK · block 44812` |
| Governance | Formal, unambiguous | "Enacted. Charter v7 supersedes v6 at block 44812." |

**Errors never apologize and never blame.** They state what happened, why, and the next action. "Oops! Something went wrong" is a banned string, enforced by lint.

### 7.4 Naming things

Product and feature names come from the canonical terminology (`01`) and stay in that world: Society, Chamber, Convergence, Crystallization, Fracture, Citizen, Charter, Treasury, Vault, Custodian, Shard, Facet, Envelope, Signal, Relay, Standing. **No cute names.** No "Sparkle," no "Buddy," no exclamation marks. The one permitted flourish is the `//` separator and the `⌁` sigil.

---

## 8. The Marketing Package (derived from this system)

Everything below is generated from the same tokens — no separate marketing design language exists.

| Asset | Spec |
|---|---|
| **Wordmark** | `FRACTAL NODE` in DM Mono 400, `.18em` tracking, uppercase. Short form `FN`. Lockup `⌁ FN // <context>` (extracted from the site's `FN//03` topbar). |
| **App icon** | Diamond sigil, `--fn-signal` stroke on `--fn-void`, with the dual-tone rim light. Square canvas, no rounding (the OS applies its own mask). |
| **Social cards** | 1200×630. Void field + grain + one orbit arc bleeding off the right edge + display-2 headline (one word solid, one outlined) + `[ NN / SECTION ]` eyebrow in signal. |
| **Motion bumper** | 1.8s: grain resolves → orbits spin up → core diamond rotates 45°→225° and scales → wordmark letter-spaces in from `.4em` to `.18em` → pulse. Reuses the exact boot sequence from `31-cli-and-terminal.md`, which is the point: the ad *is* the product's boot. |
| **Pitch deck** | Void slides. `[ NN / SECTION ]` indices. One idea per slide. Display type for statements, DM Mono for every number. |
| **Documentation site** | Product tokens at reading density; `body-l` at 15/1.65; code in DM Mono; sidebar uses the active rail. |
| **Merch / print** | Void stock, signal foil for the sigil, DM Mono classification blocks. Registration-mark corners on every printed piece — literally what they are for. |

---

## 9. Anti-Patterns

Things that would break this identity, listed so no agent or contractor reinvents them:

- Gradients as decoration. Gradients here are *light*, not paint. Only radial glows and the two-tone rim.
- Purple-to-pink SaaS gradients, glassmorphism cards, or neumorphism. All of it reads as 2021 template.
- Emoji in product UI. The sigil and the closed icon set carry that load.
- Rounded corners on structural surfaces.
- Green for money, red/green for up/down without a second cue (colorblind failure, and it makes the economy look like a trading app).
- Illustration of any kind — no mascots, no isometric people, no blob shapes. The system's imagery is geometry and real media.
- More than three accent hues visible on one screen.
- Motion without meaning. Every animation states a fact: this is live, this is arriving, this committed, this failed.
- Any typeface that is not Manrope or DM Mono.

---

## 10. Governance of the Brand

The design tokens in `32-design-system.md` are the **single source of truth**, compiled to CSS custom properties, Rust constants, Swift, Kotlin, and an ANSI/truecolor palette for the CLI (N7). A color that is not a token cannot ship — enforced by a stylelint rule banning literal hex values outside the token file. Changing a core extracted value (§2.1, §4.1) requires an ADR.
