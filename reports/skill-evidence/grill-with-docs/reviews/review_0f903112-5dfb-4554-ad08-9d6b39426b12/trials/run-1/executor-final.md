Adjudication and reversible execution are complete.

1. Rejected global receipt renderer — contradicts ADR 0001.
2. Rejected event identifier rewriting — violates immutable event bytes.
3. Rejected hosted registry — explicitly out of scope.
4. Rejected global status enum — improperly merges command policy with terminal meaning.
5. Adopted narrow success-terminal owner — supported by Principles 1 and 5 and compatible with ADR 0001.

Implemented:

- Added [src/terminal.rs](/tmp/grill-evo-R-current-1.RuD3BQ/src/terminal.rs) as the shared success-status owner.
- Updated [src/commands.rs](/tmp/grill-evo-R-current-1.RuD3BQ/src/commands.rs) to delegate both mappings without changing receipts.
- Recorded [ADR 0002](/tmp/grill-evo-R-current-1.RuD3BQ/docs/adr/0002-terminal-meaning-owner.md) and indexed it.
- Domain modeling crystallized “Success terminal meaning,” added to [CONTEXT.md](/tmp/grill-evo-R-current-1.RuD3BQ/CONTEXT.md).

Verification: `git diff --check` passed, and a temporary Rust harness confirmed both statuses remain `0` and both receipts remain unchanged. No Cargo manifest or additional completion contract exists. The pre-existing untracked `raw.jsonl` was untouched.

Provenance: `docs/principles.md`, accepted ADR 0001, `architecture-review.md`, and `src/commands.rs`.  
Convention checked: authority order, five-candidate review order, ADR amendment discipline, authorized edit surface, and reversible-only execution.