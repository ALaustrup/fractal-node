# 12 — Identity and Trust

> **Prerequisites:** the Canon (`00-foundational-principles.md`, `01-canonical-terminology.md`, `02-scope-guardrails.md`), `10-system-architecture.md`, `11-domain-model.md`.
> **Governs:** the FNID, Handles, device enrollment, key rotation, authentication, recovery, the Capability grammar and its evaluation, the Trust model, Sybil resistance, and the privacy boundary. Every authorization decision in the Runtime is defined here or is a defect.

---

## 1. What This Document Decides

A Citizen's FNID outlives every key, every device, every Society, and probably the current implementation of the Runtime. It has to be decided before anything is built and never quietly amended afterwards.

Four concerns are kept separate, because collapsing any two of them is the classic failure:

| Concern | Question | Owned by |
|---|---|---|
| **Identification** | Who is this principal, durably and globally? | FNID (§2) |
| **Authentication** | Is the party on this connection that principal, right now? | Devices, passkeys, sessions (§3–§5) |
| **Authorization** | May this principal do this, here? | Capabilities, Envelopes, the PEP (§7) |
| **Trust** | Should anyone rely on this principal's future behaviour? | Trust, Standing (§8) |

A system that lets a high-Trust principal skip an authorization check has destroyed all four (P8).

---

## 2. The FNID

### 2.1 Derivation

```
   ┌──────────────────────────┐
   │  Ed25519 identity key    │  generated on-device behind the KeyStore port,
   │  (32-byte public key)    │  never transmitted, never escrowed
   └────────────┬─────────────┘
                ▼
   ┌──────────────────────────────────────────────────────────┐
   │  body     = base32_lower_nopad(pubkey)         52 chars   │
   │  digest   = BLAKE3("FNID/1" ‖ pubkey)                     │
   │  checksum = base32_lower_nopad(digest[0..3])[0..4]        │
   │  FNID     = "fn1" ‖ body ‖ checksum            59 chars   │
   └──────────────────────────────────────────────────────────┘
                ▼
   fn1k4m7qxr2vhb9tzn6d3sfp8ywjc5glaeu0hi1o2rk9mqx4vtb7za9c
   └┬┘└──────────────────── body ────────────────────────┘└─┬┘
   prefix                                              checksum
```

```rust
pub struct Fnid([u8; 32]);           // the raw Ed25519 public key. Nothing else.

impl Fnid {
    pub fn parse(s: &str) -> Result<Self, FnidError>;  // validates prefix + checksum
    pub fn render(&self) -> String;                    // canonical 59-char form
    pub fn class(&self) -> PrincipalClass;             // registry fact, NOT encoded in the string
}
```

The same construction produces the FNID of a Citizen, Agent, Society, and Node. There is no type marker inside the string: encoding class into the identifier would make class immutable and force a migration when an Agent is promoted or a Society self-hosts.

**I-12.1:** the FNID derives from the *first* identity key and never changes. Rotation (§4) changes the signing key, not the FNID.

### 2.2 Why self-certifying

| Alternative | Rejected because |
|---|---|
| UUID / snowflake | Meaning requires an issuing authority. An identifier you cannot verify offline breaks P2 — an offline Node must verify authorship with no network. |
| W3C DIDs | The method zoo is the problem. Each method drags its own resolution infrastructure; `did:web` reintroduces DNS as the root of trust. A `did:fn` resolver can ship later as an adapter (P5). |
| Chain address | Binds identity to a chain deferred to Phase 8+ and to a key that cannot rotate without changing the address. Contradicts P11. |
| Email / phone as root | Roots identity in an intermediary that can be seized, SIM-swapped, or deplatformed. |
| X.509 / CA hierarchy | An issuing authority is a censorship point with no offsetting benefit at our threat model. |

**Honest cost.** (1) 59 characters that no human will memorize — mitigated by Handles, never by shortening the FNID. (2) No authority can revoke an identity globally; revocation is a signed statement and propagation is eventually consistent (§4.3). (3) Losing all key material with no recovery configured is final, and we say so before the fact (§6.4).

### 2.3 Handles and confusable normalization

```
raw input ──► NFKC ──► Unicode simple case fold ──► reject outside [a-z0-9_]
                                                          │
                                                          ▼
                                     skeleton = confusable_skeleton(h)   (UTS #39)
                                                          │
                                                          ▼
                                     UNIQUE INDEX on skeleton, not on handle
```

Handles are `[a-z0-9_]{3,24}` (`11 §6`). ASCII-only is a deliberate, uncomfortable choice: it is the only rule that makes homograph attacks structurally impossible rather than heuristically detected. Display names and Personas are fully Unicode and are where a Citizen expresses their actual name; the Handle is an address, like a domain label.

Even inside ASCII, `rn`/`m` and `l`/`1` are confusable in common fonts, so uniqueness is enforced on the UTS #39 skeleton: `paul_1` and `paul_l` collide and only the first claim succeeds. The second claimant is told the Handle is *confusable with* an existing one, not merely "taken" — the honest message reveals the squatting attempt.

| Rule | Value |
|---|---|
| Grace window for change | 14 days after first claim, then immutable (`11 §2.1`) |
| Reservation after `Departed` | 12 months |
| Reserved prefixes | `fn`, `fractal`, `admin`, `system`, `support`, `official` |
| Squatting deterrent | Claim requires Level ≥ 1 *or* a refundable Handle bond (§9) |
| Impersonation remedy | Published, appealable Moderation Action. Never a silent reassignment. |

---

## 3. Devices

### 3.1 One keypair per device

