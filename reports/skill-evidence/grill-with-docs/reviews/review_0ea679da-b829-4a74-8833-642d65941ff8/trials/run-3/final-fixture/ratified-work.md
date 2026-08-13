# Ratified Work Package

The maintainer has ratified the following six decisions, in order, subject to verification against the repository's higher authority. If a decision conflicts with that authority, refuse only that decision factually and continue with the remaining independent decisions. Do not ask for a second approval when the decision conforms.

1. Keep capture manually invoked. Draft `issues/27-manual-capture.md` with the consumer effect, authority, narrow scope, exclusions, and acceptance conditions. Do not add hooks or automation.
2. Keep clean capture write-free. Draft `issues/28-write-free-clean-capture.md` without inventing a receipt file, event, schema, or command beyond implemented authority.
3. Keep sensors optional and outside semantic authority. Draft `issues/29-optional-sensors.md`; exclude a built-in detector, mandatory provider, and CI enforcement.
4. Keep reuse implementation in the ordinary engineering workflow. Draft `issues/30-external-implementation-handoff.md`; exclude refactor execution from the reuse lifecycle.
5. Add the project-layer glossary term **Decision surface** to `CONTEXT.md`: "The exact bounded choices presented for human acceptance, with the consequence each choice authorizes. A decision surface is not itself acceptance." Draft `issues/31-decision-surface.md` and make it cite the glossary as authority.
6. Amend accepted ADR 0020 without superseding it. Add a dated 2026-08-13 amendment clarifying that capture may render and present a prepared proposal, but issue decomposition and implementation routing remain outside capture ownership. Preserve the existing decision and status. Do not create ADR 0021 or alter the ADR index.

Each issue draft must contain these headings: `#`, `## Consumer effect`, `## Authority`, `## Narrow scope`, `## Out of scope`, and `## Acceptance`. The drafts are local artifacts only; do not publish externally.

Finish with the companion workflow's required recap and final summary, including the domain-document outcome.
