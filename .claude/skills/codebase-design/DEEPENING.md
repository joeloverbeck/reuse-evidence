# Deepening

How to deepen a cluster of shallow modules safely, given its dependencies. Assumes the vocabulary in [SKILL.md](SKILL.md) — **module**, **interface**, **seam**, **adapter**.

## Dependency categories

When assessing a candidate for deepening, classify its dependencies. The category determines how the deepened module is tested across its seam.

### 1. In-process

Pure computation, in-memory state, no I/O. Always deepenable — merge the modules and test through the new interface directly. No adapter needed.

### 2. Local-substitutable

Dependencies that have local test stand-ins (PGLite for Postgres, in-memory filesystem). Deepenable if the stand-in exists. The deepened module is tested with the stand-in running in the test suite. The seam is internal; no port at the module's external interface.

### 3. Remote but owned (Ports & Adapters)

Your own services across a network boundary (microservices, internal APIs). Define a **port** (interface) at the seam. The deep module owns the logic; the transport is injected as an **adapter**. Tests use an in-memory adapter. Production uses an HTTP/gRPC/queue adapter.

Recommendation shape: *"Define a port at the seam, implement an HTTP adapter for production and an in-memory adapter for testing, so the logic sits in one deep module even though it's deployed across a network."*

### 4. True external (Mock)

Third-party services (Stripe, Twilio, etc.) you don't control. The deepened module takes the external dependency as an injected port; tests provide a mock adapter.

## Seam discipline

Apply the canonical seam rules in [SKILL.md §Principles](SKILL.md#principles) to the dependency categories above. For remote-owned and true-external dependencies, the production transport or provider adapter and the test adapter are the concrete variants that justify the port. Keep test plumbing behind an internal seam rather than adding it to the module's external interface.

## Testing strategy: replace, don't layer

Apply the canonical behavioural-test-surface and warrant rules in [SKILL.md §Principles](SKILL.md#principles). During migration:

- Delete an old shallow-module test only after a test at the deepened module's interface covers the same observable behaviour or evidence obligation.
- Tests assert on observable outcomes through the interface, not internal state.
- Tests should survive internal refactors because they describe behaviour, not implementation. Retain distinct oracle, fixture, metamorphic, and internal-invariant checks when the canonical warrant rule requires them.
