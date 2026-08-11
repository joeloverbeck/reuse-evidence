# ADR 0015: The portfolio location is resolved at the edge and carries its own absence

**Status:** Accepted  
**Date:** 2026-08-11  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md`](../principles/PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

Commit `9578d70`, "Move the clock and the portfolio out of ambient state," moved the clock and did not touch `src/portfolio.rs`. `RecordedInstant` is now constructed at `src/main.rs:224`, `:257`, `:309`, `:343` and carries ten unit tests in `src/case/instant.rs`. The portfolio kept reading the process environment at its point of use.

`src/portfolio.rs` is the only file in the crate that reads the environment. `nonempty_environment_path` (`src/portfolio.rs:888`) is reached through `optional_config_path` (`:834`), `config_path` (`:825`), and `state_path` (`:839`), which are called from `selected_roots` (`:642`), `selected_roots_if_configured` (`:649`), and `report` (`:108`). Platform precedence over `APPDATA`, `LOCALAPPDATA`, `XDG_CONFIG_HOME`, `XDG_STATE_HOME`, and `HOME` is written six times, at `:851`, `:856`, `:861`, `:869`, `:875`, and `:883`. None of the six has a unit test; `src/portfolio.rs` has two, both on the pure functions `derive_changes` and `next_state`.

Roots themselves are already injectable. Both root entry points return before any environment read when `root_overrides` is non-empty (`:652`, `:665`), and `scan` (`:722`) takes roots as a value and reads no environment at all. What has no value-level expression is narrower and exact: the **user-local configuration file** and the **user-local state file**.

The cost of that gap is measured, not predicted. `run_without_portfolio_configuration` (`tests/case_cli.rs:125`) exists only to spawn a subprocess with `XDG_CONFIG_HOME` pointed at a directory that does not exist, and is used **33 times**. `run_without_config_environment` (`tests/portfolio_cli.rs:83`) does the same by removing five variables. Across the three CLI test files there are 23 XDG references. `ListOutcome`, `ShowOutcome`, and `BriefOutcome` are already `pub` with `Display` (`src/case/read.rs:154`, `:160`, `:166`, `:418`, `:356`, `:172`), so these tests could assert `outcome.to_string()` in process today; the single reason they fork is that "configuration directory resolvable, configuration file absent" cannot be said as a value.

One command reads the same configuration file twice. `case brief` calls `reported_privacy` (`src/case.rs:1690`) from `src/case/read.rs:539`; that reads the file through `selected_roots_if_configured` (`src/case.rs:1696`) and then calls `derive_complete_case_privacy` (`:1668`), whose `resolve_participants` (`:2691`) reads it again through `selected_roots` (`:2695`). The second read cannot disagree with the first except by a concurrent edit between them, so removing it is behaviour-preserving and removes a time-of-check discrepancy rather than creating one.

The decisive constraint is that the absence of a portfolio location is **not uniformly fatal**. On identical environments:

| Command, no resolvable configuration directory | Behaviour | Pinned by |
|---|---|---|
| `portfolio` | exit 3, "the user-local configuration directory cannot be determined" | `missing_platform_config_directory_names_supported_environment`, `tests/portfolio_cli.rs:207` |
| `case list` | exit 0, `privacy_conflicted: unknown`, "portfolio conditions unavailable" | `review_r1_spec_1_case_list_succeeds_without_platform_configuration_directory`, `tests/case_cli.rs:2631` |

That divergence is the split between `selected_roots` and `selected_roots_if_configured`, and it is normative rather than incidental. `CONTEXT.md:40` defines `privacy_conflicted` as `unknown` when "no portfolio root selection resolves the participants," and the `case list` test is named as a review spec requirement. `FOUNDATIONS.md` §15 makes a degraded but usable query part of operational fitness.

This is exactly where the clock analogy fails. `RecordedInstant::now()?` may refuse at the edge because a clock that cannot be read is fatal for every command that needs one. A portfolio location that cannot be determined is fatal for `portfolio` and benign for `case list`, `case show`, and `case brief`. Resolution may move to the edge only if the resolved value **carries** the absence rather than raising it.

Refusal ordering is a second reason eager failure is unsafe. `case brief` refuses "case has no accepted reuse decision" at `src/case/read.rs:531` **before** it touches the portfolio at `:539`. Any design that reads or validates the configuration at the edge fires configuration diagnostics ahead of domain refusals that today come first.

## Decision

The portfolio location is resolved once at the process edge, as a value that represents its own absence.

- One type owns the user-local location: the selected root overrides, the user-local configuration file path, and the user-local state file path. The two file paths are optional. Resolving the location performs **environment reads only** — no filesystem I/O, no validation, and no refusal.
- The type replaces `root_overrides: &[PathBuf]` at the 19 declaration sites that already carry it — `src/portfolio.rs` ×4, `src/case.rs` ×11, `src/case/read.rs` ×4 — including the eight public entry points `case::open`, `case::append`, `case::authorize_early_review`, `case::decide`, `case::list`, `case::show`, `case::brief`, and `portfolio::report`. No signature gains a second parameter.
- The type is `pub`, constructed from the environment for the binary and constructed directly from explicit paths for tests and library consumers.
- Platform precedence is written once behind the six `#[cfg]` variants it replaces, and is unit-tested.
- **Every consumer keeps today's interpretation of absence at today's call site.** `selected_roots` still refuses with the same text; `selected_roots_if_configured` still returns `Ok(None)`; `report` still refuses on an undeterminable state directory at `src/portfolio.rs:108`. Exit codes, refusal text, and refusal ordering are unchanged.
- `case brief` reads the user-local configuration at most once: `reported_privacy` resolves the selection and passes the selected roots down, rather than letting `resolve_participants` re-select them.

