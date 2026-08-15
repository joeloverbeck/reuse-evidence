# Verifier Packets

Read this only when an external or cold-context verifier receives a self-contained packet
instead of direct access to the authoritative environment.

## Closure Before Collection

Packet sufficiency is a premise. Inventory every claim and the authoritative facts required to
judge it, confirm the packet contains those facts, and run a disposable fresh-context pilot.
The pilot is noncanonical.

If the pilot reports a missing fact, check the authority:

- if the authority supplies it, version and retain the superseded packet, correct the packet,
  and repeat the pilot;
- if the authority does not supply it, preserve UNKNOWN or unavailable rather than guessing.

A governing protocol may intentionally withhold facts to test whether an artifact carries its
own claim. Preserve those declared exclusions; sufficiency means fit for the stated test, not
maximum context.

## Administrator Handoff

Keep the verifier-facing prompt blind to pilot status, private targets, expected winners, and
branch information. Separately show the administrator:

- the exact prompt identity;
- the pilot and its custody path;
- the closure record;
- the validation command and admitted output; and
- the no-go branch.

Canonical collection begins only after the administrator has seen that sequence and closure
passes. When execution leaves the current thread, retain acknowledgment or a machine-readable
closure artifact.

## Response And Access Envelope

Before freezing a receiver, test every admitted provider/model/version/access stratum with a
representative real or pinned envelope. Normalize case, punctuation, and known vendor aliases
only through a table frozen before canonical responses. Normalization never changes a
judgment, reason, selected object, threshold, or branch.

For every response, record:

- the exact task material supplied;
- channels opened and material actually read;
- whether any returned material carried task-specific evidence or prior task knowledge; and
- prompt-local computation or serialization tools.

Ambient generic instructions are production context, not automatically outside evidence.
Repository, memory, prior-response, or skill material carrying task facts may defeat blindness
without network access. Preserve raw flags and disclosures, then derive admissibility from the
predeclared channel/material/claim rules; do not make one response-side boolean serve as both
provenance and verdict.
