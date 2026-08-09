# ADR 0007: Language-neutral evidence, Rust-first delivery

**Status:** Accepted  
**Date:** 2026-08-09  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md)

## Context

The initial portfolio is predominantly Rust, but it also includes a substantial TypeScript/JavaScript workspace. Cross-language repetition may indicate a shared schema, protocol, generator, or misplaced authority rather than a common runtime library.

A Rust-only domain model would make repository identity, occurrences, decisions, and verification depend on Cargo concepts that are not intrinsic to reuse evidence. A polyglot implementation would add unnecessary tooling complexity before the lifecycle is proven.

## Decision

The evidence and decision model is language-neutral; delivery is Rust-first.

- Repository, consumer, occurrence, evidence, case, decision, privacy, and verification semantics contain no Cargo-only assumption.
- The published CLI and core library are implemented in Rust.
- Version 0.1 provides strongest operational support for Rust/Cargo repositories.
- TypeScript/JavaScript npm workspaces must still be enrollable, discoverable, referenceable, and capable of clean capture, case participation, decision, and verification through agent-guided source inspection.
- Cross-language cases are valid.
- Review chooses among shared code, schema, specification, generator, protocol, existing package, or separated authority according to evidence.
- The core does not add an AST parser merely to claim language support.

## Consequences

### Positive

- The domain model fits the actual portfolio.
- Rust remains the implementation and distribution focus.
- TypeScript participation tests whether the abstraction is genuinely language-neutral.
- Cross-language reuse is not forced into a runtime-library answer.

### Negative and risks

- TypeScript discovery may initially depend more on agent reasoning and optional sensors.
- Language-specific package research differs by ecosystem.
- Users may mistake language-neutral evidence for equal first-class tooling in every language.

The documentation must distinguish semantic support from parser or IDE integration.

### Operational burden

Do not create a language-adapter framework. Add structured language-specific fields only when a real consumer or verifier uses them.

### Compatibility and migration

Recorded evidence should use repository-relative paths, symbols, artifact kinds, and open metadata that can represent multiple languages. Cargo package identity may be an optional Rust-specific detail, not a required case field.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Rust-only evidence model | Rejected | Excludes a real active repository and overfits syntax to the domain. |
| Implement the tool in TypeScript | Rejected | Rust is the selected publication and primary portfolio language. |
| Build separate Rust and TypeScript engines | Rejected | Duplicates the lifecycle before any need. |
| Promise equal parser-level support for many languages | Rejected | Unsupported breadth and likely detector scope creep. |

## Verification and review trigger

Version 0.1 must complete at least one clean TypeScript/JavaScript capture and prove that no Cargo assumption is required in authoritative state. Reopen if real cross-language cases require a domain distinction the current model cannot express without distortion.

## Supersession

None.
