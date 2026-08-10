# Architectural Decision Records

Accepted ADRs record decisions that future contributors might otherwise reverse or unknowingly contradict. They are subordinate to `docs/principles/` and authoritative over bounded design documents, PRDs, issues, skills, schemas, and code.

## Status meanings

- **Proposed** — informative only; not authority.
- **Accepted** — binding until superseded or amended.
- **Superseded** — retained as history; the named successor governs.
- **Rejected** — considered and not adopted.

Only the human decision owner can accept, supersede, or reject an ADR. Merge, implementation, tests, and publication do not change ADR status by themselves.

## Accepted decisions

| ADR | Decision |
|---|---|
| [0001](0001-manual-case-based-evidence-lifecycle.md) | Manual, case-based evidence lifecycle with no clean-run records. |
| [0002](0002-independent-consumer-thresholds.md) | Independent consumers are counted; second occurrence watches, third normally reviews. |
| [0003](0003-marker-enrolled-local-portfolio.md) | Local portfolio roots discover only marker-enrolled repositories. |
| [0004](0004-fixed-steward-and-private-dominance.md) | One fixed steward owns a case; one private participant makes it private. |
| [0005](0005-optional-sensors-no-built-in-detector.md) | Sensors are optional evidence; the project does not build a detector in version 0.1. |
| [0006](0006-reuse-decisions-not-refactor-execution.md) | Reuse review produces decisions and briefs; normal engineering performs refactors. |
| [0007](0007-language-neutral-evidence-rust-first-delivery.md) | Evidence semantics are language-neutral while delivery is Rust-first. |
| [0008](0008-published-skill-evidence-dependency.md) | The published `skill-evidence` crate governs this repository's own skills; ADR 0006's package-dependency clause is scoped to engineering-workflow skill sets. |
| [0009](0009-per-event-case-evidence-files.md) | Case events are sequence-numbered single-event TOML files; exclusive create is the expected-revision check. |

## Adding or changing an ADR

Use [`ADR-TEMPLATE.md`](ADR-TEMPLATE.md). Keep the decision as small as the real pressure allows. Name the governing principle, live decision, alternatives, consequences, compatibility effect, and supersession path.

If an ADR would conflict with a principle, amend the principle first. Do not use an ADR to smuggle a constitutional change beneath the authority layer.
