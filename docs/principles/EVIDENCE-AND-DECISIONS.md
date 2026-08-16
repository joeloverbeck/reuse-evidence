# Evidence and Decision Principles

**Status:** Adopted topic principle  
**Governed by:** [`FOUNDATIONS.md`](FOUNDATIONS.md)

This document defines the semantic lifecycle. It does not require one particular event schema, database, command spelling, or user interface.

## 1. What evidence is for

Evidence exists to support a live reuse decision. It is not a completeness archive, activity ledger, developer score, or certification program.

A case should preserve only the facts another session needs to understand:

- what responsibility may be repeating;
- which independent consumers required it;
- what concrete artifacts support each occurrence;
- what similarities and differences matter;
- what decision was proposed and accepted;
- what consequences were authorized;
- and what verification remains or has completed.

## 2. Responsibility identity

Two implementations belong to one proposed responsibility only when the evidence supports a coherent common owner. Review must examine:

- invariant behavior;
- legitimate variation;
- authority and source of truth;
- triggers and lifecycle;
- invalidation and retry rules;
- side-effect class;
- compatibility and release obligations;
- reasons to change;
- privacy and trust boundary;
- and the consumer-facing contract.

Similarity without these facts is a candidate, not a case conclusion.

## 3. Occurrences

An occurrence is one independently accepted reuse-consumer need for a responsibility, arising from real work on a primary artifact.

### Normally one occurrence

- several generated copies from one template;
- retries or continuations of one task;
- coordinated variants created together for one consumer;
- production and test code that jointly implement one consumer contract;
- a copied implementation whose only purpose is temporary migration within one accepted change;
- several call sites of an already shared implementation.

### Potentially several occurrences

- independently developed games that each need the responsibility;
- separate packages with independent release or compatibility obligations;
- distinct repositories that would otherwise maintain their own implementation;
- different subsystems with separate owners and reasons to change;
- a Rust and TypeScript implementation of the same cross-language contract.

Repository count is evidence, not the definition. One repository may contain several independent consumers; several repositories may still be one coordinated consumer.

## 4. Admissible evidence

Prefer recoverable primary artifacts:

1. accepted specifications, issues, or task definitions;
2. Git commits, trees, diffs, and history;
3. source and public interfaces;
4. tests and fixtures that state the consumer behavior;
5. release or compatibility contracts;
6. prior case events and accepted decisions;
7. external package documentation and source when alternatives are reviewed;
8. sensor reports retained by reference.

A contemporaneous session may use already-loaded context to formulate the proposal, but the durable case must point to recoverable artifacts. Memory alone is insufficient for retrospective occurrence evidence.

Generated reports are secondary evidence. They must not be allowed to cite one another until the original source disappears behind a chain of prose.

## 5. Clean capture

When bounded capture finds no plausible qualifying repeated responsibility:

- write no case event;
- create no report;
- create no first-use inventory;
- return a fixed terminal statement confirming that capture completed without a qualifying candidate.

Certainty that capture ran comes from the terminal result, not from a committed denominator of clean sessions.

## 6. Case opening and readiness

### First occurrence

No case exists. The implementation may remain local and replaceable.

### Second independent occurrence

A case may open in `watching` state. Opening should reconstruct the first occurrence from recoverable evidence and record both consumers. The purpose is memory, not pressure to extract.

When the relationship is too uncertain to survive as a useful case, record nothing and report the ambiguity.

### Third independent occurrence

The derived state normally becomes `review-ready`. This authorizes spending effort on a semantic decision. It does not choose the outcome.

### Early-review override

The human may authorize review after the second occurrence when a concrete reason is recorded, such as:

- coordinated bug fixes have already been required;
- divergence threatens a published or security-sensitive contract;
- both implementations are already materially expensive;
- a third accepted consumer is imminent rather than hypothetical;
- or the first two consumers have exposed enough real variation to make the decision less speculative than the count suggests.

The override must name why waiting is worse, the evidence bearing that claim, and the bounded review appetite. It cannot directly authorize extraction.

## 7. Review questions

A reuse review must answer, as far as the evidence allows:

