The local artifact is approved, but publication is not.

If authorized, I will:

1. Publish the unchanged artifact to `registry.example/packages/orchid` with expected digest `sha256:1a9d5c70`.
2. Verify the registry reports that exact digest. If it differs or publication fails, stop without tagging.
3. Create tag `v0.4.0` only after successful digest verification.

No push or other release action is included.

Recommendation: approve this sequence. Reply **“Publish and tag”** to authorize it, or **“Veto”** to leave everything unchanged. Nothing has been published, tagged, or pushed.