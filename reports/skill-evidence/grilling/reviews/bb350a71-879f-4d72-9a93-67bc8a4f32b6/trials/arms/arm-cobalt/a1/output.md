Proposal P-17 adopted and implemented. All premises P01–P12 were verified at the confirmed `HEAD`.

- Updated the acceptance test to require trim-then-lowercase; observed the required red failure.
- Updated all nine adapters to use `value.trim().toLowerCase()`.
- Public API shape and dependencies remain unchanged.
- Commits and publication remained out of scope.
- Recorded red/green execution in the required implementation ledger.
- Completion contract passed:
  - `node --test tests/normalize.test.mjs`
  - `git diff --check`

No unresolved or unavailable evidence remains.