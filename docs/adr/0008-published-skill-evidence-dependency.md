# ADR 0008: Published `skill-evidence` dependency for this repository's own skill governance

**Status:** Accepted  
**Date:** 2026-08-09  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](../principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)
**Amended:** 2026-08-11 by explicit maintainer direction to upgrade the ordinary crates.io dependency from `0.2.1` to `0.8.0`, again on 2026-08-11 from `0.8.0` to `0.9.0`, and on 2026-08-12 from `0.9.0` to `0.10.0`; the dependency form and boundaries below are unchanged.

## Context

This repository already carries agent skills under `.claude/skills/` and will author its own `reuse-evidence-*` packages. Nothing currently gates their revision, so any of them can be rewritten on a hunch — the exact failure the sibling `skill-evidence` project exists to refuse.

At acceptance, `skill-evidence` was published on crates.io at `0.2.1`, while the local `0.3.0` working tree was unpublished and untagged. Four sibling repositories already depended on it. `mundifold` recorded ADR 0005 after reversing a vendored fork, and mounts `skill_evidence::cli::SkillsArgs` beneath its own binary with a repository-specific `Host` and the exit map `0` success, `1` unsafe failure, `3` refusal.

Two accepted boundaries reach this decision and must be settled explicitly rather than by implication.

ADR 0006 states that `reuse-evidence` "has no package dependency on Matt Pocock's skills or any specific external workflow," and `CLAUDE.md` restates it as a prohibition on making "any other external skill set a package dependency." ADR 0006's context is implementation handoff: it refuses to couple the reuse lifecycle to one engineering workflow so that accepted decisions leave this project for ordinary engineering work. `skill-evidence` is not an engineering workflow — it decides whether a skill may be revised at all — but it does ship four skill packages, so the clause's literal wording reaches it.

Separately, `FOUNDATIONS.md` forbids extracting a shared Rust lifecycle kernel from `reuse-evidence` and `skill-evidence` before a third independent consumer creates real pressure, and version 0.1 puts "a shared event-lifecycle crate extracted with `skill-evidence`" out of scope. Consuming the published crate to govern this repository's skills is a different act from building this project's case machinery on its primitives. The line between those two is the load-bearing part of this decision.

## Decision

Adopt the published `skill-evidence` crate for this repository's own skill governance, and narrow ADR 0006's package-dependency clause to the scope its context describes.

1. **Narrowing.** ADR 0006's clause governs engineering-workflow skill sets consumed for implementation handoff — interface design, specifications, TDD, migration mechanics, and code review. It does not prohibit an ordinary versioned dependency on a published crate whose subject is this repository's own skill governance. ADR 0006's decision is otherwise unchanged: reuse review still produces decisions and briefs, and ordinary engineering still performs refactors.

2. **Dependency form.** Depend on `skill-evidence` as an ordinary crates.io dependency pinned by `Cargo.lock`. The initial resolved version was `0.2.1`; the amended resolved version is `0.10.0`. Do not depend on an unpublished local tree, and do not use a git reference.

3. **Mount.** Mount `skill_evidence::cli::SkillsArgs` beneath `reuse-evidence skills`, dispatched with a repository-specific `Host`: namespace `reuse-evidence`, command `reuse-evidence`, Cargo package `reuse-evidence`, and a skills directory resolved from this crate's own `CARGO_MANIFEST_DIR`, never from an audited `--root`.

4. **One terminal contract.** Adopt the crate's semantic exits as this binary's process contract — `0` success, `1` unsafe failure, `3` refusal — and give `reuse-evidence`'s own commands the same three meanings. A refusal is the system working: authority was absent and nothing was written.

5. **Installed assets.** Install the four operator packages and their schemas from the published crate into this repository. The installed instances live here and are committed; their shipped source versions upstream.

6. **Boundary — what this does not authorize.**
   - `reuse-evidence`'s case events, readiness derivation, decisions, briefs, and verification must not be implemented on `skill-evidence` types, schemas, or event machinery. Reuse-case authority stays in this crate.
   - This is not a step toward a shared lifecycle kernel. `FOUNDATIONS.md`'s third-independent-consumer condition is untouched, and this dependency is not evidence toward it.
   - It creates no dependency on any engineering-workflow skill set, and no peer-skill routing.
   - The mounted subtree is upstream's contract, versioned upstream. This project does not extend its own compatibility promise over commands it does not own.
   - Repository-local `events.jsonl` streams are never migrated, rewritten, reordered, or merged as part of a dependency upgrade. A changed package receives a new content hash; prior receipts remain historical evidence.

