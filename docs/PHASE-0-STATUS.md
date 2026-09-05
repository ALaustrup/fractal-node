# Phase 0 — Status

> Generated at the end of the first build session. `docs/50 PH0` defines the
> exit gate; this file records honestly which parts of it are met.

## Acceptance criteria (docs/50 PH0)

| # | Criterion | Status | Note |
|---|---|---|---|
| 1 | `cargo xtask verify` passes cold on Linux, macOS, Windows | **Partial** | Green cold on Linux from a `git archive HEAD` export — the same tree `actions/checkout` produces. macOS and Windows still unverified: no runner has executed. The workflow's fast lane was ubuntu-only and could not have satisfied this criterion however many times it ran; it is now a three-OS matrix. |
| 2 | Core compiles to x86_64, aarch64, wasm32 | **Met** | All three built from a clean export with the CI job's exact command and crate list: x86_64 30s, aarch64 3s (gcc-aarch64-linux-gnu), wasm32 21s. |
| 3 | Regenerating from `fractal-schema` produces a zero diff | **Met** | `cargo xtask codegen --check`: 6 artefacts, 0 drift. |
| 4 | Regenerating tokens produces a zero diff across five targets | **Met** | `cargo xtask tokens --check` |
| 5 | Creating a Society via web, CLI and API produces identical event streams | **Met** | `crates/bin/cli/tests/surface_equivalence.rs`. Three independent Nodes, one per surface; the web leg executes `apps/web/app.js` unmodified under Node.js. Logs normalised for ids and clocks, then compared exactly. Found a real P13 divergence on untidy input — see below. |
| 6 | Simulation runs 2,000 seeded histories asserting three invariants | **Met** | `cargo xtask sim`: 2,000 × 40 = 80,000 operations, 0 violations. Asserts five properties, not three. |
| 7 | Fast-lane CI completes under 5 minutes | **Partial** | The eight fast-lane steps run green in **66s** on a cold target directory, measured on a Linux host. A hosted runner adds checkout, toolchain install and cache restore, so the real number will be higher — but not by four minutes. Unverified on a real runner, and unmeasured on Windows and macOS, which are usually the slow ones. |

## What is actually built and green

- 14-crate Rust workspace, layered, with the dependency direction enforced by
  `cargo xtask lint-deps` (14 crates, 0 violations).
- 60 tests passing. The load-bearing ones: a Level 0 Citizen can found their
  first Society; an Agent cannot found one; a retried command creates one
  Society rather than two; replayed state equals the projection.
- Two `EventStore` implementations under a behavioural equivalence test
  (ADR-0016).
- The Policy Enforcement Point, refusing Policy-class actions to every
  non-Citizen principal, exhaustively.
- Design tokens: one source, five generated targets, drift-checked.
- API + CLI + web GUI, all over the same public API, all writing the same log.
- **Schema-first codegen (M0.4).** `crates/support/schema` is the contract;
  `cargo xtask codegen` emits the OpenAPI document, per-event JSON Schema, the
  gateway's operation table, the CLI command tree, and the TypeScript and
  JavaScript clients. `--check` fails on drift, so no surface can fall behind.
- **The simulation harness (M0.6).** 2,000 seeded histories × 40 steps = 80,000
  generated operations, asserting five properties after every single step:
  I-1 (every event names its owning Society, and sequences are dense), I-10
  (the projection always equals a fresh replay), I-14 (a sealed log never
  grows), idempotency (a retried command never mints a second Society), and P4
  (no Society was ever founded by a non-human principal). A failure names the
  seed and step, so it reproduces exactly.
- Two gates that catch different failures: `parity` proves every operation
  reached every generated surface; a CLI integration test proves the binary can
  actually *run* each one. The second was demonstrated to catch a gap the first
  waves through.
- **The offline gate (`cargo xtask offline`).** Reads the front end and fails
  the build on any reference to an origin that is not this Node. Added after
  the walking skeleton was caught loading its fonts from a CDN — see the note
  below.
