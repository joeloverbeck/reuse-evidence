# Architecture review

Disposition these candidates in order. Apply only candidates supported by the repository authority and source.

## 1. One renderer for every command receipt

Move all receipt field selection and ordering into `src/render.rs`.

## 2. Rewrite recorded event identifiers during startup

Normalize old identifier spellings in place before any command runs.

## 3. Add a hosted command registry

Publish command metadata to a central service and load it at runtime.

## 4. Merge command policy and process status

Replace each command's receipt and exit mapping with one global status enum.

## 5. Give the shared success terminal meaning one owner

`src/commands.rs` repeats the same success-status mapping in two independently edited commands. Add the narrowest internal owner for that mapping, update both commands to delegate to it, and record the structural decision without replacing ADR 0001. Name the command-layer concept in the glossary if one crystallizes.

The review is authorized to edit only `CONTEXT.md`, `docs/adr/0002-terminal-meaning-owner.md`, `docs/adr/README.md`, `src/commands.rs`, and a new `src/terminal.rs` for a surviving candidate.
