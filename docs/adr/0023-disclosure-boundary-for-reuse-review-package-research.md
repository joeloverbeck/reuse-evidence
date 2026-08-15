# ADR 0023: The disclosure boundary for reuse-review package research

**Status:** Accepted  
**Date:** 2026-08-15  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md`](../principles/PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md), [`EVIDENCE-AND-DECISIONS.md`](../principles/EVIDENCE-AND-DECISIONS.md), [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](../principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

The live decision is #41: author `reuse-evidence-review`, the capability that produces what `FOUNDATIONS.md` calls the primary outcome, "the accepted reuse decision and its verified consequence." `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §2 gives review ownership of "semantic identity, scope, alternatives, package research, non-responsibilities, migration expectations, privacy consequences, and verification conditions." The package cannot be authored honestly until it is settled whether that research may leave the machine, because a skill that performs the research and a skill that refuses to are different packages, not the same package with a flag.

### Two accepted statements pointing in opposite directions

`EVIDENCE-AND-DECISIONS.md` §9 makes the research mandatory: "Before recommending a new public Rust crate, review must search crates.io and inspect plausible alternatives" for functional fit, authority and abstraction boundary, compatibility and maintenance burden, license, release stability, transitive cost, and whether a narrow upstream contribution would be better. Not *may* — *must*, and it names the service.

`FOUNDATIONS.md` §10 refuses to let that be read as permission: network use "requires a distinct accepted capability and a precise disclosure boundary; it is not implied by package search or public distribution." It names package search explicitly, which makes this a scoping question rather than an inference a lower layer may draw for itself.

`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §10 is the topic principle that specializes the `FOUNDATIONS.md` clause, and it already supplies the shape of the answer: "Public package metadata or documentation may be queried during a decision-bound dependency review, but source disclosure and remote model use are separate concerns," followed by five conditions on sending anything derived from private material — a named live decision requiring it, an explicit disclosure preview, accepted authority, a bounded payload, and a recorded result.

§10 grants a conditional permission. It does not constitute the *distinct accepted capability* `FOUNDATIONS.md` §10 requires, it does not say who applies the five conditions or when, and it does not say what a private case's query may contain. This ADR is that capability, and it binds §10's five conditions to this one use.

### The private case is the majority path, not an edge case

Three of the five enrolled repositories on the maintainer's machine are private (#41). `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §6 makes one private participant enough to make the complete case private, so most cases that reach review will be private-dominant. A query composed from a private responsibility sends private evidence to a public service. §6 forbids writing "private repository names, paths, source bodies, symbols, commits, specifications, or reports into public state"; §10's own list of what may not be sent without authority includes "case details," and a search string derived verbatim from a private case's responsibility text is exactly that. `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §6 says separately that no sensor may "expose private material remotely without separate authority." `CLAUDE.md` lists the leak among the boundaries that must not be eroded.

### Nothing in the compiled binary is at stake

The crate's direct dependencies are `clap`, `serde`, `skill-evidence`, `toml`, and `uuid` (`Cargo.toml`:24–:29). No HTTP, TLS, or network-client crate appears anywhere in the 77 packages `Cargo.lock` resolves. Every command that documents its reach states the property directly: enrollment, the portfolio report, staging resolution, the cross-portfolio case query, and the installer each "performs no network access" ([`README.md`](../../README.md):34, :112, :124, :352, :390).

So the question is entirely about a skill package. `.claude/skills/` holds one project-owned package, `reuse-evidence-capture`; `design/v0.1-scope-and-acceptance.md` §2 names three more, and review is among the unauthored three.

### Where findings can actually be recorded

The direction ratified while #41 was written said findings are recorded as an *evidence reference* on the reuse decision. The implemented schema cannot carry that, and the correction belongs here rather than in an issue that closes:

- `DecisionContent` (`src/case.rs`:101–:120) has no evidence field and carries `#[serde(deny_unknown_fields)]` (`:100`), so a decision proposal offering one is refused. The decision is in fact the *only* one of the five event bodies without evidence references: occurrence evidence carries them into the case-opened and occurrence-appended events (`:240`), the early-review override carries its own (`:93`–`:97`, `:512`), and verification carries them per condition and per consumer (`:268`, `:280`). That near-universality is the misleading precedent.
- `EvidenceKind` has exactly one variant, `Commit` (`:357`–`:359`). `CONTEXT.md`'s glossary defines an evidence reference as pointing to "a commit, diff, specification, test, source location, report, package, or other inspectable artifact," so the domain concept is broader than the implemented enum. As implemented, a recorded evidence reference cannot name a registry, a document, or a query anywhere the field appears.

