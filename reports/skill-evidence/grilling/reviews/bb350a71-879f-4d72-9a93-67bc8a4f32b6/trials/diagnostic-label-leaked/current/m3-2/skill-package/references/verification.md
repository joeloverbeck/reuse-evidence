# Verification

Read this before deciding that a branch needs a question or that no question is needed.

## Premises And Freshness

Verify material premises against the authoritative environment: repository files, documents,
issues, commits, reports, and the actual instruments the deliverable will use. Explore instead
of asking when that evidence resolves a branch.

For pointer, successor, identity, or authority migration, compare the replacement's declared
status, maturity, approval/gate state, and authority scope with the role it will inherit.
Existence or matching bytes alone cannot promote a provisional artifact into an active or
definitive role.

Treat a third-party document's factual claims as premises. Check locations, quoted or
paraphrased anchors, evidence scope, and freshness. When a claim is pinned to a commit, compare
that commit with current HEAD. For a versioned artifact with a changelog, read the recent
version entries that may supersede the claim.

Record one of these states per material claim:

- verified or contradicted;
- unavailable because the source cannot be reached; or
- pending because a relevant check is still running.

Do not adopt a proposal on an unavailable premise; restructure, downgrade, or defer it with a
named re-verification trigger. A pending check may leave a fork open only when every possible
result yields the same recommendation; otherwise wait.

## Verifier Shape

For many independent claims, group verification by the artifact being checked so each file or
tight cluster is read once. When delegation is available, give verifiers a bounded return
contract: per claim, a coverage tag such as PRESENT, ABSENT, PARTIAL, or ANCHOR-WRONG, a short
quote, and an exact location. Verifiers return evidence, not adopt/reject decisions.

When the adjudicating thread will edit the same small set of sites, use direct, tightly scoped
reads instead of delegating only to re-fetch the text. In a mixed set, fan out the broad
screen, then direct-read surviving edit sites.

If delegation is unavailable, run the same artifact-grouped checks sequentially and say that
verification was single-threaded. External or harness-generated votes are corroboration, not
final dispositions. Research commissioned during adjudication follows the same evidence-only
contract.

If a cold or external verifier receives a self-contained packet, additionally read
[Verifier Packets](verification-packets.md) before collection.

## Edit And Search Context

Edit only from fresh, exact context that includes the complete site and its relevant
surroundings. A grep hit alone is not edit context. A bulk rewrite that may touch a planned
site invalidates earlier context; sequence the targeted edit first or refresh the read.

Keep reconnaissance searches token-anchored and output-bounded. Treat truncated or elided
output as incomplete and fetch the missing region before drawing a conclusion.

## Self-Computed Evidence

A number or fact produced by a probe, simulation, sampled query, or estimate is a premise, not
authority. Validate it against the real artifact, instrument, and path the deliverable will
use. Reconcile any computed value that enters the final deliverable again before delivery.

Inspect an instrument's write behavior before running it. When it writes, or when write
behavior is unknown, use explicit scratch output paths or a temporary directory and verify the
tracked tree stayed unchanged. An accidental tracked-tree mutation is an unratified change to
disclose and restore, not a free read.

Delivery-time reconciliation also covers claims about co-edited artifacts. Land the co-edit
before claiming the pointer, single home, or cross-reference is current, then verify it.

## Satisfied Conditional Actions

When the user has authorized an action conditionally and exploration proves the condition with
no remaining scope-changing fork, record the evidence-resolved decision and perform exactly
that action without another confirmation. If the condition itself selects the whole
deliverable, such as research versus build, expose the recommendation and let the user ratify
it before execution.
