# Design System Judgment

Use this reference when UI work touches recurring surfaces, shared components,
theme files, or visual rules that should remain coherent across the product.

## What Belongs In A Repo Design System

Keep the design system local to the consuming repo. Harness Kit provides the
process and checks; the product owns the visual language.

Minimum useful system:

- **Tokens:** primitive values, semantic aliases, component-level roles, and
  theme values.
- **Typography:** font families, type scale, line heights, numeric formatting,
  and when to use tabular numbers.
- **Spacing and density:** base grid, section rhythm, compact/dense modes, and
  exceptions.
- **Shape and elevation:** radii, shadows, outlines, borders, and stacking
  rules.
- **Component grammar:** approved components, variants, composition patterns,
  empty/loading/error states, and anti-patterns.
- **Motion:** duration bands, easing choices, reduced-motion behavior, and
  interaction affordances.
- **Accessibility:** contrast, focus, target size, labels, landmarks, keyboard
  paths, and screen-reader expectations.

## When A Token Layer Earns Its Cost

Add or enforce a token layer only when at least one of these is true:

- The repo has multiple recurring UI surfaces.
- The same visual decision appears in several components.
- Product identity matters and drift is visible.
- Multiple agents or humans are changing UI in parallel.
- A downstream app needs themes, white-labeling, or brand variants.

Do not add a token layer for a one-off report, static page, prototype, or
single bounded polish pass. Improve the rendered surface directly.

## Audit Questions

- Are raw colors, spacing values, shadows, or font sizes bypassing existing
  tokens?
- Do component variants cover loading, empty, error, disabled, active, hover,
  focus, and selected states?
- Are components composed consistently, or are agents inventing local grammar?
- Does the design system describe what to avoid, not only what to use?
- Can a future agent inspect one local file and know the product's visual
  direction?

## Enforcement

Start soft:

1. Route UI diffs through `/design` and `/a11y`.
2. Capture rendered evidence with `/qa`, `/browser`, or `/demo`.
3. Harden repeated findings into repo-local lint, tests, or scaffolded design
   guidance.

Escalate to CI only for deterministic checks: token bypass, missing states,
contrast, focus, invalid component imports, or forbidden raw values.