The decision event already has the fields this research is for, and which field applies is decided by the action:

- `existing_packages_considered` is a list of `{package, fit, reason}` (`:472`–`:476`), required with non-empty content for every implementation-authorizing action (`:2361`–`:2396`) — which includes `use_existing_dependency` and `publish_public_package`, the two actions §9's mandate fires on — and projected into the implementation brief under ADR 0012 (`src/case/render.rs`:205).
- `validate_no_change_decision_content` (`:2445`) *refuses* that field on `retain_intentional_duplication` and `wait_for_more_evidence`, so a decision authorizing no implementation cannot use it.
- `alternatives_rejected` is a list of `{alternative, reason}` required non-empty on **every** decision, because `validate_decision_content` runs `validate_common_decision_content` before it branches on the action (`:2298`–`:2299`, `:2334`–`:2345`).

So research always has a durable home, and the home is determined rather than optional.

## Decision

**Reuse review may make a network call for package research, under the boundary below. No other capability may, and nothing compiled may.**

### 1. The capability belongs to the review skill and to nothing else

It is `reuse-evidence-review`'s, exercised while reviewing one review-ready case. `reuse-evidence-capture`, `reuse-evidence-discover`, and `reuse-evidence-status` gain nothing. The compiled command gains nothing. `CONSUMER-CONTRACT.md` §2's local-first guarantee is untouched: the core lifecycle still requires no hosted account, telemetry, remote model, or external API, and every command remains network-free.

### 2. What makes a call allowed

**A call is one HTTP request.** Not one retrieval, not one research question: each request is tested on its own, so a redirect, a follow-up page, and a second search are three calls and face the test three times. Fixing this is what makes the conditions applicable at all, since a rule about "a retrieval" cannot say whether a redirect it did not anticipate is inside or outside the thing approved.

A call is allowed only when **all five** conditions below hold at the moment it is made. Failing any one makes it disallowed; there is no residual discretion. Recording is a separate obligation under item 5, because a condition that can only be met after the call cannot be part of a test applied before it.

1. **Decision-bound.** It serves `EVIDENCE-AND-DECISIONS.md` §9 research for a decision under active consideration on one review-ready case. §9 already says package search "is decision-bound research, not a mandatory capture step"; a review that proposes no new crate dependency performs no research and makes no call.
2. **Previewed in full.** Every query term and every target URL was displayed to the human, in full, before any call. A term or URL that was not displayed cannot be sent. If research needs one review did not display, it returns for a second approval rather than widening the first.
   - **Identity is the term, not its wire encoding.** What must match what was displayed is the decoded query term and the URL's scheme, host, and path. Percent-encoding, parameter order, and other transport-level rewriting leave an approved term the same term; adding, dropping, broadening, or substituting a term makes it a different one. Stating this is what keeps the condition applicable: a literal byte-identity rule would disallow nearly every real request, and a purely semantic one would license unbounded rewriting.
   - **A redirect is not an exception.** A redirect target is a URL in its own right, so an undisplayed one is not followed: review stops and returns it for approval like any other undisplayed URL. This holds whether or not the redirect stays on an approved host, because this condition gates URLs while condition 5 gates hosts, and both must pass independently. Automatic redirect-following must be disabled rather than relied upon.
3. **Approved.** The human approved that exact displayed set, for this review. Approval is never standing, never inherited from another case, and never inferred from a prior review of the same case.
4. **Retrieval only, and nothing travels but the approved text.** The call is an unauthenticated read of public information. The transmitted payload is the approved request text and nothing else: no event file, no case document, no diff, no source or test from an enrolled repository, no embedding, no repository or case identifier, no credential, no account. The approved query terms are themselves the payload and are exempt from that list by construction — what is forbidden is shipping a case *artifact*, or any case text beyond the terms the human approved. Which terms may be approved in the first place is decided by item 3 below, not by this condition.
5. **Targeted at a permitted host.** Two gates apply together, and both must pass:
   - **The class is closed.** The host is a public package registry or its API, a published package's documentation host, or the public source repository a candidate package declares. Nothing else qualifies — a general web search engine, an aggregator such as lib.rs, a mirror, an unlisted CDN, or any other host is outside the class until a further accepted decision adds it. `EVIDENCE-AND-DECISIONS.md` §4.7 is what admits the third element: it makes "external package documentation **and source** when alternatives are reviewed" admissible evidence, which reaches further than `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §10's "metadata or documentation" alone.
   - **The instance is human-approved.** The concrete host is one the human saw under condition 2. Registry and documentation hosts are stable — crates.io and docs.rs today — but the third element is an arbitrary host chosen by a third party's crate metadata, so the class alone does not close it. Displaying the specific declared URL and having it approved is what closes the set, per review, for the hosts that review actually reaches.

