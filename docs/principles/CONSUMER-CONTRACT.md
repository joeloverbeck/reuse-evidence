# Consumer Contract

**Status:** Adopted topic principle  
**Governed by:** [`FOUNDATIONS.md`](FOUNDATIONS.md)

This contract defines what an enrolled repository and maintainer should be able to rely on as `reuse-evidence` evolves. Exact command names, Rust modules, schemas, and file layouts remain design concerns until implemented and published.

## 1. Contract surfaces

The project is expected to expose several independently versioned surfaces:

- the published Rust crate and standalone binary;
- the CLI behavior and terminal meanings;
- the repository marker and local portfolio configuration;
- installed agent-skill assets;
- authoritative recorded case evidence;
- derived status and cache representations.

Cargo SemVer protects only public Rust items. It does not automatically protect command prose, installed Markdown assets, configuration, or recorded history. Those surfaces require explicit compatibility discipline.

## 2. Behavioral guarantees

An adopting repository may rely on these semantic guarantees:

### Explicit opt-in

Unmarked repositories are not portfolio participants merely because they are discoverable on disk.

### Local-first operation

The core lifecycle does not require a hosted account, telemetry, remote model, or external API.

### Safe refusal

When authority, evidence, expected revision, privacy, or schema validity is insufficient, the operation refuses without partial case mutation. Refusal is the system working safely, not an implementation failure to bypass.

### No automatic semantic decision

A detector score, occurrence count, review-ready state, test result, or generated report cannot accept a responsibility identity or extraction.

### No automatic refactoring

The tool does not modify consumer code merely because a case exists or a decision is proposed. Implementation requires the repository's ordinary authorized engineering workflow.

### Write-free clean capture

A completed no-candidate capture does not create durable evidence or certification paperwork.

### Inspectable history

Accepted case facts and decisions remain inspectable. Historical events are not silently rewritten to match the current interpretation.

### Rebuildable derivations

Indexes, readiness projections, and status views are derived and can be rebuilt from authoritative repository state.

### Private dominance

A public operation cannot silently absorb private evidence. Mixed-visibility cases remain private.

## 3. Recorded-evidence compatibility

Recorded evidence is the hardest compatibility surface because pinning an older binary cannot undo history already written.

Therefore:

- incompatible event-shape changes require a new explicit schema version;
- readers must continue to understand supported historical versions or perform a separately authorized, reversible migration;
- migrations must preserve original provenance and emit inspectable receipts;
- no migration may reinterpret a human decision silently;
- destructive history rewriting is forbidden;
- and derived projections must declare which event versions they consumed.

Additive optional fields are preferable to premature universal schemas. Structure only distinctions that real decisions, privacy, recovery, or consumers require.

## 4. Installed asset compatibility

Agent skills are part of the operational product, not incidental documentation. The installer must not silently overwrite locally modified assets.

An installation or upgrade should:

- identify every targeted path;
- compare installed content with shipped content;
- refuse atomically on conflict unless the human explicitly authorizes replacement;
- preserve the `.claude/skills/` real-file and `.agents/skills/` link convention;
- and report exact consequences.

A future removal or rename of an installed package must define how stale assets are detected rather than strand silent, conflicting instructions.

## 5. Marker and identity compatibility

Repository identity must survive path moves and renames. Marker schema changes require explicit version handling.

A repository must not be enrolled, made public, or assigned a new identity implicitly during an unrelated command.

## 6. Consumer obligations

A repository adopting the tool is responsible for:

- declaring truthful visibility;
- preserving its stable repository identity;
- reviewing cross-repository and public-disclosure consequences;
- not hand-editing authoritative case history once a compiled writer exists;
- maintaining recoverable evidence references needed by open decisions;
- implementing accepted changes through its own engineering authority;
- and verifying consumer behavior rather than treating structural migration as success.

## 7. Non-guarantees

The project does not guarantee:

- discovery of every reusable responsibility;
- absence of false-positive candidates;
- a correct abstraction without human judgment;
- improved architecture merely because cases are closed;
- compatibility with every detector or agent framework;
- hosted synchronization;
- support response times, backports, or deprecation windows;
- or a public ecosystem merely because the crate is published.

## 8. Version 0.x policy

During `0.x`, the command and asset surface may change as real portfolio use reveals the right boundary. Breaking changes take an appropriate version bump and must respect recorded-evidence and privacy guarantees.

`1.0.0` should mean that the public compatibility surfaces are understood and intentionally stabilized, not merely that a feature milestone was reached.

## 9. No lock-in

Evidence and accepted decisions must remain inspectable in repository files or documented open formats. A maintainer must be able to stop using the tool without losing the history needed to understand prior reuse decisions.

The project may provide convenience indexes and renderers, but it must not make a private opaque database the only route to authoritative state.
