# Fixture instructions

The accepted source records under `sources/` are authoritative for `docs/phase-check.md`.

The completion contract is:

1. `cmp docs/phase-check.md expected/phase-check.md`
2. `git diff --check`

Do not modify any path except `docs/phase-check.md`. Do not commit, publish, or contact external services.
