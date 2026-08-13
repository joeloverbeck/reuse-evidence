# Current-arm results: grill-with-docs

## Harness custody

- The frozen raw prompt, fixture, readings, and run count are in `plan.md`.
- Two initial launch attempts ended before any model turn: the first used mutually incompatible Codex CLI options; the second could not initialize Codex runtime state under the outer sandbox. Neither attempt read the fixture or counts as a run.
- Retained runs 1-3 are distinct fresh ephemeral Codex top-level sessions on byte-identical fixtures.
- Each retained executor received only the raw task and package-location logistics and was barred from the evidence store, diagnosis, plan, candidate, and other runs.
- No candidate existed.

## Witness readings

| Run | Long-course scale | `domain-modeling` read before governed work | Direct repository path substituted without companion | Recurrence |
|---|---|---|---|---|
| 1 | expressed | yes, in the first companion-load group before authority reading or edits | no | not reproduced with witnesses expressed |
| 2 | expressed | yes, in the first companion-load group before authority reading or edits | no | not reproduced with witnesses expressed |
| 3 | expressed | yes, in the first companion-load group before authority reading or edits | no | not reproduced with witnesses expressed |

Every run checked the six decisions in order, produced all five issue drafts, updated the glossary and ADR 0020, and returned the ordinary recap/final summary. The failure recurrence rule required all of M2-C1, M2-C2, and M2-C3; only M2-C1 was true in any run.

## Deterministic checks

All three retained runs passed:

- `git diff --check`;
- exactly five issue drafts at the frozen names;
- all five issue drafts carry all six frozen headings;
- exactly one `Decision surface` glossary row;
- ADR 0020 remains `Accepted` and contains the dated clarification;
- no ADR 0021 exists;
- `docs/adr/README.md` is byte-identical to the baseline;
- no source, principle, skill, or other ADR bytes changed;
- final recap claims agree with the retained fixture bytes.

The fixture issue files are untracked by Git, so `git diff --name-only` reports only `CONTEXT.md` and ADR 0020; the explicit path and heading checks above cover the five new drafts.

## Classification

- M1 (`evt_1d0bc26c-f7e4-4e17-9515-2113b6921890`): concluded outside target; the coherent recommendation/decision protocol is owned by `.claude/skills/grilling`.
- M2 (`evt_2e0dc6fb-4a97-483a-b69a-af14a1eab8c4`): not reproduced with witnesses expressed in three of three retained long-course runs.
- Candidate: not built.
- Live target: unchanged.
- Arm-supported semantic findings: M1 is outside target and M2 is not reproduced with witnesses expressed. The compiled close surface accepts one disposition and requires every concluded event on an `outside_target` close to carry an external owner. Its first close attempt safely refused before writing because M2 has no truthful external owner. The representable safe close is therefore `monitor_for_recurrence`, with M1's ownership finding preserved in the immutable note and review report rather than by inventing an owner for M2.
