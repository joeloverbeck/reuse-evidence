# ADR 0002: Shared success terminal meaning has one owner

**Status:** Accepted

## Context

Independent commands repeated the same mapping from successful command completion to process
status. That public terminal meaning could drift if each command continued to own a copy.

Command receipts remain command-owned under ADR 0001, so sharing this mapping must not move
receipt field selection, receipt ordering, or other command policy into the shared owner.

## Decision

The internal `terminal` module owns the shared success terminal meaning. Commands delegate
their success-status mapping to that module while continuing to render their own receipts and
retain their distinct command policy.

## Consequences

- The success-status mapping has one narrow command-layer owner.
- Commands cannot independently change that shared public meaning by editing local literals.
- Receipt rendering and command-specific policy remain outside the terminal module.
- A future change to receipt fields still follows ADR 0001's amendment discipline.
