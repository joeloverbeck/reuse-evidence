# reuse-evidence

**Evidence-gated reuse decisions for agent-developed repository portfolios.**

`reuse-evidence` is intended to help a maintainer notice when independently maintained consumers are accumulating the same responsibility, preserve the evidence, and make an explicit decision before the implementations become expensive to keep aligned.

The project is deliberately not a clone detector and not an automatic refactoring system. Similar code is only a clue. A reuse decision must establish that the consumers actually share a responsibility, that the common behavior has a coherent owner, and that sharing it will create more leverage than coupling.

## Status

**Repository enrollment, marker-only portfolio reporting with derived change state, prepared-proposal staging resolution, and the durable case lifecycle from opening through verification disposition are implemented, together with implementation-brief projection, reading, and skill governance.**

The public Rust crate and standalone `reuse-evidence` binary can enroll a Git repository, including an npm workspace with no Cargo project. Enrollment writes a human-readable version 1 TOML marker at the nearest repository root, safely revalidates an existing marker without minting another identity, and uses the binary's shared success, unsafe-failure, and refusal exit meanings.

Enrollment refuses implicit visibility, ecosystem-identity, or repository-identity conflicts and refuses malformed, truncated, or unsupported-version markers without rewriting them. Declared visibility can be changed only through the dedicated `set-visibility` command. The portfolio command freshly scans configured roots for marked Git repositories and reports current enrollment, duplicate identities, unsupported or unreadable markers, and new, moved, unavailable, visibility-changed, or identity-substituted repositories. The staging-directory command names the guarded user-local directory where prepared proposals belong without creating it. The case command can preview and atomically open a steward-local case from two or more evidenced occurrences, append a later occurrence against an expected revision, record a human early-review override on a watching case, record the exact accepted decision on a review-ready case, project that decision's implementation brief, record verification with a closed, parked, or reopened disposition, find cases across the enrolled portfolio, list every case stewarded by the current repository, and show one case's complete evidence record with freshly derived lifecycle, privacy-conflict, and staleness conditions. The binary also mounts the published `skill-evidence` lifecycle under `reuse-evidence skills` and this repository commits the four operator packages it installs. This repository now carries the explicit-only `reuse-evidence-capture` package; its installer, reuse-review proposal authoring, and the remaining project-owned `reuse-evidence-*` packages are not implemented yet.

The selected delivery constraints are:

- a public Rust crate and standalone CLI;
- local-first operation across explicitly enrolled public and private repositories;
- Claude Code skills installed as real files under `.claude/skills/`, with discovery links under `.agents/skills/`;
- durable, inspectable case evidence rather than transcript memory;
- human acceptance for every consequential reuse decision;
- implementation delegated to the repository's normal engineering workflow.

## Enrollment

From anywhere inside the Git repository to enroll:

```console
reuse-evidence enroll --ecosystem-id products --visibility private
```

`--visibility` accepts exactly `public` or `private`. A successful command writes `reuse-evidence.toml` at the nearest recognizable Git root and reports the path and values it wrote on stdout. A root is recognized when `.git` is a worktree file or `.git/HEAD` is a file in ordinary Git metadata. Enrollment adds no dependency or manifest entry to the enrolled repository and performs no network access.

Re-running the same command validates and reports the existing enrollment with exit status `0`; it preserves the complete marker byte-for-byte. A different requested visibility or ecosystem identity is a refusal and writes nothing. An agent that already knows the repository identity can guard a re-enrollment explicitly:

```console
reuse-evidence enroll --ecosystem-id products --visibility private \
  --expected-repository-id cd5dfedd-6015-4ce3-9345-853e25859b0a
```

That option verifies an existing identity only. It cannot assign a fresh identity, and a mismatch refuses without writes. Change visibility only through the deliberate command:

```console
reuse-evidence set-visibility --visibility public
```

Fresh marker creation and visibility replacement publish a complete marker atomically. A private-to-public visibility change refuses without writes when the repository stewards a case whose recorded privacy is private; privateward and unchanged requests do not inspect case state. A malformed, truncated, or unsupported-version marker is refused rather than repaired or overwritten.

