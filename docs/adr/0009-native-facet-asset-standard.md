# ADR-0009 — A native Facet asset standard (FN-ASSET/1), dynamic by default

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 3

## 1. Context

`01 §4` bans the word NFT and defines the **Facet** as the native digital asset primitive. `11 §2.9` states that Facets are dynamic by default and that static-forever assets are the degenerate case (`evolution: Immutable`), not the default. `16 §10` supplies the reasoning, and it is a design argument rather than a branding one.

ERC-721 and ERC-1155 encode one idea well: a scarce, transferable pointer to an identifier whose meaning lives somewhere else. The consequences are structural. State is an ID plus an owner, so anything richer lives off-standard and every project reinvents it incompatibly. Metadata is a URI, so the asset is only as durable as someone's hosting bill. Mutation is not modelled, so changing an asset means a bespoke contract or minting a replacement and abandoning the original's provenance. Ownership is the whole relationship, so licensing, custody and use-rights are bolted on. Royalties are a hint. Provenance records who owned it, never what happened to it.

The deepest mismatch: **these standards model a certificate, and the interesting digital objects in a social platform are not certificates — they are things that accumulate history.** An Insignia earned across three years of a Society. An instrument that records who played it. A serialized work that gains a movement each year. Modelling those as an immutable pointer plus an off-standard side channel is modelling them wrong.

## 2. Decision

We specify **FN-ASSET/1**, a native standard in which a Facet is a stateful object with declared evolution rules whose entire history is provenance. `16 §11` fixes the shape: immutable identity (`facet_id` embedding its minting Society, `standard`, `schema`, `creator`, `genesis` hash), mutable `state` with a `state_hash` and a monotonic `revision`, `evolution` and `composition` rules mutable only under their own amendment clause, `owner` and `custodian` as distinct concepts, `license` and `royalty` as first-class terms, an append-only `provenance` chain, and optional `bindings` to external chains that are explicitly lossy and non-authoritative.

Facet Standards (schemas) are one of the nine Global Registry entries; **instances are always Society-scoped** (`01 §6`, P1). Evolution is declarative and verifiable, not Turing-complete. Media live as Vault `ObjectId`s, never URLs.

## 3. Consequences

### Positive
- Evolution is a Domain Event in a Society's Log — replayable, anchored, provable — rather than an externally priced transaction. That is strictly more expressive and strictly cheaper than the on-chain equivalent.
- Provenance records *what happened*, not merely who held it, which is what makes an Insignia or an evolving work meaningful.
- `creator` never changes, even on transfer: authorship is a fact, not a property right, and conflating the two is how creator attribution gets laundered on resale.
- Licensing is orthogonal to ownership, so custody, delegation and use-rights are modelled rather than bolted on, and royalties are enforced by the Runtime rather than suggested to a marketplace.
- `facet_id` embeds its minting Society, so P1's "which Society owns you?" test is satisfied at the identifier level without a lookup.

### Negative
- **We forfeit drop-in interoperability with existing marketplaces and wallets.** A Facet is not tradeable on an external venue without a projection, and the projection is lossy by construction (`16 §15`).
- **The trust assumption is explicit and real:** evolution is enforced by our Runtime, not by a public chain. We state this rather than hide it, and anchoring (`16 §6`) converts it from "trust us" to "verify our history", which is weaker than a public chain and stronger than nothing.
- We own a standard. Schema versioning, evolution-rule semantics, composition rules and the amendment clause are all ours to specify, test and support forever.
- Rich mutable state is a larger attack surface than an ID and an owner. `16 §17` enumerates the threats; the mitigation is that evolution is declaratively verifiable rather than scripted.

