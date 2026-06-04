# Design Scaffold Template

Template for `/design scaffold`. Generate a project-local design skill only
when the repo has recurring UI work.

Do not scaffold for a one-off surface unless the user explicitly asks. As a
default threshold, require at least three UI-relevant files or one recurring
surface with clear future maintenance risk.

When recurring UI work earns a repo-owned design contract, generate source
documents the repo can inspect and edit:

- `DESIGN.md` - the stable product design contract.
- `design-contract.md` - the evidence table that explains where each design
  fact came from and how it should be used.

Do not copy external design-system catalog prose. Link to reference material
and extract only facts that are observed in the repo, provided by the user, or
clearly labeled as inferred.

## Investigation Prompts

Launch independent investigators when the repo is non-trivial.

### Surface Mapper

> Map the product's visible surfaces.
>
> Find routes, screens, documents, reports, dashboards, components, story files,
> screenshots, demos, and generated static outputs. Return a table:
>
> | Surface | Source path | Audience | Frequency | Risk |

### System Mapper

> Map existing design-system facts.
>
> Find tokens, Tailwind config, CSS variables, component libraries, typography,
> icons, charting, animation libraries, Storybook, visual tests, and a11y tools.
> Return facts only. Do not propose changes.

### Taste Mapper

> Map the product's intended feel and anti-patterns.
>
> Read README, docs, product copy, screenshots, and issue/backlog language.
> Identify the desired aesthetic, what would be off-brand, and the surfaces that
> should be treated as golden examples.

## Repo Design Contract

Emit `DESIGN.md` only when the threshold above is met. Use this nine-section
shape so future agents can find product-specific design facts without confusing
them for universal taste advice:

1. Product Intent
2. Audience and Context
3. Brand Attributes
4. Visual Language
5. Layout and Density
6. Components and Interaction
7. Content Voice
8. Accessibility and Responsiveness
9. Evidence and Governance

Each section names repo-owned facts, not generic advice. If a fact is inferred
from weak evidence, label it as inferred and lower confidence. If the repo does
not yet have enough evidence for a section, write the gap instead of inventing
the answer.

## Evidence Contract

Generate `design-contract.md` beside `DESIGN.md` when deriving a design
contract from existing screens, screenshots, sites, docs, brand references, or
user-provided examples.

Use this table:

| Source | Fact | Provenance | Confidence | Use | Evidence / Notes |
|---|---|---|---|---|---|
| `path-or-url` | Design fact to carry forward | `observed` / `provided` / `inferred` | high / medium / low | `keep` / `change` / `do-not-copy` | Artifact, quote, screenshot region, or caveat |

Provenance labels:

- `observed` - visible in a rendered artifact, screenshot, code, token file, or
  committed product copy.
- `provided` - explicitly supplied by the user, stakeholder, brand guide, or
  repo documentation.
- `inferred` - a reasoned conclusion from weaker evidence. Always include the
  source and confidence.

Use labels:

- `keep` - preserve this product-owned fact or pattern.
- `change` - the source reveals a direction or defect, but the implementation
  should intentionally differ.
- `do-not-copy` - reference-brand material, third-party style, unlicensed
  payload, competitor UI, or artifact-specific detail that must not be cloned.

Reference-brand material needs `do-not-copy` guidance unless the user explicitly
owns it or provides a license-safe source. Never let an attractive external
example silently become the repo's brand.

## Generated Skill Shape

Write the project-local skill under the repo's shared skill root, then bridge
per harness according to the repo's install contract.

```markdown
---
name: design
description: |
  Design critique and polish for [project]. Use when UI, docs, reports,
  dashboards, visual hierarchy, typography, or interaction feel changes.
  Trigger: /design.
disable-model-invocation: true
argument-hint: "[audit|polish|redesign] <surface>"
---

# /design

[One sentence describing what good design means in this project.]

## Product Feel

- Audience:
- Desired feel:
- Avoid:

## Surfaces

| Surface | Source | What Good Looks Like |
|---|---|---|

## Design System

- Tokens:
- Type scale:
- Component library:
- Icons/media:
- Motion:
- A11y tooling:

## Golden Examples

- [surface/path]: [why it is trusted]

## Evidence

Design evidence goes to `/tmp/design-[project-slug]/` unless the repo has a
stronger local convention.

## Completion Gate

- Intent: target user outcome and visual direction for this surface.
- Evidence that proves it: screenshot, render, artifact, or visual diff inspected.
- Exact command/path/route exercised: URL, screenshot path, render command, or artifact path inspected.
- Design-system facts used: local tokens, components, or documented conventions applied.
- Changes made or recommended: concrete UI changes or critique recommendations.
- Residual risk: remaining design, a11y, or QA risk after inspection.

## Red Lines

- [repo-specific things agents must not change]
```

`disable-model-invocation` is honored by Claude Code. Other harnesses may
ignore it, so keep project-local trigger text specific enough that the
filesystem skill still routes intentionally.

## Quality Gate

Before declaring the scaffold complete:

- [ ] Surfaces are real paths/routes, not placeholders.
- [ ] Design-system facts are discovered from the repo, not invented.
- [ ] `DESIGN.md` uses the nine-section contract only when recurring UI work
      earns it.
- [ ] `design-contract.md` records source, fact, provenance, confidence, and
      `keep` / `change` / `do-not-copy` use for every load-bearing design fact.
- [ ] Reference-brand material is marked `do-not-copy` unless ownership or
      license permission is explicit.
- [ ] Golden examples point at existing artifacts.
- [ ] The skill names how to capture rendered evidence.
- [ ] The skill rejects inappropriate aesthetic defaults for this product.
- [ ] No framework, component library, or token engine was added.
- [ ] No Open Design catalog payload, `od:` frontmatter parser, daemon, app, or
      adapter layer was added.
- [ ] No no-license external skill prose was copied into the repo.
