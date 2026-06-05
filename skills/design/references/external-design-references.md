# External Design References

Use these sources as inspiration and critique lenses. Do not vendor their prose,
copy no-license payloads, or treat third-party examples as repo-owned design
facts. When a reference informs `DESIGN.md`, record the fact in
`design-contract.md` with provenance and `keep` / `change` / `do-not-copy`.

## Reference Inventory

| Reference | Use For | Boundary |
|---|---|---|
| Anthropic `frontend-design` skill | Distinctive frontend direction, strong trigger phrasing, bold aesthetic reads | Do not import its default aesthetic or assume every visual surface is a marketing page. |
| Jakub Krehel `make-interfaces-feel-better` | Micro-polish: text wrapping, tabular numbers, optical alignment, radii, hit areas, animation specificity | Use as a craft checklist; do not force motion-heavy polish where the surface should be quiet. |
| Leon `taste-skill` | Design read, VARIANCE / MOTION / DENSITY dials, anti-default discipline | Treat dials as local judgment, not a global persona or framework. |
| Rams design review | PR-style design findings with impact and concrete fix suggestions | Useful output shape; do not add a hard subjective CI score. |
| Hallmark | Macrostructure variance and anti-template audits | Extract structural lessons; do not repeat, copy, or brand-match its page catalog. |
| Public `DESIGN.md` spec/library practice | Root markdown design-system contract for AI agents: tokens plus rationale | Use the format pattern; repo facts must be observed, provided, or explicitly inferred. |

## When To Load

Load this reference when:

- creating or updating `DESIGN.md`;
- using a screenshot, URL, competitor, catalog, or external skill as inspiration;
- reviewing whether a surface feels generic, template-derived, or AI-made;
- improving trigger text or evals for visual work.

## Hard Rules

- Existing `DESIGN.md` is a repo contract. Read it before visible changes and
  maintain it when durable design facts change.
- New `DESIGN.md` is required for recurring or product-facing visual work unless
  the completion gate records a one-off/internal/no-durable-fact waiver.
- `design-contract.md` owns provenance. Every load-bearing design fact says
  whether it is `observed`, `provided`, or `inferred`.
- Third-party reference brands, screenshots, and external skill examples default
  to `do-not-copy` unless ownership or license permission is explicit.
- External sources may sharpen taste; rendered artifact evidence still decides
  whether the local surface works.

## Anti-Patterns

- Copying an external skill into the repo to make it "available".
- Creating a token engine, parser, daemon, or design-system package from a
  one-off visual task.
- Letting a public library DESIGN.md overwrite product-owned facts.
- Recording "premium", "modern", or "clean" as design facts without source,
  evidence, and concrete visual implications.
