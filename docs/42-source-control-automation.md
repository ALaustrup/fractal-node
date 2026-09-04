# 42 — Source Control and Agent Automation

> **Prerequisites:** `00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`, `10-system-architecture.md`, `40-engineering-standards.md`, `41-repository-structure.md`.
> **Governs:** branching, commit format, work-unit granularity, changelogs, versioning, releases, milestone gates, repository hygiene automation, traceability, and the agent protocols that perform all of it without a human acting as the commit button.

This chapter is the implementation of **non-negotiable N1**. `40` owns engineering standards, test tiers, and the CI stage definitions; `41` owns the workspace layout and crate names. This chapter owns *when history is written, by whom, in what shape, and what the machine does about it afterwards.* Where a CI stage is named here, `40` defines what it runs; where a crate or scope name is used here, `41` defines the canonical list. Nothing is duplicated across the three.

---

## 1. The Problem, Stated Precisely

Fractal Node is built substantially by AI coding agents. An agent working a single task produces between 40 and 400 file-level edits: it writes a function, tests it, discovers a bad signature, rewrites it, adds a trait bound, fixes the four call sites, reformats, adds a doc comment, deletes a scratch helper. Every one of those is a *Change*. None of them is history worth keeping.

Two naive policies both fail, and they fail in opposite directions:

| Policy | Immediate effect | Why it fails |
|---|---|---|
| Commit every file change (auto-commit on save) | Perfect crash recovery | History becomes a keystroke log. `git log` is unreadable. `git bisect` lands on a commit that does not compile roughly 70% of the time, so bisection is worthless. Review is impossible: there is no diff that represents a decision. Revert has no safe target. |
| Commit once at the end of a session | Clean-looking log | A 3,000-line commit spanning six service boundaries. Bisect granularity collapses to "the day it broke". Review degenerates to rubber-stamping. Revert is all-or-nothing, so a single bad decision inside the batch cannot be removed without removing the good work with it. |

Both failures are the same failure: **the unit of history does not match the unit of decision.** History is only useful when a commit is a thing a human or an agent can reason about as one idea — read in five minutes, revert in one command, bisect onto with confidence that it either works or does not.

So the design starts by naming that unit, defining it testably, and then building every other mechanism — branches, messages, changelogs, gates, releases, hygiene — on top of it. Everything in this chapter follows from one decision: **the Work Unit is the atom of history.**

A second constraint shapes everything equally. Per `02 §7` and the user's standing requirement, automation that generates work for a human is worse than no automation. A hygiene bot that opens eleven pull requests a week has not saved time; it has moved the labour from writing code to triaging robots. Section 12 therefore states a hard, measured noise budget, and section 12.3 states the rule that governs every job in this chapter: **a hygiene job that produces noise gets tuned or deleted, not tolerated.**

---

## 2. The Work Unit Model

### 2.1 The hierarchy

```
   PHASE                 PH0 … PH9 — roadmap-level. Months. Has a complexity
     │                   budget (02 §5) and a gate. Named in the roadmap.
     │  contains 4–12
     ▼
   MILESTONE             M<phase>.<n>  e.g. M3.2 — a shippable increment with
     │                   written acceptance criteria. 1–4 weeks. Declared as a
     │  contains 5–40    manifest file in the repo (§10). Closing one is the only
     ▼                   event that requires a human signature.
   WORK UNIT             WU-<milestone>-<seq>  e.g. WU-3.2-014 — THE ATOM OF
     │                   HISTORY. One coherent, independently reviewable,
     │  contains 1–400   independently revertible change satisfying 00 §5.
     ▼                   Becomes exactly one commit on trunk.
   CHANGE                An edit to a file. Has no identity, no message, no
                         review, and never appears in trunk history on its own.
```

Note on notation: phases are written `PH0`…`PH9`, never `P0`…`P9`, because `P1`…`P13` are reserved for the Foundational Principles and an agent that confuses the two will cite the wrong thing in a commit trailer.

### 2.2 What makes something a Work Unit

A Work Unit is a change that satisfies **all** of:

1. **Coherent.** It can be described in one imperative sentence under 72 characters with no "and".
2. **Complete.** It satisfies all eight criteria of `00 §5` at the moment it lands, or explicitly waives a criterion with a named human approver.
3. **Green.** Trunk compiles, tests pass, and all budgets hold both before and after it — so it is a valid bisect landing point.
4. **Reviewable.** A competent reviewer can evaluate it in ≤ 20 minutes. In practice: ≤ 400 changed lines excluding generated files, lockfiles, and fixtures; ≤ 3 service boundaries touched.
5. **Revertible.** `git revert <sha>` produces a working tree. It does not depend on a later commit to be correct.
6. **Attributable.** It maps to exactly one Milestone and cites at least one Principle or requirement.

If any of the six fails, it is not a Work Unit. It is either a Change (too small to be history) or two Work Units wearing a trenchcoat (§7.6 splits it).

### 2.3 Examples and counter-examples

| Candidate | Verdict | Reasoning |
|---|---|---|
| Add `Ledger::post_balanced()` with unit tests, doc comment, and the `PostingRecorded` event | **Work Unit** | One idea; compiles; testable; revertible; cites P11. |
| Add the `wallet transfer` CLI verb, wire it to the existing API, add CLI help and a changelog line | **Work Unit** | One capability, one surface; satisfies `00 §5.2` and N3. |
| Rename `EventStore::append` to `EventStore::append_batch` across 31 call sites | **Work Unit** | Mechanical, coherent, atomic. Large line count but reviewable in minutes — the 400-line guidance is a heuristic, not a law, and mechanical renames are the standard exception. |
| Fix a null-deref in `fractal-domain-society` | **Work Unit** | Includes the regression test. Without the test it is a Change, because `00 §5.3` fails. |
| Format 900 files with `rustfmt` after a config change | **Work Unit**, `style` scope, and it goes into `.git-blame-ignore-revs` (§13.3) | Coherent and mechanical; must be alone in its commit. |
| "Add a function signature, will implement next" | **Change** | Fails Complete and Green. Belongs in a wip checkpoint (§6). |
| "Fix typo in the doc comment I wrote 20 minutes ago in this same task" | **Change** | Fold it into the Work Unit under construction. It is not history. |
| Implement the Vault manifest format *and* the Custodian attestation loop *and* the settlement posting | **Not a Work Unit — three of them** | Three boundaries (S4, S4, S5), three decisions, three revert targets. Split per §7.6. |
| "Address review comments" | **Never a commit on trunk** | Review feedback is folded into the Work Unit it belongs to before squash-merge. A trunk commit that says "address review comments" is a commit with no idea in it. |
| Bump 14 dependencies | **Work Unit** (one, batched — §11.1) | Batched deliberately; one revert target for a whole week of dependency drift. |
| A `chore:` commit containing an unrelated "while I was in here" refactor | **Violation of `02 §7`** | The refactor is its own Work Unit. Reject at review; the agent splits. |

### 2.4 Why not smaller, why not larger

Smaller units (per-file, per-function) were rejected because they break criterion 3: intermediate states of an agent's edit stream do not compile, which destroys `git bisect`, which is the single highest-value property of a project history that no human read line by line. Larger units (per-session, per-milestone) were rejected because they break criteria 4 and 5: review and revert both degrade non-linearly with diff size, and the entire safety argument for high-throughput agent work rests on cheap, precise revert.

**Honest cost of the Work Unit model:** agents must plan. An agent that starts typing without a Work Unit definition will produce a change that cannot be committed under this policy and will have to split it retroactively, which costs more than planning would have. Section 7 makes the definition an input to the work, not an output.

---

## 3. Branching Model

### 3.1 The decision

**Trunk-based development with short-lived Work Unit branches, a merge queue, squash-merge to a strictly linear `main`.** No long-lived branches. No develop branch. No release branches until PH5.

```
main  ──●───●───●───●───●───●───●───●───●───►   linear, always green, always releasable
         \       \       \       \
          ●       ●       ●       ●             wu/* branches: hours, not days
        WU-014  WU-015  WU-016  WU-017
          │       │       │       │
          └───────┴───┬───┴───────┘
                      ▼
              ┌───────────────┐
              │  MERGE QUEUE  │  batches ≤ 5, tests the batch as it will land,
              │  (speculative)│  bisects the batch on failure, evicts the culprit
              └───────┬───────┘
                      ▼  squash → one commit per Work Unit
                    main
```

### 3.2 Why, against the alternatives

| Model | Rejected because |
|---|---|
| **git-flow** (develop + release + hotfix + feature) | Optimized for scheduled releases of shrink-wrapped software by large human teams. Its cost is merge debt between long-lived branches, which agents generate faster than humans can resolve. It has no answer to "40 concurrent Work Units", and its `develop` branch is a second trunk that is never actually green. |
| **GitHub flow** (branch, PR, merge commit, deploy) | Close to correct, and this model is a hardened variant of it. Rejected as specified because merge commits produce a non-linear history in which `git bisect` traverses states that were never tested together, and because without a merge queue every merge invalidates the CI result of every open PR — the "green PR, red main" failure that appears the moment concurrency exceeds ~5. |
| **Stacked diffs** (Graphite/Phabricator style) | Genuinely excellent for a decomposed human change series, and the strongest rejected candidate. Rejected for now on three grounds: the tooling is external to git and adds a required third-party service to a project whose complexity budget is already committed; agents naturally produce *independent* units rather than dependent stacks, so the primary benefit is under-used; and restacking on conflict is an operation agents perform badly. **Revisit at PH5** if median Work-Unit dependency depth exceeds 2. |
| **No branches — commit directly to trunk** | Rejected: it removes the pre-merge gate, which is the only place an agent's work is verified against the *current* trunk rather than the trunk it started from. It also removes the pull request, which is where the human-readable rationale lives. |

