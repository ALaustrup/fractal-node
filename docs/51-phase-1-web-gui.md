# 51 — Phase 1 Execution Spec: The Web GUI

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), then `32-design-system.md`, `33-brand-identity.md`, `30-api-and-sdk.md`, `34-client-platform-strategy.md`, `41-repo-and-crate-structure.md`, `42-source-control-automation.md`, and `50-roadmap-phases.md` PH1.
> **Governs:** every screen, route, component, string, interaction, budget, and Work Unit of Milestones **M1.7 (design system v1)** and **M1.8 (web GUI)**. Where this chapter is more specific than `50`, this chapter is the instruction. Where it contradicts the Canon or `32`/`33`, the Canon and the design system win and this chapter is a defect.
> **Audience:** the AI coding agents that will execute M1.7 and M1.8, and the human who reviews their Work Units.

---

## 1. Position

`50` PH1 states the goal in one sentence and gives eleven Milestones. Two of them — M1.7 and M1.8 — are the front end, and they are the phase's long pole: the design system is the critical path (`50` PH1, dependency note), and the web GUI is the only surface an external Citizen will ever see at the exit gate. Everything else in PH1 is infrastructure that becomes visible through this chapter.

This document exists because the failure mode for a GUI milestone executed by agents is not bad code. It is **plausible ambiguity**: forty reasonable local decisions about routes, states, copy, and cache policy that add up to an application with no single mind behind it. `00 §0` names that disease. The counter-measure is to resolve every decision here, in advance, in writing, and to make the unresolved ones explicit with a default so that work is never blocked on a question (§16).

Three properties of PH1 shape everything below:

1. **There is no client core yet.** `50` M2.1 puts the local SQLite replica, the replicated event log, the outbox, and the sync engine in PH2. PH1 therefore builds against the public HTTP and WebSocket API only (§10.1). This is the single most consequential scope ruling in this chapter, and §2.2 R3 states it as law.
2. **There are no agents, no media, no Vault, no marketplace, and no governance beyond the Founder role.** Every surface below is drawn to be *correct without them* and *extensible to them* — most visibly in `MessageRow`, which ships its agent variant in PH1 even though no Agent can author a Message until PH3 (§6, §16 Q10).
3. **The GUI is not the product's only face.** P13 makes the CLI a peer. Every action specified here has a `fn` equivalent named in the screen inventory, and the command palette (§3.4) is the GUI's expression of that same command set.

---

## 2. Scope

### 2.1 In scope / out of scope

Derived from `50` PH1 Milestones and the `02 §3` deferral table. "Out" means: an agent that builds it has committed a scope violation regardless of how easy it looked (`02 §3`).

| Area | In PH1 | Out of PH1 | Lands in |
|---|---|---|---|
| Identity | Passkey registration, passkey sign-in, device-code fallback, Handle claim, session refresh, sign-out, device list (read + revoke) | Password of any kind (never), OIDC, multi-Persona switching UI, recovery *execution* | never / PH2 / PH3 |
| Recovery | Guardian **configuration** surface (`12 §6`), reachable from Settings and prompted once at 24h | The recovery *ceremony* (delay-and-notify, guardian approval UI) | PH2 |
| Society | Create, read, leave, settings (name, topic, accent, visibility), member list, role assignment from the Charter's fixed roles | Fracture, Fork, Dissolution, Crystallization, federation, transfer of founding role | PH5 / PH5 / PH2 |
| Charter | Read, diff between versions, Founder-only enact of role/permission changes | Proposals, votes, delegation, quorum, moderation policy editor | PH5 |
| Discourse | Text Chambers, Threads, Messages, revise, redact, reactions, presence, typing, read marks | Voice, Stage, Gallery, Board, Canvas, Experience Chambers; DMs; Convergences; threads-in-threads beyond one level | PH4 / PH2 / PH3 |
| Composer | Plain text, mention autocomplete for Citizens, reply-to, ⌘Enter commit, draft persistence | File attachment, image paste, rich text, slash commands, Agent mention, emoji picker | PH2 / PH3 |
| Realtime | Signal WebSocket, subscribe, resume, Gap recovery, presence, typing, `Shed`/`DEGRADED` surfacing | Push notifications, Service Worker, background sync | PH2 |
| Wallet | Citizen Wallet, Society Treasury, balance (settled vs pending), Transfer, Posting history, `ContributionReceipt` | Stake, Facets, fiat, external rails, exchange, invoices, recurring transfers | PH4+ / never in PH1 |
| Progression | XP, Level 0–12, Trust, Standing, Achievements, Unlock gates rendered as reasons | Seasons, Insignia tiles, leaderboards (never — `02 §4` engagement), Standing history charts | PH6 / PH3 |
| Profile | Own Profile, other Citizen's Profile, Handle, Level, Trust, Achievements, shared Societies | Profile Modules, custom avatars, themes, media galleries | PH2 |
| Discovery | In-Society search over Messages and Members; a Society directory listing *public* Societies by name | Interest matching, recommendations, ranking of any kind (never — `02 §4`) | PH5 / never |
| Shell | Society Rail, Chamber list, Stage, Context panel, status bar, command palette, Signal inbox, settings | Multi-pane Stage (`ultra`/`panorama`), tray, deep OS integration, window management | PH2 |
| Responsive | `handheld` and `standard` breakpoints correct and tested; `compact` correct by construction | `wide`, `ultra`, `panorama` layouts; gamepad focus; density auto-selection | PH2 |
| Themes | `void` (default) and `contrast` | `daylight`, Society accent applied beyond the sigil and rail, Profile themes | PH2 / PH2 / PH6 |
| Offline | Last-known-good rendering for the six read surfaces in R4, staleness indicators, one queued write class (R5) | General outbox, CRDT merge, conflict surfacing, local event log | PH2 |
| Platform | Web at `https://app.fractalnode.org` | PWA, Service Worker, install prompt, desktop shell, mobile apps | PH2 / PH2 / PH5 |

### 2.2 The rulings

Ten decisions an agent could plausibly get wrong. Each is law for PH1.

**R1 — The GUI talks to the public API and nothing else.** No direct database access, no internal path, no privileged header. Every call goes through `@fractal/api-client` or the Signal socket wrapper (`41 §10.1`). `fetch` appearing anywhere in `apps/web/src/**` outside `core/transport/` is a lint error and a P3 falsification.

**R2 — Every action in the GUI has a named CLI equivalent, recorded in the screen inventory (§4).** A screen whose primary action has no `fn` verb is a P13 violation and blocks the release tag, not the Work Unit.

**R3 — No wasm core in PH1.** `34 §4.2` describes the Phase-1 web target as loading `fractal-core-wasm` with an OPFS-backed SQLite replica. `50` M2.1 places that core in PH2, and `32 §8` caps initial JS at 180KB gzip against a wasm payload `34` itself measures at 1.4–1.8MB gzip. Those cannot both be true in PH1. **PH1 ships no wasm.** The web app is a server-state client with a persisted read cache (§10.3). See §16 Q3; the alternative — pulling M2.1 forward — was rejected because it makes M1.8 depend on the phase's largest unwritten component.

**R4 — Six read surfaces must render offline from last-known-good, with staleness.** Society list, Chamber list for each joined Society, the last 200 Messages of each Chamber opened this session, Wallet balance, own Profile and progression, and the Charter of each joined Society. This is P2's falsification test scoped to what PH1 can honestly hold. Any other surface offline renders the `OFFLINE` empty state with the strings in §9, never an error.

**R5 — Exactly one write class queues offline: `chamber.message.post`.** Queued Messages render at 70% opacity with a pending mark (`32 §6`), carry an `Idempotency-Key` generated at enqueue, and reconcile on reconnect. Every other write is refused offline with a cause-and-remedy string. Building a general outbox in PH1 is building M2.1 early.

**R6 — Wallet writes are never optimistic.** `10 §6` is unambiguous. The `TransferSheet` commits to a `PENDING` state that only a `ledger.transfer.settled.v1` Signal or an authoritative re-read can clear. No local balance arithmetic exists in the front end, at all — the balance is a number the API returns, never a number the client computes.

**R7 — No raster images ship in PH1.** Avatars and Society sigils are procedurally generated inline SVG from the FNID (`33 §6`). Custom avatars need Level 2 and the Vault; the Vault is PH2. The image policy in §11.4 is therefore "there are no images", which is a budget, not an omission.

**R8 — No Service Worker.** A Service Worker without the PH2 sync engine caches an application whose data model it cannot reason about. `50` M2.9 owns the PWA. Adding one early converts a stale-data bug into a stale-*application* bug.

**R9 — Statuses come from the closed vocabulary.** `LIVE · SAFE · PASS · OK · STANDBY · SANITIZED · PENDING · SEALED · DORMANT · SLASHED · FRACTURED · DEGRADED · OFFLINE` (`33 §7.2`). `StatusPill` accepts a union type generated from that list. A new status requires an amendment to `33`, in the same Work Unit.

**R10 — Every error string is rendered from the API's problem document, not composed in the client.** `30 §6` guarantees house voice in `detail` and structure in `cause`/`remedy`. The client maps `code` → layout and affordance (§10.7), never rewrites the prose. The only client-authored strings are the ones in §9, which are for states the network never sees (offline, empty, in-flight, keyboard help).

### 2.3 Complexity budget check

`02 §5` allows one new client platform per phase. PH1 adds exactly one: web. No new top-level service is introduced by the front end. Third-party runtime dependencies claimed by `apps/web` against the phase's allowance of five: **React, TanStack Router, TanStack Query, Zustand, `idb-keyval`**. That is the whole list and it is the whole budget. Vite, TypeScript, Vitest, Playwright, and Storybook are build- and test-time and do not ship. `@fractal/design-system`, `@fractal/api-client`, `@fractal/event-types`, and `@fractal/tokens` are first-party. **Any sixth runtime dependency requires something to leave** (`02 §8`).

---

## 3. Information Architecture

### 3.1 The route table

TanStack Router, file-based, under `apps/web/src/routes/` (`41 §10.1`). Path parameters are typed; a route file contains routing only.

| Route | Screen | Auth | Chunk |
|---|---|---|---|
| `/` | Boot → redirect to last Society, or `/enter` | — | entry |
| `/enter` | Sign-in | anon | entry |
| `/enter/new` | Registration | anon | entry |
| `/enter/device` | Device-code fallback | anon | `enter` |
| `/welcome/handle` | Onboarding 1 — claim Handle | session | `welcome` |
| `/welcome/society` | Onboarding 2 — found first Society | session | `welcome` |
| `/s/$society` | Society root → redirect to default Chamber | session | entry |
| `/s/$society/c/$chamber` | Chamber (Stage) | session | entry |
| `/s/$society/c/$chamber/t/$thread` | Thread (Stage) | session | entry |
| `/s/$society/members` | Member list (Stage) | session | `society` |
| `/s/$society/charter` | Charter view + diff | session | `society` |
| `/s/$society/settings` | Society settings | session | `society` |
| `/s/$society/treasury` | Society Treasury wallet | session | `wallet` |
| `/societies` | Society directory (public Societies) | session | `society` |
| `/societies/new` | Create-Society flow | session | `society` |
| `/join/$invite` | Join-Society flow | session | `society` |
| `/wallet` | Citizen Wallet | session | `wallet` |
| `/wallet/transfer` | Transfer flow | session | `wallet` |
| `/wallet/history` | Posting history | session | `wallet` |
| `/wallet/history/$posting` | `ContributionReceipt` / Posting detail | session | `wallet` |
| `/@$handle` | Profile (own or other) | session | `profile` |
| `/@$handle/progression` | Progression / XP · Trust · Standing | session | `profile` |
| `/search` | Search (scoped) | session | `search` |
| `/signals` | Notification centre (Signal inbox) | session | `signals` |
| `/settings` | Citizen settings (5 tabs) | session | `settings` |
| `/offline` | Offline page (hard, no cached shell) | any | entry |
| `/error/$code` | Terminal error page | any | entry |
| `*` | Not found | any | entry |

