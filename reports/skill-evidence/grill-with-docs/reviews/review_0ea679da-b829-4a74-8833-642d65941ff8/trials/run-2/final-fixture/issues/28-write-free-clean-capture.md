# Keep clean capture write-free

## Consumer effect

A maintainer can run routine capture without accumulating evidence, reports, or certification paperwork when no qualifying repeated responsibility exists.

The live bottleneck is operational fitness: clean capture must be cheap enough to repeat. The bearing evidence is the constitutional prohibition on clean-run records, the semantic clean-capture rule, ADR 0001, and ADR 0020's ownership of the fixed statement by the skill.

## Authority

- `docs/principles/FOUNDATIONS.md` §§12 and 15 require a clean capture to write nothing and reject derived control paperwork.
- `docs/principles/EVIDENCE-AND-DECISIONS.md` §5 requires a fixed terminal statement and no case event, report, or first-use inventory.
- `docs/principles/CONSUMER-CONTRACT.md` §2 guarantees write-free clean capture.
- `docs/adr/0001-manual-case-based-evidence-lifecycle.md` accepts the fixed result with no write.
- `docs/adr/0020-what-the-capture-skill-may-not-own.md` assigns the fixed no-candidate statement to the capture skill and forbids a command that prints it.

## Narrow scope

- State one fixed clean-capture terminal result in the `reuse-evidence-capture` package.
- Emit that result only after the bounded capture process finds no plausible qualifying repeated responsibility.
- Verify that the clean path leaves repository state and the user-local staging area unchanged.

## Out of scope

- A receipt file, case event, report, first-use inventory, status record, or certification log.
- A new compiled command, schema, event type, or durable denominator of capture runs.
- Claiming that a clean result proves architectural quality or the absence of all reusable responsibility.

## Acceptance

- Repeated clean runs emit the same fixed statement.
- A before/after comparison shows no repository or staging write on the clean path.
- The result is scoped to the bounded work inspected and makes no portfolio-health claim.
- Park or falsify this issue if real runs cannot keep the statement stable or cannot prove the no-write property; return to ADR 0020's review trigger before assigning a compiled owner.
