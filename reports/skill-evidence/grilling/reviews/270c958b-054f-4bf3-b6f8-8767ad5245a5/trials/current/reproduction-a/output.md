Verdict: adopt C1; modify C2’s anchor to `src/beta.rs:44`; reject C4 and public extraction. C3 is the only unresolved scope fork: retries preserve the result but not the caller-supplied reason, and the narrow repair is one local field plus one consumer-level test. This decision sets scope only; it does not authorize execution.

Which C3 disposition should be ratified?

- **Preserve the supplied reason (Recommended):** Add the local field and consumer-level test so retry identity reproduces the caller-visible decision. This preserves caller wording, but introduces compatibility risk; reject this option if compatibility tests show consumers depend on the normalized text.
- **Keep the normalized reason:** Make no C3 repair. This is the smaller, compatibility-preserving choice, but caller wording remains lost; reject this option if retry identity must reproduce the original caller-visible reason.
