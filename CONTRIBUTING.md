# Contributing

Thanks for looking. Please read this before opening a pull request — the answer for unsolicited
pull requests is no, and the reason is structural rather than personal.

## What this repository is

`reuse-evidence` is a single-maintainer project governed by accepted written authority. Principles
live under [`docs/principles/`](docs/principles/), architectural decisions under
[`docs/adr/`](docs/adr/), and the bounded version 0.1 scope in
[`docs/design/v0.1-scope-and-acceptance.md`](docs/design/v0.1-scope-and-acceptance.md).

A change is judged by whether it conforms to that authority, not only by whether it compiles and
passes tests. A patch that is correct in isolation is still rejected when it widens a boundary the
project has explicitly refused. Those refusals are listed in
[`AGENTS.md`](AGENTS.md); several of them look like obvious improvements from outside.

## Issues are welcome

Bug reports, reproductions, and questions are genuinely useful — open an issue.

The most valuable report identifies observed behavior, the evidence bearing the claim, and the
authority it appears to violate. You do not need to supply a fix.

## Pull requests are not

**Do not open a pull request that was not requested.** It will be closed, regardless of quality.

If you want to implement something, comment on the issue and wait for the maintainer to assign it
to you. Assignment is the signal that work is wanted and that nobody else — including the
maintainer — is already on it. Without it, you are very likely duplicating work in progress.

An assigned pull request must state which accepted authority it conforms to, and must pass the full
gate below.

## `ready-for-agent` does not mean unclaimed

The triage labels in [`docs/agents/triage-labels.md`](docs/agents/triage-labels.md) describe the
maintainer's own workflow. `ready-for-agent` means an issue is specified tightly enough for the
*maintainer's* unattended agent to implement. It is not an invitation and does not mean the issue
is available.

Issue bodies here are deliberately precise — file paths, function names, scope boundaries, and the
intended remedy. That precision exists so the maintainer's agent converges quickly. It also means
two independent implementations of the same issue tend to come out nearly identical, so a race
produces no salvageable work for whoever loses it.

## The gate

Every change must pass, on Rust 1.93:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --no-default-features --all-targets -- -D warnings
cargo test --all-features
cargo test --no-default-features
```

Run `cargo test --no-default-features` against a clean target directory when you change test
wiring. The command-line integration tests are gated behind `required-features = ["cli"]`; without
that gate they resolve a stale binary from an earlier build and report a false pass.

## AI assistance

Disclose it. This project is built with agent assistance and has no objection to the practice, but
an undisclosed agent-generated patch that contradicts an accepted ADR wastes review time that the
disclosure would have saved.

## License

Contributions are accepted under the [MIT License](LICENSE).
