# reuse-evidence

**Evidence-gated reuse decisions for agent-developed repository portfolios.**

`reuse-evidence` is intended to help a maintainer notice when independently maintained consumers are accumulating the same responsibility, preserve the evidence, and make an explicit decision before the implementations become expensive to keep aligned.

The project is deliberately not a clone detector and not an automatic refactoring system. Similar code is only a clue. A reuse decision must establish that the consumers actually share a responsibility, that the common behavior has a coherent owner, and that sharing it will create more leverage than coupling.

## Status

**Repository enrollment, marker-only portfolio reporting with derived change state, durable case opening, and skill governance implemented.**

The public Rust crate and standalone `reuse-evidence` binary can enroll a Git repository, including an npm workspace with no Cargo project. Enrollment writes a human-readable version 1 TOML marker at the nearest repository root, safely revalidates an existing marker without minting another identity, and uses the binary's shared success, unsafe-failure, and refusal exit meanings.

Enrollment refuses implicit visibility, ecosystem-identity, or repository-identity conflicts and refuses malformed, truncated, or unsupported-version markers without rewriting them. Declared visibility can be changed only through the dedicated `set-visibility` command. The portfolio command freshly scans configured roots for marked Git repositories and reports current enrollment, duplicate identities, unsupported or unreadable markers, and new, moved, unavailable, or visibility-changed repositories. The case command can preview and atomically open a steward-local case from two or more evidenced occurrences while enforcing enrollment, stable identity, idempotency, and private dominance. The binary also mounts the published `skill-evidence` lifecycle under `reuse-evidence skills` and this repository commits the four operator packages it installs. Appending or reading cases, deriving readiness, capture, review, verification, and this project's own `reuse-evidence-*` skill packages are not implemented yet.

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

Fresh marker creation and visibility replacement publish a complete marker atomically. A malformed, truncated, or unsupported-version marker is refused rather than repaired or overwritten.

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

Repeating the exact proposal reports the existing case with success and preserves every byte. Reusing its case identity for different proposed content refuses. The opening event is published by exclusive atomic create, so interruption cannot expose a partial file at the authoritative event path. Case append, read, readiness, decisions, and verification remain outside this command.

## Skill governance

The `reuse-evidence` binary mounts the command surface from the published `skill-evidence` crate under its own `skills` subcommand:

```console
reuse-evidence skills evidence install --root .
```

The registry dependency resolves to `skill-evidence` 0.2.1 in `Cargo.lock`; it is not a path or Git dependency. The host identity is `reuse-evidence` for the schema namespace, command, and Cargo package. Its operator-skill directory is resolved from this crate's own manifest directory, never from the repository supplied through `--root`.

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
