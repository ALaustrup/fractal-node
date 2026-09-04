# 31 — The CLI and Terminal Experience

> **Prerequisites:** `00-foundational-principles.md`, `01-canonical-terminology.md`, `30-api-and-sdk.md`, `32-design-system.md`, `33-brand-identity.md`.
> **Governs:** the `fractal` / `fn` binary, its command grammar, machine-readable contracts, the branded terminal experience, and the interactive dashboards.
> **Non-negotiable N3:** the CLI is a first-class front end, not a wrapper. Anything the GUI can do, the CLI can do — enforced at every release tag (P13 falsification test).

---

## 1. Why the CLI Is Load-Bearing

Three constituencies need a terminal interface, and each would be badly served by an afterthought:

1. **Agents.** An AI agent operating Fractal Node needs deterministic, parseable, self-describing output and a dry-run mode. A GUI is unusable to it and a raw HTTP client is unergonomic. The CLI is the agent's native habitat.
2. **Node operators.** Running a Node, serving as a Custodian, inspecting the ledger, and diagnosing sync are terminal work.
3. **Power users and developers.** Building Extensions, scripting Society administration, and inspecting one's own data.

Because all three exist from day one, the CLI ships in Phase 1 alongside the web GUI — not later. And because P13 makes parity a release gate, the CLI cannot silently fall behind.

**The mechanism that makes this cheap** (see `30` and `41`): the CLI command tree is *generated* from the same schema that generates the API, the SDKs, and the docs. An RPC declared without a `fn.cli` binding fails the schema lint. Parity is therefore a build error, not a review conversation.

---

## 2. Installation and Invocation

### 2.1 Global invocability

```
  Windows   winget install FractalNode.CLI      → %LOCALAPPDATA%\Programs\FractalNode\bin  (PATH)
            scoop install fractal
  macOS     brew install fractalnode/tap/fractal → /opt/homebrew/bin
  Linux     curl -fsSL https://get.fractalnode.dev | sh   → ~/.local/bin (no sudo by default)
            apt / dnf / AUR / nix packages
  Any       cargo install fractal-cli
  Desktop   the desktop app installs the CLI and offers to add it to PATH
```

Two names, one binary: `fractal` is canonical, `fn` is a shipped alias. `fn` collides with a Fish shell builtin and with `fn` from Fn Project; the installer detects both, warns, and offers `fnode` as a third alias rather than silently shadowing.

Callable from any directory. **The CLI has no project directory concept** — there is no `fractal init` producing a `.fractal/` folder that commands must be run inside. Context comes from configuration and flags, never from `cwd`. This is deliberate: agents and cron jobs do not have a working directory you can rely on.

### 2.2 Zero-dependency binary

A single statically-linked binary, ≤ 12MB compressed, no runtime, no Node.js, no Python. Cold start to first byte of output: **≤ 60ms** on `fn status`, budgeted and CI-enforced. A CLI that takes half a second to print a version number tells the user everything they need to know about the rest of the product.

---

## 3. Command Grammar

### 3.1 The shape

```
  fn [global-flags] <noun> <verb> [target] [flags]
```

Noun-verb, never verb-noun. Nouns come verbatim from `01-canonical-terminology.md`; verbs come verbatim from the verb vocabulary in `01 §8`. A CLI command that uses a word not in the Canon fails the schema lint.

```
  fn society create "Oracle Hall" --visibility discoverable
  fn society fracture soc_01H8X --plan ./split.toml --dry-run
  fn chamber post general "the relay is stable"
  fn wallet transfer @kaya 40FRC --memo "shard repair"
  fn wallet history --since 30d --format json
  fn agent grant archivist chamber.read chamber.post --limit 200/day --ttl 14d
  fn agent revoke archivist --envelope env_01H8X
  fn vault put ./masters/ --path /archive/2026 --replicas 6
  fn facet mint --standard FN-ASSET/1 --schema ./insignia.json
  fn ext publish ./dist --price 250FRC
  fn custodian status --watch
  fn charter diff v6 v7
  fn node logs --follow --boundary ledger
```

