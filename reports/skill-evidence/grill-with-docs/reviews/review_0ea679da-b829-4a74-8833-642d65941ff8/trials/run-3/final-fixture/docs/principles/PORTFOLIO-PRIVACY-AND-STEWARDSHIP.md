# Portfolio, Privacy, and Stewardship Principles

**Status:** Adopted topic principle  
**Governed by:** [`FOUNDATIONS.md`](FOUNDATIONS.md)

## 1. Explicit enrollment

A repository joins the local reuse ecosystem only by carrying a valid versioned `reuse-evidence.toml` marker.

Enrollment means:

- the repository consents to local discovery under configured portfolio roots;
- its declared visibility and stable repository identity may participate in case evidence;
- and the repository accepts the tool's consumer contract.

Enrollment does not mean:

- every file must be scanned immediately;
- every existing duplication becomes debt;
- the repository must depend on the Rust crate at runtime;
- its architecture belongs to a common product line;
- or it has accepted migration to any shared package.

Storage is not a backlog. Co-location beneath `~/src` or another root creates no authority by itself.

## 2. Portfolio discovery

User-local configuration names one or more roots to inspect. Each explicit portfolio operation rescans those roots for markers and reports:

- newly enrolled repositories;
- moved or temporarily unavailable repositories;
- duplicate stable repository identities;
- unsupported marker versions;
- and visibility changes.

Unmarked Git repositories are ignored. Version 0.1 has no daemon, watcher, background scan, or central hand-maintained repository list.

## 3. Stable repository identity

Paths are local and mutable. Evidence should identify repositories through stable opaque IDs plus recoverable Git identity and repository-relative references.

Absolute local paths are derived user-local data and should not be committed as authoritative evidence. Renaming or moving a repository must not create a new consumer occurrence by itself.

## 4. Steward repository

Every case has exactly one steward repository that owns:

- the authoritative case event stream;
- the exact accepted decision;
- verification and closure state;
- and privacy classification.

Default stewardship:

1. an intra-repository case is stewarded by that repository;
2. a new cross-repository case is normally stewarded by the repository in which the second occurrence is recognized;
3. when any participant is private, the steward must be private;
4. a public repository may steward only a case whose entire evidence surface is public.

Other repositories are referenced consumers. They do not receive synchronized copies merely because they participate.

## 5. Cross-repository write authority

Reading an enrolled repository for local comparison does not authorize writing into it.

A capture or review session may propose a case event in the current repository's steward stream. Any operation that would write into another repository must:

- identify the target;
- preview the exact consequence;
- verify expected revision;
- obtain explicit authority;
- be idempotent;
- and emit an inspectable receipt.

Version 0.1 should avoid cross-repository writes whenever a single steward record is sufficient.

## 6. Private dominance

One private participant makes the complete case private.

A mixed-visibility case must not:

- be stewarded publicly;
- write private repository names, paths, source bodies, symbols, commits, specifications, or reports into public state;
- upload private source or embeddings to a remote provider by default;
- or publish a provenance narrative that allows the private consumers to be reconstructed.

When visibility is uncertain, choose private and require a human decision to relax it.

## 7. Public extraction from private pressure

A private case may legitimately result in a public crate or repository. The public artifact receives only what its public consumer contract needs:

- the accepted public responsibility and scope;
- sanitized examples;
- public tests and schemas;
- compatibility and release rules;
- and independently appropriate public rationale.

The public artifact does not receive the private case stream, private repository identities, source paths, or confidential commercial context.

Publication of the implementation does not transfer the historical case. The original private steward closes the case by referencing the public result.

## 8. Fixed stewardship in the initial design

Version 0.1 does not support ordinary stewardship transfer. The case is the historical record of why a decision was made; the resulting shared package owns its implementation, not the originating evidence history.

If a steward repository is later retired and a real need appears, a successor mechanism may be designed from that pressure. Do not prebuild transfer, replication, or consensus machinery.

## 9. Derived local index

A user-local cache may map stable repository IDs to current paths and derive portfolio status or search indexes. It must be:

- disposable;
- rebuildable from enrolled repositories and their authoritative case state;
- excluded from version control;
- and incapable of silently overriding committed evidence.

The cache is an optimization and recovery aid, not a central source of truth.

## 10. Network boundary

The core lifecycle is local-first. Public package metadata or documentation may be queried during a decision-bound dependency review, but source disclosure and remote model use are separate concerns.

No capability may send private code, prompts containing private evidence, embeddings, case details, or repository metadata to a remote service without:

- a named live decision requiring it;
- an explicit disclosure preview;
- accepted authority;
- a bounded payload;
- and a recorded result.

## 11. Withdrawal and unavailable repositories

Removing the marker withdraws a repository from future discovery. It does not rewrite historical facts already recorded in a valid steward case.

An unavailable repository makes dependent evidence stale or unverifiable; it does not erase the occurrence. Status should report the limitation honestly and block decisions that require evidence no longer inspectable.

## 12. Public ecosystem restraint

A public repository or crates.io release is a distribution channel. Portfolio enrollment is local consent. Neither establishes a public shared ecosystem, telemetry program, hosted account model, central registry, or support obligation.