**What we buy:** a linear, bisectable, one-commit-per-idea history; a trunk that is green by construction rather than by hope; and reviews that are small because branches are short. **What it costs:** the merge queue is infrastructure that must be operated, and a red merge queue blocks everyone. Section 15.6 covers that failure mode.

### 3.3 Branch naming

```
  wu/<milestone>-<seq>-<kebab-slug>        wu/3.2-014-ledger-posting-idempotency
  hygiene/<job>-<yyyymmdd>                 hygiene/deps-20260907
  fix/<incident-id>-<kebab-slug>           fix/INC-0007-relay-signal-leak
  release/<major>.<minor>                  release/1.4        (PH5+ only)
  wip/<agent-id>/<wu-id>                   refs/wip/agent-07/WU-3.2-014  (never a branch — §6)
```

The `<milestone>-<seq>` segment *is* the Work Unit ID minus the `WU-` prefix. This is load-bearing: the branch name, the commit trailer, the PR title, the milestone manifest entry, and the changelog entry all key on the same identifier, which is what makes §13 traceability mechanical rather than archaeological.

### 3.4 Rules

| Rule | Value | Enforcement |
|---|---|---|
| Maximum branch lifetime | **48 hours** from first commit to merge | Bot warns at 24h, escalates at 36h, auto-closes with the work preserved as a `wip/` ref at 48h |
| Maximum concurrent branches per agent | 3 | Pre-push hook |
| Rebase onto trunk before entering the queue | Required | Merge queue enforces |
| Merge strategy | **Squash**, always | Branch protection: squash is the only enabled method |
| Trunk history | **Strictly linear** | `main` rejects merge commits |
| Force-push to `main` | Forbidden, no exception, no bypass role | Branch protection + repository ruleset |
| Force-push to `wu/*` | Allowed and expected (rebasing) | — |
| Delete branch on merge | Automatic | Repository setting |
| Direct push to `main` | Forbidden for every principal including the repository owner | Ruleset with no bypass list |
| Required approvals | 1 (agent-reviewer or human) for `feat`/`fix`/`refactor`; 0 for `deps`/`chore`/`docs` that pass all gates | Branch protection + CODEOWNERS |
| Required status checks | The full list in `40 §CI` | Branch protection |

**48 hours is a real number, not a slogan.** A branch older than two days has, empirically, either grown past the reviewable size or has been abandoned. Both are conditions we want detected automatically rather than discovered at milestone close.

### 3.5 Release branches

None before PH5. Until then, every release is cut from a tag on `main` and a defect is fixed by rolling forward. From PH5 — when self-hosted Nodes exist and users run versions we do not control — `release/<major>.<minor>` branches are created at the first `.0` tag of a minor line, receive only cherry-picked `fix` and `sec` commits, and are deleted 90 days after the succeeding minor line reaches general availability. Backports are agent-performed cherry-picks that must pass the same gates; a backport that conflicts is escalated to a human rather than resolved creatively.

---

## 4. Semantic Commit Strategy

### 4.1 Format

Conventional Commits 1.0.0 with a project-specific type set, a scope vocabulary bound to the service boundaries, and required trailers.

```
<type>(<scope>)<!>: <subject>
                                       ← blank line
<body: why, not what. wrap at 72. omit only for mechanical changes>
                                       ← blank line
BREAKING CHANGE: <consequence and migration>     (only when <!> is present)

Work-Unit: WU-3.2-014
Milestone: M3.2
Principle: P11, P6
ADR: 0031
Done: 8/8
Agent: claude-code/agent-07
Model: <model-id>
Task: TSK-9f2c1a
Co-Authored-By: Claude <noreply@anthropic.com>
```

| Field | Required | Notes |
|---|---|---|
| `type` | Always | From §4.2. Lowercase. |
| `scope` | Always except `ci`, `build`, `chore`, `revert` | From §4.3. Exactly one; a change needing two scopes is two Work Units. |
| `!` | When breaking | Must co-occur with a `BREAKING CHANGE:` footer. |
| `subject` | Always | Imperative mood, ≤ 72 chars, no trailing period, canonical vocabulary from `01`. |
| `body` | For `feat`, `fix`, `perf`, `sec`, `api`, `event`, `refactor` | States *why*. The diff already states what. |
| `Work-Unit:` | Always | The ID. One per commit. |
| `Milestone:` | Always | Must exist and be open in `milestones/`. |
| `Principle:` | For `feat`, `refactor`, `api`, `event`, `sec`, `perf` | One or more of `P1`–`P13`, or `N1`–`N8`. |
| `ADR:` | When a technology choice is introduced or changed (`00 §3`) | Zero-padded ADR number. |
| `Done:` | Always | `8/8`, or `k/8 Waived: <criterion>=<reason> Approved-By: @handle`. |
| `Agent:` / `Model:` / `Task:` | For every agent-authored commit | §14. |
| `Co-Authored-By:` | For every agent-authored commit | Provenance. |
| `Refs:` | Optional | Issue or proposal IDs. |

### 4.2 Type vocabulary and semver mapping

| Type | Use when | Changelog section | Semver (workspace) | Semver (public API, `30`) |
|---|---|---|---|---|
| `feat` | New user- or API-visible capability | Added | minor | minor |
| `fix` | Defect repair, with a regression test | Fixed | patch | patch |
| `perf` | Measured performance improvement; benchmark required (`02 §7`) | Performance | patch | patch |
| `sec` | Security fix or hardening | Security | patch | patch, plus advisory |
| `api` | Public API contract change (shape, semantics, deprecation) | API | minor or major | minor or **major** |
| `event` | Domain event schema addition or change (P6) | Events | minor or major | minor or major |
| `refactor` | Internal restructuring, provably no behaviour change | (excluded, see §8.4) | none | none |
| `test` | Tests only | (excluded) | none | none |
| `docs` | Documentation only, including this corpus | Documentation (Milestone notes only) | none | none |
| `build` | Toolchain, workspace, compilation, packaging | (excluded) | none | none |
| `ci` | Pipeline, gates, hygiene jobs | (excluded) | none | none |
| `deps` | Dependency version changes (batched, §11.1) | Dependencies | patch, or minor if runtime behaviour changes | patch |
| `adr` | ADR added, amended, or superseded | Decisions (Milestone notes only) | none | none |
| `style` | Mechanical formatting only; nothing else in the commit | (excluded) | none | none |
| `chore` | Repo hygiene with no source effect (branch cleanup, file moves) | (excluded) | none | none |
| `revert` | Reverts an earlier commit | Reverted | inherits the reverted commit's bump | inherits |

`wip`, `misc`, `update`, `wip:`, and `temp` are not types. The commit-msg hook rejects them by name with a message pointing here. `update` is additionally banned by `01 §8`.

Breaking-change signalling: `!` after the scope **and** a `BREAKING CHANGE:` footer stating the consequence and the migration. Either without the other is a lint failure. A `feat!` or `api!` on a crate that is published (`0.x`) bumps minor; post-1.0 it bumps major and cannot land without a human Milestone signature (§9.5).

### 4.3 Scope vocabulary

Scopes are the fourteen service boundaries of `10 §3`, plus a closed set of cross-cutting scopes. `41` holds the canonical crate-to-scope map and CI validates the commit scope against it, so a new crate cannot introduce an unregistered scope.

| Domain scopes (S1–S14) | `identity` (S1), `society` (S2), `discourse` (S3), `vault` (S4), `ledger` (S5), `economy` (S6), `progression` (S7), `asset` (S8), `governance` (S9), `agent` (S10), `extension` (S11), `market` (S12), `discovery` (S13), `relay` (S14) |
|---|---|
| Layer scopes | `ports`, `adapters`, `app`, `gateway` |
| Surface scopes | `api`, `cli`, `web`, `desktop`, `mobile`, `sdk` |
| Cross-cutting | `design-system`, `telemetry`, `crypto`, `docs`, `repo`, `ci` |

Multi-scope commits are forbidden. A change that genuinely spans `ledger` and `economy` is either a port change (`ports`) or two Work Units. This rule is what keeps the changelog groupable and `git log --grep '(ledger)'` meaningful.

### 4.4 Worked examples

```
feat(ledger): record Postings through a balanced double-entry writer

Every Fraction movement must be expressible as a balanced Posting pair so
that the Ledger trait can be swapped for the future FN L1 without domain
changes. The writer rejects unbalanced input at the type level rather than
at runtime, which removes the class of defect entirely.

Work-Unit: WU-3.2-014
Milestone: M3.2
Principle: P11, P12, P6
ADR: 0031
Done: 8/8
Agent: claude-code/agent-07
Model: <model-id>
Task: TSK-9f2c1a
Co-Authored-By: Claude <noreply@anthropic.com>
```

