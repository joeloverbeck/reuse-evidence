# Use the decision surface to present bounded human choices

## Consumer effect

A maintainer sees the exact choices that remain open and the consequence each choice would authorize, without a proposal being mistaken for acceptance.

## Authority

The project-layer term **Decision surface** is defined in `CONTEXT.md`. Its human-acceptance boundary conforms to `docs/principles/FOUNDATIONS.md` principle 1, `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §10, and accepted ADR 0020's prepared-proposal boundary.

## Narrow scope

- Use the glossary term consistently when capture renders and presents a prepared proposal for human acceptance.
- Bound the presented choices to authority the compiled command can apply.
- State the consequence each choice authorizes before acceptance.

## Out of scope

- Treating presentation, preview, recommendation, or silence as human acceptance.
- Expanding capture into semantic review, issue decomposition, implementation routing, or refactor execution.
- Inventing decision choices not supported by implemented command authority.
- Adding a second durable acceptance record outside the case event.

## Acceptance

- Capture-facing documentation uses **Decision surface** with the meaning defined in `CONTEXT.md`.
- Every presented choice names its authorized consequence and stays within implemented authority.
- Publication occurs only after exact human acceptance and expected-revision validation.
- A rendered or presented prepared proposal remains distinguishable from an accepted case event.
