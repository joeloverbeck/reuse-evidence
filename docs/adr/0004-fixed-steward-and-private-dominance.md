# ADR 0004: Fixed stewardship and private dominance

**Status:** Accepted  
**Date:** 2026-08-09  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md`](../principles/PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md)

## Context

A cross-repository case needs one authoritative history without synchronizing copies across every participant. The initial portfolio includes commercially sensitive private repositories and public repositories.

A central ledger repository would become a new source of truth and a privacy concentration point. Copying occurrence receipts into every repository would create reconciliation, partial-write, and stale-copy problems. Moving stewardship automatically into a new public extraction could leak private provenance.

## Decision

Every case has one fixed steward repository.

- Intra-repository cases are stewarded locally.
- A new cross-repository case is normally stewarded by the repository in which the second occurrence is recognized.
- One private participant makes the complete case private.
- A mixed-visibility case must have a private steward.
- Other participating repositories are referenced consumers and are not modified merely because the case exists.
- Cross-repository writes require separate explicit authority and should be avoided when the steward record is sufficient.
- Version 0.1 does not support ordinary stewardship transfer.
- A public extraction produced from private pressure receives a sanitized public contract and tests, not the private case history.
- The private steward closes the case by referencing the public result.

## Consequences

### Positive

- One authoritative stream avoids synchronization and consensus machinery.
- Private commercial evidence stays private.
- Public packages can still emerge without exposing their private origins.
- Participating repositories do not acquire surprise files or writes.

### Negative and risks

- The steward can become unavailable.
- A contributor in another repository may need local access to the steward to inspect the full case.
- Fixed stewardship may be inconvenient after repository retirement.

These risks are accepted for version 0.1. A successor mechanism should be designed only after a real steward-retirement case exists.

### Operational burden

Committed cross-repository evidence should prefer stable repository IDs, Git identities, and relative references. Absolute paths remain in the disposable local index.

### Compatibility and migration

Removing an enrolled repository does not erase historical occurrences in a steward case. Public/private visibility changes must be reported and cannot silently republish existing case evidence.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Copy case receipts into every participant | Rejected | Creates distributed synchronization and partial-write failure. |
| Central portfolio ledger repository | Rejected | New source of truth, privacy concentration, and backlog pressure. |
| Always steward in the new shared package | Rejected | Confuses implementation ownership with historical evidence and risks private leakage. |
| Automatic stewardship transfer | Rejected for version 0.1 | No real pressure; substantial provenance and idempotency complexity. |
| Keep all cross-repo cases only in user-local SQLite | Rejected | Opaque, unversioned, and not durable repository authority. |

## Verification and review trigger

Reopen when a real steward must be retired, split, or made inaccessible and the case cannot be closed or preserved safely under the fixed model.

## Supersession

None.
