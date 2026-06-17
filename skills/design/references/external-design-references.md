# External Design References

Use these sources as inspiration and critique lenses. Do not vendor their prose,
copy no-license payloads, or treat third-party examples as repo-owned design
facts. When a reference informs `DESIGN.md`, record the fact in
`design-contract.md` with provenance and `keep` / `change` / `do-not-copy`.

## Reference Inventory

Rows marked **installed** are vendored via `registry.yaml` into
`skills/.external/` and loadable by alias; run them, do not paraphrase them.

| Reference | Use For | Boundary |
|---|---|---|
| `anthropic-frontend-design` (installed) | Distinctive frontend direction: ground in the subject's world, brainstorm→critique→build passes, one signature element, calibration against the three known AI default looks | Do not import its default aesthetic or assume every visual surface is a marketing page. |
| `nutlope-hallmark` (installed) | Greenfield builds (macrostructure-first, 20-theme rotation, 58-gate slop test, pre-emit self-critique), `audit`, `redesign`, and `study` (DNA extraction from references with a no-clone gate) | MIT. Its `.hallmark/log.json` + stamp memory is per-project; respect it. Do not brand-match its example catalog. |
| `jakub-make-interfaces-feel-better` (installed) | Micro-polish: text wrapping, tabular numbers, optical alignment, concentric radii, hit areas, animation specificity | Use as a craft checklist; do not force motion-heavy polish where the surface should be quiet. |
| `emil-emil-design-eng` (installed) | Motion judgment: should-it-animate frequency table, easing decision tree, duration tables, spring vs CSS, perceived performance | Animation-frequency rules trump decorative ambition; never animate keyboard-initiated actions. |
| `leon-taste-skill` (installed, + leon-* variants) | Design read, VARIANCE / MOTION / DENSITY dials, anti-default bias correction, banned-pattern lists | React/Tailwind-specific rules don't transfer to zero-build surfaces; treat dials as local judgment, not a persona. |
| `vercel-web-design-guidelines` (installed) | Web interface guideline compliance review (a11y, focus, semantics) | Compliance pass, not an aesthetic direction. |
| Rams design review | PR-style design findings with impact and concrete fix suggestions | Useful output shape; do not add a hard subjective CI score. |
| `impeccable-impeccable` (installed) | Desloppification: 46-pattern deterministic detector (`npx impeccable detect`), 23 discipline commands (typeset, colorize, animate, layout, distill, harden, etc.), PRODUCT.md + DESIGN.md context system, brand vs product register | Apache 2.0. The skill's PRODUCT.md/DESIGN.md are per-project; don't let them shadow repo-owned `DESIGN.md`. Run the detector as a completion-gate check, not a replacement for visual inspection. |
| Public `DESIGN.md` spec/library practice | Root markdown design-system contract for AI agents: tokens plus rationale | Use the format pattern; repo facts must be observed, provided, or explicitly inferred. |

## Reference-Driven Work (studied DNA)

When the user supplies reference sites, screenshots, or named brands as
inspiration, inspiration is a **DNA transfer, not a costume change**. The
failure mode this section exists to prevent: shipping a surface a viewer
could trace back to one reference ("that's site X in a different font").
The full protocol is `nutlope-hallmark`'s `study` verb; these are the
harness-level invariants:

- **Extract DNA, never the dress.** DNA = macrostructure, component
  archetypes, type-pairing *roles* (not faces), colour-anchor *band* (not
  hex), accent footprint, density/rhythm. The dress — specific typefaces,
  specific hex values, signature imagery, named layout gimmicks — must NOT
  carry over.
- **One primary donor per surface.** With N references, first synthesize the
  shared genes (what the user's taste actually is), then pick at most one
  primary structural donor per surface. Blending five references per page
  produces template soup; cloning one produces a knockoff.
- **Name the transformation distance.** Before building, state in one line
  what was taken (e.g. "annotation-as-ornament, serif/mono pairing") and
  what was deliberately changed (different macrostructure, different hue
  family, different signature element). If you cannot name a change on at
  least two of {structure, palette, type, signature}, you are cloning.
- **Signature elements are never transferable.** A reference's single most
  recognizable move is its identity. Learn the *category* (one-ink
  discipline, generative texture, annotated figure), invent a different
  *instance*.
- **Refusal boundaries carry over.** Paid templates and marketplace listings
  are not studied; famous designers' signature work is soft-refused (DNA
  only, with the source named).

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
