# ADR-0011 — The WASM Component Model as the extension execution boundary

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 3

## 1. Context

P7 requires first-party features to be built through the same extension surfaces third parties get. P8 requires deny-by-default with every capability explicit, scoped, time-boxed and revocable. `02 §3` places third-party paid Extensions at Phase 6 and the Experience Runtime at Phase 7, and forbids executing untrusted third-party code until the Envelope system has been adversarially tested for two phases.

The execution boundary must therefore answer one question mechanically: **what can this code reach?** Every host offers two ways to answer it. Either you enumerate what a guest may not do and audit its behaviour, or the guest's imports are the complete, typed statement of its authority and there is nothing to audit. `20 §4` requires the second: an import the Envelope does not grant is **not stubbed — it is absent from the instance**, so a guest compiled against `clock` and installed without `system.clock` fails instantiation with a typed error rather than misbehaving at 3am.

Two further constraints narrow the field. Hook latency has an 8ms budget (`20 §11`), which rules out per-invocation process or container startup. And hook execution must be **replayable** from the event log for debugging and for marketplace dispute resolution (`19`), which requires no ambient time, no ambient entropy, no ambient I/O and no shared memory.

## 2. Decision

Extensions execute as **WASM components under the Component Model**, hosted by Wasmtime in `fractal-adapter-wasmtime`. The host world is declared in WIT (`fractal:host@1.4.0`): typed imports per capability domain, typed exports for lifecycle and hooks.

WASI preview2 is taken selectively (`20 §4.2`): `wasi:filesystem`, `wasi:sockets`, `wasi:http` and `wasi:cli` env/args are **not imported at all**; `wasi:clocks` requires `system.clock` and returns time coarsened to 1ms; `wasi:random` is replaced by a host `rng.next` seeded per invocation from the `Rng` port; `wasi:logging` is replaced by a rate-capped, attributed `log.emit`. Network egress exists only via `net.fetch<host:…>` against an allowlist fixed at install and shown on the consent screen.

Resource control is fuel per call plus an epoch deadline plus a hard memory ceiling. Cancellation is host-initiated, because cooperation is exactly what an adversarial guest declines to offer. **A trap never fails the user's operation**: a vetoing hook that traps yields `proceed`, and the Install's health counter increments.

## 3. Consequences

### Positive
- Authority is legible by reading the component's world, not by auditing its behaviour. `20 §16` X5 — an Install with an empty Envelope can do nothing beyond public reads — is P8's falsification test applied verbatim.
- Attenuation is structural: `attenuate` takes the parent handle and computes an intersection, so widening is unrepresentable rather than merely rejected (X4).
- Determinism holds: same module digest, same `install-context`, same inputs, same seeded `rng` produce the same outputs, so a hook execution is replayable from the log.
- Language-agnostic. An Extension author uses Rust, Go, C, or anything with a component toolchain, without us shipping a runtime per language.
- Fuel metering gives a defence against slow guests that no sandbox based on globals interception can offer.

### Negative
- **Host bindings are ongoing work.** Every hook and every capability domain in `20 §6` needs a WIT interface, a host implementation, an audit record and a version. The world is a public API from the first published version.
- **The component tooling is still maturing.** Guest-side toolchains vary in quality by language, and the `wasm-tools`/`wit-bindgen` surface still moves faster than we would like for something on a security path.
- AOT compilation and an instance pool per Society × Install are real memory: warm instances are cheap individually and not free in aggregate.
- Debugging a guest is worse than debugging native code, and Extension authors will feel that. `20 §14` carries the developer-experience mitigations; they do not eliminate the gap.
- Floating point is canonicalized and threads are forbidden, which some guest languages find awkward. That is the price of replayability.

