# Keep sensors optional and outside semantic authority

## Consumer effect

A maintainer may use an appropriate external analysis to improve candidate discovery without making that tool, its score, or its provider a prerequisite for capture or a substitute for human semantic judgment.

## Authority

Conforms to `docs/principles/FOUNDATIONS.md` principles 4, 8, and 13, `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §6, accepted ADR 0005, and `docs/design/v0.1-scope-and-acceptance.md` §§2 and 3.

## Narrow scope

- Permit bounded, decision-relevant use of an available external sensor.
- Treat sensor output only as candidate evidence referenced recoverably when useful.
- Preserve ordinary Git, source, tests, history, and agent reasoning as a complete sensor-free path.

## Out of scope

- A built-in exact, structural, fuzzy, semantic, or cross-project detector.
- A mandatory provider, model, embedding, GPU, network API, or sensor dependency.
- A generalized sensor plugin framework.
- CI failure, case opening, readiness, or decisions based on sensor output alone.

## Acceptance

- Capture and discovery work without any sensor installed.
- Sensor output cannot establish responsibility identity, consumer independence, readiness, or acceptance.
- No remote disclosure occurs without separate, explicit authority.
- No CI enforcement or mandatory provider is introduced.
