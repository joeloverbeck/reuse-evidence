# ADR 0020: What the capture skill may not own

**Status:** Accepted  
**Date:** 2026-08-12  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`EVIDENCE-AND-DECISIONS.md`](../principles/EVIDENCE-AND-DECISIONS.md), [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](../principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md), [`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md`](../principles/PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

The compiled case lifecycle is complete for version 0.1. `src/main.rs:58` exposes `open`, `append`, `override`, `decide`, `verify`, `brief`, `list`, and `show`, and [`README.md`](../../README.md):15 records what is missing: "Capture, reuse-review proposal authoring, and this project's own `reuse-evidence-*` skill packages are not implemented yet."

Nothing enters that lifecycle without a hand-authored file. Every write command takes `--proposal <PATH>` (`src/main.rs:63`, `:80`, `:97`, `:114`, `:131`), a TOML document carrying a generated UUID version 4 case identity and each occurrence's `repository_id` read out of that repository's marker. `design/v0.1-scope-and-acceptance.md` §2 puts `reuse-evidence-capture` in scope to close that gap, and `FOUNDATIONS.md` §15 makes it a correctness condition rather than a convenience: "A recurrent workflow the maintainer will avoid is unfit even when its data model and tests are correct."

`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §3 lists what the compiled command owns. Three of capture's mechanics appear nowhere on it, because none of them existed when §3 was written. Each is decided below on its own evidence.

### One responsibility, two authoritative records

`case::read::list` resolves `find_repository_root(working_directory)` and then reads only that repository's cases (`src/case/read.rs:192`–`:194`); `read_cases` returns an empty vector when no case directory exists (`:349`). `case::open` keys its existing-case guard on the case directory alone (`src/case.rs:827`), so no code path compares a proposed responsibility against any recorded one.

`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §4.2 makes the steward "the repository in which the second occurrence is recognized." In a cross-repository case the third occurrence is therefore recognized somewhere the case is not stewarded. Capture running there sees no case, proposes opening one, and the portfolio acquires two authoritative records for one responsibility with the evidence split between them. Both are individually valid, both publish successfully, and nothing reports the condition — the failure is silent, and it destroys the single decision-bearing history `EVIDENCE-AND-DECISIONS.md` §1 asks a case to preserve.

### A durable draft with nowhere durable to live

`case open --proposal … --preview` renders the exact event and states that "The `event:` section is itself an accepted prepared proposal. Save those exact event bytes after approval, then omit `--preview`" ([`README.md`](../../README.md):151). Those approved bytes are a recorded human decision that has not yet been applied, and they need a home.

The binary already resolves one, across three platforms: the `STATE` descriptor (`src/portfolio.rs:981`) reads `LOCALAPPDATA`, `XDG_STATE_HOME`, or `.local/state`, with `MACOS_HOME_RELATIVE` (`:988`) in place of the XDG path, and `ensure_state_outside_repositories` (`:556`) refuses any resulting path inside an inspected repository or inside any Git repository at all. A skill cannot reach it: `state_path` is private and `#[cfg(feature = "cli")]` (`:955`), and the resolved path is printed only inside refusals (`:526`, `:565`, `:577`, `:590`).

The alternative homes were measured rather than assumed. On the maintainer's machine `TMPDIR` is `~/.cache/agent-tmp` — beneath a directory the XDG base directory specification defines as non-essential data that may be deleted at any time — and it already holds 54 accumulated session directories for this repository alone, plus loose files from sibling projects at its root. `systemd-tmpfiles` sweeps `/tmp` at 10 days and `/var/tmp` at 30. An approved proposal in any of them is lost by ordinary housekeeping, and the maintainer approves the same decision twice. `design/v0.1-scope-and-acceptance.md` §4 requires the slice to resume from durable state with no "repeated decision" needed.

An unapproved draft carries no such cost: it is derived from recoverable repository evidence, so re-previewing reproduces it without reconstructing anything from a transcript.

### A fixed statement with a misleading nearest precedent

`EVIDENCE-AND-DECISIONS.md` §5 requires a clean capture to "return a fixed terminal statement," ADR 0001 repeats it, and `FOUNDATIONS.md` §15 makes it one of the four parts of routine clean capture. No command prints it.

ADR 0017 gave case terminal text one owner in `case::render`, reached one way. A contributor reaching for the nearest precedent concludes the fixed statement belongs there — the reasoning that produced this ADR made exactly that error before checking. ADR 0017's subject is *how a case prints*: the event-receipt spine, the three query shapes, the shared readiness lines, the privacy line and its footers. A clean capture has no case, so ADR 0017 does not reach it and supplies no owner.

A command printing the sentence would also assert nothing the binary can check. It would print on the say-so of whoever invoked it — the same assurance the skill already carries, at the cost of a new versioned command under `CONSUMER-CONTRACT.md` §1 whose entire subject is one sentence.

## Decision

The compiled command owns a capture mechanic whose silent failure would corrupt an authoritative record or lose a recorded human decision. The skill owns what the command cannot verify.

1. **Cross-portfolio case lookup is compiled.** A read-only query over enrolled repositories reports, per stewarded case, its identity, steward, responsibility, revision, state, and privacy. It writes nothing and derives no index.
2. It prints **its own shape**. ADR 0010 refused collapsing the case queries into the receipt spine and ADR 0012 refused folding the brief into `case show`; this query answers a third question and prints a third shape. Its renderer lives in `case::render` and its entry point and outcome value in `case::read`, under ADR 0017.
3. It resolves participants through `PortfolioLocation` at the edge under ADR 0015 and **refuses** when no root selection resolves. A portfolio-wide query without a portfolio has no subject. ADR 0015 leaves each consumer its own reading of that absence, and `case list`'s degradation to unknown conditions is the other reading, not a precedent binding this one.
4. **Disclosure follows `portfolio`.** The report is local operational context and not recorded case evidence, and the operator can already read the files it summarises. Every row carries the case's privacy, so authoring downstream knows what may not enter a public working tree. `case open`'s existing refusal of a public steward with any private participant remains the write-side guard.
5. **Draft path resolution is compiled.** One read-only command names the resolved user-local staging directory, reusing the `STATE` descriptor and the `ensure_state_outside_repositories` guard. Both the draft and the approved bytes live there, outside every repository by construction. An applied draft is removed once its event is published: the event file is authoritative, and a pile of applied drafts is the accumulating control record `FOUNDATIONS.md` §12 refuses.
6. **The fixed no-candidate terminal statement is the skill's**, stated verbatim in its package and emitted by it. No command prints it.

This authorizes two read-only commands and a placement rule. It does **not** authorize:

- a compiled `capture` command, or any command that judges whether occurrences share a responsibility — `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §3 keeps semantic judgment in review and human acceptance;
- a durable index, cache, or derived case list of any kind;
- writing a draft, an approved proposal, or any capture output into a repository working tree;
- staging inside `--preview`, whose write-free property ADR 0016 asserts in process;
- any committed record of a clean capture;
- extending this boundary to `reuse-evidence-discover`, `reuse-evidence-review`, or `reuse-evidence-status`, none of which has been designed. The staging directory is expected to serve review's decision proposals, but that is a prediction, not a grant.

## Consequences

### Positive

- One responsibility keeps one authoritative record, enforced by the surface whose failure would otherwise be silent and unreported.
- An approved decision survives an interruption, a session boundary, and any temporary-directory sweep, which is what `design/v0.1-scope-and-acceptance.md` §4 requires the slice to demonstrate.
- Three-platform path resolution keeps one owner. A skill package is a shipped product surface under `CONSUMER-CONTRACT.md` §4, so a path correct on the maintainer's machine and wrong on an adopter's would be a defect rather than a local inconvenience.
- No draft can reach a public working tree carrying a private participant's evidence, because the existing guard already refuses any such path.
- ADR 0017's scope is stated where the next reader of it will be standing.

### Negative and risks

- The version 0.1 command tree gains two commands before a single real capture has run. If capture's first real use finds the lookup unnecessary, it was built early.
- The staging directory is a new user-local artifact class. It is disposable and rebuildable, but it is one more thing on disk that no command currently prunes except by the deletion rule in item 5.
- The fixed statement lives in Markdown, where nothing tests it. Drift is caught by a human reading output, not by a suite.
- The lookup reports a private steward's responsibility text to whoever runs it. That is the same disclosure `portfolio` already makes about private repository identities and paths, but it is case evidence rather than enrollment metadata, and the reasoning that it is safe rests on the operator already having read access.

### Operational burden

Capture calls two read-only commands and writes one file. Neither command takes a lock, writes state, or touches the network. The staging directory needs no configuration: it is resolved, not declared.

### Compatibility and migration

Nothing recorded changes. Both commands are additive and read-only, so no recorded evidence, event schema, or existing terminal contract is affected. Their output is command behavior under `CONSUMER-CONTRACT.md` §1, changeable during `0.x` under §8. The staging directory holds no authoritative state and may be deleted at any time; only unapplied approved proposals are lost, and those are re-derivable at the cost of one re-approval.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| The skill walks enrolled repositories itself to find cases | Rejected | Puts the mechanic whose silent failure splits an authoritative record on agent diligence, which `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §3 reserves for the command. An agent reading raw case files also applies no disclosure rule at all, where a command applies one. |
| Scope cross-repository lookup out of the first capture slice | Rejected | `design/v0.1-scope-and-acceptance.md` §5's acceptance case C requires a candidate spanning at least two repositories, and the duplicate-record hazard would ship as a known defect in the entry point. |
| Fold the lookup into `case list` as a flag | Rejected | The same error ADR 0010 refused for receipts and ADR 0012 refused for the brief. `case list` answers what this repository stewards; a different question prints a different shape. |
| The skill computes the platform state path in prose | Rejected | A second owner of a three-platform rule the binary already implements at `src/portfolio.rs:981`, in a shipped asset where nothing tests it. |
| Keep drafts in a temporary directory | Rejected | Measured: `TMPDIR` is under `~/.cache`, already holds 54 stale session directories, and `/tmp` is swept at 10 days. Losing an approved proposal costs a repeated human decision. |
| Keep drafts in the repository, excluded by `.gitignore` | Rejected | `ensure_state_outside_repositories` exists precisely so the guarantee holds by construction rather than by configuration; a private participant's text in a public tree behind a gitignore entry is the weaker form of the same promise. |
| Stage approved bytes through a flag on `--preview` | Rejected | Puts a write inside the command whose write-free property ADR 0016 asserts directly, and lands the flag on all five write commands to serve a draft file. |
| A command that prints the fixed no-candidate statement | Rejected | It asserts nothing the binary can verify, and adds a versioned command under `CONSUMER-CONTRACT.md` §1 whose whole subject is one sentence. ADR 0017's owner does not reach it, because a clean capture has no case. |
| Record all of this in the implementing PRD | Rejected | `docs/README.md` ranks PRDs below ADRs and an issue closes. Items 1 and 5 leave a visible command behind; item 6's only other on-disk evidence is the absence of one, and the nearest precedent points the wrong way. ADR 0012 rejected this placement for the same reason. |

## Verification and review trigger

The decision is fit when a real capture in a repository that does not steward the case reports the existing case and declines to open a second one; when an approved proposal survives a session boundary and applies unchanged; and when a clean capture ends in the fixed statement with no file written anywhere.

Falsify item 1 if real use shows the lookup never fires because cases are in practice always stewarded where work happens — it would then be machinery bought against a hazard that does not occur. Falsify item 5 if the staging directory accumulates drafts that the deletion rule does not reach. Falsify item 6 if the emitted statement drifts across real runs, which makes the compiled owner the cheaper answer after all.

Reopen when `reuse-evidence-review` needs a home for decision proposals, which will test whether the staging directory generalises or was shaped around capture alone.

## Supersession

None. This decision does not amend ADR 0017; it records that ADR 0017's owner covers how a *case* prints and therefore does not reach a capture that has no case.
