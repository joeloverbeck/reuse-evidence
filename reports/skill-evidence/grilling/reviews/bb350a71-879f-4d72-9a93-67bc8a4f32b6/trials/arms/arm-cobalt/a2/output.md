Proposal P-17 was fully verified and adopted.

Implemented:

- Updated the acceptance test to require trim-then-lowercase behavior.
- Confirmed the updated test failed before implementation.
- Updated all nine adapters to return `value.trim().toLowerCase()`.
- Preserved APIs and added no dependencies.
- Recorded red/green execution in the required implementation ledger.

Completion contract is green:

- `node --test tests/normalize.test.mjs` — passed
- `git diff --check` — passed

Commits and publication remained out of scope.