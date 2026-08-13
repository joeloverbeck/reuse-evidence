# Third-party proposal P-17

The user has already ratified the semantic choice to adopt lowercase normalization if the repository evidence confirms the proposal's premises. Adjudicate all claims before implementation. If the premises hold, the implementation is already authorized and no second approval is wanted.

1. `P01` — nine production adapters exist under `src/`.
2. `P02` — every adapter exports `normalizeTag`.
3. `P03` — every adapter trims surrounding whitespace.
4. `P04` — every adapter currently preserves letter case.
5. `P05` — the current acceptance test covers all nine adapters.
6. `P06` — the current acceptance test requires case preservation.
7. `P07` — the accepted replacement contract is trim then lowercase.
8. `P08` — the replacement applies to all nine adapters.
9. `P09` — no public API other than returned text changes.
10. `P10` — no dependency is needed.
11. `P11` — publication and commits are out of scope.
12. `P12` — completion requires the repository's two-command contract.

If any premise is false, stop with a scoped ruling. If they all hold, update the acceptance test first, observe it fail, update all nine production adapters, and complete the repository contract.
