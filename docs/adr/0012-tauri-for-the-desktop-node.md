# ADR-0012 — Tauri v2 for the desktop shell, which embeds the Runtime and *is* a Node

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 2

## 1. Context

P2 states that the user's node holds an authoritative local replica of their own data and that the network is a synchronization medium, not the source of truth. `01 §2` defines a **Node** as a running Fractal Node instance that stores replicas and serves peers. `10 §9` draws the consequence: the desktop app *is* a Node with a GUI, which is why the desktop shell embeds the Runtime rather than being a thin client, and `34 §2.1` states it as the structural rule — the difference between the flagship desktop application and a data-centre Node is which shell is linked, not which logic is compiled.

That requirement, not bundle size, is what decides this. A desktop shell that talks over the wire to a hosted Runtime cannot hold an authoritative replica; it is a browser with a window frame. **The choice is therefore constrained to shells that can link a Rust library in-process.**

The second constraint is P13 and cost. `34 §2.3` budgets Windows desktop at 62% core plus 26% presentation shared with web, for 88% shared source. Anything that forces a second component tree for desktop costs that 26% permanently.

## 2. Decision

The desktop shell is **Tauri v2**: the system webview (WebView2 on Windows, WebKitGTK on Linux, WKWebView on macOS) rendering the same React + TypeScript + Vite front end the web target ships, with `fractal-node` linked **in-process** and reached over Tauri IPC rather than over the network. Windows is the flagship and ships in Phase 2; macOS and Linux follow in Phase 5 on the same shell, where the cost is signing, notarization and packaging rather than code.

Distribution is an NSIS per-Citizen installer as primary, WinGet, MSIX to the Microsoft Store from Phase 5, and MSI/WiX for managed enterprise only. Updates use the Tauri updater against a signed versioned manifest with differential payloads and staged rollout at 1%/10%/50%/100%, halting automatically on a crash-rate regression, and **the Citizen may defer but is never forced mid-session**.

## 3. Consequences

### Positive
- The flagship is a genuine Node: full local replica, real filesystem, OS keychain, background sync while minimized, and every read path answerable offline (P2, `34 §12.1`).
- ~12 MB shipped shell against Electron's 150 MB+, and a cold-start budget of 1.5s that is achievable rather than aspirational.
- One React codebase across web, PWA and three desktop targets, which is what makes desktop cheap rather than a second product.
- No new language in the build: Rust is already required for the core.
- Deep OS integration is available — toast notifications with reply, tray with live status, jump list, `fractal://` protocol handler, `.fnbundle` and `.fnfacet` file associations, Windows Hello for wallet confirmations via the WebAuthn platform authenticator.

### Negative
- **WebView2 version skew across machines, and Microsoft's Evergreen runtime updates under us.** It has broken layout before and it will again. `34 §4.1` budgets ~1 engineer-week per quarter for regression triage; that is a permanent tax, not a ramp-up.
- No control over the compositor, and GPU rasterization quirks on older Intel drivers.
- Debugging spans two runtimes and two toolchains, which slows every desktop-specific defect.
- Webview inconsistency across three OSes means the Phase 5 macOS and Linux targets will surface rendering differences the shared codebase cannot fully hide.