The marker is open, human-readable TOML with exactly these version 1 fields:

```toml
schema_version = 1
repository_id = "cd5dfedd-6015-4ce3-9345-853e25859b0a"
ecosystem_id = "products"
visibility = "private"
```

`repository_id` is a generated opaque UUID. It contains no repository path, directory name, Cargo package identity, or npm package identity. `ecosystem_id` is a declared reporting label; it does not partition which enrolled repositories may later be compared.

Every current command path uses one of three process statuses:

| Status | Meaning |
|---:|---|
| `0` | Success |
| `1` | Unsafe failure; no no-write guarantee is claimed |
| `3` | Refusal; nothing was written, and stderr names the condition and resolution |

The default `cli` feature enables argument parsing and builds the standalone binary. Library-only consumers can exclude that dependency:

```console
cargo build --no-default-features --lib
```

## Portfolio report

Configure roots outside any repository in the platform's user-local `reuse-evidence/config.toml` file:

```toml
portfolio_roots = ["/home/alice/src", "/work/selected-products"]
```

On Linux the command uses `$XDG_CONFIG_HOME/reuse-evidence/config.toml`, falling back to `$HOME/.config/reuse-evidence/config.toml`. On macOS it uses `$XDG_CONFIG_HOME` when set, otherwise `$HOME/Library/Application Support/reuse-evidence/config.toml`; on Windows it uses `%APPDATA%\reuse-evidence\config.toml`.

Run a fresh marker-only scan with:

```console
reuse-evidence portfolio
```

One or more `--root` values replace the configured roots for that invocation:

```console
reuse-evidence portfolio --root /home/alice/src/games --root /work/tools
```

With neither configured nor supplied roots, the command refuses and names the expected configuration file. Each run walks the selected roots afresh, groups repositories with valid version 1 markers by declared ecosystem identity, and names each repository identity together with its current path and visibility. Ecosystem identity is presentation only; it never filters the selected roots.

Duplicate repository identities are reported as conflicts with every current path and make the command refuse with status `3` until every enrolled repository has a unique stable identity. A marker carrying another integer schema version is reported by path and version without interpreting its newer fields. A present marker that cannot be read as the supported schema is reported separately by path and reason. A Git repository with no marker remains entirely absent; removing a marker withdraws its repository from the next report.

The first successful observation reports each enrolled repository as `new`. Later runs report the same stable identity at another path as `moved`, a previously observed repository missing beneath the roots scanned in that invocation as `unavailable`, and a marker whose current visibility differs from its previous observation as `visibility changed`. The current marker always wins: a stale cached identity or visibility is never presented in place of the marker, and the cache is corrected after the scan. A repository whose marker was removed is withdrawn rather than reported as unavailable; a present unreadable marker remains visible as that distinct condition.

The delta file is derived user-local state at:

- Linux: `$XDG_STATE_HOME/reuse-evidence/portfolio.toml`, falling back to `$HOME/.local/state/reuse-evidence/portfolio.toml`;
- macOS: `$XDG_STATE_HOME/reuse-evidence/portfolio.toml`, falling back to `$HOME/Library/Application Support/reuse-evidence/portfolio.toml`;
- Windows: `%LOCALAPPDATA%\reuse-evidence\portfolio.toml`.

The file is disposable and contains the absolute local paths needed to compare observations. Deleting it does not change the current enrolled set; the next successful run rebuilds it and reports the current repositories as new because no prior observation remains. The command refuses if the selected state path resolves inside any inspected repository or another recognizable Git worktree, so the derived file is excluded from repository version control by construction. It is not authoritative evidence and has no committed compatibility promise.

Portfolio reporting remains read-only with respect to every repository it inspects: only an unambiguous successful report may update the user-local delta file. State updates are serialized by a user-local lock and published atomically; an unchanged observation preserves the existing state file. The command performs no network access and emits no score, ranking, percentage, or health metric. Paths shown in the interactive portfolio report are local operational context; they are not recorded case evidence.

## Resolve prepared-proposal staging

Name the user-local directory where capture may keep prepared proposals:

