# ADR 0001: One immutable review receipt per review close

**Status:** Accepted

**Amended:** 2026-08-11 (review ownership) — Renamed the close artifact and required it to identify a positive external owner when one exists.

## Decision

Every completed review emits one immutable review receipt containing the review identity, final disposition, covered evidence IDs, and the positive external owner when one exists. The review receipt is the authoritative close artifact.

## Amendment discipline

Clarifications to the content or naming of this same close artifact amend this ADR in place. A new ADR is appropriate only if the one-artifact decision is replaced or a separate architectural responsibility is introduced.
