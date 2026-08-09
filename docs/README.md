# Documentation Map

## Authority order

Earlier layers govern later layers:

1. [`principles/FOUNDATIONS.md`](principles/FOUNDATIONS.md)
2. the topic principles indexed by [`principles/README.md`](principles/README.md)
3. accepted architectural decisions under [`adr/`](adr/)
4. bounded design documents under [`design/`](design/)
5. future PRDs, issues, specifications, skills, schemas, code, tests, and reports

A lower layer cannot silently supersede a higher one. When a conflict is real, amend the governing principle or ADR first through an explicit human decision, then update dependent material.

## Active documents

### Principles

- [`principles/README.md`](principles/README.md) — conformance and index.
- [`principles/FOUNDATIONS.md`](principles/FOUNDATIONS.md) — mission, primary outcome, operating rules, and prohibitions.
- [`principles/EVIDENCE-AND-DECISIONS.md`](principles/EVIDENCE-AND-DECISIONS.md) — occurrence, case, review, decision, and verification semantics.
- [`principles/PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md`](principles/PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md) — enrollment, local discovery, stewardship, and private dominance.
- [`principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md) — ownership of capture, discovery, review, status, mechanics, and external implementation.
- [`principles/CONSUMER-CONTRACT.md`](principles/CONSUMER-CONTRACT.md) — promises and compatibility obligations to repositories that adopt the tool.

### Architectural decisions

See [`adr/README.md`](adr/README.md).

### Active bounded design

- [`design/v0.1-scope-and-acceptance.md`](design/v0.1-scope-and-acceptance.md) — the smallest implementation slice currently authorized.

## Documents intentionally absent

There is no speculative roadmap, product-line architecture, universal sensor contract, detector benchmark program, migration backlog, or hosted-service design. Add one only when a named live decision is blocked on it and the principles authorize the work.
