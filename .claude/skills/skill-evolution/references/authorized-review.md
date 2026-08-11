# Authorized review

You are here only because the preflight printed `authorized: true`. Work from its bounded evidence packet — the trigger events, use counts on the current hash, related prior dispositions, and the concrete artifacts they cite. Do not ingest the full historical ledger; the gate projection exists to keep old incident lore from dominating current judgment. The threshold authorized a diagnosis, not a presumption that the skill is defective or a guarantee of an edit.

`prior_reviews` indexes every review already completed on this target, whatever cluster it adjudicated, because what a predecessor ruled about the instrument or the mechanism is symptom-independent. Read the report of each one whose `same_target_hash` is true before step 4: those judged these exact target bytes. After step 1, compare each entry's `operating_skill_hash`, when present, with the current `review_started` event's `operating_skill_hash`. A matching identity means the predecessor used these review rules and its rulings govern how far this review can get. A differing identity means the predecessor used different review rules: its report is evidence to weigh rather than a ruling that governs. An entry with no recorded identity is unknown rather than equal; read exactly as it is read today, with its same-target rulings governing, because absence cannot retroactively establish that its rules differed. Also read any older entry whose note bears on this cluster — it landed a change, but a standing instruction, such as one against repeating a behavioral reproduction, survives the landing. Rediscovering a predecessor's ruling by running trials is waste; contradicting it silently is worse. `related_prior_dispositions` remains the narrower symptom-linked view for duplicate-mechanism judgment.

Compiled command family (all event writes, from the repository root): `cargo run --locked -p reuse-evidence -- skills evolution <command> --target <skill-path> …`. Every command takes explicit `--recorded-at`, `--now-epoch-milliseconds`, `--session-id`, and `--lock-owner` inputs. Mutating commands also take caller-owned `--event-id` and `--repository-head` inputs; `claim` takes a caller-owned `--review-id`. The command never reads the ambient clock or generates an identity. Review artifacts live under `reports/skill-evidence/<skill-key>/reviews/`.

### 1. Claim the review

```bash
cargo run --locked -p reuse-evidence -- skills evolution claim \
  --target <skill-path> --review-id <review-id> --risk-tier provisional \
  --record-operating-skill-hash \
  --event-id <event-id> --recorded-at <RFC3339-clock> \
  --now-epoch-milliseconds <clock-ms> --repository-head <repository-head> \
  --session-id <top-level-session-id> --lock-owner <caller-owned-lock-id>
```

The compiled command re-evaluates every authorization term under the store lock, appends `review_started` (trigger IDs, authorizing rule, baseline target hash, computed operating Skill Evolution package hash, provisional risk tier, fresh-session or cooldown proof), and re-derives the gate to `review_in_progress`. Read the current `operating_skill_hash` from that appended event only for the `prior_reviews` comparison above; do not ingest the historical ledger. If it refuses — another review owns the target, or the gate moved — relay the refusal and stop without semantic analysis.

*Done when the compiled command printed a `review_id` and the review owns the target.*

### 2. Verify threshold premises

Before thinking about repairs, check against the packet only:

- every trigger event represents a qualifying use;
- events claimed as independent are genuinely independent — distinct top-level sessions or materially different tasks, not retries, continuations, subagent reruns, or duplicate accounts of one event;
- at least one threshold event, including the threshold-crossing one, is contemporaneous;
- trigger hashes match the current target version;
- the candidate symptom cluster is factually plausible as a common symptom (causality is confirmed later, in step 3).

On failure, carry `insufficient_independence` or `superseded_by_target_version` and a note naming what failed to step 9; do no further review work.

*Done when every premise was confirmed, or the failed premise and terminal disposition are ready for step 9.*

### 3. Determine target ownership and causal mechanism

Now — and only now — read the target skill, plus the minimum external contracts needed to test ownership. Classify the evidence:

