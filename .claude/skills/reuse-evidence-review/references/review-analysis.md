# Review analysis

Load this reference only after the compiled queries establish one healthy review-ready case and its
privacy boundary. The questions below are the analysis structure, not optional prompts. Answer each
from recorded occurrences and inspected evidence; use `not supported by the recorded evidence`
when that is the honest result.

## Evidence discipline

Build one compact evidence map before judging the case:

- each recorded repository-and-consumer pair;
- its stated independence basis;
- every recoverable commit and optional repository-relative path;
- the authority, lifecycle, release, compatibility, privacy, and reason-to-change facts the artifact
  actually supports;
- any shared authored doctrine that could make several repositories one coordinated context; and
- the uncertainty or contradiction the evidence leaves.

Evidence authorizes only the claim it bears. Code shape supports similarity; it does not establish
one responsibility. A shared authored mandate is evidence against independent discovery unless a
consumer artifact establishes pressure beyond conformance. Do not repair a thin case by turning
review into an unbounded source investigation.

## The eleven questions

1. **One responsibility?** Do the occurrences actually share one coherent authority, policy,
   representation, contract, or independently changing decision? Include the shared-doctrine and
   independence result. This answer supports `identity_verdict`.
2. **Invariant?** What exact behavior or consumer-facing contract remains the same across the
   evidence-bearing consumers? Separate a behavioral invariant from similar files or algorithms.
3. **Legitimate variation?** Which differences follow from distinct consumers, languages,
   environments, policies, or contracts, and which are accidental divergence? This answer constrains
   both scope and non-responsibilities.
4. **Same reasons to change?** Do the consumers change in response to the same authority, lifecycle,
   invalidation, retry, release, or compatibility decisions? Name any divergence that would make a
   shared owner unstable.
5. **Owner and lifecycle?** Who can coherently own the invariant, its source of truth, releases,
   compatibility, and maintenance? `nobody yet` is a valid evidence result.
6. **Cost of sharing?** What coupling, dependency direction, release coordination, migration,
   privacy, trust, or compatibility cost would each plausible shared scope create?
7. **Existing owner?** Does an existing package, crate, standard, schema, or upstream project already
   own the responsibility adequately? When this question makes Rust package research decision-bound,
   take the package-research branch before answering it.
8. **Code, generation, or contract?** Would generated artifacts, a centralized schema or fixture
   corpus, or a versioned contract create more leverage than one runtime implementation?
9. **Narrowest valid scope?** Which scope creates real leverage while preserving coherent authority,
   privacy, compatibility, reversibility, and independent change? State why every narrower plausible
   choice is insufficient and why every broader one buys no supported benefit.
10. **Wrong abstraction?** Is an existing shared surface coupling consumers that no longer share one
    responsibility or reason to change? If so, assess splitting, inlining, or narrowing it rather
    than extending it.
11. **Falsifier?** What recoverable future evidence would show the proposed identity, action, scope,
    owner, or boundary is wrong? Do not state a condition that could only confirm the proposal.

## Full scope ladder

Compare every rung that the evidence leaves plausible:

1. local helper;
2. module;
3. workspace package;
4. private cross-repository package;
5. public package;
6. generated artifacts;
7. centralized schema, specification, fixture corpus, or other versioned contract;
8. existing dependency;
9. upstream contribution;
10. intentional duplication;
11. deferral for more evidence; and
12. de-abstraction by splitting, inlining, or narrowing an existing shared surface.

For each plausible rung, compare leverage, owner, dependency direction, privacy, release and
compatibility burden, migration cost, rollback or re-splitting path, and required consumer-level
proof. The chosen action follows the narrowest defensible rung; public extraction is not a default.

## Outcome checks

- `existing_abstraction_is_wrong` with `split_inline_or_narrow_existing_abstraction` is a first-class
  result when question 10 supports it.
- `different_responsibilities` with `retain_intentional_duplication` is success, not a failed review.
- `insufficient_evidence` with `wait_for_more_evidence` is success when the case records a useful
  verification question rather than pretending certainty.
- A same-responsibility verdict does not force extraction; an existing dependency, generator,
  contract, local deepening, private scope, or continued duplication may remain narrower.
- The decision's verification conditions state what would count as evidence of the accepted
  consequence. They do not claim the consequence has already been implemented.

The analysis is complete only when every decision field can be traced back to these answers and the
evidence map, with every uncertainty kept visible.
