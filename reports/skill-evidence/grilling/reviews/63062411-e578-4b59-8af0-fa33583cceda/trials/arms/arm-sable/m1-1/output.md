All 12 material claims are verified against the repository.

Disposition: reject the proposal’s immediate-change recommendation. The README explicitly defines case preservation as the current contract, and the acceptance test enforces it. No callers or release obligations were found, but the documented compatibility promise is sufficient to favor preserving the default.

Next decision — recommendation first:

1. **Add an opt-in lowercase flag (recommended).** Preserves the documented default while enabling trim-then-lowercase. Cost: widens all nine adapters’ interfaces.
2. **Immediately lowercase by default.** Keeps the API simple and directly eliminates case mismatches. Cost: intentionally breaks the documented and tested case-preserving contract.

Should implementation preserve the current default with an opt-in flag, or make lowercase immediate?

No files were changed. The existing test and `git diff --check` pass.