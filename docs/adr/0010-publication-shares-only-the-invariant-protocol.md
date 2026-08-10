# ADR 0010: Publication shares only the invariant protocol

**Status:** Accepted  
**Date:** 2026-08-10  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

ADR 0009 requires every later-event writer to take an exclusive lock on the immutable opening event, re-read and validate the case revision and operation eligibility while holding it, and hold it through exclusive creation of the typed next event file. That protocol is stated once as authority and implemented twice: `case::append` (`src/case.rs:402`) and `case::authorize_early_review` (`src/case.rs:516`).

The two implementations have already diverged. On an exact idempotent retry whose participants cannot be resolved because no portfolio root is configured, `existing_append` propagates the failure (`src/case.rs:920`) and the command refuses; `existing_early_review` swallows it and reports `privacy: private` (`src/case.rs:683`) and the command succeeds. Neither branch is covered by any test: `run_without_portfolio_configuration` in `tests/case_cli.rs` is used only for `case show` and `case list`. One question about a privacy receipt therefore has two shipped answers, and the suite cannot tell.

That is the observed pressure. It is not an argument from predicted convenience: the remaining later event types named in [`design/v0.1-scope-and-acceptance.md`](../design/v0.1-scope-and-acceptance.md) §2 have no open issues, and this decision does not rest on them.

An architectural review of the case modules recommended collapsing the whole pipeline — proposal parsing, content validation, eligibility, privacy timing, and receipt fields alongside the protocol — into one parameterized module, removing roughly 970 of `src/case.rs`'s 1802 lines.

The project's own thresholds bear directly on that recommendation. `FOUNDATIONS.md` §5 counts independently accepted consumer needs; §6 treats a second occurrence as watching and a third as normally worth reviewing; §4 says to share only the invariant behavior whose authority can be owned coherently, and to keep implementations separate where triggers, invalidation rules, or reasons to change differ; §9 says the narrowest valid reuse scope wins. ADR 0002 records the threshold itself.

Applied to this crate's own internals, the protocol has two occurrences and a rule already written down as one rule. The surrounding concerns have two occurrences and visibly different shapes: `append`'s eligibility check derives privacy as part of validating the proposed occurrence, while `authorize_early_review` derives privacy separately and before the revision check; `append` derives its receipt's readiness from the recorded occurrence count, while `authorize_early_review` states it as a constant.

Without this decision recorded, the restraint leaves no trace. A module owning half the pipeline, with duplicated proposal readers, recorded-event validators, and outcome types standing beside it, reads as unfinished work rather than as a boundary.

## Decision

A shared internal publication module absorbs only the behavior ADR 0009 already states as one rule.

- The module owns sequence selection from an expected revision, the exclusive lock on the opening event, the re-read under that lock, the expected-revision check, exclusive creation of the typed event file, the occupied-path outcome, and the idempotent-retry comparison by recorded event identity and exact bytes.
- The retry comparison reads a permissive recorded envelope. It takes no per-event-type parameter.
- Proposal documents and their content validation, eligibility rules against the current case, when and how case privacy is derived, and receipt fields remain owned by each event type.
- Opening a case is not a publication and is not absorbed. It takes no lock, has no expected revision, and compares an existing record by semantic fields rather than by event identity and bytes.
- The publication ordering is enforced structurally: creating an event file is reachable only from a value that holds the lock and has re-read the case.
- This authorizes sharing the protocol. It does not authorize unifying proposal parsing, eligibility, privacy derivation, or receipts across event types, and it does not authorize extracting anything from this crate into a shared package.

This records a boundary on internal sharing only. It makes no claim about a user's portfolio and creates no new consumer-facing surface.

## Consequences

### Positive

- ADR 0009's publication protocol has one implementation, so a later event type cannot omit the lock or the re-read.
- The retry-privacy divergence is resolved deliberately rather than by whichever copy a reader happens to open.
- The concerns most likely to differ between event types stay separable, so a third event type is free to need a different shape without a parameter being retrofitted to fit it.
- The absorbed module needs no clock, environment, or portfolio scan, so it is directly testable.

### Negative and risks

- Duplication remains in proposal readers, recorded-event validators, and outcome types. This is deliberate and will look like incompleteness.
- If the third later event type turns out to match the existing two on every remaining axis, the restraint will have cost one extra round of consolidation.
- A boundary defined by "what ADR 0009 already states" needs re-reading whenever ADR 0009 is amended.

### Operational burden

Adding a later event type means implementing its proposal, eligibility, privacy, and receipt, then publishing through the shared protocol. The protocol itself is not re-implemented or re-reviewed.

### Compatibility and migration

No recorded evidence changes. Event files, filenames, sequence semantics, and schema versions are untouched, so `CONSUMER-CONTRACT.md` §3 is not engaged. One receipt changes: an exact retry that cannot resolve participants reports `privacy: unknown` with the existing portfolio-unavailable footer instead of refusing or asserting `private`. That is a command-surface change permitted by `CONSUMER-CONTRACT.md` §8 during `0.x`, and it preserves the idempotency `FOUNDATIONS.md` §11 requires of side effects.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Collapse the whole pipeline behind one parameterized module | Rejected | The eligibility, privacy-timing, and receipt shapes would be parameterized from two occurrences that already differ in those respects, against `FOUNDATIONS.md` §4 and §9. |
| Leave both implementations and fix only the divergence | Rejected | The protocol stays duplicated, so the next later event type can drift in the same way and nothing prevents it. |
| Defer all sharing until a third later event type exists | Rejected | The divergence is present and untested now, and the protocol's authority is already written as one rule. Waiting means a third copy is written before it is removed. |
| Record the restraint in the implementing issue instead of an ADR | Rejected | `docs/adr/README.md` ranks ADRs authoritative over issues, and an issue closes. A restraint whose only on-disk evidence is an absence would be reversed without anyone knowing it existed. |
| Generalize this into a rule that the project applies its own occurrence thresholds to all of its internals | Rejected | That would abstract a standing rule from one instance, which is the error this decision restrains. |

## Verification and review trigger

The decision is fit if a third later event type is implemented by supplying its own proposal, eligibility, privacy, and receipt while reusing the publication protocol unchanged, and if the retry-privacy question has exactly one answer afterwards.

Reopen when a third later event type exists and its proposal shape, eligibility rule, privacy timing, and receipt fields match the existing two closely enough that a parameterization would couple nothing that changes for different reasons. At that point the wider collapse has three occurrences behind it and should be reconsidered on its merits.

Falsify this decision if implementing the protocol module forces any event type's eligibility or privacy derivation to move to a point in the sequence it does not belong, or if the shared retry comparison cannot be expressed without a per-event-type parameter. Either result would mean the protocol is less invariant than ADR 0009 states, and the duplication is the cheaper cost.

## Supersession

None.
