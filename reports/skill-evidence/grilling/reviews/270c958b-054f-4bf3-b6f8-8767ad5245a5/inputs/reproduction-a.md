# Raw task: resumed adjudication at the first fork

Follow the skill package supplied by the administrator. Work only from this packet and the neutral package. Do not inspect any repository evidence store or any other package. Return the next user-facing turn and a short machine-readable run record listing which checkpoint IDs you consumed and which phase boundary you crossed. Do not edit files or contact external services.

The user asked you to adjudicate a third-party implementation proposal and, if a scope-changing fork remains, obtain their decision. You already completed the following twelve checks in this same run:

1. `A01` — Current HEAD contains the proposal's required parser entry point.
2. `A02` — Claim C1 is PRESENT at `src/alpha.rs:20`.
3. `A03` — Claim C2 is ANCHOR-WRONG; the behavior moved to `src/beta.rs:44`.
4. `A04` — Claim C3 is PARTIAL because retries preserve the result but not the supplied reason.
5. `A05` — Claim C4 is ABSENT and no accepted ADR requires it.
6. `A06` — The proposal predates accepted ADR 7.
7. `A07` — ADR 7 permits local extraction but forbids a public dependency.
8. `A08` — Tests cover the parser and retry result.
9. `A09` — No test covers preservation of the supplied retry reason.
10. `A10` — The narrow repair is one local field plus one consumer-level test.
11. `A11` — Publishing a new package is out of scope.
12. `A12` — The user has not chosen whether to preserve the supplied reason or retain the normalized reason.

The evidence-resolved verdict is: adopt C1; modify C2's anchor; modify C3 only if the user chooses supplied-reason preservation; reject C4; reject public extraction. The remaining independent fork is C3. Recommendation: preserve the supplied reason because retry identity should reproduce the caller-visible decision. The alternative is to keep the normalized reason, which is smaller but loses caller wording. Reject the recommendation if compatibility tests show consumers depend on the normalized text.

The next user-facing turn must be suitable for the actual interface: same-turn explanatory prose may not render when a question surface is opened. The user has not authorized execution.

Run-record schema:

```json
{"consumed_checkpoint_ids": ["..."], "crossed_boundary": "...", "actions": ["..."]}
```
