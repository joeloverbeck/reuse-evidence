# Keep sensors optional and outside semantic authority

## Consumer effect

Allow maintainers to improve candidate discovery with available tools without making capture depend on a provider or mistaking similarity output for a reuse decision.

## Authority

Conforms to `docs/principles/FOUNDATIONS.md` §§8 and 13, `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §6, ADR 0005, and `docs/design/v0.1-scope-and-acceptance.md` §§2–3.

## Narrow scope

Accept recoverable references to optional external sensor evidence as candidate-discovery input. Preserve ordinary Git, source inspection, history, tests, and agent reasoning as a complete path when no sensor is available.

## Out of scope

- A built-in detector or generalized sensor plugin framework.
- A mandatory provider, model, embedding service, GPU, network call, or remote source upload.
- CI enforcement based on candidates or scores.
- Sensor authority over responsibility identity, independence, readiness, or acceptance.

## Acceptance

- The core workflow completes without a sensor.
- Any sensor result is treated only as referenced candidate evidence.
- Human semantic authority and private-disclosure boundaries remain explicit.
- Reopen only if repeated real cases fail specifically because detector capability is absent, under ADR 0005's trigger.
