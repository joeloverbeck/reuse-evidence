# Triage Labels

The engineering skills use five canonical triage roles. This file maps those roles to the GitHub labels configured for this repository.

| Canonical role      | GitHub label       | Meaning                                  |
| ------------------- | ------------------ | ---------------------------------------- |
| `needs-triage`      | `needs-triage`     | Maintainer needs to evaluate this issue  |
| `needs-info`        | `needs-info`       | Waiting on reporter for more information |
| `ready-for-agent`   | `ready-for-agent`  | Fully specified, ready for an AFK agent  |
| `ready-for-human`   | `ready-for-human`  | Requires human implementation            |
| `wontfix`           | `wontfix`          | Will not be actioned                     |

When a skill names a triage role, use the corresponding GitHub label from this table.

## Coordination

`coordination` is an orthogonal workflow label, not a canonical triage state. Apply it to an open parent coordinating separately `ready-for-agent` child issues; the parent is not independently grabbable.
