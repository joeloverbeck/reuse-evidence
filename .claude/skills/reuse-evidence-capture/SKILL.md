---
name: reuse-evidence-capture
description: Capture qualifying reuse pressure after completed material work.
argument-hint: "[completed task or review fixed point]"
disable-model-invocation: true
---

# Reuse Evidence Capture

Manually inspect one completed piece of material work for repeated responsibility. End with either
the fixed write-free no-candidate result or an exact human-authorized case event recorded by the
compiled `reuse-evidence` command.

Arguments are optional. Use a supplied completed-task reference or review fixed point to bound the
work; otherwise use the completed material work in the current conversation. Invoke this skill only
after the work has reached a terminal implementation result, normally after code review.

## Boundaries

- Capture evidence and plausible independent consumer pressure. Leave responsibility identity,
  abstraction design, extraction, implementation, and refactoring to reuse review, the human, and
  the ordinary engineering workflow.
- Treat a second occurrence as permission to remember and a third as ordinary review readiness.
  Neither count recommends extraction.
- Use recoverable repository artifacts for every durable claim. Current conversation context may
  orient the investigation, but never parse a transcript or cite conversation memory as evidence.
- Keep the clean path entirely write-free: no case event, prepared proposal, report, first-use
  inventory, receipt, scratch file, cache, index, build output, or sensor artifact.
- Use only enrolled repositories for portfolio comparison. Optional local, read-only sensor output
  may suggest where to look; capture works without a detector, model setup, GPU, external API,
  network access, or sensor score.
- Write proposal material only beneath the directory returned by `reuse-evidence
  staging-directory`. The only repository write is the compiled case command publishing one exact
  approved event in its steward repository.

## Workflow

### 1. Bound the completed work

Identify the repository, accepted task or specification, review fixed point, completed commit or
diff, and the tests and source that state the result. If the implementation is not terminal or the
current occurrence has no recoverable commit evidence, report that dependency and stop without
writing.

Inspect only the completed work and likely prior occurrences of its responsibility. Name the
consumer effect, the coherent authority or contract that may be repeating, the evidence bearing the
claim, and the narrow search terms that can find a prior occurrence. Keep working notes in context;
run only read-only commands on this path.

*Done when one completed evidence boundary and one responsibility-sized search boundary are
explicit, or the missing terminal/recoverable input has been reported with no write.*

### 2. Establish plausible independent pressure

For each possible occurrence, identify a real reuse consumer, its distinct authority, lifecycle,
release or compatibility obligation, or reason to change, and at least one recoverable commit
reference with an optional repository-relative path. Count consumer needs, not repositories, files,
functions, tests, retries, or copies.

Treat these as one occurrence unless primary evidence establishes otherwise: production and test
code serving one contract, generated copies, coordinated variants, retries or continuations, a
temporary migration copy, and several implementations created by one accepted change.

Run the shared-doctrine test before counting repositories independently: search the candidate
repositories' authored agent instructions, constitutions, principles, and inherited policy for a
mandate requiring the alleged responsibility. A shared authored mandate is evidence of one
coordinated consumer context, not evidence that each conforming implementation independently
discovered the responsibility. Count them separately only when recoverable consumer evidence shows
independent pressure beyond conformance to that mandate.

When no qualifying repeated responsibility remains, write nothing and return exactly:

`Capture complete: no qualifying repeated responsibility found. Nothing was written.`

When a plausible relationship is too uncertain to survive as a useful case, state the specific
ambiguity in one concise sentence, then end with that same fixed terminal line. Do not preserve an
uncertain candidate as an inventory or report.

*Done when at least two plausible independent occurrences remain with recoverable evidence, or the
fixed terminal line has ended a write-free capture.*

### 3. Find the authoritative case before drafting

Run the read-only portfolio query before creating a proposal:

```console
reuse-evidence case find [--root <portfolio-root> ...]
```

Compare the recorded responsibility text and evidence semantically; the query reports candidates
but does not decide identity. If it reports the same responsibility, name the case, steward path,
revision, state, and privacy. Take the append branch in that steward unless the case is `closed`. A
closed case is terminal in version 0.1: relay that existing compiled condition and stop without
staging or opening a duplicate history. If the query reports no matching case, take the opening
branch in the repository where the second occurrence is being recognized. Never open a second
history merely because capture ran outside the existing steward.

Relay a query refusal or damaged-history condition and stop without staging. Use this query rather
than a portfolio report that can update user-local observation state.

*Done when the candidate is bound either to one existing stewarded case or to one justified new
case and steward, with no proposal file yet created.*

### 4. Resolve identities and stage one draft

Read [Prepared proposal mechanics](references/prepared-proposals.md) now and follow the opening or
append branch exactly.

Obtain the staging directory by running `reuse-evidence staging-directory`; do not derive a path
from platform environment variables. Only now create that directory if absent. Resolve each
occurrence's stable repository identity from that concrete enrolled repository's valid
`reuse-evidence.toml` marker, and generate a UUID version 4 locally for a new case. Never ask the
human to transcribe either identity.

Write one draft beneath the returned staging directory. Include only the responsibility-sized
facts a later session needs: consumer, independence basis, and commit evidence with an optional
repository-relative path. Keep absolute paths and proposal material out of every repository working
tree.

*Done when one staged opening or append draft contains generated/resolved identities and every
occurrence has recoverable evidence accepted by the current command shape.*

### 5. Preview the exact event and wait for the human

From inside the steward repository, run the corresponding compiled case command with `--preview`,
the staged draft, and the same selected portfolio roots. Save the exact bytes after the receipt's
`event:` line beneath the staging directory without parsing or reserializing them. These candidate
event bytes remain non-authoritative until accepted.

If preview refuses a public steward with a private participant, relay that existing compiled
refusal and stop; never present the refused content as an approvable event. Treat every other
refusal the same way.

Present the complete preview receipt, including target file, revision, state or readiness, privacy,
and exact event bytes. Ask the human to authorize those exact bytes for publication in the named
steward and the removal of their staged copies after verified publication. A requested content
change invalidates the preview: update the draft, preview again, replace the candidate event bytes,
and seek approval of the replacement.

*Done when the exact event has explicit human authorization, or a refusal/decline has left case
history unchanged and the staged state accurately reported.*

### 6. Publish once, read back, and remove applied staging

After exact approval, supply the saved event bytes—not the earlier draft—to the same case command
without `--preview`. Run it from the steward repository with the same case identity, expected
revision where applicable, and portfolio roots. No non-steward participant repository receives a
write.

Accept only a created receipt or an exact idempotent-existing receipt as publication. Read the named
event file back and byte-compare it with the approved staged event. Then remove the applied draft
and approved event from staging; the steward event is authoritative. Preserve an approved staged
event after any refusal or unsafe failure, report the exact outcome, and do not claim publication.

Relay the compiled receipt and the staging cleanup result. If an append derives `review-ready`, say
only that semantic review is now authorized and extraction is not.

*Done when the steward records exactly the approved bytes and applied staging is removed, or the
approved bytes remain staged with the failed/refused publication accurately reported.*
