# Keep capture manually invoked

## Consumer effect

A maintainer deliberately starts bounded capture after material implementation work and can trust that no hidden hook, daemon, or background process is scanning or recording evidence.

## Authority

Conforms to `docs/principles/FOUNDATIONS.md` principles 12 and 15, `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §§2 and 4, accepted ADR 0001, and `docs/design/v0.1-scope-and-acceptance.md` §§2, 3, and 6.

## Narrow scope

- Preserve manual invocation as the version 0.1 capture entry point.
- Keep each run bounded to the completed work and likely prior occurrences.
- Make the terminal outcome clear enough that the maintainer knows the requested capture completed.

## Out of scope

- Stop, post-commit, or CI hooks.
- Daemons, watchers, background scans, and automatic evidence creation.
- Broad repository or architecture audits.
- Any reminder mechanism without separately accepted evidence of missed capture.

## Acceptance

- Capture starts only through an explicit maintainer invocation.
- No hook or automation is installed or required.
- A routine run remains bounded and reaches a fixed terminal outcome.
- The implementation preserves the manual, case-based lifecycle and its write-authority gates.
