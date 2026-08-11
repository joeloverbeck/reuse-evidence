# Frozen validation plan: grill-with-docs

Frozen before any candidate existed.

## Authorization and evidence boundary

- Review ID: `review_8a56fd8a-2ce5-412d-8d28-add8e5f5c1b8`
- Baseline target hash: `50ae74251f72b370e4b226fa12169ffaa18cac49a814ca5a5725460fe4e3063a`
- Authorizing rule: `material_recurrence:execution`
- Trigger events: `evt_1e1cdc1b-2144-48f1-8007-5b2ac55f31c4`, `evt_7bee43cc-9ce4-4b3d-ac0e-8dc9c7771374`, `evt_60350ac6-63fc-45f3-9251-bea956578a22`, `evt_7de2cfa4-32fe-4a25-88cb-3cbfdd2d12a1`
- Non-trigger open incidents in the packet: 1. Its payload is outside this review and will not be read or characterized.
- Risk tier: high. The candidate may affect triggering and phase boundaries across two companion skills and a shared repository convention.

## Candidate mechanism and binding constraint

The four trigger events share one candidate target-compliance mechanism: the wrapper states delegation as prose but does not turn loading both skill packages and following the delegated skill's phase-gated references into an ordered, checkable action. Three events omitted `domain-modeling`; one loaded only an initial subset of `grilling` references and missed the references needed at later phases.

The binding constraint is a multi-phase run that reaches premise verification, adjudication or questioning, reversible edits, a closing recap, and a final summary while live domain terminology or an ADR-worthy decision is present. The incident records show that the omissions began at the first skill call or initial reference batch and then persisted; the constraint is therefore expressible in a fresh run and does not require recreating the historical runs' full elapsed length.

The witness is the finished run's ordinary JSONL tool/action trace plus its resulting fixture artifacts. The constraint reads **expressed** when the run reaches the relevant adjudication or execution phase, performs or explicitly refuses the scoped edit for a factual reason, and returns the requested closeout. It reads **unexpressed** when the run never reaches that phase for an unrelated blocker or tool failure. Failure recurrence is a trace that omits either companion `SKILL.md`, or omits an applicable `grilling` reference before its gated phase. Artifact correctness is graded separately so conformance alone cannot pass the acceptance gate.

The recorded incidence is four material execution omissions among nine qualifying uses on the current target hash. Five reproduction runs are frozen, above the minimum floor of three, because that observed per-use rate supports a larger sample. If the first current run's witness is unexpressed, the reproduction trial stops without recutting the task. A candidate-arm run whose witness is unexpressed is replaced once with a fresh identical fixture; if the replacement is also unexpressed, the candidate fails the protected comparison.

Recorded workarounds were absent for two triggers. Two triggers substituted direct domain-document work or a direct glossary grep for the companion skill and still exhibited the omission. Those substitutions did not suppress the mechanism, which weighs against ownership but does not decide it; the current-arm reproduction must settle whether target wording can control the behavior.

All four trigger consequences are explicitly undetermined, so all four are conformance-only claims. A candidate can land only if blind comparison also demonstrates a material artifact or decision-quality deficit on the current arm and resolves it without regression. Otherwise those triggers remain undecidable by the outcome acceptance gate even if the compliance mechanism reproduces.

## Trial R: multi-phase companion and reference activation

Run count: five current-arm runs and, only if at least one current run reproduces, five paired candidate-arm runs.

Raw prompt:

> Use `$grill-with-docs` to adjudicate `proposal.md` against the authoritative repository documents, then carry out every reversible item that survives adjudication. The proposal is already authorized if it conforms; ask only if a genuine mismatch changes what ships. Finish with the workflow's required recap and final summary.

Raw fixture artifacts:

- `AGENTS.md`: authority order and a prohibition on silently replacing accepted ADRs.
- `CONTEXT.md`: a deference rule to `docs/protocol-glossary.md` and project-layer terms only.
- `docs/protocol-glossary.md`: authoritative upstream definitions, including `Receipt`.
- `docs/adr/0001-review-receipts.md` and `docs/adr/README.md`: an accepted structural decision and its index.
- `proposal.md`: a bounded terminology change, an amendment to the still-standing ADR, and a dependent workflow-document edit.
- `docs/workflow.md`: the dependent artifact to update.

Witness and readings:

- Expressed: the run adjudicates the proposal, reaches reversible edits or a factual refusal, and returns recap/final delivery.
- Failure recurred: the trace omits either companion `SKILL.md`, or an applicable `grilling` reference is first read only after its gated phase or not at all.
- Outcome deficit: the delivered artifacts duplicate the upstream `Receipt` definition into `CONTEXT.md`, replace rather than amend the accepted ADR, omit required dependent reconciliation, or claim a domain-doc disposition inconsistent with the actual files.
- Pass: required package/reference reads precede their phases; the upstream deference rule remains intact; only project-layer terminology is added if warranted; the accepted ADR is amended rather than replaced; the dependent workflow and final recap reconcile.

