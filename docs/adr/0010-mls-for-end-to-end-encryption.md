# ADR-0010 — MLS (RFC 9420) for end-to-end encrypted group messaging

**Status:** Accepted
**Date:** 2026-09-03
**Deciders:** Andrew
**Phase:** 2

## 1. Context

N6 requires end-to-end encryption for private messages, voice and video, with **no server-side plaintext path present in the code, not merely unused**. P8 makes this non-tradeable: it sits first in the `00 §2` conflict order.

The shape of the problem is set by what a Society is (`14 §4.1`): a group with continuous churn, several devices per member, and sizes from 3 to roughly 5,000. That combination eliminates the two obvious answers. Pairwise Double Ratchet sessions cost O(N) work per sender and O(N²) per group — 500 members at three devices each is 1,500 sessions per sender. Sender-key schemes (Olm/Megolm) make adds cheap but **removal weak**: a removed device retains the key until every sender rotates.

Removal is the operation a Society performs most. `11 §2.4` has a `Suspended` and a `Departed` membership state; `12 §7` makes revocation immediate and retroactive to in-flight actions. A moderation removal on Tuesday whose subject can still read Wednesday's Messages is not a moderation action, it is a delay. **Removal being cheap *and* cryptographically effective is the deciding property.**

## 2. Decision

We use **MLS (RFC 9420)**, via OpenMLS in `fractal-adapter-mls` (`41 §5.5`). One MLS group per `EndToEnd` Chamber, per DM pair, and per Convergence. **Leaves are devices, not Citizens**, which is what makes multi-device work without sharing keys between devices.

The Runtime is the MLS Delivery and Authentication Service: it stores KeyPackages, orders and broadcasts handshake messages, and enforces epoch consistency. It is **untrusted for confidentiality and trusted only for availability and ordering** — exactly the trust model MLS assumes and exactly the one we can honestly claim. Credentials are the FNID key chain from `12`, so a peer verifies "this device belongs to `@handle`" without trusting the directory.

Because a quiet group never exercises post-compromise security, we force it: every device Updates at least every 7 days or 500 group messages, whichever comes first. Proposals batch and Commit at 64 proposals or 10 seconds, except that `Charter.moderation.immediate_commit_on_removal` defaults **true**, forcing a Commit within one round trip on a moderation removal.

## 3. Consequences

### Positive
- Add, remove and update are O(log N) ratchet-tree operations, so churn at Society scale is tractable.
- Forward secrecy at two granularities — per-message within an epoch, and destruction of `init_secret_N` on epoch advance — plus post-compromise security: a device compromised at epoch N is locked out at N+2 with nobody needing to notice the compromise.
- Device fan-out is one ciphertext per group per Message regardless of device count; per-device work is a `Welcome` at join and a path secret per Commit.
- The `exporter_secret` yields SFrame media keys for Phase 4 voice/video and the franking key for `14 §6` moderation, so one key schedule serves three subsystems.
- It is an IETF standard with independent implementations, which matters more for a cryptographic protocol than for anything else in this corpus.

### Negative
- **MLS provides no history.** A device joining at epoch 90 cannot read epoch 12. `14 §4.5` resolves this with three explicit mechanisms of increasing risk, each a stated choice; the Vault-backed archive in particular trades forward secrecy *for the archive* against a Citizen-held key, and the client says so in one sentence at the moment of enabling.
- **Commit batching means a removed member can decrypt for up to the batch window.** Acceptable for a voluntary `Left`, unacceptable for a moderation removal — hence the immediate-commit default and the epoch storm it causes during a mass ban.
- **A scale ceiling around 1,000 leaves**, where KeyPackage churn and Commit size bind. Above it a Chamber requesting E2EE is told so and offered a smaller Chamber or transport encryption — **never silently downgraded**.
- The ecosystem is young. Implementation care, a security review gate (`40 §10.5`) and an interop test corpus are permanent costs.
- Metadata is not hidden: membership, timing, size, epoch churn and connection origin are visible to the Runtime. We pad to fixed buckets and **do not** implement cover traffic or mixnets, declining openly rather than half-implementing (`14 §4.6`).