```console
reuse-evidence staging-directory
```

The command prints exactly one path: `<state-home>/reuse-evidence/prepared-proposals`. It uses the same platform state-home precedence described for `portfolio.toml` above and has no separate configuration or override. When the state home cannot be determined, the command refuses and names the platform environment variables that can resolve it.

Resolution reuses the portfolio state's outside-repository guard. A path inside a configured portfolio repository, or inside any other recognizable Git repository, refuses with status `3`. The command creates no directory, file, lock, cache, or other state, and performs no network access. The caller creates the directory only when it has a prepared proposal to write there.

## Open a case

Prepare a TOML proposal that contains a generated UUID version 4 case identity, the proposed responsibility, and at least two occurrences:

```toml
case_id = "00000000-0000-4000-8000-000000000011"
responsibility = "normalize durable event identities"

[[occurrences]]
repository_id = "00000000-0000-4000-8000-000000000013"
consumer = "rust-release-tool"
independence = "separate release lifecycle"

[[occurrences.evidence]]
kind = "commit"
reference = "1111111"
path = "src/event.rs"

[[occurrences]]
repository_id = "00000000-0000-4000-8000-000000000014"
consumer = "web-deployment-tool"
independence = "independent npm workspace and owner"

[[occurrences.evidence]]
kind = "commit"
reference = "2222222"
path = "packages/events/src/id.ts"
```

`commit` is the version 1 evidence kind. Its `reference` is required; `path` is optional and, when present, must be repository-relative without `..`. The proposal carries no Cargo-specific field.

Preview the exact event and computed privacy consequence without writing:

```console
reuse-evidence case open --proposal open-case.toml --root /home/alice/src --preview
```

The `event:` section is itself an accepted prepared proposal. Save those exact event bytes after approval, then omit `--preview` to create `reuse-evidence/cases/<case-id>/0001-case-opened.toml` in the enrolled repository containing the current directory:

```console
reuse-evidence case open --proposal approved-case-opened.toml --root /home/alice/src
```

One or more `--root` values select the portfolio roots used to resolve every participant's stable identity and declared visibility. With no override, the command uses the same user-local portfolio configuration as `portfolio`. It scans markers without updating the derived portfolio state and writes only the opening event in the steward repository.

The event is open TOML with schema version 1, sequence 1, a generated event UUID, a command-supplied `recorded_at` UTC RFC 3339 timestamp, the case identity, proposed responsibility, steward identity, privacy consequence, and the complete occurrences. Applying a prepared preview validates its envelope against the current steward and participant visibility, then preserves the approved bytes exactly. Absolute local paths are refused. A public steward with any private participant is refused before writing; a private steward records the case as private.

Repeating the exact proposal reports the existing case with success and preserves every byte. Reusing its case identity for different proposed content refuses. The opening event is published by exclusive atomic create, so interruption cannot expose a partial file at the authoritative event path. Decisions and verification remain outside this command.

## Append an occurrence

Prepare one later occurrence using the same consumer and evidence fields as an opening proposal:

```toml
[occurrence]
repository_id = "00000000-0000-4000-8000-000000000015"
consumer = "desktop-packager"
independence = "separate distribution contract"

[[occurrence.evidence]]
kind = "commit"
reference = "3333333"
path = "src/package.rs"
```

Recover the current revision with `case show`, then preview the exact next event without writing:

```console
reuse-evidence case append 00000000-0000-4000-8000-000000000011 \
  --expected-revision 1 --proposal append-occurrence.toml \
  --root /home/alice/src --preview
```

Save the exact `event:` bytes after approval and repeat without `--preview`. A successful revision-1 append exclusively creates `0002-occurrence-appended.toml`, modifies no existing event, and reports the case identity, file, resulting revision, derived state, readiness basis, and privacy consequence. A third occurrence derives `review-ready` with `readiness_basis: occurrence-count`; the receipt explicitly states that this authorizes semantic review and does not authorize extraction.

