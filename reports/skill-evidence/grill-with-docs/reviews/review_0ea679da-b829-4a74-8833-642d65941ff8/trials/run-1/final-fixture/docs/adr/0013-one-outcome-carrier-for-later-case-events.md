# ADR 0013: One outcome carrier for later case events

**Status:** Accepted  
**Date:** 2026-08-11  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

ADR 0010 declined to collapse the later-event pipeline while two later event types existed, and named its own reopen condition: "when a third later event type exists and its proposal shape, eligibility rule, and privacy timing match the existing two closely enough that a parameterization would couple nothing that changes for different reasons."

That third type shipped. `reuse_decision_accepted` landed in `fc8188f` (#18) at `+983` lines of `src/case.rs` and `+1863` lines of `tests/case_cli.rs`, in the file that already carried the most churn in the repository.

Two passes have since measured what the reopen condition actually exposed, rather than predicting it.

The first pass moved the expected-revision check and the existing-event retry comparison into `src/case/publication.rs`. This was conformance, not expansion: ADR 0010 already assigns that module "the expected-revision check" and "the idempotent-retry comparison by recorded event identity and exact bytes," and the three preview paths were implementing both outside it. The check now has one implementation instead of four, `existing_event` is private, and no terminal byte changed.

The second pass gave steward-and-case location and typed event-path construction one owner each, and collapsed four `read_case_for_*` readers — which differed only in one resolution sentence — into one. `decide` had already grown private helpers `load_decision_case` and `decision_event_path` for exactly these steps, so the third occurrence had begun the extraction by hand. Net effect: `-30` production lines, no terminal byte changed.

**After both passes the three commands are still 103, 110, and 108 lines.** That is the observation this decision rests on. The remaining duplication is not the protocol and not the location. It is the two branch shapes — preview and publish — each of which maps two protocol outcomes onto that event type's own outcome value.

Those outcome values are shadows of a spine that already has one owner:

| | effect | case_id | event_path | revision | privacy | event | extra |
|---|---|---|---|---|---|---|---|
| `AppendOutcome` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | `state` |
| `EarlyReviewOutcome` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| `DecisionOutcome` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | `action` |

`EventReceipt` (`src/case.rs:479`) already declares `state: Option<read::CaseState>` and `notice: Option<&str>`, which is exactly where those two extras land. Three structurally identical effect enums stand beside the structs at `src/case.rs:430`, `:448`, and `:467`. The carrier and the spine are two spellings of one shape, and the branch shapes cannot be written once while they produce three types.

ADR 0010 permitted the spine one owner: "The three event-type receipts share one spine — heading, `case_id`, `file`, `revision`, the readiness fields where present, privacy, then the exact event bytes on a preview — and that spine may have one owner." Its refusals are named and specific: proposal parsing, eligibility, privacy derivation, and collapsing the case queries' output into the event receipt. None is engaged by unifying the value that feeds the spine.

`OpenOutcome` (`src/case.rs:374`) is not a shadow of the spine. Opening records no revision, carries `Visibility` rather than `ReportedPrivacy`, has no retry path, and is not a publication. ADR 0010 states that sharing the spine "must not widen it into states it cannot reach."

Without this decision recorded, the exclusion of `OpenOutcome` from a unified later-event carrier exists only as an absence, and reads as an oversight rather than as a boundary. ADR 0010, ADR 0011, and ADR 0012 each rejected recording their restraint in an implementing issue for that reason.

## Decision

The three later case event types share one internal outcome carrier.

- One outcome value and one effect enum — preview, created, existing — replace `AppendOutcome`, `EarlyReviewOutcome`, `DecisionOutcome`, and their three effect enums.
- The carrier's fields are the spine's: effect, case identity, event path, revision, readiness where the event type reports it, reported privacy, decision notice where the event type reports it, and the exact event bytes.
- Which heading the carrier renders, and which optional fields it populates, remain each event type's decision, as ADR 0010 requires.
- Because the branch shapes then produce one type, the preview and publish outcome mapping may be written once.
- Opening is not absorbed. `OpenOutcome` keeps its own shape, its `Visibility`, and its absence of a revision.

This authorizes sharing the later-event outcome carrier and the branch shapes that produce it. It does not authorize unifying proposal documents, content validation, eligibility rules, or privacy derivation across event types; it does not authorize folding the case queries' output into the carrier; and it does not authorize extracting anything from this crate.

If the maintainer reads ADR 0010's refusals as already covering the outcome carrier, this is a narrow amendment to ADR 0010 rather than a reading of it. Either way, every refusal ADR 0010 names is preserved here unchanged.

## Consequences

### Positive

- Four effect enums become one and three outcome structs become one, so a later event type supplies a heading and its optional fields rather than a struct, an enum, and two match arms.
- The outcome carrier and the receipt spine stop being two spellings of one shape that must be kept aligned by hand.
- The preview and publish branch shapes become one, completing what the two prior passes started without touching anything ADR 0010 withheld.

### Negative and risks

- The carrier gains two optional fields that only one event type each populates. A fourth later event type that populates neither carries two absent fields.
- `OpenOutcome` standing beside a unified later-event carrier will read as unfinished work. It is the boundary this decision exists to record.
- If a later event type needs an outcome field that is neither readiness nor a notice, the carrier grows a third optional rather than that type keeping its own shape.

### Operational burden

Adding a later event type means supplying its heading and optional fields, plus its proposal, eligibility, and privacy as ADR 0010 requires, then publishing through the shared protocol. No outcome struct, effect enum, or branch shape is written.

### Compatibility and migration

No recorded evidence changes. No receipt text changes: the same headings, fields, and order are printed, so `CONSUMER-CONTRACT.md` §3 is not engaged and §1 command behaviour is unchanged. The decision is fit only if that remains true, which the existing command tests already assert byte-exactly.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Share the branch shapes without unifying the carrier | Rejected | The branch shapes cannot be written once while they produce three distinct types. The carrier is what makes them shareable. |
| Absorb `OpenOutcome` as well | Rejected | Opening records no revision, has no retry path, and is not a publication. ADR 0010 forbids widening the spine into states it cannot reach. |
| Use `EventReceipt` itself as the outcome | Rejected | The receipt borrows its heading and event bytes; an outcome must own them and outlive the render. |
| Collapse proposal readers, eligibility, and privacy at the same time | Rejected | ADR 0010 withholds all three, and the two prior passes produced no evidence that they have converged. This decision does not disturb that restraint. |
| Defer until a fourth later event type exists | Rejected | ADR 0010's reopen condition has already fired, and two measured passes established what remains. Waiting means a fourth carrier is written before three are removed. |
| Record this in the implementing issue instead of an ADR | Rejected | The restraint needing a durable trace is the exclusion of `OpenOutcome`. An issue closes; the absence would read as an oversight. ADRs 0010, 0011, and 0012 rejected the same alternative for the same reason. |

## Verification and review trigger

The decision is fit if the next later event type named in [`design/v0.1-scope-and-acceptance.md`](../design/v0.1-scope-and-acceptance.md) §2 is added by supplying a heading and its optional fields, with no new outcome struct, effect enum, or branch shape written, and every receipt still prints byte-identically.

Falsify it if any later event type needs an outcome field that is neither readiness nor a decision notice, or if sharing the carrier forces opening's shape to change. Either result means the carrier is less invariant than the spine it mirrors, and three structs are the cheaper cost.

Reopen it if a later event type's receipt must print a field the spine does not carry, or if the case queries ever need the same carrier — which ADR 0010 refused for receipts and this decision does not revisit.

## Supersession

None.
