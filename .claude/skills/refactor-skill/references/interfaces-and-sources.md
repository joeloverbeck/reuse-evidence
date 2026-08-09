# Interfaces and Sources

Read this reference during Step 2 and use it again for Step 5. It governs the
public interface, dependency graph, source authority, requirement custody, and
identity-preserving pointer migration around the target skill.

## Public interface ledger

Treat an externally consumed heading or token as an interface, not formatting.
For each item record:

| Field | Meaning |
| --- | --- |
| Interface ID | Stable run-local key |
| Kind | Identity field, invocation policy, heading/fragment, input/output field, outcome token, edge ID, requirement ID, command, template, or client field |
| Current owner and location | Exact package path and anchor/field |
| Active consumers | Exact caller files and references |
| Strict invariant | Bytes, semantic value, entrypoint anchor, or resolved endpoint that must remain |
| Planned final home | Executable text or compatibility stub |
| Migration authority | Exact approved decision, or `none` |

Search active canonical roots for path-style references, Markdown fragments,
plain skill names, `$skill-name`, `/skill-name`, old headings, filenames,
contract tokens, edge IDs, requirement IDs, and distinctive phrases. The bare
skill-name scan is mandatory; narrow it by active roots and inspect context
rather than skipping it because the term is noisy.

Count consumers by canonical source file, not mirror occurrences. Historical
reports and archives are provenance witnesses unless repository guidance makes
them active callers.

### Compatibility stubs

In `STRICT_REFACTOR`, retain every externally referenced `SKILL.md` heading. The
heading may hold a short stub that states the branch trigger, links directly to
the extracted reference and fragment, says when it must be read, and preserves
the return contract. External callers continue to link to the stable entrypoint;
do not make them depend on an internal reference path.

Removing or renaming the anchor, redirecting callers into `references/`, or
changing an interface token is `INTERFACE_MIGRATION` even when every new link
resolves.

## Dependency ledger

Literal duplication and path reachability do not prove interoperability. For
each inbound or outbound capability relationship record:

| Field | Meaning |
| --- | --- |
| Edge identity | Existing stable ID or run-local key |
| Caller and firing point | Capability and exact branch condition |
| Callee/current owner | Skill, workflow section, principle, executable owner, or other exact owner |
| Input | Exact bounded input and freshness |
| Return | Exact declared result consumed by the caller |
| Boundary | Authority, mutation, non-recursion, and prohibited inference |
| Failure behavior | Blocked, inconclusive, invalid, or optional branch behavior |
| Final locations | Caller stub/body and callee target |

Preserve active-capability and re-entry guards, one-input/one-return limits,
branch-local loading, owner-specific effects, and blocked behavior. A refactor
that preserves every word but loads the edge after its firing point is invalid.

A verbatim passage found in sibling skills is only a candidate cross-skill
contract. Classify its owner and consumers; do not assume every copy is jointly
editable. In strict mode, change no sibling. In migration mode, every sibling
change needs an approved ledger row.

## Source and authority classification

Classify every external artifact before treating it as a dependency or duplicate:

| Class | Refactor treatment |
| --- | --- |
| Foundational authority | Read applicable sections; never edit or weaken |
| Active capability owner | Preserve its public contract and current section identity |
| Temporary unconsumed workflow | May be a bounded active owner; never retire or absorb during strict refactoring |
| Partially consumed workflow or requirements inventory | Preserve atom-level custody and residual ownership; it is conversion custody, not automatic runtime authority |
| Historical report, conversion report, or evidence | Use only for provenance/witness needs; do not make it a runtime dependency |
| Client mirror | Verify target and byte identity; edit only the canonical source |
| Internal target reference, script, asset, or manifest | Govern through target content/file custody |

The repository's current guidance decides which roots and classes are live. A
workflow header saying it is temporary does not mean the refactor may delete it;
retirement belongs to the workflow-consumption process and its own complete
custody proof.

## Requirement-custody ledger

Workflow-derived skills must preserve each staged requirement's identity and
ownership status. Record:

| Requirement ID | Source artifact | Target-owned behavior | Residual or shared owner | Runtime-authority status | Final home | Active consumers |
| --- | --- | --- | --- | --- | --- | --- |

Moving a requirement statement is allowed only when its behavior, owner,
inapplicability, residual gap, and runtime-authority statement remain unchanged.
Do not infer full consumption from the existence of a skill, and do not make a
requirements inventory a runtime dependency when the target contract says it is
conversion custody only.

## Pointer identity and rebasing

For every moved Markdown link or plain relative path, record its pre-move source,
raw text, resolved canonical target, and fragment. Recompute the relative spelling
from the destination and prove the resolved target and fragment are identical.
Existence alone is insufficient: an unchanged relative string can resolve to a
different existing file after moving into `references/`.

Check:

- inline, image, and reference-definition Markdown links;
- local fragments and explicit HTML anchors;
- plain paths in prose and code blocks;
- sibling skills, workflows, principles, schemas, scripts, and assets;
- `references/...` strings whose apparent owner changes when a new directory is
  introduced; and
- directional prose such as `above`, `below`, `earlier`, `later`, `top`,
  `bottom`, `foot`, or `end` of a file/section.

The interface helper checks local Markdown endpoints and fragments plus selected
contract surfaces. It deliberately does not infer plain-path ownership or
directional prose; inspect and disposition those manually.

## Search discipline

Use `rg --hidden -l` for broad discovery, then narrow before requesting line
output. Sweep active roots in this order:

1. target path and folder name, beginning with a path-style sweep such as
   `rg --hidden --glob '!.git/*' -l 'skills/<name>|<name>/'`;
2. Markdown links to the target and every fragment;
3. old headings, filenames, contract tokens, edge IDs, requirement IDs, and
   distinctive moved phrases;
4. mandatory bare skill/invocation names, scoped for example as
   `rg --hidden --glob '!.git/*' -l '<name>' .claude/skills/ docs/`; and
5. directional prose inside every moved source and destination, using a bounded
   sweep such as
   `rg --hidden -i -l --glob '*.md' '\b(above|below)\b|\b(earlier|later) in (this|the) (file|section)\b|at the (top|bottom|foot|end) of (this|the) (file|section)' <skill-dir>`.

Do not follow the whole repository's symlinks to verify one mirror, and do not
pipe a potentially failing broad `rg -L` into a successful count. Verify the
canonical roots and exact mirror separately, require each command to exit
cleanly, and use `readlink .agents/skills/<name>`, an exact-root search such as
`rg -l '<name>' .agents/skills/<name>/`, plus `find -L` or byte comparison as
appropriate. An explicitly named symlink root does not require `rg -L`.

## Interface migration ledger

Before `INTERFACE_MIGRATION`, persist one row per approved change:

| Change ID | External surface | Current interface | Approved replacement | Consumers | Compatibility plan | Authorized paths | Authority | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |

An authorization may come from the user's exact initial request; do not ask
again for an already exact scope. A newly discovered consumer, owner change,
workflow retirement, or broader path reopens `DECISION_REQUIRED` before editing.

After editing, run `interface-audit.mjs verify ... --mode migration`. Reconcile
every reported change exactly once to the approved ledger. A clean link graph
does not authorize or prove a semantic migration; the dependency and requirement
ledgers must also agree.