An expected revision that does not match refuses without writing. Retrying the exact prepared event at its occupied sequence reports the occurrence as already recorded with success; a different event identity at that sequence is a revision conflict. Unknown cases, a repeated participant-and-consumer pair, and a private participant under a currently public steward also refuse without writing. Participant resolution uses the same `--root` overrides or user-local portfolio configuration as case opening. Because an exact retry writes nothing, it succeeds even when participants cannot be resolved: every later event type then reports `privacy: unknown` and a footer naming whether no root selection is configured or a recorded participant no longer resolves to exactly one enrolled repository. Every later-event writer serializes its fresh revision check through a transient operating-system lock on the immutable opening event, then holds that lock through exclusive creation of the typed next event; concurrent writers against one revision cannot both publish.

## Authorize early review

An early-review override applies only to a watching case whose two occurrences do not already satisfy the ordinary review threshold. Prepare a TOML proposal that records all three required parts of the human decision:

```toml
reason = "coordinated compatibility fixes are already required"
review_appetite = "compare the two contracts for at most one working day"

[[evidence]]
kind = "commit"
reference = "4444444"
path = "docs/compatibility.md"
```

Recover the current revision with `case show`, then preview the exact event without writing:

```console
reuse-evidence case override 00000000-0000-4000-8000-000000000011 \
  --expected-revision 1 --proposal early-review.toml \
  --root /home/alice/src --preview
```

Save the exact `event:` bytes after approval and repeat without `--preview`. A successful revision-1 override exclusively creates `0002-early-review-authorized.toml`, modifies no existing event, and reports the case identity, file, resulting revision, `state: review-ready`, `readiness_basis: early-review-override`, the review-only authorization notice, and the current privacy consequence. One or more `--root` values select the portfolio roots used to resolve every recorded participant, with the user-local portfolio configuration used when no `--root` override is supplied. The consequence is private when the immutable case privacy, the steward's current declared visibility, or any participant's current declared visibility is private. An exact no-write retry remains successful if current portfolio conditions are unavailable and conservatively reports private rather than making an unsupported public claim.

The command refuses a missing reason, evidence collection, review appetite, or expected revision; an empty evidence collection; a stale revision; an unknown case; a current public steward for a private case; a second override; and an override on a case already review-ready from three or more occurrences. Every refusal exits with status `3` and writes nothing. Retrying the exact prepared event at its occupied sequence reports that early review is already authorized with status `0`, preserves every file byte-for-byte, and reports privacy exactly as an exact append retry does. Once recorded, the early-review override remains the displayed readiness basis even if a later append raises the occurrence count to the ordinary threshold.

## Record an accepted reuse decision

A decision applies only to a review-ready case, whether readiness came from three independent occurrences or a recorded early-review override. The proposal records the responsibility verdict independently from the chosen action, plus the complete human-accepted scope and verification contract. For example, a change-authorizing proposal has this shape:

```toml
identity_verdict = "same_responsibility"
action = "publish_public_package"
accepted_scope = "the durable event identity contract"
non_responsibilities = ["case lifecycle storage"]
compatibility_consequences = "preserve the existing event identity spelling"
verification_conditions = ["all named consumers pass their public contract tests"]
invariant_contract = "one opaque UUID identifies one immutable event"
required_consumer_level_tests = ["each consumer round-trips an event identity"]
rollback_or_resplitting_path = "restore consumer-local implementations"

[[affected_consumers]]
repository_id = "00000000-0000-4000-8000-000000000013"
consumer = "rust-release-tool"
expectation = "migrate after the package publishes"

[[alternatives_rejected]]
alternative = "retain intentional duplication"
reason = "coordinated fixes already cross the consumer boundary"

[[existing_packages_considered]]
package = "uuid"
fit = "supplies identifiers but not the event contract"
reason = "the invariant remains portfolio-owned"

[[migration_expectations]]
order = 1
expectation = "publish the invariant contract and its tests"
```

Recover the current revision with `case show`, then preview the exact accepted event without writing:

```console
reuse-evidence case decide 00000000-0000-4000-8000-000000000011 \
  --expected-revision 3 --proposal reuse-decision.toml \
  --root /home/alice/src --preview
```

