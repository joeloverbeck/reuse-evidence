# ADR 0016: The process boundary tests the terminal contract; behaviour is tested across the module interface

**Status:** Proposed
**Date:** 2026-08-11
**Decision owner:** Repository maintainer
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

ADR 0015 moved the portfolio location to the edge and closed with an explicit deferral: it does not authorize "rewriting the CLI test suite, or converting any existing test from the process boundary; the location merely makes in-process expression possible." This is that deferred decision, taken against measurement rather than prediction.

**The seam is already open.** All four write commands are `pub` and take `working_directory`, the portfolio location and `recorded_at` as values. Every outcome carrier implements `Display` — `OpenOutcome` (`src/case.rs:506`), `LaterEventOutcome` (`:530`), `BriefOutcome`, `ShowOutcome` and `ListOutcome` (`src/case/read.rs:172`, `:356`, `:418`) — and `TerminalFailure` (`src/lib.rs:160`) renders the exact `refusal: …\nresolution: …` prose the suite asserts off stderr, with `meaning()` returning the `ExitMeaning`. `RecordedInstant::from_unix_seconds` is `pub` (`src/case/instant.rs:61`). Nothing structural blocks in-process expression; only the absence of a decision did.

**What the boundary is paid for.** Every one of the 104 tests in `tests/case_cli.rs` was parsed individually. 98 of them (94%) assert both terminal text and filesystem state in the same test; 3 assert text only, 1 filesystem only, 2 neither. Median test length is 56 lines. Across the file exactly two exit statuses are ever asserted — `0` (178 times) and `3` (59 times).

So the process proves three wiring facts per test: argv reaches the right command, `ExitMeaning` reaches the right exit status, and the text reaches the right stream. `ExitMeaning::status()` is a `pub const fn` with no I/O (`src/lib.rs:41`), so the second fact is a pure function re-proved 237 times. Everything else those tests assert — refusal prose, receipt text, byte-exact write-free guarantees, revision conflicts, privacy derivation — is `Display` output and filesystem state.

**What the boundary cannot state.** Three things are not merely more expensive through a process:

- A terminal *meaning*. Status `1` covers every non-refusal failure, so `ExitMeaning::UnsafeFailure` cannot be distinguished from any other status-`1` outcome at the boundary.
- A revision race without timing. `tests/case_cli.rs` holds the production lock, spawns two children and sleeps 100ms at four sites. The marker lock is `File::lock`, held per open file description, so two threads in one process contend exactly as two processes do.
- A chosen instant. Every command takes `recorded_at`, but the binary always supplies `RecordedInstant::now()` (`src/main.rs:280`, `:312`, `:346`, `:380`) and ADR 0015 refused to add a flag. A test at the boundary can only recover whatever instant the run happened to choose.

**One thing this measurement found.** Asserting `ExitMeaning` directly showed that `UnsafeFailure` is not merely uncovered by the case suite but unreachable from it. `read_steward` (`src/case.rs`) collapses every non-supported `marker::MarkerRead` outcome into one refusal, so a marker that genuinely cannot be read from disk is a refusal to the case surface; `marker_for_enrollment` (`src/lib.rs:379`) separates that same input as an unsafe failure. Both are defensible in isolation — nothing has been written at either point, so the refusal's no-write guarantee is truthful — but one input has two meanings. This ADR does not resolve that; it records it and pins the current behaviour in `tests/case_library.rs`.

## Decision

The process boundary tests the terminal contract. The module interface tests behaviour.

- **The CLI suites own** argv dispatch, the exit status each `ExitMeaning` maps to, and which stream carries which text. `CONSUMER-CONTRACT.md` §1 makes "the CLI behavior and terminal meanings" an independently versioned surface, and this is the only instrument that reaches it.
- **In-process suites own** everything else: refusal and receipt prose via `Display`, write-free byte-equality, revision conflicts, privacy derivation, terminal meanings as `ExitMeaning` values rather than exit statuses.
- **New behaviour is asserted in process.** A new later event type adds in-process tests for its behaviour and reaches the CLI suite only for terminal facts it newly introduces.
- **Existing tests are not converted.** The 98 conflated tests pass, run in 0.36s, and guard the surface `CONSUMER-CONTRACT.md` §1 protects. Converting them is churn against a compatibility surface with no behaviour change to show for it.
- **Fixture scaffolding has one owner.** `tests/support/mod.rs` owns the invariant core — a nonce'd temporary root removed on drop, and the Git-repository and enrollment-marker scaffolds. Suite-specific helpers stay with their suite.
- **Recorded event file names stay hand-written in tests.** `tests/support/mod.rs` defines each name once, as a literal. They are deliberately *not* derived from `case::naming`: `CONSUMER-CONTRACT.md` §3 makes recorded evidence the hardest compatibility surface, and a hand-written name pins that layout independently of the code that produces it, where a derived one would agree with a naming defect instead of catching it. This is not a second owner of ADR 0011's vocabulary; it is an independent pin on ADR 0009's layout.

This authorizes a placement rule and the scaffolding that serves it. It does **not** authorize:

- converting, rewriting, or deleting any existing CLI test;
- widening `case::naming`, or any other `pub(crate)`/`pub(super)` item, to public API for test convenience;
- a command-line flag, configuration key, or environment variable for `recorded_at`;
- any change to receipt text, refusal text, exit statuses, or refusal ordering;
- resolving the `read_steward` / `marker_for_enrollment` divergence, which needs its own decision against its own evidence;
- moving the interrupted-write test in process — `RLIMIT_FSIZE` is per-process and that test must keep spawning `sh`.

## Consequences

### Positive

- A new later event type stops costing 900–1900 test lines that reuse almost nothing; four more are in `v0.1` scope.
- The revision race is stated without two child processes or a timed sleep.
- `recorded_at` is chosen by the test, so byte-for-byte retry comparison is a direct assertion rather than a recovered value.
- `ExitMeaning::UnsafeFailure` gains its first coverage from a command surface, and the surface divergence it exposed is now pinned rather than latent.
- Six duplicated `Fixture` implementations collapse to one, before an in-process suite could create a seventh.

### Negative

- Two test instruments exist, so a contributor must choose. The rule above is the whole of that choice.
- Behaviour asserted in process is no longer also asserted through the binary for *new* tests, so a dispatch or stream-routing regression on a new command is caught only by the terminal-contract test written for it.
- The 98 conflated tests remain conflated. This ADR bounds the problem going forward rather than repairing it.

### Reopen conditions

- Reopen the no-conversion clause when a change is actually blocked by a conflated test, rather than merely slowed by one.
- Reopen the naming clause if the recorded-evidence file-name layout gains a second supported shape, where a hand-written literal per test would no longer state which shape is intended.
