Single-threaded verification supports all twelve premises:

- Adopt P01–P05 and P07–P10: nine adapters export the same trim-only function; one test imports all nine; there are no flags, alternate entry points, or dependencies.
- Adopt P06: the acceptance test explicitly requires `"MiXeD"` to remain `"MiXeD"`.
- Adopt P11–P12: commits, publication, and implementation are not authorized.
- Important qualification: the README explicitly defines case preservation as the current contract. That satisfies the proposal’s stated condition for rejecting an immediate breaking change.

Recommendation: add an opt-in lowercase flag while preserving the current default. This fixes case-sensitive mismatches for opting-in callers without silently breaking the documented contract. Its cost is a new API dimension across all nine adapters. I would reject this option if you intend to authorize a deliberate breaking contract change and update the test and documentation accordingly.

Decision: should the eventual implementation:

1. **Preserve the default and add an opt-in lowercase flag (recommended)**, or
2. **Change all nine adapters immediately to trim then lowercase**, accepting a breaking contract change?

No files were changed. The existing test passes, and the repository diff check reports no errors.