# ADR 0017: One owner for case terminal text

**Status:** Accepted  
**Date:** 2026-08-11  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

ADR 0010's 2026-08-10 amendment permitted two owners it did not place. It said the three event-type receipts "share one spine — heading, `case_id`, `file`, `revision`, the readiness fields where present, privacy, then the exact event bytes on a preview — and that spine may have one owner," and its rejected-alternatives table added that "shared readiness vocabulary may still take one owner." Both remain implemented where they happened to be written.

Where they were written is two files, and the rendering crosses between them in both directions.

`src/case.rs` holds `REVIEW_ONLY_NOTICE` (`:28`), `PORTFOLIO_UNAVAILABLE_FOOTER` (`:32`), `PARTICIPANTS_UNRESOLVED_FOOTER` (`:33`), `ReportedPrivacy::write_receipt_line` (`:593`), the `EventReceipt` spine and its `Display` (`:614`, `:625`), and the `Display` implementations for `OpenOutcome` (`:647`) and `LaterEventOutcome` (`:671`).

`src/case/read.rs` holds `CaseState::write_receipt_lines` (`:63`), `render_condition` (`:115`), the three query outcome types and their `Display` implementations (`:154`–`:442`), and seven brief field helpers (`:190`–`:353`). That is roughly 315 of the module's 1108 lines.

The spine in the writer calls down into the reader for the readiness lines (`src/case.rs:632`). The queries in the reader call up into the writer for the portfolio footer (`src/case/read.rs:411`, `:438`) and the review-only notice (`:73`). Of the fifteen sites at which `src/case/read.rs` reaches upward, those two items are receipt text and nothing else.

ADR 0011 examined a wider re-cut of these two modules and declined it, listing "receipt strings" among the things that stay where they are. It reasoned about the boundary as a whole and found the shared substrate was three concerns rather than one. It did not weigh rendering as a candidate on its own, and at that time terminal text had no test instrument of its own to be an owner for.

ADR 0016 supplied that instrument: in-process suites own "refusal and receipt prose via `Display`." `CONSUMER-CONTRACT.md` §1 already made the CLI behavior and terminal meanings an independently versioned surface. Terminal text is now a named, versioned, separately tested surface whose implementation has no home.

The coupling is also load-bearing rather than untidy. `write_authorized_implementation` (`src/case/read.rs:190`) encodes five of `validate_recorded_decision`'s invariants as `.expect()` (`:229`, `:238`, `:252`, `:262`, `:276`), and `BriefOutcome::fmt` a sixth (`:178`). A renderer in one module panics if a validator in another loosens — the reader asserting the writer's invariants across a boundary neither module owns.

ADR 0011's own reopen condition has not fired: no change is blocked by the coupling, and the recorded-event validators have acquired no independent reason to move. This is a separate decision against separate evidence, which is the relationship ADR 0011 predicted for the validators and ADR 0014 later took. `FOUNDATIONS.md` §4 asks that shared behavior have one authority and one reason to change. Printing has one authority — `CONSUMER-CONTRACT.md` §1 — and one reason to change.

## Decision

One internal module owns how a case prints.

- `case::render` owns every case renderer: the event-receipt spine, the `Display` implementations for the opening outcome, the later-event outcome and the three query outcomes, the shared readiness lines, the privacy line and its two footers, the review-only notice, and the brief's field helpers.
- It owns text and nothing else. It performs no input or output of its own, takes no lock, and derives no state. The `brief`, `show` and `list` entry points read and therefore stay in `case::read`, together with the outcome values they build; only the printing of those values moves.
- It is reached one way. `src/case.rs` and `src/case/read.rs` render nothing and hold no receipt string. `case::render` depends on both; neither depends on it.
- The three query shapes and the receipt spine stay distinct values. ADR 0010 refused collapsing the queries into the spine because they answer a different question and print a different shape, and ADR 0012 refused folding the brief into `case show`. Both refusals stand: this gives the renderers one home, not one shape.
- Which fields an event type prints stays that event type's decision under ADR 0010. `LaterEventHeadings` (`src/case.rs:405`) and the per-type heading and notice constants stay with their event type.
- The brief renderer receives a decision whose implementation-authorizing fields are not optional. `read_reuse_decision_event` (`src/case/read.rs:977`) produces it once `validate_recorded_decision` has proved those fields present, so the six `.expect()` sites become a shape the renderer cannot hold.
- `read::CaseRecord` and the three query outcome values become projections whose fields are readable within `case`. They are derived state under `CONTEXT.md`, their invariants are established at parse time, and rendering is their only other consumer.

This authorizes a placement and one type refinement. It does **not** authorize:

- any change to receipt text, refusal text, footers, field order, spelling, or refusal ordering;
- collapsing the queries into the receipt spine (ADR 0010) or folding the brief into `case show` or `case list` (ADR 0012);
- changing `DecisionContent`'s serialized shape or any recorded evidence byte;
- moving recorded-event validation, repository-root discovery, steward reading, storage-path validation, or readiness derivation — the remaining crossings between `src/case.rs` and `src/case/read.rs` stay, still governed by ADR 0011's reopen condition;
- widening any item to public API; the crate's existing re-exports are unchanged;
- a second rendering target such as a machine-readable or structured output format.

## Consequences

### Positive

- The surface `CONSUMER-CONTRACT.md` §1 versions and ADR 0016 assigns to in-process tests has one file to read and one file to test.
- The receipt spine and the shared readiness vocabulary take the single owner ADR 0010's amendment permitted but never placed.
- A renderer can no longer panic because a validator in another module loosened. Panic-on-drift becomes a compile error.
- Output changes stop touching the parser, and every renderer becomes reachable from a hand-built projection value with no repository on disk — the in-process instrument ADR 0016 assigned to receipt prose.
- `src/case/read.rs` stops reaching upward for receipt text entirely. What remains of its upward reach is event vocabulary, recorded-event validation, repository location, steward reading, and privacy derivation — none of it about printing.
- A new later event type supplies a heading and its receipt fields rather than a renderer.

### Negative and risks

- `read::CaseRecord` and the three query outcome values expose fields that are private today. Values that encapsulated them now rely on construction to hold their invariants.
- An outcome value and its `Display` live in different files, so reading how one prints costs a hop. That is the price of the text having one owner.
- The two modules still cross. This removes rendering from the crossing and leaves validation, root discovery and steward reading, so the boundary will still read as unfinished.
- `case::render` is the first module under `case` that depends on both siblings. If it later grows derivation rather than formatting, it becomes the place where reading and writing meet by accident.
- The move is large and almost entirely mechanical, and a mechanical move across a compatibility surface is where a silent text change hides. The gate in `CONTRIBUTING.md` is the mitigation: 201 tests pass at `9d2c8d8` and must pass unchanged.

### Operational burden

None ongoing. Adding a later event type still declares its name under ADR 0011, and its proposal, eligibility and privacy timing under ADR 0010. Only the place where its receipt fields are printed changes.

### Compatibility and migration

None. No recorded evidence changes, no receipt or refusal text changes, no exit status changes. `case::{brief, list, show}` and the three query outcome types keep their public paths through the existing re-exports, so `CONSUMER-CONTRACT.md` §1 and §3 are not engaged.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Move only `src/case/read.rs`'s renderers, as the reviewing document proposed | Rejected | Leaves the spine in `src/case.rs` still calling down for the readiness lines, and the notice and footers still in the writer. Rendering keeps two homes and one crossing, so the same decision returns the next time a receipt line changes. |
| Give `case::render` the `brief`, `show` and `list` entry points too | Rejected | It puts `fs::read_dir`, repository-root discovery and steward reading inside the module that formats, so every renderer stays reachable only through a repository on disk — the condition ADR 0016 wrote the in-process instrument to escape. Keeping the commands with reading is also what makes this decision's own falsification condition testable. |
| Move the renderers into `src/case.rs` beside the spine | Rejected | That file is 3086 lines and carries the most churn. It would own publication, validation and every renderer, and projection would still reach up into the writer to print. |
| Repair the six `.expect()`s and leave the modules alone | Rejected | Removes the panic but not its cause: the reader would still assert the writer's invariants, and the rendering vocabulary would still cross in both directions. Kept as the fallback if this decision is falsified. |
| Split `DecisionContent` into no-change and change variants | Rejected | Reaches `CONSUMER-CONTRACT.md` §3's hardest surface for a result a refined read-side view achieves with no serialized-shape change. Reconsider only if a second consumer of the distinction appears. |
| Give the queries and the receipts one value | Rejected | ADR 0010 refused this and the reason has not changed. One home is not one shape. |
| Record this in the implementing issue instead of an ADR | Rejected | ADR 0011 durably recorded that the wider re-cut was declined and that receipt strings stay. A later decision moving part of that list must leave the same kind of trace, and an issue closes. |

## Verification and review trigger

The decision is fit if the next receipt or query text change touches `src/case/render.rs` and nothing else, and the next later event type named in `design/v0.1-scope-and-acceptance.md` §2 adds no renderer.

Falsify it if `case::render` needs to read a file, take a lock, or derive state in order to print. That would mean rendering was not separable and the coupling was substantive rather than positional. Falsify it equally if the refined decision view forces `validate_recorded_decision` to change what it accepts.

Reopen it when a second rendering target is proposed, because one owner of the text is not automatically one owner of the format; or when the remaining crossings between `src/case.rs` and `src/case/read.rs` acquire their own reason to move, which stays governed by ADR 0011's reopen condition.

## Supersession

None. This decision amends neither ADR 0010, ADR 0011 nor ADR 0012. It places an owner ADR 0010's 2026-08-10 amendment already permitted, and it takes a decision about a substrate ADR 0011 declined to weigh separately.
