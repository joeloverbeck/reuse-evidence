# reuse-evidence

**Evidence-gated reuse decisions for agent-developed repository portfolios.**

`reuse-evidence` is intended to help a maintainer notice when independently maintained consumers are accumulating the same responsibility, preserve the evidence, and make an explicit decision before the implementations become expensive to keep aligned.

The project is deliberately not a clone detector and not an automatic refactoring system. Similar code is only a clue. A reuse decision must establish that the consumers actually share a responsibility, that the common behavior has a coherent owner, and that sharing it will create more leverage than coupling.

## Status

**Repository enrollment and marker-only portfolio reporting implemented.**

The public Rust crate and standalone `reuse-evidence` binary can enroll a Git repository, including an npm workspace with no Cargo project. Enrollment writes a human-readable version 1 TOML marker at the nearest repository root, safely revalidates an existing marker without minting another identity, and uses the binary's shared success, unsafe-failure, and refusal exit meanings.

Enrollment refuses implicit visibility, ecosystem-identity, or repository-identity conflicts and refuses malformed, truncated, or unsupported-version markers without rewriting them. Declared visibility can be changed only through the dedicated `set-visibility` command. The portfolio command freshly scans configured roots for marked Git repositories and reports current enrollment, duplicate identities, and unsupported marker versions. Historical portfolio deltas, the case lifecycle, capture, review, verification, and installed skill assets are not implemented yet.

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

`--visibility` accepts exactly `public` or `private`. A successful command writes `reuse-evidence.toml` at the nearest ancestor containing `.git` and reports the path and values it wrote on stdout. It adds no dependency or manifest entry to the enrolled repository and performs no network access.

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

With neither configured nor supplied roots, the command refuses and names the expected configuration file. Each run walks the selected roots afresh, reports only Git repositories with valid version 1 markers, groups them by declared ecosystem identity, and names each repository identity together with its current path and visibility. Ecosystem identity is presentation only; it never filters the selected roots.

Duplicate repository identities are reported as conflicts with every current path and make the command refuse with status `3` until every enrolled repository has a unique stable identity. A marker carrying another integer schema version is reported by path and version without interpreting its newer fields. Unmarked or otherwise invalid repositories are absent. Removing a marker withdraws its repository from the next report.

This report is read-only: it writes no repository, cache, or configuration state, performs no network access, and emits no score, ranking, percentage, or health metric. New, moved, unavailable, and visibility-changed historical conditions require derived state and remain unimplemented.

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
