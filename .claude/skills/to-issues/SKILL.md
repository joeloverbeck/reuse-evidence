---
name: to-issues
description: Break a plan, spec, or PRD into independently-grabbable issues on the project issue tracker using tracer-bullet vertical slices. User-invoked only — a flow that ends at the issue-breakdown step should ask the user to type /to-issues rather than invoke this skill directly.
disable-model-invocation: true
---

# To Issues

Break a plan into independently-grabbable issues using vertical slices (tracer bullets).

This skill is user-invoked only. Proceed only when the user explicitly invoked `/to-issues` or `$to-issues`. If the user named a different skill instead, or an implicit workflow merely reaches issue breakdown, stop and ask the user to invoke one of those forms; do not substitute this workflow.

The issue tracker and triage label vocabulary should have been provided to you — run `/setup-matt-pocock-skills` if not.

## Process

### 1. Gather context

Work from whatever is already in the conversation context. If the user passes an issue reference (issue number, URL, or path) as an argument, fetch it from the issue tracker and read its full body and comments. A non-reference argument is breakdown direction, not noise — scope bounds, a judgment delegation, or approval granted in advance; step 4's argument-borne approval rule says how an advance approval is honored.

When the source cites precursor, sibling, or blocking issues or PRDs, exact-read the items that can affect scope, duplication, or dependencies before finalizing the breakdown. Metadata-only reads are enough when only identity, state, labels, or URL matter; read full bodies and comments when blocker, scope, coverage, or issue-body decisions depend on their contents. Classify each cited prerequisite as satisfied, an open blocker, or out of scope/deferred, and carry only unsatisfied blockers into the child issues.

When exact reads or codebase exploration reveal a conflict between the source and live repository or tracker state, or a proposed slice would introduce an unratified interface, custody, scope, reversibility, or dependency choice, record it as a decision delta or assumption rather than silently embedding it. Carry each material choice into the approval checkpoint; if it would change the breakdown's shape or publication course, stop for explicit user direction.

### 2. Explore the codebase (optional)

If you have not already explored the codebase, do so to understand the current state of the code. Issue titles and descriptions should use the project's domain glossary vocabulary, and respect ADRs in the area you're touching.

Look for opportunities to prefactor the code to make the implementation easier. "Make the change easy, then make the easy change."

### 3. Draft vertical slices

Break the plan into **tracer bullet** issues. Each issue is a thin vertical slice that cuts through ALL integration layers end-to-end, NOT a horizontal slice of one layer.

<vertical-slice-rules>

- Each slice delivers a narrow but COMPLETE path through every layer (schema, API, UI, tests)
- A completed slice is demoable or verifiable on its own
- Any prefactoring should be done first
- A non-code deliverable the source plan mandates (a spec, an ADR) is a valid slice — exempt from the layer-cutting rule, but still independently completable and verifiable; it typically blocks the code slices that consume it

</vertical-slice-rules>

### 4. Quiz the user

Present the proposed breakdown as a numbered list. For each slice, show:

- **Title**: short descriptive name
- **Blocked by**: which other slices (if any) must complete first
- **User stories covered**: which user stories this addresses (if the source material has them)

Ask the user:

- Does the granularity feel right? (too coarse / too fine)
- Are the dependency relationships correct?
- Should any slices be merged or split further?

Before asking for approval, present a compact publication checkpoint:

- **Target and prerequisites**: name the target tracker and summarize the satisfied, open-blocker, and out-of-scope/deferred prerequisite decisions that affect publication.
- **Decision deltas and assumptions**: list source-versus-live conflicts, new choices, and assumptions that materially affect interface, custody, scope, reversibility, or dependencies; state the recommended resolution and approval status. Approval of the slice titles alone does not ratify them.
- **Category and child-state disposition**: for every proposed child, name the category role and state role selected from the target tracker's canonical triage state machine (in this repo, `.claude/skills/triage/SKILL.md` §Roles), and name the parent's category role that any approved state transition must preserve. Each child must publish with exactly one category and one state; if no truthful category-and-state pair exists, stop rather than publish it. Approval of the slice titles alone does not ratify these roles.
- **Parent disposition**: state the parent's current triage state and the state it will have after publication. If the parent is already AFK-ready, do not create equally AFK-ready children unless the user explicitly approves the exact parent-label transition first; otherwise keep the parent as the sole AFK-grabbable ticket and park the child breakdown. Treat any parent-label change as a triage-state transition: name it before approval and never infer permission from approval of the slice boundaries alone. Prefer a repository-defined umbrella or coordination state for a parent that must remain open but not AFK-grabbable. If none exists, propose an existing non-AFK triage state that truthfully describes the parent's post-publication role and obtain approval for that exact transition. Never invent a label; validate the fallback against the target tracker's canonical triage state machine (in this repo, `.claude/skills/triage/SKILL.md` §Roles). If no valid non-AFK state exists, park the child breakdown. Never close the parent.
- **Coverage gate**: show the union of user stories covered by the proposed slices, plus every story explicitly deferred or out of scope and why. If the source has no user stories, state that coverage mapping is not applicable.

Iterate until the user approves the breakdown. If the approval question goes unanswered (timeout, user away), never publish on the timeout — park the breakdown and re-present it when the user returns.

**Argument-borne approval.** When the invocation argument itself grants approval — conditional ("if you judge this is needed, do so") or unconditional — treat it as the approval for this breakdown only if all three hold: no parent-label transition is proposed, no unratified decision delta remains, and no ambiguous scope or title collision is open. Then present the full publication checkpoint as a record in the final response and proceed to publish. If any of the three fails, stop for explicit user direction as usual — advance approval covers the breakdown's shape, never a parent mutation or an unratified delta.