- **Standing is derived, not asserted.** The first-hearth gate's inputs are
  read from the event log rather than the request body; `docs/11 §2.3`.
- `cargo xtask verify`: format, clippy `-D warnings`, tests, and all five
  Canon gates — green.

## What remains before the gate closes

1. **Run CI once.** The fast lane and the full lane have both now been
   executed locally from a clean export — including `cargo-deny`, which had
   never run anywhere and failed on two counts when it did. Between them these
   local runs found and fixed three CI blockers before a runner ever saw
   them.
   What remains genuinely unexecuted is the *hosted* part: the Windows and
   macOS legs of the matrix (criterion 1), the aarch64 cross-compile under
   `dtolnay/rust-toolchain`, `cargo-deny` (needs network), and the real
   wall-clock of a runner (criterion 7). A CI file that has never run on the
   platforms it claims to cover is still a hypothesis.

2. **A remote.** The repository is local-only. Push to an origin so the commit
   protocol gate has something to check against.

3. **Choose a licence.** `LICENSE` is a deliberate placeholder. It interacts
   with the marketplace (`docs/19`), the Extension SDK and self-hosted Nodes
   (`docs/50 PH6`), and should be decided with those in view.

## Note on falsifying the simulation

Three bugs were deliberately injected to check the harness could see them: a
no-op `seal()`, a replay that drops a field, and a Policy Enforcement Point that
waves Agents through. The second and third were caught immediately.

**The first was not**, and that finding was worth more than the other two. I-14
was asserted after every step but never *exercised* — nothing in the simulation
ever tried to write to a sealed Society, so the assertion could not fire. The
step that was supposed to do it was a comment saying the invariant would handle
it. After adding a real append attempt, the same injected bug is caught at seed
0, step 29.

An invariant that is checked but never exercised is decoration, and the only way
to tell the difference is to break the thing on purpose.

## Note on the two gates

Adding an operation to the contract and not implementing it in the CLI passes
`parity` — the operation is present in all five generated surfaces, because the
generator put it there. `crates/bin/cli/tests/reachable.rs` walks the real
argument parser and catches it. Both were verified by deliberately breaking
them; a gate that has never failed is a gate nobody has tested.

## Note on what criterion 5 found when it stopped being a memory

"Verified by hand" survived the whole of Phase 0. Making it a test took an
afternoon and found two things, neither of which hand-verification could have.

**A real P13 divergence.** Given `"  FirstHearth  "` — whitespace, the most
ordinary input defect there is — the web GUI founded a Society and the CLI and
the API both refused it with `invalid_handle`. The GUI trimmed before sending;
nothing else did. Same intent, three answers. `Handle::parse` already strips a
leading `@` and lowercases, so normalisation was always its job; whitespace had
simply drifted out to one front end. Trimming moved into the parser and the
`.trim()` calls came out of `app.js`. Normalisation performed by a caller is
normalisation each caller can get differently.

Note which test caught it: the first version used clean inputs and passed while
the divergence was live. Tidy fixtures test the path where surfaces already
agree.

**A test that was lying.** The first version passed a deliberately injected
domain bug with four green ticks. `CARGO_BIN_EXE_` covers only the package
under test, and cargo rebuilds only what that package needs — so
`cargo test -p fractal-cli` left `fractal-node` exactly as it was, and the
Runtime under test had been compiled before the bug existed. The test now
builds the Runtime it drives. A test that silently exercises a stale artefact
is worse than no test: it reports on a build nobody has any more.

Both were caught the same way as everything else this week — by trying to break
the thing on purpose and being surprised when nothing broke.

## Note on the Windows leg, before it has run

Windows is the primary platform (N2) and the fast lane had never targeted it.
Rather than push and wait, the hazards were tested here as far as a Linux host
can test them.

**Line endings — real, and already covered.** Rewriting the working tree to
CRLF fails `codegen --check`, `tokens --check` and `cargo fmt --check`
immediately: those gates compare bytes, and a CRLF checkout changes every one
of them. But cloning with `core.autocrlf=true` — exactly the config a hosted
Windows runner carries — produces an LF working tree anyway, because
`.gitattributes` has forced `* text=auto eol=lf` since the first commit, and
the woff2 files come back byte-identical because they are marked `binary`. All
six gates pass on that clone. The protection was already there; now it is
demonstrated rather than assumed.

