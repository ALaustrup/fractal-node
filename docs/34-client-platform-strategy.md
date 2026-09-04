# 34 — Client Platform Strategy

> **Prerequisites:** `00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`, `10-system-architecture.md`, `33-brand-identity.md`.
> **Governs:** the shared-core contract across every Front End; the per-target stack, packaging, and update story for Windows, Web, PWA, Android, iOS, macOS, Linux, and the CLI; the Windows desktop flagship specification including the responsive ladder from handheld to ultrawide; the mobile decision; design-token portability (N7); parity enforcement (P13); the parallel roadmap; and the per-platform performance, offline, and testing budgets.

---

## 1. What this chapter fixes

N2 says cross-platform from day zero, never a later port. That sentence is cheap to write and expensive to keep. It is kept by one structural commitment — **the product is a Rust library with several shells** — and by a set of invariants that make drift detectable rather than merely discouraged.

| # | Invariant | Falsification test |
|---|---|---|
| I1 | Exactly one implementation of every domain rule exists, and it is in Rust. | Any business rule (validation, permission check, balance arithmetic, conflict resolution, XP formula) implemented in TypeScript, Swift, or Kotlin is a violation. Detected by a review checklist plus a lint on forbidden identifiers in shell code. |
| I2 | Every Front End reaches the Runtime through the same public API surface, in-process or over the wire (P3). | Grep shells for HTTP paths absent from the schema registry, or direct store access. |
| I3 | Core crates compile to `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `aarch64-apple-ios`, `aarch64-linux-android`, and `wasm32-unknown-unknown` on every commit to `main` (N2). | The build matrix is a required check. A target may not be marked `allow-failure`. |
| I4 | One design token source compiles to every surface, including the Terminal (N7). | Regenerate in CI; a non-empty `git diff` fails the build. |
| I5 | One core version per release. All shells in a release tag link the identical core commit. | The release manifest records one core SHA; a shell pinning a different SHA blocks the tag. |
| I6 | No platform-only capability without an ADR that names the platform mechanism it depends on and the degradation on every other target. | Parity gate, §9. |
| I7 | Every target has an offline answer for every read path and a queued answer for every write path (P2), or an explicit, declared, reviewed exception. | Airplane-mode suite per target, §13. |
| I8 | Performance and accessibility budgets are per-target and enforced in CI (P10, N8). | Budget job, §11. |

The rest of this chapter is the elaboration of those eight lines.

---

## 2. The shared-core architecture

### 2.1 The layer diagram

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  SHELL LAYER — per platform, deliberately not shared                                 │
│  window/lifecycle · OS integration · input · rendering host · store packaging         │
│                                                                                       │
│ ┌─────────┐┌─────────┐┌─────────┐┌─────────┐┌─────────┐┌─────────┐┌─────────┐┌──────┐│
│ │ Win32/  ││ Browser ││ Service ││ Android ││  UIKit  ││ AppKit  ││ GTK/    ││ TTY  ││
│ │ WinUI + ││   tab   ││ Worker  ││ Activity││ scene   ││ scene   ││ wlroots ││ +PTY ││
│ │ WebView2││         ││ +manifest││        ││         ││         ││ WebKitGTK││     ││
│ └────┬────┘└────┬────┘└────┬────┘└────┬────┘└────┬────┘└────┬────┘└────┬────┘└───┬──┘│
└──────┼──────────┼──────────┼──────────┼──────────┼──────────┼──────────┼─────────┼───┘
       │          │          │          │          │          │          │         │
┌──────┼──────────┼──────────┼──────────┼──────────┼──────────┼──────────┼─────────┼───┐
│  PRESENTATION LAYER — shared where the idiom matches, forked where it does not        │
│      ▼          ▼          ▼          ▼          ▼          ▼          ▼         ▼    │
│ ┌──────────────────────────────────┐ ┌───────────┐┌───────────┐ ┌───────────────────┐│
│ │  React + TypeScript + Vite       │ │ Jetpack   ││  SwiftUI  │ │  ratatui + ANSI   ││
│ │  ONE codebase: web, PWA, desktop │ │ Compose   ││           │ │  Terminal (31)    ││
│ │  responsive ladder §6            │ │ (Phase 5) ││ (Phase 5) │ │                   ││
│ └──────────────┬───────────────────┘ └─────┬─────┘└─────┬─────┘ └─────────┬─────────┘│
└────────────────┼───────────────────────────┼────────────┼─────────────────┼──────────┘
                 │                           │            │                 │
┌────────────────┼───────────────────────────┼────────────┼─────────────────┼──────────┐
│  BINDING LAYER — thin, generated, never hand-written business logic                   │
│                 ▼                           ▼            ▼                 ▼          │
│  ┌────────────────────────┐  ┌──────────────────────────────┐  ┌───────────────────┐  │
│  │ wasm-bindgen (browser) │  │  UniFFI  →  Kotlin / Swift   │  │ direct Rust calls │  │
│  │ Tauri IPC (desktop)    │  │  (mobile, Phase 5)           │  │ (CLI, headless)   │  │
│  └───────────┬────────────┘  └───────────────┬──────────────┘  └─────────┬─────────┘  │
└──────────────┼───────────────────────────────┼───────────────────────────┼───────────┘
               └───────────────────┬───────────┴───────────────────────────┘
                                   ▼
╔══════════════════════════════════════════════════════════════════════════════════════╗
║  THE CORE — one Rust workspace, identical bytes of logic on every target              ║
║                                                                                       ║
║  ┌──────────────┐┌──────────────┐┌──────────────┐┌──────────────┐┌─────────────────┐  ║
║  │  fractal-    ││  fractal-    ││  fractal-    ││  fractal-    ││  fractal-       │  ║
║  │  domain-*    ││  core        ││  sync        ││  store       ││  crypto         │  ║
║  │  (10 §3)     ││ (client core,││ (outbox,     ││ (local       ││ (MLS, FNID,     │  ║
║  │  pure, no I/O││  query cache,││  CRDT merge, ││  replica,    ││  KeyStore port) │  ║
║  │              ││  no PEP)     ││  reconcile)  ││  projections)││                 │  ║
║  └──────────────┘└──────────────┘└──────────────┘└──────────────┘└─────────────────┘  ║
║                                                                                       ║
║  ┌ ─ ─ ─ ─ ─ ─ ─ ─  OUTSIDE THE CORE — SERVER ONLY  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐             ║
║    fractal-app-*  ·  fractal-api-*  ·  fractal-adapter-*  ·                           ║
║    fractal-node  ·  fractal-cli   — none of these compiles to wasm32.                 ║
║    The POLICY ENFORCEMENT POINT lives in fractal-app-kernel and stays                 ║
║    inside the trust boundary (41 §8.1, 10 §8). A client MAY compute an                ║
║    ADVISORY affordance hint from fractal-domain-agent's pure decision                 ║
║    function — to grey out a control rather than offer it and fail — but               ║
║    every command is re-decided at the authoritative PEP. A hint that                  ║
║    disagrees with the PEP is a UI defect, never an authorization                      ║
║    outcome.  (61 X2)                                                                  ║
║  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘              ║
║  ┌──────────────┐┌──────────────┐┌──────────────┐┌──────────────┐┌─────────────────┐  ║
║  │ ledger client││ offline queue││ Signal client││ tokens (N7)  ││ ports (P5 traits)│ ║
║  └──────────────┘└──────────────┘└──────────────┘└──────────────┘└─────────────────┘  ║
╚══════════════════════════════════════════════════════════════════════════════════════╝
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
        ┌────────────────────┐         ┌────────────────────────┐
        │  EMBEDDED  — the   │         │  REMOTE — the same core │
        │  desktop/mobile    │         │  running as a headless  │
        │  shell IS a Node   │         │  Node (10 §9)           │
        └────────────────────┘         └────────────────────────┘
```

The reason the desktop shell embeds the core in-process (`10 §11`) rather than talking to a hosted Runtime is P2. A Node holds an authoritative local replica; a thin remote-only shell cannot. The same crates, unmodified, are what a headless Node runs. **The difference between the flagship desktop application and a data-centre Node is which shell is linked, not which logic is compiled.**

### 2.2 What is shared and what is not

| Concern | Shared? | Where it lives | Why |
|---|---|---|---|
| Domain rules, invariants, state machines | Always | `fractal-domain-*` | I1. Two implementations of the XP formula is two economies (P12). |
| Command/query handling, policy evaluation | Always | `fractal-app` | The Policy Enforcement Point must be on every path (`10 §8`). |
| Local replica, projections, migrations | Always | `fractal-store` (SQLite native / OPFS-backed VFS in the browser) | One schema, one migration ladder. |
| Sync engine, outbox, conflict policy | Always | `fractal-sync` | The per-class conflict table (`10 §6`) is one table, not seven. |
| Crypto, MLS group state, FNID, signing | Always | `fractal-crypto` | N6 is unverifiable if reimplemented per platform. |
| Design tokens | Always, generated | `design/tokens/` → §8 | N7. |
| API types, event schemas, error taxonomy | Always, generated | schema registry → TS/Swift/Kotlin/Rust | Drift here is silent breakage. |
| Layout, navigation, gesture, focus order | Never | shell/presentation | A phone is not an ultrawide; pretending otherwise produces two bad products. |
| OS integration (notifications, tray, share, files) | Never, but behind one Rust trait per capability | `fractal-shell-ports` | Shells implement; the core requests. |
| Media capture and playback pipeline | Partly | codecs and ladder logic in Rust; capture and surface handoff native | Hardware access is irreducibly native. |
| Text input, IME, autocorrect | Never | platform | Reimplementing IME is how products lose non-Latin markets. |

