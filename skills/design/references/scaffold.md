# Design Scaffold Template

Template for `/design scaffold`. Generate a project-local design skill only
when the repo has recurring UI work.

Do not scaffold for a one-off surface unless the user explicitly asks. As a
default threshold, require at least three UI-relevant files or one recurring
surface with clear future maintenance risk.

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

- Artifact/render inspected:
- Intent:
- Design-system facts used:
- Changes made or recommended:
- A11y/QA evidence:
- Residual design risk:

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
- [ ] Golden examples point at existing artifacts.
- [ ] The skill names how to capture rendered evidence.
- [ ] The skill rejects inappropriate aesthetic defaults for this product.
- [ ] No framework, component library, or token engine was added.
- [ ] No no-license external skill prose was copied into the repo.