**One real gap, closed.** `codegen` pipes generated Rust through `rustfmt` via
stdin, and rustfmt resolves `rustfmt.toml` relative to the *current directory*
— so `newline_style` depended on where xtask was invoked from. On Windows the
default is CRLF, which would have failed `codegen --check` for a reason with
nothing to do with the contract. Now passed explicitly as
`--config newline_style=Unix`.

**Checked and clean:** no reserved device names (CON, PRN, AUX, NUL, COM*,
LPT*), no characters illegal in Windows paths, no case-only filename
collisions, no trailing dots or spaces, no path over 200 characters, no
symlinks, no file-permission assumptions, no test asserting on a POSIX path
string. One external command is invoked — `rustfmt` — which rustup shims on
every platform.

What this does NOT establish: that the Rust toolchain, the tests and the
adapters behave identically on a real NTFS filesystem. Only the runner answers
that.

## Note on the first clean-checkout run

Attempting the fast lane against a `git archive HEAD` export — byte-for-byte
what `actions/checkout` gives a runner — failed three gates immediately:
contract drift, token drift and API/CLI parity. Nothing had drifted. The files
those gates compare against were not in the repository.

`**/dist` was ignored, the JavaScript reflex. But
`packages/api-client/dist/index.js` and `packages/tokens/dist/*` are not
bundler output — the Runtime *serves* both to the browser, so a fresh clone
could not run the web GUI, and the three gates whose whole purpose is to prove
no surface has fallen behind had no surface to compare against. A drift check
against a file the repository does not contain is not a check.

Every local tree had those files sitting untracked on disk, so `verify` was
green on every machine that had ever run `codegen` — which was all of them.
This class of defect is invisible until something builds from the repository
rather than from a working directory, and that is the entire argument for
running CI early rather than at the end of a phase.

Fixed, and the lane now runs green from a clean export in 66 seconds.

## Note on the three findings from the first end-to-end run

Running the whole loop for real — CLI create, then the same Societies read back
through a browser — found three defects that every green gate had missed. All
three are fixed, and the fixes were kept honest by first breaking them again:

1. **The founding gate read its input from the caller.** `societies_founded`
   and `founder_level` arrived in the request body and defaulted to zero, so
   the first-hearth allowance renewed on every request and both demo Societies
   were recorded as `origin: first_hearth`. The Runtime now derives both from
   the log. Injecting the original bug fails two unit tests and is caught by
   the simulation at seed 0, step 10. Half the hole remains open by design:
   the identity the count is taken against is still asserted (PH1's passkey
   session closes it), and every affected response now says so in its
   `warnings`.

2. **The GUI loaded its typography from `fonts.googleapis.com`.** P2 and P9,
   broken silently, past seven gates that were all reading Rust. The fonts are
   vendored (76 KB, latin, SIL OFL) and `cargo xtask offline` now refuses any
   third-party reference. Verified against six evasions, including a
   protocol-relative `//host` URL and a `fetch()` to an analytics endpoint, and
   verified not to fire on prose that merely names a URL.

3. **No favicon**, so every page load 404'd. Added as an SVG built from the
   header mark.

A fourth surfaced while fixing the first: the generated client returned
`json.data` and discarded the envelope's `warnings` entirely, which made the
warning channel decorative — the Runtime could say `unauthenticated` and no
front end would ever hear it. `createClient` now takes an `onWarning` sink,
defaulting to `console.warn`, drained on every successful response.

The pattern in all four is one thing: **every gate was reading the Rust.**
Nothing read the front end, and nothing ran the system end to end. Both are now
in CI.

## Note on the toolchain

`rust-toolchain.toml` moved from 1.83.0 to 1.98.1 during this session. The
initial conservative pin failed on the first real dependency: the crate
ecosystem now requires edition 2024. Recorded as ADR-0015.
