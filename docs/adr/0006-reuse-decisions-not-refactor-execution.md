# ADR 0006: Reuse decisions, not refactor execution

**Status:** Accepted  
**Date:** 2026-08-09  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](../principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md)

## Context

The maintainer already uses strong engineering skills for architecture design, TDD, implementation, and code review. Reproducing those workflows inside `reuse-evidence` would create a weaker duplicate, blur responsibility, and tempt the reuse lifecycle to treat implementation completion as proof that its decision was correct.

The reuse system still needs to carry an accepted decision into engineering work and later verify whether the authorized result landed across all consumers.

## Decision

`reuse-evidence` owns evidence, readiness, semantic review, accepted reuse decisions, implementation briefs, verification, and closure.

It does not implement refactors.

- Review may consult architecture vocabulary and package research.
- An accepted decision produces a bounded implementation brief.
- The repository's normal engineering workflow owns interface design, specifications, tickets, TDD, code changes, migrations, review, and commits.
- `reuse-evidence` has no package dependency on Matt Pocock's skills or any specific external workflow.
- Interoperability occurs through durable briefs and explicit verification conditions.
- After implementation, `reuse-evidence` independently verifies the accepted scope and consumer behavior.
- A verification failure reopens or parks the case rather than editing code automatically.

## Consequences

### Positive

- Each capability owns one coherent result and lifecycle.
- Existing high-quality engineering skills remain reusable.
- The reuse decision can be judged independently from implementation enthusiasm.
- Other users can adopt different engineering workflows.
- The project does not become an orchestration suite.

### Negative and risks

- The user performs an explicit handoff between review and implementation.
- Verification must recover the accepted decision and implementation result later.
- An external workflow may ignore or misinterpret the brief.

Mitigation: the brief is exact, versioned, and contains verification conditions; the case remains open until independent verification.

### Operational burden

The brief must be compact and decision-bearing. It should not duplicate every ticket or implementation step. A peer skill name is not durable routing authority.

### Compatibility and migration

External skill sets may evolve without forcing this crate to update, provided they can consume ordinary Markdown or equivalent durable briefs.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Add a `reuse-extract` implementation skill | Rejected | Overlaps existing workflows and gives the project two responsibilities. |
| Automatically refactor after decision acceptance | Rejected | Unsafe across private repositories and conflates authority with execution. |
| Produce only a finding and never verify | Rejected | Leaves migration completeness and abstraction fitness unobserved. |
| Depend directly on Matt Pocock's repository | Rejected | Unnecessary coupling and excludes other workflows. |

## Verification and review trigger

Reopen only if repeated implementation failures are caused by an information gap that a bounded improvement to the implementation brief cannot solve. Do not infer that the reuse lifecycle should own implementation merely because an external run failed.

## Supersession

None.