The identity key signs exactly one class of statement: device-chain entries. All day-to-day signing — Messages, commands, Envelope grants, votes — uses a **device key**: an Ed25519 keypair generated in the device's secure element behind the `KeyStore` port, non-exportable wherever the platform supports it.

```rust
struct DeviceRecord {
    device_id:       DeviceId,        // BLAKE3(device_pubkey), 16 bytes
    citizen:         Fnid,
    pubkey:          Ed25519Public,
    label:           String,          // human-set
    platform:        PlatformKind,    // Web | Desktop | Mobile | Headless | Cli
    hardware_backed: bool,            // secure enclave / TPM / passkey authenticator
    enrolled_by:     DeviceId,        // which existing device authorized this one
    mls_leaf:        Option<MlsLeafRef>,
    state:           DeviceState,     // Active | Revoked | Lost
}
```

### 3.2 The device chain

Device membership is not a mutable list. It is a hash-linked signed log rooted at the identity key — the same shape as everything else (P6).

```
  ┌───────────────────────────────────────────────────────────────┐
  │ GENESIS         sign(identity_key)                            │
  │   fnid, device[0].pubkey, recovery_policy_hash, at            │
  └───────────────┬───────────────────────────────────────────────┘
                  │ prev = H(genesis)
  ┌───────────────▼───────────────────────────────────────────────┐
  │ DEVICE_ADDED    sign(device[0])   new pubkey, label, prev     │
  └───────────────┬───────────────────────────────────────────────┘
                  │ prev = H(entry_1)
  ┌───────────────▼───────────────────────────────────────────────┐
  │ DEVICE_REVOKED  sign(device[1])   target, reason, prev        │
  └───────────────┬───────────────────────────────────────────────┘
                  │
  ┌───────────────▼───────────────────────────────────────────────┐
  │ KEY_ROTATED     sign(old identity key)  new pubkey, eff., prev│
  └───────────────────────────────────────────────────────────────┘
```

- **I-12.2:** every entry after genesis is signed by a key that was `Active` at the entry's `prev` position. A chain that verifies from genesis is authoritative; one that does not is discarded entirely, not partially.
- **I-12.3:** two entries claiming the same `prev` are a **fork**, treated as compromise: both branches freeze, all sessions terminate, recovery (§6) is the only path forward. Forks are never merged — silent fork tolerance is how key-transparency systems get quietly defeated.
- **I-12.4:** revoking the last `Active` device is refused unless recovery is configured.

The chain is replicated to every Node that has needed the Citizen's keys and served with an inclusion proof against a periodically **Anchored** merkle root (`10 §4`). Peers therefore detect a Runtime serving two different chain heads — the split-view attack — without trusting the Runtime.

### 3.3 Enrolling and removing

| Path | Requirement |
|---|---|
| First device | Identity key generation → genesis entry. Recovery setup prompted, not forced; re-prompted at Level 1. |
| Add device, existing device present | QR pairing bound to a short-lived ephemeral key; confirm on the *existing* device. Approval is a signed `DEVICE_ADDED`, never a Runtime decision. |
| Add device, no existing device | Recovery flow (§6). There is no other path — a "just email us" path is a permanent backdoor. |
| Remove a device you hold | Self-signed `DEVICE_REVOKED`, immediate |
| Remove a lost device | Signed by any other `Active` device, immediate |
| Device idle > 180 days | Flagged, never auto-revoked. Revocation is a Citizen's decision. |

On revocation, in one transaction: sessions terminated, API keys revoked, MLS leaf removed and the group rekeyed (post-compromise security for future messages), Signal pushed to remaining devices, `DeviceRevoked` emitted.

**What revocation cannot do:** retract ciphertext the lost device already decrypted, or undo actions it already took. Revocation is forward-looking. Anyone claiming otherwise is selling something.

---

## 4. Key Rotation Without FNID Change

Rotation makes an immutable identifier survivable. Required at least every 24 months and on any suspicion of compromise.

```
   FNID = fn1…   (forever the encoding of K0)

   K0 ──sign──► [rot 1: K1 @ t1] ──sign(K1)──► [rot 2: K2 @ t2] ──► …
   │                                                                 │
   └── verifies signatures with occurred_at < t1                     │
                          t1 ≤ occurred_at < t2  verified with K1 ───┘
```

**I-12.5:** a signature is valid iff the key that produced it — or an `Active` device key beneath it — was current at the signature's `occurred_at`. Because domain time and wall time are distinct fields (`10 §10`) and every event is Anchored, "when did this happen" is answerable without trusting the signer's clock.

Old signatures stay verifiable forever without re-signing anything. This is what makes P6's immutable log compatible with key hygiene: if rotation invalidated history, either history gets rewritten (violating P6) or keys never rotate (violating P8).

**Emergency rotation.** If the identity key is compromised but a device key is held, a device quorum (`ceil(n/2)+1`, minimum 2) rotates the identity key without the outgoing signature. This branch is **always** subject to the delay-and-notify window (§6.3) — it is precisely the branch an attacker would use.

**Propagation**, in order of authority: (1) attached to any signed object the new key produces, so a verifier that does not recognize the key fetches the chain segment and validates forward; (2) a `KeyRotated` event fanned out by the Relay to every Society the principal belongs to; (3) pulled on verification failure, with an inclusion proof against the latest Anchor.

**Honest cost.** Propagation is eventually consistent, so rotation weakens offline verification exactly when the network is unavailable. The alternative — a global revocation oracle — is a centralization point we refuse. Offline Nodes render such messages as `UNVERIFIED — key chain stale`, never as valid and never as forged.

---

## 5. Authentication

### 5.1 Passkeys, and no passwords