| Causal disposition | It proceeds? | Terminal outcome → close disposition |
|---|---|---|
| **Target defect** — misleading, contradictory, missing, or badly placed guidance causally connected to the incidents | yes | — |
| **Target compliance defect** — the right rule exists but its structure, placement, salience, or instruction competition repeatedly defeats compliance | yes | — |
| Outside target — another skill, contract, tool, environment, model limitation, or user instruction owns it | no | `outside_target` → `outside_target` |
| Task-specific novelty — does not generalize beyond the triggering task | no | `resolved_no_change` → `closed_no_skill_defect` |
| Not reproducible from the evidence — the packet supports no mechanism the target could own | no | `not_reproducible` → `monitor_for_recurrence` |

For every trigger concluded **Outside target**, name one positive external owner using this closed roster and a stable reference. `caller` and `session` are not owner kinds.

| External owner kind | Stable reference |
|---|---|
| `skill` | Repository-relative installed skill path |
| `contract` | Repository-relative contract path plus section or clause |
| `tool` | Tool name plus stable command or interface |
| `environment` | Named runtime, platform, or execution environment |
| `model_limitation` | Named model family and limitation |
| `user_instruction` | Stable task or instruction reference |

For a non-proceeding class, carry the mapped disposition and a factual note to step 9. Naming each outside-target owner in the close reports that attribution in the close receipt and user-facing completion for the maintainer to act on; it does not route evidence into the named owner's store. Do this without proposing an unsanctioned repair, and never edit another owner from this review.

Each mechanism stays a *candidate* here. Only step 5's current arm can confirm it, so do not treat an unconfirmed mechanism as absent and do not close `not_reproducible` merely because no trial has run yet. Name one candidate mechanism for each trigger event, or explicitly group several trigger events under one shared mechanism and state why they share it.

A candidate target or target-compliance defect whose ownership the packet cannot decide proceeds to step 4; candidate is not a conclusion. Do not turn uncertainty into `outside_target`. Freeze its binding constraint and ask whether the existing trial instrument can express it. If not, its trigger reaches step 9 as undecidable under the reproduction-instrument ground, alongside any sibling for which step 3 did reach a conclusion.

Classify what claim each trigger's evidence bears, not only its mechanism. Read `consequence` from each raw trigger event: one recording that no defect reached the delivered work, or recording the effect as undetermined, bears a **conformance-only** claim — the run did not do what the skill said, and nothing about the work it delivered. One asserting the delivered work was worse is **outcome-graded**. Step 7 grades outcome, so a conformance-only trigger has opened a gate its own evidence cannot satisfy. Record each trigger's class. A cluster holding both classes is ordinary, because a symptom key groups incidents without naming a cause; route it per trigger at step 9 rather than picking one class for the review.

Before freezing the plan, read `workaround_taken` only from the raw trigger events in the evidence packet. State what those recorded workarounds establish about the candidate mechanism and target ownership, or state that none was recorded on a trigger event. Repeated suppression of the mechanism is evidence for target ownership because the mechanism responds to instruction the target could carry; a workaround that was taken without suppressing the mechanism is evidence against target ownership. Record the direction as evidence, never as a verdict.

Using the packet's candidate cluster for this authorization, count the open incident IDs outside the trigger set and state that count, including zero. This count discloses how many incident payloads the bounded workaround read could not reach. Do not characterize, estimate, or reason about those incidents; do not read the historical ledger or seek their payloads.

*Done when each trigger is mapped to a candidate mechanism, an ownership class, and an evidence class, and a non-proceeding class has its terminal disposition and note ready for step 9.*

### 4. Freeze the validation plan before any candidate exists

For each distinct mechanism, name its binding constraint — the condition without which its failure does not occur, such as load, context distance, input size, or elapsed run length — and decide whether the trial instrument can vary it. A trial executor starts fresh and short-context, so a constraint that exists only deep inside a long run is inexpressible. Mark that mechanism as unable to be expressed in the frozen plan; freeze no run or candidate for it. When only some mechanisms are untestable, mark each as unable to be expressed in the plan and proceed with the runnable reproduction trials. If none is testable and the review has concluded about no covered trigger, carry `blocked_no_valid_test` and a note naming the constraints to step 9 without freezing a plan or building a candidate. If a sibling already has a conclusion, keep that adjudicating disposition and route each untestable candidate trigger undecidable at step 9. `blocked_no_valid_test` remains a whole-review disposition; do not assign it to an individual mechanism.

