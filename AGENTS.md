# AGENTS.md — Working Agreement for AI Coding Agents

> Read this before doing anything in this repository. It is short on purpose.
> It is binding. `docs/00-foundational-principles.md` outranks it; nothing else does.

---

## 1. Load the Canon first. Every time.

Before any implementation task, read these four files in full:

```
docs/00-foundational-principles.md    the thirteen principles + the definition of done
docs/01-canonical-terminology.md      the vocabulary and the banned-word list
docs/02-scope-guardrails.md           what we are NOT building yet
docs/03-phase-authority.md            which phase your capability belongs to
```

Then read the chapter that governs what you are building, and its stated prerequisites.

If you have not read the Canon, you are not qualified to write code here — not because of policy, but because you will violate an invariant you did not know existed, and it will be found in a phase where it is expensive.

---

## 2. The five rules

**R1 — Stay in phase.** Check `docs/phases.toml`. If the capability you are about to build is not assigned to the current phase, stop. Write a proposal in `docs/proposals/` instead. Scope expansion is the failure mode this project is most likely to die of.

**R2 — Use the canonical vocabulary.** Never `user` (say Citizen or Principal). Never `channel`/`room`/`forum` (say Chamber). Never `bot` (say Agent). Never `NFT` (say Facet). Never `server` (say Node or Runtime). The full list is `01 §10`, and a lint enforces it. Introducing a new term requires adding it to `01` in the same PR.

**R3 — Cite the principle.** Any non-obvious design decision names the principle it serves (P1–P13) in the PR description. A decision that serves no principle is decoration and should be removed.

**R4 — Halt on conflict.** If two principles genuinely conflict and `00 §2`'s resolution order does not settle it, **stop and ask**. Do not resolve it unilaterally. Escalation costs an hour; a violated invariant discovered in PH5 costs a phase.

**R5 — Done means all eight.** `00 §5` lists eight criteria. Seven of eight is not done; it is a defect with good marketing. Do not set a task to `completed` unless all eight hold, and say which ones you verified and how.

---

## 3. Committing

Full protocol in `docs/42-source-control-automation.md`. The short version:

- **Commit at Work Unit boundaries, never per file.** A Work Unit is one coherent, independently reviewable, independently revertible change that satisfies all eight done-criteria.
- **Verify before committing.** Run `cargo xtask verify`. It must be green. A red build is never committed, not even with a note.
- **One Work Unit = one branch = one PR = one squashed commit** on a linear trunk. Branch lifetime is capped at 48 hours.
- **Checkpoint in `refs/wip/<agent>/<wu-id>`,** never on a branch, never as a "WIP" commit. Those refs are garbage-collected at 7 days and are squashed out of existence before the real commit is composed.
- **Compose the message from the Work Unit definition** — Conventional Commits with the required trailers (`Work-Unit`, `Milestone`, `Principle`, `ADR`, `Done`, `Agent`, `Model`, `Task`, `Co-Authored-By`).
- **If the Work Unit turns out to be too large,** split it using the procedure in `42`. Do not commit a half-finished one.

**Never:** commit failing code · commit secrets or generated artifacts · amend published history · force-push to trunk · bypass hooks · self-merge without gates · commit "WIP" · fix an unrelated thing "while I'm in here".

---

## 4. Writing code

- **Dependency direction is law.** `domain → ports` only. A domain crate that imports an adapter or a vendor type fails the build. If you need I/O in the domain, you need a port, not an exception.
- **No `unwrap` or `expect` in production paths.** `thiserror` in libraries, `anyhow` only at binary edges.
- **No `SystemTime::now`, no `thread_rng`, no `Uuid::new_v4`** anywhere in domain or application code. Use the `Clock`, `Rng`, and `IdGen` ports. This is what makes the whole system replayable, and replayability is what makes the invariants verifiable.
- **No literal hex colors** outside `tokens/`. No new UI pattern without a `docs/32` entry.
- **No float arithmetic in ledger crates.** Ever.
- **Every state change is a domain event.** Never mutate a projection directly.
- **Every abstraction needs two callers** — except the P5 swappable-boundary list, which is abstracted by decree.

---

## 5. Testing

- The fifteen invariants in `docs/11 §7` are each a property test. If your change touches a boundary that could break one, extend the simulation, do not just add a unit test.
- New behavior gets tests at the level appropriate to its risk class (`docs/40 §7`).
- **A failing test you did not cause is not yours to delete or skip.** Quarantine it per `40`, open the issue, and say so. Deleting someone else's failing test is the single most damaging thing an agent can do in this repository.
- If a test passes whether or not your implementation is correct, it is not a test. Write one that fails when the code is wrong.

---

## 6. Writing the PR description

```
  What:      one sentence
  Why:       the Work Unit goal + the principle served (P#)
  Phase:     PH<n> M<n.n> WU-<n.n>-<nnn>
  Done:      which of the 8 criteria you verified, and how
  Risk:      what could this break, and what covers that
  ADR:       if this introduced or changed a technology choice
```

Reviewers are checking for what automation cannot: whether the tests mirror the implementation rather than the requirement, whether an abstraction earned its keep, whether the Canon was actually followed rather than cited. Make those things easy to check.

---

## 7. When you are unsure

The correct action, in order of preference:

1. Read the governing chapter again. The answer is usually there.
2. Check `docs/61-reconciliation.md` — 49 known conflicts already have rulings.
3. Write the proposal and stop.
4. Ask.

The wrong action is to pick the interpretation that lets you continue. This blueprint is 271,000 words precisely so that guessing is never necessary.
