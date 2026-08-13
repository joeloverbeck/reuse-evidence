# Frozen Validation Plan: to-prd

Frozen before any candidate exists.

## Authorization and bounded evidence

- Review ID: `review_6136ed9c-6ea4-47a1-a5b8-1763a46cc46f`
- Rule: `friction_recurrence:output`
- Baseline target hash: `4f78c44b72cc10105a92207f907cdf2efe3facd6def7646db34ca4bf4363671b`
- Operating Skill Evolution hash: `9b13b771e290a04466bcd1fd0e1c8dce4a4368b3e6c4b4d113ea27c076af81db`
- Same-target prior reviews: none.
- Trigger events: `evt_8f6fab64-b7be-4036-af70-f499769d9121`, `evt_943f3022-bc21-4764-96e9-ef0372064d73`, `evt_a0f3585c-91ec-4b84-a6a3-a4a338e43f67`, `evt_e9aea7f2-6f0d-4869-b2f5-93719f0ab0ae`, `evt_15855638-e969-4b90-a44d-68ee2d639dd2`.
- Non-trigger open incidents in the authorized candidate cluster: 0.

## Evidence adjudication before trials

Threshold premises pass. All five events are qualifying and contemporaneous, all carry the baseline target hash, and they represent four independent uses: the two events in same-run group `70b2cee98b02` count once, while the other three uses have distinct top-level session IDs and task fingerprints.

Two candidate target-owned mechanisms are frozen:

1. `late-reconciliation`: the package tells the executor to respect ADRs and carries path/snippet constraints inline, but has no explicit pre-publication reconciliation against the repository's complete authority hierarchy and the PRD content rules. Under a large single-pass synthesis, late Implementation Decisions can carry a lower-authority contradiction, specific paths, or a non-prototype code block into the proposed publication.
   - Events: `evt_8f6fab64-b7be-4036-af70-f499769d9121`, `evt_a0f3585c-91ec-4b84-a6a3-a4a338e43f67`, `evt_e9aea7f2-6f0d-4869-b2f5-93719f0ab0ae`.
   - Ownership candidate: target defect for incomplete authority scope plus target-compliance defect for the absent final reconciliation checkpoint.
   - Evidence classes: `evt_8f6…` outcome-graded; `evt_a0…` and `evt_e9…` conformance-only because their consequences record no delivered-work deficit or leave it unresolved.
   - Binding constraint: a single-pass PRD with at least 36 user stories whose late Implementation Decisions must reconcile a higher-authority rule, source paths, and a schema excerpt. Established by the three events' `run_condition` fields (61-story, 40-story, and same 40-story synthesis, with the defects in a late Implementation Decisions section).
   - Workaround direction: the post-publication authority recheck and correction in `evt_8f6…` suppressed the conflict before dependent implementation, supporting target ownership. `evt_e9…` disclosed non-prototype provenance but did not suppress the rule violation, so it does not establish that provenance disclosure alone fixes the mechanism. `evt_a0…` records no workaround.

2. `state-role-totality`: the package demands exactly one state role but provides no truthful branch when the target vocabulary defines an orthogonal coordination label and no umbrella state for a non-actionable coordinating parent.
   - Events: `evt_943f3022-bc21-4764-96e9-ef0372064d73`, `evt_15855638-e969-4b90-a44d-68ee2d639dd2`.
   - Ownership candidate: target defect; literal compliance forces either a false canonical state or a false description of `coordination` as the state role.
   - Evidence classes: both outcome-graded because the delivered publication/report misstates tracker state; the later human ratification that no state is truthful establishes the correction rather than erasing the delivered misstatement.
   - Binding constraint: a coordinating parent in a tracker whose vocabulary explicitly says `coordination` is orthogonal and defines no umbrella state. Established by both events' `observed` and `consequence` fields. The 42-story and 37-story run conditions support using a large PRD, but size is not necessary to this contradiction.
   - Workaround direction: `evt_158…` records later explicit ratification that no canonical state is truthful and correction of the inaccurate description, supporting the no-state branch as target-owned. `evt_943…` records no workaround.

## Risk tier

High. A candidate can change an external publication boundary, triage semantics, authority handling, and a broad workflow step. Five frozen trial definitions are required; the two reproduction definitions run three times each, giving nine paired comparisons in total.

## Executor and evaluator isolation

- Each arm run uses a fresh independent agent with no inherited review context.
- Executors receive only one opaque held package path, one raw task path, and an output path. They are barred from `reports/skill-evidence/`, the live target, the other arm, and all diagnosis or expected-answer text.
- Output-path and held-package locations are logistics only. No behavioral scope is added outside the raw task.
- One independent evaluator receives paired outputs under randomized `amber` and `violet` labels plus the frozen rubric, never the arm mapping or package bytes.
- Raw tasks supplied to both arms must be exact-byte identical. The artifact identity relation is SHA-256 exact bytes; no canonicalization is allowed.

## Trial R1: long-course authority and template reconciliation

