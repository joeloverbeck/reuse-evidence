# Keep capture manually invoked

## Consumer effect

Give the maintainer one explicit, bounded capture action after material work, so they know capture ran without hidden scans or automatic evidence creation.

## Authority

Conforms to `docs/principles/FOUNDATIONS.md` §§12 and 15, `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §§2 and 4, ADR 0001, and `docs/design/v0.1-scope-and-acceptance.md` §§2–3.

## Narrow scope

Define capture as a manual invocation bounded to completed work and likely prior occurrences. Preserve one fixed terminal result for a clean capture and explicit human authorization for any case event.

## Out of scope

- Stop, post-commit, CI, daemon, watcher, or background capture.
- Reminder automation without real missed-capture evidence and a separate accepted decision.
- Broad architecture audits or first-use inventories.

## Acceptance

- Capture starts only from an explicit invocation.
- The run stays bounded to the completed work and relevant prior occurrences.
- No hook or automatic evidence-creation path is added.
- Park or reframe this issue if real use shows manual capture is not being missed or if the bounded workflow cannot remain acceptable in routine use.
