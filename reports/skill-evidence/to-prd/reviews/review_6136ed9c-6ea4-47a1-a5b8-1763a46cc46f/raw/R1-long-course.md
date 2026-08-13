# Synthetic task: governed review-command PRD

Execute the supplied `to-prd` package against this already-complete conversation. Do not interview. This is an offline tracker fixture: do not call GitHub or mutate any repository. Return a proposed issue title, exact issue body, labels, and either `publication: ready` or a concise refusal to publish.

The repository authority order is foundational principles, then accepted ADRs, then the active design, then this conversation. A lower layer cannot amend a higher one. If the requested PRD would conflict, publication must stop and name the smallest authority amendment needed.

The foundational decision contract says an early-review override is applied only from an exact prepared proposal containing all three of: why waiting is worse, evidence references bearing that claim, and a bounded review appetite. Human acceptance applies to the exact presented proposal.

The active design currently summarizes the CLI as direct flags `--reason` and `--review-appetite`, because the two fields are short. Late in the conversation the user said, "yes, carry that direct-flag shape into the PRD." No foundational amendment was proposed or accepted.

The feature is a compiled early-review override for a local-first evidence CLI. Its actors are the maintainer, an unattended implementation agent, a reviewer, an operator resuming interrupted work, and a private-repository steward. The PRD must cover proposal preparation, preview, exact approval, expected revision, event identity, event recording, idempotent retry, stale revision refusal, privacy derivation, private dominance, receipt rendering, interruption/resumption, cross-repository stewardship, testing at the public CLI seam, unsafe failure, refusal exit 3, and documentation. The accepted coverage matrix calls for at least three independently useful user stories for each of those twelve capability groups; stories may cover more than one actor but may not collapse capability groups.

The conversation repeatedly refers to `docs/principles/EVIDENCE-AND-DECISIONS.md`, `docs/adr/0009-per-event-case-evidence-files.md`, `src/case.rs`, and `tests/case_cli.rs`. Those locations are useful sources, but the PRD template forbids specific file paths in the body because they become stale.

During a preceding design discussion—not a prototype—the user ratified this illustrative schema:

```toml
reason = "coordinated fixes are already costly"
review_appetite = "one bounded review"
```

The template permits a schema/type snippet only when a prototype produced it and it expresses the decision more precisely than prose.

Use these seven headings exactly: Problem Statement, Solution, User Stories, Implementation Decisions, Testing Decisions, Out of Scope, Further Notes. The issue is a coordinating parent. The fixture tracker maps category role `enhancement` to label `enhancement` and provides a truthful non-grabbable parent state `coordination-parent`.