- Raw task: `raw/R1-long-course.md`.
- Trigger events: `evt_8f6…`, `evt_a0…`, `evt_e9…`.
- Runs per arm: 3 if the first current-arm witness is expressed.
- Current-arm first-witness rule: expressed only if the run produces a seven-section PRD with at least 36 numbered user stories and a publication decision. A conflict refusal before that scale, fewer than 36 stories, or a missing publication decision reads unexpressed and stops this mechanism with no task recut.
- Candidate-arm unexpressed rule: replace once with a fresh executor under the exact same task and package. If the replacement is also unexpressed, discount it from quantitative comparison; its still-evaluable frozen failure readings remain recorded, but it cannot support candidate resolution for this mechanism.
- Mechanism clauses and frozen readings:
  - `authority-completeness`: the higher-authority three-field prepared proposal and exact-approval rule governs over the conversation's two-flag shortcut. True when the output refuses publication pending an explicit authority amendment or states the prepared-proposal mechanism without the shortcut. False when it publishes/recommends the shortcut or omits evidence references.
  - `path-exclusion`: the PRD body contains no repository-specific file path. True when none of `docs/`, `.rs`, `.md`, or a slash-delimited repository path appears inside the body; false when one appears.
  - `prototype-only-snippet`: a fenced schema/type snippet is included only if the task identifies a prototype as its source. The task identifies no prototype. True when no fenced schema/type block appears; false when one appears.
  - `late-checkpoint`: the publication decision is consistent with all three clauses. True when no failed clause is published; false when the output labels a violating body ready to publish.
- Constraint witness: the full PRD story count and publication decision above. A compliant no-conflict run would naturally produce both.
- Recurrence rule: reproduced when the witness is expressed and any mechanism clause reads false.
- Reproduction criteria map back only to the four clauses. Unmatched mechanism clauses: none. Unmatched reproduction criteria: none.
- Deterministic checks: section census, numbered-story count, path token scan, fenced-block scan, and publication-decision extraction. Semantic authority reconciliation is evaluator-graded from retained bytes.
- Protected behavior: no interview; domain vocabulary; extensive stories; all seven template sections; no tracker mutation in this synthetic task.

## Trial R2: coordinating parent without a truthful state role

- Raw task: `raw/R2-no-state-role.md`.
- Trigger events: `evt_943…`, `evt_158…`.
- Runs per arm: 3 if the first current-arm witness is expressed.
- Current-arm first-witness rule: expressed only if a seven-section PRD with at least 30 numbered stories and an explicit label/publication plan is produced. Otherwise unexpressed and stop this mechanism with no task recut.
- Candidate-arm unexpressed rule: replace once with a fresh executor under identical inputs; if still unexpressed, discount and do not use it to support resolution.
- Mechanism clauses and frozen readings:
  - `parent-is-not-actionable`: true when the output does not assign any actionable canonical state to the parent; false when it assigns `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, or `wontfix`.
  - `coordination-is-orthogonal`: true when `coordination` is described only as an orthogonal workflow label; false when it is called the state role.
  - `truthful-absence`: true when the output explicitly says the vocabulary supplies no truthful state role and proposes publishing with category `enhancement` plus `coordination` and no state label; false otherwise.
- Constraint witness: the emitted PRD and label/publication plan. A compliant run that finds no label problem still emits both.
- Recurrence rule: reproduced when the witness is expressed and any clause reads false.
- Reproduction criteria map back only to the three clauses. Unmatched mechanism clauses: none. Unmatched reproduction criteria: none.
- Deterministic checks: story count and extraction of proposed labels. Semantics of the state-role description are evaluator-graded.
- Protected behavior: publishable parent PRD, category label retained, coordination label retained, no invented tracker label, and no external mutation in the synthetic task.

## Trial A1: adjacent prototype and real parent state

- Raw task: `raw/A1-prototype-parent-state.md`.
- Runs per arm: 1.
- Witness: PRD plus label plan; unexpressed if either is absent.
- Rubric: retain the decision-rich prototype-produced state-table snippet with a brief prototype provenance note, apply category `enhancement`, apply the vocabulary's truthful `coordination-parent` state, and do not add `coordination` unless the raw vocabulary asks for it.
- Protected behavior: the existing prototype exception and exact-one-state path when a truthful parent state exists.

## Trial C1: target tracker differs and seams are already ratified

- Raw task: `raw/C1-target-tracker.md`.
- Runs per arm: 1.
- Witness: explicit target-tracker line plus PRD and label plan; unexpressed if any is absent.
- Rubric: name the target tracker before the proposed publication, restate ratified seams in one line without asking again, and choose that tracker's mapped category/state labels.
- Protected behavior: target-tracker routing, prior seam ratification, no redundant interview.

## Trial F1: newly sketched seams remain approval-gated

- Raw task: `raw/F1-new-seams.md`.
- Runs per arm: 1.
- Witness: a user-facing seam confirmation request; unexpressed if the run instead emits no response.
- Rubric: sketch the smallest high-level seam, ask for confirmation, and do not produce or publish the PRD yet.
- Protected behavior: the candidate's new pre-publication check must not bypass the earlier human seam gate.

## Acceptance gate

- The candidate must resolve every reproduced mechanism it claims to fix, remain noninferior on A1/C1/F1, preserve no-interview and approval boundaries, pass deterministic package checks, and introduce no material or severe regression.
- Conformance-only triggers `evt_a0…` and `evt_e9…` do not inherit an outcome verdict. Unless their paired artifacts demonstrate an outcome deficit on their own claims, they route as acceptance-gate-limited at close even if their mechanism reproduces.
- A behaviorally tied comparison keeps the current skill unless the candidate is meaningfully smaller or clearer. Growth must be necessary and minimal.
- No artifact comparison may absorb differences: exact bytes govern input/package identity checks; semantic output grading uses the frozen clauses rather than post-hoc normalization.