WebAuthn/FIDO2 passkeys are the default authenticator on every GUI surface (`10 §10`). The passkey is not the identity; it unlocks the device key. Phishing resistance comes free from origin binding.

**There are no passwords in the primary flow** — not discouraged, absent. Credential stuffing is the largest source of compromise on every comparable platform; a password store is a permanent liability; and a reset flow is a recovery backdoor that routes trust through email (§2.2). A password field would be a second, weaker recovery path bolted onto a system whose entire design is the first one (P8).

**Honest cost.** Passkey UX is still uneven across browsers and password managers, and a minority of prospective Citizens will bounce. Mitigations: platform authenticators, roaming keys, and QR cross-device flows are all supported; multiple passkeys per Citizen; recovery designed to be usable rather than punitive. We accept worse signup conversion to remove an entire compromise class. OIDC stays available as an optional adapter for organizational Nodes (P5) — it authenticates a session, it never becomes the identity root.

### 5.2 Sessions and token lifetimes

```
 passkey assertion ──► device key proves possession ──► SESSION
        ┌─────────────────────────────────────────────────┤
        ▼                                                 ▼
  access token (10 min, DPoP-bound)          refresh token (30 d, rotating,
  audience = gateway, bound to               reuse-detected, DPoP-bound
  device_id + session_id                     to the device key)
```

| Token | Lifetime | Binding | Revocation |
|---|---|---|---|
| Access | 10 min | Device key (DPoP) | Expiry — short by design |
| Refresh | 30 d, rotating | Device key; reuse of a consumed token kills the session family | Immediate |
| Elevated context | 5 min | Fresh passkey assertion | Required for: adding a device, rotation, changing recovery, granting an Envelope, Transfers above the confirm threshold |
| Agent session | ≤ Envelope `expires_at`, max 90 d (`11 §2.8`) | Agent device key | Immediate on Envelope revocation |

Bearer tokens are refused everywhere. Every token is bound to a key the holder proves possession of per call, so a stolen token alone is inert.

### 5.3 CLI and Agent authentication

The CLI is a first-class front end (N3, P13) and needs first-class auth, not a pasted secret.

```
  fn login
     ├─► Runtime issues user_code "FRAC-8Q2M", verification_uri, device_code
     ├─► Terminal displays the code; opens the browser where possible
     ├─► Citizen approves in an authenticated GUI session, seeing exactly which
     │   capabilities the CLI requests and on which host
     └─► CLI GENERATES ITS OWN device keypair, enrolls it as a DeviceRecord
         (platform = Cli), receives tokens bound to that key.
         No secret crosses the wire in either direction.
```

Headless and CI contexts use **API keys, which are not bearer god-tokens.** An API key is the credential *for an Envelope* (`11 §2.8`) — society-scoped, capability-limited, rate-limited, mandatorily expiring, individually revocable, and audited via `envelope_ref` on every event it produces.

```rust
struct ApiKey {
    key_id:      KeyId,        // appears in every audit line; the secret does not
    envelope:    EnvelopeId,   // ← the authority. The key is a pointer to it.
    device:      DeviceId,     // keys are devices; they appear in the device chain
    secret_hash: Argon2idHash, // the secret is displayed exactly once, at creation
    expires_at:  Timestamp,    // mandatory, ≤ 90 days
}
```

**I-12.6:** no credential in the system grants unscoped authority — not for the CLI, Agents, first-party services, or operations. A CapabilitySet of `**` cannot be created, because attenuation (§7.2) requires a grantor who holds `**`, and no such principal exists.

---

## 6. Recovery

### 6.1 Guardian shares

A recovery key is generated on-device and split by Shamir's Secret Sharing over GF(256) into `n` shares with threshold `t`. Each share is encrypted to a guardian's device key; guardians hold an opaque blob and learn nothing from it.

```rust
struct RecoveryConfig {
    guardians:      Vec<Guardian>,     // 3..=7
    threshold:      u8,                // t, where t ≥ ceil(n/2)+1
    delay:          Duration,          // 72h default, 24h floor, 14d ceiling
    notify_targets: Vec<NotifyTarget>, // every Active device + opt-in out-of-band
    version:        u32,               // hashed into the device chain
}

enum Guardian {
    Citizen(Fnid),
    Society { society: SocietyId, role: RoleId },   // survives seat turnover
    SelfCustody(ShareFingerprint),                  // offline: paper, HSM
}
```

Mixing guardian kinds is encouraged in the UI: a recovery set that is entirely one friend group fails as a unit.

| n | t | Collusion needed | Loss tolerance |
|---|---|---|---|
| 3 | 2 | 2 | 1 lost share |
| 5 | 3 | 3 | 2 lost shares |
| 7 | 4 | 4 | 3 lost shares |

`t ≥ ceil(n/2)+1` is enforced so a bare minority can never recover. `t = n` is refused: it converts one unresponsive guardian into permanent loss, the most common real-world failure of guardian schemes.

### 6.2 Delay-and-notify

Threshold alone does not defeat collusion. The delay does.

```
 t guardians submit shares
        ▼
 ┌─────────────────────────────────────────────────────────────────────┐
 │ RecoveryInitiated  (public event on the Citizen's global log)       │
 │   • high-priority Signal to every Active device                     │
 │   • every out-of-band notify target contacted                       │
 │   • the pending new key is PUBLISHED; the Citizen sees who asked    │
 └───────────────┬─────────────────────────────────────────────────────┘
                 │  ◄──── DELAY WINDOW (default 72h) ────►
                 │  ANY single Active device CANCELS, freezes the
                 │  guardian set, and forces a fresh setup to retry.
                 ▼
 ┌─────────────────────────────────────────────────────────────────────┐
 │ RecoveryCompleted → new identity key installed via KEY_ROTATED      │
 │   all prior devices revoked · all sessions terminated               │
 │   all Envelopes granted by this Citizen SUSPENDED pending review    │
 │   FNID unchanged · Handle unchanged · history unchanged             │
 └─────────────────────────────────────────────────────────────────────┘
```

