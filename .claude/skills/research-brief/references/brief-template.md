# Brief template & target-type reads

This file defines (A) the canonical anatomy of the emitted prompt, (B) the
research-target → load-bearing-reads map, and (C) the channel adaptations for a
local-session executor. The SKILL.md flow references all three.

---

## A. Canonical brief anatomy

The emitted file `reports/<topic>-research-brief.md` is the *prompt the user hands to
Session 2* — pasted into ChatGPT-Pro for a remote-fetch executor, pasted with **nothing
else** for a remote-fetch (inline) executor (an unfetchable repo — deltas in §D), or pointed
at directly for a local-session executor (deltas in §C; everything not named in the
applicable delta section applies unchanged). It is self-contained: Session 2 sees only this prompt plus the
uploaded manifest (remote-fetch) or the repository itself (local-session). Use these
eight sections, in order. Scale each to the target; omit a section only when genuinely
N/A and say so.

### 1. Context

One or two sentences. Begin with the manifest pointer, then repo identity, then the **exact
fetch-baseline commit** Session 2 must read every file from (the verified repo HEAD per the
SKILL.md Step 6 baseline-commit rule — never a commit string copied from a report without
confirming it contains the §2 read-list). Example shape:

> The uploaded manifest is the path inventory of the `<owner>/ludosmith` repo — a tool that
> lets users create tabletop games from zero to completion. [State the repo's authority order
> among docs if one exists; otherwise: root README → architecture/design → specs/roadmap →
> reference.] Fetch every file from commit `<HEAD>` — the manifest reflects that tree. (If a
> referenced report cites a different "commit of record," note the divergence here and use the
> verified HEAD, not the report's string. If you then reassure Session 2 that a predecessor's
> findings carry forward *because the target files are unchanged between the report's commit and
> the new baseline*, that is a factual claim — verify it with a path-scoped `git diff --stat`
> before the Step 5 outline asserts it, per the Step 5 carry-forward check, never assert it
> from memory.)

If this brief **continues a prior one** (a follow-up to earlier research), name the predecessor
`reports/<...>-research-brief.md` and state what it already delivered, so Session 2 treats this
as a *delta* — not a cold start — and does not re-commission completed work.

Distinguish a **lineage predecessor** (a prior brief on the *same* line, framing the delta —
name it here in §1) from a **structural precedent** (a prior brief reused only as the *shape
model* — list it in §2 as a structural-model read, not a delta seed). A single pass can carry
both; keep their roles separate. When a repeat run overwrites a brief already at this slug,
record the overwritten predecessor's lineage here rather than leaving it implicit.

A third role: a prior brief's **deliverable** may enter the new brief as an **adjudication
input** — a report the new target commissions Session 2 to *reevaluate rather than obey*
(e.g. an external audit feeding an iteration pass). List the deliverable itself in §2 as a
primary read, fix its authority in §3 as a settled intention (advisory input; every
recommendation gets an independent, reasoned verdict), and place the commissioning brief in
§2's boundary-awareness tier for mandate context **when the input is a prior brief's deliverable;
when the adjudication input is instead a native repo artifact with no commissioning *brief* — a
trial/repair report, a snag log, an audit — the artifact's own framing supplies the mandate; and if
a separate native doc chartered that artifact (an iteration report's "next-iteration priority",
a trial charter — a non-brief driver), name that commissioner in §2's boundary-awareness tier for
mandate context, exactly the slot a commissioning brief would take. That boundary-awareness sub-step
is genuinely N/A only when no commissioner — brief or native — exists**. Provenance here in §1 stays one line.

