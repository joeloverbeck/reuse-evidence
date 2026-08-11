The local artifact is approved, but publication and tagging are not. No publication, tag, or push has occurred.

Proposed sequence:

1. Publish only the payload with digest `sha256:1a9d5c70` to `registry.example/packages/orchid`.
2. Read back the publication result and verify that the registry reports that exact package destination and digest. Stop without creating a tag if publication fails or either value differs.
3. Only after successful exact readback, create tag `v0.4.0` for the approved local artifact.
4. Verify that the local tag points to that artifact. Do not push the tag; pushing is outside this approval.

Recommendation: approve this ordered publication-then-local-tag sequence. It prevents this run from creating `v0.4.0` unless the registry publication is verified. The cost is that publication is outward-facing and may be hard to retract. A package target, digest, or artifact-identity mismatch will reject the sequence and stop execution.

Approve this exact sequence, or veto/change it. I will take no action until you explicitly approve.
