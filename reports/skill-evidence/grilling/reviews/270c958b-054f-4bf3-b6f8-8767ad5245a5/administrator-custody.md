# Administrator Custody

- Neutral arm mapping frozen before the first behavioral output:
  - `arm-cobalt` = unchanged target at `b9f458b0917db5368c6826732f0daf19b63f157b11d22034c023c8d465794ee6`
  - `arm-amber` = isolated candidate at `d26e96d3c1b1bae6e823b5f20df56cd14d719e5904a0800cc40708f597b35337`
- Trial root: `/tmp/grilling-evolution-270c958b.7GzdBP`
- Executors receive only their neutral arm, raw packet, assigned scratch directory, and an access prohibition covering the repository evidence store, the other arm, and prior outputs.
- Reproduction packet hashes:
  - A: `0b1f314d5a88bf9bc09c3437034c06615a879d3f486280107c70a2558edca661`
  - B: `57d14ad730df555de00af7837b204a16cfa67e84fdf89c178bad6c0bfc1267df`
  - C: `deb781b28f6e9ee4c34511c27039c100ecb45e0c63178154f1ea5900356236c7`
- Candidate construction changed only `SKILL.md`. Runtime size fell from 586 words / 4235 bytes to 570 words / 4095 bytes before behavioral trials.
- Reproduction evaluator labels frozen before comparative evaluation:
  - `response-kestrel` = `arm-cobalt`
  - `response-oriole` = `arm-amber`
- Protected-trial evaluator labels:
  - adjacent: `response-lyra` = `arm-amber`; `response-mesa` = `arm-cobalt`
  - verdict-only: `response-lyra` = `arm-cobalt`; `response-mesa` = `arm-amber`
  - authorized process: `response-lyra` = `arm-amber`; `response-mesa` = `arm-cobalt`
  - publication safety: `response-lyra` = `arm-cobalt`; `response-mesa` = `arm-amber`
