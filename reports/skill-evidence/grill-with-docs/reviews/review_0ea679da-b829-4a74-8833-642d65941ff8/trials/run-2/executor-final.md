Executed all six ratified decisions in order; no authority mismatch changed what ships.

- Added five local issue drafts:
  - [27-manual-capture.md](/tmp/reuse-evolution-grill-r2.9TcCVZ/issues/27-manual-capture.md)
  - [28-write-free-clean-capture.md](/tmp/reuse-evolution-grill-r2.9TcCVZ/issues/28-write-free-clean-capture.md)
  - [29-optional-sensors.md](/tmp/reuse-evolution-grill-r2.9TcCVZ/issues/29-optional-sensors.md)
  - [30-external-implementation-handoff.md](/tmp/reuse-evolution-grill-r2.9TcCVZ/issues/30-external-implementation-handoff.md)
  - [31-decision-surface.md](/tmp/reuse-evolution-grill-r2.9TcCVZ/issues/31-decision-surface.md)
- Added **Decision surface** to [CONTEXT.md](/tmp/reuse-evolution-grill-r2.9TcCVZ/CONTEXT.md:27).
- Added the dated clarification while preserving accepted status and the existing decision in [ADR 0020](/tmp/reuse-evolution-grill-r2.9TcCVZ/docs/adr/0020-what-the-capture-skill-may-not-own.md:7).
- Did not create ADR 0021, alter the ADR index, publish issues, add automation, or access anything outside the fixture.

Ratified in scope: five local drafts, one glossary term, and one non-superseding ADR amendment. Out of scope remained hooks, new commands/events/schemas, built-in detection, CI enforcement, refactor execution, implementation routing, and external publication.

Domain-document outcome: `CONTEXT.md` was updated with the newly crystallized project-layer term; ADR 0020 received the ownership clarification. No other domain-document change was needed.

Artifact class: local, non-product planning and authority documentation.  
Provenance: `ratified-work.md` and the repository principles, accepted ADRs, active v0.1 design, and glossary justified these artifacts.  
Convention checked: every issue has all six required headings; ADR status remains `Accepted`.  
Proof: validation assertions and `git diff --check` passed. No repository completion contract exists in this fixture, so no product test suite was applicable.