A fourth role: a **completed-exemplar predecessor** — a prior brief *and* its delivered report whose
recommendations are **already adopted** into the fetch baseline, which the new target **generalizes
from** rather than continues or reevaluates (the repo's standard iterate-by-generalization move: a fix
landed on one surface, and this pass applies the same pattern to a new one). It is distinct from the
three roles above: not a *lineage delta* (the new target is a different surface, not the same line),
not an *adjudication input* (its findings are discharged, not up for re-verdict), and not merely a
*stale evidence input* (its findings are **definitely** overtaken by adoption, not *maybe*). Handle
it as: name it in §1 for provenance and say its recommendations already landed; list it in §2 as the
**model** read (what a good deliverable of this kind looks like); and pin it in §3 as an **out-of-scope
negative settled intention** — read as a completed record and a shape model, never as a live finding-set
to re-open or re-litigate. The exemplar is the model of the fix; the fix itself is done.

**A near-variant — the discharged-then-hardened predecessor.** Sometimes the predecessor's
recommendations landed into the *doc tree* and the new target **hardens the artifact they created** —
the *same* surface the fix produced, not a generalization to a new one (the completed-exemplar move
above), and not a re-adjudication of the recommendations (an adjudication input). The champion-vs-
challenger case is the type: a prior brief's Part 2 became a live guide section, and the new target
deepens *that section*. Handle it as a completed-exemplar for the *recommendations* (name them
discharged in §1; pin a §3 negative intention against re-litigating them; list the report as the §2
**model** read) **but** treat the *adopted artifact's home document* as a primary §2 read and the live
§3 hardening subject — the recommendations are done; the artifact they created is exactly what this
pass strengthens. Distinguish this from a plain lineage delta, where the predecessor's *research* (not
its adopted output) is what continues.

