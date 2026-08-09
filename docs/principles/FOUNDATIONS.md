# Reuse Evidence Foundations

**Status:** Adopted foundational authority  
**Decision owner:** Repository maintainer  
**Adopted:** 2026-08-09

## Constitutional path

Human intent → bounded evidence capture → mechanical and semantic checks → inspectable reuse proposal → human decision → externally implemented change → independent verification → authoritative case state

Structural validity never equals semantic acceptance. Similarity never equals shared responsibility. A completed implementation never silently proves that its abstraction was warranted.

## Mission

`reuse-evidence` exists so a maintainer can make timely, inspectable decisions when independently maintained consumers are accumulating the same responsibility, before parallel implementations become expensive divergence.

The primary outcome is the **accepted reuse decision** and its verified consequence, not the scan, event stream, report, crate, skill, index, or workflow used to reach it.

A successful use may conclude that the code should remain duplicated, wait for more evidence, use an existing dependency, extract at a narrow scope, centralize a schema or generator, create a private or public package, contribute upstream, or split a wrong abstraction. The system succeeds when it improves the decision—not when it maximizes extraction.

## Human-selected delivery constraints

The maintainer has selected these constraints for the initial repository:

- the project is a public Rust crate and standalone CLI;
- it operates local-first across explicitly enrolled public and private repositories;
- it installs agent skills as repository assets, with `.claude/skills/` as the real location and `.agents/skills/` as discovery links;
- durable evidence does not depend on conversation transcripts;
- consequential decisions require human acceptance;
- accepted refactors are implemented by the repository's normal engineering workflow, not by the reuse lifecycle itself.

These constraints authorize a bounded implementation path. They do not prove any particular event schema, database, command tree, detector, plugin system, or abstraction kernel.

## Current active value stream

The active bottleneck is proving the smallest real end-to-end lifecycle:

1. explicitly enroll real repositories;
2. capture a real second independent occurrence;
3. derive ordinary review readiness from a third occurrence or a narrowly accepted early-review reason;
4. produce and accept an exact decision;
5. hand implementation to ordinary engineering work;
6. verify the accepted result;
7. preserve private evidence correctly;
8. keep ordinary no-candidate capture cheap and write-free.

Work that does not materially advance or falsify this path is parked. In particular, detector development, hosted services, automatic hooks, plugin frameworks, broad legacy migration, generalized package recommendation, and shared lifecycle extraction are not active workstreams.

## Foundational principles

### 1. Human semantic authority

The human is the semantic authority. Agents, scanners, tests, statistics, package searches, and review reports produce proposals and evidence. They do not accept a responsibility identity, abstraction, migration, publication, or closure.

A merge, passing test, published release, closed issue, or generated report is not human acceptance.

### 2. Outcome before mechanism

State work in terms of the maintainer effect: a better-timed, better-grounded reuse decision with acceptable operational burden. A preferred tool or architecture is replaceable unless the human has selected it as a binding constraint.

Existing repositories, prior tools, popular skills, detector output, and legacy code provide hypotheses and hard cases. They do not grant authority to an inherited workflow, schema, repository boundary, or abstraction.

### 3. Reuse after real repetition

A reusable layer normally emerges from repeated pressure in real primary work. The first implementation may be clean, local, replaceable, and deliberately uncommitted to a product line.

Repeated shapes in a legacy corpus, generated output, one coordinated implementation episode, or predicted similarity among future products do not establish independent pressure.

### 4. Responsibility before code shape

The object of judgment is a coherent responsibility: an authority, representation, policy, contract, or independently changing decision. Similar text, AST shape, names, call patterns, or workflow stages are clues only.

Keep implementations separate when ownership, triggers, invalidation rules, compatibility, release policy, side effects, or reasons to change differ. Share only the invariant behavior whose authority can be owned coherently.

### 5. Independent consumers are the evidence unit

Count independently accepted consumer needs, not files, functions, sessions, retries, or copies. A distinct repository may still be the same consumer context; several modules in one repository may be genuinely independent consumers. The evidence must establish the distinction.

### 6. The second occurrence remembers; the third normally reviews

A first occurrence creates no reuse case. A second independent occurrence may open a watching case so the pressure is not forgotten. A third independent occurrence normally makes review worthwhile.

The threshold controls the cost of **designing and owning a new abstraction**. It does not forbid an existing mature dependency on a first consumer, and it does not make extraction mandatory at the third.

A human may authorize review after the second occurrence only for a concrete, recorded risk or cost. The override authorizes review, never automatic extraction.

### 7. Review and implementation are different responsibilities

Reuse review decides whether consumers share a responsibility and what the narrowest valid disposition is. It may produce an implementation brief.

Interface design, TDD, implementation, migration, and code review belong to ordinary engineering capabilities. `reuse-evidence` returns afterward to verify the accepted decision. It must not grow a parallel implementation workflow merely because it can describe one.

### 8. Claim-sized evidence

Evidence authorizes only the claim it bears. A detector score can support candidate discovery; a diff can support what changed; tests can support named behavior; a package search can support what alternatives were examined. None of these alone proves a shared responsibility or a good abstraction.