1. Do the occurrences actually share one responsibility?
2. What exact behavior or contract is invariant?
3. Which differences are legitimate variation rather than accidental divergence?
4. Do the consumers change for the same reasons?
5. Who should own the shared authority and lifecycle?
6. What coupling, dependency direction, release, migration, privacy, or compatibility costs would sharing create?
7. Does an existing package, crate, standard, schema, or upstream project already own the responsibility adequately?
8. Is generation or a versioned contract better than one runtime implementation?
9. What is the narrowest valid scope?
10. Would the current shared abstraction be better split rather than extended?
11. What evidence would falsify the proposed decision?

Review should state uncertainty. It must not inflate weak similarity into a confident architecture recommendation.

## 8. Permitted decisions

Use orthogonal fields in the durable decision rather than one ever-growing status enumeration.

### Identity verdict

- same responsibility;
- different responsibilities;
- insufficient evidence;
- existing abstraction is wrong.

### Action

- retain intentional duplication;
- wait for more evidence;
- use an existing dependency;
- extract or deepen locally;
- create a workspace package;
- create a private cross-repository package;
- publish a public package;
- centralize a schema, specification, or fixture corpus;
- replace copies with generated artifacts;
- contribute missing behavior upstream;
- split, inline, or narrow an existing abstraction.

### Lifecycle

- proposed;
- accepted;
- implementing outside the reuse lifecycle;
- awaiting verification;
- verified and closed;
- parked;
- reopened.

The exact accepted decision must also name scope, non-responsibilities, affected consumers, compatibility consequences, and verification conditions. A decision whose action authorizes implementation must additionally name its migration expectations and rollback or re-splitting path; retain intentional duplication and wait for more evidence omit both.

Amended 2026-08-16. The #46 fixture walkthrough drove one accepted decision on each branch of the action list above and found that this sentence required migration expectations and a rollback or re-splitting path from decisions whose action authorizes neither. The compiled surface has always refused them there and `tests/case_cli.rs` pins it, so this records the existing rule rather than changing behaviour. The rest of this section is intact, and no field changes owner: ADR 0012 records the same carve-out for the three implementation-shaped fields it owns and remains the sole home of alternatives rejected, while `CONTEXT.md`'s **Reuse decision** entry carries the matching glossary correction. The decision owner accepted this amendment.

## 9. Existing dependencies

The threshold doctrine governs when this portfolio should create and own a new abstraction. It does not require local duplication when a mature external dependency already satisfies the first consumer.

Before recommending a new public Rust crate, review must search crates.io and inspect plausible alternatives for:

- functional fit;
- authority and abstraction boundary;
- compatibility and maintenance burden;
- license;
- release stability;
- transitive cost;
- and whether a narrow upstream contribution would be better.

Equivalent ecosystem research may be required for TypeScript or another language when the live decision needs it. Package search is decision-bound research, not a mandatory capture step.

## 10. Implementation handoff

An accepted change decision produces a bounded implementation brief containing:

- the accepted responsibility identity;
- evidence-bearing consumers;
- invariant contract;
- explicit non-responsibilities;
- chosen home and scope;
- alternatives rejected;
- existing packages considered;
- required consumer-level tests;
- compatibility and release consequences;
- migration order;
- rollback or re-splitting strategy;
- and verification commands or conditions.

The brief is a durable result, not a peer-skill routing instruction and not proof that implementation occurred.

## 11. Verification and closure

A case closes only when evidence shows that:

- the accepted shared or separated surface exists;
- every named consumer migrated or has an explicit accepted exception;
- consumer behavior remains correct through the relevant public interfaces;
- forbidden dependency directions or privacy leaks were not introduced;
- compatibility and release obligations were handled as accepted;
- and the resulting abstraction still owns the responsibility that review authorized.

A verification failure reopens or parks the case. It does not rewrite the accepted historical decision as though it never happened.

Later consumer pressure may reopen a closed case, including to split a previously accepted abstraction.

## 12. Legacy discovery

Legacy repositories may be searched to find candidate pressure, but repeated shapes in a corpus are hypotheses only. Discovery output remains temporary until the human selects a candidate and admissible evidence establishes independent consumers.

Existing files do not create a migration backlog. A historical case does not compel retrofit unless a live decision accepts it.

## 13. Metrics

Counts may help orient a human: occurrences, watching cases, review-ready cases, stale references, or awaiting-verification cases. They are not a health score and must not be presented as proof that a repository is clean, reusable, well-architected, or improving.