This authorizes one resolved location value and one platform-precedence owner. It does **not** authorize:

- resolving root *selection* at the edge, which means reading or validating the configuration file there;
- collapsing `selected_roots` and `selected_roots_if_configured` into one entry point, or otherwise reconciling the `portfolio` / `case list` divergence above;
- any change to refusal text, exit codes, refusal ordering, or receipt output;
- a command-line flag, configuration key, or environment variable for the configuration or state file path;
- moving, caching, or changing the format of the user-local state file;
- rewriting the CLI test suite, or converting any existing test from the process boundary; the location merely makes in-process expression possible.

`CONTEXT.md` gains **Portfolio location** as a core term, distinguished from **Portfolio root**: a root is a scanned directory, the location is where the tool resolves roots and state from.

## Consequences

### Positive

- Platform precedence has one home and its first unit tests, replacing six untested `#[cfg]` variants.
- The 33 uses of `run_without_portfolio_configuration` and the `portfolio` no-environment case become expressible in process, against `Display` output that is already public.
- The public entry points name the side effect they have. Today `case::list` reads `XDG_CONFIG_HOME` and its signature does not say so.
- `case brief` performs one user-local configuration read instead of two.
- A ninth entry point inherits the location instead of reaching for the environment.

### Negative and risks

- 19 declaration sites and roughly 47 mention sites change type in one mechanical, compile-checked pass. The change is wide and shallow, and a partial application does not compile.
- The library's public signatures change. This is permitted under `CONSUMER-CONTRACT.md` §8 at `0.x` and is not permitted silently after `1.0.0`.
- Carrying `Option<PathBuf>` moves the "which absence is fatal here?" question to each call site instead of deleting it. That question is genuinely per-command, but the type does not enforce the answer, so a future entry point can choose the wrong one. Only the two spec tests above pin it.
- The win is availability, not conversion. Nothing here makes the 33 tests move; it makes moving them possible.

### Operational burden

Adding a command means threading one existing parameter and choosing, once, whether an unresolvable location refuses or degrades. Adding a platform means one `#[cfg]` variant in one function.

### Compatibility and migration

No recorded evidence, marker, configuration, or state format changes, so `CONSUMER-CONTRACT.md` §3 and §5 are untouched. §1 command behaviour is unchanged: no refusal text, exit code, ordering, or receipt line differs, which the existing CLI suite already asserts byte-exactly. The Rust surface changes under the §8 `0.x` policy.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| `PortfolioLocation::resolve()?` at `main.rs`, as the source review proposed | Rejected | Measured against the suite: raising at the edge converts `case list` without a configuration directory from exit 0 with `privacy_conflicted: unknown` to exit 3, breaking `tests/case_cli.rs:2631` and contradicting `CONTEXT.md:40`. The literal clock analogy does not survive the fact that a missing location is fatal for one command and benign for three. |
| Resolve root *selection* at the edge (`Selected` \| `Unconfigured`) | Rejected | It requires reading the configuration file at the edge, which fires "is not UTF-8" and "is invalid" diagnostics ahead of domain refusals that precede the portfolio today, notably `src/case/read.rs:531`. It also pressures the two root entry points into one, and that divergence is specified behaviour. It would additionally remove the second configuration read, which this decision achieves without touching refusal order. |
| Remove `case brief`'s duplicate configuration read only | Rejected as the whole change, adopted as part of it | It banks one filesystem read and leaves the six untested platform variants and all 33 forking tests exactly as they are. It is orthogonal to injection, not a smaller version of it, so it is included rather than chosen. |
| Add a `--config` / `--state` flag instead | Rejected | It adds permanent `CONSUMER-CONTRACT.md` §1 surface to solve a test-reachability problem, and invites the user-local location to become per-invocation state that ADR 0003 deliberately keeps user-local. |
| Give `scan` the location too | Rejected | `src/portfolio.rs:722` already takes roots as a value and reads no environment. The source review listed scanning as environment-bound; it is not. |
| Park until a ninth entry point exists | Rejected | The cost is already paid 33 times in one file, and the six platform variants are the only untested branch class in the module. Waiting means threading the environment through a ninth signature first. |
| Record this in the implementing issue instead of an ADR | Rejected | The durable content is a restraint, not a refactor: absence is carried, never raised at the edge, and the `portfolio` / `case list` divergence is deliberate. Both read as unfinished work and would be "tidied" by the next review — this review proposed exactly that. |

## Verification and review trigger

The decision is fit if, after implementation, `src/portfolio.rs` contains one platform-precedence owner with unit tests, no file outside it reads the environment, the full CLI suite passes unchanged with no edit to any expected string or exit code, and at least one test that previously required `run_without_portfolio_configuration` runs in process against `Display` output.

Falsify it if preserving the `portfolio` / `case list` divergence requires the location type to grow per-command variants, which would mean the invariant is the decision rather than the value. Falsify it also if threading the type reaches significantly beyond the 19 sites that carry `root_overrides` today, which would mean the parameter was not in fact already present everywhere it is needed.

Reopen it if a second consumer needs the user-local location for something other than portfolio discovery, if a location must vary per invocation, or at `1.0.0`, when `CONSUMER-CONTRACT.md` §8 stops permitting free signature change.

## Supersession

None. ADR 0003 continues to govern what a portfolio root is and how enrollment is discovered; this decision governs only where the location is resolved. It disturbs no restraint held by ADR 0010, 0011, 0013, or 0014.
