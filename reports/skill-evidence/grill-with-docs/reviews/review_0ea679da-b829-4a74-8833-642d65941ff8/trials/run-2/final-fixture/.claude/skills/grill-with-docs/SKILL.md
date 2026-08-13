---
name: grill-with-docs
description: Run grilling with domain-modeling; create or update domain docs only when terms or ADR-worthy decisions crystallize.
disable-model-invocation: true
---

Use the `grilling` skill with `domain-modeling` as its companion (Claude aliases: `/grilling` and `/domain-modeling`).

- First classify the `grilling` shape: interview, adjudication, or user-directed execution of a documented process.
- Let `grilling` own the decision protocol: verify premises, collapse branches, ask only outcome-changing questions, recap, then execute reversible work when ratified.
- Use `domain-modeling` only for live term or decision work. Check the governing context, then update `CONTEXT.md` or ADRs only when a term or ADR-worthy structural decision crystallizes.
- If no domain-doc update is needed, say that explicitly in the final recap.
