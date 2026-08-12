# Current-arm results: grill-with-docs

## Retained runs

Trial R retained runs 2, 3, and 4 as the three frozen current-arm runs. Each used a fresh Codex top-level session, the unchanged target hash `50ae74251f72b370e4b226fa12169ffaa18cac49a814ca5a5725460fe4e3063a`, the exact frozen prompt, and byte-identical fixture inputs before package mounting.

Run 1 is retained for audit but excluded from the trial count. Its observer redirected `raw.jsonl` inside the executor workspace, so the executor saw one extra untracked logistics file. Runs 2–4 captured JSONL and the final response outside the fixture and have no such mismatch. The replacement did not recut the prompt, fixture, witness, or rubric.

## Witness readings

| Run | Binding condition | Companion-loading failure | Artifact result |
|---|---|---|---|
| 2 | expressed | not reproduced | pass |
| 3 | expressed | not reproduced | pass |
| 4 | expressed | not reproduced | pass |

In all three retained runs:

- a complete `grill-with-docs`, `grilling`, and `domain-modeling` package read preceded adjudication and every domain-document edit;
- all five architecture-review candidates were dispositioned in order;
- candidates 1–4 were rejected without implementing their prohibited effects;
- candidate 5 reached the ADR-worthy structural phase and was implemented;
- the changed task paths were exactly `CONTEXT.md`, `docs/adr/0002-terminal-meaning-owner.md`, `docs/adr/README.md`, `src/commands.rs`, and `src/terminal.rs`;
- the shared success mapping had one owner and both commands delegated to it;
- ADR 0002 was accepted and indexed without replacing ADR 0001;
- the glossary defined only the project-layer term; and
- the final recap matched the artifact state.

The current arm therefore expressed the distinct late-crystallization condition three times without reproducing the omitted-companion mechanism or an outcome deficit. No candidate was built, and the adjacent/regression trials were not reached.
