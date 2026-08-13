# ADR 0019: One event records verification and its disposition

**Status:** Accepted  
**Date:** 2026-08-12  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`EVIDENCE-AND-DECISIONS.md`](../principles/EVIDENCE-AND-DECISIONS.md), [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](../principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

**The lifecycle has no ending.** `CaseState` (`src/case/read.rs:27`) has four variants, and an accepted decision dominates every other route to `awaiting-verification` (`src/case/read.rs:124`). Nothing supersedes it, so every case that reaches a decision stays there permanently. `EVIDENCE-AND-DECISIONS.md` §8 names three lifecycle values that currently have no representation at all: verified and closed, parked, and reopened.

`design/v0.1-scope-and-acceptance.md` §2 lists "record verification, park, close, and reopen outcomes" as the last unbuilt durable case mechanic. Its §4 slice cannot pass steps 9 through 11 without it, and §8's publication threshold asks for "the smallest coherent lifecycle," which a lifecycle with no terminal state is not.

**The number of remaining event types was counted twice, differently, and never decided.** ADR 0011's context names "accepted decision, verification, park/close, and reopen." ADR 0014's names "verification, park, close, and reopen." Each was sizing its own decision against a number rather than fixing one, and the two numbers disagree. The count is still open, and it has to be chosen before the first byte is recorded: `CONSUMER-CONTRACT.md` §3 makes recorded evidence the hardest compatibility surface, so splitting one recorded event type into four afterwards costs a case event schema version.

**The decision event already answered the same shape question.** `EVIDENCE-AND-DECISIONS.md` §8 requires "orthogonal fields in the durable decision rather than one ever-growing status enumeration," and `reuse_decision_accepted` implements exactly that: `identity_verdict` states what review found, `action` states what it authorized. Verification has the same two parts — what the evidence shows, and what the human does about it.

**The accepted decision already names what verification must answer.** `verification_conditions` and `affected_consumers` are required on every decision whatever its action, including the two that authorize no implementation (`src/case.rs:2001`, `:2042`). Verification therefore does not invent its own question set; it answers the one the decision recorded. That is what lets the compiled command check completeness without semantic judgment, which is the boundary `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §3 draws.

**ADR 0013's carrier already has the two optional fields this needs, and no more.** `LaterEventOutcome` (`src/case.rs:454`) declares `state: Option<read::CaseState>` and `notice: Option<&'static str>`. A disposition is one of three fixed values, so it renders through the `&'static str` notice without widening the carrier. ADR 0013 stated its own fitness test as the next later event type being added "by supplying a heading and its optional fields, with no new outcome struct, effect enum, or branch shape written." One event type meets that test once. Four would pay for it four times.

## Decision

One later case event type, `verification_recorded`, records both the verification evidence and its disposition.

- **Orthogonal fields.** Per-condition and per-consumer results state what the evidence shows. A separate `disposition` field — `closed`, `parked`, or `reopened` — states what the human decided about it.
- **The decision supplies the question set.** The proposal must answer every `verification_conditions` entry and every `affected_consumers` entry of the accepted decision, exactly once each. Each result is `met`, `not_met`, or `accepted_exception`; an accepted exception must state its reason. Results carry evidence references in the shape occurrences already use.
- **Closing is mechanically gated, not judged.** `closed` requires every result to be `met` or `accepted_exception`; any `not_met` result admits only `parked` or `reopened`. This states `EVIDENCE-AND-DECISIONS.md` §11 mechanically. Whether a condition was actually met, and whether an exception is acceptable, remain the human's.
- **Derived state.** `closed`, `parked`, and `reopened` become `CaseState` variants that supersede `awaiting-verification`, as an accepted decision supersedes readiness. None carries a readiness basis and none authorizes review.
- **The reopen loop re-verifies.** A parked or reopened case may record a further `verification_recorded` event against the standing accepted decision. `closed` is terminal in version 0.1.
- **No disposition event types.** `case_closed`, `case_parked`, and `case_reopened` are not created.

This authorizes one additive event type, its proposal document, its eligibility rule, its privacy derivation, and the three derived states. It does **not** authorize:

- a second accepted decision on a case — the `has_decision` guard (`src/case.rs:1278`) stands, so ADR 0012's projection still chooses among exactly one decision and its reopen condition does not fire;
- executing repository-defined verification commands or capturing their output; verification records evidence references like every other event;
- any event on a closed case, including reopening it under later consumer pressure;
- collapsing verification's proposal reader, eligibility rule, or privacy derivation into another event type's, which ADR 0010 still withholds;
- any change to existing receipt text, refusal text, refusal ordering, exit statuses, or recorded event bytes.

## Consequences

### Positive

- The case lifecycle gains an ending. `EVIDENCE-AND-DECISIONS.md` §8's three terminal values become derivable from the event stream, and `design/v0.1-scope-and-acceptance.md` §4's slice can complete.
- One proposal document, one eligibility rule, one privacy derivation, and one revision per closure, instead of four of each and two expected-revision round trips.
- Verification's completeness is checkable against the decision's own recorded fields, so the compiled command enforces coverage while deciding nothing semantic.
- ADR 0013's carrier gets its stated test using the two optional fields it already declares.

### Negative and risks

- Evidence and disposition are one recorded act. A maintainer who has gathered verification evidence but is not ready to dispose of the case records nothing, and that evidence lives outside the case until they are.
- Placing `disposition` inside `verification_recorded` is a recorded-evidence commitment. Separating them later costs a case event schema version under `CONSUMER-CONTRACT.md` §3.
- A reopened case can be re-verified but not re-decided, so a verification showing that the decision itself was wrong has no recorded correction path in version 0.1.
- Three terminal variants join a `CaseState` whose four current variants all describe an active case, and none of the three carries a readiness basis.

### Operational burden

- Recording verification is one prepared proposal, one preview, one human approval, and one exclusive create — the same shape as an append, an override, or a decision.
- The proposal's size tracks the decision's: one entry per recorded verification condition and per affected consumer, both of which the maintainer already wrote when accepting the decision.

### Compatibility and migration

- No recorded event changes and no existing receipt text changes. `verification_recorded` is additive at case event schema version 1.
- `case list`, `case show`, and `case brief` gain the three terminal states. `case brief` continues to project the single accepted decision, so ADR 0012 stands undisturbed.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Separate `verification_recorded` plus `case_closed` / `case_parked` / `case_reopened` events | Rejected | Four proposal readers, four eligibility rules, and two revisions per closure, to split two parts of one human act. `EVIDENCE-AND-DECISIONS.md` §8 asks for orthogonal fields, and the decision event already answers the same question that way. |
| Fold the disposition into a mutable status on the case | Rejected | §8 rejects "one ever-growing status enumeration" explicitly, and FOUNDATIONS 11 forbids derived state from being an independent source of truth. |
| Let verification execute the repository's verification commands | Rejected | `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §3's list of compiled mechanics does not include running code; `CONSUMER-CONTRACT.md` §6 places verifying consumer behaviour on the adopting repository; and capturing command output into case evidence is a private-dominance leak path. |
| Allow a new accepted decision after a reopen | Parked | It fires ADR 0012's own reopen condition and grows this work by the guard change, a latest-decision projection, and multi-decision reads. The common failure is an implementation that did not land, which re-verification against the standing decision already covers. Reopen when a real verification shows the accepted decision itself was wrong. |
| Support only `closed` and `parked` | Rejected | `EVIDENCE-AND-DECISIONS.md` §11 states that a verification failure "reopens or parks the case." Dropping reopen would require amending a principle. |
| Record this in the implementing issue instead of an ADR | Rejected | The restraint needing a durable trace is the absence of three disposition event types that ADRs 0011 and 0014 both named as expected. An issue closes; the absence would read as an oversight. ADR 0013 rejected the same alternative for the same reason. |

## Verification and review trigger

The decision is fit if `verification_recorded` is added by supplying a heading and ADR 0013's two optional fields — with no new outcome struct, effect enum, or branch shape written, and every existing receipt still printing byte-identically — and if one case runs open, append, decide, verify with disposition `reopened`, and verify with disposition `closed`, deriving every state from the event stream alone.

Falsify it if the disposition needs an outcome field the carrier does not have, if `closed`'s completeness rule cannot be checked without judging whether a condition was met, or if verification evidence is routinely gathered in sessions that cannot yet dispose of the case — which would show the two parts are not one act after all.

Reopen it when a verification shows the accepted decision itself was wrong, when a closed case must absorb later consumer pressure, or if a second supported case event schema version ever makes the one-type commitment cheap to revisit.

## Supersession

None.
