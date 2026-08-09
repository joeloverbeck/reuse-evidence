---
name: tdd
description: Test-driven development. Use when the user wants a red-green development loop or integration tests.
---

# Test-Driven Development

TDD is the red → green loop. This skill is the reference that makes that loop produce tests worth keeping: what a good test is, where tests go, the anti-patterns, and the rules of the loop. Every section applies on every cycle — consult them before and during the loop, not after.

When exploring the codebase, read `CONTEXT.md` (if it exists) so test names and interface vocabulary match the project's domain language, and respect ADRs in the area you're touching.

## What a good test is

Tests verify behavior through public interfaces, not implementation details. Code can change entirely; tests shouldn't. A good test reads like a specification — "user can checkout with valid cart" tells you exactly what capability exists — and survives refactors because it doesn't care about internal structure.

Before testing a public invariant or choosing a golden fixture, read [tests.md](tests.md) for the required coverage and sensitivity checks; see [mocking.md](mocking.md) for mocking guidelines.

## Seams — where tests go

A **seam** is the public boundary you test at: the interface where you observe behavior without reaching inside. Tests live at seams, never against internals.

**Test only at pre-agreed seams.** Before writing any test, write down each seam under test and its authority. A seam ratified in an authoritative PRD, live issue, ADR, or explicit user decision is already agreed; ask the user only when the seam is absent or ambiguous. No test is written at an unconfirmed seam. You can't test everything — agreeing the seams up front is how testing effort lands on the critical paths and complex logic instead of every edge case.

When those sources do not settle the seam, ask: "What's the public interface, and which seams should we test?"

## Anti-patterns

- **Implementation-coupled** — mocks internal collaborators, tests private methods, or verifies through a side channel (querying the database instead of using the interface). The tell: the test breaks when you refactor but behavior hasn't changed.
- **Tautological** — the assertion recomputes the expected value the way the code does (`expect(add(a, b)).toBe(a + b)`, a snapshot derived by hand the same way, a constant asserted equal to itself), so it passes by construction and can never disagree with the code. Expected values must come from an independent source of truth — a known-good literal, a worked example, the spec.
- **Horizontal slicing** — writing all tests first, then all implementation. Bulk tests verify _imagined_ behavior: you test the _shape_ of things rather than user-facing behavior, the tests go insensitive to real changes, and you commit to test structure before understanding the implementation. Work in **vertical slices** instead — one test → one implementation → repeat, each test a **tracer bullet** that responds to what the last cycle taught you.

## Rules of the loop

- **Red before green.** Write the failing test first, then only enough code to pass it. Don't anticipate future tests or add speculative features.
- **One slice at a time.** One seam, one test selector or explicitly named parameterized/subtest case, one minimal implementation per cycle.
- **Retain the cycle evidence.** For each behavior, keep one compact TDD evidence row: seam authority; test file and exact test selector or explicitly named parameterized/subtest case; observed public entry point; red command and intended plus observed failure; green command and result. This skill owns that evidence.
- **Use an authoritative verifier when it already supplies red.** If an authoritative PRD, live issue, ADR, or explicit user decision supplies an already-failing public verifier, confirm that its observed failure is the intended failure for the authorized criterion; that failure may then serve as red, without adding a duplicative test only to satisfy chronology. Keep one focused case per cycle whenever the verifier supports focus. When the verifier is indivisible and reports a finite named failure set that forms one atomic acceptance criterion, one aggregate cycle is allowed: record the verifier path and complete command, fixture or input identity, seam authority, the exact intended and observed red set, the minimal implementation, and the final green result. In this branch, the evidence row may name the verifier and input identity instead of a test file and selector, and pre-review reconciliation records `no changed tests; existing verifier evidence` and compares the existing-verifier evidence identities with their reconciliation identities in both directions.
- **Keep evidence commands replayable.** Repeat each complete runnable command in its row or define a stable command key in the same evidence section and reference that key. A prose label such as "same suite", "refusal test", or "verifier suite" is not a command or key.
- **Reconcile tests to seams before review.** Before `/code-review`, derive and retain a changed-test inventory from the scoped diff using the repository's native test structure. Keep one reconciliation row per changed test selector or explicitly named parameterized/subtest case: test file; exact selector or case identity; observed public entry point; seam authority; TDD evidence row. File- or class-level summaries do not satisfy this gate. Compare the inventory identities with the reconciliation identities in both directions and require both differences to be empty. If a test enters through a module, helper, mutable global, side channel, or other boundary the authority did not ratify for that behavior, ratify that seam or rewrite the test at the agreed public boundary. This gate concerns the public behavior each row claims: a private-invariant test cannot substitute for that public-behavior proof. Reconcile every evidence row before `/code-review` only after this coverage check passes.
- **Review repairs and refactoring.** When `/code-review` finds missing or incorrect observable behavior, `/implement` must re-enter one new red → green slice. For each actionable finding that `/code-review` marks `TDD re-entry required: yes`, append one evidence row keyed by review pass, axis, and finding, run the focused red, then apply only enough production repair to reach green; consume those routing fields as supplied rather than reclassifying the finding here. Never rewrite or extend an earlier row to absorb a later finding. If the same test selector is reused, the new row must name the distinct new parameterized/subtest case and its focused red command. Before final handoff, compare the applicable finding identities in `/code-review`'s `Findings fixed during review` ledger with the appended review-reentry row identities in both directions and require both differences to be empty. A repair that only refactors without changing behavior stays outside the red → green loop. Hand the reconciled, scoped committed change back to `/code-review`; that skill owns finding classification and the two-axis review, `/implement` owns the repair loop, and this skill owns TDD evidence.
