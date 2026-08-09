---
name: writing-great-skills
description: Author, revise, and finish agent skills — the authoring path, vocabulary, and finishing checks that make a skill predictable.
---

# Writing Great Skills

A skill makes a stochastic agent predictable by stabilizing its process, not its wording or output. Use this reference to author a new skill or revise an existing one.

Arguments: an optional directive, source artifact, precedent, or target skill. Without one, use the conversation's request.

Within reuse-evidence, this skill and current repository guidance are the authoring authority. Treat bundled or other generic skill-authoring instructions as supplementary only where compatible. Never let their default location, initializer, frontmatter allowlist, or validator override the authorized target or write set, `.claude/skills/` ownership, `.agents/skills/` discovery links, client-specific invocation metadata, or repository-native checks; report the mismatch as a compatibility finding.

When another skill dispatches this one, treat its exact source, logical and physical
target, authority boundary, preservation envelope, invocation reach, and write set as
binding authoring inputs. Author only within that write set and return the file
inventory, preservation mapping, unresolved dependencies, pointers, client fields,
and checks the caller requested. Do not perform the caller's source deletion,
transaction edits, or canonical adoption unless the dispatch explicitly assigns them.

## Authoring path

### 1. Establish evidence and the local contract

When a routine has run before, read the named reports, sessions, artifacts, and closest applicable sibling before drafting. Separate stable behavior, vocabulary, file shapes, and ordering from run-specific inputs. From a sibling skill, import only moves that share the new skill's decision surface; source-specific mechanics stay behind.

For an existing skill, inventory and read its complete package before changing it. Map its triggers, branches, safety rules, arguments, outputs, context pointers, instruments, and client metadata so restructuring cannot silently drop behavior.

Check root guidance and the current skill standard, loader, or local validator when frontmatter, discovery, packaging, or layout is involved. If the repository has no authoritative check, use a known-loading same-mode sibling as evidence for client-facing fields and policy files. A generic validator that rejects the same repository-native contract on that sibling is a compatibility finding, not authority to delete required fields.

If a load-bearing input does not exist yet, stop at a bounded dependency stub or handoff that names what is missing, what remains invariant, and what event allows authoring to resume.

*Done when the available evidence and applicable repository contract are recorded, stable conventions are separated from input, and no required design input is being guessed.*

### 2. Settle scope, invocation, and description

Settle only forks that change what ships. Use evidence where it decides; otherwise recommend one option with its consequence and ask for the user's ruling. Do not turn cosmetic preference into a gate.

Choose invocation by reach:

- A **model-invoked** skill is visible for autonomous selection and dispatch from other skills. Its description names what it does and one trigger per material branch; prune trigger synonyms and body identity.
- A **user-invoked** skill is reached explicitly by the human. Its description is a short human-facing identity, and every client surface the repository ships must encode the same explicit-only choice. Put argument expectations in the body or supported metadata rather than assuming one universal field.

Model reach spends **context load** in always-visible metadata; explicit reach spends **cognitive load** because the human must remember the skill. If many explicit-only skills become hard to find, a small router can name them and provide ready invocation lines, but it cannot implicitly fire an explicit-only target.

*Done when scope, arguments, invocation mode, trigger branches, and every outcome-changing fork are explicit and mutually consistent across shipped client surfaces.*

### 3. Draft the smallest executable shape

Use three content forms deliberately:

- **Steps** are ordered actions in `SKILL.md`. End each with a **completion criterion** that is checkable and demanding enough to force the needed legwork.
- **Reference** is consulted on demand. Inline what every branch needs; put lengthy or branch-only material behind a **context pointer** that says when to load it and what to do with it. Default new supporting prose to `references/<topic>.md`, subject to the current standard and an established house-form exception. Co-locate each concept's rules and caveats.
- An **instrument** is an executable check, extraction, or transformation under the repository's script convention. Point to it and run it; do not inline it as prose.

These forms sit on an **information hierarchy** ranked by how immediately the agent needs the material: in-file step, then in-file reference, then reference disclosed behind a pointer. Push too little down and the top bloats; push too much and you hide material the agent actually needs. That tension is the whole placement decision.

A **branch** is a materially different valid route through the skill. Use branch need, not a line threshold, to decide disclosure. Split into another invocable skill only when it needs independent reach. Split a sequence only after an observed premature-completion problem survives a sharper completion criterion, and only across a real context boundary.

Draft from the evidence gathered in step 1. Keep safety invariants, output contracts, and ownership boundaries visible at the point where they constrain action. When revising, preserve every mapped behavior unless the request explicitly authorizes a change.

*Done when every action has an observable bound, every branch reaches exactly the material it needs, every pointer has a target and condition, and the package contains no unsupported behavior change.*

### 4. Prune the package

Keep each meaning in one authoritative place. An output that must stand alone may repeat necessary instructions; the skill itself should point rather than duplicate.

The **environment** is a source of truth too — script definitions, config files, the directory layout, `--help` output — and a skill that restates it is a **cache**: a copy of a lookup, earning its load only when the lookup is expensive. Cache what the agent cannot find by looking: the unwritten convention, the reason behind a choice, the gotcha no config confesses.

Check each sentence for:

- **Relevance** — it still bears on this skill and its current environment.
- **Duplication** — the same instruction has another authoritative home.
- **Sediment** — an old layer remains only because removal feels risky.
- **Sprawl** — live reference crowds the common path and belongs behind a branch pointer.
- **No-op** — removing it would not change agent behavior.
- **False premise** — an empirical rule has not reproduced in the current environment.

A **leading word** can compress repeated explanation when a familiar concept genuinely steers behavior; it is useful only if it changes the process more reliably than the prose it replaces.

**Negation** is the failure mode beside that lever: steering by prohibition drags the forbidden behavior into context and makes it *more* available, not less — the ban half-reads as an instruction to do the thing. Prompt the **positive**: state the target behavior so the banned one is never spoken. A prohibition earns its place only as a hard guardrail you cannot phrase positively, and even then it is paired with the positive target.

*Done when every remaining instruction has one home and a behavioral reason to remain, with uncertain domain knowledge retained rather than deleted by intuition.*

### 5. Finish and prove it

Verify the completed package against the current repository:

1. Parse and load its frontmatter and every client-facing policy surface; confirm the chosen invocation mode and arguments work as intended.
2. Resolve every context pointer. For a sibling-skill dispatch, also confirm reachability; hand the human a ready invocation when the target is explicit-only.
3. Execute every instrument and reproduce every empirical environment claim against a real or inert representative target.
4. Read the package end to end for coherent order, complete sentences, stable numbering, and preserved public outputs.
5. Run applicable repository checks, mirror checks, and other completion contracts from root guidance.

Forward-test in a fresh independent context when the skill has material branches, relies on non-obvious judgment, or changed substantially. Give the test the skill plus representative raw input, not the diagnosis or expected winner; record the expected process before the run and compare actual decisions and artifacts. Keep the test read-only or in scratch space unless the user authorizes realistic effects. A timeout, leaked setup, unexercised material branch, or missing result is not a pass.

A fresh context is not automatically source-blind. When a comparison or claim
requires source-blindness, follow
[Source-blind forward tests](references/source-blind-forward-tests.md) to make
implicit startup context part of the isolation boundary and audit it. Do not
impose that harness on an ordinary independent forward test that makes no
source-blind claim.

*Done when all applicable checks pass, every material branch and invariant has proportionate evidence, and no unresolved failure is reported as completion.*
