The completed local change is ready for closeout. It affects `one.md`, `two.md`, and `three.md`; both the focused tests and the full suite passed.

If authorized, I will perform exactly these three outward-facing mutations on `example/repo#42`, in order:

1. Post the already-previewed GitHub comment whose exact body digest is `sha256:8ee1a244`, then read it back.
2. After that comment is confirmed, remove the `ready-for-agent` label, then read the labels back.
3. After the label removal is confirmed, close issue 42, then read back its final state.

Release publication and tag creation remain out of scope. No comment, label change, issue closure, release, or tag action has occurred.

**Go (recommended):** authorize the exact three-mutation sequence above. **Veto:** perform none of them; choose this if the destination, comment digest, order, or scope is not exactly right. Reply `go` or `veto`—I will not cross the outward-facing mutation boundary before your answer.
