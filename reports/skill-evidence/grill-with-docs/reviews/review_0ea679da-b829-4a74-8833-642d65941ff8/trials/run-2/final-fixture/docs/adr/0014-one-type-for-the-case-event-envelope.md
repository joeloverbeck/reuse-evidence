# ADR 0014: One type for the case event envelope

**Status:** Accepted  
**Date:** 2026-08-11  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

`CONTEXT.md:17` defines the **case event envelope** as a domain term: "the type-independent part of every case event… Every event type records the same envelope; only the body differs." No type carries it.

Four structs restate the same five fields — `src/case.rs:318`, `:333`, `:344`, `:357`. The envelope guard is written four times, at `src/case.rs:2513`, `:2625`, `:2648`, and `:2677`: each combines the same `schema_version`/`sequence`/`event_type` triple, the same UUID-version-4 check on `event_id`, and the same `recorded_at` parse, differing only in the sequence predicate (`!= OPENING_SEQUENCE` for opening, `<= OPENING_SEQUENCE` for the later three) and in its refusal wording. `recorded_at` leaves `RecordedInstant` at `src/case.rs:1732`, `:1755`, `:1780`, `:1824` and is parsed back at `:2532`, `:2644`, `:2669`, `:2696` through one shared `validate_recorded_at`.

The duplication is not across paths. Each guard already serves both the write path (`src/case.rs:2084`, `:2351`, `:2413`, `:2460`, validating the human-supplied prepared event) and the read path (`src/case/read.rs:910`, `:949`, `:973`, `:998`, validating recorded history). It is duplicated across event types, four times, and `design/v0.1-scope-and-acceptance.md` §2 names four more later event types still to record — verification, park, close, and reopen. That same section makes "validate a minimal case event envelope" an in-scope durable-case-mechanic in its own right.

ADR 0011 named this follow-up and withheld it: "A module owning naming invites later pressure to absorb the recorded-event validators next. That would be a separate decision against separate evidence, not a continuation of this one." Its reopen condition is "when the recorded-event validators acquire an independent reason to move." Four event types carrying four copies of one guard, with four more types scoped, is that reason.

The mechanism is already in production bytes. `ReuseDecisionAcceptedEvent` flattens its content at `src/case.rs:363`, so `#[serde(flatten)]` already writes a recorded case event today.

Three facts were measured against the repository's own `toml 0.9.5` and `serde 1.0.219` rather than predicted:

- A flattened envelope serializes **byte-identically** to the five inline fields. `CONSUMER-CONTRACT.md` §3 makes recorded evidence "the hardest compatibility surface," and this is the condition that decides whether the envelope may own the written shape at all. It holds.
- An internally-tagged enum — the only shape that yields one parse per event instead of the two at `src/case/read.rs:865` and `:884`/`:936`/`:960`/`:984` — **moves `event_type` ahead of `schema_version`** in the emitted bytes. It reads today's layout correctly but cannot write it.
- `deny_unknown_fields` survives flattening, but its diagnostic degrades: the span collapses to line 1 and the `expected one of …` field list disappears. That text reaches the terminal through `src/case/read.rs:1002`.

`CONTEXT.md:17` names four parts of the envelope — schema version, sequence, event type, event identity — plus the encoding filename. The code's envelope has five. `recorded_at` is in every event struct and in every guard but not in the term. Giving the envelope a type forces that divergence to be settled rather than carried.

## Decision

The case event envelope has one type.

- `case::event::Envelope` owns `schema_version`, `sequence`, `event_id`, `event_type`, and `recorded_at`, flattened into each of the four event bodies at the position those fields occupy today.
- **Recorded bytes do not change.** Byte-identical serialization of every existing event type is the acceptance condition, re-verified against the repository's pinned `toml` version before the first writer is changed.
- One envelope validator, taking the event type and its sequence predicate as arguments, replaces the repeated guard in the four validators. The per-type refusal wording stays a per-type argument, not a shared string.
- `recorded_at` stays a `String` on the envelope. A single accessor performs the named refusal, so the four `validate_recorded_at` call sites collapse to one owner with no terminal text changed.
- `case_id`'s version-4 check, which currently sits inside opening's envelope guard at `src/case.rs:2533`, moves to opening's body validation, where it belongs.
- The envelope does not absorb any event type's body validation.
- `CONTEXT.md:17` is corrected to name `recorded_at` as the envelope's fifth part, so the term and the type agree.

This authorizes one type and one validator for the envelope's five fields. It does not authorize:

- typing `recorded_at` as `RecordedInstant` in the recorded shape;
- one parse per event by any means that changes the recorded byte order;
- folding `publication.rs`'s retry reader or `read.rs`'s discriminator into the envelope type;
- absorbing proposal documents, content validation, eligibility, or privacy derivation, all of which ADR 0010 withholds and this decision does not disturb;
- any new schema version, migration, or compatibility promise.

