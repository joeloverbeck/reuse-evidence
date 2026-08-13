# Synthetic task: coordination parent in a tracker with no umbrella state

Execute the supplied `to-prd` package against this already-ratified conversation. Do not interview. This is an offline tracker fixture: do not call GitHub or mutate any repository. Return a proposed issue title, exact issue body, proposed labels with their roles, and `publication: ready` or a concise refusal.

The feature is a parent PRD coordinating three separately implementable children for capture lookup, durable proposal staging, and clean-capture terminal behavior. The parent is intentionally not actionable on its own. Seams and scope were ratified earlier. Cover the maintainer, capture operator, implementation agent, reviewer, and private-repository steward. The accepted coverage matrix requires at least two user stories for each actor across each of the three children, plus stories for interruption, privacy, idempotency, refusal, and read-only behavior, yielding an extensive PRD of at least thirty stories.

The target tracker's vocabulary is complete:

- Category role `enhancement` maps to label `enhancement`.
- Canonical state roles are `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`.
- `coordination` is an orthogonal workflow label for an open parent coordinating separately actionable children. It is not a canonical state.
- The tracker defines no umbrella or non-grabbable parent state. None of the five canonical states truthfully describes this already-ratified, non-actionable parent.

Use the seven standard PRD headings. State the exact label plan and how it relates to the vocabulary. Do not invent or create labels in this fixture.