### 3.2 Global flags (identical on every command)

| Flag | Effect |
|---|---|
| `--society <id\|handle>` | Sets the Society context (P1 — most commands need one) |
| `--as <principal>` | Act as an enrolled Agent or an alternate Persona |
| `--format <human\|json\|ndjson\|table\|csv\|yaml>` | Output contract (§4) |
| `--dry-run` | Full policy + domain evaluation, returns effects, emits nothing |
| `--yes` | Skip interactive confirmation (never skips a *policy* confirmation) |
| `--quiet` / `--verbose` / `--trace` | Verbosity; `--trace` prints the correlation ID and every API call |
| `--no-color` / `NO_COLOR` | Disables ANSI |
| `--profile <name>` | Named credential/context profile |
| `--timeout <dur>` | Request timeout |
| `--idempotency-key <k>` | Explicit key; otherwise generated and echoed |

### 3.3 Configuration precedence

```
  explicit flag  >  FN_* environment variable  >  --profile  >  ~/.config/fractal/config.toml  >  built-in default
```

Config is a single TOML file. `fn config show --explain` prints every effective value **and where it came from** — the single most useful diagnostic command in any CLI, and almost never implemented.

### 3.4 Discoverability

`fn help`, `fn <noun> --help`, and `fn help <noun> <verb>` are generated from the same schema as the docs, so help text can never drift from behavior. Shell completions (bash, zsh, fish, PowerShell, nushell) are generated and installed by the package. `fn search <term>` does fuzzy discovery across all commands and flags.

---

## 4. Machine-Readable Output (the agent contract)

This section is a contract, not a feature list. Agents depend on it.

### 4.1 Format guarantees

| Format | Contract |
|---|---|
| `human` | Default when stdout is a TTY. **No stability guarantee.** May change freely. |
| `json` | A single JSON document. **Stable within an API major version.** Additive changes only. |
| `ndjson` | One JSON object per line, for streams and long lists. Flushed per record. |
| `table` | Fixed columns, tab-separated, header line. Stable. |
| `csv` | RFC 4180. Stable. |
| `yaml` | For config-shaped output. Stable. |

**Auto-detection:** if stdout is not a TTY and `--format` is absent, the CLI emits `json`. Piping `fn wallet history` into `jq` works without a flag. This is the single highest-leverage ergonomic decision in the CLI.

### 4.2 The JSON envelope

Every non-stream `json` response has exactly this shape:

```json
{
  "ok": true,
  "data": { "...": "command-specific" },
  "meta": {
    "correlation_id": "01JC8X...",
    "society_id": "soc_01H8X...",
    "acted_as": "fn1qk7...",
    "envelope_ref": null,
    "api_version": "v1",
    "cli_version": "0.4.2",
    "dry_run": false,
    "elapsed_ms": 41
  },
  "warnings": []
}
```

Errors are the same envelope with `ok: false` and a structured `error`:

```json
{
  "ok": false,
  "error": {
    "code": "capability_denied",
    "title": "Transfer refused",
    "detail": "Envelope env_01H8X permits wallet.transfer<=100FRC/day; 140 FRC requested; 60 FRC remaining in this window.",
    "missing_capability": "wallet.transfer<=140FRC/day",
    "retryable": false,
    "remedy": {
      "human": "Raise the limit in Policy, or send 60 FRC.",
      "command": "fn agent grant archivist 'wallet.transfer<=200FRC/day' --ttl 7d"
    },
    "docs": "https://docs.fractalnode.dev/errors/capability_denied"
  },
  "meta": { "...": "..." }
}
```

The `remedy.command` field is the differentiator. An agent that hits a wall is handed the exact next action rather than being left to infer it. Combined with `GET /v1/capabilities` (`30`), an agent can plan instead of probing — which matters because probing generates `AgentActionBlocked` events that count against its Trust.

