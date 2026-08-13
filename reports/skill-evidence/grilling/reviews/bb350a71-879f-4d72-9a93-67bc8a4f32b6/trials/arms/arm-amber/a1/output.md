Proposal P-17 was fully verified and adopted.

Implemented:

- Updated all nine adapters to trim, then lowercase.
- Updated the acceptance test first and observed the required failure.
- Confirmed the test passed after implementation.
- Recorded red/green custody in the implementation ledger.

Completion contract is green:

- `node --test tests/normalize.test.mjs`
- `git diff --check`

Commits, publication, dependencies, and unrelated changes remained out of scope.