# 70 — Innovation Proposals

> **Prerequisites:** the Canon (`00`–`03`) and `50-roadmap-phases.md`.
> **Status:** Proposals, not Canon. Nothing here is in the roadmap until Andrew accepts it and something else is cut to pay for it (`02 §8`).
> **Governs:** nothing yet. That is the point — these are put forward for judgment, not smuggled into scope.

---

## 0. Why This Chapter Exists

The brief invited creative additions. What follows are twelve, each argued on its merits and priced honestly. They are ordered by **leverage per unit of cost**, not by how interesting they are.

Four of them I would adopt now. They are marked **ADOPT**. Three are marked **STRONG** — genuinely valuable, but they should wait for the phase where they land naturally. The rest are marked **CONSIDER** and are recorded so they are not lost.

Every proposal states: the idea, the problem it solves, the principle it serves, the cost, the phase, and — importantly — **the way it could go wrong**. A proposal without a stated failure mode is a pitch, not a design.

---

## 1. The Commons  **ADOPT · PH1 · low cost**

**The idea.** One platform-level Society — `The Commons` — that every Citizen belongs to from the moment of registration, at Level 0, automatically. It is a real Society with a real Charter, real Chambers, a real Treasury, and real moderation. It is not a support forum and not a welcome bot.

**The problem it solves.** The single highest-mortality moment in any community product is the first sixty seconds, when a new arrival lands in an empty room. Fractal Node's design makes this *worse* than average, because our unit is a Society and a new Citizen has none. The current PH1 onboarding says "create a Society" — which asks the loneliest possible thing of someone who just arrived: build the room yourself, then stand in it alone.

The Commons inverts that. A new Citizen lands somewhere already alive, learns what a Chamber is by being in one, earns their first XP by participating rather than by configuring, and founds their own Society when they have a reason to — which is the only time anyone should found one.

**Why it is not a hack.** The Commons is subject to the same Charter machinery as every other Society. It is the platform's proof that the primitive is sufficient: if we cannot run our own front door on our own abstraction, the abstraction is wrong. It also gives us a permanent dogfooding surface where every governance, moderation, and economy change is felt by the team first.

**Principle served.** P1 (the atomic container is sufficient for everything, including us), P7 in spirit.

**Cost.** Near zero in engineering — it is a seeded Society. Real cost is **moderation and stewardship**, which is a staffing commitment from day one, not a feature. Budget a named human steward before PH1 launch.

**How it goes wrong.** The Commons becomes the only place anyone goes, and Societies never form — the platform collapses into a single forum with extra steps. **Mitigation:** the Commons is deliberately *shallow*. It has no Vault beyond a small gallery, no Facet minting, no Treasury spending by members, and a hard cap on Chamber count. It is a lobby, and it is designed to be slightly less satisfying than the Society you will build. Track the ratio of Commons activity to Society activity as a health metric; if it exceeds 40% after PH2, the Commons is too good and must be narrowed.

---

## 2. The Platform Charter  **ADOPT · PH1 · low cost, high brand leverage**

**The idea.** Fractal Node the company publishes and versions its own Charter, in the same machine-readable format Societies use, binding the operator to specific commitments: what data is collected, what the platform may and may not do to a Society, the emission parameters it cannot change unilaterally, the notice period before a material change, the export guarantee, and the conditions under which a Society may take its data and leave. Amendments follow a published process with a notice window. It is rendered in the product with the same `CharterView` component every Society gets — diff-first, showing what changed, who signed, when it took effect.

**The problem it solves.** Every platform that has ever promised sovereignty has broken the promise, and users know it. A statement of values is worth nothing; a versioned, diffable, notice-bound instrument that the platform submits to *using its own governance machinery* is worth a great deal — and it is checkable.

**Why it is unusually strong here.** It costs almost nothing to build (the machinery already exists for Societies) and it is the most credible possible expression of the entire product thesis. It is also the best marketing asset in the document: a competitor cannot copy it without actually binding themselves.

**Principle served.** P9, P12, and the brand thesis in `33 §1` — *an instrument, not a platform.*

**Cost.** Engineering: days. **Legal and organizational: real.** The Platform Charter must be reviewed by counsel and must be something the company genuinely intends to honor, because publishing one and violating it is far worse than never publishing one. This is a governance commitment wearing a feature's clothing, and it should be adopted with that understood.

**How it goes wrong.** It is written vaguely to preserve flexibility, and becomes a terms-of-service document with better typography — which is worse than nothing, because it makes a promise it visibly does not keep. **Mitigation:** every clause must have a falsification test, exactly like the Foundational Principles. A clause a third party cannot check does not go in.

