---
name: skill-evolution-status
description: Read-only readiness census for evidence-gated skills in the current repository — distinguishes targets ready for Skill Evolution, queued pre-close evidence that remains deferred, evidence retired as untestable by an instrument-limited close, eligible targets that remain blocked, and stores whose eligibility cannot be determined.
disable-model-invocation: true
---

# Skill Evolution Status

Run one read-only readiness census across `reports/skill-evidence/` in the current repository. This skill is a reporter, not an authorizer: the destination session's fresh Skill Evolution preflight remains final.

Arguments: none.

## Hard boundaries

- Read only: never write `events.jsonl` or `gate-status.json`, run a preflight or derive command that refreshes projections, claim a review, or modify a target.
- Census only: never diagnose incidents, inspect target prose semantically, propose repairs, invoke Skill Evolution, or record evidence.
- Current repository only: never search sibling repositories or a global evidence directory.
- Canonical mechanics only: use the compiled command's shared Rust event validation, target hashing, and in-memory gate derivation; never duplicate eligibility thresholds or trust a stored projection as current.
- Safe uncertainty: never print an evolution command for queued pre-close evidence, evidence retired as untestable, self-targeting, an active timer, an owned review, an unreadable stream, or a missing target.

## Workflow

### 1. Run the census

From the repository root, run exactly:

```bash
cargo run --locked -p reuse-evidence -- skills evolution-status \
  --root . --now-epoch-milliseconds "<clock-ms>"
```

Capture one exact census clock in epoch milliseconds and pass it as
`<clock-ms>`; replay with the same value. The compiled command scans every
evidence store, validates `events.jsonl`, hashes each live target, and derives
current gate state without persisting it. It distinguishes the two freshness
proofs explicitly:

- a captured threshold session ID is ready only in another session that exposes a different ID; a no-ID destination remains blocked and waiting does not help;
- a missing threshold session ID uses the 12-hour clock, which another session cannot bypass.

It also relays the canonical post-review provenance derived by Skill Evidence Capture. Evidence that could only have fired before a completed same-hash review is reported as deferred and receives no command; a genuinely new post-review incident can reopen the gate, with the bounded trigger cluster and provenance shown on the actionable entry.

Portability contract: run the repository's compiled `reuse-evidence` command in a
checkout containing the sibling Skill Evolution skill at its standard
`.claude/skills/` path and the standard `reports/skill-evidence/` store. The
command fails safely when that contract is absent or incompatible.

_Done when the compiled command prints one reconciled census or a safe failure stating that nothing changed._

### 2. Relay the result and stop

Relay the compiled command output verbatim. Commands appear only under `Ready to evolve` and are already shaped for copy-paste into another top-level session. `Deferred after review` identifies the canonical queued subset of collecting stores and explicitly supplies no command, distinguishing evidence a review accounted for from evidence queued behind a close that reached no conclusion. `Retired as untestable` names stores whose open incidents left the gate under an instrument-limited close: real evidence that no longer drives eligibility and never will, listed separately so it is not read as an absence of evidence. `Eligible but blocked` gives the active blocker and all available timing, ownership, quarantine, or routing detail. `Could not determine` names stores that might conceal eligibility but cannot be trusted. Other closed, collecting, and current-hash-reset stores appear only in the omitted count.

Do not reinterpret readiness, add commands to blocked entries, suggest early review, or continue into evolution in this session.

_Done when the exact census or safe failure has been delivered and no further action was taken._
