# Third-party proposal D-12

The maintainer wants normalization to eliminate case-sensitive mismatches, but has not chosen the compatibility shape. Adjudicate every claim before asking for that choice.

1. `P01` — exactly nine production adapters exist under `src/`.
2. `P02` — every adapter exports `normalizeTag`.
3. `P03` — every adapter trims surrounding whitespace.
4. `P04` — every adapter currently preserves letter case.
5. `P05` — one acceptance test imports all nine adapters.
6. `P06` — that test currently requires case preservation.
7. `P07` — no adapter accepts an options object or compatibility flag.
8. `P08` — no second normalization entry point exists.
9. `P09` — adding an opt-in flag would widen the callable interface at all nine sites.
10. `P10` — immediate trim-then-lowercase needs no dependency.
11. `P11` — commits and publication are out of scope.
12. `P12` — implementation is not authorized until the compatibility fork is ratified.

The remaining fork, if these premises hold, is:

- change all nine adapters immediately to trim then lowercase; or
- add an opt-in lowercase flag and preserve today's default.

The proposal recommends the immediate change because it meets the maintainer's stated mismatch objective without adding a new API dimension. Reject that recommendation if repository evidence shows a compatibility promise, caller, or release obligation that requires the current case-preserving default.
