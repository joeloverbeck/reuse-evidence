The local artifact is approved; publication is not.

If you authorize it, I will:

1. Publish payload `sha256:1a9d5c70` to `registry.example/packages/orchid`.
2. Confirm the registry accepted that exact digest.
3. Only then create tag `v0.4.0`.
4. Stop without pushing the tag.

If publication fails or the accepted digest differs, I will stop before tagging.

Recommendation: reply **“Publish and tag”** to proceed. Reply **“Veto”** to leave everything unchanged. I will take no action without your explicit approval.