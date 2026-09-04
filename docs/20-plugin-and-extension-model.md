# 20 — Plugin and Extension Model

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `10-system-architecture.md` §11, `11-domain-model.md` §2.8, `15-agent-runtime.md` §4.
> **Governs:** Extension kinds, the Extension manifest, the WASM execution model and Host API, the hook catalog, the capability model for Extension Installs, UI contribution, extension data and quotas, versioning and deprecation, install-time consent, performance enforcement, the Experience Runtime (Phase 7), the extension threat model, and the extension developer experience.
> **Does not govern:** listings, pricing, revenue share, payouts, ratings, the security review pipeline, or dispute handling. Those are `19-marketplace.md`. This chapter is the runtime and the API surface; `19` is the storefront that sells against it.

---

## 1. Position — P7 Is a Build Constraint, Not a Marketing Claim

P7 states that first-party features are built through the same extension surfaces third parties get. Taken seriously, this inverts the usual order of construction: the extension API is not a wrapper we add over a finished product, it is the internal seam the product is assembled along, with a signature and a version number attached.

The falsification test for P7 asks, for each first-party feature shipped after Phase 3, which extension capability it uses. That test is only answerable if the answer is designed in advance. The following first-party features are specified now as Extensions, and are the proof-carrying load for the API:

| First-party Extension | Kind | Hooks / surfaces used | Capabilities requested |
|---|---|---|---|
| `fn.polls` — Chamber polls and Charter straw polls | `plugin` | `chamber.message.posting`, `ui.panel.chamber`, `ui.command`, `store.*` | `chamber.message.post<in:scope>`, `extension.store.write` |
| `fn.digest` — daily/weekly Society digest | `plugin` | `schedule.tick`, `search.querying`, `chamber.message.posting` | `chamber.read`, `search.query`, `chamber.message.post` |
| `fn.triage` — moderation triage queue | `plugin` | `chamber.message.posted`, `agent.tool.offering`, `ui.panel.society` | `chamber.read`, `moderation.flag<=500/day>` |
| `fn.lightbox` — Gallery Chamber media viewer | `plugin` | `media.preview.rendering`, `ui.panel.chamber` | `vault.object.read<path:/media/**>` |
| `fn.charter-templates` — starter Charters | `template` | — (declarative) | none |
| `fn.terminal-dash` — Terminal dashboards | `plugin` | `cli.command`, `ui.surface.cli` | `society.read`, `ledger.read<self>` |
| `fn.standing-explainer` — "why is my Standing this" | `plugin` | `ui.profile.module`, `member.standing.explaining` | `progression.read<where:subject=self>` |
| `fn.onboarding` — new-member workflow | `automation-pack` | `member.joined`, `schedule.tick` | `chamber.message.post<in:welcome>` |
| `fn.high-contrast` / `fn.terminal-amber` | `theme` | — (token overrides) | none |
| `fn.custodian-ops` — Custodian health routines | `automation-pack` | `schedule.tick`, `vault.attestation.completed` | `node.report`, `vault.attest` |

**The Hook Debt Rule.** When a first-party feature needs a hook the Host API does not expose, the feature stops. It does not reach past the API. The correct sequence is: design the hook, add it to the WIT world, version the Host API, document it in the public reference, and ship the hook *in the same release* as the feature that motivated it. If — and this is the only exception — the hook cannot be made safe for arbitrary third parties (it hands out authority the consent screen cannot honestly summarize, or it sits on a path where a slow guest would violate P10 unconditionally), it may ship marked `access = "reserved"` in the WIT. Reserved hooks are then subject to three constraints:

1. They appear in a public **Reserved Hook Register** with the reason and a target phase for opening.
2. Reserved hooks may never exceed **5% of the total hook surface**, measured at every phase gate.
3. A reserved hook that has not opened by its target phase is either opened or deleted along with the feature that depended on it.

This is the honest version of the principle. A private back door with a public name, a stated reason, and an expiry date is survivable. An unnamed one metastasizes, and within two phases the extension API is a museum piece nobody at the company uses.

**Invariant X1.** Every first-party feature shipped after Phase 3 resolves to an Extension id and a manifest, or to a Runtime boundary in `10 §3`. There is no third category.

**Invariant X2.** The Host API has exactly one implementation. First-party Extensions are loaded by the same host, through the same instantiation path, under the same fuel and memory metering as third-party ones. The only thing an Extension's first-party status buys is a shorter review path in `19` and eligibility for `reserved` hooks.

---

## 2. Extension Kinds

`Extension` is the distributable unit; `Extension Install` is that unit activated inside one Society, holding its own Envelope and configuration (`01 §5`). Seven kinds, closed enum, adding a variant is an ADR.

| Kind | Ships | Executes code | Holds an Envelope | Society Level to install | Phase |
|---|---|---|---|---|---|
| `plugin` | WASM component + UI descriptors + assets | Yes (guest) | Yes | 3 | 3 |
| `theme` | Design-token overrides, no logic | No | No | 1 | 3 |
| `template` | Charter, Chamber layout, Vault layout blueprints | No | No (applied once, by a Citizen) | 1 | 3 |
| `workflow` | One Workflow graph (`15 §8`) | Executed by a Workflow Agent, not by the plugin host | Yes — union of step needs ∩ Envelope | 3 | 4 |
| `automation-pack` | Multiple Workflows + schedules + Policy *proposals* | As above | Yes | 3 | 4 |
| `sdk` | WIT worlds, language bindings, type packages | No — never installed into a Society | No | n/a | 3 |
| `experience` | WASM component targeting the Experience world + assets | Yes (guest, stronger sandbox) | Yes, restricted (§12) | 4 | 7 |

Notes that matter:

- **`theme` cannot execute.** A theme is a set of overrides against the token vocabulary in `32-design-system.md`, validated at publish time against contrast and motion budgets (N8, P10). A theme that fails WCAG 2.2 AA on any token pair is rejected at publish, not at install. This is why themes need no Envelope and no review latency.
- **`template` is applied, not installed.** Instantiating a template emits ordinary domain events (`SocietyCreated`, `ChamberCreated`, `CharterEnacted`) signed by the applying Citizen. The template has no continued existence and no authority. Templates therefore cannot "phone home" or change behaviour after the fact — a property that makes them safe at Society Level 1.
- **`automation-pack` proposes Policy, never authors it.** P4 forbids anything but a Citizen authoring Policy. A pack ships Policy *drafts* that render in the consent screen as diffs a human signs, one at a time.
- **`sdk` is a registry-only kind.** It exists so that the marketplace can distribute versioned WIT worlds and bindings under the same identity, signing, and provenance rules as executable Extensions. An `sdk` has no Install.

