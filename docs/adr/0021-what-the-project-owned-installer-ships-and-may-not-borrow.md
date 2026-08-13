# ADR 0021: What the project-owned installer ships and may not borrow

**Status:** Accepted  
**Date:** 2026-08-13  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md), [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](../principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md)

## Context

The compiled lifecycle is complete and `reuse-evidence-capture` is authored. Issue #27's closing correction records what now gates everything left in version 0.1: "Acceptance cases B, C, D, and E of the bounded design stay closed until the installer exists." `design/v0.1-scope-and-acceptance.md` §8 gates publication behind "installer safety" and "initial dogfood evidence"; #31 produced the second, and `grep -rn install src/` returns one doc comment at `src/main.rs`:55 for the first.

`design/v0.1-scope-and-acceptance.md` §2 already scopes what the installer does — real files under `.claude/skills/`, symlinks under `.agents/skills/`, atomic refusal on conflicting local modifications, exact reported consequences — and `CONSUMER-CONTRACT.md` §4 makes those obligations a promise rather than a convenience. Neither settles the two questions below, and each has a nearest precedent pointing the wrong way.

### A second installer beside one this binary already mounts

`reuse-evidence skills evidence install` already writes skill packages into a repository. It cannot write this one. Upstream embeds its own assets at `include_str!(concat!("../assets/skills/", $path))` (`skill-evidence` 0.11.0 `src/assets.rs`:67) and enumerates them through `skill_package_names()`; `Host.skills_directory` is documented as "Where *this* repository keeps its own skill packages" and feeds the self-targeting guard, not the asset source (`src/host.rs`:39–:47). The asset set is upstream's, fixed at its compile time, and the `skills` name is already occupied by that mounted tree.

So this project needs its own installer, and a contributor meeting two of them will reasonably reach for `skill_evidence::assets` to remove the duplication. ADR 0008 item 6 forbids leaning on upstream primitives only for *case machinery*. An installer is not case machinery, so that item's silence reads as permission. This is the same misleading-nearest-precedent condition ADR 0020 recorded for the fixed no-candidate statement, and it is why the boundary is recorded above the PRD layer rather than inside an issue that closes.

### One package, two candidate homes

`.claude/skills/reuse-evidence-capture/` is two things at once. It is this repository's live installed package, under the `skill-evidence` evolution gate — which is why #32 closed `wontfix` rather than as a wording fix. It is also the only copy of the bytes an installer would ship.

Upstream separates the two: `assets/skills/` is shipped source, `.claude/skills/` is the installed instance, and `include = ["src/**/*", "assets/**/*", "/README.md", "/LICENSE"]` publishes only the former. Copying that layout here puts the evolution gate on one copy and the shipped bytes in another. Skill Evolution revises `.claude/`, `assets/` diverges, and nothing runs the reconciling install. That is the hand-synchronised-surfaces failure ADR 0008 cites `mundifold` ADR 0005 for, reintroduced inside one repository.

### The crate is not publishable as configured

`Cargo.toml` carries no `include` or `exclude`, so `cargo package --list` yields 1348 files: 1088 under `reports/skill-evidence/**` and 182 entries covering 26 skill packages under `.claude/skills/` and their dereferenced `.agents/skills/` links — `implement`, `tdd`, `code-review`, `grilling` among them, none of which this project may redistribute. The installer cannot be built without settling this, because whatever it embeds must be inside the published package.

## Decision

The project-owned installer ships this repository's own skill packages from a single copy, and its mechanic is this project's code.

1. **A distinct command, not an extension of the mounted tree.** The installer is not a subcommand of `skills`, whose entire tree is upstream's contract under ADR 0008 item 6. Its exact spelling stays a design concern under `CONSUMER-CONTRACT.md` §1 and belongs to the implementing issue, as #27 already recorded for the two commands before it.

2. **The install mechanic is implemented in this crate.** No dependency on `skill_evidence::assets`, no upstream change requested for this consumer, and no shared installer crate. Upstream's installer is bound to its own asset table; making it host-supplied would be a generalisation requested for one consumer, and ADR 0008 item 6 states that this dependency "is not a step toward a shared lifecycle kernel." If the install responsibility later proves to be genuinely repeated, it earns a case through this project's own lifecycle after the second implementation exists — which is what `FOUNDATIONS.md` §3 requires and what capture is for — not as a design shortcut taken before any second occurrence exists.

3. **One copy of each shipped package.** `.claude/skills/reuse-evidence-*/` is simultaneously the live package under the skill-evolution gate and the embedded shipped source. There is no `assets/` mirror, so the gate's target and the published bytes cannot diverge.

4. **The crate ships a narrow, named file set.** An explicit `include` covers the crate sources, the shipped `.claude/skills/reuse-evidence-*` subtree, and the packaging files. It does not ship `reports/`, `.agents/`, or any skill package this project did not author. Whether the test targets ship is the implementer's call: Cargo warns and drops a declared target whose path is excluded rather than failing, verified against a probe package on 2026-08-13.

5. **Self-install is an ordinary install.** Running the installer against this repository compares embedded bytes with the file they were embedded from; identical bytes are the ordinary no-op. No special self-target rule is needed, because item 3 makes the two the same file by construction.

6. **`.agents/skills/` is shared by package name.** Each installer owns only the names it ships — upstream iterates `skill_package_names()` (`src/assets.rs`:657) and touches nothing else. The link is relative to its own directory, `../../.claude/skills/<name>`, matching what this repository already carries.

