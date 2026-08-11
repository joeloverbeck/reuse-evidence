# Current-arm reproduction results

Raw prompt and fixture contract are frozen in `plan.md`. Five retained runs used fresh ephemeral Codex top-level sessions and unchanged target bytes.

| Run | Session | Witness expressed | Both companion skills loaded before adjudication | Applicable grilling references loaded before edit/close phases | Failure recurred | Artifact rubric |
|---|---|---|---|---|---|---|
| 1 | `019ff0f1-64bb-7490-a389-3e384b8e1477` | yes | yes | yes | no | pass |
| 2 | `019ff0f2-a760-7b82-b0ce-0e2d6ee730d6` | yes | yes | yes | no | pass |
| 3 | `019ff0f2-a751-7121-92e3-13c020685430` | yes | yes | yes | no | pass |
| 4 | `019ff0f4-92eb-78f2-ac9d-2c4effca36b3` | yes | yes | yes | no | pass |
| 5 | `019ff0f4-92df-7413-8011-606f9708ab02` | yes | yes | yes | no | pass |

For each run, the ordinary trace read `.claude/skills/grill-with-docs/SKILL.md`, `grilling/SKILL.md`, and `domain-modeling/SKILL.md` before adjudication. It also read `verification.md`, `adjudication.md`, `questions.md`, `recap.md`, and `execution.md` before the first fixture mutation and final closeout.

Each run changed exactly the four frozen paths:

- `CONTEXT.md`
- `docs/adr/0001-review-receipts.md`
- `docs/adr/README.md`
- `docs/workflow.md`

Every run preserved the upstream glossary, declined ADR 0002, amended ADR 0001 in place, reconciled the workflow and index, and returned the required domain-doc outcome. The artifact copies and executor final responses are retained under `trials/run-1/` through `trials/run-5/`; raw JSONL traces for runs 4 and 5 are retained as `run-4.jsonl` and `run-5.jsonl`.

Result: the candidate mechanism was **not reproduced with witnesses expressed** in all five retained runs. No candidate arm was built or run.