`/s/$society` takes a **Handle**, not a `soc_` ULID: `FN://ORACLE-HALL/GENERAL` is the breadcrumb (`33 §7.2`) and the URL must agree with it. The client resolves handle → `SocietyId` once per session and caches it; a `soc_…` in the path position is accepted and 301-redirected to the canonical handle form so that identifiers pasted from the CLI work.

### 3.2 Deep links

One canonical `https` URL per addressable object. `fractal://` is the same address with the scheme swapped and the shorthand segments expanded to canonical nouns — mechanically derivable in both directions, which is what lets `fn ... o` (`31 §7.2`) hand an object to the GUI without a lookup table.

| Object | `https` (canonical) | `fractal://` |
|---|---|---|
| Society | `/s/oracle-hall` | `fractal://society/oracle-hall` |
| Chamber | `/s/oracle-hall/c/general` | `fractal://society/oracle-hall/chamber/general` |
| Thread | `/s/oracle-hall/c/general/t/thr_01H9B…` | `fractal://society/oracle-hall/chamber/general/thread/thr_01H9B…` |
| Message | `…/t/thr_01H9B…#m-01HYQ3MC…` | `fractal://…/thread/thr_01H9B…/message/01HYQ3MC…` |
| Citizen | `/@kaya` | `fractal://citizen/kaya` |
| Posting | `/wallet/history/01HYQ3MD…` | `fractal://posting/01HYQ3MD…` |
| Charter version | `/s/oracle-hall/charter?v=7` | `fractal://society/oracle-hall/charter/7` |
| Invite | `/join/inv_01HYQ…` | `fractal://join/inv_01HYQ…` |

PH1 registers `web+fractal:` via `registerProtocolHandler` (`34 §4.2`) behind an explicit opt-in in Settings — never on first load, which browsers correctly treat as hostile. Native `fractal://` handling belongs to the desktop shell in PH2. An unauthenticated deep link stores the target in `sessionStorage`, routes to `/enter`, and resumes after sign-in; a deep link to an object the Citizen may not read renders the `not_found` surface, never an existence oracle (`30 §6.5`).

### 3.3 Back-button semantics

| Interaction | History |
|---|---|
| Change Society, Chamber, Thread; open Profile, Wallet, Charter, Members | **push** |
| Open the command palette, a `ContextMenu`, a tooltip, a toast | **none** — the palette is not a route and Esc closes it |
| Open `TransferSheet`, create-Society, join-Society, any cancelable modal | **push**, so Back reads as Cancel |
| Advance an onboarding step | **push**; Back returns to the prior step with entered values intact |
| Toggle the Context panel, change density, switch a settings tab | **replace** |
| Type in search | **replace**, debounced 250ms |
| Submit search | **push** |
| Commit a Transfer, create a Society, post a Message | **replace** the modal entry, so Back cannot re-open a committed flow |

The last row is the one that matters. A committed consequential action must not be re-enterable by Back — the history entry is replaced with the resulting surface at the moment the command is accepted, before settlement.

### 3.4 The command palette

`32 §5.3`: *every action in the product is reachable here; it is the GUI's expression of P13.* That is a contract with a testable shape.

- The palette's command set is **generated** from the same schema that generates the CLI command tree (`30 §12.2`, `31 §8` `fn schema`). Each entry carries: id, canonical title, the `fn` invocation it mirrors, required capability, and target route or command.
- A command the Citizen cannot currently perform is **shown, disabled, and annotated with the reason** — the Level, Trust, Charter role, or Envelope that is missing. Hiding it would teach nothing; `30 §6.4` `unlock_required` already names the gate, and the palette renders the same sentence.
- Sections, in fixed order: `NAVIGATE`, `CREATE`, `VALUE`, `GOVERN`, `VIEW`, `SYSTEM`. Recent commands (max 5, per device, `localStorage`) sit above them under `RECENT`.
- Typing `>` restricts to commands; `@` to Citizens; `#` to Chambers; `$` to Wallet actions; `/` to search. Empty input shows `RECENT` then `NAVIGATE`.
- A parity test asserts that the set of palette command ids equals the set of CLI leaf commands minus an explicitly declared allowlist (commands with no GUI meaning, e.g. `fn config show`). The allowlist lives in the repo and requires a review to grow.

---

## 4. Screen Inventory

Every screen states its four data states (`32 §6`). Where a cell says "n/a" the state cannot occur — for example a screen with no remote read has no stale state. Endpoint numbers reference the `30 §4.3` table.

### 4.1 Entry and onboarding

| Screen | Route | Purpose | Data | States | Primary action (CLI) | Keyboard |
|---|---|---|---|---|---|---|
| Boot | `/` | Resolve session, restore last Society, hand off | `GET /v1/citizens/me` (1) | loading: sigil + orbit, no text before 400ms · error: `/error/boot` · stale: renders cached shell · empty: n/a | — | Esc aborts to `/enter` |
| Sign-in | `/enter` | Authenticate an existing Citizen | `POST /v1/sessions` (passkey assertion) | loading: button in-flight · error: inline under the field · empty/stale: n/a | Sign in (`fn auth login`) | Enter submits; `⌥D` device fallback |
| Registration | `/enter/new` | Create a Citizen and claim a Handle | `POST /v1/handles` (3), `POST /v1/citizens` | loading · error: `validation_failed`, `conflict` inline · empty/stale: n/a | Claim and continue (`fn auth register`) | Enter submits; Tab order: handle → passkey → alt path |
| Device fallback | `/enter/device` | Passkey-less path on unsupported browsers | `POST /v1/sessions/device` (`30 §7.2`) | loading: code + `SignalDot` pulse · error inline · stale: n/a | Continue on another device | `⌘C` copies the code |
| Onboarding 1 | `/welcome/handle` | Confirm Handle, set display name | `PATCH /v1/citizens/me` (2) | loading · error inline · empty/stale: n/a | Continue (`fn citizen persona set`) | Enter continues; Esc is disabled here |
| Onboarding 2 | `/welcome/society` | Found the first Society | `POST /v1/societies` (6) | loading: commit ripple then progress line · error: inline, cause + remedy · empty/stale: n/a | Found the Society (`fn society create`) | Enter commits; Back returns with values |

### 4.2 The Society shell

The shell (`32 §4.1`) is one persistent layout; the Stage swaps. Society Rail, Chamber list, status bar, and topbar are resident on every route below and are not re-mounted on Stage navigation.

| Screen | Route | Purpose | Data | States | Primary action (CLI) | Keyboard |
|---|---|---|---|---|---|---|
| Society Rail | resident | Cross-Society navigation | `GET /v1/societies?member=me` (5) | loading: 3 skeleton sigils · empty: a single `+` tile · error: rail keeps last-known-good, status bar shows `DEGRADED` · stale: unread dots dim | Switch Society (`fn society open`) | `↑/↓` roving; `1–9` jump; Enter select |
| Chamber list | resident | Spaces in the current Society | `GET …/chambers` (14 read side) | loading: 5 skeleton rows · empty: "No Chambers yet." + Create · error inline in panel · stale: rendered from cache | Open Chamber (`fn chamber open`) | `↑/↓`, Enter, `[`/`]` prev/next |
| Chamber | `/s/$s/c/$c` | Read and post in a text Chamber | `…/threads` (15), `…/messages` (17), Signal `Society`+`Chamber` scopes | loading: 8 `MessageRow` skeletons · empty: first-message state (§5.4) · error: banner above composer, history stays readable · stale: `STALE 4m` in status bar, composer still live | Post (`fn chamber post`) | `⌘Enter` send; `j/k` move; `r` reply; `e` revise own; `⌥↑` edit last |
| Thread | `…/t/$thread` | A focused reply chain | `…/threads/{tid}/messages` (17) | as Chamber; empty cannot occur (a Thread has ≥1 Message) | Reply (`fn chamber post --thread`) | as Chamber; Esc returns to Chamber |
| Composer | inside Chamber/Thread | Author a Message | `POST …/messages` (18), `Idempotency-Key` required | loading: send disabled, row appears at 70% · error: row stays with a retry affordance · offline: queued (R5) · empty: placeholder copy | Send | `⌘Enter` send; `↑` edit last; `@` mention; Esc clears reply-to |
| Member list | `/s/$s/members` | Who is here, with Standing | `…/memberships` (11) | loading: table skeleton · empty: impossible (Founder exists) · error inline · stale: cache + banner | Assign role (`fn society role set`) | `↑/↓`, Enter opens Profile, `/` filters |
| Society settings | `/s/$s/settings` | Name, topic, accent, visibility, join policy | `GET/PATCH` society (7) with `If-Match` | loading: field skeletons · error: `precondition_failed` → "Reload; this Society changed." · stale: fields locked read-only · empty: n/a | Save (`fn society set`) | `⌘S` saves; Esc discards with confirm |
| Charter | `/s/$s/charter` | Read governance; diff versions | `…/charter` (8), `…/charter/versions` | loading: clause skeletons · empty: n/a · error inline · stale: read-only with `STALE` | Enact (Founder) (`fn charter enact`) | `d` diff mode; `←/→` version; `⌘Enter` enact |
| Create Society | `/societies/new` | Found a further Society | `POST /v1/societies` (6) | loading: 3-line progress · error: `unlock_required` renders the gate · empty/stale: n/a | Found (`fn society create`) | Enter commits |
| Join Society | `/join/$invite` | Accept an invite, accept the Charter | `GET` invite, `POST …/memberships` (12) | loading · empty: "This invite is spent." · error: `charter_forbids` inline · stale: n/a | Join (`fn society join`) | Checkbox must be focused and checked before Enter arms |
| Society directory | `/societies` | Browse public Societies | `GET /v1/societies?visibility=public` (5) | loading: 6 `SocietyCard` skeletons · empty: "No public Societies match." · error inline · stale: cached list | Open / Request to join | `↑/↓/←/→` grid, Enter |

### 4.3 Value, identity, and system

| Screen | Route | Purpose | Data | States | Primary action (CLI) | Keyboard |
|---|---|---|---|---|---|---|
| Wallet | `/wallet` | Citizen balance, settled vs pending | `GET /v1/wallets/{wid}` (24) | loading: tabular skeleton, never a spinner over a number · empty: balance 0 with "Nothing has moved yet." · error: last-known balance + `DEGRADED` · stale: `AS OF 4m AGO` beside the figure | Transfer (`fn wallet transfer`) | `t` transfer; `h` history; `⌘R` re-read |
| Treasury | `/s/$s/treasury` | Society Wallet | as Wallet, Society-scoped | as Wallet | Transfer from Treasury (role-gated) | as Wallet |
| Transfer | `/wallet/transfer` | Move Fraction | `POST /v1/transfers` (25), `Idempotency-Key` **required**, elevated context above threshold | loading: `PENDING`, sheet stays open · error: `insufficient_funds`, `capability_denied` inline with remedy · offline: refused, not queued (R6) · stale: n/a | Commit (`fn wallet transfer`) | Tab: amount → recipient → memo → commit; `⌘Enter` commits; Esc cancels while un-armed only |
| Posting history | `/wallet/history` | Every Posting, both sides | `GET …/postings` (26), cursor paging | loading: dense skeleton rows · empty: "No Postings." · error inline · stale: cached page + banner | Open Posting (`fn wallet history`) | `j/k`, Enter, `⇧G` end, `/` filter |
| Posting detail / `ContributionReceipt` | `/wallet/history/$p` | Why this Fraction moved | posting + `GET …/contribution/{ref}` | loading · empty: n/a · error inline · stale: n/a | Copy reference | `y` copies the reference; Esc back |
| Profile (own) | `/@$handle` | Identity home | `GET /v1/citizens/me` (1), progression (37) | loading: skeleton · empty: no Achievements → outlined-word empty block · error inline · stale: cached | Edit Persona (`fn citizen persona set`) | `e` edit; `p` progression |
| Profile (other) | `/@$handle` | Another Citizen | `GET /v1/citizens/{fnid}` | loading · empty: shared-Societies list may be empty · error: `not_found` · stale: cached | Transfer to (`fn wallet transfer @h`) | `t` transfer; `m` message where a shared Society exists |
| Progression | `/@$handle/progression` | XP, Level, Trust, Standing, Achievements, next Unlocks | `GET …/progression` (37) | loading: meter skeleton · empty: Level 0 renders as a real state, not an empty one · error inline · stale: cached | View a gate's requirement | `↑/↓` across Unlock rows |
| Search | `/search?q=` | Find Messages, Citizens, Chambers in scope | `GET /v1/search` (41), scoped | loading: result skeletons · empty: "No matches for …" + scope hint · error inline · stale: n/a | Open result | `↑/↓`, Enter, `⌥1–3` switch facet |
| Command palette | overlay | Every action | generated command set + (5), (11) for `@`/`#` | loading: inline spinner in the result row only · empty: "No command matches." · error: n/a · stale: n/a | Run command | `⌘K` open; `↑/↓`; Enter run; `⇥` complete; Esc close |
| Notification centre | `/signals` | Signal inbox | `GET /v1/societies/{sid}/events?since=` (38) per joined Society | loading: skeletons · empty: "Nothing has happened yet." · error inline · stale: cached, banner | Open the subject | `j/k`, Enter, `a` mark all read |
| Settings | `/settings` | Account · Devices · Appearance · Privacy · Recovery | (1), devices, local prefs | loading per tab · empty: no second device → prompt · error inline · stale: read-only | Save per section | `⌘S`; `⌃⇥` next tab |
| Offline | `/offline` | Hard-offline landing (no cached shell) | none | the only state | Retry | Enter retries |
| Error | `/error/$code` | Terminal failure | none | the only state | Reload / Sign out | Enter reloads |

