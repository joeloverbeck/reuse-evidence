# Reuse Evidence Context

This glossary is the shared language for `reuse-evidence`. Use these terms consistently in principles, ADRs, PRDs, issues, skills, code, reports, and user-facing output.

## Core terms

| Term | Meaning |
|---|---|
| **Primary outcome** | A timely, inspectable human decision about repeated responsibility before independently maintained implementations become expensive divergence. |
| **Responsibility** | A coherent authority, representation, policy, contract, or decision whose behavior and lifecycle can be reasoned about as one thing. Similar syntax alone is not a responsibility. |
| **Reuse consumer** | An independently maintained product, game, subsystem, package, workflow, or repository context that genuinely needs a responsibility. This is distinct from a repository that merely installs the tool. |
| **Occurrence** | One independently accepted reuse-consumer need for a responsibility, supported by recoverable evidence from real work. Files, functions, retries, and generated copies are not automatically separate occurrences. |
| **Independent** | Arising from a distinct consumer need rather than a retry, coordinated variant, generated copy, or internal duplication created by one implementation episode. |
| **Reuse pressure** | Accumulated evidence that several independent consumers require the same responsibility or that an existing shared abstraction is serving consumers that now change for different reasons. |
| **Case** | The durable evidence and decision history for one proposed shared responsibility. A case begins only when at least two plausible independent occurrences exist. |
| **Watching** | The normal state after a second independent occurrence: remember the pressure, but do not yet require a reuse review. |
| **Review-ready** | The normal state after a third independent occurrence, or after a human accepts a documented early-review reason. It authorizes semantic review, not extraction. |
| **Early-review override** | A human-authorized decision to review after the second occurrence because concrete cost or risk makes waiting materially worse. It is not permission to extract automatically. |
| **Reuse review** | A semantic investigation of whether occurrences share a responsibility, what varies legitimately, where authority belongs, and what action is warranted. |
| **Reuse decision** | The exact human-accepted disposition of a case, including its scope, non-responsibilities, migration expectations, and verification conditions. |
| **Implementation brief** | The bounded handoff produced by an accepted decision for the normal engineering workflow. It is not implementation and does not schedule downstream work by itself. |
| **Verification** | Independent evidence that an accepted decision was implemented as authorized and that the named consumers still satisfy their behavioral contracts. |
| **Wrong abstraction** | An existing shared surface that couples consumers which no longer share one responsibility or reason to change. A valid decision may split it and deliberately restore duplication. |
| **Sensor** | An optional external tool or analysis that proposes similarity candidates. A sensor does not establish semantic identity or decision authority. |
| **Evidence reference** | A recoverable pointer to a commit, diff, specification, test, source location, report, package, or other inspectable artifact. Conversation memory alone is not an evidence reference. |
| **Enrolled repository** | A repository that contains a valid `reuse-evidence.toml` marker and therefore consents to local portfolio discovery under its declared visibility. Enrollment is not a migration obligation or backlog. |
| **Portfolio root** | A user-local directory configured for discovery of enrolled repositories. Repositories beneath it are ignored unless they carry the marker. |
| **Ecosystem** | A local set of enrolled repositories that may be compared for reuse pressure. It does not imply a public community, shared runtime, or product-line framework. |
| **Steward repository** | The one repository that owns a case's authoritative event stream and accepted decision. Other participating repositories are referenced consumers, not synchronized copies. |
| **Private dominance** | The rule that one private participant makes the complete case private. Private evidence may not be written into a public steward or public report. |
| **Derived state** | Rebuildable projections such as case readiness, portfolio status, and indexes. Derived state is not an independent source of truth. |
| **Clean capture** | A completed capture that finds no qualifying repeated responsibility. It produces a fixed terminal result and no durable case event. |

## Relationships

- A **responsibility** may have several **occurrences**.
- An **occurrence** belongs to one **reuse consumer** and is supported by **evidence references**.
- A second independent occurrence may open a **case** in a **steward repository**.
- Accumulated occurrences derive **watching** or **review-ready** state.
- A **reuse review** proposes a **reuse decision**.
- Only the human accepts the decision.
- An accepted decision may produce an **implementation brief**.
- **Verification** determines whether the case may close, park, or reopen.
- **Sensors** can help find candidates but never change these authority relationships.
