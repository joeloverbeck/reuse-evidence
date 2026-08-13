Proposal P-17 was adopted: all twelve premises were verified at the confirmed fixed point.

Implemented the authorized trim-then-lowercase contract across all nine adapters and updated the acceptance test first. Red-green custody was recorded:

- Updated test failed before production changes.
- Focused test passed after implementation.
- `node --test tests/normalize.test.mjs` passed.
- `git diff --check` passed.

No dependencies, commits, publication, or public API shape changes were introduced. No unresolved evidence or scope forks remain.