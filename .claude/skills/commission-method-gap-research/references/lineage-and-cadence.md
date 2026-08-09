# Lineage and Cadence

Use this reference to decide whether a new commission is due and to make it a true delta over all
earlier research.

## Contents

- [Discover lineage before judging cadence](#discover-lineage-before-judging-cadence)
- [Classify in-flight work by function](#classify-in-flight-work-by-function)
- [Validate historical artifacts without rewriting lineage](#validate-historical-artifacts-without-rewriting-lineage)
- [Decide whether another audit is due](#decide-whether-another-audit-is-due)
- [Build the adopted-lineage fence](#build-the-adopted-lineage-fence)
- [Build the settled-negative fence](#build-the-settled-negative-fence)
- [Make the operating envelope binding](#make-the-operating-envelope-binding)
- [Assemble the evidence packet](#assemble-the-evidence-packet)
- [Leave a recoverable marker](#leave-a-recoverable-marker)

## Discover lineage before judging cadence

Do not assume a repository layout. Search the target name, prior names, stable brief/report markers,
and declared versions across the repository's current report, research, decision, and archive
conventions. When available, inspect version-control history and the issue tracker for adopted
landings and deferred triggers. Discover candidates with target-identifying searches, then read
only candidates that could alter lineage state; do not inventory unrelated history or tracker
work. Distinguish current authority from archived evidence. A generic commissioning framework is
not target lineage unless it identifies this target or records a target-specific disposition.
Keep every discovery search inside the repository root resolved from the target. The caller's
working directory is not evidence that another repository owns the target.

Record the target identity:

- requested, resolved, and real paths;
- target name and declared version, or `unversioned`;
- content hash;
- commit/blob identity and worktree status when version control exists;
- canonical implementation, host adapter, symlink mirror, byte-identical mirror, or unresolved
  authority posture.

Then place every prior line in exactly one state:

1. brief authored, research not returned;
2. report returned, not yet adjudicated;
3. recommendation adopted, with exact landing location and version;
4. recommendation rejected, with reason;
5. recommendation deferred, with a concrete trigger and whether it has fired; or
6. superseded, with the replacing line named.

`None found` is a valid census result and means the proposed run would be the first audit. When the
target declares no version, use material changes to the canonical package on the current lineage as
the cadence proxy; do not count duplicate imports or commits reachable only through other branches
as separate current-baseline versions.

Do not open a new audit over in-flight or unconsumed work merely because reconstructing its state is
inconvenient.

## Classify in-flight work by function

Do not treat every active or unconsumed artifact as overlapping research. Classify each current
line before deciding cadence:

| Current state | Default cadence effect | Evidence treatment |
| ------------- | ---------------------- | ------------------ |
| Target-specific method-gap brief; research not returned | `postpone` | Record the brief identity and wait for its report. |
| Target-specific method-gap report; not yet adjudicated | `postpone` | Consume or disposition the report before commissioning a duplicate line. |
| Active target execution or edit | Not overlap by itself | Treat its records as interval evidence; proceed only when a coherent target and evidence baseline can be frozen. |
| Downstream internal adjudication, calibration, or repair | Not overlap by itself | Map it as existing feedback machinery and interval evidence; apply the same baseline-stability gate. |
| Unrelated repository work | No cadence effect | Keep it in custody status; give it a content identity only if the commission relies on it. |

When target execution or downstream work is changing too rapidly to freeze a trustworthy baseline,
prefer `postpone`. Otherwise disclose its exact state in the brief and distinguish it from
target-specific method-gap research. If this classification changes an earlier recommendation,
return to the owner-decision pause in `SKILL.md`.

## Validate historical artifacts without rewriting lineage

Treat a validator or evaluator result as versioned evidence. Before executing one against a
historical artifact, record:

- the artifact path, immutable identity or current hash, and original date or ref;
- the validator path plus its version, commit/blob identity, or current hash; and
- whether the validator is contemporaneous with the artifact or is a later retrospective check.

A retrospective pass or failure is a compatibility observation, not proof that the artifact did
or did not conform when produced. Preserve the historical bytes. Never edit an old brief, report,
or trial merely to satisfy a later rule. When the contemporaneous validator is recoverable and safe
to run, execute it separately and report both results; otherwise mark original conformance
`unknown` and state the later rule that caused drift. Feed a material drift observation into the
method map or lineage disposition without erasing the earlier evidence.

## Decide whether another audit is due

Evidence favoring `commission now` includes:

- the skill has stable machinery and an observable revision history;
- roughly ten meaningful versions have accumulated since the last external method-gap audit;
- a substantial prior research line was adopted and changed the baseline;
- recurring failures resist the skill's internal categories;
- known gaps or judgment calls survive several landing reports;
- the skill now makes stronger claims than its evidence machinery was built to support;
- its operating envelope, user population, scale, or consequence of error changed materially; or
- an out-of-distribution test exposed missing organs that need prior-art research.

The ten-version interval is a configurable heuristic, not a threshold. Prefer `postpone` when a
target-specific method-gap brief or report is already in flight, the skill is changing too rapidly
to freeze a useful baseline, or a deferred trigger has not fired. Prefer `decline` when the concern
is out of scope, already covered, a policy disagreement, or unsupported by material consequence.

Return one recommendation with the evidence that controls it. A census cannot prove that research
is unnecessary forever.

## Build the adopted-lineage fence

For every adopted research line, record:

| Research line | Capability added | Current location | Version/date | Remaining limit |
| ------------- | ---------------- | ---------------- | ------------ | --------------- |

Instruct the researcher to treat these capabilities as established. Reopen one only when credible
evidence shows capability failure, conflict with another instrument, or a newly declared operating
condition that it does not cover.

## Build the settled-negative fence

Silence is not a fence. For each deliberate exclusion record:

| Area | Permanent or this run | Reason | Reopen trigger or `none` |
| ---- | --------------------- | ------ | ------------------------ |

Only the owner may settle or reopen a policy negative. Research may challenge the supporting facts,
but it cannot silently make the policy choice.

## Make the operating envelope binding

Record constraints as requirements for every recommendation:

- maintainer count and roles;
- available one-time and recurring time;
- compute, tools, and infrastructure;
- access to users, participants, and domain experts;
- privacy, sanitization, publication, and source-access limits; and
- maximum acceptable ceremony and recordkeeping.

An adapted instrument that exceeds the envelope is not adoptable merely because its institutional
form is well supported.

When any evidence source is dirty or untracked, record its current hash and provide those exact
bytes through the declared access mode. A repository commit identifies only files present at that
commit.

## Assemble the evidence packet

Include enough inspectable evidence for the executor to reconstruct the skill in force:

- authoritative `SKILL.md` plus behavior-bearing references, templates, schemas, scripts, tests,
  evaluators, host metadata, and mirrors;
- intended outcomes, declared scope, version history, and authority structure;
- execution records, skill-audit reports, friction and snag ledgers, manual exceptions, known gaps,
  passing tests that coexist with unease, and failures current categories cannot explain;
- prior briefs, reports, dispositions, witness results, and exact adopted landings; and
- the declared evidence-delivery mode and privacy posture.

Retain raw records when policy permits. A later instrument may interpret evidence that the current
skill could not.

## Leave a recoverable marker

Each brief must record at least the audited skill identity, commission date, stable brief ID, prior
audit, in-flight state, and intended consumption owner. When a report returns, the lineage should
later add its identifier, recommendation dispositions, witness results, and adopted landing
version. Filenames may help discovery, but recoverable provenance is the invariant.

A ratified census outcome is lineage too. When the owner ratifies `postpone` or `decline`, offer
one minimal census marker at the repository's existing research location — for example
`<target>-method-gap-census.md` beside prior briefs — recording the target identity and hash, the
ruling and its date, the reopening or re-census triggers, and the census's controlling evidence. It
is a disposition record, never a brief, and it is written only with the owner's consent; without
it, the ratification and its triggers survive only in conversation, where the next census cannot
recover them.