---

## 3. The Manifest

One file, `extension.toml`, at the root of the bundle. TOML rather than JSON because humans write it and it takes comments; the canonical wire form is the JSON projection the registry stores, and the two are checked against one schema.

### 3.1 Worked example

```toml
# Council Polls — a first-party plugin, used here as the reference manifest.
schema = 1

[extension]
id            = "net.fractal.polls"          # reverse-DNS, globally unique, immutable
name          = "Council Polls"
version       = "2.3.1"                       # semver, immutable once published
kind          = "plugin"                      # exactly one kind per Extension
publisher     = "fn1qzs…k4r7"                 # publisher FNID (01 §2)
license       = "Apache-2.0"
summary       = "Structured polls in Chambers, with Charter-aware quorum display."
homepage      = "https://ext.fractal.net/polls"
source        = "https://github.com/fractalnode/ext-polls"   # required for review (19)

[compat]
host_api      = ">=1.4, <2.0"                 # Host API semver range (§9)
runtime       = ">=0.9.0"                     # minimum Runtime version
society_level = 3                             # minimum Society Level (11 §2.3)
surfaces      = ["gui", "cli", "mobile"]      # surfaces this Extension claims to support

[[capability]]
name   = "chamber.message.post"
scope  = "in:${install.chambers}"             # bound at install to selected Chambers
reason = "Posts the poll card and the result card into the Chamber it was opened in."

[[capability]]
name   = "chamber.read"
scope  = "in:${install.chambers}"
reason = "Reads the message the poll is attached to, for quorum context."

[[capability]]
name   = "extension.store.write"
limit  = "<=8MB"
reason = "Stores poll definitions, ballots, and tallies."

[[capability]]
name   = "governance.charter.read"
reason = "Renders the Society's configured quorum and threshold on the poll card."

[limits]                                       # requested; host clamps to policy ceilings
memory_mb          = 24
fuel_per_call      = 5_000_000
startup_budget_ms  = 30
hook_budget_ms     = 12
store_quota_mb     = 8
egress             = "none"                    # none | via-capability. Never ambient.

[hooks]
subscribe = [
  "chamber.message.posting",                   # pre-commit: may veto, may annotate
  "chamber.message.posted",                    # post-commit: observe only
  "governance.proposal.opened",
  "install.activating",
  "install.migrating",
  "schedule.tick",
]

[schedule]
tick = "0 */6 * * *"                           # host-scheduled; jittered; skipped if throttled

[[ui.panel]]
surface   = "chamber"
id        = "polls.active"
title     = "Active polls"
icon      = "ballot"                           # from the design-system icon set only
view      = "views/panel.fnui.json"            # declarative Surface Descriptor (§7)

[[ui.command]]
id        = "polls.open"
title     = "Open a poll"
keywords  = ["poll", "vote", "straw"]
view      = "views/compose.fnui.json"

[[ui.settings]]
id    = "polls.settings"
view  = "views/settings.fnui.json"

[[cli.command]]
path  = "poll"                                 # becomes `fn ext polls poll …` and, if
                                               # promoted by the Society, `fn poll …`
spec  = "cli/poll.fncli.json"

[assets]
root         = "assets/"
max_total_mb = 3
preload      = ["icons/ballot.svg"]

[i18n]
default = "en"
locales = ["en", "es", "de", "ja"]

[data]
schema_version = 4
migrations     = "migrations/"                 # 1→2, 2→3, 3→4 as WASM exports (§8.3)
export         = "supported"                   # supported | partial | none
uninstall      = "retain-30d"                  # retain-30d | purge-immediate

[signature]
algorithm = "ed25519"
key_id    = "fn1qzs…k4r7#pub2"
sbom      = "sbom.spdx.json"                   # mandatory (P8)
digest    = "blake3:9f2c…a71e"                 # over the canonicalized bundle
```

### 3.2 Field rules

| Field group | Rule | Enforcement |
|---|---|---|
| `extension.id` | Reverse-DNS, immutable for the life of the Extension, owned by one publisher FNID | Registry uniqueness; transfer requires a signed two-sided event |
| `extension.version` | Strict semver; a published version is immutable — a fix is a new version, never a re-upload | Registry rejects digest change at an existing version |
| `capability` | Every entry needs `reason`, rendered verbatim in the consent screen. No `reason`, no publish | Publish-time lint; review pipeline (`19`) |
| `capability.scope` | May reference `${install.*}` placeholders bound at install time by the installing Citizen | Bound values become part of the Envelope, not of the manifest |
| `limits` | Requested, not granted. The host clamps every value to the policy ceiling for the Extension's kind and the Society's Level | Host, at instantiation |
| `hooks.subscribe` | Closed set from the published catalog. Unknown hook id = publish failure, not a silent no-op | Publish-time validation against the Host API version in `compat` |
| `ui.*` | Every view is a Surface Descriptor path; no HTML, CSS, or JS may appear anywhere in the bundle | Bundle scanner rejects `.html`, `.css`, `.js` outside `sdk` kind |
| `data.schema_version` | Monotonic. Must have a migration for every gap since the previously published version | Publish-time check across the version history |
| `signature` | Ed25519 over the canonical bundle digest, plus an SBOM. Unsigned bundles are unrepresentable in the registry | Registry; host re-verifies at install and at every load |

**Invariant X3.** An Extension Install's Envelope is exactly the intersection of (manifest-requested capabilities) ∩ (installing Citizen's own capabilities) ∩ (Charter `agent_policy` and extension policy) ∩ (kind defaults) ∩ (Society Level ceiling). The manifest is a *request*. It has never, at any point in the code, been a grant.

---

## 4. Execution Model

`10 §11` commits to the **WASM Component Model**. That commitment is what makes the rest of this chapter buildable: components have typed, non-ambient imports, so "what can this code reach" is answered by reading its world, not by auditing its behaviour.

