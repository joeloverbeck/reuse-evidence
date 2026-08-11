Adjudication and reversible execution are complete.

- Adopted `Review receipt` as the review-layer term in [CONTEXT.md](/tmp/grill-evo-R-current-4b/CONTEXT.md).
- Amended [ADR 0001](/tmp/grill-evo-R-current-4b/docs/adr/0001-review-receipts.md) in place to require the positive external owner when one exists.
- Updated the [ADR index](/tmp/grill-evo-R-current-4b/docs/adr/README.md) and [workflow](/tmp/grill-evo-R-current-4b/docs/workflow.md).
- Rejected creating ADR 0002: repository authority explicitly assigns naming and content clarifications for the existing close artifact to ADR 0001.
- Left `proposal.md` and the upstream protocol glossary unchanged.

Recap: in scope were the project glossary, existing ADR, ADR index, and dependent workflow. No work remains pending or unresolved.

Domain-doc outcome: `CONTEXT.md` was updated because the canonical project-layer term crystallized; ADR 0001 was amended because the external-owner requirement clarified its existing artifact.

Artifact class: reversible process and domain documentation.  
Provenance: `docs/principles.md`, accepted ADR 0001, `docs/protocol-glossary.md`, `proposal.md`, and `docs/workflow.md`.  
Convention checked: upstream glossary deference and accepted-ADR amendment discipline.  
Verification: final content and terminology searches agree across all governed documents. Git-based diff validation was unavailable because this workspace is not exposed as a Git worktree.