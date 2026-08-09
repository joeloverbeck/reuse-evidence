# reuse-evidence

**Evidence-gated reuse decisions for agent-developed repository portfolios.**

`reuse-evidence` is intended to help a maintainer notice when independently maintained consumers are accumulating the same responsibility, preserve the evidence, and make an explicit decision before the implementations become expensive to keep aligned.

The project is deliberately not a clone detector and not an automatic refactoring system. Similar code is only a clue. A reuse decision must establish that the consumers actually share a responsibility, that the common behavior has a coherent owner, and that sharing it will create more leverage than coupling.

## Status

**Pre-implementation foundation.**

This repository currently defines the mission, authority model, evidence semantics, privacy rules, capability boundaries, accepted architectural decisions, and the bounded acceptance target for version 0.1. It does not yet claim to implement the planned command or skill surface.

The selected delivery constraints are:

- a public Rust crate and standalone CLI;
- local-first operation across explicitly enrolled public and private repositories;
- Claude Code skills installed as real files under `.claude/skills/`, with discovery links under `.agents/skills/`;
- durable, inspectable case evidence rather than transcript memory;
- human acceptance for every consequential reuse decision;
- implementation delegated to the repository's normal engineering workflow.

## Intended lifecycle

1. After material implementation work, a maintainer manually invokes capture.
2. A first consumer creates no durable reuse record.
3. A second independent consumer opens a watching case.
4. A third independent consumer normally makes the case ready for review; a narrowly justified human override may authorize review after the second.
5. Review may recommend extraction, an existing dependency, generation, a shared contract, intentional duplication, deferral, or splitting a wrong abstraction.
6. The maintainer accepts or rejects the exact decision.
7. Ordinary engineering skills or tools implement any accepted change.
8. `reuse-evidence` verifies the accepted migration and closes, parks, or reopens the case.

The third consumer authorizes review. It never authorizes extraction by itself.

## Non-goals

`reuse-evidence` must not become:

- a general code-quality or architecture score;
- a universal duplication percentage;
- a built-in semantic clone detector;
- a CI gate for unreviewed candidates;
- an automatic abstraction or refactoring engine;
- a hosted portfolio service;
- a product-line framework inferred from thematically related repositories;
- a stream of clean-run certification receipts;
- or a shared infrastructure kernel extracted prematurely from this repository and `skill-evidence`.

## Documentation authority

Start with [the documentation map](docs/README.md).

The normative authority order begins at [docs/principles/README.md](docs/principles/README.md). All future design documents, PRDs, issues, code, schemas, and skills must conform to the principles and accepted ADRs or amend the higher authority first through an explicit human decision.

## Repository orientation

- [CONTEXT.md](CONTEXT.md) — shared vocabulary.
- [CLAUDE.md](CLAUDE.md) — agent operating instructions.
- [docs/principles/](docs/principles/) — constitutional principles.
- [docs/adr/](docs/adr/) — accepted architectural decisions.
- [docs/design/v0.1-scope-and-acceptance.md](docs/design/v0.1-scope-and-acceptance.md) — bounded first implementation target.
