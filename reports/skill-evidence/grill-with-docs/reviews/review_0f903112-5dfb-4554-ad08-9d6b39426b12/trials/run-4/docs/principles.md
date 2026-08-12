# Principles

1. Public terminal meanings are part of the command contract and must have one coherent owner.
2. Recorded event bytes are immutable and are not changed by a command-layer refactor.
3. A repository-local structural decision that future contributors could unknowingly reverse belongs in an accepted ADR.
4. Hosted services, broad plugin systems, and unrelated cleanup are out of scope.
5. Prefer the narrowest module that owns one invariant without absorbing callers' distinct policy.
