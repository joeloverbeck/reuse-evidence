# Decision and publication mechanics

Load this reference only after the eleven-question analysis and any required package research are
complete. It defines the existing proposal shape and the exact approval transaction; it authorizes
no new field, command, path, or compatibility promise.

## Decision vocabulary

`identity_verdict` is exactly one of:

- `same_responsibility`;
- `different_responsibilities`;
- `insufficient_evidence`; or
- `existing_abstraction_is_wrong`.

`action` is exactly one of:

- `retain_intentional_duplication`;
- `wait_for_more_evidence`;
- `use_existing_dependency`;
- `extract_or_deepen_locally`;
- `create_workspace_package`;
- `create_private_cross_repository_package`;
- `publish_public_package`;
- `centralize_schema_specification_or_fixture_corpus`;
- `replace_copies_with_generated_artifacts`;
- `contribute_missing_behavior_upstream`; or
- `split_inline_or_narrow_existing_abstraction`.

The verdict and action are orthogonal. Choose the pair the analysis supports rather than inferring
one from the other.

## Required content

Every decision contains these eight fields with non-empty content:

1. `identity_verdict`;
2. `action`;
3. `accepted_scope`;
4. `non_responsibilities`;
5. `affected_consumers`;
6. `alternatives_rejected`;
7. `compatibility_consequences`; and
8. `verification_conditions`.

Every action except `retain_intentional_duplication` and `wait_for_more_evidence` also contains all
five implementation-authorizing fields:

1. `invariant_contract`;
2. `existing_packages_considered`;
3. `required_consumer_level_tests`;
4. `migration_expectations`; and
5. `rollback_or_resplitting_path`.

The two no-implementation actions omit all five. They still require the eight common fields. An
affected consumer is a repository-and-consumer pair already recorded by an occurrence. Its
`expectation` states the accepted consequence for that consumer. Migration `order` values state the
accepted sequence. Verification conditions name observable evidence, not implementation steps.

## Implementation-authorizing proposal shape

```toml
identity_verdict = "<accepted-verdict>"
action = "<implementation-authorizing-action>"
accepted_scope = "<narrowest accepted scope>"
non_responsibilities = ["<explicitly excluded responsibility>"]
compatibility_consequences = "<compatibility and release consequence>"
verification_conditions = ["<observable condition later evidence must answer>"]
invariant_contract = "<exact invariant behavior or contract>"
required_consumer_level_tests = ["<consumer-facing behavior to preserve>"]
rollback_or_resplitting_path = "<reversible rollback or re-splitting route>"

[[affected_consumers]]
repository_id = "<recorded-repository-id>"
consumer = "<recorded-consumer>"
expectation = "<accepted consequence for this consumer>"

[[alternatives_rejected]]
alternative = "<examined alternative>"
reason = "<evidence-bearing reason it lost>"

[[existing_packages_considered]]
package = "<examined package>"
fit = "<functional and boundary fit>"
reason = "<adopted or rejected reason>"

[[migration_expectations]]
order = 1
expectation = "<first accepted migration consequence>"
```

## No-implementation proposal shape

```toml
identity_verdict = "<accepted-verdict>"
action = "<retain_intentional_duplication-or-wait_for_more_evidence>"
accepted_scope = "<scope of the intentional locality or deferral>"
non_responsibilities = ["<claim this decision does not make>"]
compatibility_consequences = "<consequence of retaining or waiting>"
verification_conditions = ["<observable condition for later verification or reopening>"]

[[affected_consumers]]
repository_id = "<recorded-repository-id>"
consumer = "<recorded-consumer>"
expectation = "<accepted local or deferred consequence>"

[[alternatives_rejected]]
alternative = "<examined alternative or researched package>"
reason = "<why the evidence does not support it>"
```

Do not include `invariant_contract`, `existing_packages_considered`,
`required_consumer_level_tests`, `migration_expectations`, or
`rollback_or_resplitting_path` in this shape.

## Exact preview and approval

From the steward repository, against the freshly re-read revision and the same selected roots:

```console
reuse-evidence case decide <case-id> --expected-revision <revision> \
  --proposal <staged-draft> [--root <portfolio-root> ...] --preview
```

The preview receipt ends with a line containing only `event:` followed by the exact TOML bytes the
command proposes to record. Preserve every byte after that line through end of output, including the
final newline, in a separate file beneath the compiled staging directory. Do not reconstruct,
normalize, parse, or serialize it.

Show the human the complete receipt and ask for authorization of:

1. the exact event bytes;
2. the named steward event path, revision, state, privacy, and implementation consequence; and
3. removal of the staged draft and event after publication is byte-verified.

A content change invalidates the preview. Generate a replacement receipt and saved event before
asking again. A refusal is terminal for that candidate: relay it verbatim and preserve staging. A
decline publishes nothing and leaves no proposal from this run in durable staging.

## One publication and byte verification

After approval, repeat the same command once without `--preview`, replacing `<staged-draft>` with
the file containing the approved event bytes. Keep the same case identity, expected revision,
selected roots, and steward working directory.

Accept only a created receipt or, when resuming an interrupted application, an idempotent-existing
receipt. Read the event path named in the receipt and compare its bytes directly with the approved
staged file. Only exact equality proves publication. Do not retry a successful mutation.

After exact equality, remove the draft and approved event. On refusal, unsafe failure, ambiguous
outcome, missing event, or byte mismatch, keep the approved bytes and report that publication is not
verified. Never edit a recorded event to repair a mismatch.

Finally run the read-only projection from the steward with the same roots:

```console
reuse-evidence case brief <case-id> [--root <portfolio-root> ...]
```

Present that output unchanged as the handoff. It is the implementation brief; author no companion
document and perform no implementation.
