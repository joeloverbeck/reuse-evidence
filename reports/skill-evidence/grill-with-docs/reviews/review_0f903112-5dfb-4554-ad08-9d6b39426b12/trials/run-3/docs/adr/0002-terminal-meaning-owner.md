# ADR 0002: One owner for the success terminal meaning

**Status:** Accepted

## Decision

The narrow internal terminal module owns the command layer's shared success terminal meaning. Commands delegate their success-status mapping to that owner while continuing to render their own receipts.

## Rationale

Keeping the shared mapping in one place prevents independently edited commands from giving the same public terminal meaning different representations. A broader global status model would also absorb command-specific receipt policy and conflict with ADR 0001.

## Relationship to ADR 0001

This decision does not replace or amend ADR 0001. Each command still owns its receipt fields and ordering; only the shared terminal meaning has a separate owner.
