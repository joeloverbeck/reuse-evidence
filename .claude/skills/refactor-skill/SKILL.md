---
name: refactor-skill
description: Split and consolidate existing Agent Skills without losing battle-tested content. Use when the user names a skill whose SKILL.md or referenced files have grown too large, dense, duplicated, or hard to load, and wants it reorganized into progressively disclosed files.
disable-model-invocation: true
---

# Refactor Skill

Refactor one existing skill package into a smaller, navigable `SKILL.md` plus
directly disclosed files. Preserve its behavior, public interface, dependency
contracts, source custody, and every live instruction unless the user explicitly
authorizes a named contract or interface change.

### Skill-local meta-tooling

The interface and preservation auditors under `scripts/` are executable
meta-tooling. They do not read, write, or adjudicate recorded evidence, gate
state, or dispositions. Their production paths read only the caller-supplied
repository, package, snapshot, comparison tree, and checklist inputs; emit
diagnostics, digests, JSON receipts, TSV checklists, and exit status; perform no
network access; use no external service; and remain confined to this skill
package.

Determinism contract: with the same Node.js runtime, command arguments, current
working directory, byte-identical explicit inputs, and the same repository-tree
bytes and path identities, each production command emits the same diagnostics,
digests, receipts, checklists, and exit status. The auditors consume no clock,
randomness, network, or external-service input. Every ordered output uses
locale-independent UTF-16 code-unit ordering.

Runtime requirement: Node.js 20 or later, with ESM `.mjs` and the built-in
`node:test` runner. Each production script has exactly one declaration entry:

- `scripts/code-unit-order.mjs` is the side-effect-free ordering owner imported
  by both auditor commands. It accepts two in-memory strings and returns their
  locale-independent UTF-16 code-unit comparison; it has no standalone command,
  reads no external input, emits no output, and writes nothing. Both auditor
  verification suites exercise it through their public commands.
- `scripts/interface-audit.mjs` provides
  `node .claude/skills/refactor-skill/scripts/interface-audit.mjs snapshot <target-skill-dir> [--scan-root <path>]...`,
  `node .claude/skills/refactor-skill/scripts/interface-audit.mjs check <target-skill-dir> [--scan-root <path>]...`,
  and
  `node .claude/skills/refactor-skill/scripts/interface-audit.mjs verify <before.json> <target-skill-dir> [--mode strict|migration]`.
  It reads the target package, configured active scan roots, repository identity,
  client manifests, and, for `verify`, the explicit baseline snapshot.
  `snapshot` emits the interface snapshot as JSON; `check` emits the current
  interface audit as JSON; `verify` emits the comparison receipt as JSON.
  Successful commands and `--help` or `-h` exit 0; detected link/interface
  problems and input or usage failures exit 2 with JSON or standard-error
  diagnostics as applicable. Its verification suite is
  `node --test .claude/skills/refactor-skill/scripts/interface-audit.test.mjs`.
- `scripts/preservation-audit.mjs` provides
  `node .claude/skills/refactor-skill/scripts/preservation-audit.mjs <before-dir> <after-dir> [summary [limit]]`,
  `node .claude/skills/refactor-skill/scripts/preservation-audit.mjs <before-dir> <after-dir> checklist`,
  and
  `node .claude/skills/refactor-skill/scripts/preservation-audit.mjs verify-checklist <checklist.tsv>`.
  It reads the explicit comparison trees or checklist. `summary` emits a bounded
  JSON inventory and preservation receipt; `checklist` emits the complete TSV
  disposition checklist; `verify-checklist` emits a JSON verification receipt.
  Successful commands and `--help` or `-h` exit 0; invalid, unreadable, or
  incomplete inputs exit 2 with standard-error diagnostics. Its verification
  suite is
  `node --test .claude/skills/refactor-skill/scripts/preservation-audit.test.mjs`.

Both verification suites write only to isolated fixtures under the operating
system's temporary directory and remove them after each test. The production
commands write no repository file.

## Capability contract

### Identity, scope, and non-goals

This capability reorganizes one exact existing skill package. It may split,
move, and consolidate material within that package while retaining the same
skill identity and invocation reach.

It does not redesign the target capability, reassign ownership, retire a source
workflow, change another skill, alter foundational principles, repair unrelated
defects, or treat shorter prose as proof of equivalent behavior. It does not
create a persistent refactor dossier; custody and comparison ledgers live in a
session scratch directory and ordinary history lives in Git.

