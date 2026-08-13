# Adjudication

Use this when the object under grill is a third-party document, finding or handoff set,
multiple sources, or a repository/environment sweep.

## Workflow

1. Verify each material claim against the authoritative environment. For many independent
   claims, use the verifier shape in [Verification](verification.md#verifier-shape).
2. Give each item an evidence-scaled disposition: commonly adopt, reject, or modify, or a
   routing owner for a finding/handoff.
3. Collapse clear dispositions into a batch and isolate only forks that change scope,
   ownership, content, or reversibility.
4. Present the verdict body and recommended collapsed disposition in a visible turn.
5. If the user requested only a ruling, stop. Otherwise ask the unresolved collapsed fork in
   the next turn, recap the result after the answer, and execute only when authorized.

The verdict presentation is the branch list in a many-item adjudication; do not add a second
list. Do not ask about evidence-resolved items merely to make the run feel interactive.

## Evidence Boundary

Before assigning warrant, record what each source is allowed to prove:

- **Direct authority**: the user's decision, preference, approval, or an artifact the
  governing process makes authoritative.
- **Proxy or simulated input**: a substituted answer, scenario, model, or harness result. It
  may support its labeled limited claim, not user preference, direct observation, or approval.
- **Process observation**: evidence about workflow friction, omissions, sequence, or
  resumability. It can justify process changes without validating the product answer produced.
- **Unwitnessed**: a claim or phase the evidence did not exercise. It limits only verdicts that
  depend on that gap.

Apply coverage per claim rather than accepting or rejecting a source wholesale. Preserve any
narrower label or human-owed debt defined by the governing process; do not rename, upgrade, or
discharge it. Re-adjudicate a source's own recommendations instead of inheriting them.

## Dispositions And Sources

Record the evidence and coverage behind every disposition. Verifiers supply facts, locations,
and coverage; the adjudicating thread alone decides what to adopt.

When sources address the same claim:

- convergence raises confidence;
- divergence is a fork to surface;
- orthogonal sources retain separate warrant rather than being forced into agreement.

Blanket approval can ratify only a recommended disposition already visible in the verdict
body. Follow [Questions And Ledger](questions.md#rendering-and-recovery) for the later question
turn and [Recap](recap.md) before execution.
