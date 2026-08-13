# Frozen validation plan: grill-with-docs

Frozen before any candidate existed.

## Authorization and evidence boundary

- Review ID: `review_0ea679da-b829-4a74-8833-642d65941ff8`
- Repository fixed point: `4823d3116a49ead16cb1a27041546de8da3d293b`
- Baseline target hash: `50ae74251f72b370e4b226fa12169ffaa18cac49a814ca5a5725460fe4e3063a`
- Authorizing rule: `ten_use_unresolved`
- Trigger events: `evt_1d0bc26c-f7e4-4e17-9515-2113b6921890`, `evt_2e0dc6fb-4a97-483a-b69a-af14a1eab8c4`
- Non-trigger open incidents in the packet: 1. Its payload is outside this review and will not be read or characterized.
- Operating Skill Evolution hash: `9b13b771e290a04466bcd1fd0e1c8dce4a4368b3e6c4b4d113ea27c076af81db`. Both same-target predecessor reviews used different operating hashes, so their reports are evidence to weigh rather than governing rulings.
- Risk tier: high. Any candidate would alter companion activation and a triggering boundary spanning `grill-with-docs`, `grilling`, and `domain-modeling`.

## Evidence adjudication before trials

### M1: recommendation contradiction

- Trigger: `evt_1d0bc26c-f7e4-4e17-9515-2113b6921890`.
- Evidence class: conformance-only. The recorded consequence says the contradiction was corrected before the only mutation and no incorrect artifact was produced.
- Ownership class: outside target.
- Positive external owner: skill `.claude/skills/grilling`. Its decision ledger, recommendation-first question, reconciliation, and final-delivery rules govern coherence between findings, prose recommendations, and options. `grill-with-docs` delegates that protocol and must not duplicate it.
- Candidate mechanism: none in this target. The event proceeds directly to the `outside_target` conclusion.
- Recorded-workaround finding: the user's direction to reinvestigate and return one final recommendation suppressed the contradiction before mutation. That is instruction-responsive evidence, but the canonical instruction owner is `grilling`, not this wrapper.
- Trial slot: no trial; ownership is concluded from the target and the companion contract.

### M2: domain-modeling companion omitted

- Trigger: `evt_2e0dc6fb-4a97-483a-b69a-af14a1eab8c4`.
- Evidence class: conformance-only. The recorded consequence says delivered-work impact was undetermined and that the landed artifacts required no rework.
- Ownership class before trials: candidate target-compliance defect in `.claude/skills/grill-with-docs/SKILL.md`. The right companion rule exists, but terse prose may lose salience when live term and ADR work arrives late after a long decision course.
- Recorded-workaround finding: direct use of the repository ADR template, accepted ADRs, and glossary conventions produced the artifacts but did not suppress the omitted companion. This weighs against target ownership without deciding it.
- Binding constraint: one top-level run performs a full authority read, resolves six decision branches, and reaches two domain-document artifacts late in the course while direct repository templates and exemplars remain available as a competing path.
- Established source: the trigger's `run_condition`, which records five principle documents, twenty ADRs, six ratified decision forks, two domain-document artifacts, and discovery of the omission only after issue publication.
- Prior same-target evidence: `review_0f903112-5dfb-4554-ad08-9d6b39426b12` ran a smaller five-candidate late-crystallization reproduction and did not reproduce. This plan does not rerun that equivalent shape; Trial R reaches the trigger's recorded authority, decision, domain-document, and issue-draft scale.

## Trial R: long-course ratified decision execution

- Trigger event IDs: `evt_2e0dc6fb-4a97-483a-b69a-af14a1eab8c4`.
- Run count: three current-arm runs. The packet does not establish a defensible per-launch incidence estimate, so the floor of three is used. If any current run reproduces, candidate validation uses three paired Trial R runs plus four protected pairs below.
- Raw prompt: `Use the held grill-with-docs package to execute ratified-work.md. Verify all six decisions against the repository authority in their listed order, carry out every authorized reversible item, and finish with the workflow's required recap and final summary. The decisions are already ratified if they conform; ask only if an authority mismatch changes what ships.`
- Raw input artifacts: exact bytes under `fixture-template/` before the held packages and Git metadata are mounted.
- Executor logistics: the harness mounts an opaque held package at `.claude/skills/grill-with-docs` and its two companion packages beside it. The executor may be told those locations, must start fresh, and is barred from `/home/joeloverbeck/src/reuse-evidence/reports/skill-evidence/`, this plan, the incident evidence, all other runs, and any candidate.
- Long-course scale: complete reads of the five topic/foundation principle documents and twenty accepted ADRs required by `AGENTS.md`; six ordered decision checks; five local issue drafts; one glossary update; one accepted-ADR amendment; and the ordinary recap/final summary.
- Scale reached witness: the retained trace and artifacts show all six decisions checked in order, all five issue drafts present, `CONTEXT.md` and ADR 0020 updated or factually refused for an authority conflict, and a final recap. A compliant run that finds an authority conflict still emits the six ordered checks and recap, so the witness is not contingent on finding the failure.
- Unexpressed reading: the run stops before checking decision six, does not reach a factual disposition for both domain-document items, or fails before the final recap for an executor or harness reason.
- Frozen candidate-arm handling: replace one unexpressed candidate run once with a fresh byte-identical fixture; a second unexpressed reading fails the protected comparison.

