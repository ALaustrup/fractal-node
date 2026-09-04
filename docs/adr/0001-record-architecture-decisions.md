# ADR-0001 — Record architecture decisions as numbered, falsifiable ADRs

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 0

## 1. Context

`00 §3` requires an ADR for every technology choice, `00 §7` makes the Canon amendable only by ADR, `02 §5` budgets zero open ADRs at a phase gate, and `40 §6` specifies when one is required and what it contains. All four presuppose a practice that does not exist yet: `docs/adr/` is empty, and the decisions already made live as summary rows in `10 §11` and as prose spread across seven chapters.

Two forces make this urgent. **Most of this code will be written by AI coding agents**, and `00 §0` records the operative fact: agents satisfy stated constraints reliably and infer unstated ones badly. A decision that exists only as an implication of a chapter is an unstated constraint. And **the expensive failure is not a wrong decision but an unrecoverable one** — `00 §3.3` demands an exit cost in engineer-weeks for every choice, and no chapter carries that number.

A summary table is not a substitute. `10 §11` gives no exit cost, no falsification test, no maintenance horizon, and no review trigger — the four fields `40 §6.3` calls out as the ones that make a record useful two years later.

## 2. Decision

We record architecture decisions as ADRs in `docs/adr/NNNN-kebab-title.md`, numbered sequentially from `0001`, never renumbered, never deleted, using the nine-section template of `40 §6.3` verbatim and in order. An ADR is required under exactly the eight triggers in `40 §6.1` and not otherwise. Lifecycle is `Proposed → Accepted | Rejected`, then `Accepted → Superseded by NNNN | Deprecated`; a superseded ADR stays in place with a forward link. An ADR is a pull request; agents may draft, **only a human may move one to `Accepted`** (`40 §6.5`), because architecture is policy and P4 reserves policy to humans. ADRs 0002–0014 seed the corpus with decisions the blueprint has already made.

## 3. Consequences

### Positive
- Every commitment acquires a stated exit cost and a mechanical falsification test, so `00 §3.3` becomes data and drift is caught by CI rather than by memory.
- Agents get one bounded, citable artifact per decision instead of reconstructing intent from five chapters.
- Rejected alternatives stay recorded, which stops the same idea returning every six months with fresh enthusiasm and no new facts.

### Negative
- Roughly half a day of human review per non-trivial ADR, and the zero-open-ADRs gate makes that time non-deferrable.
- **ADR inflation is the live risk** (`40 §6.1`): 300 ADRs is a corpus nobody reads, which is no corpus with extra ceremony. The eight-trigger list is the only defence and must be applied strictly.
- Numbering is permanent, so a mis-sequenced draft is a permanent wart. Accepted over renumbering, which breaks inbound links.

### Neutral / follow-on work
`docs/proposals/` holds out-of-phase ideas (`02 §8`); a proposal is not an ADR and does not count against the gate. Chapter summary tables remain as orientation and link to the ADR owning each row.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| Chapters only, no ADR corpus | The chapters are denser and better written than most ADRs; zero new process | Chapters record *what*, never what it costs to leave. And chapters are continuously edited, so the record of what we believed in 2026 is overwritten by what we believe now — destroying the one thing the practice preserves |
| Decision log in an issue tracker | Searchable, linked to work, no repo churn | Not versioned with the code, not reviewable as a diff, not loadable by an agent that reads the repo, and dependent on a vendor's export format. `41 §14` gates architectural files through CODEOWNERS; a tracker has none |
| Lightweight Nygard ADRs (context, decision, consequences) | Industry default; shorter, so more likely to be written | Omits exit cost, falsification test, maintenance horizon and review trigger — exactly what `00 §3` requires of every technology choice. A three-section ADR cannot discharge the Canon's own rule |

## 5. Exit Cost

**≈1 engineer-week.** The corpus is Markdown with no runtime dependency; abandoning it means deleting the `adr-gate` job and amending `40 §6`, `00 §3` and `02 §5`. The real cost is not mechanical — it is losing the exit-cost estimates in ADRs 0002–0014, which is the corpus's actual value.

## 6. Principle Served

**P5** — a swappable boundary whose exit cost is unrecorded is not measurably swappable. **P4** — the human-only `Accepted` transition encodes that architecture is policy. Indirectly **P12**: an economy whose parameters change without a recorded decision is not honest, merely current. No principle is traded away; the cost is process time.

## 7. Falsification Test

CI job `adr-gate`, required on every PR, never path-filtered:

1. **Trigger check.** A diff touching `[workspace.dependencies]`, a trait in `crates/ports/`, the `11 §7` invariant suite, a `PostingReason` variant, the crypto or authn/authz path, `docs/0[0-2]-*.md`, a new `unsafe` block, or a new crate under `crates/api/` or `apps/` requires an `ADR-\d{4}` reference in the PR body resolving to a file in `docs/adr/`.
2. **Structural check.** Every ADR parses to exactly the nine `## N.` headings of `40 §6.3`, in order, each non-empty. An empty `### Negative` fails the build.
3. **Gate check.** At a phase gate, occurrences of `**Status:** Proposed` across `docs/adr/*.md` must be 0.
4. **Link check.** Every `Superseded by ADR-MMMM` resolves to an existing accepted ADR.

The decision has stopped holding the moment `adr-gate` is disabled, made non-required, or path-filtered.

## 8. Maintenance Horizon

First-party Markdown plus ~150 lines in `xtask` (`41 §11`). No external maintainer and no third-party dependency: the structural check parses headings by regex rather than through a Markdown library, deliberately, so the gate has nothing that can be abandoned upstream.

## 9. Review Trigger

Reopen if (a) the corpus passes 60 accepted ADRs — the trigger list is then demonstrably too permissive and must be narrowed, not the practice abandoned; (b) median ADR review latency exceeds 10 working days for two consecutive months, meaning the gate is blocking code; or (c) a post-incident review finds a root cause in a change that qualified under `40 §6.1` but shipped without an ADR, which means the trigger check has a hole.
