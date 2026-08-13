# Keep reuse implementation in the ordinary engineering workflow

## Consumer effect

Give implementers an exact accepted boundary while keeping implementation quality, migration, and review under each repository's normal engineering authority.

## Authority

Conforms to `docs/principles/FOUNDATIONS.md` §7, `docs/principles/EVIDENCE-AND-DECISIONS.md` §10, `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §7, ADR 0006, ADR 0012, and `docs/design/v0.1-scope-and-acceptance.md` §§2–4.

## Narrow scope

Project the bounded implementation brief from the accepted reuse decision, hand it to the ordinary engineering workflow, and return to the reuse lifecycle for independent verification.

## Out of scope

- Interface design, tickets, TDD, code changes, migrations, review, commits, or pull requests inside the reuse lifecycle.
- A reuse-owned refactor or extraction skill.
- Hand-authored routing state or a dependency on a particular engineering-workflow package.

## Acceptance

- The brief is a read-only projection of the accepted decision and adds no authored durable record.
- Implementation occurs outside the reuse lifecycle under repository engineering authority.
- Verification checks the accepted consequence independently after implementation.
- Reopen only if repeated handoff failures expose an information gap that a bounded brief improvement cannot solve, as ADR 0006 specifies.