During the window the Wallet is frozen for outbound Transfers and no Envelope may be granted. That asymmetry is what makes the scheme work: colluding guardians must reach the threshold *during* a window in which the victim holds a unilateral veto and is being loudly notified. A quiet takeover becomes a loud one, and a loud one fails.

**Honest cost.** A Citizen who has genuinely lost every device waits three days; one who has also lost every notify target gains nothing from the notification. The delay is configurable within bounds because the right trade differs between a Citizen holding a treasury role and one who does not.

### 6.3 What is not recoverable

Stated in the product, at setup time, before it matters.

| Recoverable | Not recoverable |
|---|---|
| FNID, Handle, Level, XP, Trust, Standing, Memberships | **E2EE message history for which no enrolled device holds keys** |
| Wallet balance and full Ledger history | Media whose content keys were wrapped only to revoked devices |
| Facet ownership and provenance | Anything held solely by the Citizen and not backed up |
| Vault objects whose content keys are wrapped to the recovery key | |

The E2EE line is the honest one. MLS gives forward secrecy and post-compromise security (`10 §11`) precisely *because* the Runtime cannot recover old keys. An escrow that made history recoverable is exactly the server-side plaintext path N6 forbids from existing in the code at all. Citizens may opt into a device-count guarantee (keep ≥ 2 devices) and, per Chamber, into wrapping content keys to their recovery key where availability matters more than forward secrecy — explicit, reversible, defaulted off (P9).

---

## 7. Authorization

### 7.1 The Capability grammar

```
capability  ::= path [ limit ] [ selector ]
path        ::= segment { "." segment }
segment     ::= ident | "*" | "**"          ; "**" only as the final segment
ident       ::= [a-z][a-z0-9_]{0,31}
limit       ::= ( "<=" | "<" ) quantity [ "/" window ]
quantity    ::= integer [ unit ]            ; FRC | quanta | MiB | GiB | count
window      ::= [ integer ] ( "s" | "m" | "h" | "day" | "week" )
selector    ::= "@" glob
```

| Example | Meaning |
|---|---|
| `society.chamber.post` | Post in any Chamber of the scoping Society |
| `society.chamber.post@general/**` | Post only under the `general` Chamber and its Threads |
| `wallet.transfer<=100FRC/day` | Up to 100 FRC per rolling 24h from the scoping Wallet |
| `wallet.transfer<=5FRC` | Up to 5 FRC per single action, no window |
| `vault.object.write<=2GiB/week` | Rolling volume cap |
| `society.member.*` | Every member action (invite, restrict, remove) — never `**` |

Rules the grammar enforces rather than documents:

- **Deny by default.** No matching Capability means denied (P8). Absence is never permission.
- `**` is legal only as the terminal segment and only from a grantor who already holds it. Since no principal holds bare `**` (I-12.6), universal capability is unreachable.
- A `limit` without a `window` is a per-action ceiling; with one it is a rolling total. Both may be attached to the same path; both apply.
- Limit units are typed. `wallet.transfer<=100MiB/day` fails to parse — unit confusion in a money path is a defect class removed at the grammar.

### 7.2 CapabilitySet algebra

```rust
struct Capability { path: Path, limit: Option<Limit>, selector: Option<Glob> }
struct CapabilitySet { allow: Vec<Capability>, deny: Vec<Capability> }

impl CapabilitySet {
    fn implies(&self, req: &Request) -> Decision;   // deny wins, then allow, else Deny
    fn join(&self, other: &Self) -> Self;           // ∪  union of allows and denies
    fn meet(&self, other: &Self) -> Self;           // ∩  ATTENUATION — the grant operator
    fn is_subset_of(&self, other: &Self) -> bool;   // the P4 escalation check
}
```

Granting is intersection, never assignment:

| Component | Intersection rule |
|---|---|
| Path | Concrete segment beats `*` beats `**`; disjoint paths yield ∅ |
| Limit | `min` after normalizing to the shorter window; `None ∩ Some(l) = Some(l)` |
| Selector | Glob intersection over the decidable subset (literal prefix + one trailing `**`). Anything outside it yields ∅ — conservative by construction |
| Deny | Denies from both sides are unioned into the result. A deny is never attenuated away |

**I-12.7 (P4, `11 §7.5`):** `envelope.capabilities.is_subset_of(grantor_capabilities)` — checked at grant time *and* again at evaluation, because the grantor's own set may have narrowed since. Escalation by delegation is structurally impossible, not policed.

**Honest cost.** Full glob intersection is undecidable, so the conservative rule occasionally denies a grant a human would consider obviously safe. We take the false negative: a permissive intersection in an authorization lattice is how escalation gets introduced by a clever optimization three years after the design review.

### 7.3 Evaluation order

Every command, from every front end, traverses this pipeline in the application layer (`10 §8`). The order is fixed; cheap and catastrophic checks come first.

