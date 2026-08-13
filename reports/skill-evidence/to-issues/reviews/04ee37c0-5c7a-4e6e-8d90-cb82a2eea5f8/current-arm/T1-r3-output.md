I propose three tracer-bullet issues:

1. **Prepare and preview a revision-bound, privacy-safe handoff**
   - **Blocked by:** #40, if we adopt the recommended reuse boundary below
   - **User stories covered:** 1, 2, 3, 4

2. **Deliver the handoff with receiving-side scope verification**
   - **Blocked by:** Slice 1
   - **User stories covered:** 5; completes the end-to-end path established by stories 1–4

3. **Resume an interrupted handoff without repeating approval**
   - **Blocked by:** Slice 2
   - **User stories covered:** 6

Publication checkpoint:

- **Target and prerequisites:** Publish under PRD #17 in this tracker. The PRD cites no prerequisites. However, open issue #40 already owns exact, privacy-safe case export and recipient checksum verification. I recommend treating #40 as an external blocker for Slice 1 and keeping its export/redaction mechanics out of #17.
- **Decision deltas and assumptions:** The boundary with #40 is not stated in the PRD and needs approval. I recommend that #17 own accepted-decision binding, exact preview, stale-revision refusal, delivery to the implementation workflow, accepted-scope verification, and resumable approval custody. No new transport, command, or file schema should be implied by these issues. Resumption should retain the exact approved-preview identity and revalidate the decision revision before delivery.
- **Child disposition:** All three children would be `enhancement` + `ready-for-agent`.
- **Parent disposition:** #17 is currently `enhancement` + `needs-triage`. Before creating children, I propose retaining `enhancement`, removing `needs-triage`, and adding `coordination`. The parent would remain open and non-grabbable.
- **Coverage:** The proposed slices cover stories 1–6. Nothing is deferred or out of scope beyond the export/redaction/checksum mechanics already assigned to #40.

I haven’t created or changed any issues yet. Does this granularity and dependency structure look right, and do you explicitly approve both the #40 boundary and the exact parent-label transition?
