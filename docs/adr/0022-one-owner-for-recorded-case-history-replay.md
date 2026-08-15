# ADR 0022: One owner for recorded case history replay

**Status:** Accepted  
**Date:** 2026-08-15  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

`src/case/read.rs` does two jobs. Lines 1–537 answer the four query commands — `find`, `list`, `show`, `brief` — and hold the outcome values ADR 0017 left with them. Lines 538–1060 fold a case's recorded event stream into a `CaseRecord` and refuse every stream that violates ADR 0009's layout: sequence gaps, duplicated sequence numbers, a file name disagreeing with its recorded type, an event after a closed verification, a decision whose prefix is not review-ready. The two bands share exactly one value, `CaseRecord`, which the second builds and the first reads.

**The band with no instrument is the one that decides whether a case is readable at all.** Before this ADR's companion change, `src/case/read.rs` was the largest production module under `src/case/` — 1 060 lines — and the only one besides the 122-line `case/event.rs` with no `#[cfg(test)]` module. `case/naming.rs`, `case/instant.rs`, `case/publication.rs`, `case/render.rs` and `portfolio.rs` all have one.

**The measurement.** Every refusal in the replay band was grepped against `tests/`. Ten distinct refusal classes across thirteen sites had no assertion at any instrument:

| Refusal | Site |
|---|---|
| case contains no event files | `:578` |
| more than one early-review override | `:604` |
| more than one accepted reuse decision | `:613` |
| an event after a closed verification | `:652` |
| verification before an accepted reuse decision | `:668` |
| verification after a terminal disposition | `:682` |
| a non-opening event at opening sequence 1 | `:867`, `:891`, `:915`, `:940` |
| an event that does not match its steward-local case | `:825` |
| an event without a UTF-8 filename | `:1009` |
| a case event path that is not steward-local — **`unsafe_failure`** | `:774` |

Five of these belong to the decision and verification event types, added after ADR 0016 set "New behaviour is asserted in process." That is live drift against ADR 0016's own rule.

The last row is not merely uncovered. ADR 0016 measured that status `1` covers every non-refusal failure, so `ExitMeaning::UnsafeFailure` "cannot be distinguished from any other status-`1` outcome at the boundary." That refusal is unreachable as a distinct meaning from any CLI test, by that ADR's own finding.

**What an in-process instrument reaches without this decision.** Seven validators in the band are pure functions over plain values: `validate_event_sequences`, `validate_file_event_type`, `validate_body_sequence`, `validate_not_extended_after_closure`, `validate_verification_prefix`, `validate_decision_prefix`, `validate_unique_occurrences`. A `#[cfg(test)]` module inside `src/case/read.rs` asserts all seven with no repository on disk, and the companion change does exactly that — twenty-one tests closing three of the ten classes above. That change needed no new authority and is not what this ADR asks for.

The other seven classes sit inside `read_case` and `read_case_event`, which open a directory and read files. They stay reachable only by writing a malformed event stream into a real directory, and one of them stays unreachable entirely.

**The band also carries the file's upward coupling.** Of the eight qualified `super::` call sites in `src/case/read.rs`, seven are in the replay band — the five recorded-event validators plus `validate_verification_against_decision` and `validate_recorded_decision_participants` — as is one of the three `validate_case_storage_path` calls. Only `reported_privacy` (`:382`) belongs to the query band. Splitting does not move those validators and does not re-cut the boundary; it makes visible which side of `read.rs` actually depends on `case.rs`.

**What the two prior decisions left open.** ADR 0011 declined a wider re-cut of `src/case.rs` and `src/case/read.rs`, and named its own reopen condition: "when a change is blocked by the coupling between `src/case.rs` and `src/case/read.rs` itself rather than by naming, or when the recorded-event validators acquire an independent reason to move." Neither has fired, and this decision does not fire them — the validators stay in `src/case.rs` and keep being called upward. ADR 0017 restated the same boundary, leaving "recorded-event validation, repository-root discovery, steward reading, storage-path validation, or readiness derivation" where they are. This decision moves none of those. It divides `src/case/read.rs` along its own internal seam, which neither ADR examined.