### 4.3 Exit codes (closed set)

```
  0  success
  1  generic failure
  2  usage error (bad flags, unknown command)
  3  authentication required or expired
  4  capability denied  (the Envelope lacks it)
  5  not found
  6  conflict (idempotency, version, or state)
  7  rate limited          (Retry-After echoed in stderr JSON)
  8  network / node unreachable
  9  policy confirmation required (interactive confirmation refused or unavailable)
 10  dry-run reported blocking invariant violations
 70  internal error (a bug; prints a correlation ID and a report command)
```

Stable forever. Scripts depend on these.

### 4.4 Streaming

`--watch` and `--follow` emit `ndjson` when not a TTY. A stream begins with a header record `{"type":"header",...}`, emits `{"type":"record",...}`, and heartbeats `{"type":"heartbeat","seq":N}` every 15s so a consumer can distinguish "idle" from "hung". On reconnect it resumes with `since(seq)` — the same resume contract as the Signal WebSocket (`14`).

### 4.5 Dry run

`--dry-run` runs the full Policy Enforcement Point *and* the full domain decision, evaluates the `11 §7` invariants, returns the complete effect set, and emits no events. `fn society fracture` **requires** a dry run first and consumes the returned `dry_run_token` — the mandatory-dry-run rule from `11 §3.2` enforced at the wire, not by convention.

---

## 5. Authentication

```
  fn auth login
  ┌────────────────────────────────────────────────────┐
  │  ⌁  FRACTAL NODE // AUTHORIZATION                  │
  │                                                    │
  │  Open:  https://fractalnode.dev/device             │
  │  Code:  QRTX-8891                                  │
  │                                                    │
  │  ● waiting for approval                            │
  └────────────────────────────────────────────────────┘
```

Device-code flow with a device-bound keypair generated locally; the private key never leaves the machine and is stored in the OS keychain via the `KeyStore` port. **No password is ever typed into the CLI.**

For unattended use, `fn auth token create --envelope <id>` issues a credential that is a *pointer to an Envelope* — scoped, rate-limited, TTL-bound, revocable, and auditable — never a bearer god-token (`12`, `30`). `fn auth whoami --explain` prints the effective capability set, its limits, and its consumption.

---

## 6. The Boot Sequence

`fn` invoked with no arguments in an interactive TTY enters the Terminal. It opens with a boot sequence — 1.4 seconds, skippable with any key, disabled by `--no-boot`, `NO_COLOR`, or a non-TTY. It is the same motion as the marketing bumper (`33 §8`), which is the point: the ad is the product's boot.

```
                              ⌁

                        ┌───────────┐
                     ╱  │           │  ╲
                    │   │     ◇     │   │            ← orbit rings resolve
                     ╲  │           │  ╱               out of the grain
                        └───────────┘

                    F R A C T A L   N O D E          ← letter-spacing tightens
                        ACCESS TERMINAL                from .40em to .18em

    [ 01 / RUNTIME  ]  core 0.4.2 · wasm ok · 14 crates      OK
    [ 02 / IDENTITY ]  fn1qk7…3xz · 2 devices · key fresh    OK
    [ 03 / RELAY    ]  wss://relay.fractalnode.dev · 41ms    LIVE
    [ 04 / VAULT    ]  1.4 TiB held · 6 shards degraded      DEGRADED
    [ 05 / LEDGER   ]  block 44812 · anchored 2m ago         OK
    [ 06 / SOCIETIES]  4 active · 1 syncing                  OK

    ● 1,204.482913 FRC        L7 ▓▓▓▓▓░░ 4,210 / 6,000 XP

    ⌁ ready
    fn ›
```

