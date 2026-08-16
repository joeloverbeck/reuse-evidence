---
name: reuse-evidence-review
description: Review one review-ready case and prepare its exact reuse decision.
argument-hint: "<case-id> [--root <portfolio-root> ...]"
disable-model-invocation: true
---

# Reuse Evidence Review

Manually review one named review-ready case. End with an exact decision awaiting the human's
authorization, the recorded decision and its compiled implementation brief, or a precise refusal,
blocker, decline, or unmet evidence need. Never implement the decision.

The case identity is required. Portfolio roots are optional only when user-local configuration
already selects them. Preserve every root the human supplied and pass the same set to every
compiled command. Never infer a root, scan outside those roots, or write portfolio configuration.

## Boundaries

- Bound the investigation to one case, its complete compiled rendering, and the recoverable
  artifacts its recorded evidence references identify. A general architecture audit, candidate
  discovery, implementation, migration, refactoring, and code review remain outside this skill.
- Treat the human as the semantic authority. Review proposes exact content; only the human accepts
  the responsibility identity, action, scope, disclosure, and event bytes.
- Keep working notes in conversation context. Keep every draft and approved byte outside every
  repository working tree, beneath the directory returned by `reuse-evidence staging-directory`.
- Make no network call except the decision-bound package-research branch, after its per-request
  disclosure gate succeeds. No remote model, search engine, source upload, credential, or account is
  part of review.
- Relay every compiled refusal verbatim and stop the affected branch. Never restate refused content
  as a proposal the human could approve.
- Treat retaining intentional duplication, waiting for evidence, and splitting a wrong abstraction
  as successful decisions when the evidence supports them. Review readiness authorizes analysis,
  never extraction.

## Workflow

### 1. Reach one healthy review-ready case

From any working directory, run the read-only portfolio query with the human-selected roots, or
with configured roots when none were supplied:

```console
reuse-evidence case find [--root <portfolio-root> ...]
```

If no root selection resolves, relay the compiled refusal verbatim and stop. Do not choose a root
for the human. Select the row whose `case_id` exactly matches the argument. If no row matches, report
that no enrolled steward under the selected roots owns the named case and stop without staging. If
the row reports `condition: damaged-recorded-event-history`, relay that complete row, including its
`detail`, verbatim and stop.

Enter the row's `steward_path` and read the complete authoritative history:

```console
reuse-evidence case show <case-id> [--root <portfolio-root> ...]
```

Relay any refusal verbatim. Before analysis, report together the `privacy` from `case find` and the
`privacy_conflicted`, `stale`, `state`, readiness basis, revision, and occurrence count from `case
show`. Continue only when `state` is `review-ready`, `stale` is `false`, and
`privacy_conflicted` is `false`. Otherwise report the exact derived condition and why it blocks a
fresh decision: another state does not authorize review, stale or unknown participant conditions do
not support a decision, and a privacy conflict prevents safe publication. Stop without staging.

*Done when one healthy review-ready case is bound to its actual steward and its current privacy and
privacy-conflict condition have been shown before any semantic analysis or write.*

### 2. Inspect the recorded evidence, including shared doctrine

Use the selected roots and the stable marker identities to reach only the uniquely enrolled
repository recorded for each occurrence. Inspect every referenced commit and optional
repository-relative path needed by the review. Do not substitute conversation memory, an unmarked
repository, a generated report that only cites another report, or a similarity score for the
recorded artifact.

Test the case's independence claim again at review stakes. Inspect the participant repositories'
authored agent instructions, constitutions, principles, and inherited policy for a shared mandate
requiring the alleged responsibility. Repositories conforming to one authored doctrine are one
coordinated consumer context unless recoverable consumer evidence establishes independent pressure
beyond that mandate. Record the uncertainty rather than preserving an inflated occurrence count.

If an evidence reference is missing, unreadable, ambiguous, or insufficient to inspect the claim it
bears, return that exact unmet evidence need and stop without staging. If answering the review would
require reconstructing the entire investigation from unrecorded source rather than following the
case's references, report that the recorded evidence model is too thin and stop; do not work around
that design falsifier inside this skill.

*Done when every occurrence and independence claim used by the review is traceable to inspected,
recoverable evidence, or one precise unmet evidence need has ended the write-free branch.*

### 3. Answer the eleven review questions and compare scope

Read [Review analysis](references/review-analysis.md) now. Answer all eleven questions in order.
For each, state the evidence-supported answer or say exactly what the evidence does not settle. Use
the full scope ladder, including the no-sharing and de-abstraction outcomes, and explain why the
provisional choice is the narrowest scope that creates real leverage.

If question 7 makes Rust package research decision-bound, mark that answer and any answer depending
on it as pending rather than guessing. Complete every independent answer the recorded evidence
supports, take step 4, then return here and finish the eleven-question analysis from the assessed
findings before staging anything. Any other evidence gap remains an unmet need and stops the review.

