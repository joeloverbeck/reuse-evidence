# Foundational Principles

These documents are the repository's constitutional authority. They define what `reuse-evidence` is for, what evidence can authorize, who decides, how private repositories participate, and which responsibilities this repository must refuse.

## Conformance rule

Every ADR, design document, PRD, issue, skill, schema, command, event, test, report, and implementation change must either:

1. conform to these principles and all accepted ADRs; or
2. amend the conflicting higher authority first through an explicit human decision.

A merge, passing test suite, closed issue, published crate, generated report, or agent assertion does not amend or prove conformance. When a conflict appears, the lower-level work pauses until the authority conflict is resolved.

## Authority within this directory

[`FOUNDATIONS.md`](FOUNDATIONS.md) governs the topic documents. The topic documents specialize it without weakening it:

- [`EVIDENCE-AND-DECISIONS.md`](EVIDENCE-AND-DECISIONS.md)
- [`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md`](PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md)
- [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](CAPABILITY-AND-WORKFLOW-BOUNDARIES.md)
- [`CONSUMER-CONTRACT.md`](CONSUMER-CONTRACT.md)

If two topic documents appear to conflict, prefer the reading that preserves `FOUNDATIONS.md`, human authority, claim-sized evidence, private safety, and the narrower capability boundary. Record a clarifying amendment when the ambiguity could affect real work.

## Amendment discipline

A foundational amendment must:

- name the live decision or observed failure requiring it;
- distinguish the intended consumer effect from the proposed mechanism;
- cite the real executions or accepted artifacts that justify changing authority;
- state what existing rule is being changed and what remains intact;
- identify dependent ADRs or designs that must be updated;
- remain bounded to the evidence;
- and receive explicit human acceptance.

Do not accumulate speculative constitutional clauses for imagined future integrations. Park uncertain possibilities outside the normative authority surface until real consumer pressure makes a decision necessary.
