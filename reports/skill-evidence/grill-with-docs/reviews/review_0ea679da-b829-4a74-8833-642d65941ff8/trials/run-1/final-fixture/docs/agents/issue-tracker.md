# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues. Use the `gh` CLI for all operations.

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."`. Use a heredoc for multi-line bodies.
- **Read an issue**: `gh issue view <number> --comments`, filtering comments by `jq` and also fetching labels.
- **List issues**: `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'` with appropriate `--label` and `--state` filters.
- **Comment on an issue**: `gh issue comment <number> --body "..."`
- **Apply or remove labels**: `gh issue edit <number> --add-label "..."` or `--remove-label "..."`
- **Close an issue**: `gh issue close <number> --comment "..."`

Infer the repository from `git remote -v`; `gh` does this automatically when run inside this clone.

## Pull requests as a triage surface

**PRs as a request surface: yes.**

External contributors' PRs run through the same labels and states as issues. Maintainers' and collaborators' in-flight PRs do not.

Triage the PR, not only the code. [`CONTRIBUTING.md`](../../CONTRIBUTING.md) accepts external pull requests only against an issue the maintainer assigned to that contributor. So the first triage question is whether the PR was solicited:

- **Unsolicited** — close it with thanks and point at `CONTRIBUTING.md`. Do not label it into the work queue and do not review it as if it were assigned work. Read the diff far enough to say whether anything in it is worth salvaging separately, and credit the contributor if something is taken.
- **Assigned** — triage it with the labels above like any other request.

A close is not a judgment on the code. Concurrent independent implementations of a tightly specified issue converge, so a contributor who lost a race can produce a correct patch that is still not mergeable.

- **Read a PR**: `gh pr view <number> --comments` and `gh pr diff <number>`
- **List external PRs for triage**: `gh pr list --state open --json number,title,body,labels,author,authorAssociation,comments`, then keep only `CONTRIBUTOR`, `FIRST_TIME_CONTRIBUTOR`, or `NONE` author associations.
- **Comment, label, or close**: use `gh pr comment`, `gh pr edit --add-label` or `--remove-label`, and `gh pr close`.

GitHub shares one number space across issues and PRs. Resolve an ambiguous `#42` with `gh pr view 42`, falling back to `gh issue view 42`.

## When a skill says "publish to the issue tracker"

Create a GitHub issue.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --comments`.