### Neutral / follow-on work
`fractal-adapter-wasmtime` implements a sandbox port that is **not** on the Canon `10 §7` list; `41 §5.5` keeps it behind `#[cfg(feature = "unstable-sandbox")]` until an ADR adds it to that list, so its provisional status lives in the code rather than in someone's memory.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **JS sandbox (QuickJS, V8 isolates)** | Fastest path; the language extension authors already know; mature tooling | Isolation becomes a property of the engine's correctness rather than of the module boundary, and capability control means intercepting globals — auditing by denylist, which is the model P8 rejects. No comparable fuel metering, one language only, and weaker determinism |
| **A container per Extension** | The right isolation primitive; OS-enforced; well-understood operationally | Wrong granularity by two orders of magnitude: hundreds of milliseconds of startup against an 8ms hook budget, and tens of megabytes each. It also restores an ambient-authority model that `20 §5` spends the whole chapter taking back. Retained as the *outer* layer for Experience sessions, where the cost amortizes over a tick loop |
| **Native dynamic libraries** | Zero overhead; trivially fast; simplest host API | Zero isolation. One bad plugin corrupts the Runtime and the event log. Not a trade-off under P8 — a category error |
| **Iframes / arbitrary web UI for extension surfaces** | Familiar to web developers; unlimited UI expressiveness | A spoofing primitive against the agent-origin guarantees of `32 §5.5`; unbounded main-thread cost against P10; breaks N7's single design system; and it is meaningless on CLI and native mobile, so it would quietly make the GUI the privileged front end and violate P13 |

## 5. Exit Cost

**12–16 engineer-weeks to replace the execution boundary once third-party Extensions exist; ~4 weeks before Phase 6.** The work: implement a replacement host with equivalent capability semantics; re-specify the world in the new host's terms; re-sign and re-verify every published module; and — the dominant cost — either recompile third-party Extensions, which we cannot do, or provide a compatibility shim, which reintroduces the ambient-authority surface we removed. **The exit cost rises sharply the moment third-party code ships**, which is precisely why `02 §3` gates that behind two phases of adversarial Envelope testing and why this ADR lands at Phase 3, not Phase 6.

## 6. Principle Served

**P8** (capability-secure by construction; the deny-by-default falsification test is X5), **P7** (a first-party feature that needs a hook the plugin API lacks must add it to the plugin API), **P6** (seeded `rng` and absent ambient clocks keep hook execution replayable), **P13** (nothing about the boundary privileges the GUI). Traded: developer convenience for Extension authors, and P7 sits last in the `00 §2` conflict order precisely so that P8 wins this trade.

## 7. Falsification Test

1. **X5, mechanically**: instantiate an Extension with an empty Envelope and assert every host call returns `denied` and every write path is unreachable. Run per PR against a purpose-built adversarial fixture Extension.
2. **Absent, not stubbed**: a test installs a guest compiled against `clock` without granting `system.clock` and asserts instantiation fails with a typed error. A guest that instantiates and receives a zero clock is the violation.
3. **X4 attenuation**: property tests over generated handle graphs assert `caps(h') ⊆ caps(h)` and `expiry(h') ≤ expiry(h)` for every derived handle.
4. **Resource control**: a guest that spins is trapped by fuel exhaustion within budget; a guest blocked in a host call is trapped by the epoch deadline; neither fails the user's operation.
5. **Replay**: the same module digest, `install-context`, inputs and seed produce byte-identical outputs across runs and across hosts.
6. **X7**: every host call produces an audit record carrying `install_id` and `envelope_ref`.

## 8. Maintenance Horizon

Wasmtime is a Bytecode Alliance project with multiple corporate maintainers and a published security policy; it is not single-maintainer critical-path. The Component Model and WIT are standards-track, which is the reason to prefer them over a Wasmtime-specific embedding API. The concentrated risk is our own `fractal:host` world: once a third party compiles against `1.x`, the world is a versioned public contract governed by `20 §9`, and breaking it is a marketplace-wide event rather than a refactor.

## 9. Review Trigger

Reopen if (a) Component Model tooling stalls such that guest toolchains for two of our three target languages are unusable at the Phase 3 gate — `10 §12` pre-commits the response: ship first-party Extensions natively and delay third-party execution rather than weakening isolation, because P8 outranks P7; (b) p99 hook latency exceeds the 8ms budget with a warm instance pool; or (c) a sandbox escape is demonstrated in Wasmtime, which triggers the security review gate and an immediate kill switch on third-party execution.