Save the exact `event:` bytes after human approval and repeat without `--preview`. The command exclusively creates one new event, modifies no existing event, and derives `state: awaiting-verification` with no readiness basis. Its receipt reports the case, event path, resulting revision, privacy consequence, and whether the action authorizes implementation outside this lifecycle; this command never performs that implementation.

The identity verdict is one of `same_responsibility`, `different_responsibilities`, `insufficient_evidence`, or `existing_abstraction_is_wrong`. The permitted actions are `retain_intentional_duplication`, `wait_for_more_evidence`, `use_existing_dependency`, `extract_or_deepen_locally`, `create_workspace_package`, `create_private_cross_repository_package`, `publish_public_package`, `centralize_schema_specification_or_fixture_corpus`, `replace_copies_with_generated_artifacts`, `contribute_missing_behavior_upstream`, and `split_inline_or_narrow_existing_abstraction`.

The first two actions authorize no implementation and must omit `invariant_contract`, `existing_packages_considered`, `required_consumer_level_tests`, `migration_expectations`, and `rollback_or_resplitting_path`. Every other action requires all five with non-empty content. Every affected repository-and-consumer pair must already be evidenced by an occurrence in the case. A watching case, stale expected revision, second decision, unrecognized verdict or action, or private case under a currently public steward refuses without writing. An exact prepared-event retry succeeds without writing even when current portfolio roots or participants are unavailable; a different identity at its occupied sequence is a revision conflict.

## Project the implementation brief

After a decision is recorded, project its bounded handoff from anywhere inside the steward repository:

```console
reuse-evidence case brief 00000000-0000-4000-8000-000000000011 \
  --root /home/alice/src
```

The command takes only the case identity and optional repeated portfolio roots. It reads the opening responsibility, the recorded occurrence evidence, and the accepted decision; it requires no proposal or expected revision. It writes no event, generated brief, cache, or other artifact.

For an implementation-authorizing action, the output carries the accepted responsibility identity, every evidence-bearing consumer and any accepted expectation placed on it, the invariant contract, non-responsibilities, chosen action and scope, rejected alternatives, packages considered, consumer-level tests, compatibility and release consequences, migration order, rollback or re-splitting strategy, and verification conditions. For `retain_intentional_duplication` or `wait_for_more_evidence`, it succeeds while stating that no implementation is authorized, then renders the decision fields that explain that result. A case with no accepted decision refuses and reports its current derived state; an identity not stewarded by the current repository also refuses.

With resolvable portfolio roots, the brief reports the same current private-dominance consequence as event receipts. Without portfolio configuration it still succeeds, reports `privacy: unknown` with the conservative conditions-unavailable footer, and renders the handoff from durable case state. The text is command output under the version 0.x policy, not a separately authored or compatibility-promised document.

## Record verification and dispose of a case

Verification applies to a case with one accepted reuse decision. Prepare one result for every verification condition in the decision's recorded order and one result for every affected repository-and-consumer pair, then choose the human disposition:

```toml
disposition = "closed" # closed | parked | reopened

[[condition_results]]
condition = "all named consumers pass their public contract tests"
outcome = "met" # met | not_met | accepted_exception

[[condition_results.evidence]]
kind = "commit"
reference = "abc1234"
path = "tests/contract.rs"

[[consumer_results]]
repository_id = "00000000-0000-4000-8000-000000000013"
consumer = "rust-release-tool"
outcome = "met"

[[consumer_results.evidence]]
kind = "commit"
reference = "def5678"

[[consumer_results]]
repository_id = "00000000-0000-4000-8000-000000000014"
consumer = "web-deployment-tool"
outcome = "accepted_exception"
exception = "the accepted decision retained this language-specific adapter"
```

`met` and `not_met` require at least one recoverable evidence reference. `accepted_exception` instead requires a non-empty explicit reason and may omit evidence; `exception` is refused for every other outcome. Evidence uses the same version 1 `commit` kind and optional repository-relative path as occurrence evidence. The command records references only: it does not run tests, builds, scripts, or any other repository command.

Recover the current revision with `case show`, then preview the exact event and privacy consequence without writing:

