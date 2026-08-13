# ADR 0002: Independent-consumer thresholds

**Status:** Accepted  
**Date:** 2026-08-09  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`EVIDENCE-AND-DECISIONS.md`](../principles/EVIDENCE-AND-DECISIONS.md)

## Context

Counting matching functions, files, or code locations would overstate reuse pressure. Generated copies, coordinated variants, retries, and one implementation split across modules can create many locations while expressing one consumer need.

At the same time, waiting until a third code copy already exists can be too late when two large or compatibility-sensitive consumers have begun to diverge.

The system needs a default threshold that restrains premature abstraction without turning the number three into dogma or extraction authority.

## Decision

The evidence unit is the **independent reuse consumer**, not a code location or agent session.

- First independent occurrence: no durable reuse case.
- Second independent occurrence: a case may open in `watching` state.
- Third independent occurrence: the case normally becomes `review-ready`.
- Review readiness authorizes semantic review only. It does not choose extraction.
- The human may authorize review after the second occurrence when a concrete cost or risk is recorded with evidence and a bounded review appetite.
- The early-review override cannot directly authorize implementation.
- The threshold governs creating and owning a new abstraction. It does not prohibit use of an existing mature external dependency on the first consumer.

Independence must be justified through consumer authority, lifecycle, release, or reason to change. Repository count is not conclusive.

## Consequences

### Positive

- The rule of three remains a strong default without becoming a mechanical abstraction rule.
- Generated or coordinated copies do not inflate evidence.
- Cross-repository and intra-repository consumers can be treated consistently.
- High-cost second-use situations can be reviewed without pretending a third consumer exists.

### Negative and risks

- Independence requires semantic judgment.
- Agents may disagree about whether two contexts are distinct consumers.
- Early-review reasons could become a loophole if they are vague.

Mitigation: uncertainty records nothing or keeps the case watching; the human accepts every override and later decision.

### Operational burden

Capture should record only the consumer facts needed to explain independence. It must not create organizational charts or exhaustive dependency maps.

### Compatibility and migration

Historical cases may reconstruct occurrences from existing commits. Repeated locations must not be imported as separate occurrences without consumer evidence.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Count code locations | Rejected | Confuses shape with independent pressure. |
| Count repositories | Rejected | One repo can have several consumers; several repos can be one coordinated consumer. |
| Hard third-use rule with no override | Rejected | Can impose avoidable compatibility or coordination cost. |
| Review automatically on second use | Rejected | Too vulnerable to overfitting two examples. |
| Extract automatically on third use | Rejected | The threshold establishes review value, not semantic identity or scope. |

## Verification and review trigger

Reopen if real cases show that the second/third threshold consistently misses costly divergence or produces too many low-value reviews. Evidence must distinguish threshold failure from poor candidate discovery or review quality.

## Supersession

None.