### 4.4 The global keyboard contract

Mirrors `31 §7.2` so that muscle memory carries between the Terminal and the GUI (P13).

```
  ⌘K / Ctrl-K   command palette          ?         keymap overlay
  /             focus search             Esc       back · dismiss · cancel
  g s           go to Societies          g w       wallet        g p  profile
  g c           charter                  g m       members       g n  signals
  [ / ]         previous / next Chamber  1–9       nth Society on the rail
  j / k         move within a list       Enter     activate
  ⌘Enter        commit (send, transfer, enact)
  F6 / ⇧F6      cycle landmark regions   y         copy the focused value
  ⌘.            toggle the Context panel ⌘\        toggle density
```

`?` is available on every surface and lists the focused region's bindings first, then the global set — the GUI's equivalent of the Terminal's always-available keymap.

---

## 5. Wireframes

Box-drawing sketches for the eight surfaces that carry the phase. Dimensions match `32 §4.1`. `▌` is the active rail, `●` a pulsing signal dot, `◆` a Society sigil, `◇` a non-text Chamber (PH2) and, inside a Message row, a reaction chip, `◈` an Agent (PH3, fixture-only), `⟡` the agent-origin glyph, `▓░` the XP meter, `►` a disclosure.

### 5.1 The Society shell with a Chamber on the Stage

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ ⌁ FN // ORACLE-HALL              ⌘K  SEARCH            ● LIVE   1,204 FRC     @kaya  │ 44
├────┬──────────────────┬───────────────────────────────────────────┬──────────────────┤
│    │ ┌ 01 / CHAMBERS  │ ┌ FN://ORACLE-HALL/GENERAL ──────────── ⌘. │ ┌ 03 / CONTEXT   │
│ ◆  │ │                │ │                                          │ │              ┐ │
│ ▌  │ │▌# general    ● │ │  ────────────  TUE 03 SEP  ────────────  │ │ MEMBERS   14 │ │
│ ◇  │ │ # signal       │ │                                          │ │ ● @kaya      │ │
│ ◆  │ │ # treasury   2 │ │  ◉ @kaya            09:41                │ │ ● @rell      │ │
│    │ │                │ │    the relay is stable. holding at 14     │ │ ○ @juno      │ │
│ ◆  │ │ 02 / VOICE     │ │    custodians.                            │ │ ○ @sib       │ │
│    │ │ ◇ commons   ░  │ │    ◇ 2                                    │ │              │ │
│    │ │                │ │                                          │ │ PINNED       │ │
│    │ │ 03 / AGENTS    │ │  ◉ @rell            09:44                │ │ ⟡ charter v7 │ │
│ ◆  │ │ ◈ archivist ░  │ │    ▌ replying to @kaya                    │ │              │ │
│    │ │                │ │    then we can anchor tonight.            │ │ ACTIVITY     │ │
│    │ │                │ │                                          │ │ 09:44 posted │ │
│    │ │                │ │  ⟡ ◈ archivist      09:45   UNDER env_01H │ │ 09:41 posted │ │
│    │ │                │ │  ▌ anchor scheduled · block 44812         │ │ 09:12 joined │ │
│    │ │                │ │                                          │ │              │ │
│    │ │                │ │  ┌──────────────────────────────────────┐ │ │              │ │
│    │ ├────────────────┤ │  │ ›  message #general            ⌘⏎ →  │ │ │            ┘ │
│ 56 │ │ XP ▓▓▓▓▓░░ L7  │ │  └──────────────────────────────────────┘ │ └              │
├────┴──────────────────┴───────────────────────────────────────────┴──────────────────┤
│ FN://ORACLE-HALL/GENERAL · SYNCED 2s AGO · 14 CUSTODIANS · BLOCK 44812 · ⌘K          │ 24
└──────────────────────────────────────────────────────────────────────────────────────┘
  56px         240px                       fluid                          320px
```

Notes an implementer must honour: the agent row carries the violet rail, the `⟡` glyph, the violet handle, **and** the `UNDER env_…` affordance — four cues, never one (`32 §5.5`, P4). Unread on the rail is a 4px dot, never a count; the `2` beside `# treasury` is a *mention* count, which is a different fact and the only count the shell shows. The status bar is a live region only for transitions into `DEGRADED` and `OFFLINE`.

### 5.2 Registration

```
                    ┌ ACCESS TERMINAL // FRACTAL NODE ────────────────┐
                    │                                                ┐│
                    │        CLAIM                                    │
                    │        𝘺𝘰𝘶𝘳 𝘏𝘢𝘯𝘥𝘭𝘦     ← display-3, outlined     │
                    │                                                 │
                    │  01 / HANDLE                                    │
                    │  ┌───────────────────────────────────────────┐  │
                    │  │ @ kaya                                  ✓ │  │
                    │  └───────────────────────────────────────────┘  │
                    │  3–24 characters. Lowercase, digits, and _.     │
                    │  Yours permanently after 14 days.               │
                    │                                                 │
                    │  02 / KEY                                       │
                    │  ┌───────────────────────────────────────────┐  │
                    │  │  ◇  PASSKEY ON THIS DEVICE                │  │
                    │  │     Face, fingerprint, or security key.    │  │
                    │  └───────────────────────────────────────────┘  │
                    │  No password is created. There is nothing to    │
                    │  steal and nothing to reset.                    │
                    │                                                 │
                    │  ┌───────────────────────────────────────────┐  │
                    │  │            CLAIM AND CONTINUE          →  │  │
                    │  └───────────────────────────────────────────┘  │
                    │ ┘                                               │
                    │  No passkey on this browser?  USE ANOTHER DEVICE│
                    └─────────────────────────────────────────────────┘
                       SECURE // 256        FN://REGISTER        [ 01 / 03 ]
```

### 5.3 Onboarding 2 — found the first Society

```
                    ┌ FN://REGISTER ─────────────────────────────────┐
                    │                                               ┐│
                    │        NAME                                    │
                    │        𝘺𝘰𝘶𝘳 𝘚𝘰𝘤𝘪𝘦𝘵𝘺                             │
                    │                                                │
                    │  01 / SOCIETY NAME                             │
                    │  ┌──────────────────────────────────────────┐  │
                    │  │ Oracle Hall                              │  │
                    │  └──────────────────────────────────────────┘  │
                    │  Changeable once, within 14 days.              │
                    │                                                │
                    │  02 / ACCENT            03 / VISIBILITY        │
                    │  ◆ ◆ ◆ ◆ ◆ ◆           ┌────────┬───────────┐  │
                    │  ◆ ◆ ◆ ◆ ◆ ◆           │ PRIVATE│DISCOVERABLE│ │
                    │   ▲ selected            └────────┴───────────┘  │
                    │                         Private. Only invited   │
                    │                         Citizens can read it.   │
                    │                                                │
                    │  You will be its Founder. A Treasury, a Vault,  │
                    │  a Charter, and #general are created with it.   │
                    │                                                │
                    │  ┌──────────────────────────────────────────┐  │
                    │  │           FOUND THE SOCIETY           →  │  │
                    │  └──────────────────────────────────────────┘  │
                    │ ┘        ← BACK                                 │
                    └────────────────────────────────────────────────┘
                                                        [ 02 / 03 ]
```

### 5.4 First-message empty state (the onboarding payload)

```
│ ┌ FN://ORACLE-HALL/GENERAL ─────────────────────────────────────────────────┐ │
│ │                                                                          ┐│ │
│ │                              ·   ◇   ·                                    │ │
│ │                          ·               ·          static orbit,         │ │
│ │                              ⟡  ◈  ⟡                no rotation under     │ │
│ │                          ·               ·          reduced motion        │ │
│ │                              ·   ◇   ·                                    │ │
│ │                                                                           │ │
│ │                    NOTHING                                                │ │
│ │                    𝘩𝘢𝘴 𝘣𝘦𝘦𝘯 𝘴𝘢𝘪𝘥                                          │ │
│ │                                                                           │ │
│ │        #general is where this Society talks. Post the first Message       │ │
│ │        and the Chamber is live.                                           │ │
│ │                                                                           │ │
│ │ ┘                                                                         │ │
│ │  ┌────────────────────────────────────────────────────────────────────┐   │ │
│ │  │ ›  say something                                            ⌘⏎ →   │   │ │  ← autofocused
│ │  └────────────────────────────────────────────────────────────────────┘   │ │
│ └───────────────────────────────────────────────────────────────────────────┘ │
```

### 5.5 Wallet

```
┌ 01 / WALLET ─────────────────────────────────────────┐ ┌ 02 / RECENT ──────────┐
│                                                     ┐│ │                      ┐│
│   SETTLED                                            │ │ 03 SEP  09:44         │
│   1,204.000000000 FRC        ← data-m, tabular       │ │ ▌ − 40.000000000 FRC  │
│                                                      │ │   → @rell             │
│   PENDING                                            │ │   TRANSFER · SETTLED  │
│   40.000000000 FRC   PENDING                         │ │                       │
│   ────────────────────────────────────────           │ │ 03 SEP  08:02         │
│   AVAILABLE                                          │ │   + 12.000000000 FRC  │
│   1,164.000000000 FRC                                │ │   ← soc_oracle-hall   │
│                                                      │ │   CONTRIBUTION · OK   │
│   AS OF 2s AGO                                       │ │                       │
│                                                      │ │ 02 SEP  22:15         │
│  ┌──────────────────┐  ┌──────────────────┐          │ │   + 100.00000000 FRC  │
│  │    TRANSFER      │  │  HISTORY      ►  │          │ │   ← GENESIS · SEALED  │
│  └──────────────────┘  └──────────────────┘          │ │                       │
│ ┘                                                    │ │                     ┘ │
└──────────────────────────────────────────────────────┘ └───────────────────────┘
  Sealed panel: dual-tone rim + field glow              Framed panel, dense rows
```

Settled, pending, and available are three separate figures. A single "balance" is the shape that produces a Citizen who believes money they cannot spend is theirs, and `10 §6` forbids the arithmetic that would hide the difference.

### 5.6 TransferSheet

