# Preservation and Closeout

Read this reference during Step 6 after interface verification. It defines the
content-preservation audit, validator/link fallback, mirror proof, and worktree
reconciliation required before delivery.

## Re-read the final package

Read final `SKILL.md`, every final Markdown file, every identity-bearing client
manifest, and every modified script, schema, or asset. Confirm coherent order,
complete sentences, stable numbering, branch-local read timing, and current
client identity. A changed capability purpose or invocation reach stales both
frontmatter and client metadata.

Compare the final package with every scratch ledger:

- each source atom is inline, moved, merged with all deltas, or explicitly
  approved for pruning;
- each file row has its authorized final path, type, and digest/disposition;
- each public interface, dependency, source, and requirement row has its planned
  final owner and location; and
- every deviation was recorded before its edit with authority and reason.

For every load-bearing atom, verify the executable rule remains named at or
directly pointed from its firing point. Preservation inside late witness prose is
not sufficient.

## Run the content-preservation helper

Compare against the scratch snapshot, not Git:

```text
node .claude/skills/refactor-skill/scripts/preservation-audit.mjs <scratch-before> <final-skill-dir> summary
node .claude/skills/refactor-skill/scripts/preservation-audit.mjs <scratch-before> <final-skill-dir> checklist > <scratchpad>/preservation-checklist.tsv
node .claude/skills/refactor-skill/scripts/preservation-audit.mjs verify-checklist <scratchpad>/preservation-checklist.tsv
```

The helper performs:

1. an all-file inventory and SHA-256 census, including unsupported text-analysis
   entries and symlinks;
2. a Unicode-aware word-occurrence census, where every shortfall needs a
   disposition; and
3. overlapping ten-word window coverage to localize rewritten or missing spans.

Read the complete checklist in bounded chunks and fill every disposition cell.
For a window cluster, read the full source span; if it crosses atoms or final
homes, name every represented atom and destination. A line-identity diff is valid
only for a strictly verbatim move. Reflow, heading promotion, and indentation
changes require the tolerant census plus direct inspection.

Inventory additions and modifications expected from the approved refactor still
need ledger-backed dispositions. Any unsupported file that disappeared or
changed blocks completion unless its file row explicitly authorizes it.

Unexplained deletion fails. Restore the atom/file, obtain exact approval for the
change, or return `INCONCLUSIVE`/`INVALID`; never wave through a shortfall because
the final prose reads well.

## Run interface and deterministic checks

Interface verification from Step 5 is required even when a generic skill
validator exists. Run the helper's own deterministic tests whenever its code or
contract changes:

```text
node --test .claude/skills/refactor-skill/scripts/interface-audit.test.mjs
```

When the target ships instruments, execute each changed instrument and the
applicable target tests against scratch or inert representative inputs. A
successful run that leaves a material branch unexercised is not complete.

## Validator and link fallback

Discover the current repository-native skill validator with a bounded search,
for example:

```text
find . -name 'quick_validate.py' -o -name '*validate*skill*'
```

Run the authoritative local validator when one exists. A generic validator that
rejects the same repo-native contract on a known-good sibling is a compatibility
finding, not authority to delete local fields.

Whether or not a validator exists, the interface helper must report zero current
local Markdown path or fragment problems for the target and its active inbound
callers. Additionally enumerate and inspect changed plain paths, code-block
commands, and directional prose because they are outside the helper's parser.
Report the exact validator and fallback commands.

## Mirror and worktree proof

Verify the exact mirror entry separately from canonical-root searches:

- `readlink` when the mirror should be a symlink;
- `find -L <exact-mirror> -name SKILL.md` to prove resolution; and
- `cmp -s` when byte identity is expected.

Do not use a repo-wide symlink-following pipeline whose upstream failure can be
hidden by a successful count.

Re-run `git status --short` and compare it with the initial baseline. Report
separately:

- target paths changed by this refactor;
- pre-existing target dirt preserved or intentionally extended;
- unrelated dirt unchanged, changed externally, or no longer present; and
- unexpected paths, which block completion until classified.

Run `git diff --check` on every changed target and approved external file. A Git
diff is useful for review and whitespace checking, but it is not the dirty-target
preservation proof.

## Completion rule

Completion requires all of:

- content, file, interface, dependency, source, requirement, and migration
  ledgers reconciled;
- preservation checklist verification exit zero;
- interface verification with zero problems and no unapproved change;
- applicable deterministic tests and repository-native validation passing;
- mirror resolution/identity passing; and
- final worktree inventory matching the authorized write set.

If any condition is unavailable or unresolved, report the exact blocker or
inconclusive evidence and do not label the refactor complete.