---

## 3. The Instrument Panel  **ADOPT · PH1 (skeleton) → PH4 (full) · low cost**

**The idea.** A permanently public, real-time page — no login — showing the platform's actual operational and economic state: total supply and its provenance, emission this window against cap, active Sources and Sinks with volumes, Custodian count and aggregate durability, Shard repair queue depth, Relay latency percentiles, incident history, SLO attainment, and the anchor chain. Machine-readable at `/v1/public/instrument` as well as human-readable.

**The problem it solves.** Two at once. First, P12 requires that emission be "bounded, measured, and **published**" — this is where publishing happens, and without it the principle is unverifiable. Second, every token economy is assumed to be lying until proven otherwise, and the only proof that works is continuous public measurement that would be embarrassing to fake.

**Why it fits this brand specifically.** The existing site already has this instinct — the `[ 02 / DEVELOPMENT RELAY ]` section is a live, sanitized signal from the system as it forms. The Instrument Panel is that section grown into a permanent product surface. The brand was pointing at this before the product existed.

**Principle served.** P12 directly, P9 (it demonstrates what we *don't* collect by showing everything we do), and the honesty register of `33 §7`.

**Cost.** Low. The data already exists as projections; this is a read model and a page. Ongoing cost is the discipline of not quietly removing a metric when it looks bad — which is the actual commitment being made.

**How it goes wrong.** It becomes a speculation dashboard that attracts people interested in the number rather than the platform. **Mitigation:** the panel shows *flows and integrity*, never price, never a chart designed to be screenshotted for trading. FRC has no external price until PH9, and the panel is built to look like an operations console, not a ticker — which the design system makes natural.

---

## 4. The Undo Window  **ADOPT · PH1 · low cost**

**The idea.** Every consequential action states, up front and in the confirmation, whether it is reversible and for how long. Small transfers settle instantly and finally. Transfers above a Citizen-set threshold enter a stated hold — default 60 seconds, configurable up to 24 hours — during which the sender can cancel and the recipient sees the amount as `PENDING`. Destructive actions (leaving a Society, deleting an Object version, revoking an Envelope) get the same treatment where the domain allows. Some things are honestly irreversible (an enacted Charter amendment, a completed Fracture) and say so plainly instead of pretending.

**The problem it solves.** Irreversibility is the source of nearly all anxiety in financial and social software, and anxiety is the enemy of the ambient, unhurried feel the design system is chasing. The industry's answer has been confirmation dialogs, which do not work — people click through them — and which are a dark pattern's close cousin. A time window works because it matches how people actually notice mistakes: a second or two after committing them.

**Principle served.** P10 in spirit (the product must feel good under the hand), P12 (an honest economy tells you what it will and will not undo), and the `33 §7.3` error register.

**Cost.** Low, and it is an *architectural* saving rather than a cost: a held Transfer is a state the Ledger already models (`locked`), and the domain is event-sourced, so a cancellation is a compensating event rather than a mutation. Building this in PH1 is dramatically cheaper than retrofitting it in PH4 once the wallet surfaces exist.

**How it goes wrong.** The hold becomes a source of confusion — "did it send?" — or a vector for a sender to grief a recipient by repeatedly sending and cancelling. **Mitigation:** the recipient always sees the pending amount with its release time, so nothing is hidden; and repeated cancellation is a Trust signal, counted and visible.

---

## 5. Expeditions — Time-Boxed Societies  **STRONG · PH3 · medium cost**

**The idea.** A Society may declare, at founding, an **end date** and a dissolution plan. At the end date it seals automatically: the archive stays readable forever, the Treasury distributes per the plan, members keep their Standing and Insignia, and the lineage records it as *completed* rather than *abandoned*.

**The problem it solves.** Every community platform is a graveyard. Discord servers, Slack workspaces, and subreddits do not end — they rot, and the rot is visible and demoralizing. But an enormous fraction of real human collaboration is *inherently* time-boxed: a project, a course cohort, a game season, a book club, a conference, a campaign, a launch. Forcing those into a permanent container guarantees they become ruins.

An Expedition that ends on schedule is a **success**, and the platform should say so. "This Society completed its expedition" with a sealed archive and a completion Insignia is a fundamentally different emotional object than a dead server.

**Why it is more than a setting.** It changes what a Society is for. It makes founding one low-stakes, which feeds directly into proposal 1's goal of getting people to found Societies for real reasons. And it produces a steady stream of *completed* lineage, which is the raw material for the Cartographer (proposal 10) and for Standing that means something.

**Principle served.** P1 (lifecycle is a property of the container), P12 (dormant Societies are a real storage cost; completed ones are a bounded one).

**Cost.** Medium. Requires the Dissolution path from `11 §3.3` to be solid, scheduled execution, and careful UX around the approach of an end date. Should follow governance v1 in PH3.

**How it goes wrong.** A Society ends while it is still alive and its members are blindsided. **Mitigation:** an Expedition can be extended by the same governance process that amends a Charter, with escalating notice as the date approaches (30 / 7 / 1 day). Ending is a default, never a surprise.

---

## 6. The Supervised Autonomy Ladder  **STRONG · PH3 · low cost, high product value**

**The idea.** An Agent's Envelope is not a static grant but a **ladder with rungs**. An Agent begins at `Observer` (read only), and earns promotion to `Proposer` (may draft actions a human approves), then `Actor` (may execute a narrow class autonomously), then `Steward` (may execute a broad class, with a spending cap). Each promotion is a human act, but the platform *proposes* it when the Agent's record supports it: N actions executed, zero blocked, zero reverted, Trust above a threshold, over a minimum period. Demotion is automatic on a blocked action or a reverted outcome.

**The problem it solves.** P4 is correct and it is also, as currently specified, a chore. A Citizen must sit down and reason about capability grammar before an Agent can do anything, and most people will either grant too much once (to make the friction stop) or never grant anything. Both outcomes defeat the principle.

The ladder converts a security configuration task into a **relationship that develops**. It is how people actually delegate to other people: small things first, more over time, less after a mistake. It makes the human governance real rather than ceremonial, because the human decision at each rung is small, informed, and well-timed.

**Principle served.** P4 directly — it makes the principle usable rather than merely correct. Also P8 (the default stays deny; the ladder only changes the *path* to a grant, never the grant mechanics).

**Cost.** Low. The Envelope machinery already exists; this is a promotion-recommendation projection over the existing audit trail, plus four preset Envelope templates. The hard part is honest metric selection, which is a design decision, not an engineering one.

**How it goes wrong.** Promotion prompts become notification spam and get approved reflexively, recreating the "grant too much once" failure. **Mitigation:** promotions are *never* pushed as notifications. They appear in the Agent surface when the Citizen goes there. A promotion prompt that is dismissed does not reappear for 30 days. And a rung can never be skipped, so reflexive approval still moves only one step.

---

## 7. The Witness Protocol  **STRONG · PH5–PH6 · medium cost**

**The idea.** A Society may ask another Society to **witness** an event: a governance vote, a Fracture plan, a Charter amendment, a treasury disbursement, an escrow release, a completed Expedition. Witnessing is a signed attestation referencing the anchor at which the event occurred. It is not approval and carries no authority — it is a countersignature that says *we saw this, at this state root, at this time.*

**The problem it solves.** Federation as specified in `19`/`11` is discovery and capability sharing. That is useful and thin. There is no mechanism by which Societies build *relationships of consequence* with each other, which means the Fractal Net is a set of islands with a shared search index.

Witnessing creates a real inter-Society social fabric out of almost nothing. It gives Trust a meaning between organizations, not just between people. It makes a Fracture harder to dispute later. It gives a young Society a way to borrow credibility from an established one, and gives an established one something valuable to grant. And it is the natural substrate for later contract, escrow, and arbitration features between Societies.

**Why it is cheap.** The anchoring machinery from `16 §6` already produces the thing being signed. A Witness is a signed reference plus an event. There is no new consensus, no new consistency requirement, and no cross-Society transaction — it is a signature about a fact that already exists, which is exactly the shape our partitioning tolerates well.

**Principle served.** P1 (Societies stay atomic; the relationship is a signature, not shared state), P11 (it is chain-shaped without needing a chain).

**Cost.** Medium. Mostly protocol design and the surfaces to request, grant, and display witnesses. Should follow Fracture in PH5, because Fracture is its most valuable application.

**How it goes wrong.** Witnessing becomes a status economy — societies farming countersignatures, or a cartel of mutual witnesses inflating each other's credibility. **Mitigation:** a witness's weight derives from the witnessing Society's own Trust and is subject to the same relationship-saturation function that governs peer attestation in `17`. Ten witnesses from one cluster count for far less than three from unrelated lineages.

---

## 8. Governance as a Product  **CONSIDER · PH6 · low marginal cost**

**The idea.** Charters become a first-class marketplace category with real depth: forkable templates authored by people who know what they are doing, versioned, rated by the Societies that ran them, with a "governance changelog" showing what a Society amended and why. A Charter template can be free or paid, and its author earns when Societies adopt it.

**The problem it solves.** Governance design is genuinely hard, most communities improvise it badly, and the failure is invisible until it is catastrophic. Meanwhile there exist people — organizers, cooperative practitioners, DAO researchers, moderators with a decade of experience — who know a great deal about it and have no way to distribute that knowledge as a product.

**Why it is a differentiator.** Nobody sells governance. Templates are the natural artifact of a system that made governance machine-readable in the first place, and the marketplace and Extension machinery already exist. It also produces something rare: a body of *empirical* evidence about which governance structures actually retain members, because we can measure it across thousands of Societies (in aggregate, never per-Society, never covertly).

**Cost.** Low marginal cost once the marketplace exists. The real work is the review standard — a bad Charter template can do genuine harm.

**How it goes wrong.** A popular template contains a subtle capture mechanism — a role that quietly accumulates authority, or an amendment rule that entrenches founders. **Mitigation:** Charter templates are a high-risk review class requiring human review with a governance checklist, and adopting a template renders a plain-language summary of its power structure that the adopting Society must acknowledge.

---

## 9. Society Grants and Patronage  **CONSIDER · PH4 · low cost**

**The idea.** A Society Treasury can issue a **Grant** to a Citizen: a scheduled or milestone-triggered stream of Fraction, governed by the Charter, visible in the Treasury, and cancellable by the same process that created it. Citizens can also patronize each other directly with recurring transfers.

**The problem it solves.** The economy as specified pays for *measurable infrastructure contribution* very well and for *creative and social contribution* only through gameable proxies — which `17` correctly handles by damping them almost to zero. The result is an economy that pays generously for storing bytes and barely at all for the things that actually make a community worth being in: writing, moderating well, teaching, organizing, making things.

Grants solve this by moving the judgment where it belongs — to humans who can see the contribution — while keeping the accounting honest. The platform does not have to algorithmically detect that someone's writing is good. A Society that benefits from it can simply pay them, from a Treasury that the Society filled.

**Principle served.** P12 (it is a redistribution, not an emission — no new Fraction is created, so it cannot inflate), P4 (humans judge; the system settles).

**Cost.** Low. Scheduled Postings against an existing Treasury with Charter authorization.

**How it goes wrong.** Treasuries drain into founder-aligned recipients, or Grants become a payroll obligation a volatile Treasury cannot meet. **Mitigation:** Grants are capped as a fraction of Treasury balance by the Charter, are always visible to all members, and a Grant that cannot be funded pauses rather than overdrawing.

---

## 10. The Cartographer  **CONSIDER · PH5 · medium cost, high marketing value**

**The idea.** A live, navigable map of the Fractal Net: Societies as nodes sized by activity, lineage as edges, Fracture events rendered as visible splits, Federations as clusters, Expeditions that completed shown as sealed. Zoom from the whole net down to one Society's ancestry. Public for `Public` Societies, and a private view of your own lineage.

**The problem it solves.** The platform's central concept — societies that grow, evolve, and fracture into independent nodes — is currently invisible. It happens in a database. A user experiences Fractal Node as a chat app with a wallet unless something makes the *shape* of the network legible.

**Why the brand demands it.** The identity is built on orbits, nodes, lattices, and a diamond core. The Cartographer is that identity made functional rather than decorative. It is also, straightforwardly, the best marketing asset the project could have: a single animated image of a network fracturing and reforming explains the entire product without a word of copy.

**Cost.** Medium. Requires Atlas (S15) for the global projection, a layout engine that stays stable as the graph changes, and real performance work at scale. Should follow Fracture in PH5, when there is something to draw.

**How it goes wrong.** It becomes a vanity leaderboard — biggest Society wins — which corrupts behavior. **Mitigation:** node size reflects *activity*, not membership, and is log-scaled and bucketed rather than precise. There is no ranking, no top-N list, and no comparative metric. It is a map, not a scoreboard.

---

## 11. Portable Standing with Decay  **CONSIDER · PH5 · medium cost**

**The idea.** When a Citizen joins a new Society, they may present their Standing from a prior Society as a **reference**, admitted at a discount set by the receiving Society's Charter (default 25%, capped, and always visible as *imported*). It decays if not renewed by local contribution.

**The problem it solves.** Every Society is a cold start for its members. Someone with ten years of exemplary conduct arrives as a stranger with zero Standing, unable to do the things they are demonstrably qualified to do. Meanwhile a Society admitting an unknown has no signal at all.

**Principle served.** P1 (the receiving Society decides, so sovereignty holds), P12 (Standing is not transferable as a resource — it is *evidence*, discounted and decaying).

**Cost.** Medium. The hard part is preventing it from becoming a caste system.

**How it goes wrong.** It creates an aristocracy: established Citizens arrive powerful everywhere, newcomers are permanently behind. **Mitigation:** imported Standing is capped at a level below what local contribution can reach quickly, decays on a 90-day half-life, is always labelled as imported, and can be refused entirely by a Charter. It buys you a hearing, not a position.

---

## 12. The Rehearsal  **CONSIDER · PH5 · low cost**

**The idea.** Generalize Fracture's mandatory dry run into a platform-wide primitive. Any consequential operation — a Charter amendment, a treasury disbursement, an Envelope grant, a bulk moderation action, a Dissolution — can be *rehearsed*: fully evaluated against policy and invariants, producing a complete diff of what would change, without emitting anything.

**The problem it solves.** `11 §3.2` already requires this for Fracture and `30`/`31` already implement `--dry-run` at the API and CLI. What is missing is making it a **first-class product surface** rather than a developer flag: a "Rehearse" button next to every consequential action in the GUI, showing exactly what will change before it changes.

**Why it matters more than it sounds.** It is the highest-trust interaction pattern available in software and almost nobody offers it, because most systems cannot compute their own consequences. Ours can — deterministic domain, event-sourced state, invariants as executable property tests. This is a capability the architecture already paid for and is not yet spending.

It is also the perfect complement to the Supervised Autonomy Ladder (proposal 6): an Agent at the `Proposer` rung produces a rehearsal, and the human approves a *diff* rather than a description.

**Cost.** Low — the mechanism exists. The work is UI and making the diff human-legible.

**How it goes wrong.** Rehearsal output is too technical to be useful and gets skipped like a confirmation dialog. **Mitigation:** the diff is rendered in plain language generated from the effect set, with the technical detail one disclosure away. If it cannot be explained in a sentence, the operation is too complex and that is the finding.

---

## 13. Summary and Recommendation

| # | Proposal | Verdict | Phase | Cost | Principal risk |
|---|---|---|---|---|---|
| 1 | The Commons | **ADOPT** | PH1 | Low + stewardship | Becomes the only place |
| 2 | The Platform Charter | **ADOPT** | PH1 | Low + counsel | Written vaguely |
| 3 | The Instrument Panel | **ADOPT** | PH1→PH4 | Low | Attracts speculators |
| 4 | The Undo Window | **ADOPT** | PH1 | Low | Confusion, griefing |
| 5 | Expeditions | STRONG | PH3 | Medium | Surprise endings |
| 6 | Supervised Autonomy Ladder | STRONG | PH3 | Low | Reflexive approval |
| 7 | The Witness Protocol | STRONG | PH5–6 | Medium | Status cartels |
| 8 | Governance as a Product | CONSIDER | PH6 | Low marginal | Capture in a template |
| 9 | Society Grants | CONSIDER | PH4 | Low | Treasury drain |
| 10 | The Cartographer | CONSIDER | PH5 | Medium | Vanity leaderboard |
| 11 | Portable Standing | CONSIDER | PH5 | Medium | Aristocracy |
| 12 | The Rehearsal | CONSIDER | PH5 | Low | Illegible diffs |

**What I would do.** Adopt 1–4 into PH1. Together they cost roughly two weeks of engineering and they change the product's character disproportionately: the Commons fixes the cold start, the Platform Charter makes the thesis credible, the Instrument Panel makes P12 verifiable, and the Undo Window makes an economy feel safe to touch. All four are far cheaper now than retrofitted, and three of them are *more* about commitment than code — which means the right time to decide is before anything is built, not after.

Per `02 §8`, adopting them requires naming what leaves PH1. My recommendation: defer the Profile Module system's full catalogue to PH2 (ship three Modules, not eighteen) and defer the search surface to PH2. Neither is on the spine sentence.

Proposals 5, 6, and 7 I would schedule now into PH3 and PH5 respectively rather than leaving them to be rediscovered, because each one changes a design decision in the phase before it — Expeditions affects the Dissolution path, the Autonomy Ladder affects Envelope templates, and the Witness Protocol affects the anchor format.

The rest belong in `docs/proposals/` and should be revisited at each phase gate.
