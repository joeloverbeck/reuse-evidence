# Domain Docs

This repository uses a single-context domain-documentation layout.

## Before exploring, read these

- **`CONTEXT.md`** at the repository root for the shared domain language.
- **`docs/adr/`** for accepted architectural decisions relevant to the work.
- **`docs/principles/`** before either of those when repository authority requires it, as specified by `CLAUDE.md`.

If a referenced domain document does not exist, proceed silently. Do not propose creating it pre-emptively; use the domain-modeling workflow when real terminology or decision pressure requires it.

## File structure

```text
/
├── CONTEXT.md
├── docs/
│   ├── principles/
│   └── adr/
└── src/
```

## Use the glossary's vocabulary

When output names a domain concept—in an issue title, design proposal, hypothesis, or test name—use the term defined in `CONTEXT.md`. Do not drift to synonyms the glossary explicitly avoids.

If a needed concept is absent, reconsider whether the language belongs to this project. If the gap is real, raise it through the domain-modeling workflow.

## Respect authority conflicts

If proposed work contradicts a foundational principle or accepted ADR, stop the conflicting work and surface the conflict. A lower-level issue, PRD, implementation, or test cannot silently amend higher authority.
