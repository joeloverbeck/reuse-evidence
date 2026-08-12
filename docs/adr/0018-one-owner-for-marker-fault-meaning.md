# ADR 0018: One owner for what a marker fault means

**Status:** Proposed  
**Date:** 2026-08-12  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

ADR 0016 §24 found that one input reaches two command surfaces with two meanings, and §43 explicitly withheld authority to resolve it: "resolving the `read_steward` / `marker_for_enrollment` divergence, which needs its own decision against its own evidence." It pinned the current behaviour in `tests/case_library.rs:118` instead, in a test whose own comment says it "pins that divergence rather than asserting either answer is the intended one." This is that decision.

`marker::read` (`src/marker.rs:151`) reports four outcomes — absent, supported, unsupported schema version, and unreadable — and assigns no meaning to any of them. Three callers assign their own:

- `marker_for_enrollment` (`src/lib.rs:409`) separates all four: an unsupported version and a malformed marker are refusals with distinct wording, and an I/O read failure is an unsafe failure.
- `read_steward` (`src/case.rs:1899`) separates absent from everything else and collapses the remaining three into one refusal.
- `inspect_marker` (`src/portfolio.rs:934`) maps them one-to-one onto report entries and refuses nothing.

Run against the same three faulty markers, the first two surfaces produce this:

| Marker fault | `enroll` | `case list` |
|---|---|---|
| path is a directory | `unsafe failure: could not read '…/reuse-evidence.toml': Is a directory (os error 21)` — status 1 | `refusal: the steward repository is not validly enrolled` — status 3 |
| not valid TOML | `refusal: existing marker '…' is malformed: TOML parse error at line 1, column 6 …` — status 3 | `refusal: the steward repository is not validly enrolled` — status 3 |
| `schema_version = 99` | `refusal: marker schema version '99' is not supported` — status 3 | `refusal: the steward repository is not validly enrolled` — status 3 |

Two separate defects are visible.

**One input, two meanings.** An unreadable marker is status 1 to enrollment and status 3 to the case surface. `CONSUMER-CONTRACT.md` §1 makes terminal meanings a versioned surface, so a consumer branching on status gets a different answer depending on which command met the same broken file.

**Three inputs, one sentence.** Every marker fault reaches the case surface as the same sentence, with no path, no cause, and no schema version. Its resolution says "before opening a case" even when the command was `case list`, which never opens one. `read_steward` has six callers (`src/case.rs:663`, `:668`, `:1170`; `src/case/read.rs:168`, `:191`, `:224`), five of which are not `case open`.

The obvious rationale for the divergence does not survive the evidence. It is not read-versus-write: `case open` writes and refuses, while `enroll` writes and does not. Both reach `marker::read` before anything has been written, which is what ADR 0016 §24 observed when it called both defensible in isolation.

That observation also decides the meaning. `ExitMeaning::UnsafeFailure` is documented as "the operation failed in a way that does not carry a no-write guarantee" (`src/lib.rs:33`). It is defined by the absence of that guarantee, not by whether the fault was the environment's or the caller's. At both sites the guarantee holds, so the unsafe failure states something untrue about the run.

## Decision

The classification of a non-supported marker read has one owner, and it refuses.

1. One function in `src/lib.rs` turns a non-supported `marker::MarkerRead` into a `TerminalFailure`. `marker` continues to report and classify nothing.
2. Every marker fault is a refusal. Nothing has been written when a marker is read, so the no-write guarantee is truthful, and `UnsafeFailure` is reserved for a failure that cannot make that promise.
3. The refusal names the fault: which of the three it is, the marker path, and the underlying cause. The case surface stops collapsing them.
4. The sentence the caller contributes is its resolution, supplied as an argument the way `case::event::EnvelopeRefusal` already supplies per-event-type wording. `case list` stops telling a reader to fix the marker "before opening a case".
5. `portfolio` keeps reporting rather than refusing. It observes many repositories at once and refusing on one would discard the rest — a different question from what a single-repository command means, not a third divergence.

