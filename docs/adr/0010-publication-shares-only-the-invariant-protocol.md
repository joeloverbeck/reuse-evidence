# ADR 0010: Publication shares only the invariant protocol

**Status:** Accepted  
**Date:** 2026-08-10  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

ADR 0009 requires every later-event writer to take an exclusive lock on the immutable opening event, re-read and validate the case revision and operation eligibility while holding it, and hold it through exclusive creation of the typed next event file. That protocol is stated once as authority and implemented twice: `case::append` (`src/case.rs:402`) and `case::authorize_early_review` (`src/case.rs:516`).

The two implementations have already diverged. On an exact idempotent retry whose participants cannot be resolved — whether because no portfolio root is configured, or because roots are selected and a recorded participant no longer resolves to exactly one enrolled repository — `existing_append` propagates the failure (`src/case.rs:920`) and the command refuses; `existing_early_review` swallows it and reports `privacy: private` (`src/case.rs:683`) and the command succeeds. Neither branch is covered by any test: `run_without_portfolio_configuration` in `tests/case_cli.rs` is used only for `case show` and `case list`. One question about a privacy receipt therefore has two shipped answers, and the suite cannot tell.

That is the observed pressure. It is not an argument from predicted convenience: the remaining later event types named in [`design/v0.1-scope-and-acceptance.md`](../design/v0.1-scope-and-acceptance.md) §2 have no open issues, and this decision does not rest on them.

An architectural review of the case modules recommended collapsing the whole pipeline — proposal parsing, content validation, eligibility, privacy timing, and receipt fields alongside the protocol — into one parameterized module, removing roughly 970 of `src/case.rs`'s 1802 lines.

The project's own thresholds bear directly on that recommendation. `FOUNDATIONS.md` §5 counts independently accepted consumer needs; §6 treats a second occurrence as watching and a third as normally worth reviewing; §4 says to share only the invariant behavior whose authority can be owned coherently, and to keep implementations separate where triggers, invalidation rules, or reasons to change differ; §9 says the narrowest valid reuse scope wins. ADR 0002 records the threshold itself.

Applied to this crate's own internals, the protocol has two occurrences and a rule already written down as one rule. The surrounding concerns have two occurrences and visibly different shapes: `append`'s eligibility check derives privacy as part of validating the proposed occurrence, while `authorize_early_review` derives privacy separately and before the revision check; `append` derives its receipt's readiness from the recorded occurrence count, while `authorize_early_review` states it as a constant.

Amended 2026-08-10: the receipt half of that last observation did not support the conclusion drawn from it. The two do derive readiness differently, but their receipts already printed the same fields in the same order, and `Readiness::ReviewReadyByEarlyReviewOverride` renders byte-identically to the constants `authorize_early_review` states. The difference was in the derivation upstream of the receipt, not in the receipt. The eligibility and privacy-timing differences are real and still govern; the receipt-shape difference was not, and this decision's own implementation then gave `privacy:` and its footers one owner across both event types (`ReportedPrivacy`, `src/case.rs:226`) without the text following.

Without this decision recorded, the restraint leaves no trace. A module owning half the pipeline, with duplicated proposal readers, recorded-event validators, and outcome types standing beside it, reads as unfinished work rather than as a boundary.

## Decision

A shared internal publication module absorbs only the behavior ADR 0009 already states as one rule.

- The module owns sequence selection from an expected revision, the exclusive lock on the opening event, the re-read under that lock, the expected-revision check, exclusive creation of the typed event file, the occupied-path outcome, and the idempotent-retry comparison by recorded event identity and exact bytes.
- The retry comparison reads a permissive recorded envelope. It takes no per-event-type parameter.
- Proposal documents and their content validation, eligibility rules against the current case, and when and how case privacy is derived remain owned by each event type.
- Which fields a receipt prints stays the event type's decision. A receipt field or field group may take one owner where it has one authority and one reason to change, as `ReportedPrivacy` already does for `privacy:` and its footers. The three event-type receipts share one spine — heading, `case_id`, `file`, `revision`, the readiness fields where present, privacy, then the exact event bytes on a preview — and that spine may have one owner. Opening's privacy is derived once and has no retry path; sharing the spine must not widen it into states it cannot reach.
- Opening a case is not a publication and is not absorbed. It takes no lock, has no expected revision, and compares an existing record by semantic fields rather than by event identity and bytes.
- The publication ordering is enforced structurally: creating an event file is reachable only from a value that holds the lock and has re-read the case.
- This authorizes sharing the protocol and the receipt spine. It does not authorize unifying proposal parsing, eligibility, or privacy derivation across event types; it does not authorize collapsing the case queries' output and the event-type receipts into one value, because the queries print a different shape for a different question; and it does not authorize extracting anything from this crate into a shared package.

