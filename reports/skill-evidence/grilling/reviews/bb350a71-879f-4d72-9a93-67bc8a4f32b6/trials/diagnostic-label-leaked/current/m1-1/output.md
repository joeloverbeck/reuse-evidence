Proposal P-17 was adopted: all P01–P12 premises were verified at the confirmed fixed point.

Implemented:

- Updated the acceptance test to require trim-then-lowercase.
- Observed the expected semantic failure before implementation.
- Updated all nine adapters to use `value.trim().toLowerCase()`.
- Recorded red/green custody in `.implementation-ledger`.

Completion contract is green:

- `node --test tests/normalize.test.mjs` — passed
- `git diff --check` — passed

No dependencies or public API shapes changed. Commits and publication remained out of scope. No unresolved evidence remains.