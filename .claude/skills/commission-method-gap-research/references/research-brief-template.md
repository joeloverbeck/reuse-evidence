# Research Brief Template

## Contents

- [Instantiate the brief](#instantiate-the-brief)
- [Inline evidence contract](#inline-evidence-contract)
- [Evidence-bundle contract](#evidence-bundle-contract)
- [Validate the commission](#validate-the-commission)

## Instantiate the brief

Instantiate every section. Replace brackets with verified facts or an explicit `unknown - owner
decision required`. Remove instructional comments before delivery. Complete every commission
validation checkbox only after verifying it.

```markdown
# Research Brief — Method-Gap Audit of [SKILL NAME]

Brief ID: [stable identifier]

## 0. Commission

- Audited skill: [name, requested path, resolved/real path, version or unversioned]
- Repository scope: [resolved root and root-detection basis]
- Source identity: [content hash, commit/blob, worktree state]
- Evidence-packet identity: [immutable ref and verified path set / sibling bundle manifest + hashes / inline]
- Decision owner: [owner]
- Commissioner: [person or skill]
- Research executor: [executor or mechanism]
- Audit date: [date]
- Prior audit: [date/version/brief ID or never]
- In-flight or unconsumed work: [none or exact state]
- Internal-source access: [verified direct / complete commission artifact set / fully inlined]
- External-source access: [allowed sources, currency date, restrictions]
- Privacy and sanitization: [rules]

Recommendation: **commission now**.

This is a bounded, evidence-backed audit of missing method capabilities. It does not authorize
edits to the audited skill or adoption of recommendations.

## 1. Objective

Identify materially consequential instrument classes, evidence bridges, controls, or decision
machinery that [SKILL NAME] needs but does not adequately possess. Compare capabilities rather
than terminology. Complement the skill's internal improvement loop; do not replace it.

This audit cannot establish that no further gaps exist.

## 2. Intended outcomes and internal improvement loop

- Intended outcomes: [list]
- Material claims: [list]
- Normal use -> evidence -> adjudication -> revision loop: [map]
- Authority and escalation: [map]

## 3. Operating envelope

- Maintainers and roles: [facts]
- One-time and recurring time: [limits]
- Compute, tools, and infrastructure: [limits]
- Users, participants, and experts: [access]
- Privacy and publication: [limits]
- Maximum recurring ceremony: [limit]
- Other non-negotiable constraints: [list]

Adapt every recommendation to this envelope. Do not import institutional frameworks whole.

## 4. Evidence packet

### Commission artifact set

- Brief: [brief path or inline]
- Evidence bundle: [none or sibling <brief-stem>.evidence directory]
- Bundle manifest: [none or <brief-stem>.evidence/manifest.json]

### Current skill and supporting machinery

- [accessible source or inlined evidence + why load-bearing]

### Empirical history

- [accessible executions, audits, friction, tests, exceptions, carried gaps]

### Prior research and adjudication

- [accessible briefs, reports, dispositions, witnesses, adopted landings]

## 5. Adopted-lineage fence

Treat these as established capabilities. Reopen one only with credible evidence of capability
failure, instrument conflict, or a new operating condition.

| Research line  | Adopted capability | Current location | Version/date | Remaining limit |
| -------------- | ------------------ | ---------------- | ------------ | --------------- |
| [item or none] | [capability]       | [location]       | [version]    | [limit]         |

## 6. Settled-negative and run-scope fences

| Area           | Permanent or this run | Reason   | Reopen trigger or none |
| -------------- | --------------------- | -------- | ---------------------- |
| [item or none] | [status]              | [reason] | [trigger]              |

Do not silently reopen these areas.

## 7. Candidate gap areas

These are hypotheses. Challenge each one and check for equivalent existing machinery.

1. [candidate + local interval evidence]
2. [candidate + local interval evidence]

Also perform an open-ended sweep for consequential gap classes outside this list but inside the
declared scope.

## 8. External knowledge bases

Investigate fields selected for functional relevance:

- [field -> capability or failure problem]

Add another field only when the open sweep justifies it.

## 9. Research tasks

1. Reconstruct the skill's claim-instrument-evidence map.
2. Check whether each candidate is already covered under different terminology.
3. Identify consequential detection, validation, control, decision, or transfer failures.
4. Find relevant external instrument classes, supporting evidence, and contrary evidence.
5. Adapt credible instruments to the operating envelope.
6. Specify the cheapest first witness and required artifacts for each.
7. Give every recommendation an adopt-now, defer-with-trigger, or reject disposition.
8. Perform the open-ended sweep.
9. Prioritize surviving recommendations as a coherent roadmap.

## 10. Recommendation contract

Every recommendation must state:

- **Gap:** the missing capability, independent of the solution.
- **Consequence:** what the skill cannot reliably know, distinguish, prevent, or transfer.
- **Current coverage:** why existing machinery does not already close it.
- **Prior art:** the external instrument or failure model with claim-level citations.
- **Adapted instrument:** the smallest coherent form that fits the operating envelope.
- **Assumptions and limits:** when it should work and what it still cannot establish.
- **Cost order:** one-time and recurring maintainer time, compute/tooling, participants, and
  dependencies; use orders of magnitude rather than false precision.
- **Cheapest first witness:** one trial-sized exercise, required artifacts, and observations for
  and against adoption.
- **Risks and dependencies:** ceremony, gaming, false assurance, duplication, maintenance, and new
  failure modes.
- **Disposition:** `adopt now`, `defer with a named concrete trigger`, or `reject`, with reasons.
- **Landing target:** the proposed owning location if later adopted; this is not an edit.

`Adopt now` requires materiality, feasibility, and a proportionate witness. `Defer` requires a
named trigger. `Reject` covers out-of-scope, already-covered, unsupported, disproportionate, or
incompatible proposals. “Interesting,” “future work,” and “consider” are not dispositions.

## 11. Citation rules

Provide full bibliographic citations tied to supported claims. Mark an unverified source or claim
`relayed—unverified`. Distinguish sourced fact, expert consensus, analogy, and inference. Report
material contrary evidence and domain limits. The owner will independently verify every
decision-critical citation before adoption.

## 12. Deliverable

Return one consolidated report containing:

1. scope, operating envelope, and current method map;
2. per-area findings with evidence and dispositions together;
3. the open-ended sweep;
4. already-covered and rejected candidates;
5. conflicts and dependencies between instruments;
6. a prioritized roadmap ordered by value, feasibility, dependency, and witness cost;
7. a citation-verification register, including every `relayed—unverified` claim; and
8. an appendix of candidate witnesses and expected artifacts.

Do not return an unranked best-practices catalogue.

## 13. Authority and consumption boundary

Do not edit [SKILL NAME], adopt a recommendation, or decide policy for the owner. After the
report returns, the owner will verify critical citations, adjudicate every recommendation, run
cheap witnesses where needed, route calibration or pressure-test work separately, and revise the
skill only through its normal change process. Adopted capabilities enter the next lineage fence.

## 14. Completion condition

The audit is complete when every scoped area is reported, every recommendation satisfies the
contract, the open-ended sweep and negative findings are present, and the consolidated report is
delivered—or when a scoped area explicitly records that it yielded no defensible method gap.

This audit cannot establish that no further method gaps exist.

### Exact executor handoff

[One ready-to-run instruction that names the complete artifact set, evidence access, required
deliverable, and unverified-claim marker, and that prohibits edits and adoption using the literal
words "Do not edit" and "adopt", each unbroken by a line break.]

## Commission validation

- [ ] The target skill, authority posture, version, content identity, and worktree state are clear.
- [ ] The executor can access every named internal source through the declared delivery mode.
- [ ] The commission-owned artifact set contains only this brief and, when declared, its one
      canonical sibling evidence bundle.
- [ ] Every ref-backed path exists at the named ref; every dirty or untracked source is separately
      materialized or inlined in a canonical exact-payload block with its current hash.
- [ ] Outcomes, claims, instruments, evidence, thresholds, authority, failure, and feedback routes
      are mapped far enough to compare capabilities.
- [ ] The real operating envelope binds every recommendation.
- [ ] Every adopted prior research line is fenced.
- [ ] Rejected, deferred, and deliberately excluded areas are visible with reopening rules.
- [ ] Candidate areas come from local evidence and remain hypotheses.
- [ ] The open-ended sweep is mandatory.
- [ ] External fields are selected by function.
- [ ] Every recommendation owes prior art, adaptation, costs, a cheapest first witness, risks, a
      disposition, and a landing target.
- [ ] Claim-level citation verification and `relayed—unverified` handling are explicit.
- [ ] The report must include negative findings, conflicts, dependencies, and a prioritized roadmap.
- [ ] The executor has no authority to edit, adopt, or settle policy.
- [ ] The post-report consumption route is explicit.
- [ ] The brief makes no completeness claim.
```

## Inline evidence contract

When exact source bytes are inlined, use one canonical block per source. `Path` is a provenance
label, not an access instruction. An optional `Base-ref blob` line may appear between `Path` and
`Current SHA-256`. Choose a backtick or tilde fence of at least three characters whose exact
closing line does not occur in the payload.

Use readable UTF-8 only when the source is valid UTF-8 and its line endings are LF. Record whether
the source has one terminal LF; the validator restores that byte after separating the payload from
the closing fence. Use base64 for binary data, non-UTF-8 text, CRLF bytes, a source whose own bytes
carry bracketed placeholder-shaped text (see [Validate the commission](#validate-the-commission)),
or any source that cannot be represented safely in the readable form.

Readable UTF-8 shape:

`````text
Path: references/current.md

Current SHA-256: 64-lowercase-hex-characters

Bytes: 1234

Encoding: utf8-plaintext, LF line endings.

Terminal newline: present

~~~~text
<exact UTF-8 payload without its declared terminal LF>
~~~~
`````

Base64 shape:

`````text
Path: references/current.bin

Current SHA-256: 64-lowercase-hex-characters

Encoding: base64, no line wrapping.

~~~~text
<one canonical unwrapped base64 payload>
~~~~
`````

The validator recomputes the readable payload's UTF-8 byte count and SHA-256, and decodes and
re-hashes every base64 payload. Prose plus an unchecked digest is not an exact inline payload.

## Evidence-bundle contract

Keep the brief as `<brief-stem>.md` and the optional bundle as its sibling
`<brief-stem>.evidence/`. The bundle must contain `manifest.json` plus only the files named by that
manifest. Use this schema:

```json
{
  "schemaVersion": 1,
  "briefId": "CMGR-TARGET-YYYY-MM-DD-01",
  "files": [
    {
      "path": "files/current-reference.md",
      "sourcePath": ".claude/skills/target/references/current-reference.md",
      "sha256": "64-lowercase-hex-characters",
      "bytes": 1234
    }
  ]
}
```

Every `path` is relative to the bundle, uses `/`, stays inside the bundle, and names one regular
non-symlink file. Every `sourcePath` identifies the source bytes without exposing an inaccessible
absolute local path. List each bundled file exactly once; include no unmanifested files, nested
manifests, or extra notes. Match `briefId`, `sha256`, and `bytes` to the delivered bytes. Apply the
same privacy and sanitization rules to the bundle as to the brief.

## Validate the commission

After the semantic review and before delivery, mark every applicable `Commission validation` row
complete and run:

```text
node <this-skill-directory>/scripts/validate-research-brief.mjs <brief-path> [--bundle <bundle-dir>]
```

The validator checks section order, required commission fields, the single `commission now`
decision, unresolved template placeholders, the executor handoff, checked validation rows,
authority and completeness boundaries, canonical inline base64 and readable UTF-8 payload
identities, and the optional bundle manifest plus exact file set. It does not establish that the
brief's substantive judgments are true; the manual semantic gate remains binding.

Three of those checks match surface form rather than meaning, so a semantically correct brief still
fails when a required phrase is reworded or wrapped mid-phrase. Keep each literal on one line:

- The §0 disclaimer needs the literal `does not authorize`, followed by `edit` or `edits`, followed
  by `adoption`.
- §13 and `### Exact executor handoff` each need the literal `Do not edit` plus `adopt`. An
  equivalent prohibition in other words is rejected in both places.
- The unresolved-placeholder scan reads the whole file, inlined readable UTF-8 payloads included. A
  payload whose own bytes carry a bracketed placeholder word — `candidate`, `list`, `map`, `owner`,
  `location`, `version`, and the rest of the template's bracket vocabulary — is reported as an
  unresolved placeholder. Inline that source as base64 instead.