### 2.3 The sharing percentages, stated honestly

Measured as share of non-generated source lines in a shipped target, at the Phase 5 steady state. These are targets with tolerances, checked quarterly by a `tokei`-driven report, not aspirations.

| Target | Core (Rust) | Shared presentation | Target-specific | Shared total | Tolerance |
|---|---|---|---|---|---|
| Windows desktop | 62% | 26% (React, shared with web) | 12% | **88%** | ≥82% |
| Web | 58% (wasm) | 34% | 8% | **92%** | ≥88% |
| PWA | 58% | 34% | 8% (worker, manifest, install flow) | **92%** | ≥88% |
| macOS desktop | 62% | 26% | 12% | **88%** | ≥82% |
| Linux desktop | 62% | 26% | 12% | **88%** | ≥80% |
| Android | 55% | 0% | 45% (Compose) | **55%** | ≥50% |
| iOS | 55% | 0% | 45% (SwiftUI) | **55%** | ≥50% |
| CLI | 71% | 0% | 29% (Terminal, ratatui) | **71%** | ≥65% |

**Read the mobile row as the price of the decision in §7.** Native mobile buys best-in-class interaction and costs roughly 45% of each app's source as unshared UI. That is a real, permanent, recurring cost — two teams writing two navigation stacks forever — and it is the single largest maintenance liability in this document. It is accepted for the reasons in §7 and it is the first thing to reverse if the reversal conditions fire.

### 2.4 Where sharing breaks down, and the rule for when to stop

Four boundaries are where the shared-core model stops paying:

1. **Layout and navigation.** A shared layout engine across a 7-inch handheld and a 5120px ultrawide produces the union of two compromises. Shared *tokens* and shared *components* pay; shared *screens* do not.
2. **Background execution.** Every operating system has a different, mutually incompatible model — Windows has no meaningful constraint, Android has WorkManager and Doze, iOS has BGTaskScheduler with an opaque scheduler, browsers have Background Sync with no guarantees. The sync engine therefore exposes a *step function* (`sync_step(budget: Duration) -> StepOutcome`) and each shell decides when to call it. The core never owns a timer.
3. **Push delivery.** APNs and FCM require platform-side registration and platform-side payload handling; the browser requires the Push API with VAPID. The notification *policy* (`14 §11`) is core; the *transport* is per-shell.
4. **Text and media input.** IME, dictation, autocorrect, camera, screen capture. Always native.

**The stopping rule:** share a thing when the shared version would be at least as good as the best per-platform version. Otherwise define a port trait and let shells implement it. Sharing that degrades the experience is a P10 violation wearing the costume of engineering discipline.

---

## 3. The core-as-library contract

### 3.1 The binding surfaces

| Consumer | Mechanism | Generated from | Call style |
|---|---|---|---|
| Tauri desktop (Win/mac/Linux) | Direct Rust linkage + Tauri IPC commands to the webview | `#[tauri::command]` + shared type crate → TS via `ts-rs` | Async, JSON over IPC for commands; a binary channel for Signal frames and media |
| Web / PWA | `wasm-bindgen` + `wasm-pack`, core as an ES module | Rust types → TS declarations, one generator | Async via `Promise`; workers for the sync engine |
| Android | UniFFI → Kotlin | `.udl`-free proc-macro scaffolding | Suspend functions over a Rust-owned Tokio runtime |
| iOS / macOS native pieces | UniFFI → Swift | same source | `async`/`await`, `Sendable` façade types |
| CLI, headless Node, agents | Direct Rust dependency, no binding at all | — | Native calls |

One generator run produces all of them. Adding a core method that appears in only one binding is a build failure — the binding surface is a single `#[fractal_api]`-annotated façade module, and each backend emits from that module or fails.

### 3.2 What crosses the boundary

Only four kinds of value ever cross:

```
   SHELL                          BOUNDARY                          CORE
   ─────                          ────────                          ────
   intent  ─────────────► Command {idempotency_key, payload} ─────► app layer
                          (fire-and-forget; result arrives as an event)

   render  ◄───────────── QueryResult (immutable snapshot DTO) ◄─── projections
                          (versioned, generated, no behaviour)

   live    ◄───────────── Signal frame (delta, ordered per society) ◄ relay client
                          (push; the shell never polls)

   os      ◄───────────── ShellRequest {notify, open_url, pick_file,
                            store_key, wake_at, share} ─────────────► port trait
                          (the core asks; the shell obeys or refuses)
```

**No handles, no callbacks holding core state, no partially-initialized objects, no shared mutable memory.** The boundary is values in, values out, plus one event stream. This is what keeps four shells honest and what makes the boundary testable with a single conformance suite (§13).

### 3.3 Serialization cost, measured not assumed

| Boundary | Format | Cost at p50 | Mitigation |
|---|---|---|---|
| Tauri IPC (desktop) | JSON over the webview bridge | ~0.9 µs/KB encode + bridge hop; a 200-message Chamber page ≈ 180 KB ≈ 1.6 ms | Pagination caps at 200; Signal deltas are binary over a raw channel; media never crosses IPC — it is served from a local `asset://` origin |
| wasm-bindgen (web) | Structured values across the JS/wasm heap boundary | Copy cost dominates; a 180 KB page ≈ 2.4 ms including UTF-8 re-encode | Keep projections in wasm memory and hand JS *view slices* per rendered row rather than whole pages; virtualize every list |
| UniFFI (mobile) | Generated FFI buffers | ~1.3 µs/KB; a page ≈ 2.1 ms | Same pagination; Kotlin/Swift models are value types constructed once per page, not per frame |
| CLI / headless | none | 0 | — |

**The invariant that keeps this from rotting:** the boundary is crossed at most once per rendered page and once per Signal, never once per row and never inside a frame. A budget test asserts IPC bytes per interaction against `perf/budgets.json`; the Chamber scroll test fails at >64 KB per 60 frames.

### 3.4 Threading model

The core owns one multi-threaded Tokio runtime per process (`current_thread` on wasm32, where there is no thread pool and the executor is driven by the microtask queue). Rules:

- **The shell's UI thread never blocks on the core.** Every façade method is async; the desktop bridge and the mobile bindings both dispatch onto the core runtime.
- **The local store is single-writer, many-reader.** SQLite in WAL mode on native; on the web, one dedicated Worker owns the OPFS handle and all other contexts message it. Two tabs writing the same OPFS database is the top browser data-corruption vector, and the Worker-ownership rule is what prevents it.
- **The sync engine is a state machine driven by `sync_step`, never a thread of its own.** This is what allows iOS BGTaskScheduler, Android WorkManager, browser Background Sync, and a desktop timer to drive identical logic.
- **Crypto (MLS) runs on a dedicated blocking pool** so that a large group rekey cannot stall command handling.
- **`Clock`, `Rng`, `IdGen` are ports (`10 §7`)**, so every one of the above is deterministically testable and replayable.

### 3.5 The same core powers a headless Node

