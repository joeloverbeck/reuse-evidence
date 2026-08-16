# Package research and disclosure

Load this reference only when one active review is considering a decision that requires Rust package
research. Research is decision-bound; it is not a routine review step. A call is one HTTP request,
and every request independently passes the complete gate below before it is made.

## No-call branches

Make no network call when:

- the provisional decision proposes no new crate dependency and needs no package alternative;
- the human declines a displayed request;
- adequate findings are supplied directly for local inspection; or
- no available retrieval instrument can expose and enforce the request boundary below.

Supplied findings discharge the retrieval only after review inspects them for functional fit,
authority and abstraction boundary, compatibility and maintenance burden, license, release
stability, transitive cost, and whether a narrow upstream contribution is better. Passing supplied
findings through unexamined proves nothing.

With neither approved retrieval nor adequate supplied findings, record an unmet evidence need. Do
not recommend a new public Rust crate. Another action remains available only when the recorded
evidence supports it independently of the missing research.

## Per-request allow test

A request is allowed only when all five conditions hold immediately before sending it:

1. **Decision-bound.** It answers existing-package research for the action under active
   consideration on this one review-ready case.
2. **Previewed in full.** Display every decoded query term and the target URL's scheme, host, and
   path. Adding, removing, broadening, or substituting a term requires a new preview. Percent
   encoding or parameter ordering does not create a new term by itself.
3. **Approved for this review.** The human approves that exact displayed request set. Approval is
   neither standing nor inherited from another case, request, or earlier review.
4. **Retrieval-only payload.** Send an unauthenticated public-information read whose only payload is
   the approved request text. Send no event, case document, diff, source, test, embedding,
   repository or case identity, credential, account data, or additional case text.
5. **Permitted and approved host.** The host belongs to the closed class below and the human approved
   that concrete host under condition 3.

Use only a retrieval instrument that can send the displayed request without credentials and with
automatic redirects disabled. A redirect target is a new URL and a new HTTP request. Do not follow
an undisplayed redirect; display it and repeat the full allow test first.

## Closed host class

Only these public hosts qualify:

- a package registry or its API;
- a published package's documentation host; or
- the public source repository a candidate package declares.

A general web search engine, package aggregator such as lib.rs, mirror, unlisted CDN, remote model,
embedding service, hosted inference service, or any other host is disallowed. The class does not
substitute for approval of the concrete host.

## Public and private query terms

For a public case, query terms may be drawn directly from public case material, but the human still
approves them and every target.

For a private case, generalize terms so the private reuse consumer cannot be named or reconstructed.
Terms contain no private repository name, path, module path, symbol, internal crate or package name,
or distinctive verbatim phrasing from the case responsibility. Describe the responsibility being
searched for, not the consumer that has it. Display the generalized terms; the human's approval is
approval of both the request and the sufficiency of that generalization. Never generalize after
approval or fall back to identifying terms when a generalized query performs poorly.

## Assess and record every finding

Assess each plausible package against all seven criteria:

1. functional fit;
2. coherent authority and abstraction boundary;
3. compatibility and maintenance burden;
4. license;
5. release stability;
6. transitive cost; and
7. whether a narrow upstream contribution is better.

Research is recorded on the accepted decision event, never as an evidence reference and never in a
separate research report or cache:

- For an implementation-authorizing action, add one `existing_packages_considered` entry per
  examined package with `package`, `fit`, and `reason`.
- For `retain_intentional_duplication` or `wait_for_more_evidence`, omit
  `existing_packages_considered` and add one `alternatives_rejected` entry per examined package,
  naming why it did not fit.

If research needs an additional request, return to the per-request allow test. If the human declines
at any point, make no declined request and continue only with the findings already obtained or
supplied.