```
                 ┌ TRANSFER ────────────────────────────────────┐
                 │                                             ┐│
                 │  01 / AMOUNT                                 │
                 │  ┌───────────────────────────────┐ ┌──────┐  │
                 │  │ 40.000000000                  │ │ FRC  │  │
                 │  └───────────────────────────────┘ └──────┘  │
                 │  MAX 1,164.000000000        HALF   MAX       │
                 │                                              │
                 │  02 / TO                                     │
                 │  ┌───────────────────────────────────────┐   │
                 │  │ @rell                            ◉    │   │
                 │  └───────────────────────────────────────┘   │
                 │  Rell · Level 5 · Trust 210 · in this Society │
                 │                                              │
                 │  03 / MEMO                          optional │
                 │  ┌───────────────────────────────────────┐   │
                 │  │ shard repair                          │   │
                 │  └───────────────────────────────────────┘   │
                 │                                              │
                 │  ── BREAKDOWN ────────────────────────────   │
                 │  Amount            40.000000000 FRC          │
                 │  Fee                0.000000000 FRC          │
                 │  Debited           40.000000000 FRC          │
                 │                                              │
                 │  ▲ Transfers are final. There is no reversal. │
                 │                                              │
                 │  ┌──────────────┐  ┌───────────────────────┐ │
                 │  │   CANCEL     │  │  CONFIRM TRANSFER  ⌘⏎ │ │
                 │  └──────────────┘  └───────────────────────┘ │
                 │ ┘                                            │
                 └──────────────────────────────────────────────┘
                    Above the confirm threshold this sheet
                    requires a fresh passkey assertion (12 §5.2)
```

### 5.7 Command palette

```
        ┌──────────────────────────────────────────────────────────────┐
        │  ›  transfer                                              ⌘K │
        ├──────────────────────────────────────────────────────────────┤
        │  VALUE                                                       │
        │ ▌ Transfer Fraction                       fn wallet transfer │
        │   Open Wallet                             fn wallet          │
        │   View Posting history                    fn wallet history  │
        │  GOVERN                                                      │
        │   Transfer from Treasury                  fn wallet transfer │
        │     ▲ Requires the Steward role in this Society              │
        │  CREATE                                                      │
        │   Found a Society                         fn society create  │
        │     ▲ Requires Level 3. You are Level 0.                     │
        ├──────────────────────────────────────────────────────────────┤
        │  ↑↓ move   ⏎ run   ⇥ complete   esc close   > commands       │
        └──────────────────────────────────────────────────────────────┘
```

Disabled commands stay visible with their gate stated. This is the palette teaching the progression system, which is the only place in PH1 where a Citizen encounters `unlock_required` before they trip over it.

### 5.8 Profile and progression

```
┌ FN://CITIZEN/KAYA ───────────────────────────────────┐ ┌ 02 / PROGRESSION ─────┐
│                                                     ┐│ │                      ┐│
│    ◉        @kaya                                    │ │ LEVEL 7               │
│   ████      Kaya                                     │ │ XP ▓▓▓▓▓░░  4,120     │
│   ████      Joined 12 AUG 2026 · LIVE                │ │ 880 to Level 8        │
│                                                      │ │                       │
│  SOCIETIES                                           │ │ TRUST      210  SAFE  │
│  ◆ Oracle Hall     FOUNDER    L7                     │ │ ── separate from XP   │
│  ◆ Signal Works    MEMBER     L2                     │ │                       │
│                                                      │ │ NEXT UNLOCKS          │
│  ACHIEVEMENTS                                        │ │ L8  Custodian         │
│  ┌─────┐ ┌─────┐ ┌─────┐                             │ │ L8  Transfer ≤2,000   │
│  │  ◇  │ │  ◇  │ │  ◇  │  FIRST WORD · FOUNDER ·     │ │ T250 Guardian role    │
│  └─────┘ └─────┘ └─────┘  ANCHOR                     │ │                       │
│ ┘                                                    │ │ STANDING · ORACLE-HALL│
│  ┌──────────────┐  ┌──────────────┐                  │ │ Contribution ▓▓▓▓░    │
│  │   TRANSFER   │  │  PROGRESSION │                  │ │ Tenure       ▓▓░░░    │
│  └──────────────┘  └──────────────┘                  │ │ Governance   ▓░░░░  ┘ │
└──────────────────────────────────────────────────────┘ └───────────────────────┘
```

XP and Trust are never adjacent in a way that implies a sum, and the label `── separate from XP` is literal chrome, not a note to the implementer (`01 §7` hard rule).

### 5.9 Handheld (≤ 1100px)

The Rail collapses to a summonable sheet, the Context panel becomes a drawer, and the Stage goes full-bleed. Targets are ≥ 44px. The status bar survives — it is the most brand-defining chrome in the product (`32 §4.1`) and it is where the offline state lives.

```
┌──────────────────────────────┐
│ ◆  ORACLE-HALL / GENERAL   ⌘ │ 44
├──────────────────────────────┤
│  ◉ @kaya          09:41      │
│    the relay is stable.      │
│                              │
│  ⟡ ◈ archivist    09:45      │
│  ▌ anchor scheduled          │
│    UNDER env_01H…            │
│                              │
│                              │
│ ┌──────────────────────────┐ │
│ │ › message #general    →  │ │ 48
│ └──────────────────────────┘ │
├──────────────────────────────┤
│ OFFLINE · 1 QUEUED           │ 24  ← ember, content stays readable
└──────────────────────────────┘
```

---

## 6. Component Build Order

M1.7 delivers the LATTICE component set. `50` says "the 40 Phase-1 components"; the exact inventory is **forty-one**, enumerated below. `32 §5.2` and `§5.4` name components (`Slider`, `DatePicker`, `FilePicker`, `Sparkline`, `Chart`, `LogStream`, `InsigniaTile`, `Timeline`, `CodeBlock`) that PH1 has no surface for; they are deferred, and a Work Unit that builds one is out of phase. `SignalToast` is an addition to `32 §5.4` and its entry must be added to `32` in the same Work Unit that builds it (`02 §7`: no new UI pattern without a design-system entry).

**Every component in every wave requires all nine artifacts** (`32 §5`): anatomy, the eight states (`rest · hover · active · focus-visible · disabled · loading · error · empty`), a keyboard contract, an ARIA contract, a token map, a CLI/terminal equivalent, a visual regression test, a Storybook entry, and docs. A component with eight of nine does not exist and cannot be imported. This is enforced by a Storybook manifest check in CI, not by review attention.

| Wave | Components | Unblocks |
|---|---|---|
| **W0 — Field** | `ThemeProvider` · `Panel` (Tick/Framed/Sealed) · `Hairline` · `Rail` · `Grain` · `FieldGlow` · `Sigil` | everything; nothing else may start |
| **W1 — Atoms** | `Button` · `IconButton` · `TextField` · `TextArea` · `Checkbox` · `Switch` · `SegmentedControl` · `Skeleton` · `StatusPill` · `SignalDot` · `Avatar` · `Badge` | forms, entry screens |
| **W2 — Molecules** | `Combobox` · `SearchField` · `AmountField` · `Tabs` · `Breadcrumb` · `ContextMenu` · `Sheet` · `Drawer` · `SignalToast` · `EmptyState` · `MetricTile` · `Table` | modals, settings, history |
| **W3 — Navigation** | `SocietyRail` · `ChamberList` · `CommandPalette` | the shell |
| **W4 — Domain** | `MessageRow` · `Composer` · `WalletCard` · `TransferSheet` · `LedgerTable` · `XpMeter` · `ContributionReceipt` · `CharterView` · `SocietyCard` | every Stage surface |

Forty-one across five waves. W0 and W1 are strictly sequential; W2 may begin as soon as W1's `Button`, `TextField`, and `Skeleton` land; W3 depends on W2's `Sheet` and `ContextMenu`; W4 depends on W1 and W2 only, so it can run in parallel with W3 — which matters, because W4 is the largest wave and the one M1.8 blocks on.

Two components deserve a note. `MessageRow` ships the agent variant in PH1 with visual regression coverage driven by fixtures, because retrofitting the P4 visibility contract in PH3 means auditing every message surface twice (§16 Q10). `AmountField` is the only component allowed to touch quanta arithmetic, and it does so through a single shared formatter tested to the quantum (`11 §2.6`: 1 FRC = 1,000,000,000 quanta) — a second formatter anywhere is a P12 defect.

---

## 7. Interaction Specification

`33 §4.3` defines five signature behaviours. Applied to PH1's real surfaces, with exclusions stated because an unstated exclusion is an invitation.

| Element | Behaviour | Parameters | Reduced motion |
|---|---|---|---|
| Topbar connection dot | **Pulse** | 5px, 2s infinite, opacity 1→.25, glow `0 0 12px`→`0 0 2px`, `--fn-accent-action` | Static filled dot |
| Live Chamber marker in `ChamberList` | Pulse | as above, 4px | Static dot |
| Presence dot on `Avatar` (online) | Pulse | as above, 4px | Static filled vs hollow ring for offline |
| Composer "sending" indicator | Pulse | as above, on the send affordance only | Static + `SENDING` label |
| Status bar `SYNCED` glyph | Pulse | as above, 4px, suppressed above `STALE 60s` | Static |
| `Button` — primary variant only | **Magnetic hover** | ≤6px translate toward pointer, ≤3° tilt, spring-damped, released on exit, via `--mag-x/-y/--mag-tilt-x/-y` | Off; listeners not attached |
| `SocietyCard` in the directory | Magnetic hover | as above, ≤4px (larger surface, smaller pull) | Off |
| **Excluded from magnetic hover** | — | `MessageRow`, `LedgerTable` rows, `ChamberList` rows, member rows, palette results, `Table` cells, every dense list. `33 §4.3`: *magnetism in a dense list is nausea.* | — |
| `WalletCard` (Sealed panel) | **Field glow** | radial, ~180px, 8% `--fn-c-signal`, tracked by `--glow-x/--glow-y` | Off |
| `CharterView` panel (Sealed) | Field glow | as above, 6% | Off |
| `TransferSheet` body | Field glow | as above, 6% | Off |
| Registration / onboarding form shell | Field glow | as above, 10% — the most marketing-adjacent surface in the product | Off |
| `EmptyState` panel | Field glow | as above, 6% | Off |
| **Excluded from field glow** | — | The message list, the Stage scroll container, any region taller than two viewports (`will-change` on a long scroller costs more than the effect returns) | — |
| Mono field labels (`01 / SOCIETY NAME`) | **Letter-spacing breath** | +0.02em over 280ms with a text-shadow bloom, `--fn-ease-inout` | No change |
| Section indices, breadcrumb segments, status-bar segments, nav group headers, links | Letter-spacing breath | as above | No change |
| **Excluded from breath** | — | Any *value*: Fraction amounts, hashes, counts, timestamps, table cells. Data must not move; `33 §3.4` mandates tabular figures for exactly this reason | — |
| Composer commit (⌘Enter) | **Field ripple** | `scale(.15)→scale(3.2)`, opacity `.85→0`, 650ms `--fn-ease-out`, origin at the send affordance | 90ms opacity flash |
| `TransferSheet` confirm | Field ripple | as above, origin at pointer/focus | 90ms flash |
| Found-a-Society commit | Field ripple | as above | 90ms flash |
| Join-Society commit, Charter enact, Handle claim | Field ripple | as above | 90ms flash |
| **Excluded from ripple** | — | Navigation, filtering, tab switch, toggle, reaction, mark-as-read, copy, cancel. Ripple is scarce or it means nothing | — |
| `XpMeter` level-up | Segment fill | 650ms `--fn-dur-reveal` segment fill + one status-bar line. **Never a modal, never confetti** (`32 §5.5`) | Instant fill + status line |
| Queued/pending Message | Opacity | 70% + pending mark until settled (`32 §6`) | Identical — this is not motion |

Two implementation rules. First, magnetic hover and field glow attach pointer listeners; under `prefers-reduced-motion` the listeners are **not attached at all**, rather than attached and neutralized in CSS — the CPU cost is the point. Second, every one of these effects animates only `transform`, `opacity`, and `filter` (`32 §8`); an effect that triggers layout fails review regardless of how it looks.

---

## 8. Onboarding

The PH1 acceptance criterion is binary: **a new Citizen completes registration → Society creation → first Message in under 3 minutes, unassisted, on desktop and mobile web** (`50` PH1 AC 1). It is measured by E2E journey J1 (§13) on a throttled mid-tier profile, and it fails the phase if it regresses.

### 8.1 The flow, with a time budget

