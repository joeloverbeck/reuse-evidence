Adopted and implemented P-17. All premises P01–P12 were verified at the confirmed fixed point.

- Updated all nine adapters to trim, then lowercase.
- Updated the acceptance test first and observed the expected failure.
- Re-ran it after implementation and observed success.
- Added the required implementation ledger.
- Completion contract passed:
  - `node --test tests/normalize.test.mjs`
  - `git diff --check`

Commits and publication remained out of scope. No unresolved evidence or decisions remain.