### Neutral / follow-on work
`16 §15` retains external projection as a genuine capability — a Facet may project onto ERC-721 for portability, and is never defined by it. Turing-complete asset behaviour is deferred to the Phase 7 Experience Runtime with a real sandbox (`20 §12`), where it belongs.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **Adopt ERC-721/1155 semantics natively** | Instant conceptual familiarity, existing tooling, marketplace and wallet interoperability, no standard to write | Interoperability with an ecosystem that cannot represent evolution, provenance depth, licences, composition or enforceable royalties means interoperating by discarding every distinguishing property. We would also inherit the URI-durability problem and the royalty-as-a-hint problem, and adopt a certificate model for objects that are not certificates |
| **Mint Facets on an external chain now** | Real scarcity guarantees, no trust assumption to explain, immediate liquidity | Every mint and every evolution becomes a priced, latency-bound external transaction. `11 §2.9`'s evolving Facets would each cost a fee per state change, which makes the entire dynamic model uneconomic. It also imposes external key custody on every Citizen, contradicting P8, and hands the regulatory posture to a third party (see ADR-0008) |
| **Turing-complete evolution scripts** | Maximum expressiveness; every asset a program; no standard would ever be limiting | Unbounded audit surface, unbounded execution cost, and replay dependent on an execution engine — which breaks ADR-0005's deterministic fold. It also puts an infinite security surface at the moment of asset creation, which is the moment with the least review. Deferred to the Phase 7 Experience Runtime under a real sandbox |
| **No native asset primitive; Facets as ordinary Vault objects with metadata** | Nothing new to build; the Vault already has versioning, ACLs and content addressing | Loses ownership, transfer, licensing, royalty and provenance semantics, which are the properties the marketplace (`19`) and the progression system (`18` Insignia) are built on. It would push each of those to invent its own incompatible asset model — the exact failure `16 §10.1` identifies in the static standards |

## 5. Exit Cost

**8–12 engineer-weeks to migrate the corpus to a different standard.** The work: define `FN-ASSET/2` or the target standard; write a converter over the append-only provenance chain, which is possible only because history was never rewritten; re-issue `genesis` hashes under the new identity rules and cross-sign the old ones so existing proofs remain verifiable; migrate `LicenseSet` and `RoyaltyTerms` semantics, which have no mechanical mapping onto a static standard and require per-schema human decisions; update the Marketplace and Progression projections. The estimate assumes migrating *to* a richer standard. Migrating *to* a static standard is lossy and would be a product decision, not an engineering one.

## 6. Principle Served

**P11** (chain-agnostic by decree; external chains are lossy projections of a record we hold), **P1** (`facet_id` embeds its Society; instances are Society-scoped while schemas are global), **P12** (royalties and licences are enforced rather than suggested, so creator economics are honest), **P6** (evolution is a domain event, so an asset's state is replayable). Traded: interoperability, which sits under no principle and is a capability we recover partially through projection.

## 7. Falsification Test

1. **Determinism**: for any Facet, replaying its Society's log from `seq` 0 reproduces `state`, `state_hash`, `revision` and the full `provenance` chain bit-identically. Asserted by `xtask replay` and by `fractal-sim` invariant 11 (Fracture preserves total Facets).
2. **Genesis integrity**: `genesis == H(identity ‖ schema ‖ initial_state ‖ rules)` verified on every load; a mismatch is a hard error, not a warning. This is the check a counterfeit cannot pass (`16 §17` T6).
3. **No URIs**: a schema lint fails any `FacetSchema` `MediaSlot` that is not a Vault `ObjectId`. A URL in an asset schema is the durability failure this standard exists to avoid.
4. **Immutability of identity**: property tests assert `society_id`, `creator`, `facet_id`, `standard` and `genesis` are unchanged across every reachable evolution and transfer history.
5. **Declarative evolution**: any evolution rule requiring host execution beyond the declared verifier fails schema registration.

## 8. Maintenance Horizon

First-party standard, first-party implementation, no external maintainer. The obligation is the reverse of a dependency risk: **we are the maintainer other people depend on.** Registered Facet Standards are Global Registry entries and are effectively permanent, so schema versioning, upcasting and the amendment clause need the same discipline as the event registry (`40 §7.7`). External chain bindings sit behind the `Chain` port (ADR-0008), so a dead chain is an adapter that stops, not an asset corpus that breaks.

## 9. Review Trigger

Reopen if (a) an external standard emerges that natively expresses evolution, licensing, composition and enforceable royalties — in which case projecting onto it becomes strategically valuable and FN-ASSET/1 may become a profile of it; (b) fewer than 5% of minted Facets use a non-`Immutable` evolution rule two phases after Facet minting unlocks, which would mean the dynamic premise is unused and the complexity unearned; or (c) a registered schema's evolution rules prove unverifiable in the declarative form and pressure for scripting recurs at two consecutive phase gates.