```
fix(relay): stop leaking Signal subscriptions on abrupt socket close

A client that dropped without a close frame left its subscription in the
fan-out map forever; 41 hours of soak produced 180k orphans and a 2.3GB
resident set. The Relay now ties subscription lifetime to the connection
task's drop guard. Regression test drops 10k sockets uncleanly and asserts
the map returns to zero.

Work-Unit: WU-3.2-021
Milestone: M3.2
Principle: P10
Done: 8/8
Agent: claude-code/agent-03
Model: <model-id>
Task: TSK-b71e40
Co-Authored-By: Claude <noreply@anthropic.com>
```

```
api(agent)!: require an explicit envelope_ref on every agent-issued command

Agent commands previously inherited the envelope from the session, which
made the audit trail ambiguous when an Operator held several Envelopes.
The command now carries the Envelope it claims authority under and the
Policy Enforcement Point verifies the claim, so every AgentActionBlocked
event names the exact Envelope that failed.

BREAKING CHANGE: POST /v1/societies/{id}/agents/{id}/commands now requires
the `envelope_ref` field. Requests without it are rejected with 400
`envelope_ref_required`. SDKs 0.9.x and earlier must be upgraded; the
migration is a one-line addition documented in docs/migrations/0031.md.

Work-Unit: WU-3.3-004
Milestone: M3.3
Principle: P4, P8
ADR: 0034
Done: 8/8
Agent: claude-code/agent-11
Model: <model-id>
Task: TSK-4dd902
Co-Authored-By: Claude <noreply@anthropic.com>
```

```
deps(repo): batch weekly dependency update (14 crates, 2 minor, 12 patch)

Batched per 42 §11.1. No runtime behaviour change observed: full test
suite, soak, and the P10 performance budgets are within noise. tokio
1.x minor reviewed for the Relay's cancellation semantics; unchanged.

Work-Unit: WU-3.2-HY-0907
Milestone: M3.2
Done: 8/8
Agent: claude-code/hygiene-deps
Model: <model-id>
Task: TSK-hy-20260907
Co-Authored-By: Claude <noreply@anthropic.com>
```

```
revert(vault): revert "feat(vault): pin Shards on first read"

Reverts 4c1f9ab2. Read-triggered pinning caused Custodian selection to
converge on the three lowest-latency Nodes, collapsing replica diversity
from 5 to 3.1 within 6 hours of canary. Attestation economics (17 §4) are
invalid under that distribution. Re-landing requires a selection policy
that is diversity-aware; tracked as WU-3.4-002.

Work-Unit: WU-3.2-026
Milestone: M3.2
Principle: P12
Done: 8/8
Agent: claude-code/agent-03
Model: <model-id>
Task: TSK-11c8de
Co-Authored-By: Claude <noreply@anthropic.com>
```

---

## 5. Hooks and Local Gates

Hooks exist to make the fast, certain failures fail in under five seconds on the agent's own machine, so the merge queue is never spent discovering a missing newline. They are not a substitute for CI (`40`), and anything slow belongs in CI by rule.

### 5.1 What runs where

| Hook | Runs | Budget (p95) | On failure |
|---|---|---|---|
| `pre-commit` | `rustfmt`/`prettier`/`taplo` **auto-fix and re-stage**; secret scan (`gitleaks --staged`); large-file check (>2 MiB, or >256 KiB for text); forbidden-path check (`target/`, `node_modules/`, `*.env`, `dist/`, generated artifacts per `41`); banned-term lint (`01 §10`) | **3 s** | Auto-fixable → fixed silently and staged. Not auto-fixable → block with the exact remediation command. |
| `commit-msg` | Conventional Commit parse; type ∈ §4.2; scope ∈ §4.3; subject ≤ 72, imperative, no trailing period; required trailers present; `Work-Unit` matches branch name; `Milestone` open in `milestones/`; `!` ⇔ `BREAKING CHANGE:` | **300 ms** | Block. Print the offending field and the rule. Never rewrite a message silently — an agent that cannot compose a legal message has a defect in its Work Unit definition, and hiding it is worse than failing. |
| `pre-push` | `cargo check --workspace --all-targets`; `cargo clippy -- -D warnings` on changed crates; unit tests for changed crates; dependency-direction lint (P5); `cargo deny check bans licenses` | **60 s** | Block. |
| `post-merge` / `post-checkout` | Re-install hooks if the manifest changed; prune stale worktrees; warn on drifted toolchain | 1 s | Warn only. |

**The rule:** a hook whose p95 exceeds its budget for one week is moved to CI, not optimized in place indefinitely. Measured by `.git/hooks/.timing` samples aggregated by the weekly hygiene report (§12.2). Local latency is the scarcest resource in an agent loop; a 40-second pre-commit hook multiplied by 300 commits a week is four hours of nothing.

### 5.2 Installation and drift

Hooks are defined in `hooks/` in the repository and installed by `cargo xtask hooks install`, which is run by `cargo xtask bootstrap` and re-checked by `post-checkout`. The installed hooks are thin shims that call `cargo xtask hook <name>`, so hook *logic* is versioned with the code and cannot drift between an agent's checkout and CI. CI runs the identical `xtask` entry points, so "passed locally, failed in CI" for a hook-covered rule is a bug in the shim, not in the developer.

### 5.3 Bypass policy

`--no-verify` is permitted in exactly one circumstance: an incident response commit on a `fix/INC-*` branch, where the hook itself is the thing that is broken. Every other use is a violation. Detection: CI recomputes every hook check server-side on every push; a push whose hook checks fail is rejected regardless of what happened locally. The bypass therefore buys nothing except a faster failure, which is precisely the property that makes it safe to leave available.

---

## 6. Work in Progress Without History Pollution

Agents run long tasks on machines that can die. They need durable checkpoints. Trunk history needs those checkpoints never to exist. Both are satisfiable.

### 6.1 The mechanism

**Checkpoints live on a dedicated ref namespace that is not a branch, is not fetched by default, and is garbage collected on a schedule.**

```
refs/wip/<agent-id>/<work-unit-id>          ← durable, pushed, invisible to `git branch`
```

The agent checkpoints with:

```bash
cargo xtask checkpoint            # equivalent to:
  git add -A
  git commit -q --no-verify -m "checkpoint: $(date -uIs) WU-3.2-014"
  git push -q --force origin HEAD:refs/wip/agent-07/WU-3.2-014
  git reset --soft HEAD~1         # working tree unchanged; no local commit remains
```

Checkpoint commits are unreachable from any branch, do not appear in `git log`, never enter a pull request, and are never merged. They exist to answer exactly one question — "the machine died, where was I?" — and they answer it with `cargo xtask checkpoint restore WU-3.2-014`.

When the Work Unit is complete, the agent produces the real commit from the *working tree*, not from the checkpoint chain:

```bash
git checkout -B wu/3.2-014-ledger-posting-idempotency origin/main
# working tree already carries the finished change
git add -A && git commit           # one commit, full message, all hooks run
```

The checkpoint chain is then deleted. The Work Unit's history is one commit that never contained a broken intermediate state.

### 6.2 Parallel work: worktrees

An agent working more than one Work Unit uses `git worktree`, never branch switching, because switching invalidates the build cache and the incremental compilation state that dominates a Rust edit loop.

```
repo/                       main            (never dirty, never worked in)
repo-wt/3.2-014/            wu/3.2-014      own target/, own worktree
repo-wt/3.2-021/            wu/3.2-021
```

`cargo xtask worktree new WU-3.2-014` creates the worktree, the branch from `origin/main`, and a per-worktree `target/` symlinked into a shared `sccache`. Limit: 3 worktrees per agent (§3.4).

### 6.3 Retention and cleanup

| Object | Retained | Cleaned by | Cadence |
|---|---|---|---|
| `refs/wip/*` | 7 days after last update, or immediately on Work Unit merge | `hygiene/gc` job | Daily 03:00 UTC |
| Worktrees | 24 h after last commit in them | `post-checkout` + daily job | Daily |
| Merged `wu/*` branches | Deleted at merge | Repository setting | Immediate |
| Unmerged `wu/*` branches | 48 h (§3.4); work preserved to `refs/wip/` before deletion | `hygiene/stale-branch` | Daily |
| Objects unreachable after wip GC | 14 days (`gc.pruneExpire`) | `git gc` | Weekly |

Nothing in this table generates a notification. Cleanup that tells you it cleaned up is noise (§12).

**Alternatives rejected.** *Push wip branches to `origin` normally* — rejected: they pollute the branch list, appear in the UI, get accidentally opened as PRs, and make "how many branches are open" meaningless. *Local-only checkpoints (`git stash`, untracked backup dirs)* — rejected: not durable across machine loss, which is the only reason checkpoints exist. *`git rerere` plus a long-lived branch* — solves a different problem.

---

## 7. The Agent Commit Protocol

This is the operational heart of the chapter. It is written as an algorithm because an agent will execute it as one.

### 7.1 Precondition: when an agent MAY commit

An agent commits when, and only when, **all** of the following hold:

1. A Work Unit definition existed **before** the work started, with an ID, a one-sentence subject, acceptance criteria, and a Milestone. (An agent that discovers its Work Unit retroactively has already failed criterion 1 of §2.2.)
2. The acceptance criteria are met verbatim (`00 §5.1`).
3. All eight criteria of `00 §5` hold, or a criterion is explicitly waived with a named human approver recorded in the `Done:` trailer.
4. Local gates (§5) are green.
5. The change is within the current phase's scope (`02 §3`) and inside the complexity budget (`02 §5`).
6. The diff touches ≤ 3 service boundaries and exactly 1 commit scope.
7. No secret, credential, key, `.env`, or generated artifact is staged.