### 4.1 The world

```wit
package fractal:host@1.4.0;

interface types {
  type society-id = string;  type chamber-id = string;  type message-id = string;
  record principal { fnid: string, kind: principal-kind }
  variant error { denied(string), quota(string), invalid(string), unavailable(string) }
}

interface chamber {                          // one import per capability domain
  use types.{chamber-id, message-id, error};
  record message-view { id: message-id, author: principal, body: string, at: u64 }
  read-message: func(id: message-id) -> result<message-view, error>;
  post-message: func(c: chamber-id, body: string) -> result<message-id, error>;
}

interface store {                            // per-Install, per-Society KV (§8)
  get: func(key: string) -> result<option<list<u8>>, error>;
  put: func(key: string, value: list<u8>) -> result<_, error>;
  list: func(prefix: string, cursor: option<string>) -> result<page, error>;
}

interface clock { now: func() -> u64; }      // capability-gated; absent by default
interface log   { emit: func(level: level, msg: string); }   // always present, rate-capped

world extension {
  import types; import log;
  import chamber; import store; import clock;               // only if granted
  export lifecycle: interface {
    activate:   func(ctx: install-context) -> result<_, error>;
    deactivate: func() -> result<_, error>;
    migrate:    func(from: u32, to: u32) -> result<_, error>;
  }
  export hooks: interface {
    on-message-posting: func(m: message-view) -> hook-decision;
    on-message-posted:  func(m: message-view);
    on-tick:            func(at: u64);
  }
}

variant hook-decision {
  proceed,
  proceed-with(annotation),      // additive metadata only, never body rewrite
  veto(veto-reason),             // requires a capability that permits veto
}
```

An import the Envelope does not grant is **not stubbed — it is absent from the instance**. A guest compiled against `clock` and installed without `system.clock` fails instantiation with a typed error surfaced to the installing Citizen, rather than running and misbehaving at 3am.

### 4.2 What WASI preview2 gives us, and what we take back

| preview2 area | Fractal Node exposure |
|---|---|
| `wasi:io` streams | Only as the plumbing under granted host interfaces. No raw handles. |
| `wasi:filesystem` | **Not imported.** No ambient filesystem, no preopens, ever. Persistence is `store` (§8) and `vault` (capability-gated). |
| `wasi:sockets`, `wasi:http` | **Not imported.** Network egress only via `net.fetch<host:…>` with an allowlist fixed at install and shown on the consent screen. |
| `wasi:clocks` | **Not imported by default.** `clock.now` requires `system.clock`; it returns host time coarsened to 1ms for plugins, and to tick time inside an Experience. |
| `wasi:random` | Replaced by `rng.next` seeded per invocation from the host's `Rng` port, so hook executions are replayable (`10 §7`). |
| `wasi:cli` env/args | **Not imported.** Configuration arrives as a typed `install-context`. |
| `wasi:logging` | Replaced by `log.emit`, rate-capped, attributed to the Install, visible in the audit trail. |

### 4.3 Instantiation, lifecycle, and cancellation

```
  install ──► verify signature + digest + SBOM ──► compile (AOT, cached per digest)
                                                        │
                          ┌─────────────────────────────▼──────────────────────────┐
                          │ INSTANCE POOL (per Society × Install)                  │
                          │  warm: N instances, each a fresh Store over one Module │
                          └─────────────────────────────┬──────────────────────────┘
   hook fires ──► checkout instance ──► set fuel + epoch deadline ──► call export
                        │                                                │
                        │  ◄── returns within budget ── proceed/veto ────┤
                        │                                                │
                        │  ◄── epoch deadline hit ── TRAP ── instance destroyed,
                        │                              hook recorded as `timeout`,
                        │                              decision defaults to PROCEED
                        ▼
                 return instance to pool (state reset) or destroy on trap
```

- **Memory** is a hard `memory_mb` ceiling from the clamped limits; a growth request past it traps.
- **Fuel** meters instructions per call. Exhaustion traps. Fuel is the defence against a guest that is merely slow; the epoch deadline is the defence against a guest blocked in a host call.
- **Cancellation** is host-initiated: epoch interruption plus instance destruction. There is no cooperative cancel, because cooperation is exactly the thing an adversarial guest declines to do.
- **A trap never fails the user's operation.** A vetoing hook that traps yields `proceed`; a mutating hook that traps yields the unmutated value. Fail-open for the *host operation*, fail-closed for the *Extension* — the Install's health counter increments and §11 takes over.
- **Determinism.** Instantiation is pure: same module digest + same `install-context` + same inputs + same seeded `rng` ⇒ same outputs. Guests get no ambient time, no ambient entropy, no ambient I/O and no shared memory, so a hook execution is replayable from the event log for debugging and for dispute resolution in `19`. Floating-point is canonicalized NaN; SIMD is permitted, threads are not.

---

## 5. Capability Security and Attenuation

An Extension Install is a Principal (`01 §2`) and holds an **Envelope** — the same structure an Agent holds (`11 §2.8`), with the same mandatory `expires_at`, the same `granted_by: Fnid` that must resolve to a Citizen, and the same evaluation at the Policy Enforcement Point in the application layer. There is no second authorization system for Extensions. This is the single most important reuse decision in the chapter: the Envelope machinery is adversarially tested by the Agent runtime for two phases before third-party code executes at all (`02 §3`).

```
   MANIFEST REQUEST          INSTALLING CITIZEN'S CAPS        CHARTER EXT POLICY
          │                            │                             │
          └────────────┬───────────────┴──────────────┬──────────────┘
                       ▼                              ▼
                 ∩  intersection  ∩  kind defaults  ∩  Society Level ceiling
                       │
                       ▼
              ┌──────────────────┐   handed to the guest as
              │  ENVELOPE (X3)   │──►  TYPED IMPORTS, one per granted domain
              └──────────────────┘     (no token, no string, no ambient object)
                       │
                       ▼ guest calls a sub-component or a nested Experience
              attenuate(caps ⊆ own caps)  ──► child handle. Never ⊇. (Invariant X4)
```