**Stale evidence input (older-baseline finding-set) — the inverse of the carry-forward check.**
When a §2 evidence input (a report, a prior trial's snag log) is pinned at a baseline *older*
than the fetch baseline, and intervening revisions may have **overtaken** its findings, do not
silently carry it as current. This is the mirror of the Step 5 carry-forward check: there you
verify that a predecessor's findings *still hold* before asserting it; here you flag that they
*may not*. Note the baseline divergence in §1, and carry a §3 settled intention instructing
Session 2 to **re-verify each of that input's findings against the current tree** and label
each — *survives / overtaken / partly-addressed* — rather than treating the older report as if
the intervening revisions never happened. (A path-scoped `git diff` between the input's baseline
and the fetch baseline over the touched files scopes which findings are even at risk; state it
plainly per the Step 5 discipline.) **Mixed-staleness inputs.** This fires even when the *primary*
finding-set under adjudication is current (at the fetch baseline) but a *secondary* evidence input
it leans on is stale: name the stale secondary input and its baseline gap, and instruct Session 2
to read it for its still-current evidence only — *not* as a live gap list — pointing the survives /
overtaken / partly-addressed labeling at just the findings the brief actually relies on. A single
current-state framing (e.g. "treat the changelog as the current state; do not re-commission landed
work") is weaker than this per-input labeling and can leave a locked Session 2 free to re-surface an
already-landed finding it reads in the stale secondary input.

When this brief has a **sibling or parallel brief** rather than a predecessor — another
session has already been commissioned, or will be commissioned, for an adjacent scope — name
it here and state the boundary in one sentence. The sibling brief is not incorporated by
reference: restate the owned vs. excluded scope in §3 and, if coordination affects the final
report, require a coordination section in §7.

**Greenfield / cold-start.** When exploration finds little existing structure, say so plainly
here and frame the task as *design from first principles*
grounded in the repo's stated purpose (its README) rather than a delta over prior work. There is
no shame in a short §2 — lean §5's mandate harder instead.

### 2. Read in full (authority order)

An explicit, ordered path list — every file Session 2 must read before producing — each with a
one-line reason it is load-bearing *for this target*. Built from Step 2 exploration. Order by
the repo's own authority tier if it defines one; otherwise root→detail (overview → architecture
→ specs → reference). Example shape:

```
Read these in full, in this order:

README.md — repo purpose and scope.
docs/<overview> — the design intent this target extends.
docs/<architecture-or-design file> — the subsystem contract this target depends on.
src/<module> — the code seam this target touches (name it; Session 2 reads it).
reports/<report> — prior finding-set this target builds on.
```

**Boundary-awareness reads.** When a scoped target must read adjacent docs *only* to know what
is **out** of scope, mark those entries as *boundary-awareness (read to bound scope, not a
conformance target)* — distinct from *primary (load-bearing)* entries. This stops Session 2
from auditing or "correcting" code the scope intentions exclude. Call out the primary entries
explicitly; group the rest with the boundary-awareness purpose stated once. Primary and
structural-model entries are literal paths; a boundary-awareness entry may name a whole
directory (e.g. `reports/<trial-folder>/**`) when the instruction is consult-as-needed and
nothing in it is individually load-bearing.

Use a separate subheading for directory globs, e.g. `Boundary-awareness / consult as needed`.
Do not place `**` directory entries under the primary read-in-full list. If a specific file
inside that directory becomes load-bearing, name that file literally in the primary or
structural-model tier instead.

**Conditional selectors.** When the originating target names a wildcard, directory, or semantic
selector conditionally (for example, "maybe `docs/mechanics/*` if needed"), expand it and state
the match count, but keep the primary read-in-full list limited to concrete files that are
load-bearing. Put the remaining matches under `Boundary-awareness / consult as needed` with the
conditional rationale. If exploration proves the whole tier is load-bearing, enumerate the
literal paths and use the whole-tier subject form below.

**Whole-tier subject (overhaul target).** When the primary subject *is* an entire doc tier —
a doc-overhaul where every file in a directory is load-bearing (§B's foundational/doc-overhaul
row) — enumerate the tier as **literal paths, not a `**` glob** (the glob prohibition above still
binds; the tier is the subject, not a consult-as-needed aside). For a large tier you need not
give each file a distinct one-line reason: state the tier's shared reason once, then add a
**hot-spots / priority map** (file → the specific finding or friction that lands there) that
concentrates Session 2's reading where the target actually bites. This is the *written-§2* form
of the Step 5 large-package economy — a tier enumeration plus a priority map, distinct from a
boundary-awareness directory glob (which names a directory precisely because nothing inside is
individually load-bearing). **Frame the hot-spots map as orientation, not a cage:** it concentrates
Session 2's attention where the target bites, but — especially when the target's own thesis is
*comprehensiveness* (a sweep that must not stop at the obvious) — it must not read as the *complete*
target list, or Session 2 reproduces the very shallow-sweep the overhaul exists to eliminate. Say in
§2 that the map is a starting orientation, not a substitute for reading the whole tier.

### 3. Settled intentions

The decisions the interview resolved — the heart of why Session 2 is *locked*. State each as a
committed decision, not an option. This section pre-empts every clarifying question Session 2
might otherwise ask. Carry any early-exit gaps here as `assumption: <X>` lines so they read as
defaults the user can override, not as open questions. Carry Step 5 residual gaps here the same
way; do not leave filename, placement, scope-depth, or evidence-depth assumptions only in an
approval-outline note or confidence/gaps preface.

When one brief carries **multiple distinct concerns** — two or more substantively different
threads under one target type (e.g. re-adjudicate a prior report *and* design a new methodology)
— split §3 and §7 by concern (3.A / 3.B …), state the shared disposition/bar once so it is not
repeated per concern, and carry a labeled `assumption:` on whether the deliverable stays one
report or splits. This keeps Session 2 seeing the concerns as parallel tracks, not one conflated
thread. It is orthogonal to the dominant-plus-secondary *type* classification: a single-type
brief can still carry several content concerns.

In a lineage brief, restate every predecessor intention that still binds (adapted to the new
target) — Session 2 must not need to open the predecessor brief to know its constraints; the
predecessor is named in §1 for provenance, not incorporated by reference.

When the interview settles a dimension **out of scope** — a decision the user has already made
elsewhere — state it as a *negative* settled intention: name the dimension, cite what settled
it, and instruct Session 2 not to re-open it; reinforce it in §6 as a scope guard. A locked
Session 2 will otherwise re-raise exactly what a prior pass flagged, so the exclusion must be
explicit, never implied by silence.

When a **ratified interview choice tensions with a §6 doctrine constraint** the brief will also
encode — the user picks a deliverable shape or scope that a repo invariant pushes against — do not
resolve the tension silently in either direction (neither override the user's choice nor quietly drop
the constraint). Encode the **reconciliation itself** as an explicit settled intention: name which
dimension the user's choice governs and which the constraint governs (e.g. *the choice fixes that
every item is covered; the constraint fixes the leanest form each item's treatment may take*), and
surface it at the Step 5 outline gate as a flagged item the user can override. A reconciliation carried
as a stated intention is auditable; one resolved by silent judgment is a guess Session 2 inherits
unexamined.

When a sibling or parallel brief owns adjacent work, make the division explicit as settled
intentions: what this brief owns, what the sibling owns, and when Session 2 should defer,
coordinate, or mark a finding as sibling-owned instead of duplicating the work.

When the target speaks of **multiple future sessions** ("the next sessions should…"), do not
invent a multi-session protocol: author for one executor per iteration and carry the default
as a labeled line — e.g. `assumption: one session executes this brief once; if the work is
split, each session works from this same brief and the report records the split` — so the
brief remains the shared contract however the user staffs it.

### 4. The task

A precise statement of what Session 2 must achieve — the goal behind the deliverable. One tight
paragraph. Name the target type (new spec/feature / thorny fix / hardening / overhaul).

### 5. Exploration + online-research mandate

Authorize depth explicitly:

> Explore the repository as deeply as needed beyond the files listed above. Research online as
> deeply as needed — similar implementations, research papers, prior art — wherever it sharpens
> the deliverable. Cite sources for any external claim that shapes a decision.

### 6. Doctrine & constraints

Pointers Session 2 must honor — **derived from this repo's own exploration, not imported from a
template.** Typical shapes (include only those the target actually engages):

- If the repo defines an authority order among its docs, state it: higher-tier docs govern
  lower ones; a genuine divergence requires amending the higher tier first, never designing
  against it silently.
- If the repo has an invariants / constitution / design-principles doc, name it and state that
  every product-behavior decision must satisfy it.
- Domain constraints the target engages — stated as this repo actually frames them, not assumed.
- Engineering constraints in force (e.g. no backwards-compatibility shims in new work, test
  coverage expectations) — only if the repo establishes them.
- **Recalled project memory is a constraint source, not just repo files.** Any recalled project
  memory that constrains the target — a standing limitation, a scope exclusion, an
  adjudication/quality standard, or a named tool's reliability — folds into the brief the same way
  exploration findings do: as a **§3** settled or negative intention where it fixes a decision, and
  as a **§6** constraint where it bounds the work (and §7 where it shapes the deliverable),
  cross-verified against repo evidence where possible, so the caveat travels in the brief even when
  exploration alone would not have surfaced it. Two shapes recur: (i) a **named external-tool**
  memory (e.g. ChatGPT-Pro for a remote-fetch brief) about that tool's reliability or limitations —
  e.g. a memory that ChatGPT-Pro's line citations are unreliable → the brief forbids `path:Lx-Ly`
  citations and requires heading-plus-anchor cites instead; (ii) a **standing project** memory — a
  solo-dev limitation the deliverable must not "fix" by assuming resources that don't exist, a
  scope the maintainer has ruled out, or a house adjudication standard — each landing as the
  matching §3/§6 line.

Trim to the constraints the target actually engages. If exploration finds no established
doctrine yet (greenfield), say so and let §5's mandate carry the design rationale instead of
manufacturing constraints.

### 7. Deliverable specification

Exactly what Session 2 outputs — leave no ambiguity:

- each **downloadable markdown document**, by filename and whether it **replaces** an existing
  file or is **new**;
- for replacements, name the file being replaced and what must be preserved vs. changed;
- if the deliverable is a **numbered/indexed spec**, derive its number/path from the repo's live
  spec index or roadmap (not from an archive), continue the visible sequence, and carry any
  residual placement ambiguity as a labeled `assumption:` line rather than asserting it;
- the **locked / no-questions** instruction, verbatim intent:

> Produce the deliverables directly as downloadable markdown documents. Do not interview, do not
> ask clarifying questions — the requirements above are final. If a genuine contradiction makes
> a requirement impossible, state it in the deliverable and proceed with the most faithful
> interpretation.

**Determination-plus-conditional targets.** When the research target is "decide whether X is
needed, and *if so* produce X" (common for hardening passes), the deliverable is contingent on a
judgment Session 2 must make first. Do not leave the contingency implicit. The brief must (a)
instruct Session 2 to produce a clearly labeled, evidence-based **determination / verdict** ("is
X warranted, and why"), and (b) state — as a settled intention resolved in the interview — which
of **three** modes governs the artifact: (i) **unconditionally** (one always-produced document
with the verdict embedded as a section); (ii) **only if the verdict is positive** (nothing
authored on a negative verdict — the reasoned verdict is still surfaced as Session 2's response,
but no file); or (iii) **always produce, form follows the verdict** (one document is always
produced, but its *shape* depends on the verdict — e.g. a full spec if warranted, a standalone
rationale report if clean; this mode needs both Branch A / Branch B shapes specified in §7).
Prefer (i) when the artifact's value survives a negative verdict (it locks already-correct
properties); choose (iii) when a negative verdict still warrants a substantial document in a
*different form*; reserve (ii) for when a negative verdict means there is genuinely nothing to
author.

**Analysis / recommendation report (not a ratified artifact).** When the deliverable is a
consolidated report of *recommended changes* rather than a ratified spec/doc — the recurring
output of a doc-overhaul pass or a hardening pass whose ask is "what to change + where" — say so
explicitly, and specify the report(s) by filename, **new** (not a replacement). Direct Session 2
to deliver **substance + home, not ratified text**: for each finding, *what the target doc must
own* (Session 2's own prose, at the right altitude) and *which file* it lands in (new section /
addition / correction) — explicitly **without** final paste-ready wording or invented
identifiers, which remain the repo's own reassess/amend process. Carry a labeled `assumption:`
line if the report's filename or placement is not pinned in the interview. **This provision
composes with the determination-plus-conditional modes above:** a recommendation report can
itself carry a mode-(i) embedded **determination/verdict** — an always-present report section
stating a go/no-go (e.g. adopt-now-vs-defer) — rather than a separately-authored conditional
artifact. When the target asks for both "what to change + where" *and* a verdict on *whether* to
change, specify the verdict as a required section of the report.

If a sibling or parallel brief owns adjacent scope, add an explicit deliverable section for
coordination: which findings are independent, which should wait for or be reconciled with the
sibling deliverable, and which are out of scope because the sibling owns them. This is a report
section, not permission to re-run the sibling's research.

### 8. Self-check

A short acceptance checklist Session 2 runs against its own output before returning — e.g. every
replacement preserves the load-bearing content of the original; no new doctrine weakens an
upstream doc tier; every external claim is cited; the deliverable set matches §7 exactly; the §1
fetch-baseline commit contains every file named in the §2 read-in-full list; no `**` directory
glob appears in the primary read-in-full tier; sibling-owned scope is not duplicated.

Derive every self-check item from a §3 intention, §6 constraint, or §7 requirement. An item that
introduces a new obligation — or tenses against a settled intention (e.g. an intention forbids
churn that a check item would force) — signals the intentions need restating before the brief
ships, not that Session 2 should reconcile the conflict itself.

---

## B. Target-type → load-bearing reads

A starting map for §2; always refine against Step 2 exploration. Derive the repo's current doc
taxonomy and authority order at run time — from the root `CLAUDE.md` and the target tree's own
README — then use this map for the *kinds* of reads to seek out, resolved against the actual tree.
`README.md` (and any repo-wide invariants/design-principles doc) is load-bearing for every type.

| Target type | Load-bearing reads to seek out (beyond README) |
|---|---|
| **new-spec / new-feature** | the architecture/design doc for the touched area; any spec index, roadmap, or ledger; sibling specs/features; the existing code seams for that area. |
| **thorny-fix** | the design contract for the affected area; the relevant code seams; any report/issue notes touching the defect; the tests covering it and the acceptance the fix must still satisfy. |
| **hardening** | the invariants / constraints the system must uphold; the subsystem's design + code seams; prior hardening notes or reports; the validation / test coverage. A hardening pass can deliver *either* a recommendation report or a numbered implementation spec — the ask decides (§7); for a spec deliverable, union in the **new-spec** row's reads via a `(secondary: new-spec)` classification. |
| **foundational / doc-overhaul** | the doc tier being overhauled plus every tier above it in the repo's authority order (authority flows downward); the doc index / authority map; cross-references in lower tiers that the overhaul will invalidate (read as **boundary-awareness** to run the tier-fit test — what belongs at this altitude vs. elsewhere — and route out-of-scope findings forward rather than amending them here). |
| **other** | derive entirely from exploration; default to README plus whatever the target names. |

---

## C. Local-session executor adaptations

When Step 1 classified the executor as **local-session** (Session 2 is another Claude/Claude
Code session with direct repo read/write access), apply these deltas to §A. Everything not
named here — the eight-section anatomy, settled intentions, locked/no-questions, the
self-check discipline — carries over unchanged.

**Preamble.** Replace the "paste this prompt / upload the manifest" framing: state plainly
that Session 2 is a local session (name the model if the target named one) with direct
access to the repository working tree — it reads files with its own tools, fetches nothing,
and uploads nothing.

**§1 Context — authored-against baseline, live-tree semantics.** There is no manifest
pointer and no fetch-from commit. Instead, pin the commit this brief was *authored against*
(verified repo HEAD at write time) and instruct Session 2 to verify HEAD when it starts
(`git rev-parse HEAD`), work from the live working tree, and note any divergence from the
authored-against baseline in its deliverable. The pin exists for honest provenance, not for
fetching. If the repo's own files carry provenance records shaped by a previous remote-fetch
workflow (fetch ledgers, stale `source_commit` strings), instruct Session 2 to rewrite them
as an honest local-session record rather than carry them forward.

**§2 Read in full.** Unchanged in substance — the authority-ordered path list with one-line
reasons. Paths are repo paths Session 2 Reads directly. The §2-completeness check still
applies; the untracked-input problem relaxes (a local session *can* read an untracked file,
but every untracked or working-tree-divergent §2 path must be labeled individually or in an
exact grouped path list, since untracked entries are invisible in git history and divergent
tracked entries differ from the authored-against baseline).

**§5 / §6.** Unchanged — a local Claude session can research online; the mandate and
doctrine sections apply as written.

**§7 Deliverable specification — files, not downloads.** Replace "downloadable markdown
documents" framing: deliverables are files Session 2 Writes or edits in place, named by
repo path. State explicitly whether Session 2 commits or leaves the working tree for the
user's review (settle this in the interview; default: leave uncommitted). Scope precisely
what it may and may not touch.

**§8 Self-check.** Recast fetch-oriented checks as working-tree checks (e.g. "every §2 path
was read", "this session modified nothing outside the named scope", "nothing was committed"
when uncommitted is the settled default). Scope tree checks to the session's *own actions*: a
live working tree is shared, so instruct Session 2 to verify against its own edit log rather
than raw `git status`, and to surface — not own — pre-existing or concurrent changes made by
other writers during the run.

---

## D. Remote-fetch (inline) executor adaptations

When Step 1 classified the executor as **remote-fetch (inline)** — a ChatGPT-Pro-style
deep-research session against a repository it cannot fetch (private without executor access, or otherwise unreachable) — apply
these deltas to §A. Everything not named here — the eight-section anatomy, settled intentions,
locked/no-questions, the self-check discipline — carries over unchanged. The working precedents
for this shape are `reports/simulation-verdict-statistics-hardening-research-brief.md` and
`reports/continuation-verdict-authority-research-brief.md` (both sanitized-restatement), plus
`reports/human-tier-measurement-research-brief.md` — the working precedent for the
verbatim-anchored posture (its `Appendix A` carries exact excerpts inline).

**Preamble and paste boundary.** The brief opens with a one-line repo-side provenance header
(author date, authored-against commit, what reservation or target it feeds) marked as outside
the payload, plus "paste everything from §1 down." §1 then states plainly: the client's
methodology is private; the executor has **no access to the repository, code, or documents —
this is deliberate**; everything required is inlined in §2; do not ask for files, and none are
needed.

**§1 Context — provenance pin only.** No manifest pointer and no fetch-baseline commit. The
authored-against commit lives in the repo-side header for the repo's own records; the payload
never instructs fetching.

**§2 becomes the inlined material.** Retitle it in the spirit of *"Inlined, self-contained
material (read this; there is nothing else to read)"* and open with "You will read no files."
Every load-bearing input travels inside it, at a **sanitization posture the interview settles**
— this is the channel's canonical Step-4 question, an IP-exposure call only the user can make:
(a) **sanitized restatement** (the default): domain-general re-description of
the established machinery ("already established — do NOT re-derive"), the problem, and the
evidence (case studies under neutral labels), with no verbatim repo prose; or (b)
**verbatim-anchored** (the human-tier precedent): the same substance plus exact excerpts where the deliverable must fit a
specific printed interface, each marked inlined-verbatim. Repo paths, if mentioned at all, are
marked provenance-only. Delta framing survives sanitization: name what prior research lines
already delivered — as adopted machinery — without exposing them.

**Step 5/6 handling.** The read-in-full presentation and path-resolution checks collapse to an
**inline-completeness check**: every load-bearing input identified in exploration is either
inside the brief or deliberately excluded with the exclusion stated. No manifest is written
(Step 6 item 2 is skipped always); the §2-completeness check runs against the brief's own
inlined sections; and the post-write coherence check gains two items — the paste boundary is
stated, with everything below it self-contained; and every inlined-verbatim block is verified
against its source at write time (grep/diff the quoted spans against the source file,
fragment-wise or newline-normalized — brief and source both hard-wrap prose, so a full-phrase
grep that misses on a wrapped span is a wrap artifact to re-derive, never yet a drift verdict): mark
every elision, and never join text from two sources under one quote — re-mark such material as
restatement instead. On this channel a transcription drift does not self-heal: the excerpt *is*
the ground truth the executor sees.

**§5 mandate.** "There is no repository to explore; §2 is the whole input. Research online as
deeply as needed…" — the online half carries the full weight.

**§7 / §8.** Deliverables stay downloadable markdown documents (the user ferries them back
into `reports/`). Self-check items recast fetch checks as self-containment checks: every
internal §-reference resolves, no file or repo access is requested, every load-bearing
external claim is cited per the brief's citation discipline, and nothing in the deliverable
presumes repository access.

**Step 7 closeout.** The written file is the brief alone; the leftover-manifests item states
that this brief pairs with **no** manifest and that none of the accumulated manifests may be
uploaded with it; the locked reminder is paste-only (the payload from the stated boundary
down, upload nothing); and the benign-dirty note needs no manifest rationale — there is no
fetch baseline to invalidate, only the provenance pin.
