Proposal P-17 was adopted and implemented; all premises P01–P12 were verified.

- Updated the acceptance test first and observed the required failure.
- Changed all nine adapters to trim, then lowercase.
- Preserved the existing API and added no dependencies.
- Recorded red/green execution in the required implementation ledger.
- `node --test tests/normalize.test.mjs`: passed.
- `git diff --check`: passed.
- No commits or publication actions were performed.