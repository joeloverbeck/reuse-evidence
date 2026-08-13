# ADR Format

ADRs live in `docs/adr/` and use sequential numbering: `0001-slug.md`, `0002-slug.md`, etc.

Create the `docs/adr/` directory lazily — only when the first ADR is needed.

## Template

```md
# {Short title of the decision}

{1-3 sentences: what's the context, what did we decide, and why.}
```

That's it. An ADR can be a single paragraph. The value is in recording *that* a decision was made and *why* — not in filling out sections. And when the repo's existing ADR corpus already follows an internally consistent richer shape (a metadata block, standing sections), match the corpus rather than this template, and treat migrating it as its own surfaced decision — this template governs a repo's first ADRs.

## Optional sections

Only include these when they add genuine value. Most ADRs won't need them.

- **Status** frontmatter (`proposed | accepted | deprecated | superseded by ADR-NNNN`) — useful when decisions are revisited
- **Considered Options** — only when the rejected alternatives are worth remembering
- **Consequences** — only when non-obvious downstream effects need to be called out

A decision that emerges from an extended grilling or design session — with several ratified sub-decisions and explicitly rejected alternatives — legitimately warrants **Considered Options** and **Consequences**. Capturing them there is the intended use of these sections, not a violation of the single-paragraph default.

## Amending an accepted ADR

Editing beats superseding when the decision still stands. When a later revision *refines* an accepted ADR — clarifying a guardrail, sharpening its consequences, correcting scope — but does **not** reverse it, edit the ADR in place rather than spending a new number on it. Record the change so the trail survives:

- add an **`Amended: <date> (<revision>) — <what changed>`** line near the Status/frontmatter, and
- note it wherever this repo already records the revision (the changelog or iteration report that drove it).

Reserve `superseded by ADR-NNNN` (a *new* ADR) for an actual reversal or replacement, and `deprecated` for a decision no longer in force. A clarification is neither.

## Numbering

Scan `docs/adr/` for the highest existing number and increment by one.

## When to offer an ADR

Whether a decision clears the bar is decided in [SKILL.md](./SKILL.md) § *Offer ADRs sparingly* — the single home for the trigger (the scope gate and the three-part test). This section only illustrates what *clears* it.

### What qualifies

- **Architectural shape.** "We're using a monorepo." "The write model is event-sourced, the read model is projected into Postgres."
- **Integration patterns between contexts.** "Ordering and Billing communicate via domain events, not synchronous HTTP."
- **Technology choices that carry lock-in.** Database, message bus, auth provider, deployment target. Not every library — just the ones that would take a quarter to swap out.
- **Boundary and scope decisions.** "Customer data is owned by the Customer context; other contexts reference it by ID only." The explicit no-s are as valuable as the yes-s.
- **Deliberate deviations from the obvious path.** "We're using manual SQL instead of an ORM because X." Anything where a reasonable reader would assume the opposite. These stop the next engineer from "fixing" something that was deliberate.
- **Constraints not visible in the code.** "We can't use AWS because of compliance requirements." "Response times must be under 200ms because of the partner API contract."
- **Rejected alternatives when the rejection is non-obvious.** If you considered GraphQL and picked REST for subtle reasons, record it — otherwise someone will suggest GraphQL again in six months.