## Consequences

### Positive

- Envelope validity has one home, which is what ADR 0011 predicted would need one and deliberately did not grant.
- A fifth event type declares a body and an event-type argument. It does not transcribe five fields, a three-condition guard, a UUID check, and a timestamp parse.
- The five fields stop being four independently editable copies that must be kept aligned by hand across two paths.
- The domain term and the code agree on what the envelope is.

### Negative and risks

- The unknown-field refusal for a malformed recorded event loses its span and its `expected one of …` list. Decision events already carry that degradation today; this extends it to opening, append, and override. No case-event test asserts the current message — the two `unknown field` assertions at `tests/portfolio_cli.rs:931` and `:952` are marker tests — so the drift would be silent unless pinned.
- Two envelope readers remain: `src/case/publication.rs:88` and `src/case/read.rs:98`. Both are deliberately narrower than the envelope, and leaving them out means the report's "one owner" is owner-of-shape-and-validity, not owner-of-every-read.
- `#[serde(flatten)]` makes the written shape depend on a serde and `toml` behaviour that is asserted by measurement rather than by the type system. A dependency bump can break byte-equality without a compile error.

### Operational burden

Adding a later event type means writing its body and passing its event type and refusal wording to the envelope validator. No envelope fields, no envelope guard, and no timestamp parse are written.

### Compatibility and migration

No recorded evidence changes, and no schema version is introduced — `CONSUMER-CONTRACT.md` §3 is engaged only as the constraint that must hold, and it holds by measurement. No refusal wording changes on any validated path, so §1 command behaviour is unchanged, with the single named exception of the serde unknown-field diagnostic above. The decision is fit only while both remain true.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Type `recorded_at` as `RecordedInstant` in the recorded shape | Rejected | It replaces four named refusals with a raw serde error and breaks `tests/case_cli.rs:7127-7128`, which assert `is not UTC RFC 3339` and `is not a valid UTC instant` as terminal text. `RecordedInstant` has no serde derive; the round-trip would move into one, not disappear. Nine of the ten tests in `src/case/instant.rs` cover calendar, range, and refusal semantics that survive either way. |
| One parse per event via an internally-tagged enum | Rejected | Measured: serialization moves `event_type` ahead of `schema_version`, changing recorded bytes and preview text. Deserialization of today's layout succeeds, so a read-only enum is possible, but it reinstates two type families for one event. |
| Remove the second parse by dispatching on the filename | Parked, separate | `EventFileName::parse` already yields the type and `read.rs` already holds the filename, so this needs no serde change and carries no byte risk. It requires keeping the discriminator as a fallback for an unparseable slug, which `tests/case_cli.rs:3653` (`0001-arbitrary.toml`) pins. Independent of this decision. |
| Adopt the shared envelope in `publication.rs`'s retry reader | Parked | It reads two fields on purpose, so a corrupted recorded body still fails as identity-conflict or drift. Reading all five would reclassify that refusal as invalid — a consumer-visible change with no measured pressure behind it. |
| One envelope validator with one shared refusal string | Rejected | Each of the four refusals names its own event and its own preview command. Collapsing the wording is a `CONSUMER-CONTRACT.md` §1 change that buys nothing the parameter does not. |
| Defer until a fifth event type exists | Rejected | ADR 0011's reopen condition has fired, `design/v0.1-scope-and-acceptance.md` §2 names four more types, and waiting means writing a fifth copy before removing four. |
| Record this in the implementing issue instead of an ADR | Rejected | ADR 0011 already declared it "a separate decision against separate evidence." The restraints needing a durable trace are the exclusions of typed `recorded_at` and single-parse: both read as unfinished work, and both are refused on measured evidence that an issue would not preserve. |

## Verification and review trigger

The decision is fit if the next later event type named in [`design/v0.1-scope-and-acceptance.md`](../design/v0.1-scope-and-acceptance.md) §2 is added by writing a body and supplying an event type, with no envelope fields, no envelope guard, and no timestamp parse written, and every existing case event still serializes byte-identically — which the command tests already assert byte-exactly.

Falsify it if byte-equality does not hold for a real recorded event under the repository's pinned `toml` version at implementation time. Re-run that check before the first writer changes; a failure means the envelope may own validation and the read shape only, and the written structs stay as they are. Falsify it also if any event type needs an envelope field the other three do not, which would mean the envelope is less invariant than `CONTEXT.md:17` claims.

Implementation must pin the unknown-field refusal for a case event in a test, so the diagnostic degradation named above is witnessed rather than silent.

Reopen it if an event type needs a different envelope shape or a new schema version, or if a dependency bump breaks byte-equality.

## Supersession

None. This is the separate decision ADR 0011 named at its own reopen condition, and it disturbs no restraint held by ADR 0010, ADR 0011, or ADR 0013.
