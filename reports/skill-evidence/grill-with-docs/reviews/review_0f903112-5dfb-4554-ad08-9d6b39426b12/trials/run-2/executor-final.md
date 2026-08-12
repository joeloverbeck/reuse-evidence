Adjudication and reversible execution are complete.

1. Rejected global receipt renderer — conflicts with ADR 0001.
2. Rejected event-identifier rewriting — violates immutable recorded bytes.
3. Rejected hosted command registry — explicitly out of scope.
4. Rejected global policy/status enum — absorbs command-specific policy.
5. Adopted shared success-terminal owner — supported by Principles 1, 3, and 5 and permitted by ADR 0001.

Delivered:

- Added the narrow mapping owner in [src/terminal.rs](/tmp/grill-evo-R-current-2.J103Ww/src/terminal.rs:1).
- Delegated both commands while leaving receipts unchanged in [src/commands.rs](/tmp/grill-evo-R-current-2.J103Ww/src/commands.rs:1).
- Recorded the structural decision in [ADR 0002](/tmp/grill-evo-R-current-2.J103Ww/docs/adr/0002-terminal-meaning-owner.md:1) and indexed it.
- Added “Success terminal meaning” to [CONTEXT.md](/tmp/grill-evo-R-current-2.J103Ww/CONTEXT.md:8).

Out of scope remained receipt redesign, recorded-event mutation, hosted services, and policy consolidation. No authority mismatch or unresolved evidence remained. Verification was single-threaded and passed `rustfmt --check`, standalone Rust compilation, and `git diff --check`. The repository defines no additional completion contract.