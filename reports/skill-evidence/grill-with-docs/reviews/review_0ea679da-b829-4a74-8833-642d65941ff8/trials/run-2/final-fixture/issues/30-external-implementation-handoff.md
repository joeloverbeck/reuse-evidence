# Keep reuse implementation in the ordinary engineering workflow

## Consumer effect

After accepting an exact reuse decision, a maintainer receives a bounded implementation brief that an ordinary engineering workflow can implement, while the reuse lifecycle remains able to verify the accepted result independently.

The live bottleneck is carrying decision authority across the implementation boundary without duplicating interface design, TDD, migration, and review capabilities. ADRs 0006 and 0012 establish the handoff and its durable projected form.

## Authority

- `docs/principles/FOUNDATIONS.md` §7 separates review and implementation responsibilities.
- `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §7 assigns implementation to ordinary engineering capabilities and returns to reuse verification afterward.
- `docs/principles/EVIDENCE-AND-DECISIONS.md` §10 defines the bounded implementation-brief contents.
- `docs/adr/0006-reuse-decisions-not-refactor-execution.md` excludes refactor execution from `reuse-evidence`.
- `docs/adr/0012-implementation-brief-is-a-projection.md` makes the brief a read-only projection of the accepted decision.

## Narrow scope

- Project the accepted implementation brief from authoritative case state.
- Preserve the accepted responsibility, invariant contract, non-responsibilities, affected consumers, compatibility and migration consequences, rollback path, and verification conditions.
- End the reuse-review workflow at the durable handoff, then resume only for independent verification of the implemented result.

## Out of scope

- Interface design, ticket decomposition, TDD, code edits, migrations, code review, commits, or pull requests inside the reuse lifecycle.
- Automatic refactoring or implementation triggered by acceptance, readiness, similarity, or occurrence count.
- Peer-skill routing, a duplicated engineering workflow, or a dependency on a specific engineering-workflow skill set.

## Acceptance

- A later session can reproduce the brief from the accepted decision without a separately authored document.
- The brief contains enough accepted information for an ordinary engineering workflow to proceed without inventing reuse scope.
- No capture or review asset edits consumer implementation code.
- Park or falsify this issue if repeated handoffs fail because the accepted decision lacks information that cannot be added narrowly to the brief; use ADR 0006's review trigger before moving implementation ownership.
