---
name: commission-method-gap-research
description: Commission externally grounded method-gap research for an existing skill only. Use when the user wants to discover missing instrument classes or methodological blind spots that the skill's own execution evidence may never expose, or wants a census of prior method-gap research to decide whether another audit is due. Do not use for ordinary skill friction, rewriting, defect repair, or a generic literature search.
---

# Commission Method-Gap Research

Find **missing organs** in another skill by preparing a bounded external research commission. This
skill maps, fences, and briefs the research; it does not perform the research, edit the target, or
adopt any returned recommendation.

### Skill-local meta-tooling

The brief validator under `scripts/` is executable meta-tooling. It does not read, write, or
adjudicate recorded evidence, gate state, or dispositions. Its production path reads only the
explicit brief and optional evidence-bundle inputs, emits validation diagnostics, one success
receipt, and exit status, performs no network access, uses no external service, and remains confined
to this skill package.

Determinism contract: with the same Node.js runtime, command arguments, current working directory,
byte-identical brief and bundle inputs, and the same input path identities, the validator emits the
same diagnostics, receipt, and exit status. It consumes no clock, randomness, network, or
external-service input.

Runtime requirement: Node.js 20 or later, with ESM `.mjs` and the built-in `node:test` runner. The
single production script has one declaration entry:

- `scripts/validate-research-brief.mjs` is invoked as
  `node .claude/skills/commission-method-gap-research/scripts/validate-research-brief.mjs <brief-path> [--bundle <bundle-dir>]`.
  It reads the brief plus an explicitly named or canonical sibling bundle, including the bundle
  manifest and its declared files. A valid brief emits `Research brief valid: <resolved-path>.` to
  standard output and exits 0; validation failures emit one `- <diagnostic>` line per failure to
  standard error and exit 1; usage failures emit a diagnostic and usage to standard error and exit
  2; `--help` emits usage to standard output and exits 0. Its verification suite is
  `node --test .claude/skills/commission-method-gap-research/scripts/validate-research-brief.test.mjs`.
  The suite writes only to fixtures under the operating system's temporary directory and removes
  them after each test; the production command writes no repository file.

## Workflow

### 1. Resolve and freeze the target

Accept an exact `SKILL.md` path or a directory containing one. If a loose input resolves to zero or
multiple skills, stop and report the ambiguity. Reject non-skill targets; this skill audits skills
only.

Resolve the target's repository root from the target path, never from the session working directory
by default: use its nearest version-control root; otherwise its nearest ancestor containing a repo
entrypoint plus a skill tree; otherwise the target directory. Scope authority and lineage searches
to that root unless an explicit source points elsewhere.

Before repository-wide inspection, capture an intake custody ledger: branch or immutable ref,
persisted repository-wide status with untracked directories expanded to per-file rows,
target-package hashes, and the expected commission-artifact location when known. Treat status as
transition evidence, never as a content monitor. Pair the target hashes with content identities for
any existing intended artifact and every external evidence source at first reliance. Record each
path's role, immutable ref or digest, and status. Persist the status listing and every identity row
to one scratch ledger file outside the repository before repository-wide inspection begins, append
each later external identity at its first reliance, and read the persisted rows back before relying
on them: an in-context listing that was never written to disk is not captured. When this step
fires, apply a capture-before-reliance and final-reverification protocol; a digest check licenses
an unchanged earlier read. Classify
entries as target state, intended commission artifacts, active evidence, or pre-existing unrelated
work. Preserve a dirty worktree as found; do not adopt or normalize unrelated changes.

Read the target `SKILL.md` completely. Inventory its behavior-bearing references, templates,
scripts, tests, assets, host metadata, and discovery mirrors before drawing conclusions. Read the
repository entrypoints and active authorities that govern skills. For every inventoried path,
record `read - <role>` or `excluded - <reason>`. Record the requested path, resolved and real paths,
content hash, version when declared, commit/blob identity when available, and worktree status. Hash
every included file whose current bytes are not identified by an immutable source reference. Do
not call two paths mirrors without symlink or byte-identity proof.