**Each line is a real check, not theater.** The boot sequence is a health report that happens to be beautiful. If the Relay is down, line 03 says `OFFLINE` in `--fn-accent-danger` and the Terminal still opens in local-first mode (P2). A boot sequence that lies about system state would be the worst possible first impression for an instrument.

Frames are drawn with the `32 §10` terminal palette and degrade down the ladder: truecolor → 256 → 16 → glyphs only.

---

## 7. The Terminal: Interactive Dashboards

The Terminal is a full-screen TUI (ratatui) that is a *peer* to the GUI, not a lesser mirror. It shares the design system's layout grammar: rail, list, stage, context, status bar.

```
┌ ⌁ FN // ORACLE-HALL ──────────────────────────── ● LIVE ── 1,204 FRC ── @kaya ┐
│                                                                               │
│ ◈ SOCIETIES    │ # general                     │ MEMBERS            41        │
│ ▌◇ oracle-hall │                               │  ● kaya         L7  T+412    │
│  ◇ signal-lab  │ 14:02 @kaya                   │  ● rin          L4  T+188    │
│  ◇ the-commons │   relay is stable at 41ms     │  ○ voss         L9  T+901    │
│                │                               │                              │
│ CHAMBERS       │ 14:04 ⟡ archivist             │ AGENTS                       │
│ ▌# general     │   indexed 4,102 objects.      │  ⟡ archivist  env 12d left   │
│  # signal      │   6 shards degraded, repair   │  ⟡ steward    env  3d left   │
│  # treasury    │   scheduled 14:30.            │                              │
│  ◇ commons     │   ── under envelope env_01H8X │ VAULT                        │
│                │                               │  1.4 TiB · 6 degraded        │
│ AGENTS         │ 14:06 @voss                   │  repair 14:30                │
│  ⟡ archivist   │   approve the repair budget?  │                              │
│  ⟡ steward     │                               │ TREASURY                     │
│                │ ┌───────────────────────────┐ │  84,201.44 FRC               │
│ XP ▓▓▓▓▓░░ L7  │ │ ›                         │ │  −412 this week              │
│                │ └───────────────────────────┘ │                              │
├───────────────────────────────────────────────────────────────────────────────┤
│ FN://ORACLE-HALL/GENERAL · SYNCED 2s · 14 CUSTODIANS · BLOCK 44812 · ⌘K       │
└───────────────────────────────────────────────────────────────────────────────┘
```

Note that the agent message carries the violet `⟡`, the violet rail, and an explicit "under envelope env_01H8X" line — the same P4 visibility contract as the GUI, in a character cell.

### 7.1 Dashboard views

| View | Command | Shows |
|---|---|---|
| Society | `fn` or `fn society open <h>` | The layout above |
| Wallet | `fn wallet` | Balance, settled vs pending, sparkline, recent Postings, accrual sources |
| Custodian | `fn custodian` | Shards held, bytes served, attestations passed/failed, FRC accrued this window, repair queue |
| Node | `fn node` | Runtime health, sync lag per Society, event log position, adapter status, resource use |
| Agents | `fn agent` | Enrolled Agents, active Envelopes with TTL countdowns, recent actions, blocked actions |
| Ledger | `fn ledger` | Postings, running balance, anchor status, invariant check results |
| Governance | `fn charter` | Open proposals, votes, Charter diff |
| Market | `fn market` | Installed Extensions, updates (with capability deltas highlighted), listings |

### 7.2 Interaction

Modal, vim-adjacent, and fully discoverable: `?` shows the keymap for the focused pane at all times.

```
  ⌘K / :        command palette (the same command set as the GUI)
  Tab / S-Tab   cycle panes            j/k or ↑/↓   move within a pane
  Enter         activate               Esc          back / dismiss
  g s           go to Society          g w  wallet   g v  vault   g a  agents
  /             search in pane         n/N          next/prev match
  y             copy focused value     o            open in GUI
  ?             keymap                 q            quit
```

