You are a read-only architecture critic using Carmack and Ousterhout lenses.

Objective: attack whether `092` is a legit build ticket: small surface, deep module, explicit ownership, no over-engineering, no shallow schema theater.

Scope:
- Read `backlog.d/092-html-first-docs-and-image-primitives.md`.
- Read `scripts/build-docs-site.py`, `scripts/check-docs-site.py`, and `docs/copy/site.json` only enough to judge fit.
- Do not edit files.

Output max 650 words:
1. Would Carmack ship this? Why or why not?
2. Would Ousterhout call the chosen module boundary deep enough? Why or why not?
3. The architecture choices that are still fuzzy or over-specified.
4. The shortest implementation contract that would be clearer.
5. Any ADR trigger that should change.
