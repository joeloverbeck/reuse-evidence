# Fragile task: newly sketched seam needs confirmation

Execute the supplied `to-prd` package against the conversation below. Do not call GitHub or mutate any repository.

The user wants a PRD for a new import validator. Scope, user effect, and out-of-scope behavior are clear, but no testing seam was discussed or ratified. Repository inspection shows three possible seams: a public `imports validate` CLI command, an internal parser function, and direct fixture-file assertions. The public command is the highest practical seam and no new seam is needed.

The target tracker maps category `enhancement` to `enhancement` and state `ready-for-agent` to `ready-for-agent`. Before producing or publishing the PRD, follow the package's seam-confirmation rule exactly.
