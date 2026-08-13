Prepared and parked; no issues or parent state were changed.

1. **Complete a bounded verification batch**
   - Blocked by: #24
   - Covers stories: 1, 2, 3, 4 (met), 6, 8

2. **Record an unmet verification without closing the case**
   - Blocked by: #24 and slice 1
   - Covers stories: 3, 4 (not met), 6, 8

3. **Park and resume a partially verified batch**
   - Blocked by: #24 and slice 1
   - Covers stories: 5, 7, 8

Publication checkpoint:

- **Target:** This repository’s issue tracker.
- **Prerequisites:** #24 is an open blocker. No children may be published until it closes.
- **Decision deltas and assumptions:** None identified; implementation interfaces and storage choices remain unspecified.
- **Children:** Each would use category `enhancement` and state `ready-for-agent`.
- **Parent:** Currently `enhancement`, `needs-triage`. After publication, I recommend `enhancement`, `ready-for-agent`, plus the orthogonal `coordination` label. That exact transition requires approval.
- **Coverage:** The slices collectively cover stories 1–8; none are deferred.
- **Title guard:** No exact-title collisions currently exist.

Does the granularity and dependency structure look right? Please also explicitly approve or revise the proposed parent-label transition. Even with approval, publication remains parked until #24 is closed.
