# Disclosure and Consolidation

Read this reference during Step 3 after custody and interface invariants are
frozen. It governs information hierarchy inside the target package; it does not
authorize public-interface or semantic changes.

## Placement standard

Default newly extracted supporting prose to `references/<topic>.md`. Keep or
create a top-level sibling Markdown file only when the package already has that
house form, the file is a short central reference such as `GLOSSARY.md`, or its
name is an established output/template contract.

Keep disclosed files one level deep from `SKILL.md`. Every live disclosed file
must be directly discoverable from `SKILL.md`; do not make file A the only path
to file B. A reference may link back to `SKILL.md` or another owner when needed,
but navigation never replaces the entrypoint's conditional pointer.

## What stays in `SKILL.md`

Keep:

- identity, invocation, supported scope, non-goals, and mode selection;
- canonical inputs, outputs, authority, read/write boundaries, and outcomes;
- the ordered common path and completion gate for every step;
- safety rules or blockers an agent must know before choosing a branch;
- public entrypoint headings as executable text or compatibility stubs;
- branch pointers naming exactly when to read a disclosed file, what it owns, and
  where control returns; and
- final interface, preservation, validator, and report gates.

## What may move

Move branch-only details, long examples and templates, deep explanations,
phase-local mechanical checklists, witness histories, rationale, failure
examples, and supporting prompts when the common path can safely select their
branch first.

For every moved atom, record:

| Field | Required value |
| --- | --- |
| Trigger | Observable branch condition |
| Read deadline | Before which action, value exposure, mutation, or verdict |
| Purpose | Exact rule, schema, instrument, or decision support owned there |
| Return point | Step or gate that consumes the result |
| Public compatibility | Retained `SKILL.md` anchor/stub or `internal-only` |

A pointer saying only “see references” is invalid. Use wording such as “When X
fires, read Y in full before Z, then return its result to Step N.”

## Split prompts, not blind limits

Use branch and loading behavior as the primary signal. Secondary prompts include:

- a bounded client read truncates or cannot keep the common path coherent;
- a paragraph buries a field, contract, gate, or hard rule;
- `SKILL.md` approaches roughly 5,000 tokens or 500 lines;
- a disclosed file exceeds roughly 1,500 words or 150 lines and spans multiple
  branches; or
- a file over roughly 1,000 words or 100 lines lacks useful navigation.

Measure both lines and words because a short line count can hide dense prose.
Do not split a focused file every applicable branch truly needs at once; add
navigation and sharpen its pointer instead.

## Move first, then consolidate

Prefer a move-first sequence:

1. Move live text close to verbatim.
2. Rebase links and install public compatibility stubs.
3. Verify source-to-destination contact for every atom.
4. Consolidate only after unique deltas are explicit.
5. Compress wording only when the ledger still proves identical operational
   meaning and firing time.

After the first split, read every final Markdown file end to end. Split a file
again only by real branch or concept, keep all resulting files directly linked
from `SKILL.md`, and remove no content merely to satisfy a size prompt.

## Single-home discipline inside the target

Keep each meaning in one authoritative target-package home. Replace internal
copies with conditional pointers or short safety reminders. Merge scattered
caveats into the place where they fire. Retain an inline reminder when the branch
would otherwise become unsafe before its reference loads.

For near-duplicates, enumerate the union of their deltas before merging:

- thresholds, units, and exact commands;
- stronger/weaker modal words;
- unique conditions, exceptions, failure behavior, and provenance;
- distinct inputs, outputs, owners, and completion effects; and
- different branch timing.

If any difference cannot be proven redundant, keep both or return
`DECISION_REQUIRED`; do not silently select the cleaner version.

Cross-skill duplication is not governed by internal single-home discipline. In
`STRICT_REFACTOR`, leave every sibling file untouched and preserve the target's
behavior. Record the relationship in the dependency ledger. Moving all copies,
choosing a new owner, or replacing sibling text with a pointer is an
`INTERFACE_MIGRATION` requiring exact external scope and authority.

## Pruning boundary

Labels such as `duplicate`, `no-op`, `stale`, or `out-of-scope` are hypotheses,
not deletion authority. In strict mode, retain the atom and report the candidate.
In migration mode, prune only when the approved ledger names the exact atom,
reason, final owner or intentional absence, affected consumers, and compatibility
effect.

When uncertain whether battle-tested content is safe to remove, keep it. A
future audit may adjudicate it; a structural refactor must not guess.