A headless Node is `fractal-core` plus the gateway, minus a shell. It is the same crate graph the desktop links. Consequences worth stating: the desktop shell can serve its own local API on a loopback socket (making `fn` on the same machine talk to the desktop's embedded Runtime rather than a hosted one); a self-hosted Node in Phase 6 needs no new code path; and every performance improvement to the core lands on all eight targets simultaneously. This is the concrete cash value of P13.

---

## 4. Per-target deep dives

Each target below is specified on the same nine axes. Where a row says "inherits", the target adds no new architecture — a distinction that matters for the complexity budget (`02 §5`) and is defined in §10.1.

### 4.1 Windows desktop — the flagship (Phase 2)

| Axis | Specification |
|---|---|
| Stack | Tauri v2 · WebView2 (Evergreen, fixed-version fallback) · React + TypeScript + Vite · Rust core in-process · `gilrs` for gamepad · `windows-rs` for OS integration |
| Why | Embeds the Runtime in-process, so the flagship is a true Node (P2). ~12 MB shipped shell against Electron's 150 MB+. One React codebase with web. Rust already required for the core, so no new language in the build. |
| Honest cons | WebView2 version skew across machines; Microsoft's Evergreen runtime updates under us and has broken layout before. No control over the compositor. GPU rasterization quirks on old Intel drivers. Debugging spans two runtimes (Rust and JS) with two toolchains. |
| Distribution | NSIS per-Citizen installer as primary (§6.6); WinGet manifest; MSIX to the Microsoft Store from Phase 5; MSI (WiX) for managed enterprise deployment only. |
| Signing | EV code-signing certificate on an HSM/cloud KMS, Authenticode on every artifact, signed differential updates, SBOM published per release (P8). |
| Updates | Tauri updater against a signed, versioned manifest; differential payloads; staged rollout at 1%/10%/50%/100% with automatic halt on a crash-rate regression; the Citizen can defer, never be forced mid-session. |
| Integrations | Toast notifications with actions and reply · tray with live status and quick actions · jump list · `fractal://` protocol handler · file associations for `.fnbundle` (Society export) and `.fnfacet` · autostart via a per-Citizen Run entry, opt-in, off by default · taskbar overlay badge · Windows Hello for wallet confirmations via WebAuthn platform authenticator · FIDO2 hardware keys · background sync while minimized |
| Accessibility | WebView2 exposes UI Automation; every surface keyboard-operable; focus visible against the dark field with a 2px `--fn-electric` ring; Narrator and NVDA in the manual audit set; honours system high-contrast, reduced motion, and text scaling to 200% without clipping. |
| Performance | Cold ≤1.5 s, warm ≤600 ms, 60/120 fps, RSS ≤450 MB (§11). |
| Maintenance | Moderate. The dominant recurring cost is WebView2 regression triage, budgeted at ~1 engineer-week per quarter. |
| Team | 2 engineers (1 Rust/shell, 1 front end) plus shared design; a third at Phase 5 for the store channel. |

### 4.2 Web (Phase 1)

| Axis | Specification |
|---|---|
| Stack | React 18 + TypeScript + Vite; TanStack Router; the core as a lazily-loaded wasm module in a dedicated Worker; OPFS-backed SQLite (`sqlite-wasm`) for the local replica |
| Why | Phase 1 must ship a working GUI to the widest possible audience with zero install (`10 §11`). It is also the front end the desktop reuses, which is what makes the desktop cheap. |
| Honest cons | Storage is evictable and quota-limited (§12.2) — the web can never be as local-first as a Node, and we say so in the product with a staleness and durability indicator rather than pretending. wasm adds ~1.4–1.8 MB gz. No filesystem, no background execution guarantees, no hardware-key attestation on some browsers. |
| Distribution | Static assets on a CDN, immutable content-hashed filenames, `index.html` with a short TTL. |
| Updates | Instant on reload. A version Signal prompts a reload when the core schema advances; the shell refuses to run against a newer store schema rather than corrupting it. |
| Integrations | Web Share (outbound), Clipboard, File System Access where available, WebAuthn passkeys, Web Push, `registerProtocolHandler` for `web+fractal:` |
| Accessibility | The reference implementation for the whole product. axe-core in CI on every route and every dialog; manual audit with VoiceOver/NVDA per phase gate; WCAG 2.2 AA floor, AAA text contrast in the default theme (`33 §2.2`). |
| Performance | FCP ≤1.2 s, TTI ≤2.5 s p75 mid-tier (§11). |
| Maintenance | Low-to-moderate; browser churn is absorbed by Baseline-targeting and a 2-version support window plus current ESR. |
| Team | 2 front-end engineers. |

### 4.3 PWA (Phase 3)

| Axis | Specification |
|---|---|
| Stack | Inherits the web target entirely. Adds: Web App Manifest, a Workbox-free hand-written Service Worker (~400 lines, auditable), `navigator.storage.persist()`, Web Push + VAPID, Badging API, share_target, file_handlers. |
| Why | It answers the mobile need through Phase 4 at near-zero marginal cost (`02 §3`) and it is the only Front End available on platforms we will never staff. |
| Honest cons | iOS is the constraint: Web Push requires the Citizen to add to Home Screen; storage eviction is real; no background sync worth the name. Install discovery is poor everywhere. |
| Distribution | The same URL. No store, no review, no signing. |
| Updates | Service Worker with an explicit "new version — reload" prompt. Never silently swapping a running core. |
| Integrations | Share target (receive), file handlers for `.fnbundle`, launch handler for `fractal://` links, protocol handler, app badge for unread Signals. |
| Accessibility | Inherits. Adds standalone-display focus management and a visible back affordance where the OS provides none. |
| Performance | Offline shell boot ≤900 ms (§11). |
| Maintenance | Low. The Service Worker is the only novel risk; it is version-pinned to the app shell and has a kill switch. |
| Team | 0.5 engineer, shared with web. |

### 4.4 Android (Phase 5)

| Axis | Specification |
|---|---|
| Stack | Kotlin + Jetpack Compose + Material-3 skeleton restyled to the Lattice tokens · Rust core via UniFFI as a JNI library · WorkManager for deferred sync · FCM for push |
| Why | Best-in-class scroll, gesture, keyboard, and back-stack behaviour; access to the full background and notification model; predictable performance on low-end hardware where a webview shell is worst. |
| Honest cons | 45% unshared UI (§2.3). Two navigation implementations to keep in parity with iOS. Device fragmentation. NDK toolchain adds real CI time and a `cargo-ndk` dependency. |
| Distribution | Play Store AAB with per-ABI splits; a signed universal APK on the site for sideload and for regions without Play. |
| Signing | Play App Signing with an upload key in a KMS; the sideload APK signed with an independent key and published with its fingerprint. |
| Updates | Play in-app updates (flexible for minor, immediate when the core schema advances). |
| Integrations | FCM data-only push (payload decrypted locally — the push service never sees plaintext, N6) · notification channels mapped to `14 §11` classes · share target · app shortcuts · deep links via App Links · biometric prompt for wallet confirmations · StrongBox/Keystore for the `KeyStore` port · FIDO2 · Quick Settings tile for Node status · Glance widget for Society activity |
| Accessibility | TalkBack, switch access, 200% font scale, touch targets ≥48dp, `contentDescription` on every control, motion honouring `Settings.Global.ANIMATOR_DURATION_SCALE`. Accessibility Scanner + Espresso a11y checks in CI. |
| Performance | Cold ≤1.8 s on a mid-tier device, jank <0.5% of frames (§11). |
| Maintenance | High. Two OS release cycles a year, a device lab, and a permanently forked UI. |
| Team | 2 Android engineers at steady state. |

### 4.5 iOS (Phase 5)

| Axis | Specification |
|---|---|
| Stack | Swift + SwiftUI (UIKit where SwiftUI is still weak: text input, complex lists) · Rust core as an XCFramework via UniFFI · BGTaskScheduler · APNs |
| Why | Same reasoning as Android. On iOS the gap between a webview shell and native is widest in exactly the places this product lives: long virtualized message lists, keyboard behaviour, and gesture-driven navigation. |
| Honest cons | Review latency on every release. Background execution is opaque and cannot be relied on for durable sync. Two annual OS betas to chase. 45% unshared UI. |
| Distribution | App Store; TestFlight for pre-release; no sideload path in most regions. |
| Signing | Apple Developer Program, automatic signing in CI with an App Store Connect API key in a KMS, notarization for the macOS artifacts from the same pipeline. |
| Updates | App Store only. The core schema gate matters more here: a Citizen may be several versions behind for weeks, so the store must remain forward-readable for two minor versions and must refuse, cleanly, beyond that. |
| Integrations | APNs with a mutable-content extension that decrypts locally (N6) · Share extension · Shortcuts/App Intents (which is also how iOS gets a CLI-shaped surface) · Handoff · Universal Links · Face ID for wallet confirmation · Secure Enclave for the `KeyStore` port · Live Activities for a running governance vote or a Convergence · WidgetKit widget · Focus filters mapped to quiet hours |
| Accessibility | VoiceOver, Dynamic Type to AX5 without truncation, Reduce Motion, Increase Contrast, Full Keyboard Access on iPad. XCTest accessibility audits per screen in CI. |
| Performance | Cold ≤1.4 s on an iPhone 12-class device (§11). |
| Maintenance | High, and non-deferrable — an unmaintained iOS app breaks on the next OS release. |
| Team | 2 iOS engineers at steady state. |

### 4.6 macOS desktop (Phase 5)

| Axis | Specification |
|---|---|
| Stack | Inherits the Windows desktop target: same Tauri shell, same React front end, same core. WKWebView instead of WebView2. |
| Why | Zero new architecture. The cost is packaging and platform polish, not code (`02 §3`). |
| Honest cons | WKWebView diverges from Chromium on scroll physics, backdrop filters, and font rendering — the design system's grain and glow layers need a per-engine tuning pass. Notarization adds a hard dependency on Apple infrastructure in the release path. |
| Distribution | Signed and notarized `.dmg` and `.pkg` from the site; Homebrew cask; Mac App Store deferred (sandboxing conflicts with the local Node's file and network behaviour and would force a reduced build — an explicit ADR if ever reopened). |
| Updates | Same Tauri updater as Windows, with notarized differential payloads. |
| Integrations | Menu bar extra with live Node status · native menu bar with full keyboard shortcuts · Notification Center with actions · `fractal://` scheme · Quick Look for `.fnbundle` · Touch ID for wallet confirmation · Keychain for the `KeyStore` port · Universal Links · Continuity handoff to iOS |
| Accessibility | VoiceOver, full keyboard access, Increase Contrast, Reduce Motion. Manual audit at the Phase 5 gate. |
| Performance | Windows budgets +10% cold start allowance. |
| Maintenance | Low-to-moderate; dominated by annual notarization and OS-version drift. |
| Team | 0.5 engineer plus the shared desktop pair. |

### 4.7 Linux desktop (Phase 5)

| Axis | Specification |
|---|---|
| Stack | Inherits the desktop target. WebKitGTK 6 (GTK4). Wayland-first, X11 supported. |
| Why | The self-hosting Citizen (Phase 6) and the Custodian operator both live here; it is also where a headless Node and a GUI Node most often share a machine. |
| Honest cons | WebKitGTK is the weakest of the three webviews for performance and the most variable across distributions; it is the most likely single reason to revisit the Rust-native render path (`10 §12`). Distribution fragmentation is real and permanent. |
| Distribution | Flatpak on Flathub as primary; AppImage as the portable fallback; `.deb` and `.rpm` from CI for the two most common families; AUR maintained by the community with a published packaging contract. Nothing is promised for distributions we do not test. |
| Signing | Flatpak repo signing; detached GPG signatures and a `SHA256SUMS` file for every direct artifact; reproducible-build target for the core (stretch, tracked). |
| Updates | Flatpak/AppImage native mechanisms; the Tauri updater only for the direct-download artifacts. |
| Integrations | XDG desktop entry, `fractal://` scheme via `xdg-open`, XDG Desktop Portal for file pickers and screen capture, freedesktop notifications, Secret Service (or a file-backed fallback with an explicit warning) for the `KeyStore` port, tray via StatusNotifierItem where the desktop environment supports it and a graceful absence where it does not. |
| Accessibility | AT-SPI2 through WebKitGTK; Orca in the manual audit set. Honestly weaker than the other targets; declared as such rather than claimed. |
| Performance | Windows budgets +25% cold start allowance; the gap is measured and published. |
| Maintenance | Moderate. Support burden per Citizen is the highest of any target. |
| Team | 0.5 engineer plus community packagers under a written contract. |

### 4.8 CLI (Phase 0 skeleton, Phase 1 production)

| Axis | Specification |
|---|---|
| Stack | Rust · `clap` · `ratatui` for the Terminal · direct core linkage, no binding layer · the ANSI token target (§8) |
| Why | N3 and P13: the CLI is a peer Front End, and it is the parity oracle for the whole platform. It is also the fastest surface to build against, which is why it is the first to receive every new capability. |
| Honest cons | The Terminal's live dashboards are a real UI with real design cost, not a printf wrapper. Rendering the brand (`33 §4.3`) in a TTY requires capability detection and three degradation tiers. |
| Distribution | Single static binary per triple; `winget`, Homebrew, `cargo install`, a signed install script, `.deb`/`.rpm`, and a container image. `fn` and `fract` are the only two entry points. |
| Signing | Same key material and SBOM as the desktop; `fn --verify` checks its own signature. |
| Updates | `fn self update`, signature-verified, plus package-manager channels. |
| Integrations | Shell completions for bash/zsh/fish/PowerShell · `FRACTAL_*` environment contract (`01 §9`) · `--json` on every command for scripting · exit-code taxonomy · stdin/stdout streaming for pipelines · loopback attach to a local desktop Node |
| Accessibility | `NO_COLOR`, `--plain`, screen-reader mode that suppresses live-redraw regions and emits linear output, no meaning carried by colour alone, all box-drawing degradable to ASCII. |
| Performance | Startup to first byte ≤120 ms, RSS ≤80 MB, binary ≤18 MB (§11). |
| Maintenance | Low. It shares the core and has no platform runtime under it. |
| Team | 1 engineer, shared with core. |

---

## 5. Target tiers and support commitments

A target that is not tiered is a target whose support is decided by whoever files the loudest issue. Tier is a written commitment with a matching CI cost.

| Tier | Meaning | Targets | CI | Support commitment |
|---|---|---|---|---|
| **T1** | Flagship. Blocks a release. Full device lab, full a11y audit, full budget enforcement. | Windows desktop; Web (Chromium + Firefox + Safari current); CLI (Windows/macOS/Linux x86_64 + aarch64) | Every commit, real hardware nightly | Regression on T1 halts the release train |
| **T2** | Production. Blocks a release for its own phase gate; a T2-only regression can ship with a documented known issue. | PWA; Android (API 30+); iOS (last 2 major); macOS (last 2 major) | Every commit build, hardware nightly for the phase in which it ships and after | Fixed within one release cycle |
| **T3** | Supported, best-effort. Builds and passes the conformance suite; no hardware lab. | Linux desktop (Flatpak, Ubuntu LTS + Fedora current tested); community packages | Every commit build, VM smoke test | Fixed when reproducible; community packaging under a published contract |
| **T4** | Compiles, unsupported. Exists to keep the core honest. | Additional Rust triples in the N2 matrix | Build only | None |

**The tier rule:** a target may not be promoted without the CI and lab cost being funded in the same decision. Silent promotion — shipping a store listing for something with no device lab — is how a T3 becomes a support catastrophe.

---

## 6. Windows as flagship

Windows is the primary target because the flagship experience is *a Node you run*, and Windows is where the largest population of people who want to run something live — including the handheld gaming PC form factor that no competitor in this category treats seriously.

### 6.1 Window management and multi-monitor

- **Multi-window by design, not by accident.** A Chamber, a Wallet, an Agent run inspector, and the Terminal can each be torn out into their own OS window. The shell keeps one core and N webview windows; state is a single core subscription fanned to each window, so two windows never disagree.
- **Per-monitor DPI v2 awareness.** Dragging between a 96-DPI ultrawide and a 240-DPI laptop panel re-rasterizes without blur. Tested explicitly; a blurred drag is a bug, not a platform quirk.
- **Layout persistence per monitor topology.** Window positions are stored keyed by the hash of the connected display set, so undocking and redocking restores the layout the Citizen had on that topology.
- **Snap layouts, virtual desktops, and Alt-Tab** behave natively because the windows are native. Mica backdrop is used behind the void field at 6% so the app sits in Windows 11 rather than on top of it, disabled automatically under high-contrast.
- **The Command Palette** (`Ctrl+K`) is the keyboard spine of the flagship and reaches every API operation the Citizen is permitted, generated from the same schema that generates the CLI's verbs (§9). One vocabulary, three renderings: palette, CLI, API.

### 6.2 The responsive ladder — 800 px handheld through 5120 px panorama

One layout engine, six named breakpoints, defined once in the token source (§8) and consumed by CSS container queries. The engine is column-count-driven, not device-driven; a 1200 px window on a 5120 px monitor gets the 1200 px layout.

```
 width   800        1100       1440       1920       2560        3440            5120
   │──────┼──────────┼──────────┼──────────┼──────────┼───────────┼───────────────┼──►
   │ HANDHELD │ COMPACT │ STANDARD │  WIDE   │  ULTRA   │       PANORAMA          │
   │          │         │          │         │          │  (≥21:9 aspect gate)    │
   ├──────────┼─────────┼──────────┼─────────┼──────────┼─────────────────────────┤
   │ 1 pane   │ 2 panes │ 3 panes  │ 3 panes │ 4 panes  │ 4 panes + pinned column │
   │ rail as  │ rail    │ rail +   │ + right │ + second │ + optional second       │
   │ overlay  │ icons   │ list +   │ context │ context  │ Chamber column          │
   │          │         │ content  │ pane    │ pane     │                         │
   ├──────────┼─────────┼──────────┼─────────┼──────────┼─────────────────────────┤
   │ Comfort- │ Comfort-│ Comfort- │ Comfort-│ Compact  │ Compact (default)       │
   │ able     │ able    │ able     │ able    │ default  │                         │
   ├──────────┼─────────┼──────────┼─────────┼──────────┼─────────────────────────┤
   │ measure  │ measure │ measure  │ measure │ measure  │ measure capped 88ch;    │
   │ 64ch     │ 72ch    │ 76ch     │ 80ch    │ 84ch     │ surplus width becomes   │
   │          │         │          │         │          │ panes, never longer     │
   │          │         │          │         │          │ lines                   │
   └──────────┴─────────┴──────────┴─────────┴──────────┴─────────────────────────┘
```

| Breakpoint | Range | What changes |
|---|---|---|
| `handheld` | <1100 px, or any width with touch primary | Single pane; navigation rail becomes a bottom bar or overlay; touch targets ≥44 px; hover-only affordances (magnetic hover, field glow) disabled; the Composer gets a persistent send control because Enter is expensive on a virtual keyboard |
| `compact` | 1100–1439 | Two panes: icon rail + content. Context pane becomes a slide-over. |
| `standard` | 1440–1919 | Three panes: rail + list + content. The reference layout for all design work and screenshot tests. |
| `wide` | 1920–2559 | Adds the persistent right context pane (Members, Thread detail, Agent activity). |
| `ultra` | 2560–3439 | Adds a second context slot; density defaults to Compact; the Treasury and Standing charts gain their expanded series. |
| `panorama` | ≥3440 **and** aspect ≥21:9 | Optional second Chamber column (true side-by-side Chambers, each with its own scroll and Composer); a pinned column for the Terminal or a live Agent run. Content is centred with the field bleeding to the edges — never a full-bleed 5120 px text column. |

**The ultrawide invariant:** surplus horizontal space is spent on *more panes or more of the same pane*, never on longer lines of text and never on stretched imagery. A 5120 px window shows more of the product; it does not show a bigger version of it.

### 6.3 Handheld and gaming-PC form factors

Steam Deck-class hardware (1280×800 at 7 inches, ~216 effective DPI, controller-primary, often no keyboard) is a first-class Windows configuration, not an afterthought.

- **Detection, not guessing.** The shell reads the effective DPI, the physical display size, the primary pointer type, and whether a gamepad is attached (`gilrs`). `handheld` layout plus a 1.25× UI scale applies when the display is under 9 inches, regardless of resolution.
- **Controller navigation is a first-class input mode.** The focus graph that already exists for keyboard operability (an N8 requirement, so it exists whether or not we want gamepad support) is driven by a spatial-navigation resolver in Rust: D-pad and left stick move focus geometrically, A activates, B goes back, X opens the Command Palette, shoulder buttons cycle Chambers, triggers cycle Societies. **No new focus model is invented** — gamepad support is a second driver of the same graph, which is why it costs weeks rather than quarters.
- **Glyph swapping.** Button prompts follow the detected controller (Xbox/PlayStation/Deck) and revert to keyboard hints when a key is pressed. Prompts live in the footer bar, in the mono `label-s` register.
- **On-screen keyboard.** The shell invokes the OS text-entry panel on controller-driven focus into a text field, and reserves layout space so the Composer is never occluded.
- **Battery awareness.** On a device reporting battery, ambient motion (grain, orbits) drops to static below 20% and the sync engine's step interval doubles. This is a P10 obligation on hardware where the product competes with games for a power budget.

### 6.4 GPU acceleration

WebView2 composites through DirectComposition on the GPU by default. The rules that keep it there: animate only `transform` and `opacity`; the grain layer is a single viewport-sized composited layer, never one per element (`33 §4.4`); the field glow is a radial gradient on a promoted layer driven by CSS custom properties, not by re-layout; message lists are virtualized with a fixed row-height estimate and `content-visibility: auto` on off-screen groups. A CI trace asserts zero layout thrash and ≤3 raster-heavy layers during the Chamber scroll benchmark. Software-rendering fallback (old drivers, RDP sessions) is detected and drops ambient layers automatically rather than dropping frames.

### 6.5 OS integration inventory

Notifications with inline reply and actions; tray icon carrying Node status (`LIVE · STANDBY · DEGRADED · OFFLINE` from the closed status vocabulary, `33 §7.2`); taskbar overlay badge for unread Signals; jump list for recent Societies; `fractal://` protocol handler registered per-Citizen; `.fnbundle` and `.fnfacet` file associations with icons; opt-in autostart that launches minimized to tray as a background Node; Windows Hello and FIDO2 security keys through WebAuthn for wallet confirmations (P4 `confirm_classes`); background sync while minimized with a documented, inspectable power profile.

### 6.6 Installer decision: NSIS primary, MSIX secondary, WiX enterprise-only

| Option | Pros | Cons | Verdict |
|---|---|---|---|
| **NSIS (per-Citizen)** | No admin rights, so no UAC prompt on install; full control over protocol handlers, file associations, autostart; works with the Tauri signed differential updater; smallest artifact; identical mechanism across our own channel and WinGet | We own the uninstall correctness; no OS-level containerization; scriptable installers carry a reputational association with adware that must be countered by signing and a clean, quiet UI | **Primary** |
| **MSIX** | Clean install/uninstall guaranteed by the OS; Store distribution and Store-managed updates; declarative protocol handlers, startup tasks, and share targets; better enterprise story | Filesystem and registry virtualization complicates a local Node that manages a large replica and a keystore; the Tauri differential updater cannot patch a package — updates must go through the Store or App Installer, splitting our update path in two; Store review latency contradicts the release cadence | **Secondary, Phase 5**, for reach and for Citizens who require Store provenance |
| **WiX / MSI** | Group Policy deployment, per-machine install, the format IT departments expect | Heaviest authoring cost; per-machine install conflicts with per-Citizen autostart and keystore assumptions; no differential updates | **Enterprise channel only**, built from the same artifacts, not maintained as a general path |

The decision is driven by the update mechanism, not by the install experience: a flagship that must ship signed differential updates on our own cadence cannot have its primary channel gated on Store review. The honest cost of choosing NSIS is that uninstall completeness is our responsibility, and it is covered by an uninstall-verification test in the Windows lab.

---

## 7. The mobile question, answered honestly

Five options, scored against the principles that actually discriminate between them. Scores are 1–5, higher is better; the P-weighted total uses the conflict-resolution order (`00 §2`) as weights — P2 ×3, P10 ×3, P13 ×2, maintenance ×3, velocity ×1.

| Option | P2 local-first | P10 perf/a11y | P13 one core | Maint. cost (5 = cheapest) | Velocity | **Weighted** |
|---|---|---|---|---|---|---|
| PWA only | 2 — evictable storage, no reliable background execution, iOS push requires Home Screen install | 3 — good on Android, compromised on iOS | 5 — literally the same front end | 5 | 5 | **35** |
| Tauri Mobile v2 | 4 — real embedded core, real local store | 3 — webview scroll/keyboard/gesture gaps are most visible on mobile; a11y through the webview is weaker than the native trees | 5 | 4 | 4 | **40** |
| React Native + Rust core | 4 | 4 — native scroll and lists, JS bridge overhead on the composer path | 4 — a third UI paradigm to keep in parity | 3 | 3 | **39** |
| Capacitor + web front end | 3 — WebView storage, better than a browser but not native durability | 2 — the weakest performance ceiling of the five | 5 | 4 | 4 | **34** |
| **Native SwiftUI/Compose + Rust core** | **5** — full filesystem, Keychain/Keystore, BGTask/WorkManager | **5** — the platform's own scroll, IME, gestures, VoiceOver/TalkBack trees | **4** — same core, two more UI trees | **2** | **2** | **43** |

**Decision: PWA through Phase 4, then native SwiftUI and Jetpack Compose over the shared Rust core (UniFFI) from Phase 5.**

The reasoning is P2 and P10, and it is not close. A social product's mobile experience is 80% scrolling a long virtualized list and typing into a composer while the keyboard animates — precisely the two things webview shells do worst and native does best. Meanwhile the thing native mobile normally costs you — a second implementation of your business logic — we do not pay, because the core is already a library that compiles to `aarch64-apple-ios` and `aarch64-linux-android` and has done so since Phase 0 (I3). We are paying for two UI trees, not two products.

**The staged path:**

```
 Phase 2 ──► PWA installable, push where the platform allows, offline read of
             every core surface. This is the mobile product through Phase 4.
 Phase 3 ──► UniFFI façade frozen and conformance-tested; a throwaway
             Compose harness exercises it on real hardware. No app ships.
 Phase 5 ──► Android first (faster iteration, no review latency), iOS one
             release behind. Both reach T2 production quality in-phase.
 Phase 6 ──► Widgets, Live Activities, App Intents/Shortcuts, handoff.
```

Android first is deliberate: it surfaces binding-layer defects without a review cycle between each discovery, and it de-risks the iOS build that follows.

**Reversal conditions, stated in advance so they are recognized rather than rationalized:**

1. **Headcount.** If, at the Phase 5 gate, fewer than two dedicated engineers per mobile platform are funded, ship **Tauri Mobile for Android only** and keep iOS on the PWA. Two half-staffed native apps are worse than one good webview shell.
2. **UniFFI conformance.** If the frozen façade cannot pass the conformance suite on both platforms by the end of Phase 3, or if binding overhead exceeds 4 ms on the Chamber page benchmark, re-evaluate against Tauri Mobile, which shares the identical core with no binding layer.
3. **Parity debt.** If the parity matrix (§9) shows mobile more than one release behind the desktop for two consecutive releases, freeze mobile feature work until it is level. A permanently trailing target is a fragmented architecture with a shipping schedule.

**What was rejected and why, beyond the scores.** Capacitor loses to Tauri Mobile on every axis we care about — it has no in-process Rust core, so the Node model degrades into a thin client, which is a P2 violation at the architecture level rather than a performance complaint. React Native scores respectably but introduces a third UI paradigm (React-for-web, React-for-native, and the two of them are not the same React) while still requiring platform specialists for the native modules; it buys less sharing than it appears to and costs a whole extra ecosystem.

---

## 8. Design system portability (N7)

One source. Five outputs. No hand-written duplicate of any token, on any platform, ever.

```
   design/tokens/*.json          (DTCG format — colour, type, space, motion,
   ┌──────────────────┐          radius, elevation, breakpoint, density)
   │  SINGLE SOURCE   │          reviewed by design, owned in-repo, versioned
   └────────┬─────────┘
            │  cargo run -p fractal-tokens          (a Rust generator; no
            ▼                                        Node.js in the token path)
   ┌────────────────────────────────────────────────────────────────────────┐
   │  RESOLVE  aliases → primitives → semantic → per-theme (Void, Daylight) │
   │  VALIDATE contrast ratios · closed status vocabulary · ≤3 accents/screen │
   └────────┬───────────────────────────────────────────────────────────────┘
            ├──────────────┬──────────────┬──────────────┬──────────────────┐
            ▼              ▼              ▼              ▼                  ▼
   ┌──────────────┐┌──────────────┐┌─────────────┐┌──────────────┐┌────────────────┐
   │ tokens.css   ││ tokens.rs    ││ Tokens.swift││ Tokens.kt    ││ tokens.ansi.rs │
   │ custom props ││ const structs││ enum + Color││ object + Color││ 24-bit + 256   │
   │ + container  ││ (core, CLI,  ││ (SwiftUI)   ││ (Compose)     ││ + 16 + mono    │
   │   queries    ││  render hints)│└─────────────┘└──────────────┘│  degradation   │
   └──────────────┘└──────────────┘                                └────────────────┘
            │              │              │              │                  │
            └──────────────┴──────────────┴──────────────┴──────────────────┘
                                          ▼
                        CI: regenerate → `git diff --exit-code`
                            drift is a build failure, not a review comment
```

**Source format.** DTCG JSON, because it is a written specification rather than a tool's private format, and because the generator must be replaceable (P5 applies to our own tooling). Three layers: primitives (`void`, `signal`, raw ramps), semantic (`text-primary`, `agent`, `pending` — the `33 §2.3` assignments), and component (`button-primary-bg`). Only semantic and component tokens may be referenced by product code; a primitive reference in a component is a lint failure.

**Build step.** `fractal-tokens` is a Rust crate with a binary and a `build.rs`. It runs in CI and locally via a pre-commit hook. It is Rust rather than Style Dictionary so that the token pipeline has no Node.js dependency and can run inside the same build that produces the CLI — the CLI must not require a JavaScript toolchain to be built.

**Outputs and their consumers.**

| Output | Consumed by | Notes |
|---|---|---|
| `tokens.css` | Web, PWA, desktop (one file, three shells) | Custom properties plus the six named container-query breakpoints (§6.2) and both themes |
| `tokens.rs` | Core, CLI, any Rust-side render hint | `const` structs; no runtime parsing |
| `Tokens.swift` | iOS, macOS native pieces | SwiftUI `Color`/`Font` extensions, Dynamic Type mappings |
| `Tokens.kt` | Android | Compose `ColorScheme`, `Typography`, `Shapes` |
| `tokens.ansi.rs` | CLI Terminal | Four tiers: 24-bit truecolor, 256-colour, 16-colour, and monochrome-with-symbols for `NO_COLOR` and screen-reader mode |

**The CI checks that make drift impossible.**

1. **Regeneration check** — regenerate all five outputs and fail on any diff (I4).
2. **Literal ban** — stylelint denies hex/rgb literals outside the generated file (`33 §10`); `clippy` lint denies literal colour constants in Rust; a Swift/Kotlin grep rule does the same.
3. **Contrast gate** — every semantic text/background pairing is computed and asserted against WCAG 2.2 AA (AAA for default-theme body text) in both themes and all four CLI tiers. A token change that breaks contrast fails the build with the exact pairing named.
4. **Terminal degradation test** — the ANSI palette is rendered in all four tiers and snapshot-compared; a colour that becomes indistinguishable from its neighbour at 16 colours fails.
5. **Semantic-drift test** — the closed status vocabulary and the three-accent-per-screen rule are asserted against the component catalogue.

---

## 9. Feature parity enforcement (P13)

Parity is not a review culture. It is generated, measured, and it blocks tags.

**The matrix is generated, never maintained by hand.** The schema registry (`10 §10`) is the source of truth for every API operation. A generator walks it and emits a row per operation and a column per Front End, filling each cell from evidence:

- **API** — the operation exists in the registry with a passing contract test.
- **CLI** — a `fn` command is registered against that operation ID and its help text and `--json` output are snapshot-tested.
- **GUI (web/desktop)** — a component in the catalogue declares `operationId` in its manifest and has a passing route test.
- **Mobile** — the same declaration in the Compose/SwiftUI screen registry.

```
  ┌────────────────────────────────────┬─────┬─────┬─────┬─────┬──────┬──────┐
  │ operation                          │ API │ CLI │ Web │ Desk│ And. │ iOS  │
  ├────────────────────────────────────┼─────┼─────┼─────┼─────┼──────┼──────┤
  │ society.create                     │  ██ │  ██ │  ██ │  ██ │  ██  │  ██  │
  │ society.fracture                   │  ██ │  ██ │  ██ │  ██ │  ░░  │  ░░  │  ← blocks
  │ wallet.transfer                    │  ██ │  ██ │  ██ │  ██ │  ██  │  ██  │
  │ chamber.message.post               │  ██ │  ██ │  ██ │  ██ │  ██  │  ██  │
  │ node.custodian.attest              │  ██ │  ██ │  N/A│  ██ │  N/A │  N/A │  ← declared
  │ desktop.window.tearout             │ N/A │ N/A │  ░░ │  ██ │  N/A │  N/A │  ← ADR-0031
  └────────────────────────────────────┴─────┴─────┴─────┴─────┴──────┴──────┘
      ██ present + tested    ░░ absent    N/A declared not-applicable
```

**The release gate.** `parity-gate` runs on every release candidate and fails the tag when any operation is present in a GUI and absent from the CLI, or absent from the API. The P13 falsification test is executable: `fn parity report --tag vX.Y.Z --fail-on missing`. A GUI-only feature cannot reach a tag. This is the single most load-bearing gate in the engineering process, because it is the one that structurally prevents the CLI from decaying into a second-class surface (N3).

**Declaring "not applicable" honestly.** `N/A` is an escape hatch and is therefore constrained:

1. It must be declared in `parity/exemptions.toml` with the operation ID, the target, a one-sentence reason naming the *platform mechanism* that makes it inapplicable, an ADR reference, and a reviewer.
2. Reasons are drawn from a **closed list**: `no-such-hardware`, `no-such-os-primitive`, `platform-policy-prohibits`, `surface-has-no-analogue`, `security-posture-forbids`. "Not built yet" and "no time" are not on the list — those are `░░`, and `░░` blocks.
3. Every exemption carries an expiry date. An expired exemption fails the gate. This prevents a temporary gap from becoming a permanent architectural fork by simply aging.
4. Exemptions per target are capped at 5% of operations. Exceeding the cap is a phase-gate failure, not a warning, because it means the target has quietly become a different product.
5. A platform-only *feature* (as opposed to a platform-only rendering of a shared feature) requires its own ADR stating the mechanism it depends on and the declared degradation on every other target (I6). `desktop.window.tearout` above is the model: it is a shell capability with no domain semantics, so it has no API row and no domain event.

---

## 10. The parallel roadmap

### 10.1 How a target counts against the complexity budget

`02 §5` caps new client platforms at one per phase. That cap is about *architecture*, not about packaging, and the distinction has to be written down before it is abused in either direction.

> **A target counts against the budget when it introduces a new UI toolkit or a new binding surface. It does not count when it is a new build target of an existing shell.**

So: the desktop shell counts once (Phase 2) and macOS and Linux add zero platform slots in Phase 5, because they are the same Tauri shell, the same React front end, and the same core. Native mobile counts once (Phase 5) and covers both Android and iOS, because they share one binding surface (UniFFI), one token pipeline, one parity column group, and one design specification.

**The honest cost that this accounting hides, and how it is paid:** packaging targets still cost signing infrastructure, notarization, store listings, a device lab, and a permanent support queue. Those are budgeted in engineer-weeks in the phase plan below and reviewed at the gate — they are simply not counted as *architecture*. Phase 5 is the heaviest phase in this document and the plan says so rather than pretending three targets are free.

### 10.2 Phase-by-phase target plan

| Phase | Web | Desktop (Win) | PWA | CLI | Android | iOS | macOS | Linux | Dependencies |
|---|---|---|---|---|---|---|---|---|---|
| **0** | Shell + token pipeline | — | — | Skeleton + Terminal boot | — | — | — | — | I3 build matrix green on all six triples; `fractal-tokens` emitting five outputs |
| **1** | **T1 production**: Societies, Chambers, Vault, Wallet, Charter | — | — | **T1 production**: full parity with web | — | — | — | — | Public API + schema registry frozen for v1; parity gate live |
| **2** | Hardening, a11y audit | **T1 production**: embedded core, real Node, flagship layout ladder | — | Terminal dashboards | — | — | — | — | Local store + sync engine on native; Tauri updater + EV signing |
| **3** | Realtime hardening | Multi-window, tray, protocol handlers | **T2 production**: installable, offline, push where allowed | Loopback attach to desktop Node | — | — | — | — | Service Worker; UniFFI façade **frozen** and conformance-tested |
| **4** | Voice/video (`14`) | Voice/video, screen share, GPU pass | Voice where the platform allows | Voice status/control only | — | — | — | — | SFU; media capture ports per shell |
| **5** | Maintenance | Handheld/controller polish; MSIX channel | Maintenance | Fracture, governance verbs | **T2 production** | **T2 production** (one release behind) | **T2 production** | **T3 supported** | UniFFI conformance green; notarization + Play/App Store pipelines; device lab funded |
| **6** | Federation, marketplace | Self-hosted Node management UI | Marketplace | Node operator commands | Widgets, App Intents | Widgets, Live Activities | Menu bar Node status | Headless + GUI on one host | Multi-node self-hosting (`10 §9`) |
| **7** | Experience Runtime host | Experience Runtime host (flagship) | Read-only Experiences | Experience lifecycle verbs | Experience host | Experience host | inherits | inherits | Sandbox adversarially tested for two phases (`02 §3`) |

### 10.3 Gantt

```
                  P0      P1      P2      P3      P4      P5      P6      P7
                  ├───────┼───────┼───────┼───────┼───────┼───────┼───────┼──►
 Core (Rust)      ████████████████████████████████████████████████████████████
 Tokens (N7)      ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
 Build matrix     ████████████████████████████████████████████████████████████
 ─────────────────────────────────────────────────────────────────────────────
 Web              ▓▓▓▓▓▓▓▓████████░░░░░░░░▒▒▒▒▒▒▒▒████████░░░░░░░░████████████
 CLI              ▓▓▓▓▓▓▓▓████████▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░░░████████░░░░░░░░████
 Desktop (Win)            ▓▓▓▓▓▓▓▓████████████████████████████████░░░░░░░░████
 PWA                              ▓▓▓▓▓▓▓▓████████░░░░░░░░░░░░░░░░████░░░░░░░░
 UniFFI façade                    ▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒████████░░░░░░░░░░░░░░░░░░░░
 Android                                          ▓▓▓▓▓▓▓▓████████████████████
 iOS                                              ░▓▓▓▓▓▓▓████████████████████
 macOS                                            ▓▓▓▓████████░░░░████████████
 Linux                                            ▓▓▓▓▓▓▓▓░░░░░░░░████░░░░░░░░
 ─────────────────────────────────────────────────────────────────────────────
 Device lab                                       ████████████████████████████
 Parity gate      ░░░░░░░░████████████████████████████████████████████████████

 ▓ build-up   █ active development / production quality reached   ▒ hardening
 ░ maintenance and parity upkeep
```

### 10.4 "Production quality" per target

The gate is not "it runs". Each target reaches production quality when all of the following are true, and the target's phase does not close until they are.

| Target | Definition of production quality |
|---|---|
| Web | All §11 budgets met at p75 on mid-tier hardware; axe-core clean on every route and dialog; manual screen-reader audit passed; offline suite passes; parity gate green; three browsers in the T1 matrix |
| CLI | 100% of API operations reachable; `--json` on every command; completions for four shells; screen-reader mode audited; startup budget met; signed binaries on four channels |
| Desktop (Win) | Web criteria plus: embedded Node passes the same conformance suite as a headless Node; multi-window and multi-monitor tests pass; installer + uninstaller verification; signed differential update proven from the previous release; handheld and controller test pass on Deck-class hardware |
| PWA | Installs and launches offline on Android and iOS; push verified where the platform allows; storage-persistence request handled with an honest fallback message when denied |
| Android / iOS | Store-accepted; §11 budgets on the reference device; TalkBack/VoiceOver audit passed; background sync verified over a 24-hour soak; push decryption verified without any plaintext leaving the device (N6); parity gate green with ≤5% declared exemptions |
| macOS | Desktop criteria; notarized; menu bar and Notification Center integration audited |
| Linux | Desktop criteria on Ubuntu LTS and Fedora current under Flatpak; degraded-tray and no-Secret-Service paths verified; published, honest statement of the weaker a11y position |

---

## 11. Performance budgets per platform

Budgets live in `perf/budgets.json`, are enforced by one CI job per target, and are quoted at **p75 on the reference device** unless stated. A regression fails the build; an exception requires an approved, expiring waiver (P10).

**`perf/budgets.json` is generated from `32 §8`, which is the source** — this chapter previously had the direction of authority backwards (`61 X9`). This table owns the **per-target extensions** `32 §8` does not state: Android, iOS, Linux, macOS, PWA, battery, binary and installer size, IPC/FFI cost, and the measurement mechanism for every row. It may never restate a `32 §8` row with a different value, and the four rows above that used to are corrected. Three of the eight reported conflicts were never conflicts — CLI binary size and CLI memory are two measurements sharing a label, and both figures are now stated; `34`'s 220 KB was the *total initial payload* under a JS label.

| Target | Cold → interactive | Warm start | Interaction → paint | Memory ceiling | Binary / bundle | Frame budget | Battery |
|---|---|---|---|---|---|---|---|
| Windows desktop | ≤1.5 s | **≤400 ms** | ≤100 ms | ≤450 MB RSS (5 Societies, 10 k cached Messages) | ≤25 MB installer, ≤40 MB on disk excl. local store | 16.7 ms; 8.3 ms on ≥120 Hz | ≤3%/h active on handheld; ≤0.4%/h idle-in-tray |
| macOS | ≤1.65 s | **≤400 ms** | ≤100 ms | ≤450 MB | ≤30 MB dmg | same | ≤3%/h |
| Linux | ≤1.9 s | ≤750 ms | ≤120 ms | ≤500 MB | ≤35 MB Flatpak delta | same | not measured (declared) |
| Web | FCP ≤1.2 s, TTI ≤2.5 s | **≤800 ms** (warm cache) | ≤100 ms | ≤350 MB | **≤180 KB gz initial JS; ≤400 KB gz total initial payload**; core wasm ≤1.8 MB gz, lazy | 16.7 ms | n/a |
| PWA | Offline shell ≤900 ms | ≤700 ms | ≤100 ms | ≤350 MB | inherits | 16.7 ms | ≤4%/h active |
| Android | ≤1.8 s (Pixel 6a class) | ≤400 ms | ≤80 ms | ≤280 MB | ≤32 MB per-ABI AAB | jank <0.5% of frames | ≤2%/h active, ≤0.5%/day idle with push |
| iOS | ≤1.4 s (iPhone 12 class) | ≤350 ms | ≤80 ms | ≤250 MB | ≤40 MB download | hitch rate <5 ms/s | ≤1.8%/h active |
| CLI | ≤120 ms to first byte | n/a | ≤50 ms streaming | ≤80 MB RSS (1 Society); ≤120 MB (4 Societies) | ≤12 MB compressed artifact; ≤18 MB on disk | n/a | n/a |

**How each is measured in CI.**

| Metric | Mechanism |
|---|---|
| Desktop cold/warm start | The shell emits `startup.interactive` as a core telemetry span; a headless harness launches the built artifact 20× on the Windows/macOS/Linux lab runners and asserts p75 |
| Web FCP/TTI/bundle | Lighthouse CI on the built artifact against a throttled mid-tier profile; `size-limit` on the route manifest; wasm size asserted separately so a core growth cannot hide inside a JS budget |
| Interaction → paint | Playwright traces on a fixed script (open Society → open Chamber → post Message → open Wallet); Long Animation Frame entries asserted |
| Frame budget / jank | Chamber scroll benchmark, 10 s, 5 000-Message projection; desktop via WebView2 tracing, Android via Macrobenchmark + Baseline Profiles, iOS via XCTest `XCTOSSignpostMetric` hitch metrics |
| Memory | RSS sampled at 60 s of the standard soak on each target; a leak test asserts flat RSS across 200 navigation cycles |
| Binary size | `cargo-bloat` and artifact size gates per triple, with the per-crate delta printed on every PR |
| IPC/FFI cost | Bytes and calls per interaction asserted against §3.3 |
| Battery | Weekly, not per-commit: Android Battery Historian on a fixed 1-hour script; iOS MetricKit aggregates from the internal build; handheld Windows measured on Deck-class hardware with a scripted session |
| Accessibility | axe-core per route and dialog (web/desktop), Accessibility Scanner + Espresso (Android), XCTest audits (iOS), a linear-output snapshot for the CLI screen-reader mode |

Budgets are per-target because pretending one number covers a 5120 px workstation and a mid-tier Android device is how budgets get ignored. What is *not* per-target is the accessibility floor: WCAG 2.2 AA is identical everywhere (N8), and the CLI meets it through `--plain`, `NO_COLOR`, and screen-reader mode rather than through an exemption.

---

## 12. Offline behaviour per target (P2)

### 12.1 What works with no network

Every row below is verified by the airplane-mode suite (§13). "Read" means last-known-good with an explicit staleness indicator (`33 §2.3` Ember), never an error state.

| Capability | Win / mac / Linux | Android / iOS | PWA | Web (tab) | CLI |
|---|---|---|---|---|---|
| Read Societies, Chambers, Threads, Messages | Full | Full | Full to quota | Full to quota, evictable | Full |
| Post a Message | Queued, optimistic, ordered on reconnect | Queued | Queued | Queued (lost if the tab's storage is evicted) | Queued |
| Read Wallet balance | Full, marked stale | Full, marked stale | Full, marked stale | Full, marked stale | Full, marked stale |
| Transfer Fraction | **Refused offline** — server-authoritative (`10 §6`) | Refused | Refused | Refused | Refused |
| Read Vault objects | Cached + pinned objects | Cached + pinned | Cached to quota | Cached to quota | Cached |
| Write a Vault object | Queued with local Shards | Queued | Queued, size-capped | Queued, size-capped | Queued |
| Read Charter, roles, Standing | Full | Full | Full | Full | Full |
| Cast a governance vote | **Refused** — signed, server-authoritative | Refused | Refused | Refused | Refused |
| Draft anything | Full | Full | Full | Full | Full |
| Search local replica | Full (local index) | Full | Full | Full | Full |
| Invoke an Agent | Refused unless a local `ModelProvider` is configured | Refused | Refused | Refused | Refused unless local |

**The refusals are principled, not technical.** Money, ownership, and governance are server-authoritative by decree (`10 §6`, P12). An offline transfer that later fails is worse than a transfer that was honestly refused, and a queued vote is a vote whose outcome the Citizen cannot verify. The product states the reason inline, in the error register from `33 §7.3`: cause, then remedy, no apology.

### 12.2 Storage reality, stated without optimism

| Target | Mechanism | Practical ceiling | Eviction risk |
|---|---|---|---|
| Windows / macOS / Linux | SQLite (WAL) + content-addressed Shard files | Disk | None. The Citizen deletes it deliberately. |
| Android | SQLite + app-private files | Disk; subject to "manage storage" clearing | Low; the OS may reclaim on extreme pressure |
| iOS | SQLite + app container, `isExcludedFromBackup` on Shards | Disk | Low; "Offload App" removes the binary, keeps data |
| PWA (Android/Chromium) | OPFS + IndexedDB, `persist()` granted for installed apps | Commonly ~60% of free disk | Low once persisted; **not zero** |
| PWA (iOS) | OPFS + IndexedDB | Low single-digit GB in practice | **Real.** Storage for script-writable origins is cleared after extended non-use |
| Web (uninstalled tab) | OPFS + IndexedDB, best-effort quota | Quota-dependent, evictable | **High.** Treated as a cache, not a replica |

**The consequence is a product decision, not a footnote.** The web target displays a durability state — `REPLICA` when persistence was granted, `CACHE` when it was not — and offers the desktop Node as the path to the former. This is the honest version of local-first on the web, and it is exactly why the flagship is the desktop: **only a Node whose storage the platform cannot evict is genuinely local-first (P2).** No amount of engineering makes a browser tab a Node, and claiming otherwise would be the kind of marketing this document exists to prevent.

### 12.3 Where the sync engine differs by force

The engine is one state machine; only its *driver* and its *budget* differ.

| Target | Driver | Budget per step | Forced difference |
|---|---|---|---|
| Desktop | Timer + Signal socket, always connected when running | Unbounded | None. The reference implementation. |
| Android | WorkManager (deferred), foreground service only during an explicit long operation | 10 s per step | Doze defers steps; FCM high-priority data messages wake for `Urgent` classes only (`14 §11`) |
| iOS | BGAppRefreshTask + BGProcessingTask, opportunistic | ~25 s per step, scheduler-dependent | The OS may not run the task for hours. Push carries enough to render a notification without a step. |
| PWA | Background Sync where present; otherwise on-focus | Best-effort | On iOS, effectively foreground-only |
| Web | Foreground only, in the Worker | Foreground | Bulk catch-up is chunked so a long-absent tab does not block the first paint |
| CLI | Explicit (`fn sync-step` is a command) or daemon mode | Unbounded | None |

The rule that keeps this from fragmenting: **no target may add a conflict-resolution rule.** If a platform's constraint changes what happens on reconnect, it changes the *schedule*, never the *semantics*. A platform-specific merge rule would be a domain rule outside the core, which is an I1 violation.

---

## 13. Testing strategy across platforms

**The conformance suite is the centre of gravity.** One suite, written in Rust against the core façade, runs identically in-process on every target: native on desktop/mobile/CLI, and in the browser via `wasm-bindgen-test` in real Chromium, Firefox, and WebKit. It covers command handling, permission denial, offline queueing, conflict resolution, migration, and event replay. If a defect can be caught here, it must be caught here — every one of these tests is one test rather than eight.

**Above the core, four layers:**

| Layer | Scope | Where it runs |
|---|---|---|
| Binding conformance | Every façade method through wasm-bindgen, UniFFI-Kotlin, UniFFI-Swift, and direct linkage; identical fixtures, identical assertions | CI, every commit |
| Component | The React catalogue and the Compose/SwiftUI screen registries against a mocked core | CI, every commit |
| Journey | Playwright (web, PWA, desktop via WebDriver), Espresso (Android), XCUITest (iOS), `expect`-driven PTY tests (CLI). One fixture set, one Society seed, expressed once and consumed by all four | CI + nightly hardware |
| Visual regression | Deterministic screenshots at the six named breakpoints, both themes, plus reduced-motion and high-contrast variants | Nightly; PR-triggered on token or component changes |

**Device matrix.**

| Class | Devices | Emulator or real |
|---|---|---|
| Windows T1 | Win 11 desktop (discrete GPU, 3440×1440), Win 11 laptop (integrated, 1920×1200, 150% scale), Deck-class handheld (1280×800, controller), Win 10 22H2 minimum-spec | Real, in a self-hosted lab |
| Web T1 | Chromium, Firefox, Safari — current and current−1, plus Firefox ESR | Containers + one real macOS runner for Safari |
| Android T2 | Pixel (current), Pixel 6a (reference/mid-tier), Samsung mid-range (One UI divergence), API-30 minimum device | Real for the four; emulators for the API sweep |
| iOS T2 | iPhone 12 (reference), iPhone current, iPad (keyboard + pointer), oldest supported | Real; simulators for the OS-version sweep |
| macOS T2 | Apple silicon current, Intel oldest supported | Real |
| Linux T3 | Ubuntu LTS, Fedora current | VMs; one real machine for GPU and Wayland |

**Emulators versus real hardware.** Emulators are for breadth (OS-version sweeps, layout, a11y trees); real hardware is for anything with a number attached — startup, jank, memory, battery, GPU. **No performance budget may be signed off on an emulator.** Screenshot tests run in containers with pinned fonts and forced GPU settings; a screenshot suite that is not deterministic is a suite that gets disabled within a month.

**Accessibility automation** is per-target (§11) and catches roughly 40% of real defects — a number worth stating, because the other 60% is why every phase gate includes a manual audit with NVDA, VoiceOver, TalkBack, and Orca, plus a keyboard-only and a controller-only pass on the flagship.

**The honest cost of the lab.** Roughly USD 18–25 k of capital for devices and self-hosted runners, plus ~USD 1.5–2.5 k/month for macOS runners and cloud device time, plus **0.5 engineer permanently** on lab maintenance — flaky runners, expiring certificates, OS updates that break automation. Nightly full-matrix runs consume real wall-clock time, so the PR path runs core + bindings + component + one journey per T1 target, and the full matrix runs nightly and on every release candidate. Pretending a cross-platform lab is free is the most common way cross-platform quality dies quietly.

---

## 14. Anti-fragmentation rules

These are hard rules. Each has an owner in CI, because a rule without an enforcer is a preference.

| # | Rule | Enforcement |
|---|---|---|
| F1 | **No business logic in a shell.** No validation, no permission decision, no balance arithmetic, no conflict resolution, no XP or Trust computation outside Rust. | Review checklist + forbidden-identifier lint in shell code (I1) |
| F2 | **No platform-specific API calls.** Shells call the generated façade or the documented public API. No shell may construct an HTTP path by hand. | Grep against the schema registry (I2, P3) |
| F3 | **No platform-only feature without an ADR** naming the platform mechanism and the degradation elsewhere. | Parity gate exemption rules (§9, I6) |
| F4 | **One design system.** No colour, spacing, radius, duration, or breakpoint literal outside the generated token files. | Stylelint, clippy, and grep rules per language (I4, N7) |
| F5 | **One core version per release.** Every shell in a tag links the identical core commit. | Release manifest check (I5) |
| F6 | **No divergent conflict resolution.** Platforms may change *when* sync runs, never *what* it decides. | Conformance suite runs the identical merge fixtures on every binding (§12.3) |
| F7 | **No shell-owned persistence.** Shells hold view state only. Anything durable goes through `fractal-store`. | A dependency lint denying platform storage APIs in shell code, with a narrow allowlist for window geometry and the token cache |
| F8 | **No fork of a shared component to fix one platform.** Fix it in the shared component behind a capability flag derived from a declared platform capability, or lift the difference into a port trait. | Review; duplicated component names fail a catalogue uniqueness check |
| F9 | **New capabilities land in the API and the CLI first.** GUIs follow. | Parity gate ordering; the CLI column is the leading indicator |
| F10 | **Every target's degradation is declared, not discovered.** A target that cannot do something says so in `parity/exemptions.toml` with an expiry. | §9, cap of 5% |

**Why F9 matters more than it looks.** Building the CLI first for every capability forces the API to be complete and coherent before any GUI can paper over a gap with a bespoke call. It is the cheapest known enforcement of P3, and it is the reason the CLI stays a peer rather than becoming an afterthought (N3).

---

## 15. Alternatives rejected

Scored 1–5 against the axes that discriminate. Weighted by P2 ×3, P10 ×3, P13 ×2, maintenance ×3, ecosystem/hiring ×1.

| Approach | P2 (real Node) | P10 (perf/a11y) | P13 (one core) | Maint. | Ecosystem | **Weighted** | Verdict |
|---|---|---|---|---|---|---|---|
| **Tauri v2 + React + Rust core** (chosen) | 5 | 4 | 5 | 4 | 4 | **50** | Adopted |
| Electron + React + Rust sidecar | 4 | 2 | 3 | 3 | 5 | **38** | Rejected: 150 MB+ per install, worse cold start against the P10 budget, and the Rust core becomes an out-of-process sidecar with an IPC boundary on every call rather than in-process linkage — which weakens exactly the property that makes the desktop a Node |
| Flutter everywhere | 3 | 4 | 3 | 3 | 3 | **39** | Rejected: a third rendering model that owns its own text stack and accessibility tree; the design system would need a fourth token target and a parallel component library; the Rust core would still need FFI, so the sharing gain over the chosen path is small while the ecosystem cost is a whole new language and toolchain |
| .NET MAUI | 3 | 3 | 3 | 3 | 3 | **36** | Rejected: strongest exactly where we are already strongest (Windows) and weakest everywhere else; introduces a second managed runtime alongside Rust; Linux is not a first-class target |
| Qt (C++/QML) | 5 | 4 | 4 | 2 | 2 | **41** | Rejected: excellent technical fit for a dense desktop instrument and genuinely the closest runner-up, but licensing cost, a small and shrinking hiring pool, C++ memory-safety exposure adjacent to the crypto and ledger paths (P8), and no path to the web — which would force a second web front end and break P13 |
| Fully native everywhere (WinUI + AppKit + GTK + SwiftUI + Compose) | 5 | 5 | 3 | 1 | 3 | **41** | Rejected on maintenance alone: five UI implementations for a team this size means five perpetually unequal products and a parity gate that fails permanently. Best-in-class experiences are worthless if three of the five are always a release behind |
| Web-only (PWA everywhere, no native shell) | 1 | 3 | 5 | 5 | 5 | **41** | Rejected: browser storage is evictable (§12.2), so the flagship could never be a Node. This is a P2 violation at the architectural level — and P2 outranks both P13 and P10 in the conflict order (`00 §2`) |
| React Native everywhere incl. desktop | 3 | 3 | 4 | 3 | 4 | **40** | Rejected: desktop support is community-maintained, the ultrawide and multi-window flagship requirements are unserved, and it introduces a second React that is not the first React |

The pattern in the losing column is consistent: every rejected option either weakens the Node property (Electron sidecar, web-only, Capacitor) or multiplies UI implementations (fully native everywhere, Flutter, MAUI). The chosen architecture is the one that keeps a single in-process Rust core on every target while paying for exactly two UI implementations — one web-technology tree serving three desktop targets plus web and PWA, and one native mobile tree — and it is honest that the second one is expensive.

---

## 16. What would make us change this

Stated in advance, so the signal is recognized rather than argued away.

1. **WebView2 or WKWebView regressions break the P10 budget on the flagship for two consecutive releases.** Evaluate a Rust-native render path (Dioxus/Freya) for the desktop shell only, keeping the identical token source and the identical core (`10 §12`). The React web front end would remain; the cost is a second component tree for desktop, which is precisely the cost we are currently avoiding.
2. **UniFFI overhead or ergonomics fail the Phase 3 conformance gate.** Fall back to Tauri Mobile per §7 reversal condition 2.
3. **Mobile headcount is not funded at the Phase 5 gate.** §7 reversal condition 1: Tauri Mobile on Android, PWA on iOS, stated publicly rather than quietly.
4. **Browsers grant durable, non-evictable storage to installed PWAs across all three engines.** Re-score the web target's P2 line, and re-open whether the PWA can be a genuine Node on Android. It still would not be one on iOS.
5. **The shared-percentage report falls below its tolerance (§2.3) for two quarters.** That is drift, not a preference. The response is to lift the divergence into the core or into a port trait — not to lower the tolerance.
6. **The parity exemption cap is hit on any target.** The target has become a different product. Either fund it to parity or demote its tier honestly (§5).
