---
name: domain-modeling
description: Build and sharpen a project's domain model. Use when the user wants to pin down domain terminology or a ubiquitous language, record an architectural decision, or when another skill needs to maintain the domain model.
---

# Domain Modeling

Actively build and sharpen the project's domain model as you design. This is the *active* discipline — challenging terms, inventing edge-case scenarios, and writing the glossary and decisions down the moment they crystallise. (Merely *reading* `CONTEXT.md` for vocabulary is not this skill — that's a one-line habit any skill can do. This skill is for when you're changing the model, not just consuming it.)

## File structure

Most repos have a single context:

```
/
├── CONTEXT.md
├── docs/
│   └── adr/
│       ├── 0001-event-sourced-orders.md
│       └── 0002-postgres-for-write-model.md
└── src/
```

If a `CONTEXT-MAP.md` exists at the root, the repo has multiple contexts. The map points to where each one lives:

```
/
├── CONTEXT-MAP.md
├── docs/
│   └── adr/                          ← system-wide decisions
├── src/
│   ├── ordering/
│   │   ├── CONTEXT.md
│   │   └── docs/adr/                 ← context-specific decisions
│   └── billing/
│       ├── CONTEXT.md
│       └── docs/adr/
```

Create files lazily — only when you have something to write. If no `CONTEXT.md` exists, create one when the first term is resolved. If no `docs/adr/` exists, create it when the first ADR is needed.

The `CONTEXT.md` and `docs/adr/` that govern are those of the repo that *owns the domain under discussion* — which may not be the session's working directory (worktree, sibling checkout, cross-repo invocation). Read that repo's glossary before challenging or coining terms, and write updates there.

## During the session

When this skill is invoked as a companion to another workflow, domain-model only if something actually crystallises. After checking the governing glossary, if no term conflict, fuzzy term, resolved term, or ADR-worthy decision emerges, do not create or edit `CONTEXT.md` or ADR files. In companion mode a targeted glossary check suffices — read the glossary's preamble/deference rule and grep for the session's candidate terms; a full read is owed only before challenging an existing term or restructuring the glossary. A term resolved *into an authoritative upstream doc* (one the glossary's deference rule names) is upstream's to define — it is not a `CONTEXT.md` entry, and the recap's "considered, no update needed" line covers it. In the final recap, state the outcome either way: if updates crystallised, name where each landed — and, when an ADR-worthy-looking decision was homed in a changelog or other existing log instead of a new ADR, say so and why; otherwise state that domain modeling was considered and no domain-doc update was needed.

### Challenge against the glossary

When the user uses a term that conflicts with the existing language in `CONTEXT.md`, call it out immediately. "Your glossary defines 'cancellation' as X, but you seem to mean Y — which is it?"

### Sharpen fuzzy language

When the user uses vague or overloaded terms, propose a precise canonical term. "You're saying 'account' — do you mean the Customer or the User? Those are different things."

### Discuss concrete scenarios

When domain relationships are being discussed, stress-test them with specific scenarios. Invent scenarios that probe edge cases and force the user to be precise about the boundaries between concepts.

### Cross-reference with code

When the user states how something works, check whether the code agrees. If you find a contradiction, surface it: "Your code cancels entire Orders, but you just said partial cancellation is possible — which is right?"

### Update CONTEXT.md inline

When a term is resolved, update `CONTEXT.md` right there. Don't batch these up — capture them as they happen. Use the format in [CONTEXT-FORMAT.md](./CONTEXT-FORMAT.md). Exception: when the glossary file's own existence or shape is still an open decision in the session, hold resolved terms aside and write them in one pass once it's ratified. The same holds when a companion workflow's ratification gate governs the session's writes: pending term updates ride that gate — written in the ratified batch, with the glossary flushed no later than the deliverable that leans on the terms — and the final recap names them.

Before publishing any durable deliverable (PRD, spec, report) that coins or leans on a newly-resolved term, verify each such term against the governing `CONTEXT.md` and flush pending glossary updates first — deferring the glossary write into the deliverable itself is exactly the drift the inline rule exists to prevent.

**Before writing any term into a fresh `CONTEXT.md`, check for an authoritative upstream glossary.** When the domain ships its own (a methodology package, a standards spec, an encyclopedia of domain terms), `CONTEXT.md` must *open* with a deference rule naming it, and then hold only the terms this project's layer introduces on top — never restate or paraphrase upstream terms. See the deference rule and its example opening in [CONTEXT-FORMAT.md](./CONTEXT-FORMAT.md).

`CONTEXT.md` should be totally devoid of implementation details. Do not treat `CONTEXT.md` as a spec, a scratch pad, or a repository for implementation decisions. It is a glossary and nothing else.

### Offer ADRs sparingly

An ADR is for an **architectural or structural** decision — one about how the system or its documents are shaped — not for content the project revises through its normal versioned output. Before applying the test below, confirm the decision has **no existing home**: if the repo already records this class of decision in a changelog, an iteration report, or an upstream decision-log named by the glossary's deference rule, that log is its home and an ADR would duplicate it — record it there instead.

Otherwise, only offer to create an ADR when all three are true:

1. **Hard to reverse** — the cost of changing your mind later is meaningful
2. **Surprising without context** — a future reader will wonder "why did they do it this way?"
3. **The result of a real trade-off** — there were genuine alternatives and you picked one for specific reasons

If any of the three is missing, skip the ADR. For the template, numbering, and examples of what qualifies, see [ADR-FORMAT.md](./ADR-FORMAT.md); to *amend* an already-accepted ADR whose decision still stands (clarifying its guardrails or consequences without replacing it), see that file's § *Amending an accepted ADR*.
