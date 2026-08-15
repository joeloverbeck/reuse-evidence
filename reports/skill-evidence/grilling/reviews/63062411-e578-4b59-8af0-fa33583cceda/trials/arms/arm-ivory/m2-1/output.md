Proposal P-17 was fully verified and adopted.

Implemented:

- Acceptance test now requires trim-then-lowercase behavior.
- Confirmed the updated test failed before implementation: exit 1.
- Updated all nine adapters to use `trim().toLowerCase()`.
- Preserved exports, dependencies, and all other public behavior.
- Recorded red/green custody in the required implementation ledger.

Completion contract passed:

- `node --test tests/normalize.test.mjs` — exit 0
- `git diff --check` — exit 0

No commits, publication, external contact, or out-of-scope changes were made. No evidence remains unresolved.