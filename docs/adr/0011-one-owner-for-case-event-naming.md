# ADR 0011: One owner for case event naming

**Status:** Accepted  
**Date:** 2026-08-10  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

Every case event carries the same envelope regardless of type: a schema version, a sequence, an event type, an event identity, and a file name of the form `NNNN-<slug>.toml` that encodes the first two of those. ADR 0009 fixed that shape. Nothing owns it.

Each of the three recorded event types is spelled three ways, and only one spelling is checked by the compiler:

| Spelling | `case_opened` | `occurrence_appended` | `early_review_authorized` |
|---|---|---|---|
| `EventType` variant | `src/case.rs:181` | `src/case.rs:181` | `src/case.rs:181` |
| body string | literal, `src/case/read.rs:640` | literal, `src/case/read.rs:690` | literal, `src/case/read.rs:722` |
| file-name slug | literals, `src/case.rs:376`, `src/case.rs:1236`, `src/case/read.rs:640` | literals, `src/case.rs:465`, `src/case/read.rs:690`, `src/case/read.rs:855` | literals, `src/case.rs:693`, `src/case/read.rs:722`, `src/case/read.rs:855` |

Nine string literals stand between the enum that names the event types and the files that record them. `validate_file_event_type` (`src/case/read.rs:752`) takes both the body spelling and the slug spelling as literal arguments at each of its three call sites, so the correspondence it enforces is supplied by the caller rather than known.

The `NNNN-<slug>.toml` grammar itself is restated at five sites (`src/case.rs:376`, `465`, `693`; `src/case/read.rs:766`, `860`). The staged-temporary-file convention is implemented twice with different structure: `is_opening_temporary` (`src/case.rs:1231`) strips a hardcoded prefix, while `is_later_event_temporary` (`src/case/read.rs:837`) parses the name and match-arms the slugs. `OPENING_SEQUENCE` is declared twice (`src/case.rs:25`, `src/case/publication.rs:10`), and `MAX_CASE_SEQUENCE` lives in `src/case/publication.rs:11` even though its own refusal message derives it from "the accepted `NNNN` event layout" — the grammar, not the protocol.

`design/v0.1-scope-and-acceptance.md` §2 names four more later event types still to record: accepted decision, verification, park/close, and reopen. Its first durable-case-mechanics bullet is "validate a minimal case event envelope." Adding the fourth type means editing every site above, with the compiler catching one.

That is the observed pressure. The naming rule already has three occurrences today; this decision does not rest on the four scoped ones.

An architectural review of the case modules recommended something wider: re-cutting `src/case.rs` and `src/case/read.rs` so that a vocabulary module sits below a publication module and a projection module, with one-way edges, replacing a split made by direction of travel. Two of its premises no longer hold.

Its sequencing — do this as part of collapsing later-event publication, "not on its own, where it is mostly relocation" — expired when that collapse landed without it (`8cbed62`, `src/case/publication.rs`).

Its target shape is not reachable. It draws publication and projection as siblings, but publication depends on projection: `case::append` and `case::authorize_early_review` call `read::read_case_for_append` and `read::read_case_for_early_review` and hold `read::CaseRecord` and `read::Readiness` (`src/case.rs:456`, `520`, `555`, `684`, `745`). That edge is mandated, not incidental — ADR 0010 requires that creating an event file be reachable only from a value that holds the lock and has re-read the case. The achievable arrangement is a chain, not a fork.

Its substrate is also not one thing. Of the sixteen items `src/case/read.rs` reaches upward for, eleven are event vocabulary and recorded-event validation, three are repository location and steward reading (`find_repository_root`, `read_steward`, `validate_case_storage_path`), and two are receipt strings. Moving the envelope leaves five crossing.

`FOUNDATIONS.md` §9 says the narrowest valid reuse scope wins, and §4 says to share only the invariant behavior whose authority can be owned coherently. Naming is such behavior: ADR 0009 states it once, every event type obeys it, and it cannot vary by type without changing recorded evidence. The rest of the coupling between the two modules is not.

## Decision

