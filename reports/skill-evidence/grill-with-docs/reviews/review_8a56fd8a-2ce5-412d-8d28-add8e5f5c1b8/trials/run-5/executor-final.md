Adjudication passed with one required modification: item 3 was rejected because ADR 0001 must be amended in place, not replaced by ADR 0002.

Implemented:

- Renamed the project-layer term to `Review receipt` in [CONTEXT.md](/tmp/grill-evo-R-current-5/CONTEXT.md).
- Amended [ADR 0001](/tmp/grill-evo-R-current-5/docs/adr/0001-review-receipts.md) to require the positive external owner when one exists.
- Updated the [ADR index](/tmp/grill-evo-R-current-5/docs/adr/README.md).
- Reconciled [docs/workflow.md](/tmp/grill-evo-R-current-5/docs/workflow.md).
- Left `docs/protocol-glossary.md` unchanged.

Recap: in scope were reversible glossary, existing-ADR, index, and workflow documentation changes. Out of scope were ADR 0002, upstream glossary changes, and changes to the one-receipt architecture. No unresolved decisions or unavailable evidence remain.

Domain-doc outcome: `Review receipt` crystallized as project-layer terminology and was recorded in `CONTEXT.md`; the structural clarification landed as an amendment to accepted ADR 0001 rather than a new ADR.

Artifact class: durable domain and workflow documentation, authorized by the conforming proposal.

Provenance: `proposal.md`, `docs/principles.md`, accepted ADR 0001, `CONTEXT.md`, and `docs/protocol-glossary.md`.

Validation confirmed the workflow, glossary, ADR, and ADR index agree; no ADR 0002 was created.