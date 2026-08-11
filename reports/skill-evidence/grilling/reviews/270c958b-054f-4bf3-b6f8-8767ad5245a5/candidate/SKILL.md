---
name: grilling
description: Interview the user about a plan or design, adjudicate a third-party document against the environment, or drive a user-directed documented process, then honor the ratified outcome. Use to stress-test a plan, rule on someone else's proposal, execute an already-authorized process, or on any "grill" trigger phrase.
---

# Grilling

Resolve every decision that changes the outcome and none that evidence already resolves:

1. Verify premises against the authoritative environment.
2. Name the foreseeable decision branches.
3. Ask unresolved forks one at a time, recommendation first.
4. Record decisions in a running ledger.
5. Recap the ratified scope.
6. Execute when authorized, capturing new forks as they arise.

## Choose The Mode

Choose by the request's object and authority, not by how many artifacts it mentions:

| Mode | Use when | Behavior |
|---|---|---|
| Interview | The user wants to sharpen their plan or design. | Explore first, then ask each outcome-changing fork. |
| Adjudication | The object is a third-party report, spec, finding set, handoff, or a repository/environment sweep. | Verify claims, disposition each item, and collapse only scope-changing forks. If the user asked only for a verdict, stop after the ruling. |
| Documented process | The user asks to follow a process or a companion workflow routes an explicit, already-authorized reversible request here. | Verify the stated scope, name foreseeable branches, and execute it. Do not manufacture a design interview or a second approval; re-question only a mismatch that changes what ships. |

A sweep that is only an execution step stays in the governing execution mode. Multiple
authoritative inputs do not by themselves turn a direct execution into adjudication.

## Gate Each Phase Boundary

Complete each gate before crossing its boundary. Never backfill; recheck the table on a
mid-run or companion handoff.

| Before | Read | Required checkpoint |
|---|---|---|
| Branch list | [Verification](references/verification.md) | Premise states and branch map. |
| Question, re-question, or answers ahead | [Questions And Ledger](references/questions.md) | One fork; recommendation first; evidence visible. |
| First edit, process action, challenge, or retraction | [Questions And Ledger](references/questions.md), [Execution Contract](references/execution.md) | Ledger ready before multi-file or multi-phase mutation. |
| Recap, blanket closeout, hard-to-reverse approval, or handoff | [Recap](references/recap.md), [Execution Contract](references/execution.md) | Scope and ledger reconciled; action and veto ready. |
| Final summary | [Execution Contract](references/execution.md) | Deliverable reconciled; completion result ready. |

Read [Adjudication](references/adjudication.md) when that mode applies and [Verifier Packets](references/verification-packets.md)
only for self-contained verifier packets. Finish truncated reads before acting.

## Invariants

- Explore instead of asking when authoritative evidence can resolve the question.
- A failed premise becomes a rejected/downgraded proposal or a blocking fork; never silently
  assume it away.
- In adjudication, classify evidence before assigning warrant. Verifiers return evidence and
  coverage; the main thread owns dispositions.
- Capture every mid-execution fork before the next mutation, then reconcile those captures and
  the delivered artifact against the ratified scope.
- If the project declares a completion or landing contract, read it while planning and run it
  again before delivery.

## Final Delivery

Before responding:

1. Sweep the decision ledger, including mid-execution forks.
2. State the ratified in-scope and out-of-scope boundaries.
3. Confirm the delivered artifact matches them; scope every completeness claim.
4. Report unresolved, unavailable, or pending evidence.
5. Run the applicable completion contract, or state why none applies. For non-product or
   companion-domain work, use the conditional fields in
   [Execution Contract](references/execution.md#artifact-specific-closeout).