**The seam is already open, in the shape this crate already uses.** `UserDirectory::select` (`src/portfolio.rs:1055`) takes `lookup: impl Fn(&str) -> Option<PathBuf>` so its precedence rule is stated once, performs no filesystem access, and is tested with an in-memory substitute (`:1089`). `publication::LockedPublication::acquire` (`src/case/publication.rs:102`) takes `read_case: impl FnMut() -> Result<C, TerminalFailure>`. `src/case.rs:1494`–`:1503` and `:1969` take four more closure parameters. An injected reader is this crate's established way to state a rule without owning the input.

## Decision

One internal module owns folding a case's recorded event stream into its derived record.

- `case::replay` owns the fold and every refusal that rejects a recorded stream: the whole-set sequence check, per-event decoding, the envelope and file-identity checks, the prefix rules for a decision and a verification, and the post-closure rule.
- It owns the value that fold establishes. `CaseRecord`, `CaseState`, `Conditions` and the internal `CaseEvent` move with it, because ADR 0017 already made `CaseRecord` a projection "whose invariants are established at parse time" and this is that parse. `case::read` and `case::render` reach `case::replay` one way; it reaches neither.
- **It performs no filesystem access.** Its entry point is

  ```rust
  pub(super) fn replay(
      repository_root: &Path,
      case_id: Uuid,
      steward_repository_id: Uuid,
      event_paths: &[PathBuf],
      read_event: impl Fn(&Path) -> Result<String, TerminalFailure>,
  ) -> Result<CaseRecord, TerminalFailure>
  ```

  `case::read` keeps directory scanning, staged-temporary filtering and path sorting, and supplies the reader. Path validation stays inside `case::replay`, because `strip_prefix` and `validate_case_storage_path` are pure and their `unsafe_failure` is the one refusal no process boundary can state.
- **The reader is injected rather than the bytes passed.** Today the whole-set sequence check runs before any file is opened, and each event is then validated in ascending order. Eagerly loading the stream would hoist every I/O refusal ahead of the gap and duplicate refusals, so a stream carrying both faults would refuse differently. ADR 0016 does not authorize any change to refusal ordering, and injection preserves the current order exactly.
- **It is asserted in process, inside the module.** ADR 0016 forbids widening any `pub(crate)`/`pub(super)` item to public API for test convenience, so `case::replay` is not reachable from `tests/`. Its instrument is a `#[cfg(test)]` module, the shape five modules in this crate already use.
- The relocation is a path rename at 39 call sites — 38 in `src/case.rs`, one in `src/case/render.rs` — all caught by the compiler.

This authorizes a placement, an interface, and one relocation. It does **not** authorize:

- moving recorded-event validation, repository-root discovery, steward reading, storage-path validation, readiness derivation, or privacy derivation out of `src/case.rs`; ADR 0011's reopen condition still governs them;
- converting, rewriting, or deleting any existing CLI test;
- widening any item to public API, or introducing a `History` type; `CaseRecord` is the projection ADR 0017 named, and `CONTEXT.md` already spends *history* on the recorded event stream;
- any change to receipt text, refusal text, refusal ordering, exit statuses, or recorded evidence bytes;
- a general port-and-adapter layer; one injected reader is the whole of the seam;
- reading a case's events in any order other than the ascending sequence order `case::read` supplies.

## Consequences

### Positive

- Seven refusal classes that need a real directory today, including the `unsafe_failure` that no process boundary can distinguish, become assertable from a hand-built path set and an in-memory reader.
- The seven `expect()`s in the band are pinned by the validator that justifies each one, rather than by a distributed argument across five functions.
- `src/case/read.rs` becomes one job: scan the directory, run the four queries, print through `case::render`.
- `case::replay` is reached one way by both the query path and the writer path, the property ADR 0011 required of the naming owner and ADR 0017 of the renderer.
- Which side of the former `read.rs` depends on `src/case.rs` becomes visible: seven of eight upward calls belong to replay.

