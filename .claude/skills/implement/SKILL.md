---
name: implement
description: "Implement a piece of work based on a PRD or set of issues."
disable-model-invocation: true
---

Implement the work described by the user in the PRD or issues.

## Before editing

1. Read the repository's agent instructions and the authoritative PRD or live issue bodies and comments. For tracker reads and mutations, follow `docs/agents/issue-tracker.md` §Conventions rather than copied summaries or duplicated mechanics.
2. Record the review fixed point the user supplies. If none was supplied, ask the user to confirm the current `HEAD` before editing.
3. Run `git status --short` without a path filter. Classify all pre-existing changes, preserve unrelated dirt, and keep the implementation scope explicit.
4. Turn the acceptance criteria into a run sheet that maps each requirement to its implementation seam and verification. When `/tdd` applies, point each applicable requirement to its TDD evidence row instead of copying the TDD-specific fields into this run sheet.

## Build and verify

Use `/tdd` where possible, at pre-agreed seams.

Discover the repository's native verification surface before selecting checks. Run typechecking, focused tests, and a full test suite only when they are configured and applicable. In prose or no-build repositories, use the checks named by the repository instructions, such as single-home, cross-reference, and release-metadata checks.

Run the smallest applicable checks regularly, and run the full applicable verification set once at the end. If no automated harness exists, record the exact structural or manual checks and their results.

## Review and commit

Put only the scoped implementation into `HEAD` (a provisional commit is acceptable) before invoking `/code-review`, because that skill reviews committed `HEAD` against the recorded fixed point.

Immediately before every provisional or final commit, or any amend, rerun unscoped `git status --short`, reconcile every transition against the initial classification, and inspect `git diff --cached --name-only` plus `git diff --cached --check`; proceed only when the index is exactly the scoped implementation and unrelated dirt remains unstaged. Immediately before `/code-review`, repeat the unscoped reconciliation, require an empty index, and inspect `git diff --name-only <fixed-point>...HEAD` plus `git diff --check <fixed-point>...HEAD` so the committed review range is exactly scoped.

Resolve every actionable Standards and Spec finding. If a finding would change a ratified interface, ownership boundary, scope, reversibility, pattern, or dependency, pause and re-ratify it with the user. When a finding changes observable behavior, re-enter `/tdd` and satisfy that skill's review-reentry evidence rule before applying the production repair; behavior-neutral refactors stay in this repair loop without inventing a red cycle. After each repair, commit or amend the scoped repair into `HEAD`, rerun applicable verification and `/code-review`, and continue until both axes are green on the exact current SHA. That green-reviewed SHA is the final implementation commit. Any later commit, amend, or other `HEAD` movement requires applicable verification and both review axes to run again against the replacement SHA before the workflow is treated as green.

## Optional tracker closeout

Only when the user expressly authorizes implementation-closeout tracker mutations, follow the repository's tracker contract. Verify every requested dependent or child issue is closed before closing its parent PRD, then exact-read every requested state.

For work originating from a live issue, record an explicit closeout disposition before final handoff. If no disposition has been ratified once the green-reviewed SHA exists, perform the read-only repository, published-ref, and publication-delta checks below, disclose the exact publication set and intended tracker mutations, and ask the user to authorize any required publication plus closeout or to leave the tracker open. Treat broad closeout permission received before the publication set is known as closeout intent, not authorization for an undisclosed set; disclose the complete set and require confirmation before publishing it. If closeout remains unauthorized, leave the issue open and make its exact tracker state and the missing authorization prominent in the final response.

Before the first closeout mutation, identify the target repository and intended published ref, refresh or query its remote state, and prove that the green-reviewed implementation commit is reachable from that published ref; local branch or upstream status alone is not publication proof. Before asking for publication authorization, enumerate the exact commit delta from the queried published-ref SHA to the green-reviewed SHA (for example, `git log --oneline <published-ref-sha>..<reviewed-head-sha>`). Disclose every commit publication would make reachable, explicitly identify any commit outside the current implementation scope, and require authorization for the complete publication set. If authorization does not cover that full set, stop for a different ref or branch, or revised direction. If reachability is absent or cannot be established, stop and ask the user either to publish or authorize publication, or to explicitly authorize local-only closeout. Close permission does not grant push or publication authority. Disclose any local-only exception in both the tracker comment and final response.

Keep closeout evidence on one of two paths:

- **External-only (default):** preserve tracker comments and exact-read results outside the committed implementation, and make no tracked run-sheet or ledger change after the green-reviewed, published implementation commit.
- **Committed evidence:** if closeout must update a tracked run sheet or ledger, record that as a separate closeout-only delta after the tracker mutation; do not amend the already-published implementation commit. Rerun applicable verification and `/code-review`, finalize the delta, apply the same publication-reachability gate to the resulting final workflow commit, and exact-read every requested tracker state before reporting the final SHA.

This closeout is separate from issue publication: `/to-issues` never closes a parent, and approval to publish slices does not authorize implementation closeout. Never infer close permission from implementation completion alone.
