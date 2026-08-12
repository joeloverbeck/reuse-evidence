Adjudication and reversible execution are complete.

1. Rejected global receipt renderer — conflicts with ADR 0001.
2. Rejected event-identifier rewriting — violates recorded-event immutability.
3. Rejected hosted registry — explicitly out of scope.
4. Rejected global status enum — improperly combines shared terminal meaning with command-specific receipt policy.
5. Adopted shared success owner — both commands now delegate to the narrow internal owner in [terminal.rs](/tmp/grill-evo-R-current-4.dHPO2c/src/terminal.rs).

Delivered:

- Updated [commands.rs](/tmp/grill-evo-R-current-4.dHPO2c/src/commands.rs).
- Added accepted [ADR 0002](/tmp/grill-evo-R-current-4.dHPO2c/docs/adr/0002-terminal-meaning-owner.md) and indexed it.
- Added “Success terminal meaning” to [CONTEXT.md](/tmp/grill-evo-R-current-4.dHPO2c/CONTEXT.md).
- Left receipt rendering and candidates 1–4 unchanged.

Domain-doc outcome: domain modeling crystallized one glossary term and one ADR-worthy structural decision; both were recorded in their governing locations.

Provenance: `docs/principles.md`, accepted ADR 0001, `architecture-review.md`, and the duplicate mappings verified in `src/commands.rs`.

Convention checked: authority ordering, ADR amendment discipline, narrow invariant ownership, and the review’s edit allowlist.

Verification passed: `git diff --check` and a Rust test-mode compilation of `src/commands.rs`. The repository defines no additional completion contract or tests.