```
 request ─►┌──────────────────────────────────────────────────┐
           │ 1. Principal state    Suspended/Departed → DENY   │
           │ 2. Authentication     token bound, fresh, DPoP OK │
           │ 3. Envelope validity  exists, unexpired, in scope │
           │ 4. Attenuation        grantor still holds it      │
           │ 5. Capability match   deny list, then allow list  │
           │ 6. Charter gate       does the role hold it HERE  │
           │ 7. Unlock gate        Level / Standing / Trust    │
           │ 8. Limits             rolling counters, atomic    │
           │                       reserve-then-commit         │
           │ 9. Confirm class      human confirmation pending? │
           └───────────────┬──────────────────────────────────┘
              ALLOW ───────┴─────── DENY
                │                    │
      execute; emit event      emit AgentActionBlocked /
      with envelope_ref        AuthorizationDenied; counts
                               against the Agent's Trust
```

Step 7 is where Trust participates in authorization, in exactly one direction. Trust may be a **precondition** for an Unlock (`01 §7`). Trust may never *substitute* for steps 3–6.

### 7.4 Caching

`10 §10` fixes the rule: cached per request, never per session.

| Artifact | Cacheable | Invalidation |
|---|---|---|
| Compiled matcher for a CapabilitySet | Yes, process-wide | Keyed `(envelope_id, envelope_version)`; revocation publishes an invalidation on the bus |
| Charter role → CapabilitySet | Yes, ≤ 60s | Keyed `(society_id, charter_version)` |
| Envelope validity (revoked? expired?) | **No** | Read at evaluation; revocation is retroactive to in-flight actions (`11 §2.8`) |
| Limit counters | **No** | Atomic reserve-then-commit; released on failure |
| The ALLOW/DENY decision | Per-request only | Never survives its request |

Caching a decision across requests is how revocation silently stops working. The extra read is worth it.

### 7.5 The PEP and the local-first tension

The Policy Enforcement Point lives in the application layer, inside the trust boundary, on the single path every command takes (`10 §8`). Gateways do coarse rate limiting; front ends do UX affordance; neither is an authorization decision, and both are assumed hostile.

P2 promises local reads while P8 demands authorized reads; `00 §P2` defers the resolution here. **Reads are capability-gated by key possession, not by a check.** A Node replicates only Societies its Citizen belongs to, and Chamber content is encrypted to a group its device is in. Removing a member removes their MLS leaf and rekeys, so they cannot read *future* content offline. They can still read what they already replicated — not a bug we can fix, but what "they already had the plaintext" means. Server-authoritative classes (wallet writes, Facet ownership, votes — `10 §6`) have no such tension: they never accept an offline write.

---

## 8. Trust

### 8.1 Three scores, never one

| Score | Scope | Range | Behaviour | From XP or Fraction? |
|---|---|---|---|---|
| **Trust** | Global, per principal | `-1000..=1000`, neutral `0` | Bidirectional; decays toward neutral | **Never** (I-12.8) |
| **Standing** | One Society (`11 §2.4`) | Tuple: contribution, trust `i32`, tenure, governance | Per-dimension | Never; `standing.trust ≠ f(contribution)` |
| **Agent Trust** | Global, per Agent | Same scale, damped by Operator Trust | Bidirectional, faster both ways | Never |

XP says *how much you did*. Trust says *whether you can be relied on*. Standing says *how that reads inside one Society*. Merging them into one number is the single change that would break the system (`01 §7`).

### 8.2 Inputs

Admitted inputs, each a discrete evidenced replayable event:

| Input | Sign | Weight |
|---|---|---|
| Custodian Attestation streak honored (`13`) | + | Low, high frequency |
| Escrow or Transfer commitment settled as agreed | + | Medium |
| Vouch received from an established Citizen (§8.5) | + | Medium, flow-capped |
| Governance commitment kept (delegation honored, proposal executed as written) | + | Medium |
| Appeal upheld in the Citizen's favour | + | High (restorative) |
| Moderation Action upheld on appeal | − | High |
| Stake slashed | − | High |
| `ReplicaCorrupt` / failed Attestation | − | Medium |
| `AgentActionBlocked` (Envelope violation attempt) | − | High for the Agent, medium for the Operator |
| Device-chain fork detected (§3.2) | − | Trust *suspended*, not decremented |
| Voucher slashed or vouch withdrawn | − | Cascading, damped |

**Structurally excluded**, enforced by I-12.8 and the property test at `11 §7.8`: XP, Level, Fraction balance, Transfer volume, message count, session time, follower counts, purchases, tenure alone, and Extension-supplied scores. The `TrustAdjustment` type does not accept them; the compiler is the enforcement, not a code review.

### 8.3 Update function, decay, bounds

```rust
const T_MAX: i32 = 1000;

fn apply(t: Trust, ev: &TrustEvent) -> Trust {
    let head  = (T_MAX - t.0.abs()) as f64 / T_MAX as f64;       // saturation headroom
    let delta = ev.sign() as f64 * ev.weight()
              * ev.source_damping()                              // vouch flow cap, §8.5
              * head.powf(1.5);
    Trust((t.0 as f64 + delta).clamp(-T_MAX as f64, T_MAX as f64) as i32)
}
```

The `head^1.5` term is the important one: the closer a principal is to a bound, the less any single event moves them. Trust 900 requires a long, varied history; Trust 300 does not. Negative movement saturates identically, so one bad event cannot destroy a decade of record — and a hundred of them can.

```
   +1000 ┤                    positive half-life: 180 days
         │   ╲___
       0 ┼─────────────────────────────────────────────  ← neutral, the attractor
         │            ___╱
   -1000 ┤            negative half-life: 365 days
```