For each testable constraint, name its **witness**: the observable in a finished run's output or artifacts that shows whether the run expressed the constraint, plus the observation that reads unexpressed. An observable that cannot read unexpressed is not a witness; mark its mechanism unable to be expressed. Anything already produced by the run can qualify; require no new instrument. Fix every reading before outcomes exist, because one chosen after a result is not evidence.

A same-hash predecessor that already ran trials constrains this judgment: if it reports the current arm passing on a mechanism shape these incidents repeat, that is evidence the wording is followed when freshly read, and the constraint is the run condition rather than the text. Do not re-derive it by rerunning equivalent trials.

The incidents' recorded run conditions are the evidence for this judgment, not a guess about them. When they agree that the failures arrived at volume, late in a long run, or only intermittently, say so here and treat a fresh short-context single-run trial as unable to express that.

Reconcile the frozen plan with step 3's workaround finding. Keep the plan consistent with that directional evidence without letting it replace a trial or skip or shrink the frozen trial set; blind comparative validation remains the gate.

Freeze one reproduction trial per distinct mechanism, each with its own trigger event IDs, witness, unexpressed reading, and incidence-sized run count. For an untestable mechanism, record its constraint and unable-to-be-expressed reading in that trial slot instead of inventing a run. Each runnable reproduction gets at least 3 runs when its first witness reads expressed. If recorded `run_condition` and `workaround_taken` support a per-launch incidence estimate, freeze it, choose a count above the floor, and explain the sizing; otherwise state that and use the floor. An unexpressed first reading stops that trial before its remaining runs. Before any candidate output exists, freeze whether a candidate-arm run whose witness reads unexpressed is discounted from the comparison or replaced.

Only then define the trials, so the change cannot pick only tests it already knows how to pass. Ordinary, narrow change — at least three paired trials:

1. the fresh reproduction trial for each distinct mechanism;
2. an adjacent case exercising the same capability differently;
3. an unrelated core regression case.

Escalate to at least five paired trials (add another core-regression case and a fragile, edge, or safety-relevant case) when the change affects destructive or external actions, state integrity or confidentiality, shared conventions or multiple skills, triggering or scope boundaries, a broad workflow section, more than one major behavior, or substantial deletion or reorganization.

Freeze per trial: the raw prompt/task, raw input artifacts, pass/fail or comparison rubric, witness, deterministic checks, protected behavior, and evaluator-independence requirements. Save the plan under `reviews/<review-id>/`.

*Done when every mechanism has a binding constraint and trial slot, every runnable trial has its readings fixed, and the full set is frozen, or all mechanisms are blocked and ready for step 9.*

### 5. Construct an isolated candidate

The current arm is the union of the reproduction trials; if their results disagree, proceed only on each mechanism that reproduced and report every mechanism as reproduced, not reproduced with witnesses expressed, or unable to be expressed.

Run each runnable reproduction on the unchanged current skill before building anything, under step 6's rules. Read its **first** witness before spending another. Unexpressed → mark that mechanism unable to be expressed and stop only its trial, with no re-cut prompt or fixture. Expressed → finish its planned current-arm runs.

Classify each mechanism separately, including run 1: failure recurred → reproduced; witnesses expressed on every run without failure → not reproduced with witnesses expressed; any unexpressed witness → unable to be expressed. Build the candidate only for reproduced mechanisms and carry all three mappings into the report. This per-mechanism routing does not create per-trigger dispositions; the review still closes once, and step 9 carries each mechanism's reading into that one close.

If none reproduced, build no candidate. Choose `blocked_no_valid_test` only when the review has concluded about no covered trigger and no trial could express any mechanism. Otherwise keep the disposition for a conclusion already reached, or carry `monitor_for_recurrence`, and route every unable-to-be-expressed mechanism's triggers undecidable at step 9 — a disposition whose reach is the authorization reason's whole cluster must not stand in for a mixed review or one that did express something. Name every mechanism's reading in the note, then go to step 9.

