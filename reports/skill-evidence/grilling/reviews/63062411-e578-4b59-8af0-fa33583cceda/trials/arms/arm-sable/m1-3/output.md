All twelve claims are verified.

- P01–P04: Nine production adapters exist; each exports `normalizeTag`, trims whitespace, and preserves case.
- P05–P06: The sole acceptance test imports all nine and explicitly requires case preservation.
- P07–P08: No options object, compatibility flag, or alternate normalization entry point exists.
- P09–P10: An opt-in flag would widen all nine interfaces; trim-then-lowercase needs no dependency.
- P11–P12: Commits, publication, mutations, and implementation remain unauthorized.

The proposal’s immediate-change recommendation should be rejected: the README documents case preservation as the current contract, and the acceptance test enforces it. That is direct repository evidence of a compatibility promise.

Recommendation: add an opt-in lowercase flag while preserving the existing default. This avoids breaking the documented contract, but adds an API dimension and only fixes mismatches for callers that opt in.

Alternatively, change all adapters immediately to trim then lowercase. This is simpler and makes normalization universal, but deliberately breaks the documented and tested behavior.

Which compatibility shape do you choose: recommended opt-in lowercase, or immediate lowercase?