This records a boundary on internal sharing only. It makes no claim about a user's portfolio and creates no new consumer-facing surface.

## Consequences

### Positive

- ADR 0009's publication protocol has one implementation, so a later event type cannot omit the lock or the re-read.
- The retry-privacy divergence is resolved deliberately rather than by whichever copy a reader happens to open.
- The concerns most likely to differ between event types stay separable, so a third event type is free to need a different shape without a parameter being retrofitted to fit it.
- The absorbed module needs no clock, environment, or portfolio scan, so it is directly testable.

### Negative and risks

- Duplication remains in proposal readers and recorded-event validators. This is deliberate and will look like incompleteness.
- If the third later event type turns out to match the existing two on every remaining axis, the restraint will have cost one extra round of consolidation.
- A boundary defined by "what ADR 0009 already states" needs re-reading whenever ADR 0009 is amended.

### Operational burden

Adding a later event type means implementing its proposal, eligibility, privacy, and receipt, then publishing through the shared protocol. The protocol itself is not re-implemented or re-reviewed.

### Compatibility and migration

No recorded evidence changes. Event files, filenames, sequence semantics, and schema versions are untouched, so `CONSUMER-CONTRACT.md` §3 is not engaged. One receipt changes: an exact retry that cannot resolve participants reports `privacy: unknown` instead of refusing or asserting `private`, for every later event type. Its footer names the cause, because the two causes need different resolutions: no configured root selection keeps the existing portfolio-unavailable footer, while an unresolvable participant under selected roots gets a footer naming that condition. Neither footer carries repository identities or paths. That is a command-surface change permitted by `CONSUMER-CONTRACT.md` §8 during `0.x`, and it preserves the idempotency `FOUNDATIONS.md` §11 requires of side effects.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Collapse the whole pipeline behind one parameterized module | Rejected | The eligibility and privacy-timing shapes would be parameterized from two occurrences that already differ in those respects, against `FOUNDATIONS.md` §4 and §9. Amended 2026-08-10: the receipts were named in this reason and did not belong in it; they did not differ. |
| Collapse every case renderer, queries included, into one receipt value | Rejected | `case show` and `case list` answer a different question and print a different shape: no `file:` or `privacy:`, but `responsibility`, `occurrence_count`, `privacy_conflicted`, `stale`, and nested occurrence and evidence trees, one of them indented. Forcing those into the event-receipt spine is the same `FOUNDATIONS.md` §4 error mirrored. Shared readiness vocabulary may still take one owner. |
| Leave both implementations and fix only the divergence | Rejected | The protocol stays duplicated, so the next later event type can drift in the same way and nothing prevents it. |
| Defer all sharing until a third later event type exists | Rejected | The divergence is present and untested now, and the protocol's authority is already written as one rule. Waiting means a third copy is written before it is removed. |
| Record the restraint in the implementing issue instead of an ADR | Rejected | `docs/adr/README.md` ranks ADRs authoritative over issues, and an issue closes. A restraint whose only on-disk evidence is an absence would be reversed without anyone knowing it existed. |
| Generalize this into a rule that the project applies its own occurrence thresholds to all of its internals | Rejected | That would abstract a standing rule from one instance, which is the error this decision restrains. |

## Verification and review trigger

The decision is fit if a third later event type is implemented by supplying its own proposal, eligibility, privacy, and receipt while reusing the publication protocol unchanged, and if the retry-privacy question has exactly one answer afterwards.

Reopen when a third later event type exists and its proposal shape, eligibility rule, and privacy timing match the existing two closely enough that a parameterization would couple nothing that changes for different reasons. At that point the wider collapse has three occurrences behind it and should be reconsidered on its merits. The receipt question is settled by the 2026-08-10 amendment and is no longer part of this trigger.

Falsify this decision if implementing the protocol module forces any event type's eligibility or privacy derivation to move to a point in the sequence it does not belong, or if the shared retry comparison cannot be expressed without a per-event-type parameter. Either result would mean the protocol is less invariant than ADR 0009 states, and the duplication is the cheaper cost.

## Supersession

None.
