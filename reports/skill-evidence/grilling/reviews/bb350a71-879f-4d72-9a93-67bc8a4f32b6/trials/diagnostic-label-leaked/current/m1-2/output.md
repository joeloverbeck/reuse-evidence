Proposal P-17 was fully verified and adopted.

Implemented:

- Updated all nine adapters to trim, then lowercase.
- Updated the acceptance test first and observed the required red failure.
- Confirmed the focused test passed after implementation.
- Recorded red/green custody in `.implementation-ledger`.
- Preserved public APIs and added no dependencies.
- Made no commits or publication changes.

Completion contract is green:

- `node --test tests/normalize.test.mjs` — passed
- `git diff --check` — passed