### 3. What a private case's terms may contain, and who approves them

The case's derived privacy is read from a command, not asserted by the agent. The cross-portfolio case query reports it per case under ADR 0020 item 1, and every event receipt carries it (`EventReceipt`, `src/case/render.rs`:91). `case show` is **not** that source: its renderer emits `privacy_conflicted` and `stale` only (`:310`–`:370`), which are the case's current *conditions*, not its derived privacy. Review reports the privacy it read before analysis begins (#41 story 5).

**On a public case**, the approved terms may be drawn directly from the case. Nothing in the portfolio's privacy rules restricts disclosing public material, and generalizing it would only make the research worse.

**On a private case**, the approved query terms must be generalized so that a reader of the query cannot name or reconstruct the private reuse consumer. Concretely, the terms carry no private repository name, path, module path, symbol, internal crate or package name, and no phrasing lifted verbatim from the case's responsibility text where that phrasing is distinctive enough to identify its source. The generalization describes the *responsibility being searched for*, not the consumer that has it.

**The human approves the generalization, not merely the search.** The set displayed under the allow test's condition 2 is the generalized set, and approving it is approving that the generalization is sufficient. Review may not generalize after approval, nor send an ungeneralized term because a generalized one returned little.

### 4. Declining is a supported outcome, and it costs something

If the human declines, **no call is made.** Review then takes one of two paths:

- it proceeds from findings the human supplies directly, which are ordinary admissible evidence under `EVIDENCE-AND-DECISIONS.md` §4.7; or
- it records the research as an **unmet evidence need** and returns it, which is one of the four results `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §9 permits a capability to return.

The first path needs its reading of §9 stated rather than assumed. §9 requires that the alternatives *have been searched for and inspected* against its seven criteria before a new public Rust crate is recommended; it does not require that review be the process which issues the request. So supplied findings discharge §9 only when review actually inspects them against those seven criteria and records what it found — review still owes the judgement, and inherits only the retrieval. Findings handed over and passed through unexamined discharge nothing.

A decline is therefore not merely a skipped step. With neither a search nor inspected supplied findings, review **may not recommend a new public Rust crate**. It records the unmet need and either proposes a different action or returns without a proposal. That is the local-first guarantee holding by default, and the decision surface narrowing is its price, not a defect to work around.

### 5. How findings are recorded

Findings are recorded on the `reuse_decision_accepted` event, in the field the action makes available. **Research is never left unrecorded**, whatever the decision turns out to be:

- A decision that **authorizes implementation** records them in `existing_packages_considered` — one entry per package examined, carrying the package, its functional fit and abstraction boundary, and the reason it was adopted or set aside. The field is required non-empty for those actions and is projected into the implementation brief under ADR 0012.
- A decision that **authorizes no implementation** — `retain_intentional_duplication` or `wait_for_more_evidence` — is refused if it carries `existing_packages_considered` at all, so it records the examined packages in `alternatives_rejected` instead: one entry per package, naming the package as the rejected alternative and why it did not fit. That field is required non-empty on every decision, so this home always exists.

Either way the event file is the durable record: authoritative, inspectable under `CONSUMER-CONTRACT.md` §9, and readable by a later session that must not repeat the disclosure. That is what makes the disclosure a cost paid once per decision.

This **corrects** the direction ratified in #41 and #42 that findings become an *evidence reference* on the decision, for the schema reasons the Context gives. Both fields above already exist and are already required, so no schema change is bought.

### 6. This does not authorize

- any network access for the compiled binary, any existing command, or any command added later; that would require its own accepted decision;
- the capability for `reuse-evidence-capture`, `reuse-evidence-discover`, or `reuse-evidence-status`;
- any remote model, external model API, embedding service, or hosted inference of any kind, which `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §10 keeps a separate concern from package metadata, and which `FOUNDATIONS.md` prohibits as a *mandatory* dependency of the core lifecycle;
- uploading source, diffs, case text, event files, prompts containing case evidence, or embeddings to any service, on a public case as much as a private one;
- telemetry, usage reporting, analytics, or any call not answering a displayed, approved query;
- standing, remembered, or session-wide approval; a cached research index, user-local findings store, or shared package-research database, which `FOUNDATIONS.md` §12 refuses as a second domain of control records;
- ecosystem research outside Rust. #42 puts it explicitly out of scope and `EVIDENCE-AND-DECISIONS.md` §9 makes it decision-bound rather than universal, so a TypeScript or other-ecosystem search is not authorized here at all and needs its own accepted decision;
- adding a dependency, editing a manifest, or performing any implementation, which ADR 0006 and `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §7 leave to ordinary engineering;
- making package research a capture step, a discovery step, or a precondition of opening or appending a case;
- a hosted service, daemon, MCP server, or proxy through which such calls are made;
- authoring the review package, which is a separate slice under #41.

## Consequences

### Positive

- `EVIDENCE-AND-DECISIONS.md` §9's mandate becomes performable without eroding `FOUNDATIONS.md` §10, and the review package can be authored against a settled boundary rather than an assumed one.
- The boundary is a conjunction of conditions checkable *before* one HTTP request, over a closed class of permitted hosts, so a reader classifies a call rather than weighing it. `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §10's own five conditions separately acquire a named owner and a moment of application; they are not the same five, since §10's "recorded result" lands in item 5 rather than in the allow test.
- The private-dominant case — the majority path — gets an explicit rule instead of inheriting the public one by silence.
- The human sees the exact terms and the exact target before anything leaves the machine, which is `FOUNDATIONS.md` §1's semantic authority applied to disclosure rather than only to acceptance.
- Findings survive in the one durable decision record and reach the implementation brief, so the disclosure is paid once per decision rather than once per session.
- The local-first guarantee is preserved as a *default*, not merely as a fallback: the declining path is fully specified, and no call is the ordinary outcome of a review that proposes no new crate.

### Negative and risks

- This is the project's first accepted network capability. It is narrow, but the precedent now exists, and a later proposal will cite it. Item 6 enumerates more exclusions than the allow test has conditions for that reason.
- Generalization is a human judgement made under time pressure at the moment of approval. Nothing tests it, nothing can, and a term that leaks less than a repository name may still be distinctive enough to identify a private consumer to a determined reader. The mechanism reduces the risk; it does not eliminate it, and item 3's falsification trigger exists because of that.
- A decline narrows what review may recommend. A maintainer who declines routinely gets reviews that cannot reach `use_existing_dependency` or `publish_public_package`, and the honest response is to supply findings by hand — which is real work this ADR moves onto the human rather than removing.
- The two recording homes are not equally expressive. `existing_packages_considered` carries fit and reason as separate fields; `alternatives_rejected` carries one reason, so research recorded against a `wait_for_more_evidence` decision is compressed into a single sentence per package. Nothing is lost that a later session needs to avoid re-searching, but the record is thinner, and a schema change to even them out has not been bought because no real decision has yet shown it is needed.
- The capability is decided before a single real review has run. If reviews rarely propose a new public crate, this was decided against a decision class that does not occur.
- The boundary lives in Markdown and is enforced by the skill package obeying it. No compiled surface can check any of the allow test's conditions, which is the same assurance ADR 0020 accepted for the fixed no-candidate statement and carries the same drift risk.
- Per-request granularity and a closed host class both make the rule applicable but talkative: a redirect and an unlisted source host cost extra approvals, and the class will need reopening the first time a candidate's documentation or source falls outside it. Worse, a maintainer who tires of the prompts starts approving without reading — the exact failure condition 2's falsification trigger watches for. This is the priced cost of a boundary a reader can actually apply.

### Operational burden

Zero for a review that needs no research: no call, no prompt, no step. For a review that does, one approval showing the exact strings and the named service, plus the generalization judgement on a private case. Nothing is configured, no credential exists, and no account is created.

### Compatibility and migration

Nothing recorded changes. No event schema, marker, command, exit status, or terminal contract is touched, and `existing_packages_considered` is used as already implemented and already required. `CONSUMER-CONTRACT.md` §2's local-first guarantee is unchanged because the core lifecycle still requires no external API and the capability is optional and declinable at every invocation. What changes is what a shipped skill package may instruct an agent to do, which is installed-asset behaviour under `CONSUMER-CONTRACT.md` §4 and revisable during `0.x` under §8.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| No-network rule: the human performs the search and supplies findings | Rejected | It satisfies `FOUNDATIONS.md` §10 by refusing the question, and moves an obligation `EVIDENCE-AND-DECISIONS.md` §9 places on *review* onto the human on every decision that reaches it. `FOUNDATIONS.md` §15 makes that an unfitness: a recurrent workflow the maintainer will avoid is unfit even when correct. It survives as the fallback item 4 specifies, and as the honest outcome if the approved-terms mechanism is falsified. |
| Split rule keyed on the case's derived privacy: public cases may search, private cases may not | Rejected | It makes the majority path the forbidden one, since three of five enrolled repositories are private and `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §6 spreads that across mixed cases. It also mislocates the risk: what leaks is the composed query, not the case's privacy flag, and a public case's query can be composed from text that a private participant would have made private had it been present. Approving the exact terms tests the actual disclosure; the flag only proxies it. Reopen this if item 3 is falsified. |
| Extend `EvidenceKind` and `DecisionContent` so findings become a literal evidence reference | Rejected | A recorded-evidence schema change under `CONSUMER-CONTRACT.md` §3 — the hardest compatibility surface the project has — bought for content the decision event already carries and already requires. #42 also scopes this slice as having no code. |
| Give the compiled command the network call so the mechanic is verifiable | Rejected | `CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §3 lists what the command owns, and composing a semantic query is not on it; §2 puts package research in review. It would put an HTTP client in a binary whose 77 locked packages contain none, and make every adopter carry it for a step most runs never take. |
| Grant the capability to all four skill packages at once | Rejected | `FOUNDATIONS.md` §8 authorizes only the claim the evidence bears. The live decision is review's package research; capture, discovery, and status have no such obligation, and capture in particular is documented as write-free and network-free work. |
| Cache findings in a user-local research index so repeat searches are avoided | Rejected | `FOUNDATIONS.md` §12 refuses control records as a second domain, and the decision event already records what was examined. A cache would also be a place where private-case query terms accumulate outside any case's privacy derivation. |
| Record all of this in #41's PRD or in the review package itself | Rejected | `docs/README.md` ranks PRDs and skills below ADRs, and an issue closes. #41 states the sequencing requirement directly: neither the disclosure boundary nor the work depending on it can be settled by writing the code that assumes an answer. ADR 0012 and ADR 0020 rejected the same placement for the same reason. |

## Verification and review trigger

The decision is fit when a real private-dominant review displays generalized terms and a named target from the permitted class, the human approves them, exactly those terms are sent and nothing else, the findings land in the field the chosen action allows, and a later session reads them from the event without repeating the disclosure. It is equally fit when a review that proposes no new crate completes with no prompt and no call.

**Falsify item 3** if a real review shows that terms specific enough to return useful results are unavoidably specific enough to identify the private consumer. That falsifies the approved-terms mechanism rather than the terms of one search, and the honest successor is the split rule above for private cases, or the no-network rule outright — not a vaguer boundary. `FOUNDATIONS.md` §10's precision requirement is the thing being preserved, so an imprecise repair is not available.

**Falsify the allow test itself** if a real review meets a call the five conditions cannot classify without residual judgement. #42 fixes the response in advance: "if the decision cannot be stated precisely enough to distinguish an allowed call from a disallowed one, the honest outcome is the no-network rule, not a vaguer boundary." Sharpen the offending condition once; if the same class of call resists a second time, withdraw the capability rather than soften the test.

**Falsify the allow test's condition 2** if real use shows the preview is routinely approved without being read, which makes the disclosure gate ceremonial and would mean the boundary is enforced by nothing.

**Narrow or park the whole decision** if package research does not fire in practice because reviews rarely propose a new public crate. The capability would then have been bought against a decision class that does not occur, and the correct response is to withdraw it rather than defend it.

**Reopen item 5** the first time research is recorded against a decision that authorizes no implementation, which is when `alternatives_rejected`'s single reason field is first asked to carry what `existing_packages_considered` splits across two.

**Reopen the allow test's condition 5** the first time a candidate crate's documentation or source is only reachable at a host outside the permitted class.

**Reopen item 4** if the declining path is never exercised, since a capability that is always approved has not demonstrated that declining is genuinely supported.

## Supersession

None. This decision does not amend `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §10; it supplies the distinct accepted capability `FOUNDATIONS.md` §10 requires before §10's conditional permission may be exercised, and it binds §10's five conditions to this one use. It corrects the "evidence reference" recording direction ratified in #41 and #42, which no accepted authority carried.
