---
name: method-gap-research-status
description: Read-only family census that recommends one skill, or none, for a target-specific method-gap research census.
argument-hint: "<skill-name-family, for example game-*>"
disable-model-invocation: true
---

# Method-Gap Research Status

Run one read-only census across an explicitly named family of skills. Recommend
at most one exact target for a later, census-only method-gap commission.
This skill is a portfolio selector, not a method-gap auditor: it never concludes
that a method gap exists, decides that research is due, authors a brief, or
invokes the commission.

Arguments: one skill-name family expressed as a literal name prefix followed by
one terminal `*`, for example `game-*`. Paths, multiple wildcards, character
classes, brace expansion, and a bare `*` are refused.

## Hard boundaries

- Read only: never modify a target, evidence stream, stored projection, research
  artifact, report, marker, issue, or repository metadata.
- Family screen only: never semantically audit every target or build a
  claim-instrument map. The target-specific commission owns that work.
- Evidence is not diagnosis: never relabel ordinary friction, a Skill Evolution
  gate, a conformance defect, calibration pressure, or policy disagreement as a
  method-gap signal.
- Safe uncertainty: missing, corrupt, stale, ambiguous, or insufficient evidence
  never means a skill is healthy or research is not due.
- No authority escalation: never invoke a method-gap commission, emit a
  brief-authoring instruction, perform external research, or adopt a prior
  recommendation.
- Transient result: do not write a census report. A durable `postpone` or
  `decline` marker belongs only to a later target-specific census with the
  owner's consent.

## Procedure

### 1. Resolve and inventory the family

From the repository root, run:

```bash
cargo run --locked -p reuse-evidence -- skills method-gap-research-status \
  "<family>" --root . --now-epoch-milliseconds "<clock-ms>"
```

Choose one exact census clock in epoch milliseconds before invoking the command;
reuse that value for any replay. The compiled command validates the selector, enumerates canonical
`.claude/skills/` members, hashes their complete packages, validates each
matching `reports/skill-evidence/` event stream through the shared Rust
skill-evidence authority, derives current-hash use and incident facts in memory,
discovers target-identifying method-gap lineage candidates, extracts
  bounded hashed headers and lineage-signal lines, and inspects target-scoped
  Git status and recent commit subjects when Git is available. It writes nothing.

Fail safely if the compiled command fails, the shared evidence contract is
incompatible, or no skill matches. Do not substitute a hand-built family scan
for a failed command.

*Done when one complete inventory covers every exact family member and identifies
every indeterminate store or unavailable Git observation without changing the
repository.*

### 2. Reconstruct only decision-relevant lineage

Read [Lineage reconstruction](references/lineage-reconstruction.md) completely.
Use each candidate's inventory-provided `header_excerpt`, `signal_lines`, kind,
size, and digest first. Read an exact additional section only when those bounded
projections leave an ambiguity that could change the member's selection. Never
read embedded evidence payloads or whole reports merely to prove absence.
Search by that target's exact name, path, prior names found in those artifacts,
and stable report markers only when a named candidate reveals that the inventory
is incomplete.

For each family member, establish:

- whether target-specific research is already in flight or returned but
  unconsumed;
- the latest audited target identity and any adopted landing;
- rejected or deferred recommendations and exact reopening triggers;
- whether later target changes form a stable, observable interval; and
- whether current evidence is absent, unreadable, on an old hash, or otherwise
  incapable of supporting a comparison.

Do not read every target package semantically. A current dirty target is not
automatically excluded, but it cannot be selected when its method-bearing
baseline is changing too rapidly to freeze coherently.

*Done when every member has one recoverable lineage state, no irrelevant report
body or embedded payload was loaded, and every ambiguity that could change
selection is either resolved from an exact bounded section or explicitly
indeterminate.*

### 3. Classify candidacy without diagnosing

Read [Selection rules](references/selection-rules.md) completely. Apply its
routes, eligibility gates, ordered signals, and comparison rules to the
inventory and reconstructed lineage.

Receipts supply interval evidence, not a method-gap verdict. Repeated ordinary
friction remains owned by Skill Evolution. A family member becomes a
`screen next` candidate only when the record supports a plausible
missing-capability question with material consequence and a target-specific
census could decide whether external research is due.

Assign every member exactly one status:

- `screen next`;
- `watch`;
- `research in flight`;
- `insufficient evidence`;
- `no current signal`; or
- `could not determine`.

There may be at most one `screen next`. When tied evidence cannot support one
choice without arbitrary scoring, assign the tied members `watch` and return no
selection.

*Done when all members are classified once, ordinary iteration routes remain
separate, every positive signal cites concrete current evidence, and at most one
exact target survives.*

### 4. Render the transient census and stop

Return:

1. the family selector and number of canonical members;
2. `Screen next` with the exact target, controlling evidence, material
   consequence, baseline/lineage state, and why a target-specific census can
   resolve the uncertainty—or `None`;
3. the remaining members grouped under their exact statuses with terse reasons;
4. unavailable or pending evidence;
5. a literal statement that the census wrote nothing and did not determine that
   any method gap exists; and
6. only when one target is selected, this census-only mandate:

```text
Census only for <exact-target-path>. Decide commission now, postpone, or decline; do not author a research brief in this invocation.
```

Never shorten that mandate to a bare target: a bare target can authorize a
brief when the target-specific census recommends `commission now`.

*Done when the output accounts for the whole family, contains zero or one safe
handoff, and performs no follow-on action.*

## Completion terminals

- **Selection:** one exact `screen next` target, one census-only mandate, and
  no mutation or research authorization.
- **No selection:** every family member classified, the controlling absence,
  tie, insufficiency, in-flight work, or uncertainty stated, and no invocation.
- **Safe failure:** the failed contract or instrument named, nothing changed,
  and no hand-built substitute result.
