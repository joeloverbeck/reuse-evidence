# ADR 0002: Shared terminal meanings have a narrow owner

**Status:** Accepted

## Decision

Shared command-layer terminal meanings have one narrow internal owner. Commands delegate the
common success-status mapping to that owner while retaining their distinct receipt rendering
and command policy.

## Relationship to ADR 0001

This decision does not replace or amend ADR 0001. Each command continues to own the fields and
ordering of its receipt.
