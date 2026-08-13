# Questions And Ledger

## One Question At A Time

Ask only a fork that a reasonable user might decide differently and whose answer changes what
ships. Put the recommended option first and explain:

- what each option changes in the deliverable;
- its main benefit;
- its main cost, coupling, or identity risk; and
- the evidence or condition that would reject it.

Keep comparisons symmetric. A coined label is navigation, not an explanation. If a stronger
governing workflow already provides a decision packet, reuse it and add only a missing field.

Use the available question vehicle when it can render the full decision. Otherwise use a
visible prose prompt with the same recommended-first go/veto or option shape. Ask one
independent fork at a time; a tightly coupled sub-decision may share it, and independent
in/out toggles over one surface may use a multi-select.

Give lighter variants their own option. When options differ by an artifact shape, show the
relevant table, fields, configuration, or code preview before ratification. Validate any
computed value in a preview against the authoritative artifact first.

## Rendering And Recovery

Some question tools hide same-turn prose. Put all rationale needed to answer inside the
rendered question/options or in a prior visible turn. When findings or a verdict body are too
large for that surface, deliver them first and ask the fork in the next turn. If the client
cannot initiate another assistant turn without user input, end on the findings; ask the fork
after the user's next message if it remains unresolved.

If a question is unanswered, proceed only with work invariant across the options. After one
soft timeout, take at most one bounded invariant step and carry the fork forward. Repeated
non-response or an explicit away signal permits more invariant work, never a choice on the
user's behalf.

When the user asks for clarification, answer visibly and re-present the same fork. If they
cannot distinguish the options, decline the question call, or report a mis-selection, treat it
as a comprehension reset: infer no decision, rebuild and re-verify the comparison, say if the
recommendation changed, then re-present it. A visible rebuilt comparison may end with its own
prose go/veto prompt.

When the user volunteers answers to several pending branches, accept and record each answer;
do not re-ask them one by one.

## Running Ledger

Record each ratified, evidence-resolved, or derived decision with a stable key, the answer,
its rationale, and its status. In adjudication, the disposition list is the ledger; do not
duplicate it.

For multi-file or multi-phase execution, select or initialize the ledger before the first
mutation. Use the governing process's ledger when it has one; otherwise keep a run-local
scratch ledger unless the repository independently requires persistence. Narration is enough
for short, single-artifact work.

Capture every mid-execution fork when it is taken and before the next mutation. Consecutive
mechanical edits may batch the file write only if the visible narration captured each fork at
the time. Before delivery, sweep the ledger and ensure every mid-execution entry appears once
in the final summary; resolve omissions or extras.

A data-dependent decision may be ratified as a rule with its trigger and premise. Apply it
later only while that premise still holds; a changed premise reopens the fork.