7. **What acceptance triggered.** Acceptance flipped this ADR's status, added its row to [`README.md`](README.md), appended a dated amendment note to ADR 0006 pointing here, and narrowed `CLAUDE.md`'s boundary bullet to name engineering-workflow skill sets. The dependency work may land under the boundary in item 6.

## Consequences

### Positive

- This repository's skills become evidence-gated: revision requires accumulated independent evidence rather than a fresh opinion.
- One binary and one terminal contract for a maintainer moving between sibling repositories.
- The host `Host` value is what makes the crate's self-targeting guard work: `operator_skill()` resolves the package currently driving a workflow from `skills_directory`, so the lifecycle can refuse to evolve the very operator package running it. Mounting resolves that path to this repository; a separately installed binary resolves it into the Cargo registry, where it can never match and the guard silently never fires.
- `Cargo.lock` identifies the governance implementation, so a defect found here is reproducible upstream instead of in a private fork.
- The vendoring failure `mundifold` ADR 0005 reversed — two code and CLI surfaces kept in sync by hand — is avoided before it can start.
- The `0`/`1`/`3` contract arrives already designed, rather than being invented for this project's first commands.

### Negative and risks

- The published `reuse-evidence` CLI carries a `skills` subtree unrelated to reuse evidence. `CONSUMER-CONTRACT.md` §1 makes CLI behavior a versioned surface, so adopters may reasonably ask what this project promises about commands it does not own. Item 6 answers that, and the README must state it plainly.
- The dependency tree gains `skill-evidence`'s own dependencies. At the resolved version those are `regex`, `serde`, `serde_json`, `sha2`, and `time`, plus `clap` and `uuid` through its default `cli` feature.
- An upgrade that changes a package's shipped bytes resets its current-hash use count while preserving every historical event. That is a projection consequence, not evidence loss.
- The nearest failure mode is drift: reaching for `skill-evidence` primitives when this project's case machinery needs an event stream. Item 6 states the boundary and the review trigger below names it.

### Operational burden

- An upgrade must inspect three separate surfaces: the published Rust API, the installed assets, and forward-only event compatibility.
- A non-force install is a free preview of what an upgrade would change; it refuses rather than clobbering locally modified assets and names every one. `--force` is a deliberate second step.

### Compatibility and migration

- No reuse-case evidence exists yet, so nothing recorded by this project needs migration.
- Installed packages and recorded receipts remain inspectable files. Dropping the dependency later would not lose the history needed to understand prior skill decisions, consistent with `CONSUMER-CONTRACT.md` §9.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Standalone adoption: `cargo install skill-evidence`, no Cargo dependency | Rejected | Keeps the published surface pure and needs no narrowing, but leaves the governance implementation unpinned by this repository's lockfile, splits the binary, namespaces emitted schema identities `skill-evidence` rather than `reuse-evidence`, and renders the self-targeting guard inert because the installed binary resolves its operator-package path into the Cargo registry rather than into this repository. |
| Mount behind a default-off Cargo feature | Rejected | Preserves a clean default published surface, but adds a feature axis and a CI matrix — machinery ahead of demonstrated need. |
| Vendor or fork the lifecycle | Rejected | The failure `mundifold` ADR 0005 reversed: two surfaces synchronized by hand and defect reports that are not reproducible upstream. |
| Defer adoption until the CLI exists | Rejected | This repository's skills stay ungoverned precisely while its own skill packages are authored, and the decision returns unchanged. |
| Build this project's case machinery on `skill-evidence` primitives | Rejected | A premature shared kernel. `FOUNDATIONS.md` requires a third independent consumer first. |
| Depend on the unpublished `0.3.0` tree or a git reference | Rejected | Not a published contract; `mundifold-extract`'s git pin already shows the drift cost. |

## Verification and review trigger

The decision is fit when a non-force `reuse-evidence skills evidence install --root .` names exactly what it would change and refuses rather than overwriting, the four packages install under `.claude/skills/` with their `.agents/skills/` links, an unauthorized `skills evolution` operation refuses with exit `3`, and `reuse-evidence`'s own commands return the same three exit meanings.

Reopen if an upgrade cannot preserve this repository's recorded receipts, if the mounted subtree creates an adopter-facing compatibility obligation this project cannot honor, if the published consumer contract cannot carry required host behavior, or if item 6's boundary is breached and case machinery begins leaning on `skill-evidence` types. A reversal must name the exact published version being left and preserve all accepted event bytes.

## Supersession

None. This ADR narrows ADR 0006's package-dependency clause without replacing it; ADR 0006's decision that reuse review produces decisions and briefs while ordinary engineering performs refactors stands unchanged.