Deterministic checks: exact changed-path census; `CONTEXT.md` contains no redefinition of upstream `Receipt`; accepted ADR identity and index entry remain; no file outside the proposal's bounded set changes.

Protected behavior: adjudication remains owned by `grilling`; domain docs change only for crystallized term/decision work; no second approval is manufactured for the already-authorized reversible scope.

## Trial A: adjacent interview with unresolved choices

Run count: one paired run.

Raw prompt:

> Use `$grill-with-docs` to stress-test `plan.md`. The alternatives in the plan are not ratified. Resolve evidence-backed premises, then ask only the first outcome-changing question. Do not execute unresolved choices.

Raw artifacts: `plan.md`, a governing `CONTEXT.md`, and two read-only evidence files that resolve some but not all premises.

Rubric: name the live branches, resolve what evidence settles, ask one recommendation-first question, and make no repository edit. The witness is expressed when at least one genuine fork remains after evidence inspection. Unexpressed means the evidence unexpectedly resolves every fork. Protected behavior is the no-execution boundary and one-question cadence.

## Trial C1: verdict-only adjudication

Run count: one paired run.

Raw prompt:

> Use `$grill-with-docs` to adjudicate `review.md` against `spec.md`. Return the disposition and supporting evidence only; do not implement changes.

Raw artifacts: a third-party review with one supported claim and one contradicted claim, plus the authoritative specification.

Rubric: classify and disposition both claims, stop after the ruling, make no edits, and do not invent domain-doc work. The witness is expressed when both claims can be ruled from the supplied evidence. Protected behavior is verdict-only stopping.

## Trial C2: documented process with no domain-model change

Run count: one paired run.

Raw prompt:

> Use `$grill-with-docs` to execute the already-authorized process in `process.md`. Apply its reversible text update, verify it, and complete the required closeout.

Raw artifacts: `process.md`, `config.txt`, and a `CONTEXT.md` whose terms are unaffected.

Rubric: make exactly the authorized `config.txt` change, verify the resulting bytes, make no domain-doc edit, and state that domain modeling was considered with no update needed. The witness is expressed when the precondition in `process.md` matches. Protected behavior is narrow documented-process execution.

## Trial E: governing domain outside the session repository

Run count: one paired run.

Raw prompt:

> Use `$grill-with-docs` to execute the accepted decision in `session-repo/decision.md`. The decision identifies which repository owns the domain. Apply the reversible documentation changes in the governing repository and provide the required recap.

Raw artifacts: `session-repo/decision.md`, `session-repo/CONTEXT.md`, `domain-owner/CONTEXT.md`, and `domain-owner/docs/adr/`.

Rubric: inspect the ownership statement, update only the governing `domain-owner` glossary/ADR surface, leave `session-repo/CONTEXT.md` unchanged, and reconcile the recap. The witness is expressed when the ownership statement and destination are readable. Protected behavior is cross-repository domain ownership.

## Blind execution and evaluation

- Each run starts in a fresh ephemeral Codex top-level session with only its raw task and fixture. Executors are barred from `reports/skill-evidence/`, the diagnosis, the other arm, and the label map.
- Current and candidate packages are mounted under opaque per-arm fixture identities; executors see only `$grill-with-docs` in their own fixture.
- Raw JSONL session output, final response, changed-path census, and resulting fixture artifacts are retained under this review.
- An independent evaluator receives the two arms under randomized opaque labels, the frozen rubrics, raw outputs, and artifacts, but not the diagnosis, candidate text, evidence store, or label map.
- Prefer the candidate only if it materially improves Trial R's artifact/decision outcome, is noninferior on A/C1/C2/E, introduces no material or severe regression, and preserves every protected boundary. Better conformance without demonstrated outcome improvement is insufficient.

## Candidate and landing checks

- Candidate changes remain confined to the isolated copied target package.
- The candidate may only address ordered activation and phase-gate compliance; no unrelated prose cleanup is allowed.
- Compare exact package paths and bytes; `agents/openai.yaml` must remain byte-identical.
- Runtime prose must be token-neutral or smaller unless a demonstrated missing capability cannot be expressed by replacement.
- Run whitespace/error checks on the candidate diff and verify required frontmatter keys remain exact.
- Before landing, reconfirm the live target hash and the frozen candidate hash through the compiled commands.
- After landing, verify the changed-file receipt and the `.agents/skills/grill-with-docs` mirror symlink. No Git commit is authorized by this workflow.
