Executed all six ratified decisions; each conformed to repository authority.

- Added local drafts [27](/tmp/reuse-evolution-grill-r1.yY6ISq/issues/27-manual-capture.md), [28](/tmp/reuse-evolution-grill-r1.yY6ISq/issues/28-write-free-clean-capture.md), [29](/tmp/reuse-evolution-grill-r1.yY6ISq/issues/29-optional-sensors.md), [30](/tmp/reuse-evolution-grill-r1.yY6ISq/issues/30-external-implementation-handoff.md), and [31](/tmp/reuse-evolution-grill-r1.yY6ISq/issues/31-decision-surface.md).
- Added **Decision surface** to [CONTEXT.md](/tmp/reuse-evolution-grill-r1.yY6ISq/CONTEXT.md:28).
- Added the dated clarification to accepted [ADR 0020](/tmp/reuse-evolution-grill-r1.yY6ISq/docs/adr/0020-what-the-capture-skill-may-not-own.md:7), preserving its status and decision.
- Did not add hooks, automation, durable clean-capture records, detectors, mandatory providers, CI enforcement, refactor execution, ADR 0021, or ADR-index changes.
- No unresolved, unavailable, or pending evidence remains.

Domain-document outcome: the project-layer glossary term crystallized and was recorded in `CONTEXT.md`; no new ADR was needed because decision 6 correctly amended ADR 0020’s existing decision home.

Artifact class: local issue/process documentation authorized by the ratified work package.  
Provenance: `ratified-work.md`, verified against the foundational principles, accepted ADRs 0001–0020, and `docs/design/v0.1-scope-and-acceptance.md`.  
Convention checked: every issue has all six required headings; the ADR index is unchanged and no mirror or pointer update applies.

Validation: `git diff --check` passed. The fixture has no `Cargo.toml`, so no Rust test suite applies.