### Neutral / follow-on work
Moderation under encryption uses message franking derived from the same key schedule (`14 §6`) — reporting works without a plaintext path.

## 4. Alternatives Considered

| Alternative | Why it was plausible | Why rejected |
|---|---|---|
| **Signal protocol (Double Ratchet, pairwise)** | The best-scrutinized 1:1 security in existence; mature, well-understood, excellent libraries | O(N²) group fan-out and O(N) rekey per membership change. At 500 members × 3 devices that is 1,500 sessions per sender per message. Correct for a messenger, wrong for a community |
| **Olm/Megolm sender keys** | Proven at Matrix scale; cheap adds; simple mental model | Removal is weak: a removed device keeps the group key until every sender rotates. Removal is the operation a Society performs most, and `11 §2.4`'s Suspended/Departed states would be cryptographically meaningless. PCS guarantees are also materially weaker |
| **Adopt Matrix wholesale** | A mature federated protocol with real E2EE and existing clients — enormous free surface area | Its state-resolution DAG duplicates the per-Society event Log we already own (`10 §4`), giving two ordering systems. Its data model is not P1-shaped, so adopting Matrix means adopting a container model that is not the Society. We reuse its lessons, not its stack |
| **A third-party chat SDK** | Ships in a week; someone else operates it | Fails N6 outright — the vendor holds a plaintext path — plus P5 (vendor types reaching the domain) and P2 (no home for a local-first replica). The most tempting shortcut in the document, and it forfeits the product's premise |

## 5. Exit Cost

**10–14 engineer-weeks, and it is a genuinely expensive exit — which is why the alternatives were scored on cryptographic properties rather than convenience.** The work: implement the replacement group protocol behind the same group port; migrate live groups, which cannot be re-encrypted server-side by construction and must therefore be re-established client-side per group with a new epoch 0; re-derive SFrame media keys and franking keys from the new schedule; re-run the security review gate; and accept that history under the old scheme remains readable only through the `14 §4.5` archive mechanisms. The migration is client-driven and staged per Chamber, which is most of the estimate.

## 6. Principle Served

**P8** and **N6** directly, and P8 is first in the `00 §2` conflict order, which is why MLS's ecosystem immaturity is accepted rather than treated as disqualifying. **P1**: the Charter chooses the removal-commit behaviour, so a Society governs its own trade between epoch-storm cost and immediate revocation. **P2**: MLS group state lives in `fractal-crypto` inside the shared core, so an offline device holds its own keys.

## 7. Falsification Test

1. **No plaintext path exists in code.** A source scan asserts that no crate outside `fractal-crypto` and the client shells can construct a decrypted `MessageBody` for an `EndToEnd` Chamber, and that no Runtime code path holds a group secret. Presence, not use, is the violation (N6).
2. **Removal effectiveness.** An integration test removes a device mid-conversation with `immediate_commit_on_removal` true, then asserts the removed device cannot decrypt any message with a timestamp after the Commit, and that its stored keys fail against epoch N+1.
3. **Forced PCS.** A test advances a quiet group past 7 days of simulated time and asserts every leaf has issued an Update.
4. **No silent downgrade.** A test creates a Chamber that exceeds the leaf ceiling and asserts the client surfaces a refusal rather than falling back to transport encryption.
5. **Interop**: OpenMLS test vectors run in CI against every release.

## 8. Maintenance Horizon

RFC 9420 is an IETF standard with several independent implementations, so the protocol is not single-maintainer. OpenMLS is the concentrated risk: it is the critical-path crate for N6 and requires the `00 §3.5` vendoring plan — we pin an exact version, vendor the source in-tree for reproducibility, and maintain a build against a second implementation in CI so that a fork is a switch rather than a project. The adapter boundary means the rest of the Runtime does not know which implementation is linked.

## 9. Review Trigger

Reopen if (a) MLS implementations prove immature at Society scale above ~1,000 members, in which case `10 §12` pre-commits the fallback: sender-key groups with periodic rekeying, and **publish the reduced guarantee honestly**; (b) OpenMLS is abandoned or fails a security review, triggering the vendoring plan; or (c) the Phase 4 SFrame integration shows the `exporter_secret` derivation cannot carry media keys at the required rotation rate.
