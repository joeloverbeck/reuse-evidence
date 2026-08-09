---
name: skill-evidence-capture
description: Record one factual receipt after a completed use of another skill — clean or not — without diagnosing or changing it.
argument-hint: "<path of the exercised skill> [task label; retrospective + evidence refs]"
disable-model-invocation: true
---

# Skill Evidence Capture

Record one cheap, factual receipt after another skill completes a material use, building a trustworthy denominator of clean and problematic uses. This skill is a recorder, not a reviewer: capture is not diagnosis, no repair starts here, and safe refusal — recording nothing — is success. Invocation is manual by design; never wire hooks, per-skill capture calls, or automatic invocation.

Arguments: the path of the skill that was exercised (required). Optional: a short task label when the session held more than one skill use; `retrospective` plus concrete evidence references (artifact, diff, log, transcript) when recording a use from an earlier session.

## Hard boundaries

- Identity only: never modify the target, semantically inspect it for defects, or compare it against repository conventions; never read target prose beyond what this session already loaded.
- No diagnosis: never name a root cause, propose a repair or new rule, point at an edit site, or claim two incidents share a cause.
- No escalation: never invoke Skill Evolution, encourage an early review, or run repo-wide audits, Git-history investigations, or empirical-claim sweeps.
- Evidence is immutable: never hand-edit `events.jsonl` or `gate-status.json`, and never delete, rewrite, or mark evidence consumed. All writes go through the compiled command.
- Rust-only production path: invoke only the compiled `reuse-evidence skills evidence` commands for record, derive, and hash behavior.
- No receipt without a qualifying use; the user's curiosity, anxiety, or wish to improve a skill is not an incident.
- Cost ceiling: one compiled command plus a terse status reply. If the command cannot complete safely, record nothing and report the failure.

## Workflow

### 1. Apply the qualifying-use gate

A use qualifies only when all hold: the target was exercised as the operating contract, not merely loaded, mentioned, or consulted for one isolated fact; at least one of its instructions materially governed an action, decision, repository change, or user-facing deliverable; the run reached a terminal outcome (terminal failure included); and it was not an abort before substantive work or a premature gate refusal. A retry or continuation of the same failed task is part of the same use, not another one. Substantive authorized Skill Evolution runs qualify like any other skill use.

If the use does not qualify: write nothing, do not analyze why capture was invoked, reply exactly `No receipt recorded: preceding use was not qualifying.` and stop. If several uses coexist and the supplied task label cannot disambiguate which one is meant, ask for the minimum disambiguation instead of guessing.

*Done when the use is confirmed qualifying and unambiguous, or the fixed no-receipt line has been returned.*

### 2. Establish evidence status

A use from this top-level session is contemporaneous. Anything else is retrospective and needs at least one concrete recoverable evidence reference; memory alone is inadmissible — write nothing and say so.

*Done when the record is marked contemporaneous, or retrospective with references, or refused as inadmissible.*

### 3. Classify the observable outcome

Classify from the completed session's evidence only — never open the target to decide what it "should" have said. Exactly one outcome:

- `clean` — the skill materially governed the run; no skill-attributable friction or failure visible.
- `friction` — correct outcome, but nontrivial uncertainty, workaround, repeated interpretation, contradictory directions, avoidable reruns, or unexpected procedural cost.
- `material_failure` — material rework required, an expected output contract violated, a core behavior skipped, or wrong work caught before consequential reliance.
- `severe_incident` — corruption, destructive or irreversible state change, unsafe external action, confidentiality exposure, or materially wrong finalized work already published, relied upon, or handed off. If containment is needed, state it separately; containment never becomes a skill rewrite.

For a non-clean outcome, fix the facts — expected, observed, consequence, any workaround actually taken, the run condition, factual and compact — and exactly one coarse symptom key, a clustering aid, not a cause: `triggering` (activated or failed to activate in an observably wrong context), `execution` (process stalled, skipped, contradicted itself, or needed a workaround), `output` (artifact violated an observable contract), `state` (incorrect, unsafe, or unexpected state change), `tool-compatibility` (a prescribed command or tool did not behave as represented), `coordination` (scope, ownership, handoff, or concurrency failed observably), `cost` (substantial avoidable time, context, or procedural burden), `unknown` (does not fit without diagnosis). Do not mint specific keys to force clustering. When adjacent severities are both arguable, pick the lower and keep the concrete consequence.

The consequence records what the deviation actually cost. When the omitted or violated step was itself the check that would have revealed that cost, say so: record the effect as undetermined and name the check that did not run, rather than reporting that nothing followed. A run cannot observe what it removed the means of observing, and "no harm" asserted on a check that never ran is that check's conclusion, not an observation. The lower-severity tiebreak above does not reach this case — an undetermined consequence is not an argument for grading down.

The run condition is one compact sentence of observable session fact: the volume the use handled, how long it ran, how far from the start the problem surfaced, and which step or gate caught it. Record what the session shows and say plainly when a part is not determinable. This is still description, not diagnosis — it states the circumstances of the use, never why the skill behaved as it did. A later review cannot judge whether any fresh trial could reproduce the incident without it, and the reviewer is always a different session, so nothing about the condition survives unless it is written here.

*Done when one outcome — and, for an incident, one symptom key plus the expected/observed/consequence and run-condition facts — is fixed.*

### 4. Record through the compiled command and relay its reply

From the repository root, run exactly one:

```bash
cargo run --locked -p reuse-evidence -- skills evidence record \
  --target <skill-path> --outcome <clean|friction|material_failure|severe_incident> \
  --task-label "<short factual label>" \
  [--symptom-key <key> --expected "…" --observed "…" --consequence "…" --run-condition "…" [--workaround "…"]] \
  [--retrospective --evidence-ref <ref> …] --human
```

One run is the whole operation: the compiled Rust command hashes the target, validates and atomically appends the event under `reports/skill-evidence/<skill-key>/`, re-derives gate state, and prints the terminal reply. Relay that reply verbatim and add nothing — no fixes, no review encouragement, no gate reinterpretation. The command rejects a duplicate run group on an unchanged target: within one top-level session, reuse the same task label when re-recording would mean a retry of the same task, and give genuinely distinct uses distinct labels. Across top-level sessions, pass `--same-run-group <prior-group>` only to declare that the receipt continues an earlier run group. If the command refuses or fails, nothing was written — report that outcome as final.

*Done when exactly one terminal state holds: the event was appended and the command's reply relayed verbatim; the use was not qualifying and nothing was written; retrospective evidence was inadmissible and nothing was written; or the command failed safely, nothing was written, and the failure was reported.*

## Self-receipts

Never record clean or friction uses of Skill Evidence Capture itself. A self-targeted receipt is allowed only for an actual failed capture attempt — `material_failure` or worse, citing concrete evidence references; the compiled command enforces this. The recording invocation itself earns no receipt, and no diagnosis or self-edit follows.

No report file is produced for ordinary capture; markdown reports belong to evolution runs. Maintainers only, never loaded during capture: the design contract and source workflow live in the repository that authors this skill set, as `archive/workflows/00_shared-skill-evolution-contract.md` and `archive/workflows/01_skill-evidence-capture-workflow.md`. They are absent in a repository that only consumes the skill.