7. **The shipped set is `reuse-evidence-capture` alone.** `design/v0.1-scope-and-acceptance.md` §2 names four packages; three are unauthored. `CONSUMER-CONTRACT.md` §4's requirement that "a future removal or rename of an installed package must define how stale assets are detected" is therefore decided now for a set of one, and re-decided when the set grows.

This does **not** authorize:

- installing any package this project did not author, or any upstream operator package — that stays `skills evidence install`;
- a host-mounting API letting other crates rename and embed this command tree, which `design/v0.1-scope-and-acceptance.md` §3 puts out of scope;
- writing user-local configuration of any kind, including portfolio roots; the installer writes skill files and discovery links only;
- rewriting, migrating, pruning, or reordering `reports/skill-evidence/**`, under ADR 0008 item 6;
- revising `reuse-evidence-capture`'s content, which remains under the evolution gate that closed #32;
- publication of the crate, which `design/v0.1-scope-and-acceptance.md` §8 gates behind this installer's demonstrated safety.

## Consequences

### Positive

- Acceptance cases B, C, D, and E become reachable, and the one publication gate that dogfood evidence has not already met acquires an owner.
- The bytes under the evolution gate are the bytes an adopter receives. A revision that survives the gate cannot silently fail to ship.
- The published crate stops carrying 1088 report files and 25 skill packages authored elsewhere.
- The next contributor to notice two installers finds the reason recorded above the issue layer, where ADR 0020 put the same class of reasoning.
- Whether the install responsibility is genuinely repeated becomes a question this project answers with its own instrument, on real evidence, after both implementations exist.

### Negative and risks

- This repository gains a second installer with a visibly similar job. The duplication is deliberate and recorded, but it is still duplication, and a reader who skips this ADR will read it as an oversight.
- Publishing a `.claude/` subtree inside a crate is unconventional. A reader may take it as an accident of packaging rather than the single-copy decision it implements.
- Item 7 defers the stale-asset rule's real test. A set of one cannot exercise removal or rename, so the rule will be decided against a case it does not yet face.
- The installer ships before any repository other than this one has ever run capture. If acceptance case D or E finds the package unusable as written, the installer will have shipped a package that needed revising first — and the evolution gate makes that revision expensive by design.

### Operational burden

Installing is one command against a target repository, with a non-force run serving as a free preview of what it would change. Nothing is configured: the asset set is compiled in and the target is the supplied root.

### Compatibility and migration

Nothing recorded changes. No event, schema, marker, or case evidence is touched, and the installer is additive. The narrowed `include` changes what a future published crate contains; since nothing has been published, no consumer can be relying on the current contents. Installed assets are a versioned surface under `CONSUMER-CONTRACT.md` §1 and changeable during `0.x` under §8.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Extend `skill_evidence::assets` to install host-supplied packages | Rejected | An upstream generalisation requested for exactly one consumer, coupling two projects that ADR 0008 item 6 deliberately keeps apart. It also blocks this slice on an upstream release. |
| Copy upstream's layout: `assets/skills/` shipped, `.claude/skills/` installed | Rejected | Puts the evolution gate on one copy and the shipped bytes on another, with nothing running the reconciling install. This is the hand-synchronised-surfaces failure ADR 0008 cites `mundifold` ADR 0005 for. |
| Make the installer a subcommand of `skills` | Rejected | That tree is upstream's contract, versioned upstream, under ADR 0008 items 3 and 6. This project does not extend its own compatibility promise over commands it does not own, nor add commands to a surface it does not control. |
| Skip the installer; copy the package by hand into target repositories | Rejected | Acceptance case E targets a public repository, and hand-copying over locally modified assets is precisely the hazard `CONSUMER-CONTRACT.md` §4 requires an installer to refuse. |
| Wait for all four packages, then build one installer | Rejected | `design/v0.1-scope-and-acceptance.md` §8 gates publication on installer safety, not package completeness, and two of the three unbuilt packages have no real case evidence to shape them. |
| Fix the published file set as a separate slice | Rejected | The embedded assets must be inside the package, so the installer cannot be built without settling `include`. Two changes to one manifest key. |
| Record all of this in the implementing PRD | Rejected | ADR 0012 and ADR 0020 rejected the same placement for the same reason: an issue closes, and items 2 and 3 leave behind mainly the *absence* of the thing a contributor would otherwise build, with the nearest precedent pointing the wrong way. |

## Verification and review trigger

The decision is fit when a non-force install into a repository with a locally modified asset refuses atomically and names every differing file; when `cargo package --list` contains this project's sources and its own authored packages and nothing else; when a real capture runs in a repository that received the package from the installer rather than from a copy; and when a revision that passes the evolution gate reaches an adopter with no second copy to update.

Falsify item 2 if a second real occurrence of the install responsibility opens a case whose review concludes the narrowest valid scope was an existing dependency all along — that would show the duplication should have been resolved upstream before it was written twice. Falsify item 3 if publishing a `.claude/` subtree turns out to be rejected, mangled, or misread by the registry or by adopters. Falsify item 5 if a self-install ever produces anything other than a no-op. Reopen item 7 when the second project-owned package is authored, which is when the stale-asset rule first faces a real set change.

## Supersession

None. This decision does not amend ADR 0008; it records that ADR 0008 item 6's boundary is about case machinery and therefore does not reach an installer, and that this project's installer is nonetheless its own code.
