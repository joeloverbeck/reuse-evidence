I found one related open PRD that needs an explicit boundary decision before publication, so no child issues have been created.

1. **Preview a privacy-safe accepted-decision handoff**
   - **Blocked by:** None
   - **User stories covered:** 1, 2, 4

2. **Deliver and verify the current accepted scope**
   - **Blocked by:** Slice 1
   - **User stories covered:** 3, 5

3. **Resume an approved handoff without duplicate approval**
   - **Blocked by:** Slice 2
   - **User stories covered:** 6

Publication checkpoint:

- **Target and prerequisites:** Publish under #17 in this repository’s tracker. #17 has no cited prerequisites or comments. No open blocker is proposed.
- **Decision deltas and assumptions — approval pending:**
  - #40 already covers exact, privacy-safe case export, event bytes, and recipient checksums. I recommend keeping that work out of #17: this breakdown would hand off the accepted decision and scope, without duplicating the complete case-export format or depending on #40.
  - The PRD does not prescribe a transport or artifact schema. I recommend specifying deterministic preview, delivery, and receiver verification behavior without inventing a hosted service or premature compatibility promise.
  - “Resume without repeating approval” should reuse approval only for the unchanged decision revision. A changed revision must trigger the stale-revision refusal.
- **Child disposition:** Each child would use category `enhancement` and state `ready-for-agent`; both are canonical labels.
- **Parent disposition:** #17 is currently `enhancement` + `needs-triage`. Before creating children, I propose adding the orthogonal `coordination` label while retaining those existing labels and leaving #17 open. This exact label mutation requires your approval.
- **Coverage gate:** The slices cover all six PRD stories. No #17 story is deferred. #40’s full case export, exact event bytes, and recipient-checksum behavior remain separate and out of scope. The current inventory has no exact-title collisions; I’ll rerun the guards immediately before any creation.

Does this granularity feel right, are the dependencies correct, and should any slices be merged or split? Please also approve or revise the three decision resolutions and the exact parent-label transition.