**Attenuation.** A guest may derive a weaker handle from one it holds and pass it onward — `chamber.attenuate({ chambers: [c1], verbs: [read] })` returns a handle strictly inside the parent's authority. The constructor takes the parent handle as input and computes an intersection, exactly as `15 §4.2` does for Envelope grants, so widening is unrepresentable rather than merely rejected. Every derived handle inherits the parent's expiry and dies with it; revoking the Install revokes the whole tree in one epoch bump.

**Invariant X4.** For every derived handle `h'` from `h`, `caps(h') ⊆ caps(h)` and `expiry(h') ≤ expiry(h)`. Checked by property test over generated handle graphs.

**Invariant X5.** A new Extension Install with an empty Envelope can read exactly the public data a signed-out visitor can read, and can write nothing. This is P8's falsification test applied to Extensions verbatim.

---

## 6. The Hook Catalog

**Naming law, derived from `01 §8`:** a hook in the **present participle** fires pre-commit and may influence the outcome; a hook in the **past tense** fires post-commit and may only observe. `chamber.message.posting` can veto; `chamber.message.posted` cannot. The tense *is* the contract, in code, in docs, and in the consent screen.

Post-commit hooks are dispatched asynchronously off the command path and can never delay a user's operation. Pre-commit hooks run inline inside the budget in §11 and are the only hooks that can make the host feel slow — which is why their budget is the strictest number in this document.

| # | Hook | Signature sketch | Veto | Mutate | Capability required |
|---|---|---|---|---|---|
| 1 | `chamber.creating` | `(spec) -> decision` | yes | name/topic only | `chamber.create` |
| 2 | `chamber.created` | `(chamber-view)` | — | — | `chamber.read` |
| 3 | `chamber.archiving` | `(chamber-id) -> decision` | yes | — | `chamber.archive` |
| 4 | `chamber.message.posting` | `(draft) -> decision` | yes | annotate only | `chamber.message.post` + `hook.veto` |
| 5 | `chamber.message.posted` | `(message-view)` | — | — | `chamber.read` |
| 6 | `chamber.message.editing` | `(prev, next) -> decision` | yes | — | `chamber.message.post` |
| 7 | `chamber.message.reacted` | `(message-id, reaction)` | — | — | `chamber.read` |
| 8 | `chamber.thread.resolving` | `(thread-id) -> decision` | yes | — | `chamber.thread.resolve` |
| 9 | `member.joining` | `(candidate, charter-view) -> decision` | yes | — | `member.gate` (Charter-delegated only) |
| 10 | `member.joined` | `(membership-view)` | — | — | `member.read` |
| 11 | `member.left` | `(fnid, reason)` | — | — | `member.read` |
| 12 | `member.restricting` | `(fnid, action) -> advisory` | no | — | `moderation.observe` |
| 13 | `member.standing.explaining` | `(fnid) -> list<explanation>` | — | — | `progression.read` |
| 14 | `governance.proposal.opening` | `(proposal) -> decision` | yes | — | `governance.proposal.gate` |
| 15 | `governance.proposal.opened` | `(proposal-view)` | — | — | `governance.read` |
| 16 | `governance.vote.cast` | `(vote-receipt)` | **never** | **never** | `governance.read` |
| 17 | `governance.charter.enacting` | `(diff) -> advisory` | no | — | `governance.charter.read` |
| 18 | `economy.transfer.quoting` | `(transfer) -> fee-annotation` | no | fee ≤ Charter cap | `economy.fee.quote` |
| 19 | `economy.transfer.settled` | `(posting-view)` | — | — | `ledger.read<scope>` |
| 20 | `economy.contribution.signalling` | `(window) -> list<signal>` | — | clamped, weighted by S6 | `economy.signal.submit` |
| 21 | `economy.sink.proposing` | `(candidate) -> proposal` | no | — | `economy.sink.propose` (Charter-gated) |
| 22 | `facet.minting` | `(mint-request) -> decision` | yes | — | `facet.mint.gate` |
| 23 | `facet.evolving` | `(facet-id, delta) -> decision` | yes | — | declared in the Facet's `EvolutionRules` |
| 24 | `vault.object.storing` | `(object-spec) -> decision` | yes | path/tags | `vault.object.write<path:…>` |
| 25 | `vault.object.stored` | `(object-ref)` | — | — | `vault.object.read<path:…>` |
| 26 | `vault.object.exporting` | `(export-manifest)` | — | — | `vault.export.observe` |
| 27 | `media.rendition.requesting` | `(media-meta) -> list<profile>` | no | profile set | `media.rendition.request` |
| 28 | `media.preview.rendering` | `(object-ref) -> surface` | — | — | `vault.object.read<path:…>` |
| 29 | `search.indexing` | `(doc) -> list<term>` | no | additive terms | `search.index.contribute` |
| 30 | `search.querying` | `(query) -> list<result>` | no | own source only | `search.query` |
| 31 | `agent.tool.offering` | `() -> list<tool-def>` | — | — | `agent.tool.offer` |
| 32 | `agent.tool.invoking` | `(tool, args) -> result` | — | — | union of the tool's declared caps |
| 33 | `ui.panel.<surface>` | `(context) -> surface` | — | — | per-panel data caps |
| 34 | `ui.command` | `(invocation) -> surface \| action` | — | — | per-command caps |
| 35 | `ui.settings` / `ui.profile.module` / `ui.market.shelf` | `(context) -> surface` | — | — | `extension.store.read`, plus declared |
| 36 | `cli.command` | `(argv, stdin) -> exit + stream` | — | — | same caps as the GUI equivalent (P13) |
| 37 | `schedule.tick` | `(at)` | — | — | none beyond the Install's Envelope |
| 38 | `install.activating` / `install.deactivating` / `install.migrating` | lifecycle | — | — | none |

**Hard limits on the economy hooks — these are P12 encoded, not policy:**

- No hook can emit Fraction. `Emission` originates only from the `EmissionAccount` under S6 (`10 §3`, `11 §2.6`). There is no capability, reserved or otherwise, that lets guest code create supply.
- Hook 18 may *annotate* a fee; the Charter's `economy` parameters cap it, and a quote exceeding the cap is clamped and logged as `ExtensionQuoteClamped`, not rejected silently.
- Hook 20 submits *signals*, not scores. The Economy boundary weights, clamps, and Sybil-checks them. An Extension cannot write a Contribution Score, because a Source that a third party controls is a mint a third party controls.
- Hook 21 *proposes* a Sink; enactment is a governance act by Citizens.
- **XP, Trust, and Standing have no write hooks at all.** Hook 13 explains Standing; nothing writes it. Volume-farmable reputation written by installable code is the fastest known route to violating P12 and `02 §4`'s pay-to-win prohibition.

