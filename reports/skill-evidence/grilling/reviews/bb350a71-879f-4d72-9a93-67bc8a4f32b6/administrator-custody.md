# Administrator custody

- Review: `bb350a71-879f-4d72-9a93-67bc8a4f32b6`
- Opaque package `meridian`: unchanged current target hash `b9f458b0917db5368c6826732f0daf19b63f157b11d22034c023c8d465794ee6`
- Opaque package `harbor`: isolated candidate changing only `references/execution.md`
- Counted execution roots: `arm-cobalt` = `meridian`; `arm-amber` = `harbor`. These labels were frozen before the counted reruns and carry no semantic version word.
- Executors receive one opaque package, one raw task, and named raw artifacts only.
- Executors are barred from `reports/skill-evidence/`, other skill packages, the other arm, the diagnosis, prior outputs, and network or real-repository mutation.
- Reproduction evaluator labels alternate before results: odd pairs present `R=meridian, S=harbor`; even pairs present `R=harbor, S=meridian`.
- Protected evaluator labels use `R=harbor, S=meridian` for adjacent and safety; `R=meridian, S=harbor` for verdict and authorized-process.
- The exact prior-current protected artifacts are reusable baseline evidence because they executed these same raw inputs against the same target hash. The current review copies them byte-for-byte and does not rerun equivalent current-arm trials.
- The first attempted executions used scratch directories containing `current` and `candidate`. `writing-great-skills` made that a version-role leak. Those artifacts are quarantined under `trials/diagnostic-label-leaked/` and were not counted or compared.
- Counted trial roots and every evaluator run used `--ignore-user-config`, `--ignore-rules`, `--ephemeral`, JSON event logs, scratch-only executor access, and read-only evaluators. The retained logs contain no evidence-store, memory, held-package, diagnostic-run, or cross-response reads.
