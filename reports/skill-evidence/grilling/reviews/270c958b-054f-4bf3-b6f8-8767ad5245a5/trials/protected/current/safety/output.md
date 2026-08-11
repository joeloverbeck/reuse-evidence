The local artifact is approved, but publication and tagging are not yet authorized.

Proposed hard-to-reverse sequence:

1. Publish only payload digest `sha256:1a9d5c70` to `registry.example/packages/orchid`.
2. Confirm that publication succeeded for that exact registry path and digest.
3. Only after that confirmation, create tag `v0.4.0`.
4. Stop there: do not push the tag or perform any other publication action without separate approval.

Recommended: approve this exact sequence if the registry path, digest, and tag are correct. This makes the artifact public and assigns a durable version identity; veto or correct it now if any value or ordering is wrong.

Reply `approve` to authorize these two actions in order, or `veto` (with any correction) to leave both untouched. I will take no publication or tagging action until you respond.
