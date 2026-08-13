# Trial workspace artifact recovery

This note repairs the storage of the review's retained trial workspaces without
rewriting the review, evaluator decisions, process logs, event stream, or live
`grilling` skill.

Commit `a7a4974afe73e7418152afe794f24471540cf047` recorded twelve trial
workspaces as Git links to fixture commit
`1db12a5e6ec9435edd890ef65950d542fc70e51f`. There was no `.gitmodules`
mapping, and that commit object was not present in the outer repository. A
fresh checkout therefore produced empty workspace directories even though the
review's evaluators cited the worktree artifacts.

## Recovered artifact shape

Each former `workspace` Git link now has an ordinary replacement artifact set
under `workspace-artifacts/`, preserving the trial-relative suffix and
containing:

- `base-head.txt` and `base-tree.txt` — the exact fixture commit and tree;
- `base-commit.txt` — the retained commit object payload;
- `worktree-status.txt` — the complete pre-repair modified/untracked inventory;
- `tracked.patch` — a full-index Git patch containing every tracked worktree
  change;
- `implementation-ledger.txt` — the formerly untracked trial ledger; and
- `sha256sums.txt` — hashes for both retained artifacts and every reconstructed
  final changed file.

The base tree is represented once at `inputs/long-course-fixture/`. Before the
repair, every blob in that directory was checked against every workspace's
recorded base tree. To reconstruct one final workspace, copy that fixture,
apply the artifact set's `tracked.patch`, and copy
`implementation-ledger.txt` to `.implementation-ledger`. The resulting changed
files and ledger must match the hashes in `sha256sums.txt`.

## Evaluator path map

The evaluator Markdown retains the absolute scratch paths it originally
produced. Resolve them to repository artifacts as follows:

- current-arm responses `A1`, `A2`, and `A3` map to
  `workspace-artifacts/arms/arm-cobalt/a1`, `a2`, and `a3` respectively;
- the comparative evaluator's opaque arms map through
  `administrator-custody.md`: `meridian` is `arm-cobalt` and `harbor` is
  `arm-amber`, with its recorded odd/even `R` and `S` assignment preserved;
- quarantined diagnostic `current/m1-N` and `candidate/m1-N` paths map to the
  same suffixes under
  `workspace-artifacts/diagnostic-label-leaked/`.

The six diagnostic workspaces remain recoverable but remain quarantined and
uncounted. This repair changes artifact transport only; it does not change any
trial reading, comparison, validation decision, or close disposition.
