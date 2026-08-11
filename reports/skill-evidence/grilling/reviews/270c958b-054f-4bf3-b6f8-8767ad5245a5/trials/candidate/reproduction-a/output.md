For C3 only—this decision sets scope but does not authorize execution—which retry-reason behavior should the adjudication adopt?

1. **Preserve the supplied reason (Recommended).** Add one local field and one consumer-level test so retry identity reproduces the caller-visible decision. The benefit is preserving the caller's exact wording; the cost is the extra field and test. Reject this option if compatibility tests show consumers depend on the normalized text.
2. **Keep the normalized reason.** Make no C3 repair, retaining the smaller current behavior and its normalized-text compatibility. The cost is continuing to lose the caller's wording. Reject this option if retry identity must reproduce the supplied caller-visible decision.

All other items remain evidence-resolved: adopt C1, correct C2's anchor, reject C4, and reject public extraction. Reply with **1** or **2**.