### Negative and risks

- A third module under `case` now depends on `src/case.rs`'s validators, so the parent's fan-in grows even though its fan-out does not.
- `replay` takes five parameters, one of them a closure. That is the price of stating the fold without owning the input.
- The relocation of `CaseRecord`, `CaseState` and `Conditions` touches 39 sites across the writer path. Every one is compiler-checked, but ADR 0017 recorded that "a mechanical move across a compatibility surface is where a silent text change hides." The mitigation is that the companion in-process instrument lands first, so the validators are pinned before they move.
- The `unsafe_failure` divergence ADR 0016 recorded and ADR 0018 partially resolved is made testable here, not resolved. Nothing about its meaning changes.
- The "event without a UTF-8 filename" refusal still needs `#[cfg(unix)]` to construct an input for, wherever its test lands. `case::replay` would be the first module under `case` with a platform-conditional test.

### Operational burden

None ongoing. A new later event type still declares its name under ADR 0011, its proposal, eligibility and privacy under ADR 0010, and its receipt fields under ADR 0017. It gains one more place to state how its recorded form is read back, which is where that code already lives.

### Compatibility and migration

None. No recorded evidence changes, no receipt or refusal text changes, no refusal ordering changes, no exit status changes. `case::{brief, find, list, show}` and the query outcome types keep their public paths through the existing re-exports, so `CONSUMER-CONTRACT.md` §1 and §3 are not engaged.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Add an in-process instrument to `src/case/read.rs` and split nothing | Adopted in part, rejected as sufficient | It is free, needs no authority, and landed as this ADR's companion change — but it reaches only the seven pure validators. Seven refusal classes stay behind a real directory and one stays unreachable at any process boundary. |
| Pass eagerly-loaded event bytes: `&[(PathBuf, String)]` | Rejected | Hoists every I/O refusal ahead of the whole-set gap and duplicate refusals, changing which refusal a doubly-malformed stream receives. ADR 0016 does not authorize a refusal-ordering change. |
| Pass a deferred vector: `Vec<(PathBuf, Result<String, TerminalFailure>)>` | Rejected | Preserves ordering, but reads every event file even when the first one refuses, and `TerminalFailure` derives no `Clone`. Injection preserves both the order and the current I/O. |
| Pass bare bytes with no path | Rejected | Eleven refusals in the band interpolate `event_path.display()`. A unit without its path cannot state them. |
| Return a new `History` type | Rejected | ADR 0017 already named `CaseRecord` the projection, and `CONTEXT.md` already uses *history* for the recorded event stream. A second name for derived state is the divergence ADR 0011 was written against. |
| Re-cut `src/case.rs` and `src/case/read.rs` and move the recorded-event validators down | Rejected | ADR 0011 examined and declined it, and its reopen condition has not fired: no change is blocked by that coupling, and the validators have acquired no independent reason to move. |
| Record this in the implementing issue instead of an ADR | Rejected | The decision needing a durable trace is that the fold's input is injected rather than loaded, and why: refusal ordering is a compatibility surface. An issue closes; the next reader proposing `&[Bytes]` would find nothing. |

## Verification and review trigger

The decision is fit if all seven remaining refusal classes above gain an in-process assertion built from a path set and an in-memory reader, with no temporary directory, and `cargo test` reports no change in any CLI suite.

Falsify it if preserving the current refusal order requires the reader to know anything beyond a path, or if `case::replay` needs a value from `case::read` — either would mean the seam is not where this ADR draws it.

Reopen it if a second caller needs the fold in a different order, or if the injected reader grows a second responsibility. Either would mean one closure is no longer the whole of the seam.

## Supersession

None.
