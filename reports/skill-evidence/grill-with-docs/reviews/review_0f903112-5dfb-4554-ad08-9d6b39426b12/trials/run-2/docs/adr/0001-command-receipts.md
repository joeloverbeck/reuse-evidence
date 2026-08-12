# ADR 0001: Command receipts are rendered by commands

**Status:** Accepted

## Decision

Each command owns the fields and ordering of its command receipt. Shared terminal meanings may have one separate owner, but receipt rendering must not absorb command policy.

## Amendment discipline

Changes to receipt fields amend this ADR. A distinct owner for a shared terminal meaning is a separate structural decision and does not replace this ADR.
