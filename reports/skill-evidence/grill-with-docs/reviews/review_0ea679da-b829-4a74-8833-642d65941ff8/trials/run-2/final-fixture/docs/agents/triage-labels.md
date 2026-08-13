# Triage Labels

The engineering skills use five canonical triage roles. This file maps those roles to the GitHub labels configured for this repository.

| Canonical role      | GitHub label       | Meaning                                                        |
| ------------------- | ------------------ | -------------------------------------------------------------- |
| `needs-triage`      | `needs-triage`     | Maintainer needs to evaluate this issue                          |
| `needs-info`        | `needs-info`       | Waiting on reporter for more information                         |
| `ready-for-agent`   | `ready-for-agent`  | Specified for the maintainer's unattended agent                  |
| `ready-for-human`   | `ready-for-human`  | Requires the maintainer's hands-on implementation                |
| `wontfix`           | `wontfix`          | Will not be actioned                                             |

When a skill names a triage role, use the corresponding GitHub label from this table.

## This is not a contribution queue

These labels describe the maintainer's own workflow on a public repository. They do not assign work to anyone else, and no label makes an issue available to claim.

`ready-for-agent` in particular means only that an issue is specified tightly enough for the *maintainer's* unattended agent to implement. Outside contributors read it as an open invitation; it is not one, and [`CONTRIBUTING.md`](../../CONTRIBUTING.md) says so publicly. Keep the label descriptions on GitHub consistent with this file, because the label sidebar is the surface an outside contributor actually reads.

Assignment, not a label, is the signal that an issue is claimed. Self-assign an issue when starting work on it so a concurrent contributor can see the collision before doing the work.

## Coordination

`coordination` is an orthogonal workflow label, not a canonical triage state. Apply it to an open parent coordinating separately `ready-for-agent` child issues; the parent is not independently actionable.
