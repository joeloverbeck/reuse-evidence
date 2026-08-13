# CONTEXT.md Format

## Structure

```md
# {Context Name}

{One or two sentence description of what this context is and why it exists.}

## Language

**Order**:
{A one or two sentence description of the term}
_Avoid_: Purchase, transaction

**Invoice**:
A request for payment sent to a customer after delivery.
_Avoid_: Bill, payment request

**Customer**:
A person or organization that places orders.
_Avoid_: Client, buyer, account
```

## Deferring to an authoritative glossary

When the domain ships its own authoritative glossary (a methodology package, a standards spec, an encyclopedia of domain terms), open `CONTEXT.md` with a deference rule naming it, then define only the terms this project's layer adds on top — never restate upstream terms, or the two definitions will drift:

```md
# {Context Name}

{One or two sentence description of this context.}

> **Authoritative terms** for {domain} are defined in {the upstream glossary + its path}. This file defers to it and holds only the vocabulary this project's layer introduces on top; it never restates upstream terms.

## Language

**{Project-layer term}**:
{Definition of a term this project introduces on top of the upstream domain.}
```

## Rules

- **Be opinionated.** When multiple words exist for the same concept, pick the best one and list the others under `_Avoid_`.
- **Keep definitions tight.** One or two sentences max. Define what it IS, not what it does.
- **Only include terms specific to this project's context.** General programming concepts (timeouts, error types, utility patterns) don't belong even if the project uses them extensively. Before adding a term, ask: is this a concept unique to this context, or a general programming concept? Only the former belongs.
- **Group terms under subheadings** when natural clusters emerge. If all terms belong to a single cohesive area, a flat list is fine.
- **Defer to an authoritative domain glossary when one exists.** When the domain ships its own glossary document (a methodology package, a standards spec), open `CONTEXT.md` with a deference rule naming it, and define only the terms this project's layer introduces on top — never restate or paraphrase upstream terms, or the two definitions will drift.
- **An existing file's consistent shape wins.** When the governing `CONTEXT.md` already uses an internally consistent shape that diverges from this template — a different section heading, entry markup, or no `_Avoid_` lists — match the file, and treat migrating it to this format as its own surfaced decision. This template governs freshly created files.

## Single vs multi-context repos

**Single context (most repos):** One `CONTEXT.md` at the repo root.

**Multiple contexts:** A `CONTEXT-MAP.md` at the repo root lists the contexts, where they live, and how they relate to each other:

```md
# Context Map

## Contexts

- [Ordering](./src/ordering/CONTEXT.md) — receives and tracks customer orders
- [Billing](./src/billing/CONTEXT.md) — generates invoices and processes payments
- [Fulfillment](./src/fulfillment/CONTEXT.md) — manages warehouse picking and shipping

## Relationships

- **Ordering → Fulfillment**: Ordering emits `OrderPlaced` events; Fulfillment consumes them to start picking
- **Fulfillment → Billing**: Fulfillment emits `ShipmentDispatched` events; Billing consumes them to generate invoices
- **Ordering ↔ Billing**: Shared types for `CustomerId` and `Money`
```

The skill infers which structure applies:

- If `CONTEXT-MAP.md` exists, read it to find contexts
- If only a root `CONTEXT.md` exists, single context
- If neither exists, create a root `CONTEXT.md` lazily when the first term is resolved
- A repo *declared* multi-context (e.g. in its agent docs) but with zero or one actual contexts still starts with a root `CONTEXT.md`; create `CONTEXT-MAP.md` only when a second context actually exists

When multiple contexts exist, infer which one the current topic relates to. If unclear, ask.