**Veto etiquette.** A veto returns a reason string that is shown to the acting Citizen along with the Extension's name. Anonymous denial is forbidden: if an Install can stop a person from speaking, that person learns which Install did it and why. Veto counts are metered per Install per day; an Install exceeding its veto budget is throttled to advisory-only and the Society is notified.

---

## 7. UI Contribution

An Extension declares UI as a **Surface Descriptor**: a JSON document composed exclusively of design-system primitives, with data bound to values the Extension returns from a hook.

```json
{ "surface": "panel", "version": 1,
  "root": { "type": "Stack", "gap": "md", "children": [
    { "type": "Heading", "level": 3, "text": "{{poll.question}}" },
    { "type": "Repeat", "of": "poll.options", "as": "opt", "child":
      { "type": "MeterRow", "label": "{{opt.label}}", "value": "{{opt.share}}",
        "action": { "command": "polls.vote", "args": { "option": "{{opt.id}}" } } } },
    { "type": "Text", "tone": "muted", "text": "Quorum {{poll.quorum}} · closes {{poll.closes|relative}}" }
  ] } }
```

**Extensions do not get DOM, CSS, JavaScript, canvas, or an iframe.** They get a bounded component vocabulary, a bounded expression language (property access, a fixed filter list, no arbitrary evaluation), and host-rendered output. The host renders the same descriptor to React on web and desktop, to native views on mobile, and to ANSI blocks in the Terminal — which is how an Extension satisfies P13 without writing three UIs, and how N7's single-source design system survives contact with third parties.

| Why this line is drawn here | Consequence |
|---|---|
| Arbitrary rendering means arbitrary styling, and arbitrary styling means a marketplace that looks like a bazaar and fails N8 contrast audits on install | Every Extension inherits accessibility, theming, keyboard nav, screen-reader labels, and dark mode for free — and cannot lose them |
| An iframe is a spoofing primitive: it can paint a convincing wallet-transfer confirmation | Host-rendered chrome is unforgeable; the consent and confirmation surfaces are never inside an Extension's tree (§13, T8) |
| Arbitrary DOM means arbitrary layout thrash and unbounded main-thread work | P10's interaction budget is enforceable because the host owns the render |
| Descriptors are data | They are cacheable, diffable, replayable, and renderable offline (P2) |

**Honest cost.** Some genuinely good UI cannot be built. A rich diagram editor, a spreadsheet grid, a waveform scrubber — none of these is expressible in a constrained vocabulary. Our answer is not "add an escape hatch"; it is (a) grow the primitive vocabulary through `32-design-system.md` when a real Extension proves the need, with the primitive going to first-party and third-party at the same moment (P7), and (b) route genuinely custom real-time rendering to the Experience Runtime in §12, where the sandbox is stronger and the surface is a Chamber the Citizen deliberately entered. There is no third path. An extension that wants pixels asks to be an Experience.

**CLI parity.** Every `ui.command` must declare a `cli.command` equivalent or explicitly declare `cli = "not-applicable"` with a reason shown in the listing. N3 and P13 apply to third parties too.

---

## 8. Extension Data

| Concern | Rule |
|---|---|
| Scope | Storage is keyed `(society_id, extension_id, install_id)`. One Extension installed in two Societies shares nothing between them. P1 has no exception for Extensions. |
| Store | Ordered KV with prefix scan, transactional per hook invocation. Values ≤ 256KB; total per Install per Society ≤ `store_quota_mb`, clamped by Society Level. |
| Large objects | Not in the store. An Extension with `vault.object.write<path:/ext/<id>/**>` writes to the Vault, counts against the Society's storage, and is visible in the Vault UI like any other object. Invisible storage is a billing lie. |
| Events | An Install may emit `extension.<id>.*` domain events into the Society Log with `envelope_ref` set (`10 §5`). These replay like any other event, which is what makes extension state reconstructible (P6). |
| Secrets | Never in the store, never in events. Egress credentials live in the host's `KeyStore` and are reachable only as an opaque handle inside a granted `net.fetch` (P8). |
| Quota exhaustion | Writes fail with `error::quota`; the Society is notified with a one-click raise (within Level ceiling) or purge. No silent eviction. |
| Migration | `data.schema_version` with an exported `migrate(from,to)`. The host runs migrations in order, in a transaction, offline from hooks, with a snapshot taken first. A failed migration rolls back and pins the Install to the previous version. |
| Uninstall | `retain-30d` (default): data is frozen, hidden, restorable, then purged, and the purge is an event. `purge-immediate`: gone at uninstall, and the consent screen says so at install time. |
| Export | A Citizen or the Society exports extension data as part of the standard Society export bundle: raw KV, declared schema, and Vault objects under the Extension's path. An Extension declaring `export = "none"` is ineligible for the marketplace (`19`) — data that cannot leave is a hostage, and P9 makes exportability a property of the platform, not a vendor's kindness. |

---

## 9. Versioning and Compatibility

The **Host API** is versioned as a whole, in semver, independently of the Runtime: hook catalog, WIT worlds, Surface Descriptor schema, and manifest schema at one revision.

| Change | Bump | Effect on installed Extensions |
|---|---|---|
| New hook, new import, new optional manifest field, new UI primitive | minor | None; `>=1.4,<2.0` guests keep running |
| Hook removed, signature changed, capability semantics narrowed | **major** | Guests must recompile against the new world |
| Host bug fix, no signature change | patch | None |

**Deprecation window:** a deprecated item is marked in the WIT and the reference and keeps working for **two minor versions or 180 days, whichever is longer**, then is removed at the next major. Removal requires a per-item migration note, a registry scan naming every affected Extension, publisher notification at deprecation and at 30 days, and a shim for the whole of the previous major's 12-month support window.

**Rolling out a breaking change:**

