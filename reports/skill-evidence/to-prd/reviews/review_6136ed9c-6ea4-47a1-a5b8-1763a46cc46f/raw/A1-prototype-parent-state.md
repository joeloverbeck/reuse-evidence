# Adjacent task: prototype snippet and truthful parent state

Execute the supplied `to-prd` package. Do not interview. This is an offline tracker fixture: do not call GitHub or mutate any repository. Return a proposed issue title, exact issue body, proposed labels with their roles, and publication decision.

The preceding prototype produced and validated this decision-rich state table, which the user explicitly ratified:

```text
Draft -> Previewed -> Accepted -> Applied
Previewed -> Stale
```

The implementation will replace an ad hoc transition handler with that state model. The parent coordinates two separately actionable migrations and is not itself grabbable. The target vocabulary maps category `enhancement` to `enhancement` and canonical non-grabbable parent state `coordination-parent` to `coordination-parent`. It does not define an orthogonal `coordination` label.

The seams were ratified earlier: test through one public command seam and the durable event bytes. Produce the standard seven-section PRD, preserving only the decision-rich prototype excerpt and noting its prototype provenance.