```console
reuse-evidence case verify 00000000-0000-4000-8000-000000000011 \
  --expected-revision 4 --proposal verification.toml \
  --root /home/alice/src --preview
```

Save the exact `event:` bytes after human approval and repeat without `--preview`. The command exclusively creates the next `NNNN-verification-recorded.toml`, modifies no earlier event, and reports the case, event file, resulting revision, derived state, current privacy, and disposition. A prepared exact retry succeeds without writing. If no portfolio roots are then available, that retry reports `privacy: unknown` with the conditions-unavailable footer rather than making a public claim.

Completeness is checked against the standing decision, not against a new question set. A missing, extra, duplicated, reordered, or textually changed condition refuses; each condition is repeated exactly in its recorded position. A missing, extra, or duplicated affected repository-and-consumer pair also refuses. The compiled command checks coverage and consistency only; the human remains responsible for judging whether a result is met and whether an exception is acceptable.

`closed` requires every condition and consumer result to be `met` or `accepted_exception`. Any `not_met` result admits only `parked` or `reopened`. A parked or reopened case may record another verification against the same accepted decision; the latest disposition derives its state while every earlier verification remains visible in `case show`. A closed case is terminal in version 0.1 and refuses every new later event. It cannot be reopened without a separately accepted capability.

Every refusal exits with status `3` and writes nothing. This includes an unreadable or malformed proposal; a missing expected revision or proposal; an unrecognized disposition or outcome; empty required text; missing required evidence; an invalid or absolute evidence path; an exception missing its reason or attached to another outcome; incomplete or inconsistent condition or consumer coverage; closure over `not_met`; verification before a decision; a new event on a closed case; a stale revision; an unknown or unstewarded case; a marker fault; unresolved participants on a fresh verification; a private case under a currently public steward; and a different event identity or different bytes at the occupied sequence. Publication uses the same opening-event lock and exclusive atomic create as every later case event, so concurrent writers against one revision cannot both publish and interruption cannot expose a partial authoritative event.

## Find cases across the portfolio

From any working directory, find every case stewarded by repositories enrolled beneath the selected portfolio roots:

```console
reuse-evidence case find --root /home/alice/src
```

With no `--root` override, the query uses the user-local portfolio configuration. It refuses when neither source selects a root, because an empty report would otherwise look like evidence that no cases exist. Each case row reports its identity, steward repository identity and local path, proposed responsibility, current revision, derived state, and current complete privacy. Complete privacy is private when the case was recorded private, the steward or any uniquely resolved participant is currently private, or any participant's current visibility cannot be resolved uniquely. A case found in another enrolled repository is reported the same way as one in the current repository.

The query freshly discovers marker-enrolled repositories and reads authoritative event streams only from those enrollments. It does not update the portfolio state file or create an index, cache, projection, or other artifact, and it performs no network access. If one identified case has damaged event history, that row reports `condition: damaged-recorded-event-history`, unavailable derived fields, `privacy: unknown`, and the exact recovery detail; healthy cases remain visible. This shape is separate from the steward-local `case list` and `case show` projections.

## Read cases

From anywhere inside an enrolled steward repository, list every case it owns without requiring portfolio configuration:

```console
reuse-evidence case list
```

The listing reports each case identity, current revision, occurrence count, and state. Two occurrences derive `watching`; three or more derive `review-ready` with `readiness_basis: occurrence-count`; a recorded early-review override derives `review-ready` with `readiness_basis: early-review-override`; and a recorded decision dominates either route and derives `awaiting-verification`. The latest verification disposition then derives `closed`, `parked`, or `reopened`. Awaiting-verification and all three disposed states carry no readiness basis and authorize no review. Every review-ready result states that it authorizes semantic review and does not authorize extraction.

Show the responsibility and complete recorded evidence history for one case with:

```console
reuse-evidence case show 00000000-0000-4000-8000-000000000011
```

Both commands rebuild their output directly from the steward's event files on every invocation and write nothing. `case show` renders a recorded override's reason, review appetite, and evidence references, followed by every verification in event order with its condition results, consumer results, evidence, accepted exceptions, and disposition. A later closure never hides an earlier failed verification. With configured portfolio roots, or one or more explicit `--root` values, both reads freshly resolve participant markers and report whether the current case is `privacy_conflicted` or `stale`, including for closed, parked, and reopened cases. Without roots, the steward-local read still succeeds and reports those two current conditions as unknown.

