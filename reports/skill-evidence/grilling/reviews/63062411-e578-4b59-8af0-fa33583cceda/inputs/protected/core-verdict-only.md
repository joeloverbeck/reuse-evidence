# Raw task: verdict-only adjudication

Follow the neutral skill package. The user asks only for a ruling on three supplied claims; do not execute or ask for approval.

- `V1`: The proposal says the command is write-free. The authoritative contract says it appends an event. Disposition must be reject.
- `V2`: The proposal says retries are idempotent. The authoritative contract and test both say exact retries return the original receipt. Disposition must be adopt.
- `V3`: The proposal cites line 90, but the same rule is now at line 112 with unchanged meaning. Disposition must be modify-anchor.

Return the verdict only. Do not inspect any evidence store or mutate anything.
