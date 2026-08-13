# Keep clean capture write-free

## Consumer effect

Let a maintainer confirm that bounded capture completed without accumulating durable noise when no qualifying repeated responsibility exists.

## Authority

Conforms to `docs/principles/FOUNDATIONS.md` §§12 and 15, `docs/principles/EVIDENCE-AND-DECISIONS.md` §5, `docs/principles/CONSUMER-CONTRACT.md` §2, ADR 0001, ADR 0020, and `docs/design/v0.1-scope-and-acceptance.md` §§2 and 4.

## Narrow scope

Require a fixed terminal result and no durable case event or report for a completed no-candidate capture. Use only behavior already authorized and implemented when specifying the execution surface.

## Out of scope

- A clean-capture receipt file, event, report, inventory, or certification record.
- A new schema or command invented solely for the terminal statement.
- Statistics based on a denominator of clean runs.

## Acceptance

- A clean capture returns the fixed terminal result owned by the capture skill.
- It creates no event, report, receipt, inventory, or other durable artifact.
- Verification demonstrates the relevant working and evidence locations remain unchanged.
- Reopen the ownership decision only if real output drift makes a compiled owner demonstrably cheaper, as ADR 0020 specifies.