| # | T+ | Surface | What happens | Copy |
|---|---|---|---|---|
| 1 | 0:00 | `/` | Boot resolves no session in ≤400ms and routes to `/enter`. The sigil and one orbit render; **no text before 400ms**, because text that appears and is replaced reads as a failure | — |
| 2 | 0:05 | `/enter` | Sign-in shows a single primary action and a secondary "Create a Citizen" | "The system is listening." / "SIGN IN" / "CREATE A CITIZEN" |
| 3 | 0:10 | `/enter/new` | Handle field autofocused; availability checked on a 400ms debounce against `POST /v1/handles` dry-run | "01 / HANDLE" · "3–24 characters. Lowercase, digits, and _." · "Yours permanently after 14 days." |
| 4 | 0:35 | passkey ceremony | `navigator.credentials.create()` fires on the click, inside the gesture. Browser UI owns 5–20s | "02 / KEY" · "No password is created. There is nothing to steal and nothing to reset." |
| 5 | 0:55 | `/welcome/handle` | Display name, prefilled from the Handle, editable. One field. Skippable with Enter | "Name yourself. Your Handle is fixed; this is not." |
| 6 | 1:10 | `/welcome/society` | Society name, accent (12-stop wheel), visibility. Accent is preselected deterministically from the pending FNID so the field is never empty | "Name your Society. You can change it once, within 14 days." |
| 7 | 1:40 | commit | Field ripple, then a three-line progress readout in the classification register. The Society, Treasury, Vault, Charter v1, and `#general` are created by one command | `SOCIETY SEALED` / `TREASURY OPENED` / `#GENERAL LIVE` |
| 8 | 1:55 | `/s/…/c/general` | The shell renders with the first-message empty state (§5.4). **The composer is autofocused.** On handheld the keyboard is summoned by the same focus call | "NOTHING / has been said" · "#general is where this Society talks. Post the first Message and the Chamber is live." |
| 9 | 2:20 | first Message | ⌘Enter or the send affordance. Ripple, optimistic row, settle | "say something" |
| 10 | 2:25 | — | Status bar prints `FN://ORACLE-HALL/GENERAL · SYNCED 0s AGO`. XP meter fills one segment. No modal, no celebration | `XP +40 · FIRST WORD` in the status bar |

Twenty-five seconds of slack against the three-minute budget, and the largest variable — the browser's passkey ceremony — is the one we do not control, which is why steps 5 and 6 are one field and three fields respectively and nothing else.

### 8.2 What is deliberately deferred out of onboarding

Recovery guardian configuration (prompted once at 24h or at first sign-out, never blocking), a second device, theme choice, Profile detail, invites, notification preferences, and the Charter walkthrough. Each of these is a real thing a Citizen should eventually do and none of them belongs between registration and the first Message. `18 §5.1` sets a Level 0 Citizen's ceiling low precisely so that the first ninety seconds can be short.

### 8.3 Failure paths

| Failure | Detection | Response |
|---|---|---|
| No WebAuthn support | Feature detection before render | The passkey block renders as `USE ANOTHER DEVICE` and routes to `/enter/device`; never a dead button |
| Passkey ceremony aborted | `NotAllowedError` | Stay on the step; "Key ceremony cancelled. Try again, or use another device." No retry counter, no lockout |
| Passkey ceremony unsupported mid-flow | `NotSupportedError` | Same as above, plus a one-line reason |
| Handle taken | `conflict` from (3) | Inline: "@kaya is taken. Try @kaya_ or another Handle." Three suggestions rendered as chips |
| Handle confusable-normalized to an existing one | `validation_failed`, `cause.fields[]` | Inline with the colliding Handle named |
| Network drop after passkey, before session | Session probe on reconnect | Resume at the step; the credential already exists, so a second `create()` would fail — the client checks `GET /v1/citizens/me` first |
| Society creation denied by a Level gate | `unlock_required` | Render the gate verbatim (see §16 Q1 — this must not happen in PH1) |
| Society creation times out | No response within 10s | The `Idempotency-Key` is retained; retry replays rather than duplicating (`30 §5.6`). Copy: "Still working. This is a retry of the same request, not a second Society." |
| Registration abandoned mid-flow | Step recorded in `sessionStorage` | Returning within the session resumes at the step; the Handle reservation is not held and is re-checked |
| Passkey completion rate < 85% | Instrumented from day one (`50` PH1 risk row) | The risk has fired. `12 §11` names the response: hardware-key-only or recovery-share-first onboarding. **Not passwords** — that trade is not available |

---

## 9. Copy Specification

House voice from `33 §7`: terse, declarative, never cute; negation then assertion; second person as operator; errors state cause then remedy and never apologize. `"Oops! Something went wrong"` is a lint failure, as are "sorry", "unfortunately", "please try again later", and every exclamation mark.

| # | Surface | Trigger | String |
|---|---|---|---|
| 1 | Sign-in | header | The system is listening. |
| 2 | Registration | handle helper | 3–24 characters. Lowercase, digits, and underscore. |
| 3 | Registration | handle permanence | Yours permanently after 14 days. |
| 4 | Registration | passkey block | No password is created. There is nothing to steal and nothing to reset. |
| 5 | Registration | handle conflict | @kaya is taken. Try one of these, or another Handle. |
| 6 | Registration | ceremony cancelled | Key ceremony cancelled. Try again, or continue on another device. |
| 7 | Onboarding | Society name helper | Changeable once, within 14 days. |
| 8 | Onboarding | visibility helper (private) | Private. Only invited Citizens can read it. |
| 9 | Onboarding | visibility helper (discoverable) | Discoverable. Anyone can find it; the Charter decides who can join. |
| 10 | Onboarding | what gets created | You will be its Founder. A Treasury, a Vault, a Charter, and #general are created with it. |
| 11 | Onboarding | commit readout | SOCIETY SEALED · TREASURY OPENED · #GENERAL LIVE |
| 12 | Chamber | empty state display | NOTHING / has been said |
| 13 | Chamber | empty state body | #general is where this Society talks. Post the first Message and the Chamber is live. |
| 14 | Composer | placeholder | message #general |
| 15 | Composer | offline | Offline. This Message is queued and will post when the connection returns. |
| 16 | Composer | post refused, rate | Rate limit reached — 10 Messages per hour at Level 0. The window resets at 10:41. |
| 17 | Member list | empty filter | No Citizen matches "rel". |
| 18 | Wallet | zero balance | Nothing has moved yet. Fraction arrives from contribution, transfers, and your Society's Treasury. |
| 19 | Wallet | staleness | AS OF 4m AGO |
| 20 | Wallet | pending explanation | Pending Fraction is committed and not yet settled. It cannot be spent. |
| 21 | Transfer | irreversibility | Transfers are final. There is no reversal. |
| 22 | Transfer | insufficient funds | Transfer refused — 40 FRC requested, 12 FRC available. Send 12 FRC, or wait for 40 FRC pending to settle. |
| 23 | Transfer | capability limit | Transfer refused — Level 2 permits 100 FRC/day; 140 FRC requested. Send 60 FRC, or reach Level 4. |
| 24 | Transfer | elevated context needed | This amount needs a fresh key assertion. Confirm with your passkey. |
| 25 | Transfer | committed, unsettled | PENDING. Committed at 09:44. Settlement is usually under two seconds. |
| 26 | Transfer | self-transfer | A Wallet cannot transfer to itself. Choose another recipient. |
| 27 | History | empty | No Postings. Every Fraction that moves is recorded here. |
| 28 | Receipt | header | Why this moved |
| 29 | Receipt | formula line | Contribution 84 units × 0.5 FRC, capped at 12 FRC for the window ending 03 SEP 00:00. |
| 30 | Charter | Founder-only notice | Only the Founder can enact a Charter version in this phase. |
| 31 | Charter | enacted | Enacted. Charter v7 supersedes v6 at block 44812. |
| 32 | Join | Charter acceptance | Read the Charter. Joining accepts it. |
| 33 | Join | spent invite | This invite is spent. Ask for another. |
| 34 | Search | empty | No matches for "anchor" in Oracle Hall. Search covers Messages and Citizens in Societies you have joined. |
| 35 | Palette | empty | No command matches. |
| 36 | Palette | gated command | Requires Level 3. You are Level 0. |
| 37 | Signals | empty | Nothing has happened yet. Signals from your Societies arrive here. |
| 38 | Settings | one device | One device is enrolled. Enroll a second, or configure recovery — a single device is a single point of loss. |
| 39 | Settings | recovery unset | Recovery is not configured. Without it, losing every device is final. |
| 40 | Status bar | synced | SYNCED 2s AGO |
| 41 | Status bar | stale | STALE 4m |
| 42 | Status bar | offline with queue | OFFLINE · 1 QUEUED |
| 43 | Status bar | shed | DEGRADED · SIGNALS COALESCED |
| 44 | Offline page | body | Offline. The last state loaded is shown where it is available. Writes are refused except Messages, which queue. |
| 45 | Error page | `internal_fault` | The Runtime failed. Request 01HYQ3MB1F8T0J4W5Z2Q7R9N3D. This write was not applied. |
| 46 | Error page | version skew | This browser is running an older build than the Runtime. Reload to continue. |
| 47 | Not found | body | No such object, or you cannot see it. Both look the same from here, deliberately. |
| 48 | Session end | signed out | Signed out. Your keys stay on this device. |
| 49 | Progression | Level 0 | Level 0. Read, join up to three Societies, post, react, and receive Fraction. |
| 50 | Progression | Trust caption | Trust is not XP. XP says how much you did. Trust says whether you can be relied on. |

Strings 40–43 are the status-bar vocabulary and must be drawn from `33 §7.2`'s closed list plus the numeric interpolations shown; a new status word requires an amendment to `33` in the same Work Unit (R9).

---

## 10. Frontend Architecture

### 10.1 Structure

`apps/web/` exactly as `41 §10.1` specifies — `routes/` for routing only, `features/` as the unit of work, `core/` for the transport boundary, `shell/` for chrome, `test/` for generated MSW handlers. The boundary rules there have teeth here: `features/` may not import from `routes/`; a feature imports another only through its `index.ts`; no component calls `fetch`; no feature owns a colour, a size, or a font. All four are `eslint-plugin-boundaries` rules, not conventions.

PH1's features are `identity`, `society`, `chamber`, `wallet`, `progression`, `charter`, `search`, `signals`, and `settings` — nine, mapping onto the `10 §3` boundaries they consume. `core/` in PH1 holds the transport layer (generated client, DPoP, idempotency, problem-document decoding) and the Signal socket, **not** a wasm wrapper (R3); the directory keeps its name so PH2 lands the client core without a rename.

### 10.2 Routing

TanStack Router, chosen over React Router for typed path and search parameters and for loader-level integration with the query cache; the alternative — React Router with hand-written param types — was rejected because untyped params are the most common source of a route that renders the wrong Society. Routes are code-split at the boundaries in §3.1. The router owns scroll restoration per route key, and the Chamber route additionally restores *message anchor* rather than pixel offset, because a pixel offset in a virtualized, prepend-growing list is meaningless.

### 10.3 State

Three kinds of state, three mechanisms, no overlap.

| Kind | Mechanism | Examples |
|---|---|---|
| **Server state** | TanStack Query, keys generated from the API client's operation ids | Societies, Chambers, Messages, Wallet, Postings, Charter, progression |
| **Client state** | Zustand, one store per shell concern, no global god-store | Palette open, Context panel visibility, density, theme, reply-to target, composer drafts |
| **Durable local cache** | TanStack Query persister over IndexedDB (`idb-keyval`), an explicit allowlist of query keys | The six R4 surfaces, plus Signal resume cursors and the message outbox |

**Why not a local-first store in PH1.** A local replica with an event log and a sync engine is M2.1 (`50` PH2). Shipping a partial one now means shipping the hardest correctness problem in the product (`50` PH2 risk: *the classic local-first swamp*) without its deterministic simulation harness. So PH1 ships the honest subset: a persisted read cache that satisfies P2's read path with visible staleness, and a single-command outbox that satisfies P2's write path for the one command a Citizen will actually attempt offline (R5). The persister allowlist is explicit — persisting everything would cache Postings and progression that must never be shown as current without a re-read.

