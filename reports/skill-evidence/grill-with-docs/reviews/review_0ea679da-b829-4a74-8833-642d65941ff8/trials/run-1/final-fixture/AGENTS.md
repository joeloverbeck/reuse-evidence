# Agent Instructions

`AGENTS.md` is a symlink to this file. Maintain this file as the single agent-instruction source.

## Read authority before acting

For any material work, read in this order:

1. `docs/principles/README.md`
2. `docs/principles/FOUNDATIONS.md`
3. the topic principle documents relevant to the change
4. accepted ADRs under `docs/adr/`
5. the active design, PRD, issue, or specification

If a lower-level request conflicts with higher authority, stop the conflicting work. Propose the smallest explicit amendment to the higher authority and obtain human acceptance before changing dependent material. Editing a PRD, issue, code file, or test does not amend a principle or accepted ADR.

## Current primary path

The active repository value stream is the bounded version 0.1 proof in `docs/design/v0.1-scope-and-acceptance.md`:

- enroll repositories explicitly;
- capture a real second occurrence;
- derive ordinary third-occurrence review readiness;
- record an exact human decision;
- delegate implementation outside the reuse lifecycle;
- verify the accepted result;
- preserve private evidence correctly;
- keep routine clean capture cheap and write-free.

Work not needed for that path is parked unless the human explicitly reprioritizes it or a bounded experiment can falsify a prerequisite decision.

## Non-negotiable boundaries

Do not:

- treat code similarity as proof of one responsibility;
- treat a third occurrence as extraction authority;
- build or embed a clone detector;
- add automatic end-of-session capture or refactoring hooks in version 0.1;
- write clean or no-candidate evidence records;
- turn discovery into a general architecture audit;
- implement refactors inside the reuse-review capability;
- make Matt Pocock's or any other engineering-workflow skill set a package dependency; the published `skill-evidence` crate is a separately accepted governance dependency under ADR 0008, not an exception to this;
- leak private repository identities, paths, source, or evidence into public state;
- scan unmarked repositories merely because they sit beneath a configured root;
- build a hosted service, daemon, MCP server, sensor-plugin platform, or central portfolio repository without new accepted evidence;
- extract a shared lifecycle kernel from `reuse-evidence` and `skill-evidence` before a third independent consumer creates real pressure;
- invent verification commands, file schemas, or compatibility promises before they are accepted and implemented.

## Required reasoning for material proposals

A material proposal must state:

- the human or consumer effect sought;
- the live bottleneck or decision it addresses;
- the evidence currently bearing the claim;
- the narrowest scope that could satisfy it;
- what is explicitly out of scope;
- what existing authority it conforms to;
- and what would falsify or park the proposal.

For a new abstraction, identify the real independent consumers. Repeated files, similar names, generated copies, tests, or legacy corpus patterns are not enough.

## Documentation conventions

- Foundational principles live under `docs/principles/`.
- Architectural decisions live under `docs/adr/` and are not accepted until the human accepts them.
- Time-bounded implementation direction belongs under `docs/design/`, PRDs, or issues and must remain subordinate.
- Keep decision-bearing records compressed. Do not create hand-authored status, routing, provenance, or certification paperwork that can be derived mechanically.
- Preserve rejected or parked decisions concisely; remove their inactive machinery from default context and the active dependency graph.

## Implementation workflow boundary

`reuse-evidence` owns evidence, readiness, decisions, and verification. Ordinary engineering skills own interface design, TDD, implementation, and code review. An accepted reuse decision should produce a bounded implementation brief, then yield control. After implementation, return to reuse verification rather than duplicating the engineering workflow here.

## Agent skills

### Issue tracker

Issues and PRDs are tracked in GitHub Issues. External contributor PRs are also a triage request surface; collaborators' in-flight PRs are excluded. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses the canonical `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix` labels. The orthogonal `coordination` label marks an open parent coordinating separately `ready-for-agent` children; the parent is not independently grabbable. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repository with `CONTEXT.md` at the root and architectural decisions under `docs/adr/`. See `docs/agents/domain.md`.
