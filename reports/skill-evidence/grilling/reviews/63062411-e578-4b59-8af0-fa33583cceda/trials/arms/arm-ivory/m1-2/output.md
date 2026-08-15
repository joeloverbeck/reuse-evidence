All twelve premises are supported by repository evidence:

- P01–P04: nine production adapters exist; each exports `normalizeTag`, trims with `value.trim()`, and preserves case.
- P05–P06: the acceptance test imports all nine and expects `"MiXeD"` after trimming.
- P07–P09: there are no options parameters, compatibility flags, or alternate normalization entry points. An opt-in flag would therefore widen all nine callable interfaces.
- P10: trim-then-lowercase can use built-in string methods and needs no dependency.
- P11–P12: repository rules prohibit commits/publication, and your request explicitly withholds all mutation authorization.

Disposition: adopt the factual premises, but modify the proposal’s recommendation. The README expressly defines case preservation as part of the current contract. That is the compatibility promise the proposal identified as grounds to reject an immediate default-breaking change. No callers or release obligations were found, but their absence does not erase the documented contract.

Recommendation: add an opt-in lowercase flag while preserving the current default. This meets the mismatch objective for opting-in callers without silently breaking the documented behavior. Its cost is a broader API across nine adapters. Reject this option if you intentionally want to replace the documented contract now and accept the breaking default change.

The alternative is to change all adapters immediately to trim then lowercase. It keeps the API simple and makes mismatch elimination universal, but breaks the documented contract and current acceptance test. Choose it only if that break is intentional.

Which compatibility shape should govern the eventual implementation: **preserve the default and add an opt-in lowercase flag (recommended)**, or **make lowercase the new default immediately**?

No files were changed. The existing acceptance test passes, and `git diff --check` passes.