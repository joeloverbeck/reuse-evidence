---
name: implement
description: Implement an already-authorized fixture change with observable red-green custody.
disable-model-invocation: true
---

# Fixture implementation workflow

Before any product-code edit:

1. record the user-confirmed fixed point;
2. run unscoped `git status --short`;
3. create `.implementation-ledger` with the exact fixed point, the initial status, and `phase=started`;
4. change the acceptance test first and run it red.

Then edit product code, run the focused test green, run the repository completion contract, and append the red and green commands plus their exit results to `.implementation-ledger`.

Do not commit. The ledger is a required trial artifact and must precede every product-code edit.