Keep the analysis in context until exact decision content is ready. Do not commit a report or add a
second durable decision artifact. A result that lacks a defensible identity verdict, action, scope,
affected consumer set, or verification conditions is an unmet evidence need, not a partially filled
proposal.

*Done when the eleven answers support one provisional identity verdict, action, narrowest scope,
non-responsibility boundary, consumer set, alternatives, consequences, verification contract, and
falsifier; or one exact decision-bound research need is the only pending part before step 4; or the
review has stopped with explicit uncertainty.*

### 4. Take the package-research branch only when the decision needs it

Decide from the provisional action whether Rust package research is required. A review considering
no new crate dependency makes no network call and proceeds directly to decision content. Findings
the human supplies may be inspected locally against the required criteria without making a call.

When decision-bound research is required, read [Package research and disclosure](references/package-research.md)
and follow it per HTTP request. If the human declines, make no call. With neither an approved search
nor adequate supplied findings, do not recommend a new public Rust crate; either return the unmet
evidence need or continue only with a different evidence-supported action.

*Done when no research was required, supplied or approved findings have been assessed and assigned
to the decision field the action permits, or the no-call branch has ended with an unmet need. After
assessing findings, return to step 3 and complete every pending answer before continuing.*

### 5. Stage one complete decision draft

Proceed only after all eleven answers are complete. Read
[Decision and publication mechanics](references/decision-and-publication.md) now. Obtain the only
permitted staging location by running:

```console
reuse-evidence staging-directory
```

Relay a refusal verbatim. Only now create the returned directory if it is absent and write one draft
beneath it. Populate all eight always-required decision fields. Populate all five
implementation-authorizing fields exactly when the action authorizes implementation, and omit all
five for `retain_intentional_duplication` or `wait_for_more_evidence`. Name only recorded
repository-and-consumer pairs as affected consumers. Make every verification condition an
observable condition that later evidence can answer.

Record completed package research on the decision event, never as an evidence reference:
`existing_packages_considered` for an implementation-authorizing action, or one
`alternatives_rejected` entry per examined package for either no-implementation action.

*Done when one staged proposal expresses the complete analyzed decision in the existing compiled
shape, with no draft, note, report, or scratch artifact in any repository working tree.*

### 6. Preview exact event bytes and wait for exact authorization

Immediately before preview, rerun `case show` from the steward with the same roots. If the revision,
state, privacy-conflict condition, staleness, occurrences, or evidence changed, invalidate the draft
as a reviewed proposal and restart at step 1. Otherwise use that fresh revision:

```console
reuse-evidence case decide <case-id> --expected-revision <revision> \
  --proposal <staged-draft> [--root <portfolio-root> ...] --preview
```

Relay any compiled refusal verbatim and preserve staging. Save every byte after the receipt's
`event:` line, including its final newline, to a second file beneath the compiled staging directory
without parsing or reserializing it.

Present the complete receipt: target file, resulting revision, derived state, privacy consequence,
implementation consequence, and exact event bytes. Ask the human to authorize those exact bytes in
that steward and removal of the staged copies after verified publication. A requested content
change invalidates the preview; revise the draft, re-read the case, preview again, replace the saved
candidate bytes, and seek authorization of the replacement. On an explicit decline, make no case
write, remove only the unapproved staging created by this run, report that case history and every
repository working tree remain unchanged, and stop.

*Done when the exact event bytes have explicit human authorization, or a refusal, change request,
or decline has been handled without publishing unapproved content.*

### 7. Publish once, verify bytes, project the brief, and stop

After exact authorization, supply the saved event bytes—not the earlier draft—to the same compiled
command without `--preview`, from the same steward, against the same expected revision and roots.
Invoke publication once. A created receipt or an exact idempotent-existing receipt on resumed work
is acceptable; do not repeat a successful mutation.

Read the receipt's event file back and byte-compare it with the approved staged event. Only an exact
match proves publication. Then remove the applied draft and approved event from staging. Preserve
the approved bytes after any refusal, ambiguous result, unsafe failure, or mismatch, and report that
publication is unverified.

After verified publication, project the handoff through the compiled command:

```console
reuse-evidence case brief <case-id> [--root <portfolio-root> ...]
```

Relay a brief refusal verbatim. Present the complete compiled brief and stop. Do not author another
brief, implement or migrate anything, run engineering work, schedule a peer skill, or wait in prose
for another capability. A brief stating that implementation is not authorized is also a successful
terminal result.

*Done when the steward records exactly the approved bytes, applied staging is removed, and the
compiled brief has been handed to the human—or approved staging remains intact with the failed or
unverified publication reported exactly.*