A current public steward with recorded-private case evidence or any currently private recorded participant is privacy-conflicted. If no definite conflict is established and any participant identity does not resolve to exactly one discoverable enrollment, `privacy_conflicted` is unknown and the case is stale; a definite conflict remains true even when another participant is unresolved. Historical occurrences remain fully visible. A duplicate or missing event sequence makes the read refuse with the condition and recovery action instead of deriving a plausible result from damaged history. No cache, index, projection file, score, percentage, ranking, duplication measure, or health metric is produced.

## Skill governance

The `reuse-evidence` binary mounts the command surface from the published `skill-evidence` crate under its own `skills` subcommand:

```console
reuse-evidence skills evidence install --root .
```

The registry dependency resolves to `skill-evidence` 0.11.0 in `Cargo.lock`; it is not a path or Git dependency. The host identity is `reuse-evidence` for the schema namespace, command, and Cargo package. Its operator-skill directory is resolved from this crate's own manifest directory, never from the repository supplied through `--root`.

The install command writes four operator packages under `.claude/skills/`, relative discovery links under `.agents/skills/`, and the two versioned contracts under `schemas/skill-evidence/`. A non-force install refuses with status `3` if any installed file differs, names every differing file, and writes nothing. `--force` is the explicit replacement operation.

The mounted subtree's command contract and operator packages are versioned upstream by `skill-evidence`, not independently by this crate. `reuse-evidence` supplies the host identity and maps upstream outcomes onto the same process meanings used by its own commands: `0` success, `1` unsafe failure, and `3` refusal. Upstream diagnostic wording is not a byte-stable promise of this project.

Dependency installation and upgrades do not migrate, rewrite, reorder, or merge `reports/skill-evidence/` receipts. A changed operator package has different content and therefore a new content hash; prior receipts remain historical evidence. This dependency governs this repository's skill assets only. It is not used for reuse-case events, readiness, decisions, briefs, or verification, and does not establish or share a lifecycle kernel between the projects.

## Intended lifecycle

1. After material implementation work, a maintainer manually invokes capture.
2. A first consumer creates no durable reuse record.
3. A second independent consumer opens a watching case.
4. A third independent consumer normally makes the case ready for review; a narrowly justified human override may authorize review after the second.
5. Review may recommend extraction, an existing dependency, generation, a shared contract, intentional duplication, deferral, or splitting a wrong abstraction.
6. The maintainer accepts or rejects the exact decision.
7. Ordinary engineering skills or tools implement any accepted change.
8. `reuse-evidence` verifies the accepted migration and closes, parks, or reopens the case.

The third consumer authorizes review. It never authorizes extraction by itself.

## Non-goals

`reuse-evidence` must not become:

- a general code-quality or architecture score;
- a universal duplication percentage;
- a built-in semantic clone detector;
- a CI gate for unreviewed candidates;
- an automatic abstraction or refactoring engine;
- a hosted portfolio service;
- a product-line framework inferred from thematically related repositories;
- a stream of clean-run certification receipts;
- or a shared infrastructure kernel extracted prematurely from this repository and `skill-evidence`.

## Documentation authority

Start with [the documentation map](docs/README.md).

The normative authority order begins at [docs/principles/README.md](docs/principles/README.md). All future design documents, PRDs, issues, code, schemas, and skills must conform to the principles and accepted ADRs or amend the higher authority first through an explicit human decision.

## Repository orientation

- [CONTEXT.md](CONTEXT.md) — shared vocabulary.
- [CLAUDE.md](CLAUDE.md) — agent operating instructions.
- [docs/principles/](docs/principles/) — constitutional principles.
- [docs/adr/](docs/adr/) — accepted architectural decisions.
- [docs/design/v0.1-scope-and-acceptance.md](docs/design/v0.1-scope-and-acceptance.md) — bounded first implementation target.
