# ADR 0003: Marker-enrolled local portfolio

**Status:** Accepted  
**Date:** 2026-08-09  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md`](../principles/PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md)

## Context

The maintainer's repositories live beneath local source roots, but those roots can also contain experiments, archives, vendor checkouts, abandoned work, and unrelated repositories. Scanning every Git repository by default would create false authority, privacy risk, and operational noise.

A central manually maintained portfolio list would drift from repository reality and make local path changes authoritative state.

The maintainer prefers repositories to declare participation by adopting the tool themselves.

## Decision

Portfolio participation is marker-enrolled and local-first.

- User-local configuration names one or more roots to inspect.
- A repository participates only when it contains a valid versioned `reuse-evidence.toml` marker.
- The marker carries stable repository and ecosystem identity plus declared visibility.
- Paths are discovered data, not durable identity.
- Each explicit portfolio command rescans roots and reports new, moved, unavailable, duplicate-ID, unsupported-version, or visibility-changed repositories.
- Unmarked repositories are ignored.
- Enrollment does not require a production dependency on the crate.
- Version 0.1 has no daemon, background watcher, or central repository registry.
- A user-local index may cache discovery and case projections but remains disposable and rebuildable.

## Consequences

### Positive

- Participation is explicit and visible in each repository.
- Private and abandoned repositories are not scanned accidentally.
- Moving a repository does not require editing committed portfolio state.
- Newly enrolled repositories are detected naturally on the next command.
- The tool does not become a runtime dependency of the products it observes.

### Negative and risks

- A repository can be omitted because the marker was never added.
- Every participating repository carries a small configuration file.
- Duplicate or copied repository IDs require safe refusal and repair.

### Operational burden

The marker should remain minimal. Do not add language-specific roots, detector configuration, family taxonomies, or workflow policy until real repositories require them.

### Compatibility and migration

Marker schema versions must be explicit. A path move is not a new repository. Removing the marker withdraws future participation without rewriting historical steward cases.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Include every Git repository beneath `~/src` | Rejected | Excessive noise and privacy risk. |
| Maintain an explicit central path list | Rejected | Duplicates repository truth and drifts on moves. |
| Require a Cargo dependency in each repository | Rejected | Confuses tooling participation with runtime coupling and excludes TypeScript repos. |
| Background discovery daemon | Rejected for version 0.1 | No proven need; adds lifecycle and failure surface. |
| Repository-family taxonomy from the start | Parked | Thematic families may overfit; add only if real search noise requires them. |

## Verification and review trigger

Reopen if root rescanning becomes operationally unacceptable or real portfolios need bounded family selection that cannot be expressed without new durable distinctions.

## Supersession

None.