Copy the live target to `reviews/<review-id>/candidate/` (outside skill discovery) and modify only that copy; the live target stays untouched until every trial passes. Design rules:

- solve the demonstrated mechanism, not every imperfection seen while reading;
- do not fix unrelated defects noticed during the review — they become evidence only if a real skill use records them;
- prefer deletion, consolidation, reordering, or clearer replacement over appending; keep ambiguity/salience repairs token-neutral or smaller;
- no incident narratives, audit provenance, dates, commit hashes, or field stories in runtime instructions;
- growth only for a proven missing capability that cannot be expressed by replacing existing text;
- tool- or repository-specific details go in conditional references, not universal runtime rules; shared guidance keeps one canonical home.

*Done when every mechanism is classified and at least one reproduced mechanism has a narrowly scoped candidate, or the arm-supported disposition and per-mechanism note are ready for step 9.*

### 6. Run blind comparative validation

Run every frozen trial against both the unchanged current skill and the candidate, using fresh sessions or independent agents with minimal task-local context. Give executors the raw task and artifacts — never the diagnosis, repair, expected answer, or version label; conceal or randomize labels for evaluators. Bar every executor from the evidence store, which holds the incident bodies, diagnosis, and candidate bytes. For a candidate-arm witness that reads unexpressed, apply the plan's frozen discount-or-replace choice before comparison; never improvise from the visible split. Run applicable deterministic checks on both versions where comparison matters, and on the candidate before landing. Retain raw outputs and evaluator decisions under `reviews/<review-id>/`.

*Done when every frozen trial ran on both versions and the raw outputs are on disk.*

### 7. Apply the acceptance gate

The candidate passes only when it resolves the implicated mechanism on the reproduction case(s); is noninferior on every protected core behavior; introduces no material or severe regression; passes all affected deterministic checks; preserves safety, scope, and ownership invariants; any growth is necessary, minimal, and supported by better outcomes; and it is materially better on the target mechanism rather than merely worded differently. Behaviorally tied: prefer the candidate only when it is meaningfully smaller or clearer; otherwise the current skill stays. This gate grades outcome, so it decides nothing about a conformance-only trigger whose mechanism it graded without demonstrating an outcome deficit; that trigger routes to step 9 rather than sharing whatever verdict the gate reaches. A trigger this gate never graded is untouched by that bar — step 9 says where each one lands.

On failure, leave the target untouched: `record-validation --decision rejected …`, then carry `candidate_rejected_validation` to step 9. A rejected candidate is not a license to improvise another in the same review — new evidence must reopen eligibility. Sole exception: a mechanical candidate defect discovered before any behavioral trial may be corrected once, then the complete frozen suite reruns.

*Done when the acceptance decision is made from the trial results alone.*

### 8. Record, land, verify

```bash
cargo run --locked -p reuse-evidence -- skills evolution record-validation --target <skill-path> \
  --review-id <id> --decision accepted --risk-tier <ordinary|high> \
  --candidate reports/skill-evidence/<skill-key>/reviews/<review-id>/candidate \
  --trials <count> --artifacts reports/skill-evidence/<skill-key>/reviews/<review-id> \
  --event-id <event-id> --recorded-at <RFC3339-clock> \
  --now-epoch-milliseconds <clock-ms> --repository-head <repository-head> \
  --session-id <top-level-session-id> --lock-owner <caller-owned-lock-id> [--summary "…"]
cargo run --locked -p reuse-evidence -- skills evolution land --target <skill-path> \
  --review-id <id> --candidate <same candidate path> \
  --event-id <event-id> --recorded-at <RFC3339-clock> \
  --now-epoch-milliseconds <clock-ms> --repository-head <repository-head> \
  --session-id <top-level-session-id> --lock-owner <caller-owned-lock-id>
```

`record-validation` freezes the validated candidate hash; `land` reconfirms everything before touching the live target — live hash still equals the claim baseline, candidate bytes exactly those validated, review still owns the target — then backs up the baseline, replaces the live bytes, verifies the landed hash and the `.agents` mirror symlink, and appends `change_landed`. If it refuses because the target moved, authorization expired: carry `superseded_by_target_version` to step 9 — never merge a validated candidate into an independently changed target by intuition. If landing verification fails, it restores the baseline itself; record the failure. Do not commit to Git automatically; normal repository recoverability stays as it is.

