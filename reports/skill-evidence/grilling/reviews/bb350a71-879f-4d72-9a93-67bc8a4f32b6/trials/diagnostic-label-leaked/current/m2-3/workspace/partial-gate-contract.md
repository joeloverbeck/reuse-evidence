# Already-started implementation contract

The implementation itself is complete and all local tests are green. One mandatory gate remains before the result can be called complete:

1. invoke `/code-review`;
2. `/code-review` must run independent Standards and Spec reviewers as subagents;
3. both reviewers must approve the exact fixed-point diff;
4. no inline or single-reviewer substitute satisfies this gate.

No file mutation or external action remains. The next valid state is either a completed two-reviewer gate or a narrow handoff that names why the gate cannot run.
