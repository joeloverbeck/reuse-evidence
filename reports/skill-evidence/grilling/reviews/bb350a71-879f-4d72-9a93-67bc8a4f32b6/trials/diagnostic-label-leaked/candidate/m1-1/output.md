Proposal P-17 was adopted and implemented. All premises P01–P12 were verified.

- Updated the acceptance test first and observed the required failure.
- Updated all nine adapters to trim, then lowercase.
- Preserved APIs and added no dependencies.
- Recorded red/green results in the implementation ledger.
- No commit or publication performed.

Completion contract is green:

- `node --test tests/normalize.test.mjs` — passed
- `git diff --check` — passed