**Cache policy.** `staleTime` 30s for Messages and presence, 5s for Wallet, 5m for Charter and Society metadata, `Infinity` for the capability manifest. `gcTime` 24h for allowlisted keys, 5m otherwise. Every cached read carries the `Date` of its response, and the status bar computes staleness from the **oldest** cached read backing the current Stage — not from the newest, which would flatter.

### 10.4 The generated client

`@fractal/api-client` is generated from `schemas/openapi` and committed (`41 §10.4`). It is never hand-edited; a hand edit fails `lint-generated`. `core/transport/` wraps it with exactly five cross-cutting behaviours, in this order:

```ts
// core/transport/client.ts — the only place a request is constructed.
const request = pipe(
  withCorrelationId,      // Fn-Correlation-Id: ULID per intent, reused across retries
  withDpop,               // Authorization: DPoP <token> + proof; refresh at T-60s
  withIdempotency,        // ULID minted at INTENT time, persisted with the mutation
  withConditional,        // If-Match from the cached ETag on every PATCH
  withProblemDecoding,    // application/problem+json -> typed FnProblem, never a bare Error
);
```

`withIdempotency` is the subtle one. The key is minted when the Citizen *intends* the action — when the `TransferSheet` opens, not when the request fires — persisted alongside the pending mutation, and reused on every retry including one after a page reload. This is what makes `30 §5.6`'s replay guarantee reachable from a browser that was closed mid-transfer.

### 10.5 Signals

One WebSocket per tab, owned by `shell/SignalProvider`. It authenticates in the `Hello` frame (never a token in the URL), subscribes to `Self` plus the current Society plus the open Thread, and holds at most 12 scopes against the protocol's cap of 200.

```ts
// shell/signals/apply.ts
switch (signal.kind) {
  case "discourse.message.posted.v1":
    // Cheap and ordered: append into the cached page directly.
    queryClient.setQueryData(messagesKey(signal.body.thread_id), appendOrdered(signal));
    break;
  case "ledger.transfer.settled.v1":
  case "ledger.posting.recorded.v1":
    // Money is never patched from a Signal. The Signal is a hint to re-read.
    queryClient.invalidateQueries({ queryKey: walletKey(signal.body.wallet_id) });
    break;
  default:
    queryClient.invalidateQueries({ queryKey: societyKey(signal.society) });
}
```

The distinction is the whole policy: **Signals patch the cache only for append-only, server-ordered data. Everything else invalidates and re-reads.** A `Gap` frame (`30 §9.3`) triggers `GET /v1/societies/{sid}/events?since={seq}` paged to completion, then resumes live; the status bar shows `STANDBY` while catching up. `Shed` renders `DEGRADED`. `Bye{Revoked}` clears the session and routes to `/enter` immediately — revocation is retroactive and the client does not argue with it. Resume cursors are the highest **contiguous** applied `seq` per Society, persisted in IndexedDB, because contiguity is what makes a cursor safe.

Reconnect uses full-jitter backoff `U(0, min(30s, 0.5s·2^attempt))` exactly as specified, with a visible `STANDBY` state — never a silent retry loop, which produces a Citizen who believes the application is live when it is not.

### 10.6 Optimistic update policy

Derived from `10 §6`, restated as a front-end table because an agent implementing a mutation needs the answer at the call site.

| Mutation | Optimistic? | Rollback | Note |
|---|---|---|---|
| Post Message | **Yes** | Row stays with a retry affordance; never silently vanishes | Server-assigned `seq` reconciles order |
| Revise Message | Yes | Restore prior body | LWW on `(edited_at, device_id)` |
| React | Yes | Remove | CRDT OR-Set; a lost reaction is unacceptable |
| Read mark | Yes | none | Never blocks |
| Persona / settings field | Yes, per field | Restore | LWW per field |
| Join Society | **No** | — | The Charter decides; a client cannot predict it |
| Create Society | **No** | — | Handle uniqueness and Level gate are server facts |
| Create Chamber | **No** | — | Role-gated |
| Assign role | **No** | — | Governance |
| Enact Charter | **No** | — | Governance, signed |
| **Any Wallet write** | **Never. Not once. Not for the memo field.** | — | `10 §6`; the balance is never computed client-side |

The wallet row is enforced, not trusted: a lint rule forbids `setQueryData` on any key matching `wallet|posting|transfer` outside the invalidation path, and a unit test asserts that no reducer in `features/wallet` performs addition on a `Quanta`.

### 10.7 Error boundaries

Three nested levels, each with a different recovery.

1. **Route boundary** — one per route. A thrown render error shows the terminal error surface with the `Fn-Request-Id` and a reload action. Reports to telemetry with the correlation id.
2. **Stage boundary** — wraps the Stage only. The shell, rail, chamber list, and status bar survive; the Citizen can navigate away from a broken surface without losing the application.
3. **Panel boundary** — wraps each Context-panel section and each independently-fetched panel. A failed member list does not take down the Chamber.

Problem documents are not exceptions. They are typed values rendered by `code` from the closed registry (`30 §6.4`): `capability_denied` and `unlock_required` render the gate inline with the `remedy`; `precondition_failed` renders a reload affordance; `rate_limited` renders the reset time from `Retry-After`; `internal_fault` renders `cause.write_state` verbatim because a Citizen's next move depends on knowing whether the write landed. An unmapped code renders `detail` as-is — safe, because the API guarantees house voice — plus the request id.

### 10.8 Code splitting

Entry chunk = boot, entry screens, the shell, and the Chamber surface. Everything else is a route chunk per §3.1. The design system is imported by named export only; a barrel import that pulls all forty-one components into the entry chunk is a bundle-budget failure and CI names the importing file. Route chunks prefetch on intent — hover or focus of the link, or the palette result being highlighted — never on idle, which on a metered connection is a cost the Citizen did not ask for.

---

## 11. Performance Plan

Budgets are `32 §8`: cold start to interactive ≤2.5s p75, interaction to paint ≤100ms p95, initial JS ≤180KB gzip, route chunk ≤60KB gzip, fonts ≤90KB, memory ≤400MB.

### 11.1 Critical CSS

`tokens.css` (custom properties only, ~6KB) plus the shell's grid and the boot sigil are inlined in `index.html` — total inline budget 9KB gzip, CI-enforced. Everything else is route-level CSS loaded with its chunk. There is no CSS-in-JS runtime; components use CSS Modules over generated custom properties, so the token pipeline (N7) remains the only place a value is decided and no style computation happens at render.

### 11.2 Fonts

Two families, self-hosted, subset (`33 §3.1`). Manrope variable (300–700, latin + latin-ext) and DM Mono (300/400/500, latin). Two files, ~78KB total against the 90KB budget.

```css
@font-face {
  font-family: "Manrope";
  src: url("/f/manrope-v1.woff2") format("woff2-variations");
  font-weight: 300 700;
  font-display: swap;
  unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+2000-206F, U+2212;
}
@font-face {                        /* metric-matched fallback: kills CLS on swap */
  font-family: "Manrope Fallback";
  src: local("Segoe UI"), local("Helvetica Neue"), local("Arial");
  size-adjust: 103.5%; ascent-override: 98%; descent-override: 26%; line-gap-override: 0%;
}
```

Both faces are `<link rel="preload">`ed because both appear above the fold on every surface — the topbar is Manrope and the status bar is DM Mono. `size-adjust` on the fallback is what makes `swap` safe: without it, the swap reflows the status bar and the tabular figures in the topbar balance, which is a visible flinch on every cold load.

### 11.3 Virtualization

The message list virtualizes above 200 rows with a measured-height cache and an estimator seeded from the last 50 rows. Requirements: prepend must preserve the scroll anchor (history paging); the live tail must stay pinned when the Citizen is at the bottom and must not steal scroll when they are not; a jump to a deep-linked Message resolves by paging to the cursor, not by rendering the intervening history. Off-screen day-groups carry `content-visibility: auto` with a `contain-intrinsic-size` matched to the measured group height.

### 11.4 Images

There are none (R7). Sigils and avatars are inline SVG generated deterministically from the FNID — 3–7 nodes on a diamond lattice (`33 §6`) — costing zero network and rendering identically offline. When the Vault lands in PH2 the image policy becomes AVIF with a blurhash placeholder; PH1's policy is that the budget line reads zero.

### 11.5 Chunk budgets and CI gates

| Chunk | Budget (gzip) | Contents |
|---|---|---|
| entry | 180KB | React, router, query, transport, shell, Chamber, W0–W3 components |
| `wallet` | 45KB | Wallet, Transfer, history, receipt, `AmountField`, `LedgerTable` |
| `society` | 40KB | Members, settings, Charter, directory, create, join |
| `profile` | 25KB | Profile, progression, `XpMeter` |
| `settings` · `search` · `signals` · `welcome` · `enter` | 25KB each | as named |

CI gates, all blocking: per-chunk size; Lighthouse CI TTI p75 on a throttled mid-tier profile against staging; a synthetic INP trace over the six highest-frequency interactions (send Message, switch Chamber, open palette, open Transfer, page history, switch Society); frame-time regression on the five animated surfaces; font payload total; and `axe-core` (§12). A budget red for more than one Work Unit fires the `50` PH1 risk row and stops feature work until it is green.

---

## 12. Accessibility Plan

N8 and `32 §7`: AA floor, **AAA for body text in the default theme**, full keyboard operability, labels on every interactive element, motion never the sole carrier of meaning.

### 12.1 Work items

1. Landmark structure on the shell: `banner` (topbar), `navigation` ×2 (Society Rail, Chamber list, distinctly labelled), `main` (Stage), `complementary` (Context), `contentinfo` (status bar). `F6`/`⇧F6` cycles them, and the keymap says so.
2. Roving `tabindex` in the Society Rail, Chamber list, member list, palette results, and message list — one tab stop per region, arrow keys within. A list of forty Chambers must never be forty tab stops.
3. Focus visibility: 2px outline at 2px offset in `--fn-accent-data`, never removed, always additive to the glow (`32 §5.2` — the glow alone is not a focus indicator).
4. Modal discipline: `aria-modal`, focus trap, Esc closes, focus restores to the invoker. `TransferSheet` additionally moves focus to the amount field and announces the irreversibility notice as part of the dialog's accessible description, not as a separate visual aside.
5. `MessageRow` announces principal class in text: an Agent-authored row's accessible name begins "Agent" and includes the Envelope reference. Colour and glyph are the visual cues; the text is the guarantee.
6. `StatusPill` and `SignalDot` always carry a word (R9). A pulsing dot with no label is a violation.
7. Tabular data (`LedgerTable`, member list) uses real table semantics with a caption, scoped headers, and sortable columns exposing `aria-sort`.
8. All copy passes at 200% zoom with no fixed-height text container; the shell's grid switches to the handheld arrangement under zoom exactly as it does under width, because a zoomed 1440px viewport *is* a narrow viewport.
9. Contrast: the token build already asserts every pair; §12.4 adds the per-surface audit.
10. The `contrast` theme ships in PH1 with all glows removed and every pair ≥7:1 (`32 §9`).

### 12.2 Live regions

| Region | Politeness | Content | Throttle |
|---|---|---|---|
| Message log | `polite` | "New Message from @kaya" — author and the fact, not the body | One announcement per 2s; beyond that, "3 new Messages" |
| Balance | `polite` | "Balance settled: 1,204 FRC" — only on settlement, never on pending flicker | One per settlement |
| Sync state | `polite` | Only transitions into `OFFLINE`, `DEGRADED`, and back to `SYNCED` | Debounced 3s |
| Command palette results | `polite` | Result count on query change | Debounced 300ms |
| Form errors | `assertive` | The `detail` string | Immediate |

Everything is `polite` except a form error the Citizen just caused. `assertive` on incoming Messages would make an active Chamber unusable with a screen reader — the exact failure `32 §7` names.

### 12.3 Reduced motion

Implemented at the token layer plus explicit component opt-outs:

```css
@media (prefers-reduced-motion: reduce) {
  :root {
    --fn-dur-instant: 1ms; --fn-dur-fast: 1ms; --fn-dur-base: 1ms;
    --fn-dur-slow: 1ms;    --fn-dur-reveal: 1ms; --fn-dur-ambient: 0s;
  }
  .fn-pulse { animation: none; opacity: 1; }        /* static filled dot */
  .fn-orbit { animation: none; }                    /* static rings */
  .fn-grain { display: none; }
  .fn-ripple { animation: fn-flash 90ms linear; }   /* opacity flash */
}
```

Plus the JS rule from §7: magnetic hover and field glow do not attach listeners at all. A `useReducedMotion()` hook reads the media query and is the single source for that decision; a component that checks the media query itself is a review rejection.

### 12.4 Screen-reader test script

Run per phase gate on NVDA/Chrome (Windows) and VoiceOver/Safari (macOS, iOS), by a human, recorded.

1. Load `/` with no session. Confirm the boot state is announced once, not on every frame.
2. Complete registration using only the keyboard and the reader. Confirm the passkey ceremony is reachable and its outcome announced.
3. Complete onboarding to the first Message. Confirm the composer receives focus and is announced with its Chamber name.
4. Post a Message. Confirm the optimistic row is announced once and not re-announced on settlement.
5. Receive a Message from a second session. Confirm one polite announcement, with author.
6. Cycle all six landmarks with F6. Confirm each is named distinctly.
7. Navigate the Chamber list by arrow keys. Confirm one tab stop and per-item announcement including unread state as a word.
8. Open the command palette. Confirm the result count is announced and a disabled command announces its gate.
9. Open the `TransferSheet`. Confirm focus lands on the amount, the irreversibility notice is in the dialog description, and Esc restores focus to the invoker.
10. Attempt a transfer exceeding the balance. Confirm the error is announced assertively with cause and remedy.
11. Disconnect the network. Confirm the transition to `OFFLINE` is announced once and the queued Message state is announced as pending, not as failed.
12. Zoom to 200%. Repeat steps 3–5. Confirm nothing is clipped and no function is lost.

---

## 13. Testing Plan

| Level | Tool | Scope | Gate |
|---|---|---|---|
| Unit | Vitest | Formatters (quanta ↔ FRC — property-tested to the quantum), cursor handling, Signal reducers, backoff, idempotency key lifecycle, route param parsing | Coverage floor on `core/` and `features/*/lib` |
| Component | Vitest browser mode + Testing Library | Every one of the forty-one components: all eight states, its keyboard contract as executable steps, its ARIA contract asserted against roles and names | A component without keyboard and ARIA tests is not "done" (`32 §5`) |
| Contract | MSW handlers generated from `schemas/openapi` (`41 §10.4`) | Every feature hook against generated fixtures, including every error `code` the surface maps | Regenerating handlers must produce a zero diff |
| Visual regression | Storybook + Playwright screenshots | component × theme(`void`, `contrast`) × density(comfortable, compact) × state, plus the eight §5 surfaces at `handheld` and `standard` | Blocking on every PR (`32 §11`) |
| A11y automation | `axe-core` | Every route, every dialog in its open state, every Storybook story | Zero violations (`50` PH1 AC 4) |
| E2E | Playwright | The four journeys below | Blocking; J1 is timed |
| Parity | The P13 harness | GUI action vs `fn` action produce identical event streams | Blocking at the release tag (`50` PH1 AC 2) |

The visual matrix is capped deliberately: two themes × two densities, not three × three. `daylight` is PH2 and the `dense` mode applies only to `LedgerTable` and the member list, which get it as two extra stories rather than a matrix dimension. An unbounded matrix is how a visual regression suite becomes a suite everyone disables.

**The four E2E journeys**, each justified by a cross-boundary contract no cheaper test can cover:

- **J1 — First run.** Registration → Society creation → first Message, timed, on a desktop and a mobile viewport, on a throttled mid-tier profile. It *is* `50` PH1 AC 1; it fails the phase, not the PR.
- **J2 — Two Citizens, one Chamber.** Two browser contexts. A posts, B receives via Signal within 500ms; B reacts, A sees it; the socket is severed and resumed, and a `Gap` is recovered from the events endpoint with no duplicate and no missing Message. Covers the Relay, the resume protocol, and the cache-patch policy at once.
- **J3 — Money.** Transfer with sufficient funds → `PENDING` → settled; transfer with insufficient funds → the §9 string; a retry with the same `Idempotency-Key` after a simulated network drop produces exactly one Posting. Covers P12's spine end to end, and it is the only test that can prove the client never did arithmetic.
- **J4 — Offline.** Network disabled: the six R4 surfaces render last-known-good with staleness; a Message queues; a Transfer is refused with its remedy; on reconnect the queued Message posts exactly once. This is P2's falsification test, automated.

---

## 14. Work Unit Backlog

Per `42 §2`: each Work Unit is coherent, complete, green, reviewable in ≤20 minutes, revertible, and attributable. Each becomes exactly one squashed commit. Dependencies are Work Unit ids; `—` means it depends only on its Milestone's entry state.

### 14.1 M1.7 — Design system v1

| ID | Goal | Depends | Acceptance |
|---|---|---|---|
| WU-1.7-001 | Consume the generated token CSS in `packages/design-system` and expose `ThemeProvider` for `void` and `contrast` | — | Both themes render; `xtask tokens` regeneration produces a zero diff |
| WU-1.7-002 | Storybook harness with the nine-artifact manifest check | 001 | CI fails a story missing any of the nine artifacts |
| WU-1.7-003 | Visual regression pipeline (theme × density matrix) | 002 | A one-pixel change in `Panel` fails the PR |
| WU-1.7-004 | A11y test harness: axe on every story, keyboard-contract helper | 002 | A component without a keyboard test fails |
| WU-1.7-005 | `Panel` (Tick/Framed/Sealed), `Hairline`, `Rail` | 001 | Corner ticks are diagonal-opposed at 14px; nine artifacts each |
| WU-1.7-006 | `Grain`, `FieldGlow`, `useReducedMotion` | 005 | One grain layer per viewport; glow listeners absent under reduced motion |
| WU-1.7-007 | `Sigil` incl. deterministic Society sigil generation from an FNID | 005 | Same FNID yields byte-identical SVG across runs and platforms |
| WU-1.7-008 | `Button`, `IconButton` with magnetic hover on primary only | 005 | Magnetism absent on secondary, ghost, danger, and under reduced motion |
| WU-1.7-009 | `TextField`, `TextArea` with the `32 §5.2` field anatomy | 005 | Focus shows outline **and** glow; label, helper, and error are associated |
| WU-1.7-010 | `Checkbox`, `Switch`, `SegmentedControl` | 008 | Full keyboard and ARIA contracts |
| WU-1.7-011 | `Skeleton`, `StatusPill` (closed vocabulary type), `SignalDot`, `Badge` | 005 | A status outside `33 §7.2` fails to typecheck |
| WU-1.7-012 | `Avatar` with the three principal shapes | 007 | Citizen circle, Society diamond, Agent violet rim; class in the accessible name |
| WU-1.7-013 | `Combobox`, `SearchField` | 009 | ARIA 1.2 combobox pattern; virtualized option list |
| WU-1.7-014 | `AmountField` with the shared quanta formatter | 009 | Property test: format∘parse is identity across the quantum range |
| WU-1.7-015 | `Tabs`, `Breadcrumb` (renders `FN://SOCIETY/CHAMBER`) | 008 | Breadcrumb output matches the CLI path form exactly |
| WU-1.7-016 | `ContextMenu`, `Sheet`, `Drawer` with focus trap and restore | 008 | Focus restores to the invoker in all three |
| WU-1.7-017 | `SignalToast` + its `32 §5.4` entry | 016 | `32` amended in the same commit |
| WU-1.7-018 | `EmptyState` with the outlined-word treatment and static orbit | 006 | Orbit static under reduced motion; measure ≤88ch |
| WU-1.7-019 | `MetricTile`, `Table` (dense, sticky header, sortable, virtualized) | 011 | `aria-sort` correct; 10,000 rows scroll within the frame budget |
| WU-1.7-020 | `SocietyRail` with unread dots and roving tabindex | 012 | One tab stop; `1–9` jump; no count badges |
| WU-1.7-021 | `ChamberList` grouped by kind with the active rail | 020 | Selection is `inset 2px 0 0`, never a fill |
| WU-1.7-022 | `CommandPalette` shell: open, filter, sections, disabled-with-reason | 013,016 | Esc closes without a history entry; disabled entries state their gate |
| WU-1.7-023 | `MessageRow` incl. the agent variant (four cues) | 012 | Agent variant covered by visual regression from fixtures |
| WU-1.7-024 | `Composer` with ⌘Enter, mention picker, draft persistence, ripple | 009,013 | Ripple only on commit; draft survives a reload |
| WU-1.7-025 | `XpMeter` (5 segments, level-up fill, no modal) | 011 | Level-up emits a status-bar line and no dialog |
| WU-1.7-026 | `WalletCard` (settled / pending / available, tabular, Sealed) | 014,019 | Three figures always distinct; `--fn-accent-data` only |
| WU-1.7-027 | `TransferSheet` (breakdown, irreversibility notice, ripple) | 014,016 | Notice is in the dialog description; commit is never optimistic |
| WU-1.7-028 | `LedgerTable` (both sides of every Posting, reason chip) | 019 | Debit and credit both visible in every row |
| WU-1.7-029 | `ContributionReceipt` (source, input, formula, amount, window) | 026 | Renders the `18` formula fields verbatim |
| WU-1.7-030 | `CharterView` diff-first (what changed, who signed, when enacted) | 019 | Diff mode is the default when a prior version exists |
| WU-1.7-031 | `SocietyCard` with lineage badge and magnetic hover | 012 | Lineage badge renders `Crystallized`/`Fractured`/`Forked` or nothing |
| WU-1.7-032 | Terminal-equivalent audit: every component's CLI mapping documented | 023–031 | `32 §10` table extended; a component with no equivalent named fails |

### 14.2 M1.8 — Web GUI

