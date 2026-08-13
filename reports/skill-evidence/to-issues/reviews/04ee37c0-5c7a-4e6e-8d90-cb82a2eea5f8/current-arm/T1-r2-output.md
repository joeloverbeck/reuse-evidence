I recommend three tracer-bullet issues:

1. **Prepare and preview an exact privacy-safe evidence handoff**
   - **Blocked by:** Pending the #40 scope decision below
   - **User stories covered:** 1, 2, 4

2. **Refuse stale handoffs and verify accepted scope at delivery**
   - **Blocked by:** Slice 1
   - **User stories covered:** 3, 5

3. **Resume an approved handoff without repeating approval**
   - **Blocked by:** Slice 2
   - **User stories covered:** 6

Publication checkpoint:

- **Target and prerequisites:** Local `./tracker`. Issue #17 and its comments were exact-read. All required labels exist. Issue #40 is open and related: it already covers an exact, privacy-safe case export for an authorized recipient.
- **Decision deltas and assumptions:** The PRD does not say whether the handoff consumes #40’s case export or is a distinct accepted-decision artifact. I recommend treating #40 as a sibling rather than requiring a whole-case export, while reusing an existing privacy/checksum seam if one is available. I also recommend that previewed bytes be exactly the bytes delivered, with redaction applied first, and that approval be bound to the decision revision, payload digest, recipient, and accepted scope so any change invalidates it. These decisions are not yet approved.
- **Category and child state:** Each child would publish as `enhancement + ready-for-agent`. The parent’s `enhancement` category would be preserved.
- **Parent disposition:** #17 is currently `enhancement + needs-triage`. I propose retaining those labels and adding the orthogonal `coordination` label, making it the open, non-grabbable umbrella for the ready children. This exact parent mutation requires approval.
- **Coverage gate:** The proposed slices cover stories 1–6. Nothing is deferred. Implementing whole-case export behavior from #40 and changing the receiving workflow’s internal implementation remain out of scope.

Does the granularity and dependency chain feel right? Please also approve or revise:

1. The recommended boundary with #40.
2. The handoff identity and approval assumptions.
3. Adding `coordination` to parent #17 while retaining `enhancement + needs-triage`.

No issues or parent labels have been changed yet.