If any fails, the agent does not commit. It either continues working, splits (§7.6), or escalates.

### 7.2 Verification: the exact commands

Run in order; stop at the first failure. These are the same entry points CI runs, so a pass here is predictive rather than hopeful.

```bash
cargo xtask verify --work-unit WU-3.2-014        # the single entry point; expands to:

  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo test  --workspace --all-features
  cargo xtask lint-deps          # P5 dependency-direction lint (10 §3)
  cargo xtask lint-terms         # 01 §10 banned vocabulary in code, docs, and UI strings
  cargo xtask lint-events        # P6 event-schema compatibility (10 §10)
  cargo xtask lint-api           # 30 public-contract diff; classifies additive vs breaking
  cargo xtask budgets            # P10 perf + a11y budgets, 02 §5 complexity budgets
  cargo deny check advisories bans licenses sources
  gitleaks detect --no-git --redact
  cargo xtask done-check --work-unit WU-3.2-014  # asserts 00 §5 items 2,4,5,6,8 mechanically
```

`done-check` is what makes "eight of eight" a machine claim rather than an agent's opinion. It verifies: the capability is reachable from the public API surface and the CLI command table (items 2 and 5, N3, P13); the code path emits at least one domain event and carries a correlation ID (item 4, P6); an API reference entry, a CLI help string, and a changelog-eligible commit exist (item 5); an offline-degradation test and a deny-without-capability test exist for the touched path (item 6, P2/P8); an ADR exists if `Cargo.toml` gained a dependency or a port implementation changed (item 8). Items 1, 3, and 7 are covered by acceptance tests, the test tiers in `40`, and `budgets` respectively.

### 7.3 Composing the message

The message is *derived*, not composed freshly, which is what keeps it truthful:

| Message part | Derived from |
|---|---|
| `type` | The Work Unit's declared kind in the Milestone manifest |
| `scope` | The single owning boundary from `41`'s crate→scope map, computed from the diff |
| `subject` | The Work Unit's one-sentence definition, imperative, ≤ 72 chars |
| `body` | The Work Unit's *rationale* field plus any measurement obtained during the work (benchmark delta, soak result, defect reproduction) |
| `BREAKING CHANGE:` | `lint-api` / `lint-events` classification, plus the migration note the agent must have written to land it |
| `Work-Unit:`, `Milestone:` | The Work Unit record |
| `Principle:` | The Work Unit's declared principle(s); the agent may add, never remove |
| `ADR:` | Detected by `done-check` |
| `Done:` | `done-check` output |
| `Agent:`, `Model:`, `Task:`, `Co-Authored-By:` | The agent runtime's own identity (§14) |

An agent that cannot fill `body` with a *reason* has usually not understood the change. That is a legitimate stop-and-think signal, not a formatting inconvenience.

### 7.4 Grouping

**One Work Unit = one branch = one pull request = one squashed commit on trunk.** Intermediate commits on the `wu/*` branch are allowed and unstructured — they are squashed away and their messages are discarded. The squash commit message is the one composed in §7.3, set as the PR body and used verbatim by the merge queue. Review feedback is folded into the branch, never appended as a trunk commit.

The merge queue takes up to 5 branches per batch, builds and tests the batch as it would land, and on failure bisects the batch to evict the culprit and re-queues the innocent. This is why trunk is green by construction: nothing lands that was not tested against the exact trunk state it lands on.

### 7.5 What an agent must NEVER do

| Never | Because | Enforcement |
|---|---|---|
| Commit code that does not compile or whose tests fail | Destroys bisectability, which is the reason for the whole model | `pre-push` + CI + merge queue |
| Commit a secret, key, token, or `.env` | Irreversible in a distributed VCS; see §15.5 | `pre-commit` gitleaks, server-side push protection |
| Commit build output, `target/`, `node_modules/`, or generated artifacts | Repository bloat; generated files create false conflicts | `pre-commit` forbidden-path check + `.gitignore` |
| Amend, rebase, or rewrite a commit that exists on `main` | Breaks every clone and every published reference | `main` is protected; force-push has no bypass role |
| Force-push to `main` | As above | Repository ruleset |
| Commit "wip", "temp", "fixes", "misc", or "address review comments" to trunk | A commit with no idea in it is not history | `commit-msg` hook rejects by type and by subject pattern |
| Bypass a hook outside `fix/INC-*` | Gates that can be skipped are not gates | Server-side recheck rejects the push |
| Merge its own Work Unit without the full gate set passing | Self-approval with no verification is not review | Branch protection; the merge queue, not the agent, performs the merge |
| Close a Milestone | Requires a human signature (P4, §10.4) | Signed-tag requirement |
| Widen its own permissions, alter branch protection, or edit CI gates as part of a feature Work Unit | P4: an agent may never widen its own envelope | CODEOWNERS on `.github/`, `hooks/`, `xtask/gates/`; human review required |
| Add a `while I'm in here` refactor to a feature commit | `02 §7` | Review + scope lint |
| Cite a Principle it did not actually serve | Corrupts traceability, which is worse than omitting it | Sampled human audit at Milestone close |

### 7.6 When a Work Unit turns out to be too large

Detected when any of: diff > 400 non-generated lines; > 3 service boundaries; > 1 scope; subject needs an "and"; branch age > 24 h; or `done-check` fails on a subset of the change.

```
 1. STOP. Do not commit. Do not keep going to "finish it first".
 2. Checkpoint the whole tree: cargo xtask checkpoint
 3. Decompose on the seam, in this preference order:
      a. port/trait boundary        (lands alone, safest, usually first)
      b. service boundary S1..S14
      c. layer: domain → application → gateway → surface
      d. capability → CLI exposure → GUI exposure   (each independently done)
      e. mechanical rename/format   (always its own unit, always first or last)
 4. Register the new units in the Milestone manifest as WU-<m>-<seq>a, -b, -c,
    preserving the original ID as the parent in the manifest's `split_from`.
 5. Create worktrees; reconstruct unit (a) from the checkpoint using
    `git restore --source=refs/wip/... -- <paths>` into a clean branch.
 6. Verify (§7.2) unit (a) ALONE. It must be green with the rest absent.
    If it is not green alone, the seam was wrong — go back to step 3.
 7. Land (a) through the queue. Rebase the remaining work. Repeat for (b), (c).
 8. Delete the checkpoint ref.
```

Splitting is expected, not exceptional. The manifest records the split so a reviewer at Milestone close sees that WU-3.2-014 became three units for a stated reason, rather than seeing an ID vanish.

### 7.7 Conflicts

```
 Conflict on rebase onto origin/main
        │
        ├─ Mechanical (imports, formatting, lockfile, generated file)
        │     → agent resolves; regenerates rather than hand-merging generated
        │       output; re-runs §7.2 in full; proceeds.
        │
        ├─ Semantic, same boundary, agent understands both sides
        │     → agent resolves, and MUST add a test that fails against either
        │       side alone. Re-run §7.2. Note the resolution in the PR body.
        │
        ├─ Semantic, crosses a boundary or an invariant the agent did not author
        │     → DO NOT resolve. Comment on both Work Units, escalate to the
        │       owning agent or human, and hold. (00 §6: escalation is cheap.)
        │
        └─ Conflict on a Canon file (00/01/02), an ADR, or a Charter schema
              → NEVER auto-resolve. Human decision, by rule.
```

A branch that conflicts twice in one day is evidence that trunk is moving under it faster than it is being finished — which means the Work Unit is too large. It goes to §7.6.

### 7.8 The protocol as a flowchart

```
              ┌────────────────────────────────────┐
              │  Work Unit assigned from Milestone │
              │  manifest (ID, subject, criteria)  │
              └──────────────────┬─────────────────┘
                                 ▼
                    ┌────────────────────────┐
                    │ worktree + branch from │
                    │ origin/main            │
                    └───────────┬────────────┘
                                ▼
                   ╔════════════════════════╗
              ┌───►║  WORK  (Changes, N=40+)║◄──────────────┐
              │    ╚════════════╤═══════════╝               │
              │                 ▼                           │
              │      ┌──────────────────────┐   every       │
              │      │ cargo xtask checkpoint│  ~15 min or   │
              │      │  → refs/wip/… (§6)   │   milestone   │
              │      └──────────┬───────────┘   of thought  │
              │                 ▼                           │
              │        ┌─────────────────┐                  │
              │        │ Unit complete?  │──── no ──────────┘
              │        └────────┬────────┘
              │                yes
              │                 ▼
              │      ┌────────────────────────┐
              │      │ TOO LARGE? (§7.6 test) │── yes ──► SPLIT (§7.6) ──┐
              │      └───────────┬────────────┘                          │
              │                  no                                      │
              │                  ▼                                       │
              │      ┌────────────────────────┐                          │
              │      │ cargo xtask verify     │                          │
              │      │ (§7.2, all 11 checks)  │                          │
              │      └───────────┬────────────┘                          │
              │            ┌─────┴─────┐                                 │
              │          fail        pass                                │
              └────────────┘           ▼                                 │
                              ┌────────────────────┐                     │
                              │ done-check = 8/8 ? │                     │
                              └─────────┬──────────┘                     │
                            no ─────────┤                                │
                    ┌───────────────────┘                                │
                    ▼                  yes                               │
          ┌──────────────────┐          ▼                                │
          │ waiver approved  │  ┌────────────────────┐                   │
          │ by a human?      │─►│ compose message    │◄──────────────────┘
          └────────┬─────────┘  │ from WU definition │
                   no           │ (§7.3)             │
                   ▼            └─────────┬──────────┘
             ┌──────────┐                 ▼
             │ ESCALATE │       ┌────────────────────┐
             │  (human) │       │ commit → push →    │
             └──────────┘       │ open PR (squash)   │
                                └─────────┬──────────┘
                                          ▼
                                ┌────────────────────┐
                                │  CI full suite     │──fail──► fix on branch
                                │  (40) + review     │          (loop to WORK)
                                └─────────┬──────────┘
                                          ▼
                                ┌────────────────────┐
                                │   MERGE QUEUE      │──batch fail──► bisect,
                                │   (batch ≤ 5)      │   evict culprit, requeue
                                └─────────┬──────────┘
                                          ▼
                              ┌──────────────────────────┐
                              │ squash-merge to main     │
                              │ delete branch + wip ref  │
                              │ changelog regenerated    │
                              │ manifest WU marked done  │
                              └──────────────────────────┘
```

