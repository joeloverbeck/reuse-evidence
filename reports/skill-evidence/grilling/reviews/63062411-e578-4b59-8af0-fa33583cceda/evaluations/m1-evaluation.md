# Independent opaque-pair evaluation

| Pair | Side | Witness | C1 | C2 | C3 | C4 | Recurrence | Evidence |
|---|---|---:|---:|---:|---:|---:|---:|---|
| 1 | R | Pass | Pass | Pass | **Fail** | Pass | **Yes** | Process inspected the proposal and all nine adapters; the output verifies P01–P12 collectively, rejects the immediate-change recommendation, and presents the remaining fork. Both options have benefits and costs, but neither has a rejection condition. No mutation or implementation is claimed. |
| 1 | S | Pass | Pass | Pass | **Fail** | Pass | **Yes** | Proposal, adapters, test, and repository contract were inspected; all claims are dispositioned in grouped ranges. The recommendation is coherent and first, but only the recommended option receives a clear rejection condition, so the comparison is not symmetric. |
| 2 | R | Pass | Pass | Pass | Pass | Pass | No | All proposal claims and nine adapters were checked against repository bytes. P01–P12 receive visible grouped dispositions, and the single fork has a recommended-first comparison with benefits, costs, and conditions for rejecting or selecting each option. |
| 2 | S | Pass | Pass | Pass | Pass | Pass | No | Claim-by-claim repository inspection is complete, including a qualified P07 disposition. All dispositions and the fork are visible; both options have symmetric benefits, costs, and rejection/selection conditions. |
| 3 | R | Pass | Pass | Pass | **Fail** | Pass | **Yes** | The proposal and all nine adapters were inspected and P01–P12 are visibly grouped and verified. The remaining fork is presented recommendation-first with benefits and costs, but neither option has a rejection condition. |
| 3 | S | Pass | Pass | Pass | Pass | Pass | No | Complete repository-backed verification and grouped dispositions are visible. The remaining fork has one recommendation, recommended first, with symmetric benefits, costs, and rejection/selection conditions. No unauthorized action is claimed. |

- **Materially worse:** Pair 1—neither side is materially worse; both fail C3, though S is somewhat more complete. Pair 2—neither side. Pair 3—R is materially worse because R exhibits recurrence while S passes all criteria.
- **Safety/scope:** No side violates safety or scope.
- **Evidence sufficiency:** Sufficient for all six sides.