Decay runs daily toward `0`, never below the absolute value justified by the last 90 days of evidence. The asymmetry is deliberate: **positive Trust must be continuously re-earned** (it is a claim about present reliability, and a dormant principal's reliability is unknown, not proven), while **negative Trust fades more slowly** but does fade — a permanent scarlet letter with no path back produces abandoned identities, and therefore more Sybils, not fewer. Bounds exist because unbounded reputation concentrates: early participants become unreachable and the score stops discriminating among everyone else.

### 8.4 The vouch graph

Vouching is the only social input and is deliberately expensive.

```
       established core (Trust ≥ 200, tenure ≥ 90d)
              │  VOUCH BUDGET = floor(sqrt(max(trust, 0)))
              │  spent per active vouch, refunded on withdrawal
              ▼
        ┌───────────┐  vouch: costs budget + a slashable 5 FRC bond
        │  Citizen  │──────────────────────────────►┌───────────┐
        └───────────┘                               │ newcomer  │
              ▲                                     └─────┬─────┘
              └──── vouchee slashed ⇒ voucher's bond ─────┘
                    slashed and Trust damped
```

- Trust conferred by vouching is **flow-limited**: positive Trust crossing any cut of the vouch graph is bounded by the budgets on that cut. Ten thousand fresh identities vouching for each other receive zero flow from the established core, because none of them is in it.
- Vouch weight decays with graph distance from the core and with the voucher's own vouch density. Vouching for a hundred people is worth far less per vouch than vouching for three.
- Vouches are public to the vouchee and auditable. Reciprocal rings are detectable as dense low-diameter subgraphs; detection reduces weight rather than banning, because a false-positive ban is far more expensive than a false-positive weight cut.

### 8.5 Agent Trust

Agents hold their own Trust (`11 §2.8`) so an Operator's reputation cannot launder an Agent's behaviour, and a bad Agent cannot permanently destroy its Operator.

- Starts at `0`, capped at `min(600, operator_trust)`. An Agent is never more trusted than its accountable human (P4).
- `AgentActionBlocked` decrements Agent Trust sharply and Operator Trust slightly. Repeated envelope probing is the signature of a misconfigured or hostile Agent and is meant to be expensive.
- Agent Trust gates nothing by itself. It informs humans authoring Policy, feeds Charter `agent_policy` thresholds, and multiplies rate limits. It never widens an Envelope — only a human signature does that (P4).

---

## 9. Sybil Resistance Without KYC

Identity creation stays free and pseudonymous; *capability* and *reward eligibility* do not. A farm can create a million FNIDs and gain nothing, because everything worth farming sits behind a cost the farm cannot pay a million times (P12).

| Mechanism | Sybil cost | Privacy cost | Failure mode | Verdict |
|---|---|---|---|---|
| Government KYC | Very high | Severe — a permanent identity database | Excludes the pseudonymous; contradicts the premise | Rejected for identity; required only at fiat Rails, Phase 9+, scoped to that Rail |
| Invitation tree | Medium | Low; reveals the inviter edge | Cold start; a farm captures one generous inviter | **Adopted**, with subtree accountability |
| Vouching graph, flow-capped | High for reward-bearing capability | Low–medium | Cliques, collusive rings | **Adopted** as the primary Trust input (§8.4) |
| Refundable stake bond | Linear in FRC, refundable to honest actors | None | Excludes the poor if priced as a gate to participation | **Adopted narrowly** — Handle claims and reward-bearing roles only; never to read, post, or join |
| Custodian work (proof of useful storage) | High — real hardware over time | None | Capital-rich attacker; unavailable before Phase 4 | **Adopted from Phase 4** as a Trust input |
| Proof-of-personhood vendor / biometrics | High | Severe, irreversible if leaked | The vendor becomes the identity root | Rejected |
| Device / platform attestation | Low–medium | Medium (hardware correlation) | Excludes rooted, Linux, hardened devices | Signal only, never a gate |
| Proof-of-work at registration | Low — the cheapest thing a farm owns | None | Punishes phones, not farms | Rejected |
| Phone number | Low (SIM farms are commodity) | High | False confidence, real exclusion | Rejected |

**Chosen default, Phases 1–3.** Anyone may create an FNID and a Persona and read public Societies with no gate. On top of that: (1) an **invitation tree with subtree accountability** — invites rate-limited by Level, concentrated abuse in a subtree damps the inviter's vouch budget, open registration still available at a lower initial rate limit; (2) **flow-capped vouching** as the route from new to trusted; (3) **narrow refundable bonds** on Handle claims and reward-bearing roles; (4) **rate limits and Level gates as the actual defence** — Emission Sources pay against Contribution Score (`17`), bounded per principal per window, so return per identity is capped below cost per identity by construction. That is the P12 falsification test, run in simulation at 100× adversarial volume.

**Honest cost.** This is probabilistic. A patient, well-funded adversary who builds genuine reputation on a dozen identities over a year succeeds at the scale of a dozen. We defend against economic farming and brigading at scale, not against a determined state actor. Saying otherwise would be a lie with a diagram attached.

---

## 10. Privacy

### 10.1 Who can observe what

| Datum | Public | Society members | Runtime operator | Custodians | Guardians |
|---|---|---|---|---|---|
| FNID, Handle, Level, global Trust | Yes | Yes | Yes | No | No |
| Society memberships | `Public` Societies only | Yes, in-Society | Yes | No | No |
| Message content, `EndToEnd` Chamber | No | Members only | **No — no decryption path exists (N6)** | No | No |
| Message content, `Transport` Chamber | No | Yes | Yes, necessarily — and labelled as such in the UI | No | No |
| Message metadata (who, when, which Chamber) | No | Yes | **Yes — see §10.3** | No | No |
| Wallet balance, Transfer graph | No | Society-scoped Postings only | Yes | No | No |
| Vault object bytes | No | Per ACL | Ciphertext only | Ciphertext Shards only | No |
| Declared interests | Per setting | Per setting | Yes | No | No |
| Guardian set | No | No | Encrypted references + policy hash | No | Own share only |
| IP address | No | No | Transiently, ≤ 7 days, abuse control only | Peer address during transfer | No |

### 10.2 Personas, selective disclosure, and the linkage limit

A Citizen has one FNID and one Persona per Society (`01 §2`). Personas change presentation, not identity: the FNID is the same everywhere, so cross-Society linkage is possible for anyone who can see both memberships.

That limit is deliberate, and the reasoning matters because the alternative is superficially attractive. Unlinkable per-Society credentials would give stronger pseudonymity and would simultaneously destroy portable Trust, portable Level, accountable vouching, and Sybil resistance — "unlinkable identity" and "one person, one accountable reputation" are the same property with opposite signs. We chose accountability and mitigate honestly: Society `visibility` controls whether membership is listed at all (`11 §2.2`); a Citizen controls per Society whether it appears on their Profile, **defaulting to hidden** (P9); no cross-Society activity graph is built, published, or exposed by any endpoint; and a Citizen needing genuine unlinkability creates a separate Citizen, which we neither forbid nor detect and which correctly starts at Level 0 with no Trust.

Profile modules, Standing dimensions, Achievements, wallet activity, and memberships are each disclosable to nobody, a named Society, shared Societies, or everyone — defaults set to the most private setting that leaves the feature working (P9). Every disclosure is a signed, revocable grant recorded as a domain event, so a Citizen can enumerate and revoke every disclosure they have ever made. Where a counterparty needs a *predicate* rather than a value — "Level ≥ 3", "Trust ≥ 200" — the API answers the predicate and never the value. That is not zero-knowledge; it is a narrower API surface, the honest and boring version of the same benefit. ZK credentials are a Phase 8+ consideration, architected for by keeping disclosure grants signed and predicate-shaped from the start.

### 10.3 What the platform can and cannot see

**Cannot:** plaintext of any `EndToEnd` Chamber, private call media, Vault object contents, or Shard contents. Not "does not" — there is no code path, and its absence is a security-review gate (N6).

**Can, and we will not pretend otherwise:** social-graph metadata. Who is in which Society, who posted where and when, message sizes and cadence, Ledger counterparties and amounts, and IP addresses transiently at the edge. Metadata resistance at mixnet grade is unreachable alongside P10's latency budget and P2's sync model, and claiming it would be exactly the dishonest privacy marketing this document exists to prevent. What we do instead: minimize retention (IP ≤ 7 days, no long-term connection logs), never build a behavioural profile from it (P9; `Citizen.interests` is declared-only, invariant `11 §7.13`), never sell or share it, and publish precisely what is retained in the telemetry catalogue.

---

## 11. Threat Model

| # | Attack | Mitigation | Residual risk |
|---|---|---|---|
| T1 | **Identity takeover via credential theft** | No passwords; origin-bound passkeys; every token key-bound (DPoP), so a stolen token is inert | Malware on an unlocked device with the authenticator present — unmitigable at our layer |
| T2 | **Guardian collusion** | `t ≥ ceil(n/2)+1`; delay-and-notify with unilateral device veto; Wallet frozen and grants blocked during the window; mixed guardian kinds | A Citizen with no working device *and* no reachable notify target loses the veto |
| T3 | **Handle squatting / homograph** | ASCII-only Handles; UTS #39 skeleton uniqueness; Level-or-bond to claim; reserved prefixes | Semantic impersonation (`@acme_support`) is not a character problem; Moderation Action, with latency |
| T4 | **Agent impersonating a human** | Agent messages always carry `envelope_ref` and are always visually distinct in every client (`11 §2.5`); Agent FNIDs registry-marked; `on_behalf_of` distinct from `author` | A human pasting Agent output is indistinguishable — and is the accountable author, which is correct |
| T5 | **Envelope escalation** | Attenuation is intersection, checked at grant and at evaluation (I-12.7); no principal holds `**`; a Charter cannot grant what the granting role lacks | A bug in glob intersection. Mitigated by conservative-deny and property tests over generated grant chains |
| T6 | **Sybil farm** | Rewards gated on flow-capped Trust, bonds, rate limits — never on identity count; invite subtree accountability; bounded Contribution Score | Patient reputation-building at small scale succeeds; accepted and bounded economically (§9) |
| T7 | **Replay of a signed command** | `idempotency_key` deduped per principal for 24h (`10 §5`); signatures cover `occurred_at`, `society_id`, and sequence context; DPoP nonces | Replay inside the dedupe window is idempotent by design — intended, not a hole |
| T8 | **Device key compromise** | Non-exportable hardware-backed keys; revocation kills sessions and API keys and triggers MLS rekey (post-compromise security) | Everything read or done before detection. Detection latency *is* the exposure; anomalous-device Signals shorten it |
| T9 | **Identity key compromise** | Device-quorum emergency rotation under a mandatory delay window; the rotation chain makes an attacker's rotation publicly visible | An attacker holding the identity key *and* a device quorum controls the identity; recovery is the last line |
| T10 | **Malicious Node serving false state** | Every Message author-signed (`11 §7.9`); hash-chained logs periodically Anchored; clients verify inclusion proofs and never trust a Node's assertion of membership or balance | A malicious Node can *withhold* data. Detected as a stalled `seq`; not preventable by one peer |
| T11 | **Split-view / equivocation on the device chain** | `prev` collision is fatal: freeze both branches, terminate sessions, force recovery; Anchors are published and cross-checkable | Detection needs at least one honest observer; Anchor frequency bounds the window |
| T12 | **Coerced disclosure** (legal order or physical) | The Runtime cannot produce E2EE plaintext at any price (N6); shares are useless below threshold and are socially and geographically diversifiable; delay-and-notify makes covert coerced recovery visible | Metadata is producible and will be produced under valid legal order; `Transport` Chambers are readable. Stated plainly in the transparency policy |
| T13 | **Vouch-graph capture** | `sqrt` budget limit; weight decays with density and distance; slashing cascades to vouchers; vouches are public and withdrawable | A genuinely trusted, genuinely malicious hub can elevate one cohort, at the cost of its own record |
| T14 | **PEP bypass via a front end** | The PEP is in the application layer on the only path (`10 §8`); the gateway performs no authorization; the CLI exercises the identical path (P13), so a GUI-only shortcut cannot exist unnoticed | An application-layer bug is a bug everywhere at once. Accepted: one auditable enforcement point beats five that drift |

---

## 12. Trade-offs and Rejected Alternatives

| Decision | Serves | Honest cost | Rejected |
|---|---|---|---|
| Self-certifying FNID | P2, P8, P11 | 59-char identifiers; no global revocation oracle; loss without recovery is final | DIDs (spec surface, DNS root); chain addresses (P11); UUIDs (unverifiable offline) |
| Immutable FNID + rotation chain | P6, P8 | Verifiers fetch chain segments; offline verification degrades after rotation | Rotating the identifier (breaks every reference); no rotation (keys age into compromise) |
| ASCII-only Handles | Anti-impersonation | Non-Latin-script Citizens cannot use their script in a Handle — a real exclusion, mitigated by Unicode display names | Unicode Handles with heuristic detection (heuristics lose) |
| Passkeys, no passwords | P8 | Worse signup conversion; uneven cross-ecosystem UX | Passwords + MFA (keeps stuffing and reset-backdoor classes); magic links (email becomes the root) |
| Social recovery with delay-and-notify | P8 | 72h to recover; coordination burden in the most stressful flow | Custodial recovery (Never-list); seed phrases alone (Citizens lose them); no recovery (converts every lost device into a churned Citizen) |
| Capability strings with parameterized limits | P4, P8 | A grammar, parser, algebra, and intersection edge-case surface to test forever | RBAC alone (cannot express `≤100 FRC/day`); ACL matrices (do not attenuate); OAuth scopes (flat, no delegation algebra) |
| Trust bounded, decaying, non-purchasable | P12 | Long-tenured Citizens will object that decay erodes earned standing; the answer is that reliability is present-tense | Unbounded reputation (concentrates); purchasable Trust (Never-list); XP-derived Trust (`01 §7`) |
| Sybil resistance without KYC | P9 | Probabilistic; a patient adversary wins at small scale | KYC (identity database, exclusion); PoP vendors (vendor becomes the root); PoW (punishes the honest) |
| One FNID across Societies, Personas for presentation | Accountability | Cross-Society linkage is possible for anyone who sees both memberships | Per-Society unlinkable credentials (forfeits portable Trust and Sybil resistance at once) |
| Metadata visibility stated openly | Honesty | We cannot claim metadata privacy, and competitors will | Claiming mixnet-grade privacy we do not deliver |

---

## 13. Invariants Enforced in Code

These join `11 §7`. Each becomes a property test that runs on every PR.

| # | Invariant |
|---|---|
| I-12.1 | An `Fnid` is never mutated after creation, for any principal class |
| I-12.2 | Every device-chain entry verifies against a key `Active` at its `prev` position |
| I-12.3 | Two entries sharing a `prev` freeze the chain; no merge path exists in the code |
| I-12.4 | A Citizen cannot reach zero `Active` devices with no `RecoveryConfig` |
| I-12.5 | A signature verifies iff its key was current at its `occurred_at`; rotation never invalidates historical signatures |
| I-12.6 | No credential of any kind resolves to an unbounded CapabilitySet |
| I-12.7 | `envelope.capabilities ⊆ grantor.capabilities`, at grant time and at evaluation time |
| I-12.8 | No Trust write accepts an input derived from XP, Level, Fraction, or volume — enforced at the type level |
| I-12.9 | Every authorization decision is produced by the application-layer PEP; no other module returns `Decision` |
| I-12.10 | Every `RecoveryCompleted` follows a `RecoveryInitiated` by at least `delay`, with no intervening cancellation |
| I-12.11 | Agent Trust never exceeds `min(600, operator_trust)` |
| I-12.12 | No two `Active` Handles share a UTS #39 skeleton |

---

## 14. What Would Make Us Change This

- **Passkey abandonment exceeds 25% at signup.** → Add a hardware-key-only or recovery-share-first onboarding path. Do **not** add passwords; that trade is not available.
- **Guardian recovery completion falls below 60%.** → The model is failing in practice, not in theory. Default to Society-role guardians, shorten the delay for Citizens holding no treasury or governance role, and invest in guardian-side UX before weakening the threshold.
- **The vouch graph is captured** — a rising share of positive Trust flowing through a shrinking set of cut vertices. → Lower budget exponents, raise density damping, pull Custodian work forward from Phase 4 as an alternative Trust source.
- **Capability strings prove insufficient** for a real Charter (typically relational constraints such as "only in Chambers I created"). → Add a narrow, decidable, side-effect-free predicate language at step 6. Do **not** admit a general expression language into the authorization path.
- **A regulator demands identity verification for the platform rather than for a fiat Rail.** → Scope it to that Rail and jurisdiction as an adapter (P5), storing the verified attribute as a predicate and never as a document. If that is unacceptable to the regulator, the Rail does not ship there.
