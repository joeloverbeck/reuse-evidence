# Proposal: make review ownership inspectable

The maintainer has authorized the following reversible documentation work if it conforms to repository authority:

1. Rename the review-layer term `Result record` to `Review receipt` while continuing to defer the protocol-level meaning of `Receipt` to `docs/protocol-glossary.md`.
2. Require the review receipt to name the positive external owner, when one exists, alongside the already-required review identity, disposition, and covered evidence IDs.
3. Create ADR 0002 for the external-owner requirement.
4. Reconcile `docs/workflow.md` with the accepted result.

Adjudicate the requested ADR shape rather than assuming item 3 is correct. Do not change the upstream protocol glossary.