This authorizes the classification owner, the meaning change, and the wording that names the fault. It does **not** authorize:

- changing `marker::read`'s four outcomes, the marker schema, or the marker file name;
- changing what `portfolio` does with a faulty marker;
- re-cutting the `src/case.rs` / `src/case/read.rs` boundary, still governed by ADR 0011's reopen condition;
- widening `marker` or any other item to public API beyond what is public today;
- changing any refusal text for inputs other than a faulty marker.

## Consequences

### Positive

- One input has one meaning. A consumer branching on exit status gets the same answer from every single-repository command.
- A reader of `case list` learns which marker is broken and why, instead of a sentence that fits three unrelated faults.
- The resolution matches the command that ran.
- `UnsafeFailure` regains a definition that matches its documentation.

### Negative and risks

- `enroll` and `set-visibility` change from status 1 to status 3 for an unreadable marker. That is a `CONSUMER-CONTRACT.md` §1 terminal-meaning change, permitted during `0.x` under §8, but it is a real change for anything scripting those commands.
- `ExitMeaning::UnsafeFailure` loses its only coverage from a command surface — the coverage ADR 0016 §53 recorded as a gain. It remains reachable only from post-write failures, such as an atomic publish that fails after a temporary file exists, which are harder to provoke in a test. This decision narrows the meaning correctly and makes it more expensive to cover.
- Refusal text on the case surface changes, so any consumer matching that sentence must update. §1 and §8 again.
- `tests/case_library.rs:118` must be replaced. That is intended: ADR 0016 wrote it to pin a divergence, not to defend it.

### Operational burden

None ongoing. A fifth `MarkerRead` outcome would be classified once in the owning function rather than at each surface.

### Compatibility and migration

No recorded evidence changes, no marker schema change, and no migration. The change is confined to terminal meaning and refusal text on the two single-repository surfaces, both `CONSUMER-CONTRACT.md` §1 surfaces changeable under §8.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Fix the diagnostics only, leaving `enroll` an unsafe failure | Rejected, and the fallback if this is declined | It removes the collapse, which is the defect a user actually hits, and costs no exit-status change. But it leaves one input with two meanings, which is the divergence ADR 0016 named, so the deferred decision would still be open. |
| Make every marker fault an unsafe failure | Rejected | It would have a read-only `case list` announce that it may have written something. The `UnsafeFailure` doc comment is explicit that the meaning is about the no-write guarantee. |
| Keep the divergence and record why | Rejected | The available rationale is that enrollment is about to write, and the evidence refutes it: `case open` also writes and refuses on the same input. |
| Give `MarkerRead` the classification as a method on itself | Rejected | `marker` is a `pub` module with no dependency on the terminal contract today, and each caller still contributes its own resolution sentence, so the owner belongs beside `TerminalFailure`. |
| Record this in the implementing issue instead of an ADR | Rejected | ADR 0016 deferred this by name and pinned the behaviour in a test. A decision that unpins it must leave a trace of the same durability, and an issue closes. |

## Verification and review trigger

The decision is fit if every marker fault produces one sentence naming which fault it is at every single-repository surface, the same input yields the same exit status from `enroll` and from `case list`, and `tests/case_library.rs`'s divergence pin is replaced by an assertion that the two surfaces agree.

Falsify it if a surface is found that reaches `marker::read` after having already written something, which would mean a refusal there is untruthful and the divergence was tracking a real distinction.

Reopen it if `marker::read` gains a fifth outcome that no single classification fits, or if a single-repository command acquires a reason to continue past a marker fault the way the portfolio scan does.

## Supersession

None. ADR 0016 §43 declined to resolve this and referred it to "its own decision against its own evidence"; this is that decision, and it amends nothing in ADR 0016 beyond retiring the test pin that ADR 0016 §24 created to hold the question open.