---

## 8. Changelog Generation

### 8.1 Two artefacts, two audiences

| Artefact | Granularity | Generated by | Audience | Human involvement |
|---|---|---|---|---|
| `CHANGELOG.md` | Every user-visible Work Unit | `git-cliff` from Conventional Commits, on every merge to `main` | Operators, integrators, `git archaeology` | **None.** Fully mechanical. A human editing this file by hand is a defect. |
| `docs/releases/M<x>.<y>.md` | Per Milestone | Agent-drafted from the same commit range, then human-approved | Citizens, prospective users, the marketing surface | **Approval required.** The narrative is a claim about the product; a machine may draft it, a human signs it. |

The split exists because the two documents answer different questions. `CHANGELOG.md` answers "what changed, precisely, and where do I find it". Release notes answer "why should you care". Generating the second mechanically produces the bullet-list-of-commit-subjects that nobody reads; writing the first by hand produces drift.

### 8.2 Tooling

**`git-cliff`**, configured in `cliff.toml`, invoked by `cargo xtask changelog`.

*Why:* it is a single static binary, it consumes Conventional Commits natively, its templates are data rather than code, it runs identically in CI and locally, and it adds no runtime dependency to the workspace. *Honest cost:* the template language is another small DSL to maintain, and grouping rules live outside the Rust type system. *Alternatives rejected:* `release-please` (opinionated about release PRs and versioning in ways that conflict with §9's lockstep decision, and it wants to own the release commit); `changesets` (excellent for JS monorepos with independent package versions, but it requires a human-authored changeset file per change — which is exactly the busywork this chapter exists to eliminate, and it duplicates information already present in the commit); `conventional-changelog` (Node toolchain in a Rust-first repo); hand-written (drifts within two weeks, always).

### 8.3 Mapping

```toml
# cliff.toml (excerpt)
[git]
conventional_commits = true
filter_unconventional = false      # unconventional commits are an ERROR, not a silent drop
protect_breaking_commits = true

commit_parsers = [
  { message = "^feat",     group = "<!-- 01 -->Added" },
  { message = "^api",      group = "<!-- 02 -->API" },
  { message = "^event",    group = "<!-- 03 -->Domain Events" },
  { message = "^fix",      group = "<!-- 04 -->Fixed" },
  { message = "^sec",      group = "<!-- 05 -->Security" },
  { message = "^perf",     group = "<!-- 06 -->Performance" },
  { message = "^deps",     group = "<!-- 07 -->Dependencies" },
  { message = "^revert",   group = "<!-- 08 -->Reverted" },
  { message = "^(refactor|test|build|ci|chore|style|docs|adr)", skip = true },
]
```

Every entry renders as: subject, scope, short SHA, Work Unit ID, and `BREAKING` marker where applicable.

```markdown
## [0.4.0] — 2026-09-07

### Added
- **ledger:** record Postings through a balanced double-entry writer (`4c1f9ab`, WU-3.2-014)
- **cli:** `fn wallet transfer` with confirmation and idempotency key (`77e0b31`, WU-3.2-017)

### API
- **BREAKING — agent:** require an explicit `envelope_ref` on every agent-issued command (`9ab3c02`, WU-3.3-004) — migration: `docs/migrations/0031.md`

### Fixed
- **relay:** stop leaking Signal subscriptions on abrupt socket close (`b2f4d18`, WU-3.2-021)
```

### 8.4 Changes with no user-facing impact

`refactor`, `test`, `build`, `ci`, `chore`, `style`, `docs`, and `adr` are excluded from `CHANGELOG.md` — but they are **not lost**. They remain in `git log`, they remain attached to their Work Unit and Milestone, and they appear in two places a human can reach:

1. **The Milestone manifest's completion record** (§10.3), which lists every Work Unit closed under the Milestone regardless of type.
2. **`docs/releases/M<x>.<y>.md`**, whose "Under the hood" section is drafted from exactly these excluded types and whose "Decisions" section lists the `adr` commits.

The rule is: *nothing is deleted from history to keep the changelog clean; it is routed to the document whose readers care.* An agent that suppresses a change rather than routing it has broken traceability (§13).

### 8.5 Deriving user-facing text

The changelog line is the commit subject, unchanged. This is deliberate and it is why §4.1 requires the subject to be written in canonical vocabulary (`01`) and in imperative mood: the subject is a published artefact from the moment it is typed. An agent that writes `fix(relay): fix the thing` has written a changelog line that says "fix the thing", and `commit-msg` rejects vacuous subjects against a stop-list (`fix`, `update`, `changes`, `stuff`, `various`, `the thing`, `it`).

---

## 9. Versioning and Releases

### 9.1 What is versioned, and how

| Surface | Scheme | Independent or lockstep | Rationale |
|---|---|---|---|
| The workspace (all `fractal-*` crates) | SemVer, `0.y.z` until PH5 | **Lockstep** — one version for every internal crate | They are one deployable Runtime with one release cadence and internal-only consumers. Independent versioning of crates that always ship together produces a version matrix that describes nothing and costs a release-engineering system to maintain. |
| Public API (`30`) | Path-versioned `/v1`, plus a dated contract revision `v1.2026-09-07` | **Independent** of the workspace | Third parties depend on the contract, not on our binary. The contract may be stable across many Runtime releases; that stability is the product. |
| SDKs (Rust, TS, Python) | SemVer, published | **Independent per SDK** | They have their own consumers, their own ecosystems, and their own breaking-change semantics. An SDK patch must be shippable without a Runtime release. |
| Extensions (`20`) | SemVer, declared in the Extension manifest | **Independent per Extension** | Authored by third parties; we do not control their cadence. |
| Facet Standard (`FN-ASSET/1`) | Major-only, `FN-ASSET/1`, `/2` | Independent | A standard with minor versions is a standard nobody can implement. |
| Charter schema (`9`, S9) | SemVer with a migration per major | Independent | Governance documents outlive releases. |

Pre-1.0 (through PH4) the workspace uses `0.y.z` where a breaking change bumps `y`. The `1.0.0` tag is a deliberate, human decision at the PH5 gate, tied to the first release running on Nodes we do not control.

### 9.2 Tags

```
v0.4.0            workspace release (annotated, GPG-signed, human)
sdk-ts-v0.9.2     per-SDK release
api-v1.2026-09-07 API contract revision
M3.2              milestone close marker (annotated, signed, human)
```

### 9.3 The release pipeline

```
 TRIGGER  ── human runs `cargo xtask release --minor` OR a Milestone closes
    │        (never time-based; never on every merge)
    ▼
 ┌─────────────────────────────────────────────────────────────────────┐
 │ 1. PRECHECK   trunk green · zero open ADRs for the milestone        │  auto
 │               (02 §5) · budgets green · no open sec advisories      │
 │ 2. COMPUTE    semver bump from commit types since last tag (§4.2)   │  auto
 │ 3. BUMP       workspace version, lockfile, SDK manifests            │  auto
 │ 4. CHANGELOG  git-cliff regenerates CHANGELOG.md for the range      │  auto
 │ 5. NOTES      agent drafts docs/releases/M3.2.md                    │  auto-draft
 │ 6. COMMIT     chore(repo): release v0.4.0   (the only chore that    │  auto
 │               may touch versions)                                   │
 ├─────────────────────────────────────────────────────────────────────┤
 │ 7. SIGN-OFF   HUMAN reviews notes + diff, signs the tag:            │  ◄── HUMAN
 │               git tag -s v0.4.0 -m "…"                              │
 ├─────────────────────────────────────────────────────────────────────┤
 │ 8. BUILD      reproducible builds: linux/x86_64, linux/aarch64,     │  auto
 │               windows/x86_64, macOS universal, wasm32 (N2)          │
 │ 9. ATTEST     SBOM (CycloneDX) · provenance (SLSA L3) · cosign      │  auto
 │               signatures over every artefact and the SBOM (P8)      │
 │10. PUBLISH    crates.io (SDKs) · npm (TS SDK) · release assets ·    │  auto
 │               container image · desktop installers                  │
 │11. ANNOUNCE   release notes published; Signal to operators          │  auto
 │12. CANARY     10% of hosted Runtime for 60 min, budgets watched     │  auto
 │13. PROMOTE    100%                                                  │  auto if canary green
 └─────────────────────────────────────────────────────────────────────┘
```

### 9.4 Rollback

```
 Severity 1 (data loss, security, ledger integrity):
   1. Promote the previous release (single command, ≤ 5 min, no build).
   2. Freeze the merge queue.
   3. Open INC-#### ; the Work Unit that caused it is identified by bisect
      over the tag range — cheap, because every commit on main is green.
   4. `git revert` the offending Work Unit on trunk. NEVER rewrite history
      to remove it: the revert is itself a fact worth recording (P6 thinking
      applied to the repository).
   5. Ship the revert as a patch release with the incident referenced.

 Severity 2 (degraded, no integrity risk):
   Roll forward. A fix release within 24h beats a rollback that undoes
   unrelated correct work.

 Ledger-affecting releases are never rolled back by redeploying an older
 binary alone: the Postings written under the new version are facts. The
 rollback procedure for S5 is defined in 16 and requires a compensating
 Posting set, not a deployment change.
```

### 9.5 Automated vs human

**Commits, merges, changelogs, version computation, builds, SBOMs, and publishing are fully automated. Two things require a human signature and always will:**

1. **Closing a Milestone** (§10.4) — because it asserts that acceptance criteria are met, which is a judgement about the product.
2. **Signing a release tag** — because it is a claim to the outside world, made under our name, about software running on other people's machines.

This is P4 applied to the repository: agents execute, humans set policy and take responsibility for what leaves the building. The user is not the commit button; the user is the signature.

---

## 10. Milestone Gates

### 10.1 A Milestone is a file

Milestones are declared as manifests in `milestones/`, not as issue-tracker state. Rationale: the manifest is versioned with the code, reviewable in a diff, readable offline (P2 applied to our own process), and queryable by the same tools that read the repository. Issue trackers were rejected as the source of truth because their state is invisible to `git bisect`, invisible to an agent working offline, and lost if the tracker changes.

### 10.2 Manifest format

```yaml
# milestones/M3.2.yaml
id: M3.2
phase: PH3
title: Ledger writes are balanced, replayable, and idempotent
opened: 2026-08-24
status: open                # open | gated | closed
owner: "@andrew"            # the human who signs the close

serves:
  principles: [P11, P12, P6]
  non_negotiables: [N4]
  spine: "…earn Fraction in it…"      # 02 §2

acceptance_criteria:
  - id: AC1
    text: Every Fraction movement is expressible as a balanced Posting pair.
    verified_by: cargo test -p fractal-domain-ledger --test balance_invariants
  - id: AC2
    text: Replaying the Log reproduces every Wallet balance exactly.
    verified_by: cargo xtask replay-check --societies 100
  - id: AC3
    text: A duplicate Transfer command produces exactly one Posting pair.
    verified_by: cargo test -p fractal-app --test idempotency

budgets:                    # 02 §5, evaluated at close
  new_services: 0
  new_api_resource_families: 1
  new_runtime_dependencies: 2
  new_economic_sources: 0

work_units:
  - id: WU-3.2-014
    kind: feat
    scope: ledger
    subject: record Postings through a balanced double-entry writer
    principles: [P11, P6]
    status: merged
    commit: 4c1f9ab2
  - id: WU-3.2-021
    kind: fix
    scope: relay
    subject: stop leaking Signal subscriptions on abrupt socket close
    principles: [P10]
    status: merged
    commit: b2f4d18e
  - id: WU-3.2-026
    kind: revert
    scope: vault
    subject: revert read-triggered Shard pinning
    split_from: null
    status: merged
    commit: 0f19d7c4

adrs:
  required: [0031]
  open: []                  # MUST be empty to close — 02 §5

exit:
  changelog_range: v0.3.0..HEAD
  release_notes: docs/releases/M3.2.md
  docs_updated: [16-ledger-and-assets.md, 30-api-and-sdk.md, 31-cli-and-terminal.md]
```

### 10.3 The close check

`cargo xtask milestone check M3.2` — run on every merge that touches the manifest, nightly, and as a required check on the close PR:

| # | Check | Fails when |
|---|---|---|
| 1 | Every acceptance criterion's `verified_by` command exists and passes | any failure |
| 2 | Every `work_units[].status` is `merged` or `dropped` (with a reason) | any `open` |
| 3 | Every merged Work Unit's commit exists on `main` and its trailers match the manifest | drift between manifest and history |
| 4 | `adrs.open` is empty and every `adrs.required` file exists with status `accepted` | `02 §5` violation |
| 5 | Complexity budgets computed from the diff range ≤ declared budgets | overrun |
| 6 | P10 performance and accessibility budgets green at `HEAD` | regression |
| 7 | Every doc in `exit.docs_updated` has a commit in the range | documentation drift (§11.5) |
| 8 | `CHANGELOG.md` regenerates byte-identically from the range | changelog drift |
| 9 | Release notes draft exists and is non-empty | missing narrative |
| 10 | Zero open `sec` advisories; zero `TODO(WU-*)` referencing this Milestone | leftover work |

All ten green ⇒ status may move to `gated`. Nothing moves it to `closed` except §10.4.

### 10.4 Closing

A Milestone closes when a human, named in `owner`, signs an annotated tag:

```bash
git tag -s M3.2 -m "Milestone M3.2 closed: ledger writes balanced, replayable, idempotent"
```

The signature is the audit anchor for `00 §5` and for P4: it is the human approval that every Work Unit under the Milestone inherits (§14). An agent may prepare everything — the manifest, the checks, the notes, the tag message — and must not create the tag.

---

## 11. Repository Hygiene Automation

Every job below states its trigger, its action, its output, and its **noise budget**: the maximum number of items per period that may require a human to look at anything. A job that exceeds its budget twice in a rolling four weeks is automatically disabled and must be re-justified before re-enabling (§12.3).

| # | Job | Trigger | Agent action | Output | Noise budget |
|---|---|---|---|---|---|
| 1 | **Dependency update** | Mon 06:00 UTC | Resolve all compatible updates, batch into **one** Work Unit, run full suite + soak + budgets, read changelogs for behavioural notes, write the body | 1 PR/week, auto-merged if all gates green | **0** notifications on success; 1 on failure |
| 2 | **Security advisory** | On advisory publication (`cargo audit`/OSV) | Patch immediately as a `sec` Work Unit; if no patch exists, apply mitigation or open a blocking issue | PR immediately; page a human only for CVSS ≥ 7.0 with no fix | ≤ 1/month expected |
| 3 | **Stale branch cleanup** | Daily 03:00 UTC | Preserve unmerged work to `refs/wip/`, delete `wu/*` > 48 h, prune merged refs, prune worktrees | Nothing | **0** |
| 4 | **wip GC** | Daily 03:10 UTC | Delete `refs/wip/*` untouched > 7 days; `git gc` weekly | Nothing | **0** |
| 5 | **Dead code detection** | Weekly, Wed | `cargo udeps`, `cargo machete`, unreachable-symbol scan, unused feature flags. Delete what is provably dead and covered by tests | 1 PR/week if anything found; nothing if not | 0 |
| 6 | **TODO/FIXME aging** | Weekly | Every `TODO`/`FIXME` must carry `TODO(WU-3.4-002):`. Unattributed ones become a PR that attributes or deletes them. At 60 days, escalate to a Work Unit in the current Milestone; at 90 days, block the Milestone close | 1 aggregated report/month; 0 PRs unless unattributed | ≤ 1/month |
| 7 | **Documentation drift** | On merge | Compare touched code paths to the doc map in `41`. If a public API, CLI verb, event schema, or design token changed and the owning doc did not, the PR fails the `done-check` — *before merge*, not after | Blocks the PR; never a follow-up issue | 0 (it is a gate, not a notification) |
| 8 | **Test flakiness triage** | Nightly + on CI failure | 200 repeats of failing tests; ≥ 1 flake ⇒ quarantine with `#[ignore = "flaky: FLK-####"]`, open a Work Unit in the current Milestone, and record the flake rate. Quarantine expires in 14 days and then blocks the Milestone | 1 Work Unit per flake; **0** human notifications | 0 |
| 9 | **License / SBOM refresh** | Weekly + every release | Regenerate CycloneDX SBOM; `cargo deny check licenses`. New license outside the allowlist blocks the dependency PR | Blocks; nothing otherwise | 0 |
| 10 | **Large-file / repo weight** | Pre-commit + weekly | Reject > 2 MiB (256 KiB for text) at commit; weekly report of pack growth; propose LFS or removal if the repo grows > 5%/month | 1 report/month only if growth exceeds threshold | ≤ 1/month |
| 11 | **Lint-rule adoption** | Monthly | Evaluate new `clippy` lints and rustc warnings; enable one batch, auto-fix, land as a `style`/`refactor` Work Unit, add to `.git-blame-ignore-revs` if mass-reformatting | 1 PR/month | 0 |
| 12 | **Merge-queue health** | Continuous | Track queue latency, eviction rate, batch-failure rate | Page only if p95 queue time > 45 min or eviction rate > 20%/day | ≤ 1/month |
| 13 | **Traceability index** | On merge | Rebuild `traceability.jsonl` (§13.2) | Nothing | 0 |
| 14 | **Hygiene budget report** | Fri 16:00 UTC | Count every human-facing notification produced this week per job; compare to budget | **The one weekly digest a human reads** | 1/week |

### 11.1 Dependency update policy

Batched **weekly**, never per dependency. One PR, one Work Unit, one revert target.

| Class | Policy |
|---|---|
| Patch, no behavioural note | Auto-merge on green |
| Minor, no behavioural note | Auto-merge on green |
| Minor touching a P5 port implementation, crypto, the Ledger, or the Relay | Human review required |
| Major | Its own Work Unit, never batched, ADR if it changes a technology choice (`00 §3`) |
| New dependency | Counts against `02 §5`'s five-per-phase budget; requires an ADR naming the principle served and the exit cost |
| Security | Out of band, immediately (job 2) |

Everything is pinned; `Cargo.lock` and `pnpm-lock.yaml` are committed; `cargo vendor` is available for the supply-chain-critical set (P8).

---

## 12. The Anti-Busywork Rules

### 12.1 The rules

1. **No pull request for a formatting-only change.** The pre-commit hook fixes formatting and re-stages. If formatting reached CI, the hook is broken — fix the hook.
2. **No issue for something an agent can just fix.** If an agent can produce the fix and the gates can verify it, the correct output is a merged Work Unit, not a ticket describing one.
3. **Batch dependency updates weekly, never per dependency.** Fourteen PRs is fourteen review contexts for one decision.
4. **No bot comment that a human must read and dismiss.** A bot either blocks (it is a gate) or is silent (it is not needed). "FYI" comments are banned.
5. **No required human action that a gate could decide.** If a rule is expressible as a check, it is a check. If it is not expressible, it is a judgement and belongs in a Milestone signature, not in a per-PR prompt.
6. **No status-update automation.** Progress is derivable from the manifest and the history; a bot that posts "3 of 12 Work Units complete" is generating reading.
7. **No approval theatre.** Auto-mergeable classes (`deps`, `chore`, `docs`, `test`, `ci` with all gates green) require zero approvals. Requiring a rubber stamp trains humans to rubber-stamp the things that matter.
8. **One digest, one day.** All non-blocking automated output aggregates into the Friday report (job 14).

### 12.2 The measured budget

> **A human faces at most 5 automated notifications per week, and at most 1 automated decision per weekday.**

Counted as: anything that sends a push, an email, a mention, or a request for review to a human principal. Blocking gates on a PR a human already opened are *not* counted — they are feedback on work in progress. The weekly hygiene report is 1 of the 5 by construction, so the automation has 4 remaining.

| Metric | Target | Source |
|---|---|---|
| Human-facing notifications / week | ≤ 5 | job 14 |
| Human decisions required / week | ≤ 5 | job 14 |
| Human-authored commits / week | ≤ 2 (Canon, ADRs, release notes edits) | `git log --author` |
| Merge queue p95 latency | ≤ 45 min | job 12 |
| Trunk red time | ≤ 30 min/month | CI |
| PRs auto-merged without human touch | ≥ 85% | CI |
| Median Work Unit branch lifetime | ≤ 6 h | job 12 |

### 12.3 The hard rule

**A hygiene job that produces noise gets tuned or deleted.** Concretely: a job exceeding its stated noise budget in two of any four consecutive weeks is disabled automatically by job 14, and re-enabling requires a PR that changes either its trigger, its threshold, or its output — a PR that merely re-enables it is rejected. There is no "we'll get used to it" state. Alert fatigue is how a team stops reading the alert that mattered, and a system that trains its operator to ignore it is worse than no system.

---

## 13. Traceability

### 13.1 The chain

```
  line of code
      │  git blame -w -C -C --ignore-revs-file .git-blame-ignore-revs
      ▼
  commit ────────── Work-Unit: WU-3.2-014
      │                      │
      │                      ▼
      │             milestones/M3.2.yaml ── work_units[WU-3.2-014]
      │                      │
      │                      ├── serves.principles: [P11, P12, P6]
      │                      ├── serves.non_negotiables: [N4]
      │                      ├── acceptance_criteria: AC1..AC3
      │                      ├── phase: PH3
      │                      └── owner: @andrew  (signed tag M3.2)
      ▼                                 │
  ADR: 0031 ──► docs/adr/0031-*.md      ▼
      │                        00-foundational-principles.md §1 (P11)
      ▼                        02-scope-guardrails.md §3 (phase placement)
  Agent: / Model: / Task:  ──► agent run record (§14)
```

Every arrow is a lookup on a literal string that CI has already validated. Nothing in the chain requires interpretation.

### 13.2 Making it queryable

`cargo xtask trace` reads a generated `traceability.jsonl` (rebuilt on every merge, job 13) with one record per Work Unit joining commit SHA, files touched, scope, principles, ADRs, Milestone, phase, agent, model, task, and the approving human.

```bash
cargo xtask trace --file crates/fractal-domain-ledger/src/posting.rs --line 88
#  WU-3.2-014 · feat(ledger) · M3.2 · PH3 · P11,P12,P6 · ADR-0031
#  4c1f9ab2 · agent-07 · TSK-9f2c1a · approved-by @andrew (tag M3.2)

cargo xtask trace --principle P12          # every Work Unit that claims economic honesty
cargo xtask trace --milestone M3.2 --kind revert
cargo xtask trace --agent agent-07 --since 2026-08-01
cargo xtask trace --unattributed           # code with no principle citation — should be empty
```

`--unattributed` is the falsification test for this section: if it returns rows outside `chore`/`style` commits, traceability has holes.

### 13.3 `git blame` ergonomics

Mass mechanical changes destroy `blame` unless they are declared. Policy:

- A mechanical change (reformat, rename, lint-fix sweep) is **always its own commit**, type `style` or `refactor`, containing nothing else.
- Its SHA is appended to `.git-blame-ignore-revs` in the **same** Work Unit, with a comment naming the change.
- `blame.ignoreRevsFile` is set by `cargo xtask bootstrap`, and CI verifies that every `style` commit on trunk appears in the file.
- Reviewers and agents use `git blame -w -C -C` (ignore whitespace, detect moves within and across files) as the default; `xtask` wraps it as `cargo xtask blame`.

```
# .git-blame-ignore-revs
# Adopt rustfmt 2027 edition style across the workspace — WU-3.2-031
a91d0c7e5f3b2841c6d90ab4e7f1c2d3e4b5a697
# Rename EventStore::append → append_batch (31 call sites) — WU-3.3-002
c3f78b1d0e2a49576b8c1d0e2f3a4b5c6d7e8f90
```

---

## 14. Auditability of Agent Work

Fractal Node is built substantially by agents and is *about* governed agency (P4). A repository whose provenance cannot be reconstructed would contradict the product it produces. Concretely, three questions must be answerable for any line in the tree, years later:

**Who wrote it, under what model, on whose authority?**

| Recorded | Where | Why it matters |
|---|---|---|
| `Agent:` — the agent identity (`claude-code/agent-07`) | Commit trailer | Correlates a defect class to a specific configuration or prompt lineage |
| `Model:` — exact model identifier and version | Commit trailer | Model behaviour changes; when a regression class appears, the first question is which model produced the cohort |
| `Task:` — the task/run ID (`TSK-9f2c1a`) | Commit trailer, joined to the run record | Recovers the full prompt, tool calls, and reasoning trail for the change |
| `Co-Authored-By:` | Commit trailer | Machine-readable authorship in every git host and analysis tool |
| Human approver | The signed `M<x>.<y>` tag, inherited by every Work Unit in the manifest | P4: authority terminates in a human signature, exactly as an Envelope grant must (`00 §1 P4` falsification test) |
| Run record | `provenance/<task-id>.json`, retained 24 months, containing prompt hash, tool inventory, model, agent version, timestamps | Reproducibility and incident forensics |

The run record stores a **hash** of the prompt plus a redacted summary, not the raw prompt, because prompts routinely contain repository content and occasionally contain operator context that P9 says we do not retain by default.

**Why this matters here specifically.** Three reasons, all practical. First, when a defect pattern emerges — say a whole class of missing offline paths — the fastest remediation is `cargo xtask trace --agent X --since Y` and a targeted sweep, which is only possible if the cohort is identifiable. Second, licensing and provenance questions about AI-authored code are live, and a project that can produce a signed, per-commit provenance chain answers them in minutes rather than by audit. Third, it is the internal falsification test for P4: if a capability landed in this repository whose authority chain terminates in an agent rather than in a human's signed Milestone tag, we have built the thing we said we would not build.

---

## 15. Failure Modes and Incident Procedures

Automation does not remove failure; it changes who notices and how fast. Each failure below has a detection mechanism, a procedure, and a systemic fix, because an incident that produces only a fix and no systemic change will recur.

### 15.1 An agent commits a regression that gates did not catch

**Detection:** canary budget breach (§9.3 step 12), a telemetry alert, or a later Work Unit failing on trunk.

**Procedure.** Freeze the merge queue. `git bisect` the tag range — cheap and reliable precisely because every commit on `main` is green and squashed, which is the whole return on the Work Unit model. Identify the Work Unit. `git revert <sha>` as its own Work Unit, citing the incident ID; do not rewrite history. Ship as a patch release if the regression reached users. **Systemic fix, mandatory:** the revert Work Unit must add the test that would have caught it, or state in its body why no test could have. A revert with neither is not closed.

### 15.2 The changelog is wrong

**Detection:** `milestone check` item 8 (`CHANGELOG.md` must regenerate byte-identically), or a human reading the release notes.

**Cause is always upstream.** The changelog is a pure function of commit messages; a wrong changelog means a wrong commit message. **Procedure:** never hand-edit `CHANGELOG.md`. If the commit is unmerged, fix the message on the branch. If it is on trunk, add a `docs(repo):` commit carrying a `Changelog-Amend: <sha>` trailer that `cliff.toml` renders as a correction line in the same release section — a correction, not a silent overwrite, because the wrong line may already be published. **Systemic fix:** if the subject was vacuous, add the offending phrase to the `commit-msg` stop-list (§8.5).

### 15.3 A Milestone gate falsely passes

The most dangerous failure in the chapter, because it is the one that converts "eight of eight" from a fact into a habit.

**Detection:** quarterly, and at every phase gate, a human audits a random sample of **5 Work Units per Milestone** against `00 §5`, item by item. This sampled audit is the only recurring human obligation this chapter creates beyond signatures and the Friday report, and it is deliberate: a fully self-certifying system certifies itself.

**Procedure:** if a sampled unit fails, the Milestone reopens; every Work Unit in it is re-checked against the failed criterion specifically; the check that should have caught it is strengthened; and `done-check` gains a case. **Systemic fix:** any criterion that fails sampling twice moves from `done-check` heuristic to a hard CI gate with an explicit test, or the criterion's wording is fixed because it was not mechanically checkable in the first place.

### 15.4 History genuinely needs rewriting

Only three causes qualify: a committed secret (§15.5), a file that must be removed for legal reasons, or repository corruption. "The history is ugly" never qualifies.

**Procedure.** Human decision, no exceptions. Freeze the queue and announce. Rewrite with `git filter-repo` on a mirror. Verify every tag and every `Work-Unit` trailer survives, and re-verify `traceability.jsonl` rebuilds identically. Force-update with a written record in `docs/incidents/`, publish the SHA mapping, and require every clone to re-clone — do not attempt to rebase existing work across a rewrite. Signed tags are re-signed by the human owner; unsigned rewritten history is not accepted.

### 15.5 A secret is committed

Assume compromise from the moment the object exists, not from the moment it is noticed.

```
 T+0    Detection: pre-commit gitleaks (blocked, done) OR server-side push
        protection OR the weekly scan OR the provider's own leak alert.
 T+1m   REVOKE AND ROTATE FIRST. The credential is burned. Removing it from
        history is cleanup, not remediation, and doing it first wastes the
        only minutes that matter.
 T+5m   Open INC-####. Freeze the merge queue.
 T+15m  Determine exposure: was the commit pushed? Was the repo public?
        Any fork, mirror, CI cache, or artefact carrying it?
 T+1h   Purge with git filter-repo (§15.4). Invalidate CI caches and any
        build artefact that embedded it.
 T+24h  Post-incident: how did it enter? Add the pattern to gitleaks rules.
        If a hook was bypassed, the bypass path is closed.
```

**Systemic fix:** every secret incident ends with either a new detection rule or a structural change that made the secret unnecessary (a port, a runtime-injected credential, a short-lived token). `10 §10` already forbids secrets in events, logs, and telemetry; this extends the same rule to the repository, and the answer is always the same — the secret should not have been a file.

### 15.6 The merge queue jams

**Symptoms:** p95 queue time > 45 min, or eviction rate > 20%/day (job 12).

**Procedure.** Reduce batch size from 5 to 1 to isolate a systematically failing branch. If trunk itself is red, `git revert` the last landed Work Unit immediately rather than diagnosing — trunk red time has a 30 minute/month budget and diagnosis happens on a branch. If the cause is flaky tests, quarantine per job 8. If the cause is capacity, the queue is under-provisioned and that is a spend decision, not an engineering one. **Never** the answer: disabling required checks to drain the queue. A drained queue with the gates off is a trunk with no guarantee, and every downstream mechanism in this chapter assumes trunk is green.

### 15.7 An agent produces a plausible but false commit message

Trailers that lie (a `Principle:` not served, a `Done: 8/8` that is not) are the corrosive failure because they are invisible to every mechanical check. Mitigations: `done-check` mechanizes five of the eight criteria so the claim is mostly machine-verified; the sampled audit (§15.3) covers the remainder; and `cargo xtask trace --principle P12` lets a reviewer read every claim of a given principle in one pass, where a false claim reads as obviously out of place. The residual risk is accepted and stated rather than pretended away.

---

## 16. Trade-offs and Rejected Alternatives

| Alternative | Honest appeal | Why rejected |
|---|---|---|
| **Manual commits by a human** | Perfect judgement per commit; no protocol needed | It makes the human the throughput ceiling for a system explicitly designed to have agents as the throughput. It is also the thing the user named as unacceptable. It does not scale past one agent, and the judgement advantage is largely recoverable by mechanizing `00 §5`. |
| **One commit per agent session** | Trivial rule; no splitting logic | Session boundaries are an artefact of machine scheduling, not of meaning. A session may contain three ideas or one third of an idea. Bisect and revert granularity become random. |
| **Auto-commit on every save** | Perfect recovery; zero agent effort | Destroys bisect (most commits do not compile), destroys review, destroys revert, and makes `git log` a keystroke log. The recovery benefit is fully captured by wip checkpoint refs (§6) at zero cost to history. |
| **No branches, commit straight to trunk** | Simplest possible model; no queue to operate | Removes the only place work is verified against *current* trunk, and removes the PR where rationale is recorded. Viable for one agent; incoherent for forty concurrent Work Units. |
| **Merge commits instead of squash** | Preserves the agent's intermediate steps | Those steps are noise by construction (§6). Merge commits make `main` non-linear, so bisect traverses combinations that were never tested. The intermediate detail is preserved in the PR and in the wip ref for the retention window, which is where it belongs. |
| **`release-please`** | Mature, automates release PRs and versioning | It wants to own versioning and the release commit, which conflicts with the lockstep decision (§9.1) and with a human-signed tag being the release authority (§9.5). Fighting an opinionated tool costs more than templating `git-cliff`. |
| **`changesets`** | Best-in-class for independently versioned packages; excellent human release notes | Requires a human- or agent-authored changeset file per change — literally the busywork rule 2 forbids — and it duplicates data already in the commit. It would be the right choice if SDKs dominated the repository; they do not. |
| **Independent per-crate versioning** | Precise dependency expression; consumers upgrade granularly | The `fractal-*` crates ship together, always, to one Runtime. Independent versions of always-co-released crates encode no information and cost a release-engineering system. Revisit only if internal crates gain external consumers. |
| **Issue tracker as the milestone source of truth** | Familiar; good UI; free notifications | Invisible to `git bisect`, invisible offline, lossy across tool migrations, and it splits the definition of done between two systems. The manifest is in the repo; the tracker may mirror it, never own it. |
| **Stacked diffs** | Genuinely better for dependent change series | See §3.2. Adds a required external service, and agents produce mostly independent units. Revisit at PH5 if dependency depth rises. |
| **Signed commits (every commit)** | Strong per-commit provenance | Key management for ephemeral agent identities is a real operational cost, and the marginal assurance over a signed Milestone tag plus per-commit provenance trailers is small. **Decision: signed tags now; signed commits reconsidered at PH5** when self-hosted Nodes make supply-chain provenance externally consequential. |
| **Monorepo with per-directory CI only** | Faster CI | The dependency-direction lint (P5) and the API/event compatibility checks are inherently whole-workspace. Partial CI would let a cross-boundary violation land. `40` handles CI cost with caching and test tiers instead. |

---

## 17. What Would Make Us Change This

Stated in advance so the signal is recognized rather than rationalized (`10 §12` applies the same discipline to the architecture).

- **Merge queue p95 exceeds 45 minutes with correct provisioning.** The batch model has hit its limit. → Move to per-boundary queues keyed on the scope vocabulary (§4.3), which is already a partition of the codebase.
- **Median Work Unit dependency depth exceeds 2.** Agents have started producing genuinely stacked work. → Adopt stacked diffs (§3.2) rather than pretending the units are independent.
- **The sampled audit (§15.3) finds a false `Done: 8/8` in more than 1 of 5 sampled units, twice running.** `done-check` is not mechanizing enough. → Convert the failing criteria to hard CI gates before writing another feature.
- **Human-facing notifications exceed 5/week for a month despite tuning.** The hygiene system has become the thing it was built to prevent. → Delete jobs, starting with the lowest-value, until the budget holds. Deletion is the intended remedy, not a defeat.
- **Fractal Node crates gain external consumers before PH5.** → Split the workspace into a lockstep internal core and independently versioned published crates; keep one release train.
- **Agent throughput exceeds ~200 Work Units/week.** Review capacity, not CI, becomes the bottleneck. → Expand the auto-merge classes with evidence, and raise the sampled-audit rate to compensate — never lower the gates.
