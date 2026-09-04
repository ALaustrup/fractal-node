# Security Policy

## Reporting

Report vulnerabilities privately to **security@fractalnode.net**. Do not open a public issue.

Include: what you found, how to reproduce it, and what an attacker could do with it. We will
acknowledge within 72 hours and give you a remediation timeline within 7 days.

## Scope and posture

Fractal Node's security posture is defined by three Foundational Principles, and a report that
demonstrates a violation of any of them is in scope regardless of exploitability:

- **P8 Secure by Default** — deny by default; every capability explicit, scoped, time-boxed, revocable.
- **P9 Privacy by Default** — minimum collection; most-private defaults; no covert behavioural signals.
- **P4 AI-First, Human-Governed** — an Agent that widens its own authority is a critical finding.

Highest-severity classes, in order:

1. **Envelope escalation** — any grant conferring a capability the grantor did not hold.
2. **Policy Enforcement Point bypass** — any path that mutates state without passing the PEP.
3. **Ledger invariant violation** — unbalanced Postings, or emission outside the published cap.
4. **Server-side plaintext access** to end-to-end encrypted content (N6).
5. **Extension sandbox escape** — a WASM component exceeding its declared manifest.
6. **Cross-Society data leakage** — any read crossing a `society_id` boundary without authorization.

## What we will not do

We will not ask you to delay disclosure beyond 90 days, and we will not pursue researchers acting
in good faith within this policy.