One internal module owns case event naming.

- It owns the event type and its spellings: the Rust variant, the recorded body string, and the file-name slug, derived from one declaration rather than restated.
- It owns the `NNNN-<slug>.toml` grammar in both directions — constructing an event file name and parsing one — and the sequence bounds that grammar implies, including the maximum the four-digit layout permits.
- It owns the staged-temporary-file name convention, as one predicate rather than one per event class.
- Both the publication path and the projection path reach it one way. It reaches neither.

This decision does not re-cut the boundary between `src/case.rs` and `src/case/read.rs`. Repository-root discovery, steward reading, storage-path validation, recorded-event body validation, readiness derivation, and receipt strings stay where they are, and the two modules continue to cross in both directions.

It authorizes nothing ADR 0010 withholds. Proposal documents, content validation, eligibility rules, privacy derivation, and receipt fields remain owned by each event type.

It changes no recorded evidence. The same file names are written and read, byte for byte, so `CONSUMER-CONTRACT.md` §3 is not engaged.

## Consequences

### Positive

- A later event type is named once. Its body spelling and its file-name slug cannot disagree, because neither is written by hand.
- The envelope becomes unit-testable. Name construction and parsing currently have no test that does not first build a repository on disk.
- The two staged-temporary-file predicates become one, so the opening event and later events cannot diverge on what a partial write looks like.
- `MAX_CASE_SEQUENCE` sits with the grammar that determines it rather than with the protocol that happens to enforce it.

### Negative and risks

- `src/case.rs` and `src/case/read.rs` still reach into each other. This decision deliberately leaves that, and it will read as a half-finished version of the review's recommendation.
- A module owning naming invites later pressure to absorb the recorded-event validators next. That would be a separate decision against separate evidence, not a continuation of this one.
- The event type gains behavior beyond serialization, so a future type whose file name does not follow `NNNN-<slug>.toml` would have to break the owner rather than write a literal beside it. That is the intended cost.

### Operational burden

Adding a later event type means declaring it once in the owning module and implementing its proposal, eligibility, privacy, and receipt as ADR 0010 requires. No file name, slug, or temporary-name rule is written outside the owner.

### Compatibility and migration

None. Event files, slugs, sequence semantics, schema versions, and receipts are unchanged. No migration and no consumer-visible difference.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Re-cut the case modules into vocabulary, publication, and projection with one-way edges | Rejected | The shape is unreachable as drawn — ADR 0010 mandates the publication-to-projection edge — and the shared substrate is three concerns, not one. Beyond naming it is relocation, which the recommending review itself said. |
| Leave naming as literals until a fourth event type exists | Rejected | The rule already has three occurrences, and the fourth type would be written against nine uncoupled literals first. That is ADR 0010's divergence one layer down, and cheaper to prevent than to detect. |
| Own naming in an `impl` block inside `src/case.rs` | Rejected | Projection would still reach up into the writer's file for it, so the edge stays two-way and the grammar keeps no home reachable one way from both sides. |
| Put the grammar in `src/case/publication.rs` | Rejected | ADR 0010 scopes that module to ADR 0009's protocol. Projection never publishes but must parse the same names, so it would have to depend on the publisher to read. |
| Record this in the implementing issue instead of an ADR | Rejected | The decision that needs a durable trace is the restraint — that the wider re-cut was examined against current source and declined. An issue closes, and the review recommending the re-cut does not. |

## Verification and review trigger

The decision is fit if the next later event type named in `design/v0.1-scope-and-acceptance.md` §2 is added by declaring its name once, and no slug, file-name, or temporary-name literal is written outside the owning module.

Falsify it if the grammar cannot be expressed without a per-event-type parameter beyond the type itself, or if projection needs a name spelling that publication does not produce. Either would mean naming is less invariant than ADR 0009 states, and the literals are the cheaper cost.

Reopen the wider re-cut when a change is blocked by the coupling between `src/case.rs` and `src/case/read.rs` itself rather than by naming, or when the recorded-event validators acquire an independent reason to move.

## Supersession

None.