### Neutral / follow-on work
`34 §6` specifies the responsive ladder from 800px handheld to 5120px panorama on this shell, including the invariant that surplus width buys panes, never longer lines. Gamepad input via `gilrs` and multi-monitor window management are shell-layer concerns and do not touch the core.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **Electron + React + a Rust sidecar** | The largest desktop ecosystem, most predictable rendering, and the deepest tooling; the front end would be identical | The Rust core becomes an out-of-process sidecar with an IPC boundary on every call rather than in-process linkage — which weakens exactly the property that makes the desktop a Node. Plus 150 MB+ per install and a cold start that does not fit the P10 budget. `34 §15` scores it 38 against Tauri's 50 |
| **Fully native per OS (WinUI + AppKit + GTK)** | Best possible platform fidelity, best accessibility trees, no webview to fight | Rejected on maintenance alone: three more UI implementations for a team this size means three perpetually unequal products and a parity gate (`34 §9`) that fails permanently. Best-in-class experiences are worthless if two of the three are always a release behind |
| **Qt (C++/QML)** | The closest runner-up: excellent for a dense desktop instrument, real in-process core linkage, mature ultrawide and multi-window support | Licensing cost, a small and shrinking hiring pool, C++ memory-safety exposure adjacent to the crypto and ledger paths (P8), and no path to the web — which would force a second web front end and break P13 |
| **Web-only, PWA everywhere, no native shell** | Cheapest of all; one artifact; instant updates; no signing or store review | Browser storage is evictable and quota-limited (`34 §12.2`), so the flagship could never be a Node. That is a P2 violation at the architectural level, and P2 outranks both P13 and P10 in the `00 §2` conflict order |

## 5. Exit Cost

**8–12 engineer-weeks to replace the shell, keeping the React front end and the core unchanged.** The work: reimplement the shell layer — window and lifecycle management, tray, notifications, protocol and file-association registration, updater, gamepad, multi-monitor — against the new host (~5 weeks); re-do installer, signing and store channels for three OSes (~3 weeks); re-run the per-target accessibility and performance budget suites (~2 weeks). The number is bounded precisely because `34 §2.2` keeps OS integration behind `fractal-shell-ports` traits: shells implement, the core requests. A move to a Rust-native render path (Dioxus/Freya) is materially more expensive because it also replaces the 26% shared presentation layer, and that is the cost `10 §12` names when it lists this as a possible response to webview regressions.

## 6. Principle Served

**P2** (the shell embeds the Runtime, so the desktop app is an authoritative replica), **P10** (~12 MB, 1.5s cold start, native webview compositing), **P13** and **N2** (one core, one React tree, five targets). Traded: rendering determinism, since a system webview is not a controlled one — accepted because P2 outranks P10 in the `00 §2` order and no alternative offered both.

## 7. Falsification Test

1. **The Node property**: with the network disabled, the desktop app must render the society timeline, chat history, wallet balance, media gallery and profile from the local replica, with a staleness indicator, and must queue writes to the outbox. Any surface rendering an error instead is a P2 violation. Run per release as the airplane-mode suite (`34 §13`).
2. **In-process, not remote**: a test asserts the desktop build makes zero HTTP requests to a Runtime endpoint for any read served by the local replica, and that `fractal-node` is linked into the shell process rather than spawned as a sidecar.
3. **One core per release** (`34 §I5`): the release manifest records one core SHA; a shell pinning a different SHA blocks the tag.
4. **Budgets**: cold ≤1.5s, warm ≤600ms, RSS ≤450 MB, 60/120fps, enforced per release in CI (`34 §11`).
5. **Shared-source floor**: the quarterly `tokei` report must keep Windows desktop at ≥82% shared (`34 §2.3`); two consecutive quarters below is drift, and the response is to lift divergence into the core, not to lower the tolerance.

## 8. Maintenance Horizon

Tauri is a community project with a foundation and multiple corporate contributors; v2 is the current major and the plugin ecosystem is active. It is not single-maintainer, but it is smaller than Electron and that is the honest risk. The mitigation is structural rather than contractual: the shell is roughly 12% of desktop source (`34 §2.3`), all OS integration sits behind `fractal-shell-ports`, and the React front end and Rust core are both independent of Tauri — so a shell replacement is the 8–12 weeks costed above rather than a rewrite. WebView2 itself is maintained by Microsoft and is the component we control least.

## 9. Review Trigger

Reopen if (a) WebView2 or WKWebView regressions break the P10 budget on the flagship for two consecutive releases — `10 §12` and `34 §16` pre-commit the response: evaluate a Rust-native render path for the desktop shell only, keeping the identical token source and core; (b) Tauri v2 maintenance stalls such that a security fix takes more than 30 days; or (c) the shared-source report for desktop falls below its 82% tolerance for two quarters.