Done when the exact skill implementation, repository scope, and governing authority are
unambiguous.

### 2. Classify the concern

Read [Gap classifier](references/gap-classifier.md) completely. Separate absent capability from
ordinary friction, conformance defects, out-of-distribution pressure, verdict calibration, human
transfer calibration, and policy disagreement.

If no proposed concern can meet the method-gap qualification test, name the proper route and stop.
For a mixed request, keep only the qualifying method-gap portion in this commission and list the
separately routed portions. Treat hypothetical Features from a `skill-audit` report as candidate
evidence, never as proof that a method gap exists.

An open-ended request need not name a candidate gap. Classify the requested search for missing
capability itself, then let the census and method map generate evidence-grounded candidates.

Done when every concern has exactly one route and the commissioned scope contains only plausible
method-gap questions.

### 3. Census lineage and decide whether to commission

Read [Lineage and cadence](references/lineage-and-cadence.md) completely. Discover the repository's
actual lineage convention rather than assuming filenames or directories. Reconstruct prior briefs,
returned reports, dispositions, adopted landings, and deferred triggers. Classify current work by
function and consumption state as that reference defines; do not collapse active target execution
or downstream adjudication into overlapping method-gap research. Recommend exactly one outcome:
`commission now`, `postpone`, or `decline`.

