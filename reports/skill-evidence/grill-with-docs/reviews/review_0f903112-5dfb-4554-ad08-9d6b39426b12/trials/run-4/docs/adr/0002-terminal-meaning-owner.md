# ADR 0002: Shared success terminal meaning has one owner

**Status:** Accepted

## Decision

The command layer's shared success terminal meaning has one narrow internal owner. Commands delegate their success-status mapping to that owner while retaining ownership of their receipt fields, ordering, and rendering.

## Context

The capture and report commands independently mapped successful completion to the same process status. Keeping those copies independently editable risks divergent public terminal meanings.

## Consequences

- Shared success-status changes have one command-layer location.
- Commands continue to own their distinct receipt policy as required by ADR 0001.
- Other command policy and non-success terminal meanings remain outside this owner unless separately justified.