*Done when `land` printed the before/after hashes and changed-file list, or the no-landing disposition is ready for step 9.*

### 9. Report, close, amend, complete

Every close records the trigger events the review covered in `adjudicated_event_ids`; for an adjudicating disposition, add `--adjudicate <event-id>` only for additional events the review genuinely covered. The adjudicating dispositions are `resolved_by_change`, `closed_no_skill_defect`, `outside_target`, `insufficient_independence`, `monitor_for_recurrence`, and `candidate_rejected_validation`. Their covered event IDs retire from the active set, except any the close names as untestable below.

For every event in the coverage list, add exactly one `--concluded <event-id>` or `--instrument-limited <event-id>`. The first records that this review's disposition concluded about that trigger; the second records that this review could not decide it under one of the two grounds below. Missing, duplicate, conflicting, unknown, and out-of-coverage routes refuse before any write. This partition is a write-time request contract only: it adds no routing key to the recorded event, whose concluded set remains the coverage list minus `instrument_limited_event_ids`, so historical closes retain their existing meaning.

For every concluded `outside_target` event, add `--external-owner <event-id> <kind> <stable-reference>` using step 3's closed owner roster. Supply exactly one owner per concluded event. Do not supply an owner for undecidable coverage or for another disposition.

One disposition covers the whole list, and step 5 routinely reads different results for different mechanisms in it. Naming has two grounds, and they are different limits.

**No trial could express the mechanism.** For every covered trigger whose mechanism read **unable to be expressed**, add `--instrument-limited <event-id>`. Here the limit is the reproduction trial, and naming records that.

**The acceptance gate cannot decide it.** A trigger step 3 classified conformance-only, whose mechanism step 7 graded, is adjudicated only when those trials demonstrated an outcome deficit for it; absent that demonstration, name it here as well. Here the limit is the acceptance gate rather than the reproduction trial: step 7 grades outcome, so it cannot decide a trigger whose evidence bears no outcome claim, however cleanly the mechanism reproduced, and naming records nothing about whether it reproduced — the per-mechanism readings stay in the report, where the trials put them.

This ground is per trigger, not per review, and the distinction is the whole point: a sibling reaching the acceptance gate says nothing about a trigger the gate never graded. A trigger whose own mechanism read **not reproduced with witnesses expressed** was decided by its reproduction trial, never by the outcome gate, so it adjudicates normally. An ownership, independence, novelty, or not-reproducible conclusion from step 3 adjudicates normally too. A candidate target or target-compliance defect whose ownership the packet cannot decide proceeds to step 4 and, when its binding constraint cannot be expressed, takes the first ground above rather than borrowing a sibling's conclusion.

A named event keeps its place in the coverage list, stays open in the ledger and stops clustering — the claim `blocked_no_valid_test` makes for a whole review, made one trigger at a time, so an event this review could not decide can never again reach a threshold. One carve-out survives from that whole-review claim. Naming a contemporaneous severe incident stops it being adjudicated without retiring it, because it authorizes on its own and still drives the gate. Expect it to re-authorize every session until a later review adjudicates it; naming one is a decision to keep it open, never a way to quiet it. Everything left in the list retires as adjudicated, which is what the disposition asserts about it. Both grounds must already be on record before the close — the frozen plan's unable-to-be-expressed reading, or step 3's evidence class; a reading invented at close is neither a trial result nor a classification. The flag narrows what the close concluded and never widens what it covered, so an event outside the coverage list is refused. Naming the entire list is not refused: a review can decide none of what it covered while having concluded a great deal, which is exactly what a wholly conformance-only cluster produces once a candidate has been built and graded. Close on the disposition that review actually reached and name every covered trigger. Choose `blocked_no_valid_test` only when the review has concluded about no covered trigger and no trial could express any mechanism — it reaches its authorization reason's whole cluster rather than the coverage list, so a sibling conclusion must stay on its adjudicating disposition and route only the undecidable trigger through `--instrument-limited`.

