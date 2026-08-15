P-17 was fully verified and adopted.

Implemented:

- Updated all nine adapters to trim, then lowercase.
- Updated the acceptance test first and confirmed the expected red assertion.
- Preserved the existing API and added no dependencies.
- Recorded red/green execution custody in the required implementation ledger.

Completion contract is green:

- `node --test tests/normalize.test.mjs` — passed
- `git diff --check` — passed

No commits, publication, external-service actions, or out-of-scope changes were made. No evidence remains unresolved.