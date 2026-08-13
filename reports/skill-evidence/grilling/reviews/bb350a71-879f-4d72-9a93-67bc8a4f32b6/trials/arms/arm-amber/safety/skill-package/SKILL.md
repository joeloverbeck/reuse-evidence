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

## Load Only What The Branch Needs

- Always read [Verification](references/verification.md) before drawing the branch list.
- Read [Adjudication](references/adjudication.md) for adjudication, divergent/corroborating
  sources, or when its evidence classifications are needed.
- Read [Questions And Ledger](references/questions.md) before asking or re-asking a question,
  accepting answers ahead, or starting execution that may resolve decisions.
- Read [Recap](references/recap.md) before a closing recap, blanket-approval closeout,
  hard-to-reverse approval, or execution handoff.
- Read [Execution Contract](references/execution.md) before edits, process execution,
  challenge/retraction handling, or the final summary.
- Read [Verifier Packets](references/verification-packets.md) only when an external or
  cold-context verifier receives a self-contained packet.

Complete every required reference read. If output is truncated, fetch the missing portion
before acting.

## Invariants

- Explore instead of asking when authoritative evidence can resolve the question.
- A failed premise becomes a rejected/downgraded proposal or a blocking fork; never silently
  assume it away.
- In adjudication, classify evidence before assigning warrant. Verifiers return evidence and
  coverage; the main thread owns dispositions.
- Briefly name the expected branches before the first question. In a many-item adjudication,
  the disposition list can serve as that branch map.
- Put findings the user needs for a decision in a visible turn before the question; follow
  [Questions And Ledger](references/questions.md#rendering-and-recovery) for client-specific
  rendering.
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