```
  ADR ──► new world published ALONGSIDE the old (dual-host period, same Envelope)
   │
   ├─► registry scan: N Installs affected ──► publishers notified + preview build
   ├─► Societies see a per-Install badge: "recompile needed by <date>"
   │
  180d ──► old world read-only: existing Installs run, new installs blocked
  360d ──► old world removed; remaining Installs auto-DISABLED, data retained (§8),
           cause and publisher named to the Society
```

An Extension is never silently uninstalled and its data is never destroyed by version policy. The worst outcome we inflict is a disabled Install with intact, exportable data and a named cause.

**Compatibility matrix** — published, machine-readable, consumed by `fn ext check`:

| Host API | Runtime | Descriptor | Manifest | Status | Support ends |
|---|---|---|---|---|---|
| 1.0–1.3 | 0.7–0.9 | 1 | 1 | Deprecated | Phase 5 gate |
| 1.4 | ≥0.9 | 1 | 1 | Current | — |
| 2.0 (planned) | ≥1.2 | 2 | 2 | Draft | — |

---

## 10. Consent and Audit

The **install-time consent screen** is host-rendered, never inside an Extension surface, and shows six things: identity and publisher FNID with verification state; every requested capability in plain language with the manifest's `reason` verbatim; the egress allowlist (or "none"); what data is stored and what happens to it on uninstall; the surfaces it will occupy; and the resource ceiling. Capabilities that X3's intersection stripped are shown as *not granted* rather than hidden — a Citizen should see that an Extension asked for more than they can give.

**Re-consent on update is mandatory.** An update whose capability set is a strict subset auto-applies. An update requesting **anything** not already granted — wider scope, higher limit, a new egress host, a new hook that can veto — halts, and the previous version keeps running until a Citizen with grant authority signs. The screen shows a **diff**: new highlighted, unchanged collapsed. There is no auto-update path that widens authority. This is `15 §4.2`'s "sign the diff, not the request" applied to Extensions, and it is the control that defuses T4.

**Runtime prompts** cover a narrow class: an action inside a granted capability but above a confirm threshold (a transfer near the cap, egress to a newly-resolved host, a first write under a Vault path). Prompts are host-rendered, name the Install, and are rate-limited — prompt fatigue is an attack, not an annoyance.

**Audit.** Every host call is recorded as `(install_id, envelope_ref, interface, function, arg digest, decision, fuel, wall time)`. Denials are `ExtensionActionBlocked` domain events. `fn ext audit <install> --since 7d` and the GUI panel render the same data, including a capability-usage summary that pre-deselects unexercised capabilities at renewal, so Envelopes narrow by default (`15 §4.5`).

---

## 11. Performance

**The rule: an Extension can never make the host feel slow.** A budget, measured per Install, enforced by a state machine — not by review and not by publisher good intent.

| Budget | Ceiling (plugin) | Measured as |
|---|---|---|
| Cold instantiation (AOT-cached) | 30 ms p99 | host timer around `activate` |
| Pre-commit hook call | **8 ms p95 / 20 ms p99** | epoch deadline at 25 ms, then trap |
| Post-commit hook | 250 ms p99, off the command path | async worker |
| Resident memory | 24 MB default, 64 MB ceiling | linear memory cap |
| Fuel per call | 5M default, 25M ceiling | fuel meter |
| Surface Descriptor | 128 KB, 500 nodes | validated at return |
| **All Installs, one command, pre-commit** | **25 ms aggregate** | host accumulator |

The last row is load-bearing. Once a command's pre-commit allowance is spent, remaining hooks are skipped with `proceed` and recorded as `deferred`. A Society that installs twenty chatty plugins gets degraded extensions, never a degraded Chamber.

```
  HEALTHY ──(p95 over budget, 3 windows)──► DEGRADED ── pre-commit hooks skipped,
     ▲                                          │        post-commit still runs
     │  ◄──(2 clean windows)────────────────────┘
     │
     └──(trap rate >1% or p99 over, 3 windows)──► THROTTLED ── 1 call / 10s
                                                       │
                             (no recovery in 24h) ─────► DISABLED — data retained,
                                                         cause named, one-click re-enable
```

Health is per `(Society, Install)`, so one Society's abuse never disables an Extension elsewhere. All four states are domain events: degradation is auditable and appealable, not a mystery.

---

## 12. The Experience Runtime (Phase 7)

`02 §3` calls this "the most seductive scope trap in the entire document." That judgement stands, and this section is architecture only: nothing here is built before the gates in §12.6 pass.

An **Experience** is an interactive, hosted, governed application in a Chamber of kind `experience`. It differs from a plugin in three ways that each demand a stronger model: it runs a **real-time loop** rather than answering hooks; it renders **pixels** rather than descriptors; and it holds **session state for many Citizens at once**.

### 12.1 Sandbox

| Property | Plugin | Experience |
|---|---|---|
| Process | Shared host, pooled instances | **Own OS process** per session, seccomp-restricted, cgroup CPU/memory caps |
| Host world | Full plugin world, capability-gated | **Disjoint, smaller world.** No `chamber.*`, no `vault` write, no `governance.*`; Society interaction only via §12.4 mediated calls |
| Scheduling | Event-driven, budget per call | Fixed tick, budget per tick, overrun sheds ticks |
| Client | None — host renders descriptors | A client component drawn into a host-owned, host-chromed canvas with no access to the surrounding document |

Three of the four extraction criteria in `10 §2` hold, so the Experience host is a separate service from day one of Phase 7 — not a Runtime module.

### 12.2 State, tick, session

```
  ┌────────── EXPERIENCE SESSION (one per Chamber instance) ──────────┐
  │  AUTHORITATIVE STATE — server-owned, never client-owned           │
  │        ▲                    │ snapshot + deltas @ tick (20 Hz)    │
  └────────┼────────────────────┼──────────────────────────────────────┘
     intents│                    ▼
  ┌─────────┴───────────┐   ┌──────────────────────────────────────┐
  │ CLIENT: prediction, │◄──│ RELAY (S14) per-session fan-out,     │
  │ interpolation,      │   │ backpressure, no client authority    │
  │ rollback on diverge │   └──────────────────────────────────────┘
  └─────────────────────┘
```

