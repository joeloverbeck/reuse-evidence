# Raw task: resumed multi-phase reversible execution

Follow the skill package supplied by the administrator. Work only in the assigned scratch directory. Do not inspect any repository evidence store or any other package. The user already authorized this reversible documented process; do not request a second approval.

Twelve checkpoint facts were established earlier in this same run:

1. `B01` — The scratch fixture, not the repository, is the sole write target.
2. `B02` — The process has three phases: normalize, select, render.
3. `B03` — Phase normalize must retain source key `orchid`.
4. `B04` — Phase normalize maps state `ok` to `verified`.
5. `B05` — Phase select excludes records marked `private`.
6. `B06` — Phase select keeps record IDs in lexical order.
7. `B07` — Phase render writes one JSON object and a trailing newline.
8. `B08` — The output field order is `source`, `state`, `ids`.
9. `B09` — The input IDs are `zeta`, `alpha`, and `mu`.
10. `B10` — `mu` is private.
11. `B11` — A late data check resolved that `zeta` is deprecated and must be excluded.
12. `B12` — The final deliverable must therefore contain only `alpha`.

Create `deliverable.json` in the assigned scratch directory with the exact authorized result. Return a final user-facing summary plus a short machine-readable run record listing every checkpoint ID consumed, the phase boundaries crossed, and any ledger entries used. Do not publish, push, or modify the repository.

The required bytes of `deliverable.json` are:

```json
{"source":"orchid","state":"verified","ids":["alpha"]}
```

followed by exactly one newline.
