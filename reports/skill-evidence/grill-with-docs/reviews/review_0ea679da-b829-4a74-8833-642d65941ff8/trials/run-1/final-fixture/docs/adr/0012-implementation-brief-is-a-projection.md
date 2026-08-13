# ADR 0012: The implementation brief is a projection of the accepted decision

**Status:** Accepted  
**Date:** 2026-08-10  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`EVIDENCE-AND-DECISIONS.md`](../principles/EVIDENCE-AND-DECISIONS.md), [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](../principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md), [`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md`](../principles/PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md)

## Context

Nothing in this repository records a reuse decision. `src/case.rs` records three event types and `src/main.rs:58` exposes `open`, `append`, `override`, `list`, and `show`. The next slice records the fourth — the accepted decision — which `FOUNDATIONS.md` names as the project's primary outcome. Fixing its fields is therefore the first point at which the brief's home must be settled, because `CONSUMER-CONTRACT.md` §3 makes recorded evidence the hardest compatibility surface: a field added afterward takes a new schema version.

Two principles reach the brief and point in different directions.

`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §2 gives `reuse-evidence-review` the result "an inspectable proposed reuse decision and, when accepted, a bounded implementation brief," and `EVIDENCE-AND-DECISIONS.md` §10 calls the brief "a durable result." Read together and literally, they invite a second authored document.

`FOUNDATIONS.md` §12 says to record only decision-bearing facts and to "generate routing, status, hashes, and projections mechanically where possible," and its prohibitions include "no hand-authored routing, provenance, status, or certification paperwork where it can be generated." `CONSUMER-CONTRACT.md` §2 requires derived projections to be rebuildable from authoritative state.

The overlap is what makes this decidable rather than a matter of taste. `EVIDENCE-AND-DECISIONS.md` §8 requires the exact accepted decision to name its identity verdict, action, scope, non-responsibilities, affected consumers, compatibility consequences, migration expectations, rollback or re-splitting path, and verification conditions. Of §10's twelve brief contents, eight are already recoverable from those fields together with the case's opening event and recorded occurrences. Exactly four are not: the invariant contract, alternatives rejected, existing packages considered, and required consumer-level tests. Those four are also the reasons the accepted decision is that decision, which is what `EVIDENCE-AND-DECISIONS.md` §1 asks a case to preserve, and §9 already obliges review to examine plausible alternatives before recommending a new public crate.

So an authored brief would restate eight recorded facts in a second artifact with nothing holding them aligned, in exchange for four facts that belong in the case anyway.

The reversal risk is concrete and scheduled rather than hypothetical. `design/v0.1-scope-and-acceptance.md` §2 puts `reuse-evidence-review` in scope; whoever builds it will read §2's result line and write a brief document. The restraint's only other on-disk evidence would be the absence of one.

Privacy sharpens the same point. The brief is the artifact most likely to travel, because it is the handoff out of the reuse lifecycle into ordinary engineering work that may happen in another repository. `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §6 forbids private repository names, paths, source, or symbols reaching public state. A rendered brief inherits the privacy derivation the event receipts already perform (`ReportedPrivacy`, `src/case.rs:217`). An authored document inherits nothing.

## Decision

The accepted reuse decision is the single durable record. The implementation brief is derived from it and is never authored.