A request that explicitly says to commission the research or author the research brief counts as
authorization when the census recommends `commission now`; a bare invocation of this skill that
names only a target is such a request, with the census gate and the conflict pause below applying
to it unchanged. If newly discovered target-specific
research, an unstable target baseline, or another material fact changes the recommendation to
`postpone` or `decline`, present that conflict and pause for an informed owner decision; do not
assume the earlier authorization survives it. A cadence check, census, or request to decide whether
research is due does not authorize a brief: present the census and recommendation, then stop. Let
the owner narrow, postpone, or decline; do not re-argue the choice. When the owner ratifies
`postpone` or `decline`, offer the durable census marker defined in
[Lineage and cadence](references/lineage-and-cadence.md#leave-a-recoverable-marker) and write it
only with the owner's consent.

Done when overlapping method-gap research is ruled out, the target and evidence baseline can be
frozen coherently, the delta over prior research is explicit, and the owner has authorized one
bounded commission—or the correct non-commission terminal has been delivered.

### 4. Map the skill and assemble evidence

Read every path marked `read` in the target inventory, plus target-identifying execution history:
skill-audit reports, trials, friction records, changelogs, tests, evaluator results, manual
exceptions, landing records, and unresolved judgment calls. Search by target names, paths, stable
markers, and versions first; read only hits that could change the lineage or method map. Do not
reconstruct unrelated repository history. Build the claim-instrument map defined in the classifier:
intended outcomes, material claims, instruments, evidence artifacts, thresholds, authority,
failure routes, and feedback routes.

When a current validator or evaluator is applied to a historical artifact, follow [Lineage and
cadence](references/lineage-and-cadence.md#validate-historical-artifacts-without-rewriting-lineage).
Capture both identities before execution, distinguish contemporaneous from retrospective results,
and never rewrite history to satisfy a later rule.

State the real operating envelope: maintainers, time, tools and compute, participant or expert
access, privacy constraints, and maximum acceptable recurring burden. Interview the owner only for
missing choices that would materially change scope, privacy, acceptable burden, settled negatives,
or the research executor. Prefer evidence-backed defaults for everything else.

Stop exploring when every inventory path is accounted for, every target-specific lineage candidate
is classified, and every method-map row either cites current machinery/evidence or records an
explicit blank. Done when an external researcher can inspect the skill actually in force without
inventing its goals, machinery, constraints, or authority model.

### 5. Assemble the research delta

Build three explicit fences from the census and owner decisions:

- adopted lineage that must not be rediscovered without credible contrary evidence;
- settled negatives, each with a reason and reopening trigger or `none`; and
- the operating envelope as a binding design constraint.

Generate candidate gap areas from blank or weak cells in the method map and from interval evidence.
State each as a hypothesis, not a conclusion. Select external bodies of knowledge by the function
they may supply, and require an open-ended sweep beyond the named candidates.

Do not search external literature, verify prior art, or decide whether a candidate is true during
commissioning. Those are executor tasks. Name functional vantages and the claims the executor must
test; use external access only later through the authorized research handoff.

Done when every candidate has local evidence, every external claim remains an explicit research
task, the audit stays open to unnamed gap classes, and the fences make this run a true delta.

### 6. Emit the executor-ready brief

Read [Research brief template](references/research-brief-template.md) completely, then instantiate
it without dropping sections. Choose an evidence-delivery mode the executor can actually use:
verified direct references, a fully inlined brief, or one authorized commission artifact set. An
artifact set consists of the brief plus at most one sibling `<brief-stem>.evidence/` directory that
obeys the template's manifest contract; it is one deliverable, not a new authority surface. Put
every exact inline source in one of the template's canonical base64 or readable UTF-8 payload
shapes so the validator can recheck its identity.

Prefer the bundle when the executor's declared transport or context constraints, line/fence
safety, or repository policy make inlining unreliable. Treat any size guideline as a sourced,
versioned executor constraint, not a universal cutoff. An explicit owner request for inlining
overrides that preference only after a delivery preflight confirms that the executor can receive
the complete brief intact, privacy rules permit it, and every canonical payload validates. If no
declared mode can carry the complete artifact set, stop. Never hand off inaccessible local paths.
Apply the repository's privacy and sanitization rules.

Honor the user's requested output form, whether inline or at an exact path. With no requested form,
write to the repository's existing research/report location; if none exists, return the brief
inline rather than creating a new authority directory. When the executor will read a repository
ref, verify that every named source exists at that ref. Materialize or inline dirty and untracked
source bytes with hashes rather than pretending they exist at the ref. Give the brief a stable
identifier and record the target, evidence-packet, and artifact-set identities it audits. Complete
the template's semantic quality gate, then run its deterministic validator before presenting it:

```text
node <this-skill-directory>/scripts/validate-research-brief.mjs <brief-path> [--bundle <bundle-dir>]
```

The command must pass. It auto-detects the canonical sibling bundle when `--bundle` is omitted.

Done when the complete artifact set is self-contained for its declared access mode and every
required report field, negative result, citation rule, witness, disposition, and authority
boundary is commissioned.

### 7. Hand off without adopting

Provide the exact ready-to-run instruction for the authorized research executor. Do not invoke a
paid or external mechanism without separate authorization. Restate the consumption route:
independently verify critical citations, adjudicate every recommendation, run cheap witnesses where
needed, and revise the target only through its normal change process.

Re-hash the target, the authorized artifact set, and every active content-identity entry, then
compare per-file final status with the intake custody ledger. The only permitted commission-owned
repository state is the authorized artifact set: one brief and, when declared and validated, its
single sibling evidence bundle. If an active evidence source changed after its bytes entered the
brief, rebuild and revalidate against the final identity or stop. Record any other status transition
or same-status digest change as concurrent rather than attributing it to the commission.

For an unrelated path the commission never read, report only observed status transitions; do not
claim byte identity or authorship. The terminal may proceed when the target identity is unchanged,
the artifact set is exact, every active evidence identity matches the delivered brief, and no
unexplained transition threatens that scoped custody. Preserve pre-existing unrelated changes.

Done when the owner has the brief plus an executable handoff, and no recommendation has been
mistaken for an adopted change.

## Completion terminals

- **Census terminal:** target identity, lineage state, due/not-due reasoning, one
  `commission now`/`postpone`/`decline` recommendation, and no brief. Close its custody in scope:
  re-verify the target identity against intake or characterize its change, attribute observed
  status transitions as concurrent rather than commission-owned, and state that the run wrote
  nothing into the repository beyond an owner-consented census marker, when one was ratified.
- **Commission terminal:** one validated research brief with at most one verified sibling evidence
  bundle, one exact executor handoff, the scoped custody result for target, artifacts, and active
  evidence identities, and the named post-report consumption route.

Never claim that a bounded audit proves the target has no method gaps.
