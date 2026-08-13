I prepared and parked this breakdown; no issues were created and #25 was not changed.

1. **Start and resume a bounded verification batch**
   - Blocked by: none within the breakdown
   - User stories: 1, 2, 7, 8

2. **Attach evidence and record verification results for one case**
   - Blocked by: slice 1
   - User stories: 3, 4, 7, 8

3. **Park a selected case with a named reason**
   - Blocked by: slice 1
   - User stories: 5, 7, 8

4. **Close completed cases while retaining incomplete cases**
   - Blocked by: slice 2
   - User stories: 6, 7, 8

Publication checkpoint:

- **Target and prerequisites:** The tracker containing #25. Its maintainer comment identifies #24 as an open blocker and explicitly forbids child publication until #24 closes. The complete inventory does not contain #24, and an exact read of #24 failed, so its closure cannot be verified. Recommendation: continue treating it as open and keep this breakdown parked.
- **Decision delta:** The source comment and live tracker inventory disagree about whether #24 is inspectable. This remains unresolved; approval of the breakdown would not authorize bypassing it.
- **Children:** Each proposed child would use category `enhancement` and state `ready-for-agent`. Those labels exist.
- **Parent:** #25 currently has `enhancement` and `needs-triage`. It remains unchanged while parked. Eventual publication should preserve both and add the orthogonal `coordination` label; that exact parent mutation requires explicit approval.
- **Coverage:** The slices collectively cover stories 1–8. Nothing is deferred or out of scope.
- **Collisions:** The current complete inventory has no exact-title collisions for these proposed titles; a fresh check is still required immediately before publication.

Does this granularity feel right, are the dependencies correct, and should any slices be merged or split? Publication also requires #24 to be verifiably closed and explicit approval to add `coordination` to #25.
