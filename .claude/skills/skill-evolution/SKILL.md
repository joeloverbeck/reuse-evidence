---
name: skill-evolution
description: Evidence-gated revision of another skill — hard-refuses unless the freshly derived gate authorizes a review, and lands only a blind-validated candidate.
argument-hint: "<path of the target skill directory>"
disable-model-invocation: true
---

# Skill Evolution

Revise a target skill only when accumulated prospective evidence has opened a mechanical authorization gate, review confirms the target owns the recurring failure, and fresh blind comparative trials prove an isolated candidate materially better without unacceptable regression. This skill is deliberately unavailable for preventive checking, aesthetic cleanup, reassurance, or reaction to a newly noticed imperfection: it is event-authorized, never schedule-authorized, and a hard refusal on a closed gate is a successful, complete run. Invoking it is authorization to land a validated candidate — no second "implement now" prompt — and equally authorization for nothing at all when the gate or validation says no.

Arguments: the path of the target skill directory (required). Everything else is derived mechanically from its evidence store, `reports/skill-evidence/<skill-key>/`.

## Hard boundaries

- Only the freshly derived gate authorizes work. No rationale substitutes: not reassurance, a noticed possible improvement, staleness, size or ugliness, one non-severe incident, many clean uses, a new model or tool version without observed incompatibility, old `skill-audit` history, a prior premature refusal, explicit user insistence, or an "obvious tiny" change.
- Until the preflight passes: never read the target semantically, inspect it or repository conventions for improvement opportunities, propose or suggest anything, append any incident, or edit anything. Repeated invocations do not advance the gate, create evidence, or shorten the cooldown.
- Fresh session means a different top-level session. Never satisfy the cooldown through a child agent, subagent, continuation, or context reset inside the same top-level session.
- Self-targeting is forbidden. If this skill's own evidence gate ever opens, route the bounded evidence packet to an independent skill-authoring workflow; judging the live skill via a copied candidate of itself is still self-review.
- Evidence is immutable: every event write goes through the compiled command; never hand-edit `events.jsonl` or `gate-status.json`, and never delete or rewrite earlier events.

## Workflow

### 1. Run the mechanical preflight

From the repository root:

```bash
cargo run --locked -p reuse-evidence -- skills evolution preflight \
  --target <skill-path> --recorded-at <RFC3339-clock> \
  --now-epoch-milliseconds <clock-ms> --session-id <top-level-session-id> \
  --lock-owner <caller-owned-lock-id>
```

One run is the whole check: the compiled command validates event-stream integrity, regenerates `gate-status.json`, hashes the live target, and evaluates every authorization term — authorized workflow, eligible or quarantined-eligible state, hash match, no active review, fresh-session/cooldown, non-self target. Clock, session, and lock identities are explicit caller inputs; the command never reads the ambient clock or generates an identity.

*Done when the compiled command printed either a refusal block or an authorized evidence packet.*

### 2. On refusal: relay and stop

Relay the compiled command's refusal verbatim — gate state, failed condition, terminal outcome — and end the run with no analysis, no suggestions, and no softening ("we could still look at…"). Safe refusal is success.

*Done when the refusal was relayed unchanged and nothing else was done to or about the target.*

### 3. On authorization: run the review

Read `references/authorized-review.md` and execute it end-to-end, starting by claiming the review.

*Done when exactly one terminal outcome was reached and recorded through the compiled command, the review report exists under `reports/skill-evidence/<skill-key>/reviews/`, and the user-facing completion states whether the live skill changed.*

Maintainers only, never loaded at runtime: the design contract and source workflow live in the repository that authors this skill set, as `archive/workflows/00_shared-skill-evolution-contract.md` and `archive/workflows/02_skill-evolution-workflow.md`. They are absent in a repository that only consumes the skill.
