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
| [0010](0010-publication-shares-only-the-invariant-protocol.md) | Publication shares only ADR 0009's invariant protocol; proposal, eligibility, and privacy derivation stay per event type, while the event-receipt spine may take one owner. |
| [0011](0011-one-owner-for-case-event-naming.md) | Case event naming has one owner reached one way by publication and projection; the `case.rs`/`case/read.rs` boundary is not re-cut. |
| [0012](0012-implementation-brief-is-a-projection.md) | The accepted decision is the single durable record; the implementation brief is projected from it and never authored. |
| [0013](0013-one-outcome-carrier-for-later-case-events.md) | The three later case event types share one outcome carrier and effect enum; opening keeps its own shape. |
| [0014](0014-one-type-for-the-case-event-envelope.md) | The case event envelope has one type and one validator, flattened into each body; recorded bytes and refusal wording are unchanged. |
| [0015](0015-portfolio-location-resolved-at-the-edge.md) | The portfolio location is resolved once at the edge as a value that carries its own absence; each consumer keeps its own interpretation, so the `portfolio` refusal and the `case list` degraded projection both stand. |
| [0016](0016-terminal-contract-at-the-process-boundary.md) | The process boundary tests the terminal contract; behaviour is tested across the module interface. New behaviour goes in process; existing CLI tests are not converted. |
| [0017](0017-one-owner-for-case-terminal-text.md) | Every case renderer moves to `case::render`, reached one way; the receipt spine and shared readiness vocabulary take the owner ADR 0010 permitted, and the brief's panic-on-drift becomes a type. |
| [0018](0018-one-owner-for-marker-fault-meaning.md) | Classifying a non-supported marker read has one owner: every marker fault is a refusal naming which fault it is, with the resolution supplied by the command that ran. Resolves the divergence ADR 0016 §43 deferred. |
| [0019](0019-one-event-records-verification-and-its-disposition.md) | One `verification_recorded` event records the verification evidence and its disposition in orthogonal fields; no separate close, park, or reopen event type exists. A parked or reopened case re-verifies against the standing decision. |
| [0020](0020-what-the-capture-skill-may-not-own.md) | The command owns a capture mechanic whose silent failure would corrupt an authoritative record or lose a recorded human decision; the skill owns what the command cannot verify. Cross-portfolio case lookup and prepared-proposal path resolution are compiled; the fixed no-candidate terminal statement is not, because ADR 0017's owner covers how a case prints and a clean capture has none. |
| [0021](0021-what-the-project-owned-installer-ships-and-may-not-borrow.md) | The project-owned skill installer is this crate's own code under a command outside the mounted `skills` tree, and it ships one copy of each package embedded straight from `.claude/skills/`, with no `assets/` mirror. The published crate carries a narrow named file set. |
| [0022](0022-one-owner-for-recorded-case-history-replay.md) | Recorded-history replay takes one owner, `case::replay`, reached one way by the query path and the writer path. It performs no filesystem access: the event reader is injected, which preserves the current refusal ordering that eagerly loading the stream would change. |

## Adding or changing an ADR

Use [`ADR-TEMPLATE.md`](ADR-TEMPLATE.md). Keep the decision as small as the real pressure allows. Name the governing principle, live decision, alternatives, consequences, compatibility effect, and supersession path.

If an ADR would conflict with a principle, amend the principle first. Do not use an ADR to smuggle a constitutional change beneath the authority layer.
