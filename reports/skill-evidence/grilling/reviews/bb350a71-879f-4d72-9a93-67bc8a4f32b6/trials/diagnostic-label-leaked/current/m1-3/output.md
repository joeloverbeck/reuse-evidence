Proposal P-17 was adopted and implemented.

- Verified all premises P01–P12 at confirmed HEAD.
- Updated the acceptance test first and observed the expected failure.
- Updated all nine adapters to trim, then lowercase.
- Preserved public APIs and added no dependencies.
- Kept commits and publication out of scope.
- Completion contract passed:
  - `node --test tests/normalize.test.mjs`
  - `git diff --check`

No unresolved evidence or scope changes remain.