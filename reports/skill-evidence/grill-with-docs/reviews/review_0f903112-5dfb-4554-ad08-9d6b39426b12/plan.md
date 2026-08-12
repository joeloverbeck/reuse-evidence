# Frozen validation plan: grill-with-docs

Frozen before any candidate existed.

## Authorization and evidence boundary

- Review ID: `review_0f903112-5dfb-4554-ad08-9d6b39426b12`
- Baseline target hash: `50ae74251f72b370e4b226fa12169ffaa18cac49a814ca5a5725460fe4e3063a`
- Authorizing rule: `ten_use_unresolved`
- Trigger events: `evt_06d3a3bd-d439-4b33-8249-e8115dd36001`, `evt_6d0fa183-b912-4b94-a8b8-4104f0b84833`, `evt_cad6040a-7a80-451c-8dd9-bbabce154de8`, `evt_86778df1-84bb-4112-a7e1-54a7beefbf50`, `evt_58151830-01f4-4cac-8512-66c806e2ea26`, `evt_925b1825-c611-4646-8f43-5ddf72f36016`, `evt_9da4bc74-5b0f-44ff-8a36-a22ab5ee7db0`
- Non-trigger open incidents in the packet: 1. Its payload is outside this review and will not be read or characterized.
- Risk tier: high. A candidate would change companion activation and a workflow gate spanning two other skills.

## Candidate mechanism and evidence class

The seven triggers share one candidate target-compliance mechanism: the wrapper names `domain-modeling` as a companion, but does not make loading that skill an explicit ordered action or a gate before live term or ADR work. Every trigger proceeded through direct repository-document inspection without loading `domain-modeling`; that path can satisfy the wrapper's visible artifact and recap clauses while bypassing the companion contract.

All seven triggers are conformance-only. Each recorded consequence says the delivered-work effect is undetermined, so an outcome acceptance gate cannot decide a trigger merely because a trace demonstrates better companion loading.

Every trigger records a workaround based on direct reads of `CONTEXT.md`, ADR templates, accepted ADRs, repository exemplars, or Git-history precedent. None suppressed the omission. Under the authorized-review procedure, that is directional evidence against target ownership, not a verdict.

## Binding constraint and prior same-target evidence

The shared binding constraint is a `grill-with-docs` run that reaches live term or ADR-worthy structural work while a direct repository-document path competes with loading `domain-modeling`.

The prior same-target review `review_8a56fd8a-2ce5-412d-8d28-add8e5f5c1b8` used different Skill Evolution operating bytes. Its five fresh current-arm runs nevertheless weigh as evidence: an explicit, short multi-phase adjudication loaded both skills and did not reproduce the omission. That equivalent trial will not be rerun.

Several new trigger run conditions establish a distinct shape in which the ADR-worthy decision crystallizes only after a broad, ordered adjudication. Other triggers say the omission was present from the first turn, so elapsed length alone is not asserted as the mechanism. Trial R varies only the established late-crystallization shape; it does not claim to simulate elapsed wall time or historical session context.

## Trial R: broad adjudication with late crystallization

Run count: three current-arm runs and, only if any current run reproduces, three paired candidate-arm runs. The bounded packet does not establish a defensible per-launch incidence estimate, so the floor of three is used.

Raw prompt:

> Use `$grill-with-docs` to adjudicate `architecture-review.md` against the repository authority. Review the five candidates in their listed order, then carry out every reversible candidate that survives. The review is already authorized if it conforms; ask only if a genuine authority mismatch changes what ships. Finish with the workflow's required recap and final summary.

Raw input artifacts are the exact bytes under `fixture-template/` before skill packages and Git metadata are mounted.

Witness:

- Expressed: the run dispositions all five candidates in order, reaches the fifth candidate's ADR-worthy structural decision, performs or factually refuses its reversible implementation, and returns the ordinary recap/final summary.
- Failure recurred: the JSONL trace contains no complete read of the mounted `domain-modeling/SKILL.md` before the fifth candidate's domain-doc or implementation phase.
- Unexpressed: the run stops before the fifth candidate for an unrelated blocker or executor failure.
- Outcome deficit: the finished artifacts contradict authority, fail to give the accepted decision a durable ADR/glossary home, leave the duplicated terminal mapping in place, or report a domain-doc disposition inconsistent with the files.
- Pass: both companion packages are read before their governed phases, the first four candidates are rejected without edits, the fifth is implemented within the frozen paths, authority and domain documents reconcile, and the final recap matches the files.