| ID | Goal | Depends | Acceptance |
|---|---|---|---|
| WU-1.8-001 | `apps/web` scaffold: Vite, TS strict, boundary lint, budget CI job | WU-1.7-001 | A `fetch` outside `core/transport` fails lint; the budget job runs on PR one |
| WU-1.8-002 | `core/transport`: generated client + correlation, DPoP, idempotency, conditional, problem decoding | 001 | Every one of the five behaviours has a unit test |
| WU-1.8-003 | MSW handlers generated from `schemas/openapi` | 002 | Regeneration produces a zero diff |
| WU-1.8-004 | Query client, key factory, IndexedDB persister with the R4 allowlist | 002 | A non-allowlisted key is never persisted (asserted) |
| WU-1.8-005 | Router skeleton with all §3.1 routes and route-level chunking | 001 | Chunk names match §11.5; every route resolves |
| WU-1.8-006 | Error boundaries at route, Stage, and panel levels | 005 | A thrown Stage error leaves the shell operable |
| WU-1.8-007 | Problem-document renderer mapping every `30 §6.4` code | 002,006 | Every code has a case; an unmapped code renders `detail` + request id |
| WU-1.8-008 | Boot route: session resolve, last-Society restore, ≤400ms text rule | 002,005 | No text paints before 400ms |
| WU-1.8-009 | Sign-in with passkey assertion | 008 | Keyboard-only completion; error inline |
| WU-1.8-010 | Registration: Handle claim with debounced availability + passkey create | 009 | Conflict renders three suggestions; ceremony abort is recoverable |
| WU-1.8-011 | Device-code fallback route | 010 | Reachable when WebAuthn is absent; code copyable |
| WU-1.8-012 | Onboarding steps 1–2 with back-preserving state | 010 | Back restores entered values; Esc disabled on step 1 |
| WU-1.8-013 | Found-a-Society command + the three-line commit readout | 012 | Idempotency key survives a reload mid-commit |
| WU-1.8-014 | The Society shell layout: five regions, F6 cycling, landmarks | WU-1.7-020,021 | Six landmarks named distinctly; regions do not remount on Stage change |
| WU-1.8-015 | Status bar: path, sync state, custodian count, anchor block | 014 | Staleness computed from the oldest cached read backing the Stage |
| WU-1.8-016 | `SignalProvider`: connect, Hello, Subscribe, Ack, heartbeat | 002,014 | ≤12 scopes; token never in the URL |
| WU-1.8-017 | Signal apply policy: patch appends, invalidate everything else | 016,004 | A lint forbids `setQueryData` on wallet keys |
| WU-1.8-018 | Resume, backoff, Gap recovery via the events endpoint | 016 | J2's sever-and-resume passes with no duplicate or gap |
| WU-1.8-019 | Chamber surface: threads, messages, paging, virtualization, anchoring | 014,WU-1.7-023 | Prepend preserves anchor; 10,000 messages hold the frame budget |
| WU-1.8-020 | Composer wiring: optimistic post, retry affordance, ripple | 019,WU-1.7-024 | A failed post never vanishes silently |
| WU-1.8-021 | Message actions: reply, revise (If-Match), redact, react (CRDT) | 019 | `precondition_failed` renders the reload remedy |
| WU-1.8-022 | Presence and typing | 016,019 | Coalesced under `Shed`; never announced to screen readers |
| WU-1.8-023 | Offline read cache surfacing + staleness on the six R4 surfaces | 004,015 | J4's read half passes |
| WU-1.8-024 | Message outbox: enqueue, 70% pending render, reconcile once | 020,023 | Exactly one Posted event after reconnect (asserted) |
| WU-1.8-025 | Chamber empty state as the onboarding payload | WU-1.7-018,019 | Composer autofocused; keyboard summoned on handheld |
| WU-1.8-026 | Member list with roles and Standing | 014,WU-1.7-019 | Filterable; Enter opens the Profile |
| WU-1.8-027 | Society settings with `If-Match` and discard confirmation | 026 | A concurrent change renders the reload remedy |
| WU-1.8-028 | Charter view + version diff + Founder enact | WU-1.7-030 | Diff default; enact ripples and prints the governance line |
| WU-1.8-029 | Society directory + join flow with Charter acceptance | WU-1.7-031 | Join is armed only after the Charter checkbox is checked |
| WU-1.8-030 | Wallet surface: settled / pending / available, staleness | WU-1.7-026 | No client-side arithmetic (asserted by test) |
| WU-1.8-031 | Transfer flow incl. elevated context above the threshold | WU-1.7-027,030 | Never optimistic; J3 passes |
| WU-1.8-032 | Posting history with cursor paging | WU-1.7-028 | Deep paging performs no offset query (contract test) |
| WU-1.8-033 | Posting detail and `ContributionReceipt` | 032,WU-1.7-029 | Formula fields render verbatim |
| WU-1.8-034 | Treasury surface reusing the Wallet feature, role-gated | 030 | `capability_denied` renders the role remedy |
| WU-1.8-035 | Profile (own and other) | 014 | `not_found` for unreadable Citizens, never an existence oracle |
| WU-1.8-036 | Progression surface: XP, Trust, Standing, Unlocks | WU-1.7-025,035 | XP and Trust never adjacent as a sum |
| WU-1.8-037 | Command palette wiring to the generated command set | WU-1.7-022 | Parity test: palette ids == CLI leaves minus the allowlist |
| WU-1.8-038 | Scoped search over Messages, Citizens, Chambers | WU-1.7-013 | Facets switchable by keyboard; empty state names the scope |
| WU-1.8-039 | Notification centre over the events endpoint | 018 | Mark-all-read; opens the subject |
| WU-1.8-040 | Settings: Account, Devices, Appearance, Privacy, Recovery config | 014 | Recovery prompt appears once at 24h, never blocking |
| WU-1.8-041 | Handheld layout: rail sheet, context drawer, 44px targets | 014 | J1 passes on a mobile viewport |
| WU-1.8-042 | Deep-link resolution incl. `web+fractal:` opt-in | 005 | Unauthenticated deep links resume after sign-in |
| WU-1.8-043 | Back-button semantics per §3.3 | 005,031 | A committed Transfer cannot be re-entered by Back |
| WU-1.8-044 | Offline and terminal error pages | 007,023 | Version skew renders the reload string |
| WU-1.8-045 | Critical CSS inlining + font loading with `size-adjust` | 001 | Inline ≤9KB; CLS ≤0.02 on cold load |
| WU-1.8-046 | Performance CI: chunk budgets, Lighthouse, INP, frame time, fonts | 045 | All five gates blocking |
| WU-1.8-047 | axe on every route and every open dialog | 006 | Zero violations (`50` PH1 AC 4) |
| WU-1.8-048 | E2E J1 (first run, timed, desktop + mobile) | 025,041 | Under 3 minutes on the throttled profile |
| WU-1.8-049 | E2E J2 (two Citizens), J3 (money), J4 (offline) | 024,031 | All three blocking |
| WU-1.8-050 | Keymap overlay (`?`) generated from the binding registry | 037 | Every binding in §4.4 is listed and correct |

Fifty Work Units for M1.8 and thirty-two for M1.7. Any Work Unit exceeding ~400 changed lines excluding generated files is two Work Units wearing a trenchcoat (`42 §2.2`) and must be split before the first commit, not after.

---

## 15. Definition of Done

`00 §5` has eight points. Seven of eight is a defect with good marketing. Mapped to this phase's GUI:

| # | `00 §5` criterion | What satisfies it here |
|---|---|---|
| 1 | Satisfies its written acceptance criteria verbatim | Every Work Unit's acceptance column in §14 is green, and `50` PH1 AC 1, 3, and 4 hold |
| 2 | Reachable through the API, the CLI, and a GUI | Every screen's primary action names its `fn` verb (§4); the palette parity test (WU-1.8-037) and the P13 event-stream parity suite are green |
| 3 | Tested at the level appropriate to its risk | §13's five levels; money and identity carry E2E coverage, chrome carries component coverage |
| 4 | Emits domain events and correlated telemetry | Every mutation sends `Fn-Correlation-Id`; the front end reports INP, route timings, and error codes tagged with it |
| 5 | Documented: API reference, CLI help, changelog | Each Work Unit's changelog line; each component's Storybook docs page; the keymap overlay is generated, so it cannot drift |
| 6 | Degrades correctly offline and denies correctly without permission | R4/R5 offline behaviour proven by J4; `capability_denied` and `unlock_required` render the gate on every surface, proven by contract tests |
| 7 | Respects performance and accessibility budgets | §11.5 and §12 gates blocking in CI; no budget carried red across a Work Unit |
| 8 | Its ADR exists if it changed a technology choice | ADRs required for: the state-management choice (§10.3), the no-wasm ruling (R3), and every resolution in §16 that is adopted |

---

## 16. Open Questions

Each has a proposed default. **Work is never blocked on one of these**: an agent adopts the default, cites the question id in the PR, and the human resolves it before the phase gate. A default adopted and not overturned becomes an ADR at the gate.

| # | Question | Proposed default |
|---|---|---|
| Q1 **RESOLVED — `61 X7`** | `50` PH1 AC 1 requires a new Citizen to create a Society; `18 §5.1` gates founding a Society at **Level 3**, and a new Citizen is Level 0. These cannot both hold. | **Ruled: the first-hearth exemption**, landed in `18 §5.1` and `11 §2.3`. The allowance is one-time per FNID, consumed at `SocietyCreated`, not restored on Dissolution or departure, and not consumed by a Crystallization. Original proposal: **First-Society exemption:** every Citizen may found exactly one Society at Level 0; the Level 3 gate applies from the second onward. This preserves J1's intent — the adversarial capability is *mass* Society creation, not the first one — and makes the phase's headline criterion reachable. Requires an ADR amending `18 §5.1`. |
| Q2 **SUPERSEDED — `61 X-GA`** | PH1 budgets **zero economic Sources**, yet the Wallet, Transfer, and `ContributionReceipt` surfaces need Fraction to exist. Where does the first FRC come from? | **Ruled: `PostingReason::GenesisAllocation`** — 100 FRC to each new Citizen's global Wallet (locked until Level 1) and 250 FRC to each new Society Treasury, aggregate hard cap 50,000,000 FRC, posted **from the `EmissionAccount`** so `11 §7.4` needs no exception, drawn against `B(1)`, published in the supply dashboard, retired at the PH4 exit gate. Larger, capped, Citizen-inclusive, and without the separate account this original default proposed: A one-time, bounded, published **genesis allocation** of 100 FRC to each new Society Treasury, recorded as a Posting from the `GenesisAccount` with a named `PostingReason`. It is an allocation, not a Source: no rate, no formula, no recurrence, and it counts against nothing because nothing emits. The `ContributionReceipt` surface ships against fixtures until PH4's first real Source. |
| Q3 | `34 §4.2` puts the wasm core and OPFS SQLite in the Phase-1 web target; `50` M2.1 puts the client core in PH2; `32 §8` caps initial JS at 180KB. | **R3 stands: no wasm in PH1.** `34 §4.2` is amended to describe the PH2 state and to note the PH1 subset. |
| Q4 | Does PH1 ship a PWA? `50` M2.9 says PH2; `34 §4.3` says Phase 3. | **No PWA, no Service Worker, no manifest in PH1** (R8). `34 §4.3` is corrected to PH2 to agree with `50`. |
| Q5 **RESOLVED — `61 X-TH`** | Theme names disagree: `32 §9` says `void` / `daylight` / `contrast`; `41 §10.2` says `default` / `high-contrast` / `terminal-amber`. | **Ruled: `32` wins on names, `41` wins on file layout** — `packages/tokens/src/theme/{void,daylight,contrast}.json`, with `terminal-amber` renamed `cli-amber` and moved out of `theme/` because it is a CLI palette variant and not a theme. **`32` wins.** `41 §10.2` is corrected in the same Work Unit as WU-1.7-001. PH1 ships `void` and `contrast`; `daylight` is PH2. |
| Q6 **RESOLVED — `61 X-GW`** | Grace windows are stated three ways: Handle "immutable after a grace window" (`01 §2`), field example "changeable once, within 14 days" (`32 §5.2`), Society name "you can change it once" (`33 §7.3`). | **14 days, once, for both Handle and Society name.** All three documents are aligned to that sentence, and the copy strings in §9 are the canonical wording. |
| Q7 | The notification centre exists in PH1, but Web Push belongs to the PWA in PH2. | **In-app Signal inbox only.** No permission prompt, no push registration, no badge. The inbox reads the events endpoint per joined Society. |
| Q8 | Search: `30 §4.2` places the Discovery family later, but PH1 needs search, and adding a fourth resource family would breach `02 §5`. | **In-Society scoped search only**, served by the Discourse projection over `GET /v1/search?society=…`, covering Messages, Citizens, and Chambers within joined Societies. Global discovery is PH5 and is when the Discovery family is created. |
| Q9 | `32 §4.3` auto-selects Compact density at `wide`+, but PH1 does not implement `wide`. | **Comfortable by default, with an explicit override in Settings persisted per device** (`⌘\` toggles). Auto-selection arrives with the `wide` layout in PH2. |
| Q10 | Should `MessageRow` implement the agent variant when no Agent can post until PH3? | **Yes** — implemented, visual-regression covered, fed by fixtures. The P4 visibility contract is cheaper to build once than to retrofit across every message surface, and `SystemNotice` messages already need a non-human presentation in PH1. |
| Q11 **RESOLVED — `61 X8`/`N10`** | `30 §4.2`'s per-family Phase column disagrees with `50` throughout (Identity/Society/Discourse marked Phase 0; Progression marked Phase 2 while `50` M1.6 delivers it in PH1). | **`50` governs sequencing** by its own charter. `30 §4.2`'s Phase column is corrected to reference `50`'s `PH<n>` notation rather than a second numbering, which is also what `42 §2.1` requires of phase notation. |
| Q12 | Does the Context panel persist per Chamber or per Society? | **Per Society, per device.** Per-Chamber persistence produces a panel that appears and disappears while moving between Chambers, which reads as a bug. |
