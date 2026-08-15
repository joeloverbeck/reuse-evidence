Proposal P-17 was adopted: all twelve premises were verified at the confirmed fixed point.

Implemented:

- Updated the acceptance test first and observed the required failure.
- Changed all nine adapters to trim, then lowercase.
- Preserved exports, dependencies, and all other public behavior.
- Recorded red/green custody in the implementation ledger.
- Made no commit or publication.

Completion contract is green:

- `node --test tests/normalize.test.mjs` — passed
- `git diff --check` — passed

No evidence remains unresolved or pending.