1. The decision event records `EVIDENCE-AND-DECISIONS.md` §8's fields plus the four brief contents with no other home: the invariant contract, alternatives rejected, existing packages considered, and required consumer-level tests.
2. The brief is a read-only projection over recorded case state, addressed by case identity so it can be produced in any later session. It writes nothing and commits nothing.
3. No brief is hand-authored, committed as a separate document, or permitted to state anything not recoverable from recorded events.
4. `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §2 is satisfied by `reuse-evidence-review` producing that projection. §2 assigns the responsibility for the result; it does not select the mechanism, and this decision does not amend it.
5. The projection applies the same privacy derivation as the event receipts and opens no new disclosure path.

This authorizes the brief's form and the decision event's coverage of it. It does not authorize:

- any decision event field beyond those named above — the rest of the vocabulary must still be earned by the implementing slice under `design/v0.1-scope-and-acceptance.md` §2;
- verification, park, close, or reopen events;
- writing the brief into any repository, including the steward;
- a machine-readable or compatibility-promised brief format;
- folding the brief into `case show` or `case list`, which answer a different question and print a different shape, as ADR 0010 already refused for receipts.

## Consequences

### Positive

- The brief cannot drift from the decision, because there is only one record.
- The four otherwise-homeless brief contents become recorded case evidence, which is where `EVIDENCE-AND-DECISIONS.md` §1 and §9 already want them.
- The brief inherits private dominance mechanically rather than by an author remembering it.
- The handoff survives interruption. `design/v0.1-scope-and-acceptance.md` §4 requires the slice to resume from durable state, and a brief addressed by case identity satisfies that where a one-time output would not.
- No new durable artifact class appears, so `FOUNDATIONS.md` §12 is honored rather than argued around.

### Negative and risks

- The decision event becomes this project's largest event, and four of its fields are earned from principle text rather than from a completed real review. If a real decision leaves them empty, they were modeled too early.
- A projection renders only what was recorded. A reviewer who needs to say something the fields do not hold must change the schema or say it outside the case.
- One accepted decision per case keeps the projection unambiguous today. When reopen exists, the projection must name which accepted decision it renders. That is a later decision and is deliberately not prebuilt.
- A reader of `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §2 alone will still expect an authored document. Recording this is the mitigation.

### Operational burden

Accepting a decision is one preview, one approval, and one publication — the shape `append` and `override` already use. Producing the brief is one read-only invocation against a case identity, with nothing committed and nothing to maintain afterward.

### Compatibility and migration

No decision event has ever been recorded, so nothing migrates. The decision event carries its own schema version; fixing these fields now is what avoids a later version bump under `CONSUMER-CONTRACT.md` §3 on its hardest surface. The brief's rendered text is command behavior under `CONSUMER-CONTRACT.md` §1, changeable during `0.x` under §8, and is not promised as a stable format.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Hand-author the brief as the review skill's document | Rejected | Restates eight recorded facts in a second artifact with nothing keeping them aligned, and inherits no privacy enforcement. `FOUNDATIONS.md` §12 forbids hand-authored paperwork that can be generated. |
| Record only §8's fields and settle the brief later | Rejected | The accepted decision would record nothing that leaves the lifecycle, leaving `CLAUDE.md`'s "delegate implementation outside the reuse lifecycle" unmet for another cycle, and adding the four fields afterward costs a schema version under `CONSUMER-CONTRACT.md` §3. |
| Print the brief only as part of the acceptance receipt | Rejected | The handoff normally happens in a later session, and `design/v0.1-scope-and-acceptance.md` §4 requires resumption from durable state. A one-time output cannot be reproduced without re-recording the decision. |
| Fold the brief into `case show` | Rejected | The same error ADR 0010 refused for receipts. `case show` answers what the case records; the brief answers what an implementer should do. |
| Commit the rendered brief beside the events for provenance | Rejected | A generated projection stored beside its source is the derived-state duplication `FOUNDATIONS.md` §12 and `CONSUMER-CONTRACT.md` §2 refuse. |
| Record this in the implementing PRD or issue | Rejected | [`docs/README.md`](../README.md) ranks PRDs below ADRs and an issue closes, while the restraint's only other on-disk evidence is an absence. ADR 0010 and ADR 0011 rejected the same alternative for the same reason. |

## Verification and review trigger

The decision is fit if the first real accepted decision fills every recorded field with something a later session needs, and the projected brief is sufficient to hand to ordinary engineering without a supplementary document being written anyway.

Falsify it if a real review cannot express its accepted decision within the recorded fields without distortion, or if the first handoff requires a companion document authored beside the projection. Either result means the brief is a genuinely authored artifact, and this decision should be reversed rather than extended with more fields.

Reopen it when reopen and verification events exist and the projection must choose among several accepted decisions, or if a real consumer needs the brief as a stable machine-readable format rather than as command output.

## Supersession

None. This decision does not amend `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §2; it records that §2 assigns the brief's responsibility to review while leaving the mechanism to this layer.
