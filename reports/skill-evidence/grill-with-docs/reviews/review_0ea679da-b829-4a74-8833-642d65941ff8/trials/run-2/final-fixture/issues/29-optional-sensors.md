# Keep sensors optional and outside semantic authority

## Consumer effect

A maintainer may use an available sensor to improve candidate discovery without making capture depend on one provider or allowing a score to decide responsibility identity, readiness, or action.

The live bottleneck is the evidence lifecycle, not detector development. ADR 0005 records that ordinary Git, source inspection, tests, history, and agent reasoning are a valid path and that no common adapter has yet earned authority.

## Authority

- `docs/principles/FOUNDATIONS.md` §§8 and 13 limit sensor output to claim-sized discovery evidence.
- `docs/principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md` §6 makes sensors optional and denies them case-opening, independence, decision, CI, and disclosure authority.
- `docs/adr/0005-optional-sensors-no-built-in-detector.md` rejects a built-in detector, a mandatory detector, and a generalized plugin framework for version 0.1.
- `docs/design/v0.1-scope-and-acceptance.md` §§2 and 3 permits optional external sensors while excluding built-in detection, required providers, remote upload, and CI failure.

## Narrow scope

- Let capture or discovery consume a bounded, recoverable reference to optional sensor output when one is already available and appropriate.
- Label sensor findings as candidate evidence requiring independent semantic verification.
- Preserve a complete path that uses only repository evidence and agent reasoning.

## Out of scope

- A built-in exact, structural, fuzzy, semantic, or cross-project detector.
- A mandatory provider, model, embedding store, GPU, network API, or remote upload.
- A generalized sensor plugin framework or compatibility promise.
- CI failure, automatic case creation, readiness, decision, or refactoring based on sensor output.

## Acceptance

- Capture and discovery remain usable with no sensor installed or configured.
- Any sensor-assisted result identifies the sensor output as candidate evidence and performs semantic checks before proposing a case action.
- No sensor result alone changes durable state or process exit as a policy enforcement mechanism.
- Park or falsify this issue if repeated real cases fail specifically because external sensors and bounded analysis cannot find candidates; use ADR 0005's review trigger before expanding detector ownership.