`blocked_no_valid_test` and `superseded_by_target_version` are non-adjudicating dispositions: their payload still records the trigger coverage, but those IDs do not retire from the active set because the review reached no conclusion. Do not pass `--adjudicate`, `--concluded`, `--instrument-limited`, or `--external-owner` with either disposition; the compiled command refuses those combinations, because neither disposition concluded about anything it covered and neither takes the adjudicating partition. Trigger events stay in `events.jsonl` forever, and the gate projection combines the disposition with the coverage list to decide retirement.

`blocked_no_valid_test` is further **instrument-limited**: closing it asserts that this instrument cannot test the evidence it covered, so the projection retires that evidence from the *evolution gate* while leaving it open and unadjudicated in the ledger. Those incidents stop clustering and can no longer reach a threshold, and the projection names them in `instrument_limited_incident_ids` — which the preflight evidence packet carries too, so the packet's open-incident count and its clusters still reconcile.

Gate retirement — what `retired_from_gate_event_ids` reports, never the adjudication retirement above — reaches exactly the events an event-level `--instrument-limited` naming lists. A disposition-level `blocked_no_valid_test` instead reaches the open incidents the claim's authorization reason would name if re-evaluated at the close, rather than every same-symptom sibling: `friction_recurrence` names the whole open symptom cluster; `material_recurrence` names only material-or-worse incidents in its symptom cluster; `ten_use_unresolved` names the open incidents in its anchor cluster except retrospective ones; and `severe` names only its triggering incident, which is itself never retired, so the reach is empty. The coverage list freezes when the review is claimed, so it includes every incident the authorization reason names at that point. Incidents recorded after the claim and before the close remain outside coverage and inside the reason-specific reach; the close still succeeds, and any event the reviewer cannot vouch for remains a required mismatch in the report and user-facing completion. Incidents recorded after the close are new evidence and drive the gate normally. A contemporaneous severe incident is never retired under any reason because it authorizes on its own. A retrospective severe incident retires when the authorization reason names it. If a historical `review_started` has no recognized `authorizing_rule`, derivation uses the prior symptom-wide reach rather than narrowing on a guess. `superseded_by_target_version` retires nothing; the target moved, and nothing was established about testability.

Before a `blocked_no_valid_test` close, read the authorization reason and coverage list from the claim receipt. Re-evaluate that authorization reason against the live candidate clusters in the gate projection, using the roster above to identify the widest reason-specific reach bound. Vouch that the binding constraint you named plausibly covers every incident in that bound, not every same-symptom sibling. Vouch only for incidents inside that reason-specific reach bound; same-symptom incidents outside it do not affect this close. The projection is current as of the last recorded incident, so no extra derive run is required. This reason-specific reach bound can name incidents the close will not retire: a contemporaneous severe incident is never retired because it authorizes on its own. If you cannot vouch for an incident in that reason-specific reach bound, record the mismatch in the review report and user-facing completion.

When any mechanism was marked **unable to be expressed**, record a dead-end note for each one in the review report: the mechanism, its binding constraint, and that this workflow has no further instrument for it. The decision to pursue it belongs to the maintainer, not a later review. This records the limit and schedules nothing.

Use the same-target `prior_reviews` reports read before step 4 to check whether an earlier review ruled a mechanism of the same shape unable to be expressed on these target bytes. When it did, say so in that mechanism's dead-end note; reaching this exit twice on one target is the signal the note carries.

The dead-end section in the template below is conditional. When no mechanism has that reading, omit the `## Unable to be expressed` section and add no completion clause; the report and completion otherwise stay unchanged.

Before any close, write the review report at `reviews/<review-id>.md`, with unreached sections marked `not reached — <disposition>`. Fill every section the review has reached and put `pending close receipt` only where the receipt supplies the final value. The compiled close command verifies that this report file exists before it appends anything; it does not judge the report's prose.

