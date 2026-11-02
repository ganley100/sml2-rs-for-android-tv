# Domain Docs

How engineering skills should consume this repository's domain documentation when exploring the codebase.

## Before exploring, read these

- **`GLOSSARY.md`** at the repository root, or **`GLOSSARY-MAP.md`** if it exists; in the latter case, read the glossary for each relevant context.
- **`docs/adr/`**: read decisions relevant to the area being changed.

If these files or directories do not exist, proceed without flagging their absence. Create them only when domain terms or decisions actually need to be resolved.

## Use the glossary's vocabulary

Use domain terms as defined in the glossary when naming concepts in issues, proposals, refactors, and tests. If a needed concept is missing, reconsider whether the term is appropriate or identify the vocabulary gap.

## Flag ADR conflicts

If a proposal or change contradicts an existing ADR, surface the conflict explicitly rather than silently overriding the decision.

## Layout

This is a single-context repository. Domain docs belong at the root (`GLOSSARY.md`) and system decisions belong in `docs/adr/`.
