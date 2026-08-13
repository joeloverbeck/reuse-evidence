# ADR 0001: Manual, case-based evidence lifecycle

**Status:** Accepted  
**Date:** 2026-08-09  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`EVIDENCE-AND-DECISIONS.md`](../principles/EVIDENCE-AND-DECISIONS.md)

## Context

The maintainer needs confidence that reuse capture actually happened, but the preceding skill-audit system demonstrated how quickly per-run certification and accumulated paperwork can become the product instead of supporting it.

Reuse decisions do not need a denominator of all coding sessions. Most material implementations will not create qualifying repeated responsibility. Recording every clean scan or first use would create status volume without improving a live decision.

Automatic end-of-session hooks would also risk hidden scans, noise, premature cases, and pressure to refactor simply because the agent was asked to inspect.

## Decision

`reuse-evidence` uses a manual, case-based lifecycle.

- The maintainer explicitly invokes capture after material implementation work.
- A completed no-candidate capture returns a fixed terminal result and writes nothing.
- No durable reuse state exists for a first occurrence.
- A case begins only when at least two plausible independent occurrences are supported by recoverable evidence.
- Authoritative case facts and decisions are written only through the compiled command with expected-revision checks, idempotency, atomicity, and receipts.
- Conversation or transcript context may help during a contemporaneous run but is not the durable authority.
- No Stop hook, post-commit hook, daemon, or background process creates evidence in version 0.1.

## Consequences

### Positive

- Routine use stays cheap.
- Case history contains decision-bearing pressure rather than a certification denominator.
- The maintainer knows capture was intentionally invoked.
- Evidence writes are explicit and inspectable.
- The design avoids recreating the bureaucratic failure that motivated the project.

### Negative and risks

- Capture can be forgotten.
- The system will not have statistics about how often no reuse was found.
- A later retrospective case may require reconstructing the first occurrence from Git evidence.

These costs are accepted. A reminder-only hook may be considered later if real missed-capture evidence outweighs the noise and complexity.

### Operational burden

An ordinary clean run should require one invocation, bounded investigation, one terminal result, and no committed artifact.

### Compatibility and migration

No legacy repository must generate historical clean receipts or first-use inventories. Existing evidence may be imported only for a selected real case.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Record every qualifying coding session | Rejected | Creates a denominator that reuse decisions do not consume. |
| Automatic Stop-hook capture | Rejected for version 0.1 | Hidden cost, noise, and premature evidence risk. |
| Periodic scan only | Rejected as sole lifecycle | Loses contemporaneous task context and timely second-occurrence memory. |
| Transcript-led authoritative evidence | Rejected | Formats are unstable and conversation is not durable repository authority. |

## Verification and review trigger

Reopen this ADR if ordinary capture is frequently missed in real use and a bounded reminder experiment demonstrates lower total burden without automatic evidence creation.

## Supersession

None.
