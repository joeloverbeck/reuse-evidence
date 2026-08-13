# Prepared proposal mechanics

Load this reference only after a qualifying candidate has survived independence assessment and
`reuse-evidence case find` has selected the opening or append branch.

## Opening branch

Create a draft with a freshly generated UUID version 4 and at least two plausible independent
occurrences:

```toml
case_id = "<generated-uuid-v4>"
responsibility = "<one coherent authority, representation, policy, or contract>"

[[occurrences]]
repository_id = "<resolved-enrolled-repository-id>"
consumer = "<independently maintained consumer>"
independence = "<distinct authority, lifecycle, release obligation, or reason to change>"

[[occurrences.evidence]]
kind = "commit"
reference = "<recoverable-commit>"
path = "<optional-repository-relative-path>"

[[occurrences]]
repository_id = "<resolved-enrolled-repository-id>"
consumer = "<independently maintained consumer>"
independence = "<distinct authority, lifecycle, release obligation, or reason to change>"

[[occurrences.evidence]]
kind = "commit"
reference = "<recoverable-commit>"
path = "<optional-repository-relative-path>"
```

Preview from the proposed steward repository:

```console
reuse-evidence case open --proposal <staged-draft> [--root <portfolio-root> ...] --preview
```

Opening writes the approved event only in the enrolled repository containing the command's working
directory. The compiled preview resolves all participants and supplies the authoritative privacy
consequence. A public steward with any private participant refuses before publication.

## Append branch

Use the existing case identity and obtain its fresh revision with `reuse-evidence case show` from
inside the steward. Draft exactly one new occurrence:

```toml
[occurrence]
repository_id = "<resolved-enrolled-repository-id>"
consumer = "<independently maintained consumer>"
independence = "<distinct authority, lifecycle, release obligation, or reason to change>"

[[occurrence.evidence]]
kind = "commit"
reference = "<recoverable-commit>"
path = "<optional-repository-relative-path>"
```

Preview from that steward repository:

```console
reuse-evidence case append <case-id> --expected-revision <revision> \
  --proposal <staged-draft> [--root <portfolio-root> ...] --preview
```

The participant-and-consumer pair must be new to the case. A private new participant under a
currently public steward refuses before publication.

## Exact approval transaction

The preview receipt ends with a line containing only `event:` followed by the exact TOML bytes the
command proposes to record. Preserve every byte after that line through end of output, including
the final newline, in a second file beneath the compiled staging directory. Do not reconstruct,
normalize, or serialize it.

Show the human the complete receipt and ask for authorization of:

1. those exact event bytes;
2. the named steward event path and privacy consequence; and
3. removal of the staged draft and event after exact publication is read back.

After approval, repeat the same command without `--preview` and replace `<staged-draft>` with the
file containing the approved event bytes. For append, keep the same case identity, expected
revision, and portfolio roots. For opening, keep the same portfolio roots and working directory.

On success, byte-compare the named steward event file to the approved staged file before removing
both staged files. An `existing` receipt is successful only when this exact comparison holds. On
refusal or failure, keep the approved file so an interrupted session never requires the human to
approve the same event again.
