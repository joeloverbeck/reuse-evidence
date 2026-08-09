# Source-blind forward tests

Load this reference only when a comparison or evidence claim requires an
executor that cannot see the source rationale, repository history, expected
behavior, version role, or another run. Source-blind is stronger than fresh: a
new session still fails when startup policy, persistent memory, or accessible
files supply withheld context.

Freeze the raw task, packet, comparison rule, deterministic checks, and
invalidation rule before launching. Give each executor a fresh scratch packet
containing only the complete skill package and shared raw input; add only the
frozen candidate delta or package to its paired run. Keep evaluator material,
version labels, expected winners, other outputs, and authoring rationale absent.

Launch a non-reused ephemeral context with user configuration, repository rules,
and persistent memory disabled. Restrict reads to the scratch packet and effects
to read-only operation or command-owned scratch outputs. State in the task that
memory lookup and reads outside the packet are forbidden. For Codex CLI, use the
available equivalents of `--ignore-user-config`, `--ignore-rules`, `--ephemeral`,
`--sandbox read-only`, `--skip-git-repo-check`, `--json`, a scratch `-C`, and a
scratch output path. Runner controls carry the isolation claim; prompt wording
alone does not.

Retain the command shape, packet identity or hashes, session identity, process
log, final output, and output hash. Check the log for outside reads and the
output for withheld-material leakage before comparison. An outside read,
inherited hidden context, reused session, leaked setup, missing receipt, or
missing output invalidates the run. Quarantine its output as diagnostic material
only and rerun the affected pair in new contexts so no valid result is compared
with a contaminated counterpart.

If implicit context cannot be disabled or its reads cannot be audited, report
the source-blind validation as blocked. Do not count, compare, or adopt from the
run merely because the visible packet was isolated.
