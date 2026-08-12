Adjudication and reversible execution are complete.

1. Rejected global receipt renderer — conflicts with ADR 0001.
2. Rejected event-identifier rewriting — violates immutable recorded bytes.
3. Rejected hosted registry — explicitly out of scope.
4. Rejected global policy/status enum — conflates receipt policy with terminal meaning.
5. Adopted shared success-terminal owner — supported by principles, ADR 0001, and duplicated source mappings.

Delivered:

- Added [terminal.rs](/tmp/grill-evo-R-current-3.XDuX8O/src/terminal.rs) as the narrow success-status owner.
- Updated [commands.rs](/tmp/grill-evo-R-current-3.XDuX8O/src/commands.rs) to delegate both mappings.
- Recorded accepted [ADR 0002](/tmp/grill-evo-R-current-3.XDuX8O/docs/adr/0002-terminal-meaning-owner.md) and indexed it.
- Added “Success terminal meaning” to [CONTEXT.md](/tmp/grill-evo-R-current-3.XDuX8O/CONTEXT.md).

Out of scope remained unchanged: receipt rendering, recorded events, hosted services, and unrelated cleanup.

Validation passed: `git diff --check` and a scratch Rust compile/runtime harness exited successfully. No unresolved evidence or repository completion contract exists. Domain modeling was considered and resulted in the glossary and ADR updates above.