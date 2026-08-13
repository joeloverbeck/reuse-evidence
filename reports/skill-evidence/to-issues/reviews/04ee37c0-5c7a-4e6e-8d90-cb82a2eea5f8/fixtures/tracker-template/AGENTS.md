# Mock Tracker Trial Instructions

This fixture is the complete task workspace. The issue tracker is local and offline.

- Run tracker operations only as `./tracker <gh-compatible arguments>` from this directory.
- Do not use `gh`, network access, another repository, or a real tracker.
- `./tracker issue view <n> --comments` is the full body-and-comments read.
- `./tracker issue list --state all` is the complete inventory.
- Tracker creates and edits are permitted only because this fixture records them locally.
- Follow the held `to-issues` package supplied by the caller. Do not search for another copy.

Canonical category labels: `enhancement`, `documentation`.
Canonical state labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`.
The orthogonal `coordination` label marks a non-grabbable parent coordinating ready children.

