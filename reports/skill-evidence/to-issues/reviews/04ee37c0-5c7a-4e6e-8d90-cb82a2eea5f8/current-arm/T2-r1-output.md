Prepared and parked. No tracker state was changed because #25’s maintainer comment says #24 is an open blocker and forbids child publication until it closes.

1. **Record verification outcomes for a bounded case set**
   - **Blocked by:** #24
   - **User stories covered:** 1, 2, 3, 4, 8

2. **Close only fully verified cases**
   - **Blocked by:** #24 and slice 1
   - **User stories covered:** 6, 8

3. **Park a verification with a named reason**
   - **Blocked by:** #24 and slice 1
   - **User stories covered:** 5, 8

4. **Resume a partially verified batch**
   - **Blocked by:** #24 and slices 1–3
   - **User stories covered:** 7, 8

Publication checkpoint:

- **Target and prerequisites:** the tracker containing #25. The explicit open blocker is #24; its record is unavailable for direct reading, so the maintainer’s comment on #25 is controlling. No satisfied or deferred prerequisites were identified.
- **Decision deltas and assumptions:** I recommend fixing the batch’s case membership when it is selected so resumption cannot expand its scope. I also recommend defining a complete case as one whose verification conditions all have attached support and a recorded met result; a not-met result keeps the case open for parking or later resolution. Per-condition progress and the fixed membership should survive resumption. These choices are pending approval. No interface or storage schema is proposed.
- **Category and child-state disposition:** every child would use category `enhancement` and state `ready-for-agent` after the blocker clears. The parent’s category remains `enhancement`.
- **Parent disposition:** while parked, #25 remains open with `enhancement` and `needs-triage`. On later publication, I recommend retaining those labels and adding the orthogonal `coordination` label; that exact parent mutation requires explicit approval. The parent will not be closed.
- **Coverage gate:** the slices cover stories 1–8. Nothing is deferred or out of scope.

Does this granularity feel right, are the dependencies correct, and do you approve the stated assumptions plus the future addition of `coordination` to #25?