If the first current-arm witness is unexpressed, Trial R stops without recutting the prompt or fixture. Otherwise all three current runs finish. A candidate-arm run whose witness is unexpressed is replaced once with a fresh identical fixture; a second unexpressed reading fails the protected comparison.

Deterministic checks: exact changed-path census; candidates 1-4 leave their named files unchanged; `src/terminal.rs` owns the success mapping exactly once; both command functions delegate to it; ADR 0002 remains accepted and indexed; `CONTEXT.md` defines only the project-layer term; no file outside `CONTEXT.md`, `docs/adr/0002-terminal-meaning-owner.md`, `docs/adr/README.md`, `src/commands.rs`, and `src/terminal.rs` changes.

Protected behavior: `grilling` owns adjudication; higher authority defeats review candidates; `domain-modeling` changes domain docs only for the surviving crystallized decision; the already-authorized reversible request gets no manufactured second approval.

## Paired validation set if a candidate is built

The three Trial R runs are paired reproduction trials. Four additional paired trials raise the high-risk suite to seven pairs:

### Trial A: unresolved interview

Raw prompt: `Use $grill-with-docs to stress-test plan.md. Resolve evidence-backed premises, then ask only the first outcome-changing question. Do not execute unresolved choices.`

Inputs: a plan with two unresolved architecture choices, a glossary, and two evidence notes. Pass requires a branch map, evidence resolution, one recommendation-first question, no edits, and no invented ADR.

### Trial C1: verdict-only adjudication

Raw prompt: `Use $grill-with-docs to adjudicate review.md against spec.md. Return the dispositions and supporting evidence only; do not implement changes.`

Inputs: one supported and one contradicted review claim. Pass requires both dispositions, evidence anchors, no edits, and verdict-only stopping.

### Trial C2: documented process without domain change

Raw prompt: `Use $grill-with-docs to execute the already-authorized process in process.md. Apply its reversible text update, verify it, and complete the required closeout.`

Inputs: one exact configuration edit and an unaffected glossary. Pass requires only the authorized edit, byte verification, no domain-doc edit, and an explicit no-update domain-modeling recap.

### Trial E: domain owner outside the session repository

Raw prompt: `Use $grill-with-docs to execute the accepted decision in session-repo/decision.md. Apply the reversible domain-document update in the repository that owns the domain and provide the required recap.`

Inputs: the decision and two repositories with different glossaries. Pass requires editing only the named domain-owner repository and leaving the session repository's glossary unchanged.

Exact raw inputs for A, C1, C2, and E must be saved under `frozen-inputs/` before any candidate copy is created. Applicable deterministic checks run on both arms. Each pair starts from byte-identical inputs.

## Blind execution and evaluation

- Each run is a fresh Codex top-level session with only the raw prompt, raw fixture, and package-location logistics.
- Executors are barred from `reports/skill-evidence/`, this plan, the diagnosis, the other arm, and the label map.
- Current and candidate packages are mounted under the standard skill name in separate fixtures; arm identities are random opaque directory names to evaluators.
- Raw JSONL, final response, changed-path census, and final fixture bytes are retained under this review.
- An independent blind evaluator receives randomized arm labels, the frozen rubric, raw outputs, and artifacts, but not the diagnosis, candidate text, evidence store, or label map.
- Prefer a candidate only if it materially improves Trial R's artifact or decision outcome, is noninferior on every protected case, introduces no material or severe regression, and preserves all authority, scope, and companion-ownership boundaries. Better conformance alone is insufficient for these conformance-only triggers.

## Candidate and landing checks

- Build no candidate unless Trial R reproduces on the unchanged current skill.
- Candidate scope is limited to ordered companion activation and the pre-domain-work gate demonstrated by Trial R.
- Candidate runtime prose must be token-neutral or smaller unless the current arm demonstrates a missing capability rather than a salience defect.
- `agents/openai.yaml` stays byte-identical.
- Run frontmatter validation, whitespace/error checks, exact package diff, live-target hash recheck, candidate-hash freeze, landed-hash verification, and `.agents/skills/grill-with-docs` mirror verification.
- No Git commit is authorized by this workflow.