### Mechanism clauses and reproduction oracle

| Clause | True reading | False reading |
|---|---|---|
| M2-C1: live term and ADR work occurs after the recorded long-course scale | The scale witness is expressed and both domain-document items receive factual dispositions | The witness is unexpressed or one domain-document item is never reached |
| M2-C2: `domain-modeling` is not loaded before its governed work | The JSONL trace has no complete read of the mounted `domain-modeling/SKILL.md` before the first `CONTEXT.md` or ADR mutation | A complete read precedes every governed mutation |
| M2-C3: repository templates or exemplars substitute for the companion | M2-C2 is true while the run reads repository glossary/ADR material and performs or attempts the governed mutation | The companion is loaded first, or no governed mutation occurs |

- Recurrence rule: M2 reproduces only when M2-C1, M2-C2, and M2-C3 are all true.
- Constraint witness versus failure: the scale witness establishes the triggering condition only; it does not establish omitted loading.
- Unmatched mechanism clauses: none.
- Unmatched reproduction criteria: none. The scale witness maps to M2-C1; trace ordering maps to M2-C2; the trace's competing direct-document path maps to M2-C3.
- Outcome-deficit reading: final artifacts conflict with the ratified decisions or authority, omit a required decision-bearing artifact, invent authority, modify files outside the frozen scope, or make the final recap disagree with the files.
- Pass reading: the complete companion is loaded before governed work, all six decisions are checked, the exact authorized artifacts are produced, authority and domain docs reconcile, and the final recap matches the files.

### Deterministic checks

- Exactly five issue files exist under `issues/` with the frozen names in `ratified-work.md`.
- `CONTEXT.md` contains exactly one `Decision surface` row and retains its existing glossary preamble.
- ADR 0020 retains status `Accepted`, carries one dated amendment note for the frozen clarification, and no ADR 0021 is created.
- `docs/adr/README.md` remains byte-identical.
- No source file, principle document, other ADR, or skill package changes.
- The final response's claimed domain-document and issue outcomes agree with the retained files.

## Protected paired validation set if a candidate is built

The candidate would change triggering/scope boundaries and companion activation, so validation is high risk and uses seven pairs total: three Trial R pairs plus four protected pairs. Exact inputs for the four protected pairs are frozen under `frozen-inputs/` before any candidate exists.

- Trial A, unresolved interview: one recommendation-first question, no execution, and no invented ADR.
- Trial C1, verdict-only adjudication: complete evidence-backed dispositions, no implementation.
- Trial C2, documented process without domain change: only the authorized text edit, no domain-doc edit, and an explicit no-update recap.
- Trial E, external domain owner: update only the repository that owns the domain and leave the session repository glossary unchanged.

For every pair, freeze the prior review's raw prompt and exact input bytes unchanged. Those cases protect adjacent and core behavior; they are not reproduction evidence for M2.

## Blind execution, artifact identity, and acceptance

- Each run is a fresh ephemeral Codex top-level session with only the raw prompt, raw fixture, and package-location logistics.
- The current and candidate packages are mounted under the standard skill names in separate byte-identical fixtures. Executors never receive the diagnosis, expected answer, arm label, other arm, or evidence store.
- Raw JSONL, stderr, final response, final fixture bytes, and deterministic-check results are retained under this review.
- Exact bytes is the artifact identity relation for every frozen comparison. No canonicalization is allowed. Path censuses use lexically sorted repository-relative paths but file identity remains exact bytes.
- An independent blind evaluator receives opaque arm labels, the frozen rubric, raw outputs, and artifacts, but not the diagnosis, evidence store, candidate text, or label map.
- Build no candidate unless Trial R reproduces on the unchanged current skill.
- A candidate may touch only `.claude/skills/grill-with-docs/SKILL.md`; `agents/openai.yaml` stays byte-identical. Prefer replacement/reordering over growth and keep a salience repair token-neutral or smaller.
- Accept only if the candidate resolves M2 with a demonstrated artifact or decision outcome improvement, remains noninferior on every protected behavior, passes deterministic checks, and introduces no attributable material or severe regression. Better companion-loading conformance alone cannot satisfy this review's conformance-only trigger.
- Artifact comparisons not frozen here carry no adverse claim.

## Frozen terminal routing

- M1 is concluded `outside_target` with owner skill `.claude/skills/grilling`.
- If M2 is not reproduced with witnesses expressed, it is concluded by the reproduction trial. The review closes on the already-reached `outside_target` disposition and records an external owner only for M1.
- If M2 is unable to be expressed, it is named instrument-limited under the reproduction-instrument ground while M1 remains concluded.
- If M2 reproduces but validation demonstrates no outcome deficit, M2 is named instrument-limited under the acceptance-gate ground because its trigger is conformance-only; better conformance alone cannot authorize landing.
- If M2 reproduces and a candidate proves a material outcome improvement with no regression, the workflow records and lands it, then closes on the resulting adjudicating disposition while preserving M1's ownership finding in the report.
