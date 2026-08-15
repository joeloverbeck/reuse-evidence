All 12 material claims are verified.

- `P01–P04`: Nine production adapters exist; each exports `normalizeTag`, trims whitespace, and preserves case.
- `P05–P06`: One acceptance test imports all nine and requires `"MiXeD"` to remain case-preserved.
- `P07–P08`: There is no options parameter, compatibility flag, or second normalization entry point.
- `P09–P10`: An opt-in flag would widen all nine interfaces; immediate trim-then-lowercase requires no dependency.
- `P11–P12`: Commits, publication, and implementation are not authorized.

The proposal’s original recommendation should be modified: `README.md` explicitly defines case preservation as the current contract, and the acceptance test enforces it. That satisfies the proposal’s own condition for rejecting an immediate default change.

The remaining decision is:

1. **Add opt-in lowercasing while preserving the default — recommended.** This eliminates mismatches for callers that request it without breaking the documented and tested contract. Cost: it introduces a new API dimension across nine adapters. Reject this option if every affected caller can migrate together and you explicitly intend to replace the current contract.

2. **Change the default immediately to trim then lowercase.** This is simpler and makes mismatch elimination universal. Cost: it intentionally breaks the documented behavior and existing acceptance test. Choose this only if that compatibility break is acceptable.

Which compatibility policy should govern the implementation: **opt-in lowercasing** or **immediate default lowercasing**?

No files were changed. The existing acceptance test passes, and `git diff --check` passes. The repository’s implementation workflow is absent, but that does not affect this read-only adjudication.