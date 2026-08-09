---
name: code-review
description: Review the changes since a fixed point (commit, branch, tag, or merge-base) along two axes — Standards (does the code follow this repo's documented coding standards?) and Spec (does the code match what the originating issue/PRD asked for?). Runs both reviews in parallel sub-agents and reports them side by side. Use when the user wants to review a branch, a PR, work-in-progress changes, or asks to "review since X".
---

Two-axis review of the diff between `HEAD` and a fixed point supplied by the user or invoking workflow:

- **Standards** — does the code conform to this repo's documented coding standards?
- **Spec** — does the code faithfully implement the originating issue / PRD / spec?

Both axes run as **parallel sub-agents** so they don't pollute each other's context, then this skill aggregates their findings.

The issue tracker should have been provided to you — run `/setup-matt-pocock-skills` if `docs/agents/issue-tracker.md` is missing.

## Process

### 1. Pin the fixed point

Use the fixed point supplied by the user or invoking workflow — a commit SHA, branch name, tag, `main`, `HEAD~5`, etc. Record which source supplied it. If neither supplies one, or the supplied value is ambiguous, ask the user rather than choosing.

Resolve both endpoints once and record their exact commit SHAs: resolve the supplied fixed point as `fixed_point_sha`, and resolve `HEAD` as `reviewed_head_sha`. Build every review input from those captured SHAs: `git diff <fixed-point-sha>...<reviewed-head-sha>` (three-dot, so the comparison is against the merge-base) and `git log <fixed-point-sha>..<reviewed-head-sha> --oneline`. Never leave a symbolic `HEAD` in a reviewer command after capture.

Before going further, confirm both values resolve as commits (`git rev-parse <value>^{commit}`), record `reviewed_head_sha`, and confirm the pinned diff is non-empty. A bad ref or empty diff should fail here — not inside two parallel sub-agents.

### 2. Identify the spec source

The user or invoking workflow may supply an authoritative spec source directly; record its provenance and use it without reopening the caller's selection. Otherwise, look for the originating spec in this order:

1. Issue references in the commit messages (`#123`, `Closes #45`, GitLab `!67`, etc.) — fetch via the workflow in `docs/agents/issue-tracker.md`.
2. A path the user passed as an argument.
3. A PRD/spec file under `docs/`, `specs/`, or `.scratch/` matching the branch name or feature.
4. If nothing is found or the candidates conflict, ask the user which source is authoritative. If they say there isn't one, the **Spec** sub-agent will skip and report "no spec available".

### 3. Identify the standards sources

Build and record the applicable standards-source list before launching review. Always inspect the repository's root agent instructions (`AGENTS.md` and/or `CLAUDE.md`, when present); use the changed paths, artifact type, and caller context to identify domain- or authoring-specific standards; then add general repository standards such as `CODING_STANDARDS.md` or `CONTRIBUTING.md`. Record every applicable source. If the search finds none beyond any root instructions and the smell baseline, record `no additional applicable standard`.

In embedded use, active invoking-workflow contracts and retained implementation evidence are applicable standards sources whenever they constrain changed artifacts. Include the current `/tdd` contract when changed tests or TDD evidence are in scope, the current `/implement` contract for repair/review handoff, and the caller-selected run sheet or evidence ledger when it defines agreed seams, commands, or evidence fields. Point to the owning workflow contract instead of copying its schema, and treat these materials as standards/process inputs rather than substitutes for the product spec.

On top of whatever the repo documents, the Standards axis always carries the **smell baseline** below — a fixed set of Fowler code smells (_Refactoring_, ch.3) that applies even when a repo documents nothing. Two rules bind it:

- **The repo overrides.** A documented repo standard always wins; where it endorses something the baseline would flag, suppress the smell.
- **Always a judgement call.** Each smell is a labelled heuristic ("possible Feature Envy"), never a hard violation — and, like any standard here, skip anything tooling already enforces.

Each smell reads *what it is* → *how to fix*; match it against the diff:

- **Mysterious Name** — a function, variable, or type whose name doesn't reveal what it does or holds. → rename it; if no honest name comes, the design's murky.
- **Duplicated Code** — the same logic shape appears in more than one hunk or file in the change. → extract the shared shape, call it from both.
- **Feature Envy** — a method that reaches into another object's data more than its own. → move the method onto the data it envies.
- **Data Clumps** — the same few fields or params keep travelling together (a type wanting to be born). → bundle them into one type, pass that.
- **Primitive Obsession** — a primitive or string standing in for a domain concept that deserves its own type. → give the concept its own small type.
- **Repeated Switches** — the same `switch`/`if`-cascade on the same type recurs across the change. → replace with polymorphism, or one map both sites share.
- **Shotgun Surgery** — one logical change forces scattered edits across many files in the diff. → gather what changes together into one module.
- **Divergent Change** — one file or module is edited for several unrelated reasons. → split so each module changes for one reason.
- **Speculative Generality** — abstraction, parameters, or hooks added for needs the spec doesn't have. → delete it; inline back until a real need shows.
- **Message Chains** — long `a.b().c().d()` navigation the caller shouldn't depend on. → hide the walk behind one method on the first object.
- **Middle Man** — a class or function that mostly just delegates onward. → cut it, call the real target direct.
- **Refused Bequest** — a subclass or implementer that ignores or overrides most of what it inherits. → drop the inheritance, use composition.

### 4. Launch both sub-agents in parallel

When both axes are available, use the active client's sub-agent surface to launch isolated Standards and Spec reviewers concurrently. Preserve the two distinct prompts below. If the client requires separate launch calls, start both before awaiting either; client-specific tool names or sub-agent types are examples for that client, not part of this skill's contract.

Every reviewer packet must name `reviewed_head_sha`, use only the pinned diff and commit list from step 1, and require the axis report to open with `Reviewed HEAD: <reviewed-head-sha>`.

Every reviewer packet must also require each actionable finding to end with `Repair class: observable behavior | behavior-neutral` and `TDD re-entry required: yes | no`. Set the latter to `yes` whenever satisfying the finding requires changing observable behavior; otherwise set it to `no`. These fields classify and route findings only: `/tdd` and `/implement` retain ownership of red → green and repair mechanics, and reviewers must not edit.

Before dispatch, allocate and record one `review_pass_id` (`R1`, `R2`, ...) that is unique and monotonically increasing within the implementation workflow. Never reuse an allocated pass ID, including after discarding a pass because `HEAD` moved. Give every launched axis reviewer that ID and the retained list of prior unresolved findings for its axis. Require each axis report to open with `Review pass: <review-pass-id>` immediately after its `Reviewed HEAD` line.

Require every new actionable finding to carry `Finding ID: <review-pass-id>-standards-<ordinal>` or `Finding ID: <review-pass-id>-spec-<ordinal>`, with the ordinal assigned in axis-report order. Once emitted, the finding ID is immutable: a later pass retains it only for the same unresolved finding, while every distinct later finding receives a new ID even when it concerns the same seam, test, or evidence row. Never rename an earlier finding or extend it to absorb a later one. Keep the finding ID and routing fields together when returning the finding; `/tdd` retains ownership of downstream review-reentry reconciliation.

**Standards sub-agent prompt** — include:

- The exact reviewed HEAD SHA, pinned full diff command, and pinned commit list.
- The complete standards-source list from step 3, including any active workflow contracts and retained run sheet/evidence, **plus the smell baseline from step 3** pasted in full. Include both the selected-source list and the baseline even if the reviewer inherits context or can read the filesystem, so its prompt is self-contained and applies the exact inputs this run selected.
- Two coverage lines immediately after `Review pass`: `Standards sources checked: <each selected standards-source path/name>; smell baseline` and `Workflow evidence checked: <applicable workflow source -> retained evidence row or identity>` (or `Workflow evidence checked: N/A`). A `No actionable findings` report still owes both lines. `N/A` is valid only when no workflow source selected in step 3 constrains changed tests or evidence.
- The brief: "Report — per file/hunk where relevant — (a) every place the diff violates a documented standard: cite the standard (file + the rule); (b) any baseline smell you spot: name it and quote the hunk; and (c), where workflow sources constrain tests or evidence, whether changed tests and evidence rows honor the agreed seams, runnable commands, and evidence fields: cite the workflow source and quote the changed hunk or row. Distinguish hard violations from judgement calls — documented-standard breaches can be hard, but baseline smells are always judgement calls, and a documented repo standard overrides the baseline. Skip anything tooling enforces. Under 400 words."

**Spec sub-agent prompt** — include:

- The exact reviewed HEAD SHA, pinned diff command, and pinned commit list.
- The path or fetched contents of the spec.
- The brief: "Report: (a) requirements the spec asked for that are missing or partial; (b) behaviour in the diff that wasn't asked for (scope creep); (c) requirements that look implemented but where the implementation looks wrong. Quote the spec line for each finding. Under 400 words."

If the spec is missing, skip the Spec sub-agent and note this in the final report.

After all launched reviewers return, verify that `git rev-parse HEAD^{commit}` still equals `reviewed_head_sha`. If HEAD moved, discard every axis report from that pass and restart at step 1 against newly captured endpoints; never combine reports produced against different trees.

### 5. Aggregate

Present the two reports under `## Standards` and `## Spec` headings, verbatim or lightly cleaned, retaining each report's `Reviewed HEAD: <reviewed-head-sha>` line. Do **not** merge or rerank findings — the two axes are deliberately separate (see _Why two axes_).

After the two axis reports, add `## Findings fixed during review`. If an earlier pass in the same implementation workflow found issues that were repaired before this rerun, record each finding's axis, original finding, repair class, whether TDD re-entry was required, repair, rerun-reviewed HEAD SHA, and rerun evidence; otherwise write `None`. A green rerun reports no residual findings, not that no findings were ever found. In embedded use, return the ledger and routing fields to the invoking workflow; `/implement` remains the owner of repair and rerun mechanics, and `/tdd` owns any required review-reentry cycle.

When the ledger is present, use this exact schema:

| Review pass | Finding ID | Axis | Original finding | Repair class | TDD re-entry required | Repair | Rerun-reviewed HEAD SHA | Rerun evidence |
|---|---|---|---|---|---|---|---|---|

Before handoff, run an aggregate-conformance gate. Build the set of every actionable finding ID emitted across all retained passes and compare it in both directions with the union of IDs in the final residual axis reports and the fixed-during-review ledger; both differences must be empty. Require every fixed row to populate every schema cell, preserve the finding's original axis, text, repair class, and TDD routing, and use the final `reviewed_head_sha` as its rerun-reviewed SHA. Require every launched final axis report, every fixed row, and the one-line summary to name that same reviewed SHA. `None` is valid only when the pass census contains no finding that was fixed before the final report. If any identity is missing, duplicated with conflicting content, renamed, or absorbed into another finding, or any required value is absent, the review is not green and must not be handed off.

For every launched final Standards report, parse `Standards sources checked` and compare its source entries in both directions with step 3's selected standards-source list; both differences must be empty, and the line must also name the smell baseline. When a selected workflow source constrains tests or evidence, require `Workflow evidence checked` to name that source and the retained evidence row or identity it checked; accept `N/A` only when none applies. These checks also bind a report with no actionable findings. A missing or incomplete coverage line makes the Standards report incomplete: discard it and rerun that axis before handoff.

End with a one-line summary naming the reviewed HEAD SHA, residual findings per axis, the worst residual issue _within each axis_ (if any), whether every actionable finding carries its stable identity and both routing fields, whether the aggregate-conformance gate passed, and whether the fixed-during-review ledger is `None` or present. Don't pick a single winner across axes — that's the reranking the separation exists to prevent.

## Why two axes

A change can pass one axis and fail the other:

- Code that follows every standard but implements the wrong thing → **Standards pass, Spec fail.**
- Code that does exactly what the issue asked but breaks the project's conventions → **Spec pass, Standards fail.**

Reporting them separately stops one axis from masking the other.