### Required input and mode

The user must name one skill by exact path, folder, or skill name, for example:

```text
$refactor-skill .claude/skills/grilling
```

Resolve an ambiguous name by exact frontmatter `name`, then folder name. If
multiple live targets remain, return `BLOCKED_INPUT` with the candidates and the
exact choice needed.

Choose exactly one mode:

- `STRICT_REFACTOR` is the default. It preserves behavior, frontmatter identity,
  invocation policy, public entrypoint anchors, contract tokens, dependency
  edges, requirement custody, source authority, and external callers. It writes
  only the canonical target package. A compatibility stub inside `SKILL.md` is
  part of the target package and is preferred to changing callers.
- `INTERFACE_MIGRATION` applies only when the user's exact request or a later
  explicit decision authorizes named public-interface changes and every external
  file allowed to change. Before editing, record the approved migration ledger
  required by [Interfaces and Sources](references/interfaces-and-sources.md).

A generic request to split, shorten, consolidate, or refactor selects
`STRICT_REFACTOR`; it does not authorize pruning live behavior, changing sibling
skills, updating callers, or retiring temporary workflows.

### Outputs and owners

The output is the same canonical skill package with a smaller executable
entrypoint, directly discoverable supporting files, and current internal
pointers. In `STRICT_REFACTOR`, every external interface and owner is unchanged.
In `INTERFACE_MIGRATION`, only approved interface changes and approved external
files may differ.

The target skill remains owner of its capability contract. Foundational
principles, active sibling skills, temporary workflows, requirements inventories,
historical evidence, and client mirrors retain their existing authority classes.

### Read set, write set, and authority

Always read current repository guidance, applicable principles, the complete
working-tree target package, its client metadata, its exact mirror state, and the
current validators or checks used for delivery. Read active callers, callees,
temporary workflows, requirements inventories, and reports only when needed to
classify a live interface, dependency, source, or historical witness.

`STRICT_REFACTOR` may write only the canonical target package. It must not edit a
mirror copy, caller, sibling skill, workflow, requirement inventory, report,
principle, or unrelated dirty file. `INTERFACE_MIGRATION` may additionally write
only the exact external paths approved in its migration ledger. Both modes may
create and remove bounded scratch artifacts outside the repository.

Naming a target authorizes reversible structural work inside that target. It
does not authorize semantic pruning or external mutation. A proposed prune,
ownership change, public-anchor removal, contract-token change, dependency-edge
change, workflow retirement, or external edit returns `DECISION_REQUIRED` unless
the current request already authorizes that exact change and scope.

### Prerequisites, outcomes, and invalidation

The current target, repository guidance, scratch snapshot, custody ledgers,
interface snapshot, relevant owners, and required validators are load-bearing.
An absent or contradictory owner, unresolved target, unwritable scratch location,
or failed required instrument returns `BLOCKED_INPUT` or
`BLOCKED_INFRASTRUCTURE` for the affected branch.

An invocation ends as one of:

- `COMPLETE`: every ledger reconciles, every required check passes, and the
  final package matches the authorized mode;
- `DECISION_REQUIRED`: a proposed contract, interface, source-custody, prune, or
  external-scope change lacks exact authority;
- `BLOCKED_INPUT` or `BLOCKED_INFRASTRUCTURE`: a named condition prevents safe
  continuation;
- `INCONCLUSIVE`: preservation or interface equivalence cannot be established
  from current evidence; or
- `INVALID`: the result lost content, weakened behavior, broke an interface,
  exceeded authority, or conflicts with repository principles.

A strict refactor invalidates no consumer contract. An approved interface
migration invalidates every named caller or dependent contract in its migration
ledger until updated and reverified. Resume only from the exact working-tree
snapshot, persisted scratch ledgers, unchanged mode, and unchanged authority.
If the target, caller set, owner, validator, or approved scope changes, refresh
the affected baseline before continuing. Read-only retries are idempotent;
repeated edits must not duplicate files, pointers, or ledger rows.

## Workflow

### 1. Establish complete custody

Check `git status --short`, resolve the canonical target and mirror, and create a
scratch directory outside the repository. Snapshot every working-tree entry in
the target before editing. Then read
[Custody Ledgers](references/custody-ledgers.md) in full and build its content,
file, public-interface, dependency, and source-custody ledgers.

Run the interface snapshot before the first edit:

```text
node .claude/skills/refactor-skill/scripts/interface-audit.mjs snapshot <target-skill-dir> --scan-root .claude/skills --scan-root docs > <scratchpad>/interface-before.json
```

Adjust scan roots only from current repository guidance; exclude mirrors and
historical report/archive trees unless they are active callers.

**Done when:** the scratch snapshot covers every target entry, all ledgers cover
their required rows, the interface baseline is readable and clean, and no
unclassified atom, file, interface, dependency, or source remains.

### 2. Freeze interfaces, dependencies, and source custody

Read [Interfaces and Sources](references/interfaces-and-sources.md) in full.
Classify every inbound and outbound relationship, public entrypoint anchor,
contract token, dependency edge, requirement ID, mirror, workflow, and historical
witness. Select `STRICT_REFACTOR` unless exact authority already selects
`INTERFACE_MIGRATION`.

In strict mode, retain public `SKILL.md` anchors as short compatibility stubs and
leave every external file untouched. In migration mode, persist the exact
approved migration ledger before the first external edit. A newly discovered
scope-changing migration returns `DECISION_REQUIRED`.

**Done when:** every external relationship has an authority class and final
disposition, strict-mode invariants are frozen, and every migration-mode change
is explicitly authorized with a compatibility plan.

### 3. Design progressive disclosure

Read [Disclosure and Consolidation](references/disclosure-and-consolidation.md)
in full. Keep the invocation contract, ordered common path, early safety rules,
public compatibility anchors, required context pointers, and final gates in
`SKILL.md`. Move branch-only detail, examples, templates, explanations, and
mechanical checklists into focused files directly linked from `SKILL.md`.

For every extracted file record its trigger, read deadline, purpose, and return
point. Plan moves before compression. In strict mode, a prune candidate blocks;
exact duplicate consolidation is permitted only when every unique delta and
firing point survives.

**Done when:** every atom has one authorized final home, every disclosed file has
a direct conditional pointer from `SKILL.md`, public anchors retain compatible
stubs, and no nested disclosure chain is required to find a live instruction.

### 4. Edit from the ledgers

Apply the smallest file-scoped patches that realize the frozen plan. Rebase each
moved relative link from its new source location while preserving its canonical
target and fragment. Update ledger homes before deviating from the plan.

If an edit fails, is interrupted, or returns an ambiguous result, stop before
another patch. For scratch-only operations, reread the scratch target. For any
operation that may have touched the repository, refresh target inventory,
`git status --short`, and exact edited paths; classify the result as no write,
partial write, or complete write, then resume from the ledgers.

**Done when:** the planned target files exist, every edit is classified, no
unauthorized path changed, and all internal pointers resolve from their new
locations.

### 5. Verify interface preservation

Run the interface verifier against the pre-edit snapshot:

```text
node .claude/skills/refactor-skill/scripts/interface-audit.mjs verify <scratchpad>/interface-before.json <target-skill-dir> --mode strict
```

For an authorized migration, use `--mode migration`; every reported interface
change must appear exactly once in the approved migration ledger, and every
current local link and fragment must still resolve. Also inspect changed plain
paths and directional prose, which the helper intentionally does not infer.

**Done when:** strict verification reports zero changes and zero problems, or
every migration change is authorized and reconciled with zero link problems.

### 6. Prove content and package preservation

Read [Preservation and Closeout](references/preservation-and-closeout.md) in full.
Run the all-file, word-occurrence, window, checklist, validator, mirror, and
worktree checks against the scratch snapshot—not against `HEAD`. Resolve every
shortfall or changed entry with a ledger-backed disposition.

**Done when:** the final package and all ledgers agree, every required instrument
passes, unexplained deletion is zero, and pre-existing target and unrelated dirt
are reconciled separately.

### 7. Deliver the final report

Report:

- target, canonical path, and selected mode;
- files created, changed, moved, or removed;
- what stayed inline and what moved behind each conditional pointer;
- every consolidation, approved prune, or interface migration with its reason
  and authority;
- public-interface, dependency, requirement-custody, workflow-source, pointer,
  fragment, and mirror results;
- preservation helper, interface helper, deterministic test, validator, and
  fallback commands with results;
- initial-versus-final worktree reconciliation, separating pre-existing target
  dirt from unrelated dirt; and
- any retained uncertainty, blocker, or battle-tested instruction kept because
  equivalence was not established.

**Done when:** the report matches the actual file inventory and authorized mode,
and it makes no broader completeness claim than the checks establish.