### 5. Publish the issues to the issue tracker

For each approved slice, publish a new issue to the tracker where the parent issue lives; absent a parent issue, use the tracker of the repo the work targets — when that differs from the session's working repo, name the target tracker explicitly (one line) before publishing. Use the issue body template below. These issues are considered ready for AFK agents, so publish each with the approved category and state labels unless instructed otherwise.

Publish issues in dependency order (blockers first) so you can reference real issue identifiers in the "Blocked by" field.

Before any parent mutation, staging, or child create, keep a compact working publication ledger in conversation or session scratch — never in the public issue bodies. Record the approved title, story coverage, category role, and state role for every slice; the target tracker and proof for every mapped category and state label; prerequisite decisions; the approved parent category and state disposition; each dependency wave; and (as they become known) created identifier/URL and verification state.

Prove that every selected, repo-mapped category and state label exists before any parent mutation or child publication. A targeted label query is preferred; after a transient query failure, same-tracker parent or recent-issue metadata showing each exact label is acceptable fallback proof and must be named in the ledger. If a required label is genuinely absent, follow the target repo's label-creation convention using its triage vocabulary; if that convention or authority is unavailable, stop rather than inventing a label.

Make publication idempotent before any parent mutation or child create:

- Query a complete, paginated open-and-closed tracker inventory using exact title equality for every approved child title, or use an exact-title endpoint whose completeness is guaranteed. A limited result page is not proof of absence unless its completeness is independently established, and a broad text search is not proof of absence.
- If a same-title item matches the same parent and approved coverage, exact-read it and either adopt it into the ledger or stop for user direction. Surface unrelated title collisions and agree an unambiguous title before publishing.
- On resume after interruption or compaction, reconstruct the ledger by exact-reading every known issue identifier, then run the same complete exact-title inventory for every approved title that still lacks a verified identifier. Adopt only a match that exact-reads as the same parent and approved coverage; surface unrelated or ambiguous collisions and stop for user direction. Finish the reconstructed ledger before any parent mutation, staging, or create. Never infer from a missing local response that a remote create did not succeed.

If the approval checkpoint authorized a parent-label transition, apply and exact-read it into the ledger only after the ledger exists, label proof is recorded, and the initial title guards and any adoption or collision decisions are complete, and immediately before the first dependency wave. Verify that the parent retains exactly one approved category role and carries the approved post-publication state role. This is the only parent mutation this skill may perform; if the approved transition cannot be verified, stop before publishing children.

Stage and publish one dependency wave at a time so later bodies can use verified blocker identifiers:

1. Stage each issue body outside the repository, or use the tracker's equivalent draft surface.
2. Check required headings and approved coverage, then scan for workstation/scratch paths, patch or conflict markers, unexpanded placeholders, and authoring notes such as `TODO` or `TBD`.
3. Read the staged body once for agent readiness, then run the fresh exact-title guard for that not-yet-created item. Exclude children already published or adopted earlier in the same run.
4. Publish using the tracker's file-backed body option where available (`--body-file` for `gh`) rather than inline shell text for Markdown-heavy bodies, and apply the approved category and state labels.
5. Record the returned identifier and URL before advancing the ledger.

After each create or adoption — or after a dependency wave when later bodies need its identifiers — exact-read every created or adopted issue. Verify title, open state, exactly one label for the approved category role, exactly one label for the approved state role, no conflicting category or state labels, required headings, the full Parent and Blocked by sections, and the approved story mapping. If anything is wrong, edit and reverify it; do not advance a dependent wave from an unverified blocker.

After successful verification, remove only the exact temporary body files created by this run, never a glob or parent directory, and prove those paths no longer exist. Record cleanup proof in the ledger. Before the final response, run a family-level metadata readback and confirm that every approved slice has a verified issue with the approved category and state roles, coverage is unchanged since approval, the parent retains its approved category and post-publication state, and temporary-file cleanup is accounted for. If any row is missing or any readback fails, keep working instead of reporting completion.

<issue-template>
## Parent

A reference to the parent issue on the issue tracker (if the source was an existing issue, otherwise omit this section). If the source has user stories, add one concise mapping such as `Addresses stories 1, 4, 7` or `Enables stories 1-4; direct user-facing coverage N/A`. Foundational enablement does not count as direct coverage: downstream slices or explicit deferrals must still close the coverage union.

## What to build

A concise description of this vertical slice. Describe the end-to-end behavior, not layer-by-layer implementation.

Avoid specific file paths or code snippets — they go stale fast. Exception: if a prototype produced a snippet that encodes a decision more precisely than prose can (state machine, reducer, schema, type shape), inline it here and note briefly that it came from a prototype. Trim to the decision-rich parts — not a working demo, just the important bits.

## Acceptance criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Blocked by

- A reference to the blocking ticket (if any)

Or "None - can start immediately" if no blockers.

</issue-template>

The template is a floor, not a ceiling — check the target repo's contribution and conformance conventions (CLAUDE.md, principles or agents docs) for mandatory issue-body sections and append them to every published issue.

Do NOT close any parent issue. Do not change its labels, comments, or other tracker state unless the publication checkpoint named the exact mutation and the user explicitly approved it. If an already AFK-ready parent remains unchanged, park the child breakdown instead of publishing equally AFK-ready children.