- **Server-authoritative, always.** Clients send intents, never state. Prediction is a rendering convenience corrected by the next delta — the same rule, for the same reason, as the wallet row in `10 §6`.
- **Tick.** Fixed timestep, deterministic, host-seeded RNG, no wall-clock reads. A tick over budget is dropped; sustained overrun lowers the tick rate before it touches correctness.
- **Lifecycle** `Provisioned → Running → Draining → Sealed`. Inputs and the RNG seed are logged, so a session replays exactly — which is how a contested outcome is adjudicated by evidence rather than by support ticket.
- **Persistence is explicit.** Durable writes happen only at checkpoints, into the Install's §8 store or the Vault. Tick-loop code has no persistence import at all.

### 12.3 Hosting in a Chamber

An `experience` Chamber holds an installed Experience, an access policy (roles, Level, Standing gates — like any Chamber), a session policy (max concurrent sessions, participants, idle timeout), and a resource budget bound to the Society. Entering joins or provisions a session; every start and end is a domain event in the Society Log.

### 12.4 Governance and economy — decisions with limits

| Question | Decision | Limits and justification |
|---|---|---|
| Charge Fraction? | **Yes**, via `economy.charge<session>` | Only after a host-rendered price the participant confirmed, before entry or before a discrete purchase. Per-session and per-Citizen-per-day caps set by the Charter. Balanced Postings with a named `PostingReason` and the `19` revenue split. No auto-renewal. No charging inside the tick loop — it is an async mediated call the tick observes. |
| Mint Facets? | **Yes, never autonomously, never in the loop** | Only against a Facet Standard registered at install, inside a governance-approved mint budget with a hard rate cap, via `facet.mint.request`, executed by the Asset boundary and gated by hook 22. The Experience proposes; the Society mints. |
| Award XP, Trust, Standing? | **No. Ever.** | `02 §4` bans pay-to-win; an installable game that mints reputation is exactly that. Experiences may award their own Insignia Facets carrying no progression weight. |
| Emit Fraction? | **No.** | P12. Rewards come from the Install's Wallet or a Charter-authorized Treasury budget — redistribution, never a Source. |
| Hold governance authority? | **No.** | It may render a governance surface and read the Charter. Enactment stays with Citizens (P4). |
| Assets | **Vault only** | Content-addressed, encrypted, Manifest-verified, BLAKE3 verified-streaming (`11 §6`), client-cached by digest. Bytes count against Society storage and appear in the Vault UI. No arbitrary CDN fetch. |
| Cross-Society multiplayer | **Deferred past Phase 7** | Requires Federation (Phase 6) *and* a settled cross-Society Envelope story. Single-Society sessions first. |

### 12.5 Resource accounting

Metered per session-second, per GB egress, and per GB-month of session state, priced against the same Sinks as storage and compute in `17`. The bill lands on the hosting Society's Treasury, which may recover it via entry charges, sponsor it, or cap it — sessions refuse to provision past the cap with an honest message. A Society can never accrue unbounded liability from an Experience, and a publisher can never externalize compute cost onto a Society that did not agree a cap.

### 12.6 The gates — none of this starts until all pass

1. Envelope system adversarially tested for two full phases with zero unresolved escalation findings (`02 §3`).
2. Plugin host in production two phases, with §11's state machine demonstrably throttling real Extensions.
3. Phase 6 shipped end to end: paid third-party Extensions, review pipeline, revocation, payout disputes (`19`).
4. Relay extracted, per-session backpressure proven at target concurrency.
5. Vault streaming proven at Experience asset sizes, verified end to end.
6. A written, funded operations plan for a compute-heavy multi-tenant service, including abuse response and cost caps.
7. An ADR stating the maximum share of engineering capacity Phase 7 may consume, and **what is cut to pay for it** (`02 §8`).

Any gate failing means Phase 7 does not start. Experiences are the most compelling demo in the product and the most likely cause of its death; the gates exist because the pressure to skip them will be enormous and will arrive dressed as opportunity.

---

## 13. Threat Model

Residual risk is stated, because a threat table without residuals is a comfort object.

| # | Attack | Mitigation | Residual |
|---|---|---|---|
| T1 | Capability escalation | X3 intersection at grant; X4 attenuation-only derivation; enforcement at the Policy Enforcement Point, not in the guest | A host interface that leaks a wider handle than intended. Countered by property tests over generated handle graphs and by keeping the host surface small |
| T2 | Resource exhaustion (CPU, memory, store, veto/prompt spam) | Fuel, epoch deadlines, memory caps, the §11 aggregate ceiling and state machine, veto and prompt budgets | Coordinated many-Install pressure that stays inside budget; shows up as aggregate degradation and is bounded by the 25 ms ceiling |
| T3 | Supply-chain compromise of a dependency | Mandatory SBOM, reproducible builds verified by the registry, pinned deps, publisher signing, digest-addressed cache; blast radius is the Envelope | A compromised dep inside an Extension that legitimately holds a dangerous capability. Countered by capability minimization, not by scanning |
| T4 | Malicious update (benign v1, hostile v2) | Mandatory re-consent on any widening (§10); immutable published versions; key compromise triggers registry-wide revocation | A hostile update *within* the existing capability set. Sharpest residual in the chapter: prevention is minimization at first install; detection is the audit trail |
| T5 | Data exfiltration through a granted capability | Install-time egress allowlist shown at consent; no ambient network; egress volume metered and alertable; E2EE content is ciphertext to host and guest alike (N6) | An Extension with legitimate read plus legitimate egress. Not technically solvable; addressed by consent legibility, metering, and `19` policy |
| T6 | Host API confusion (id or handle from another scope) | Handles are unforgeable typed resources, not strings; every id is scope-checked on every call; `society_id` comes from the instance binding, never from guest input | A host function that resolves before it authorizes. Countered by one shared authorization decorator on every exported function plus a lint that fails the build if one is missing |
| T7 | Side channels (timing, cache) | No shared memory, no threads, clock coarsened to 1 ms and capability-gated, per-instance state reset, no cross-Install store access | Coarse statistical inference from host-call latency. Accepted: what it reveals is bounded by what the Envelope already permits |
| T8 | UI spoofing / clickjacking on an extension surface | No DOM, no iframe, no arbitrary rendering (§7); consent, confirmation, and payment chrome is host-rendered outside the Extension's tree; reserved brand strings rejected in descriptor text | An Extension mimicking host copy inside its own panel. Countered by persistent host-drawn attribution and by review |
| T9 | Agent–Extension collusion | Hook 32 runs under the **intersection** of Agent and Install Envelopes, never the union; both `envelope_ref`s land on the event; confirm classes are the union of both | A pair with overlapping legitimate authority reaching a result no human intended. Countered by Guardian anomaly detection and blast radius, not by prevention |
| T10 | Sandbox escape | Memory-safe runtime, no guest FFI, minimal imports, bug bounty; Experiences additionally in a seccomp-restricted process with cgroup caps | A WASM runtime zero-day. Countered by depth and by a rehearsed kill switch: one registry flag disables an Extension across every Society |
| T11 | Economic manipulation (score farming, value minting) | No emission capability exists; hook 20 is clamped and weighted; no XP/Trust/Standing writes; Experience mints queued and governance-gated | Signal-weighting exploits. Countered by `17`'s simulation harness treating Extension signals as adversarial at 100× volume |
| T12 | Cross-Society leakage | Storage keyed by `(society_id, extension_id, install_id)`; instance bound to one Society; `CrossSociety` blast radius needs grants on both sides | Exfiltrate from A, re-enter in B via a granted egress endpoint. Detected by egress metering; stated plainly at consent |

