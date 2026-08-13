# Capability and Workflow Boundaries

**Status:** Adopted topic principle  
**Governed by:** [`FOUNDATIONS.md`](FOUNDATIONS.md)

A capability owns one coherent domain result and lifecycle. Boundaries follow authority and independently changing decisions, not a convenient sequence of workflow stages.

## 1. Domain substrate

The domain substrate contains:

- authoritative case events;
- evidence references;
- exact human-accepted decisions;
- unresolved decision state;
- verification results;
- and declared ownership, revision, and privacy.

It does not contain a duplicate copy of repository code, a full session transcript, a general backlog, or hand-authored routing state.

## 2. Planned capability set

### `reuse-evidence-capture`

**Result:** a bounded factual proposal to open or append a reuse case, or a fixed no-candidate terminal result.

Capture:

- is manually invoked after material implementation work, normally after code review;
- is bounded to the completed work and likely prior occurrences;
- inspects the diff, accepted task, tests, source, existing cases, and enrolled portfolio as needed;
- may use optional sensor evidence;
- establishes plausible independent consumer pressure;
- and writes only through the compiled command after the human authorizes the exact event.

Capture does not:

- decide the abstraction;
- recommend extraction merely because a threshold was reached;
- implement a refactor;
- write clean receipts;
- inventory first uses;
- or launch a broad architecture audit.

### `reuse-evidence-discover`

**Result:** a temporary read-only candidate set for human selection.

Discovery searches one repository, a bounded set, or the enrolled portfolio for:

- the same responsibility appearing in at least two independently maintained consumer contexts;
- or an existing abstraction serving consumers that now change for different reasons.

Discovery does not report general shallowness, coupling, naming, large modules, missing interfaces, or speculative framework opportunities. Those belong to architecture review.

Nothing becomes durable merely because discovery found it. The human selects a candidate; capture or review then establishes admissible evidence.

### `reuse-evidence-review`

**Result:** an inspectable proposed reuse decision and, when accepted, a bounded implementation brief.

Review owns semantic identity, scope, alternatives, package research, non-responsibilities, migration expectations, privacy consequences, and verification conditions.

Review may use architecture vocabulary or consult external design skills, but it does not implement the change.

### `reuse-evidence-status`

**Result:** a read-only projection of enrolled repositories and case lifecycle.

Status may report watching, review-ready, parked, awaiting-verification, stale, unavailable, privacy-conflicted, closed, and reopened cases. It must not produce a portfolio quality score or reprioritize work.

## 3. Compiled command surface

The Rust command owns mechanics that must not depend on agent discretion:

- marker and schema validation;
- repository identity;
- event validation;
- expected-revision checks;
- atomic and idempotent writes;
- privacy enforcement;
- derived state;
- installer conflict safety;
- and inspectable terminal receipts.

The command does not decide whether two implementations share a responsibility. Semantic judgment remains in review and human acceptance.

## 4. Manual invocation and hooks

Version 0.1 uses manual invocation. The maintainer values knowing that capture actually ran, while automatic hooks risk noisy, premature, or hidden evidence creation.

No Stop hook, post-commit hook, daemon, or background scan may create case evidence automatically.

A later reminder-only hook may be considered when real missed-capture evidence justifies it. A reminder must not scan broadly, write evidence, or imply that every coding session requires capture.

## 5. Conversation and transcripts

A contemporaneous agent may use current conversation context to understand the task, but durable case claims must reference recoverable repository artifacts.

Version 0.1 has no Claude Code or Codex transcript parser. Transcript formats are not a stable authority boundary, and parsing them would create broad integration work before the primary lifecycle is proven.

## 6. External sensors

A sensor may propose candidate pairs or clusters and preserve a report reference. Sensors may be exact, AST-based, embedding-based, cross-language, or cross-project.

Sensor use is optional. The core capability must work through ordinary Git, source inspection, search, and agent reasoning when no sensor exists.

No sensor result may:

- open a case without human-authorized evidence;
- establish independence;
- choose a decision;
- fail CI by default;
- or expose private material remotely without separate authority.

## 7. External engineering workflow

Accepted reuse decisions leave the reuse lifecycle for implementation.

Ordinary engineering capabilities own:

- interface and seam design;
- specifications and tickets;
- test-driven implementation;
- migration mechanics;
- code review;
- and commits or pull requests.

The maintainer currently uses high-quality external skills such as `codebase-design`, `tdd`, `implement`, `code-review`, and `improve-codebase-architecture`. `reuse-evidence` should interoperate through durable implementation briefs, not copy those skills or depend on their package layout.

After implementation, the reuse lifecycle resumes for independent verification and closure.

## 8. Boundary with architecture review

Architecture review asks where a repository would benefit from deeper modules, better seams, or less coupling.

Reuse discovery asks whether independently maintained consumers are repeating one responsibility or whether a shared abstraction has ceased to be one responsibility.

A finding that lacks repeated consumer evidence belongs to architecture review, not this project. A reuse case may later require architecture design, but the disciplines remain distinct.

## 9. Minimal control

Capabilities return durable results, blockers, decision requests, or unmet evidence needs. They do not hand-author peer routing or suspend themselves inside prose waiting for another skill.

Any controller should derive enabled work from case state and user intent. It may own retry, deduplication, resumption, and termination, but it must not accumulate semantic reuse judgment.

Version 0.1 should use direct user invocation and the smallest compiled state transitions. It does not need an orchestration platform.

## 10. Human waits

Before asking for a consequential human decision, persist or present:

- the exact proposal;
- evidence references;
- current case revision;
- privacy consequence;
- allowed decision surface;
- and the effect each option would authorize.

Application occurs only against the accepted proposal and expected revision. Verification is separate from acceptance.
