Proposal P-17 was fully verified and adopted.

Implemented:

- Updated all nine adapters to trim, then lowercase.
- Updated the acceptance test first and observed red before production edits.
- Recorded red/green custody in the required implementation ledger.
- Preserved the public API and added no dependencies.
- Made no commit or publication changes.

Completion contract is green:

- `node --test tests/normalize.test.mjs` — passed
- `git diff --check` — passed

No unresolved or unavailable evidence remains.