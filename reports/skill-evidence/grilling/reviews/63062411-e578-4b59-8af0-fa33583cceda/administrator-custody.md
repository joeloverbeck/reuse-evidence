# Administrator Custody

- Review: `63062411-e578-4b59-8af0-fa33583cceda`.
- Opaque package `cairn`: unchanged current target hash `b9f458b0917db5368c6826732f0daf19b63f157b11d22034c023c8d465794ee6`.
- Opaque package `lumen`: the isolated candidate, if one is constructed.
- Counted roots: `arm-sable` = `cairn`; `arm-ivory` = `lumen`. Neither root nor package name carries a version role.
- Reproduction evaluator labels are frozen before results: odd pairs use `R=cairn, S=lumen`; even pairs use `R=lumen, S=cairn`.
- Protected labels are frozen before results: adjacent and publication use `R=lumen, S=cairn`; verdict-only and authorized-process use `R=cairn, S=lumen`.
- Executors receive one opaque package, one raw task, and named raw artifacts only. The evidence store, diagnosis, authoring rationale, other package, prior outputs, evaluator material, and network are outside their permitted envelope.
- Runner controls use ignored user configuration and rules, ephemeral sessions, JSON event logs, scratch-only execution roots, and read-only evaluators. The administrator audits every log for outside reads and withheld-material leakage.
- Current M2 and protected baselines are reused only because the prior artifacts ran the identical bytes against the identical target hash. M1 and M3 current arms are freshly collected.
