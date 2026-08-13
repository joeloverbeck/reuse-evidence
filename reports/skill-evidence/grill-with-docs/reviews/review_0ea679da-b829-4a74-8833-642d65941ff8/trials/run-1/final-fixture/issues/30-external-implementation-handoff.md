# Keep reuse implementation in the ordinary engineering workflow

## Consumer effect

A maintainer receives an exact, durable implementation handoff from an accepted reuse decision while ordinary engineering remains responsible for interface design, tests, code changes, migration, and review.

## Authority

Conforms to `docs/principles/FOUNDATIONS.md` principles 7 and 11, `docs/principles/EVIDENCE-AND-DECISIONS.md` §10, `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §7, accepted ADR 0006, accepted ADR 0012, and `docs/design/v0.1-scope-and-acceptance.md` §§2 and 4.

## Narrow scope

- Project the bounded implementation brief from the exact accepted reuse decision.
- Hand the brief to the repository's ordinary engineering workflow.
- Return to the reuse lifecycle only for independent verification against the accepted conditions.

## Out of scope

- Refactor execution within capture, review, status, or another reuse-lifecycle capability.
- A reuse-specific implementation, TDD, migration, or code-review workflow.
- Automatic implementation after acceptance.
- Peer-skill routing state or a dependency on a particular engineering-workflow package.

## Acceptance

- The handoff is reproducible from recorded case state and contains no separately authored brief facts.
- No reuse-lifecycle command or skill modifies consumer code as part of accepting the decision.
- Ordinary engineering owns implementation and review.
- Reuse verification independently checks the result and closes, parks, or reopens the case without implementing repairs.