`o` deep-links the focused object into the desktop or web GUI (`fractal://society/…`). Moving between CLI and GUI mid-task is one keystroke — the practical expression of P13.

### 7.3 Accessibility in the Terminal

The Terminal must be usable in a screen-reader-friendly mode: `--simple` renders a linear, non-redrawing, semantically-ordered stream instead of a full-screen TUI, because TUI redraws are hostile to screen readers. All information remains reachable. Reduced motion disables the pulse and the boot animation. Every color-coded state also carries a word or glyph.

---

## 8. Agent Ergonomics

Beyond the machine-readable contract, four features exist specifically because agents are first-class (P4):

1. **`fn capabilities`** — prints the caller's effective CapabilitySet with limits and current consumption. An agent plans against this instead of probing.
2. **`--dry-run` everywhere**, returning the full effect set and invariant evaluation (§4.5).
3. **`fn explain <command>`** — describes what a command would do, which capabilities it requires, which events it would emit, and whether it is reversible. Reads like a man page written for a planner.
4. **`fn schema`** — dumps the machine-readable command tree (JSON) so an agent can discover the entire surface without scraping help text.

Combined, these let an agent operate the platform correctly on the first attempt, which matters because a wrong attempt is a `AgentActionBlocked` event with reputational cost.

---

## 9. Extension and Plugin Surface

Extensions may contribute CLI commands under a reserved namespace: `fn ext:<extension> <verb>`. The namespace prefix is mandatory and non-negotiable — an Extension can never shadow or override a core command, which closes an obvious phishing and confusion vector. Extension commands are declared in the manifest (`20`), inherit the global flags, and must honor the same output contracts. `fn ext list --commands` shows exactly which commands come from where.

Developer loop: `fn ext new`, `fn ext dev --watch` (hot reload against a sandbox Society), `fn ext test`, `fn ext audit` (capability diff vs the previous version), `fn ext publish`.

---

## 10. Performance Budgets

| Command class | Budget | Enforcement |
|---|---|---|
| `fn --version`, `fn status` (cached) | 60ms | CI benchmark, fails on regression |
| Any single-API-call command | 250ms + network | CI benchmark |
| Terminal cold open | 400ms to first frame | CI benchmark |
| Terminal frame | 16ms | trace |
| Binary size | ≤ 12MB compressed | build gate |
| Memory (Terminal, 4 Societies) | ≤ 120MB | soak test |

---

## 11. Trade-offs and Rejected Alternatives

| Choice | Why | Honest cost | Rejected |
|---|---|---|---|
| Rust + clap + ratatui | Shares the core crates directly (`41`), single static binary, fast start | Rust TUI ecosystem is smaller than Go's | Go + Bubble Tea (excellent TUIs, but cannot link the Rust core — would mean a second implementation, violating P13); Node/Python (runtime dependency, slow start) |
| Generated command tree | Makes P13 parity a build error | Codegen complexity in `xtask` | Hand-written commands (guaranteed drift — this is precisely how CLIs become second-class) |
| Auto-JSON when piped | Removes the most common friction for scripts and agents | A user who pipes `human` output for reading must pass `--format human` | Always-human default (hostile to automation); always-JSON (hostile to humans) |
| Noun-verb grammar | Matches the Canon vocabulary; groups discovery naturally | Slightly more typing than verb-first | Verb-noun (`fn create society`) — reads better in isolation, discovers worse at scale |
| No project directory | Works identically from anywhere; safe for agents and cron | No implicit per-repo context | `.fractal/` directory (a footgun for unattended execution) |
| Boot sequence that runs real checks | Brand and diagnostics in one artifact | 1.4s on interactive open (skippable) | Decorative animation (would be a lie, and lies in an instrument are unforgivable); no boot (forfeits the strongest brand moment we have) |
| TUI as a peer, not a mirror | Serves operators and agents genuinely | A second interaction surface to maintain and test | Read-only CLI (relegates terminal users to second class, violating N3) |
