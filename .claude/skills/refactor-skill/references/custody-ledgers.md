# Custody Ledgers

Read this reference during Step 1 before any repository edit. It defines the
working-tree snapshot and the ledgers that make a refactor resumable and
auditable without turning repository process paperwork into durable truth.

## Snapshot the working tree

1. Run `git status --short` and record unrelated dirt separately from dirt inside
   the target package.
2. Resolve the target directory and its `.agents/skills/<name>` entry, if any.
   Edit the canonical source, usually `.claude/skills/<name>`. Determine whether
   the mirror is a symlink, copy, missing entry, or broken entry.
3. Create a unique scratch directory outside the repository. Copy every target
   entry exactly as listed, preserving file bytes, entry types, and symlink text.
   The working tree—not `HEAD`—is the preservation `before`, including untracked
   files and inherited edits.
4. Store scratch ledgers beside that snapshot. Do not add them to the target
   package or another repository path.

The snapshot must precede the first edit. `git diff` cannot replace it: new
move-first destinations may be untracked, an inherited dirty baseline is not
`HEAD`, and a mid-run commit changes the comparison base.

## Read the complete package

List every target entry. Read `SKILL.md` in full, every Markdown file it points
to, every sibling Markdown file even when its pointer is weak, every
identity-bearing client manifest, and every script, schema, asset, or symlink
whose role or identity can affect the refactor. Inspect executable write behavior
before running a helper.

Do not infer an unread file's role from its name. A script may be a validator, a
manifest may repeat invocation policy, and an untracked reference may already be
part of the live target.

## Content-atom ledger

Segment every Markdown and identity-bearing manifest into independently
dispositionable atoms:

- frontmatter and client identity fields;
- headings with their bodies;
- numbered steps and completion gates;
- bullet groups, tables, code blocks, templates, and gotchas;
- required emitted fields, schemas, commands, validators, clobber guards,
  provenance rules, and stamps;
- every `must`, `never`, `required`, `done when`, refusal, or equivalent hard
  rule, even when buried in rationale; and
- cross-skill routing, source-custody, authority, and compatibility rules.

For each atom record:

| Field | Meaning |
| --- | --- |
| Source | Exact file and section or line range in the scratch snapshot |
| Atom | Stable run-local label |
| Distinctive witness | Phrase or structure used to locate it after editing |
| Firing point | Before which decision, effect, or completion claim it is needed |
| Initial disposition | `inline`, `move`, `merge`, or `prune-candidate` |
| Final home | Exact planned file and heading |
| Merge target or prune reason | Required for `merge` or `prune-candidate` |
| Authority | `strict-preserve` or exact approved migration/prune decision |

A surrounding narrative atom does not cover a hidden load-bearing rule. Give the
rule its own row and keep its imperative at the firing point; examples, witness
history, and rationale may move behind a conditional pointer.

In `STRICT_REFACTOR`, `prune-candidate` is a blocker, not permission to delete.
An exact-duplicate `merge` is allowed only when the ledger proves every unique
delta, trigger, strength word, threshold, and completion effect survives.

## File ledger

Every listed target entry not fully represented by content atoms receives one
file-level row:

| Field | Meaning |
| --- | --- |
| Path | Relative target-package path |
| Entry type | File, directory, or symlink |
| Starting SHA-256 | File bytes or symlink text |
| Role | Script, asset, schema, manifest, generated file, or other exact role |
| Disposition | `preserve-byte-identical`, `move`, `modify`, or `prune-candidate` |
| Authority and reason | Why the mode permits the disposition |

An unsupported or unreadable entry blocks completion unless the user explicitly
authorizes a named disposition without relying on content equivalence.

## Interface and dependency ledger seeds

Before Step 2, seed separate rows for:

- every frontmatter/client identity and invocation field;
- every heading in `SKILL.md` referenced by an external active caller;
- every stable input/output field, outcome token, edge ID, requirement ID,
  validator command, and output/template name;
- every inbound and outbound skill/workflow relationship; and
- every external source or mirror encountered during package reads.

[Interfaces and Sources](interfaces-and-sources.md) defines their full fields and
dispositions. No content edit begins while any seed is unclassified.

## Persistence and resumption

Persist the complete ledgers in the scratch directory as inspectable Markdown,
TSV, or JSON before editing. Conversation summaries may project them but are not
their only home. When a final home or disposition changes, update the scratch
ledger first, record the reason and authority, then edit.

Across an interruption, confirm the target bytes, interface baseline, mode,
authority, and caller roots still match. If any changed, refresh only the
affected baseline and rows. An atom, file, interface, dependency, or source with
no current disposition blocks continuation.
