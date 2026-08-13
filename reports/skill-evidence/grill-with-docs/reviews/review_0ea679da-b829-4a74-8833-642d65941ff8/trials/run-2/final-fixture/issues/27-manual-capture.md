# Keep capture manually invoked

## Consumer effect

A maintainer can deliberately run capture after material implementation work and know when the bounded check occurred, without hidden work or evidence creation at session, commit, or CI boundaries.

The live bottleneck is proving that timely second-occurrence memory can remain cheap enough to repeat. Evidence bearing that claim is the active version 0.1 value stream and ADR 0001's accepted manual, case-based lifecycle.

## Authority

- `docs/principles/FOUNDATIONS.md` §§12 and 15 require decision-bearing records, write-free routine clean capture, and acceptable operational burden.
- `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §§2 and 4 make capture manual in version 0.1 and forbid evidence-creating hooks.
- `docs/adr/0001-manual-case-based-evidence-lifecycle.md` accepts explicit invocation and excludes Stop hooks, post-commit hooks, daemons, and background processes.
- `docs/design/v0.1-scope-and-acceptance.md` §§2, 3, and 6 keep capture manual and bound routine use to one explicit invocation.

## Narrow scope

- Provide the repository-installed `reuse-evidence-capture` skill with an explicit manual entry point after material implementation work.
- Bound each run to the completed work and likely prior occurrences.
- Make the terminal outcome clear enough that the maintainer knows capture completed.

## Out of scope

- Stop, post-commit, CI, scheduled, daemon, watcher, or background invocation.
- Automatic evidence creation, automatic review, or automatic refactoring.
- A reminder hook; that requires real missed-capture evidence and a separate accepted decision.
- A general architecture audit or a scan of unmarked repositories.

## Acceptance

- The capture package documents and exercises one explicit manual invocation.
- No installed asset or command registers or recommends an automatic hook.
- A bounded manual run can end in either an exact case-event preview or the fixed clean-capture result.
- Park or falsify this issue if real use shows that manual invocation is routinely missed; reopen authority through ADR 0001's stated reminder-only review trigger before adding automation.