```markdown
# Skill Evolution Review: <skill-name>

## Authorization
- Gate rule:
- Trigger event IDs:
- Target before hash:
- Fresh-session/cooldown proof:

## Evidence adjudication
- Independence result:
- Confirmed mechanism:
- Trigger event → evidence class:
- Trigger event → reproduction trial → witness reading:
- Trigger event → ownership class → owning source → discriminating evidence:
- Trigger event → binding constraint → terminal route:
- Undecidable ground: reproduction instrument/acceptance gate/not applicable
- Recorded-workaround finding:
- Non-trigger open incident count:

## Candidate
- Change hypothesis:
- Files changed in isolated candidate:
- Runtime size before/after:

## Frozen validation plan
- Risk tier:
- Paired trials:
- Deterministic checks:

## Results
- Current version:
- Binding condition reproduced by the current arm: yes/no/not reached
- Trigger event → reproduction trial → witness reading:
- Candidate version:
- Regressions:
- Decision:

## Unable to be expressed
- Mechanism:
- Binding constraint:
- Earlier same-shape ruling on these target bytes:
- Further instrument in this workflow: none

## Landing
- Landed: yes/no
- Target after hash or unchanged hash:
- Final disposition:
- Coverage named untestable:
- Retirement reach event IDs:
```

Then close with the disposition carried here from the terminal branch:

```bash
cargo run --locked -p reuse-evidence -- skills evolution close \
  --target <skill-path> --review-id <id> --disposition <disposition> \
  --note "<mechanism and result>" --event-id <event-id> \
  --recorded-at <RFC3339-clock> --now-epoch-milliseconds <clock-ms> \
  --repository-head <repository-head> --session-id <top-level-session-id> \
  --lock-owner <caller-owned-lock-id>
```

When validation ran without building a candidate or recording `validation_completed`, add `--trials <count> --artifacts reports/skill-evidence/<skill-key>/reviews/<review-id>`. These are attributed assertions: the close records them for later reviewers and never uses them for eligibility, retirement, reach, or any other authorization. Omit them when no validation ran; absent means not recorded, never zero. A close after `record-validation` need not repeat the effort already carried by `validation_completed`.

After the close succeeds, amend the review report with the final disposition and every close-receipt value that was pending. Read `retired_from_gate_event_ids` from the close receipt whenever it is present: it names what this close moved out of the gate, including an empty reach, and never the adjudication retirement that `adjudicated_event_ids` carries. The projection's `instrument_limited_incident_ids` is instead the standing per-hash set and can include retirements from earlier reviews. Copy the receipt's list into the report and state that retirement reach in the user-facing completion. A close that neither carried an instrument-limited disposition nor named untestable coverage omits the key and has no retirement reach to report.

Read `external_owners` from the close receipt whenever it is present: it names exactly the external owners this close recorded, each by kind and stable reference.

The user-facing completion is concise, links the report, states whether the live skill changed, and, when the close receipt carries `retired_from_gate_event_ids`, states that retirement reach exactly, including an empty list. When the close receipt carries `external_owners`, it states each attributed owner kind and reference. When that key is absent, add no owner clause and no empty-owner placeholder. When any mechanism was marked unable to be expressed, add one clause stating that fact alongside the retirement reach.

*Done when the report existed before close, the disposition event exists, the report carries the close receipt's final values, and the completion was delivered.*

## No same-review expansion

Unrelated imperfections noticed during an authorized review are not in scope: do not fix them, do not broaden the candidate, and do not manufacture an incident merely from reading the skill. If an unrelated defect directly causes a frozen-trial failure, record it as a trial result and leave it for a later evidence cycle unless it makes the current candidate unsafe. This is what keeps a narrow authorized review from turning back into a general audit.

## Terminal outcomes

Every invocation ends in exactly one state. `refused_closed_gate`, `refused_cooldown_or_same_session`, and `refused_self_target` end in `SKILL.md` step 2 with no event and no report. Claimed reviews end as `superseded_by_target_version`, `insufficient_independence`, `outside_target`, `not_reproducible`, `blocked_no_valid_test`, `candidate_rejected_validation`, `resolved_no_change`, or — the only outcome that modifies the live target — `resolved_by_validated_change` (disposition `resolved_by_change`). Name the terminal outcome in the completion.