A stack of disconnected demonstrations cannot establish framework, product-line, portfolio, or ecosystem fitness. Those claims require representative end-to-end use on named real consumers.

### 9. The narrowest valid reuse scope wins

Review considers local helper, module, workspace package, private shared package, public package, generator, versioned contract, existing dependency, upstream contribution, intentional duplication, deferral, and de-abstraction.

Choose the narrowest scope that creates real leverage while preserving authority, privacy, compatibility, and independent change. Public extraction is not inherently better than local reuse.

### 10. Private-first portfolio safety

Enrollment is explicit. One private participant makes the complete case private. Private repository identity, paths, source, and evidence do not enter public case state or public extraction history.

The tool operates locally by default. Network use requires a distinct accepted capability and a precise disclosure boundary; it is not implied by package search or public distribution.

### 11. Durable, inspectable continuity

Conversation may help produce evidence, but it cannot be the only record of occurrences, decisions, approvals, pending verification, or return points.

Authoritative case facts and accepted decisions must be versioned and inspectable. Derived indexes and status projections must be rebuildable. Side effects require narrow authority, expected-revision protection, idempotency, and an inspectable receipt.

### 12. Control records are not a second domain

Record only decision-bearing facts that must survive: evidence references, occurrence identity, accepted decisions, effects, provenance, and verification. Generate routing, status, hashes, and projections mechanically where possible.

Do not create a denominator of clean sessions, first-use inventories, or hand-authored certification forms. A clean capture writes nothing.

### 13. Optional sensors, semantic judgment

External tools may find exact, structural, semantic, cross-language, or cross-project candidates. Their output is optional sensor evidence.

The core product must not equate a score with identity, require one detector, or build a detector before the evidence lifecycle itself proves useful. Sensor integration should remain minimal until repeated real integrations force a stable boundary.

### 14. Wrong abstractions are valid findings

The system must be able to conclude that an existing abstraction should be split, inlined, or narrowed. Restoring duplication can be correct when consumers have diverged in responsibility or lifecycle.

A system that only promotes sharing will eventually manufacture the failure it was created to prevent.

### 15. Operational fitness is part of correctness

The maintainer must be able to run capture, inspect a case, make a decision, understand private consequences, resume later, and verify closure at acceptable burden.

A recurrent workflow the maintainer will avoid is unfit even when its data model and tests are correct. Routine clean capture should be one manual invocation, bounded investigation, a fixed terminal result, and no committed artifact.

### 16. Public distribution does not prove a public ecosystem

Publishing the repository or crate makes distribution possible. It does not establish adoption, community demand, support obligations, a hosted service, plugin marketplace, or generalized multi-user governance.

External requests may become evidence, but no public ecosystem is assumed before it exists.

## Authority, adoption, and experiments

Approval to implement a speculative mechanism authorizes an experiment, not its adoption as permanent architecture. Adoption requires the human to inspect a representative end-to-end result and explicitly accept the exact premise and scope.

If an experiment fails, exhausts its appetite, or remains unadopted:

- park or reject it explicitly;
- remove its inactive machinery from default context and the active dependency graph;
- rejustify any survivor independently;
- preserve only concise evidence needed to avoid repeating the same decision blindly;
- and allow a clean restart without inherited technology choices.

Legacy repositories or historical cases do not create a migration backlog merely because they are available. Migration requires an observed need or explicit human objective.

## Hard prohibitions

Claude Code, Codex, and future contributors must not gradually erase these boundaries:

- no automatic refactoring from a detector result, occurrence count, or review-ready state;
- no third-occurrence rule presented as extraction authority;
- no general code-quality, architecture-health, DRY, or portfolio score;
- no first-use case inventory and no clean-run evidence stream;
- no broad architecture audit disguised as reuse discovery;
- no CI failure for unreviewed similarity candidates;
- no built-in clone detector before a separate accepted decision grounded in real failure of external sensors;
- no mandatory detector, embedding model, GPU, network provider, or external API for the core lifecycle;
- no scan of unmarked repositories merely because they are co-located;
- no central mutable ledger, hosted service, daemon, or background agent before a bounded fault-tested need exists;
- no private evidence in public state and no public steward for a mixed-visibility case;
- no product-line framework inferred from several games, worlds, stories, or repositories that merely resemble one another;
- no shared Rust lifecycle kernel extracted from `reuse-evidence` and `skill-evidence` until a third independent consumer creates real pressure;
- no transcript parser treated as authoritative evidence;
- no orchestration platform or peer-skill routing graph beyond the smallest durable handoff required;
- no hand-authored routing, provenance, status, or certification paperwork where it can be generated;
- no representation burden counted as primary progress merely because the representation created the burden;
- no speculative compatibility implementation beyond cheap reversible preservation without a real consumer;
- no inferred secondary objective displacing the active primary path without recorded human reprioritization;
- no accepted abstraction protected from later evidence that it is wrong.

## Amendment rule

A PRD, issue, skill, schema, code change, test, report, or release cannot silently amend this document. Amend the foundation first, with explicit human acceptance and a named evidence-bearing reason, then update dependent authority in order.