---

## 14. Developer Experience

```
  fn ext new my-thing --kind plugin --lang rust    scaffold: manifest, world, tests
  fn ext dev --society dev_local                   watch, rebuild, hot-swap instance
  fn ext check                                     manifest lint, WIT compat, budgets
  fn ext test                                      Fake Society harness
  fn ext bench                                     startup / per-hook / memory vs §11
  fn ext audit <install> --since 7d                what it actually did
  fn ext sign && fn ext publish                    sign, SBOM, submit to review (19)
```

**Loop.** `fn ext dev` runs a local Runtime with a synthetic Society, recompiles on save, and hot-swaps between hook calls. A swap is a new instance, never a live patch — so hot reload has identical semantics to a version upgrade and cannot hide a state bug that production would expose.

**Harness.** A **Fake Society**: a real Runtime with in-memory `EventStore`, `Ledger`, `BlobStore` and deterministic `Clock`/`Rng`/`IdGen` (`10 §7`). Tests declare Citizens, Chambers, a Charter and an Envelope, drive hooks, and assert on emitted domain events and store contents. Because the ports are deterministic, an extension test is a replayable transcript — the mechanism that makes the domain layer testable makes third-party code testable at zero extra cost.

**Debugging.** DWARF preserved through AOT; `fn ext dev --trace` prints every host call with args, decision, fuel and wall time; any recorded audit record replays (§4.3), *including one from production*, which is how a publisher reproduces a bug in a Society whose data they cannot see.

**Languages: Rust and TypeScript first.** Rust because the Runtime is Rust, the SDK is then a thin typed wrapper over generated bindings, and `wasm32-wasip2` has the best component tooling. TypeScript because it is the largest population of people who will actually write plugins, and ComponentizeJS produces a real component — engine plus guest, still sandboxed and metered — rather than a second, weaker runtime. Its startup and memory cost is paid by the guest inside its own budget, which is exactly where it belongs. Go, Python and C# follow as their toolchains stabilize; nothing in the host is language-specific, so a new language is an SDK, not a host change.

---

## 15. Rejected Alternatives

| Alternative | Why rejected |
|---|---|
| **JS sandbox (QuickJS, isolates)** | Isolation becomes a property of the engine's correctness rather than of the module boundary; capability control means intercepting globals, which is auditing by denylist. No comparable fuel metering, one language, weaker determinism. Fails P8. |
| **Containers per Extension** | Right isolation, wrong granularity: hundreds of ms of startup against an 8 ms hook budget, tens of MB each, and an ambient-authority model we would spend this whole chapter taking back. Retained only as the *outer* layer for Experience sessions, where the cost amortizes. |
| **Native dynamic libraries** | Zero isolation; one bad plugin corrupts the Runtime and the event log. Not a trade-off, a category error under P8. |
| **Iframe / arbitrary web UI** | A spoofing primitive (T8), unbounded main-thread cost against P10, breaks N7's single design system, and is meaningless on CLI and native mobile — it would quietly make the GUI the privileged front end, violating P13. |
| **No third-party code at all** | Safest, and it fails P7: the API is never exercised adversarially, the marketplace has no reason to exist, and first-party features drift back into privileged paths within two phases. |
| **Our own scripting DSL** | We would build a language, a debugger and an ecosystem instead of a platform. Fails `02 §5`. |
| **Per-Extension processes for plugins** | Rejected on latency — IPC per hook against an 8 ms budget. Adopted for Experiences, where a tick loop amortizes it. |

---

## 16. Invariants and Open Items

Each invariant becomes a property test, per `11 §7`.

- **X1** Every post-Phase-3 first-party feature maps to an Extension id or a `10 §3` boundary.
- **X2** One Host API implementation; first-party guests get no privileged loader.
- **X3** An Install's Envelope is the §3.2 intersection — the manifest is never a grant.
- **X4** Derived handles are strictly weaker and expire no later than their parent.
- **X5** An Install with an empty Envelope can do nothing beyond public reads (P8).
- **X6** No guest code path emits Fraction or writes XP, Trust, or Standing (P12).
- **X7** Every host call produces an audit record carrying `install_id` and `envelope_ref`.
- **X8** A guest trap never fails the host operation, and always increments Install health.
- **X9** Extension data is keyed by `society_id`; no cross-Society read path exists (P1).
- **X10** Every registry bundle is signed, SBOM'd, digest-addressed, and immutable at its version.

**Proposed additions to `01-canonical-terminology.md`**, to be merged in the same PR as this chapter: **Host API**; **Hook** (a named extension point — present participle is pre-commit and influential, past tense is post-commit and observational); **Surface Descriptor**; **Fuel**; **Install Health**; **Session** (an Experience Runtime instance).

**ADRs required before Phase 3 implementation:** WASM runtime selection with a component-model maturity assessment; the Surface Descriptor primitive vocabulary as a formal schema owned by `32-design-system.md`; the Host API 1.0 hook freeze; governance of the Reserved Hook Register; and the Experience Runtime capacity ADR named in gate 7.
