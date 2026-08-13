Proposal P-17 was adopted and implemented. All 12 premises were verified at the confirmed `HEAD`.

- Updated the acceptance test first and observed the required failure.
- Changed all nine adapters to trim, then lowercase.
- Added the required implementation ledger.
- Preserved public APIs and added no dependencies.
- No commit or publication was performed.

Completion contract is green:

- `node --test tests/normalize.test.mjs` — passed
- `git diff --check` — passed

No unresolved or unavailable evidence remains.