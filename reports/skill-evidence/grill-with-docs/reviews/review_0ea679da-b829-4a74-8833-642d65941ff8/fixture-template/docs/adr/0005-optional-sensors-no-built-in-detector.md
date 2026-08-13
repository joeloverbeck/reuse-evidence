# ADR 0005: Optional sensors, no built-in detector

**Status:** Accepted  
**Date:** 2026-08-09  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](../principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md)

## Context

Mature tools already detect exact, structural, fuzzy, semantic, cross-language, and cross-project duplication. The open problem for this repository is not vector search or AST extraction; it is deciding whether independent consumers share one responsibility, preserving evidence over time, handling privacy, and authorizing the right scope.

Embedding a detector would add language parsers, model downloads, threshold calibration, cache formats, GPU or CPU performance concerns, and false-positive policy before the case lifecycle itself is proven.

At the same time, optional detector output can improve discovery in large repositories.

## Decision

`reuse-evidence` treats detectors and other analyses as optional sensors.

- The core project does not build an exact, AST, fuzzy, embedding, or cross-project clone detector in version 0.1.
- No detector is a mandatory dependency.
- Skills may invoke an installed external sensor when available and appropriate.
- Sensor output is retained by recoverable reference or a minimal imported result, not as copied source or committed embeddings.
- A sensor score proposes candidates only. It does not establish responsibility identity, independence, readiness, or a decision.
- Ordinary Git, source search, history, tests, and agent reasoning remain a valid path without a sensor.
- There is no generalized sensor plugin framework in version 0.1.
- A common adapter contract may be designed only after at least two real integrations create repeated pressure for one.

## Consequences

### Positive

- Development remains focused on the distinctive evidence lifecycle.
- The project avoids duplicating mature detector work.
- Users can choose local, remote, exact, or semantic tools according to their privacy and repository needs.
- False-positive judgment remains explicit.

### Negative and risks

- Discovery quality varies with available tools and agent skill.
- External sensor output formats may change.
- Some users may expect an all-in-one scanner.

These are accepted. The README and skills must state the boundary honestly.

### Operational burden

Sensor use must be bounded and decision-relevant. Do not cold-index every repository after every session or require large model downloads for clean capture.

### Compatibility and migration

Sensor reports are evidence references, not authoritative case schemas. Losing a sensor cache must not corrupt case history. Remote sensors require separate privacy authority.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Build a new Rust semantic detector | Rejected | Duplicates existing work and distracts from the product's unique value. |
| Depend mandatorily on one existing detector | Rejected | Adds heavy operational and compatibility coupling. |
| Ignore detectors entirely | Rejected | They can materially improve candidate discovery in mature codebases. |
| Build a generic sensor-plugin framework immediately | Rejected | Predicted integration before repeated pressure. |
| Fail CI on sensor candidates | Rejected | Similarity is not semantic authority. |

## Verification and review trigger

Reopen if repeated real cases cannot be found with external sensors and bounded agent analysis, and the failure can be traced specifically to missing detector capability rather than review, configuration, or evidence semantics.

## Supersession

None.
