# Core task: target tracker differs from the working repository

Execute the supplied `to-prd` package. Do not interview. This is an offline fixture: do not call GitHub or mutate any repository. Return the one-line target-tracker notice, proposed issue title, exact seven-section PRD body, labels with roles, and publication decision.

The current working repository is `orchestrator`, but the PRD's work targets the `widget-runtime` repository and its tracker. The already-ratified seam is the `widget-runtime` public `widget check` command; restate it in one line and do not ask for confirmation again.

The PRD covers adding deterministic validation to that command, public diagnostics, tests, compatibility, and documentation. It is itself the independently implementable unit of work. The `widget-runtime` vocabulary maps category role `bug` to label `kind/bug` and AFK-ready state role `ready-for-agent` to label `agent-ready`.
