# ADR 0009: One file per case event

**Status:** Accepted  
**Date:** 2026-08-10  
**Decision owner:** Repository maintainer  
**Governing principles:** [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md), [`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md`](../principles/PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md), [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md)

## Context

Repository participation is implemented; the durable case record is not. The next implementation slice must write authoritative case events, and the layout it chooses is the project's least reversible commitment: `CONSUMER-CONTRACT.md` §3 observes that pinning an older binary cannot undo history already written.

Two shapes were live. This repository already commits `reports/skill-evidence/*/events.jsonl`, so an append-only JSON Lines stream per case is visible prior art on disk. A single rewritten document per case is simpler to read.

Both were assessed against obligations that already bind: `FOUNDATIONS.md` §11 requires expected-revision protection, atomicity, and idempotency; `CONSUMER-CONTRACT.md` §2 requires that historical events not be silently rewritten; §9 requires evidence to stay inspectable in repository files or documented open formats.

The steward repository is a Git repository under ordinary branch-and-merge work. Appending to a shared line-oriented file means two branches that each record an occurrence both edit the same final-line region, producing merge conflicts inside authoritative evidence during normal use. Rewriting a whole-case document means every append puts the entire recorded history — including occurrences recorded weeks earlier — inside one write's failure window.

Without this decision recorded as authority, the layout would exist only in a PRD, which [`docs/README.md`](../README.md) places at a layer that cannot bind later work.

## Decision

A case's authoritative event stream is a directory of sequence-numbered single-event files.

- Each case owns `reuse-evidence/cases/<case-id>/` in its steward repository, beside the existing root `reuse-evidence.toml` marker.
- Each event is one file named `NNNN-<event-type>.toml`, where `NNNN` is the zero-padded sequence number.
- Events use TOML, matching the marker's existing committed format.
- A case's **revision** is the highest sequence number present.
- An append is an exclusive create of the next sequence file. The filesystem's refusal to create an existing path *is* the expected-revision check; no separate compare-and-swap is required.
- Recorded events are never rewritten or reordered. Correction happens by appending a later event.
- A reader refuses a case whose sequence numbers are duplicated or non-contiguous rather than interpreting a damaged stream.
- Event files are committed evidence. The disposable user-local index under `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §9 remains separate and cannot override them.

This authorizes a file layout and the concurrency primitive it implies. It does not authorize any particular event vocabulary, field set, or schema version; those must be earned by the implementing slice per [`design/v0.1-scope-and-acceptance.md`](../design/v0.1-scope-and-acceptance.md) §2. It does not authorize cross-repository writes, which continue to require the separate authority in `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §5.

## Consequences

### Positive

- An append cannot rewrite history, because it only ever creates a path that did not exist.
- Expected-revision protection comes from an operation the operating system already makes atomic, rather than from application-level locking.
- The write reuses the existing atomic create primitive in `src/lib.rs`; no new append primitive is needed.
- A partial write can damage at most the one event being recorded.
- Ordinary parallel branch work does not produce merge conflicts in evidence; when two branches do record the same event type at the same sequence, Git reports it loudly as an add/add conflict.
- Each recorded event appears in review as exactly one added file.
- Committed evidence uses one format across the marker and the case stream.

### Negative and risks

- A case accumulates one file per event rather than one file per case.
- Reading a case requires listing and sorting a directory instead of reading a single path.
- Two branches recording *different* event types at the same sequence merge cleanly into a duplicated sequence number, so the reader must detect it rather than the merge doing so. The refusal above is the mitigation.
- Sequence ordering lives in filenames, so hand-deleting a file leaves a gap. This is detectable, and consumers are already obliged not to hand-edit authoritative history once a compiled writer exists (`CONSUMER-CONTRACT.md` §6).

### Operational burden

Recording one occurrence is one file creation. Inspecting a case is one directory listing. Neither requires the tool: the layout is readable with `ls` and a text editor, satisfying the no-lock-in obligation in `CONSUMER-CONTRACT.md` §9.

### Compatibility and migration

No case evidence exists yet, so this decision migrates nothing. Event files carry their own schema version; an incompatible future event shape takes a new version under `CONSUMER-CONTRACT.md` §3 rather than a silent reinterpretation. Changing this layout later would require a separately authorized, reversible migration that preserves original provenance and emits an inspectable receipt.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Append-only JSON Lines per case | Rejected | Every append mutates a tracked file in place; parallel branches conflict inside authoritative evidence; requires a new safe-append primitive; diverges from the marker's format. |
| One rewritten document per case | Rejected | Every append rewrites recorded history in place, against `CONSUMER-CONTRACT.md` §2, and widens each write's failure window to the whole case. |
| Match `skill-evidence`'s existing on-disk layout for consistency | Rejected | Shared file shape is not shared responsibility. ADR 0008 scopes that dependency to skill governance, and `FOUNDATIONS.md` prohibits extracting a shared lifecycle kernel before a third independent consumer creates real pressure. |
| Nonce-suffixed event filenames | Rejected | Removes the exclusive-create guarantee, so concurrent writers silently produce duplicate sequences instead of conflicting. |
| User-local database as the authoritative store | Rejected | Already rejected by ADR 0004; opaque, unversioned, and not durable repository authority. |

## Verification and review trigger

The decision is fit if the implementing slice records occurrences, derives readiness, and rebuilds derived state entirely from these files, and if an interrupted append leaves the prior events intact and the case readable.

Reopen if a real case accumulates enough events that per-event files become unreadable or slow in practice, if the duplicate-sequence condition occurs often enough in real branch work to be a burden rather than a rare loud failure, or if an accepted event vocabulary cannot be expressed in TOML without distortion.

## Supersession

None.
