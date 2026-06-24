# Design Scaffold Template

Template for `/design scaffold`. Generate and maintain a repo-local design
contract for recurring or product-facing visual work.

Do not scaffold for a one-off surface unless the user explicitly asks. Default
threshold: at least three visual-surface files, one recurring surface with clear
maintenance risk, or a product-facing surface whose visual language future
agents will change.

When recurring UI work earns a repo-owned contract, generate source documents
the repo can inspect, edit, and **mechanically enforce**:

- `DESIGN.md` — the stable visual contract, in the `@google/design.md` format
  (machine-readable token frontmatter + human-readable rationale).
- `design-contract.md` — the evidence table for where each design fact came
  from and how it should be used.

If `DESIGN.md` already exists, read it before visual work and update it in the
same change when durable facts change (tokens, type, spacing/radii, motion,
component grammar, density, a11y rules, golden examples, anti-patterns). If the
change is visual but intentionally does not update `DESIGN.md`, record a
one-off/internal/no-durable-fact waiver in the completion gate.

Do not copy external design-system catalog prose. Link to reference material and
extract only facts observed in the repo, provided by the user, or clearly
labeled inferred.

## Investigation Prompts

Launch independent investigators when the repo is non-trivial.

### Surface Mapper
> Map the product's visible surfaces. Find routes, screens, documents, reports,
> dashboards, components, story files, screenshots, demos, generated static
> output. Return: `| Surface | Source path | Audience | Frequency | Risk |`

### System Mapper
> Map existing design-system facts. Find tokens, Tailwind config, CSS variables,
> component libraries, typography, icons, charting, animation libs, Storybook,
> visual tests, a11y tools. Facts only; propose nothing.

### Taste Mapper
> Map intended feel and anti-patterns. Read README, docs, product copy,
> screenshots, issue/backlog language. Identify desired aesthetic, what is
> off-brand, and the surfaces to treat as golden examples.

## The DESIGN.md format (`@google/design.md`)

Adopt the open Google Labs DESIGN.md format wholesale — it is the converging
cross-agent standard (Anthropic's `frontend-design` is moving to it; DTCG hit
W3C stable 2025.10), and adopting it gives a free validator, a token exporter,
and interop instead of a bespoke format to maintain.

A `DESIGN.md` is YAML token frontmatter + a Markdown body of ordered sections.

```markdown
---
colors:        # name -> hex
  background: "#ffffff"
  foreground: "#0a0a0a"
  primary: "#2643d0"
typography:
  fontFamily: "Geist, sans-serif"
rounded:       # named dimensions
  sm: "4px"
  md: "8px"
  lg: "16px"
spacing: ["4px", "8px", "16px", "24px", "32px"]
---

# Overview
## Colors
## Typography
## Layout
## Elevation & Depth
## Shapes
## Components
## Do's and Don'ts
```

Sections may be omitted, but those present keep this order. Each section names
repo-owned facts, not generic taste advice; label inferred facts and lower
confidence; write the gap where evidence is missing instead of inventing.

The format is **purely visual by design.** Product intent, audience, brand
voice, and governance do not belong in `DESIGN.md` — carry them in the generated
project-local design skill (below) or a `PRODUCT.md`, not the visual contract.

> Tooling note (v0.3.0): `lint`, `diff`, and `export` work; the `spec` subcommand
> is broken (it fails to load its bundled `spec.md`). Use `lint` to validate the
> contract and the example above for the shape; do not rely on `npx
> @google/design.md spec`.

## Enforce the contract — gate the diff, don't trust prose

A rules file rots and loses to nearby examples. When agents author the UI, the
contract must enforce itself. Wire these into the consumer repo (the skill emits
them; HK itself, a Rust + static-docs repo, is not the dogfood target — a real
component UI is):

1. **Validate the contract.** `npx @google/design.md lint DESIGN.md` — emits
   JSON `findings` with `error`/`warning`/`info`. Treat any `error` as a failing
   gate (malformed tokens, bad refs, section-order). Proven: a clean contract
   returns `errors: 0`; a malformed token (e.g. a string where a dimension map
   belongs) returns errors. This is the falsifiable oracle.
2. **Emit the token layer.** `npx @google/design.md export DESIGN.md --format
   dtcg` → W3C DTCG `tokens.json` (consumable by Style Dictionary v5+, Tokens
   Studio, Figma). `--format css-tailwind` → Tailwind v4 `@theme`. The DESIGN.md
   is the source of truth; tokens are generated, never hand-maintained alongside.
3. **Lint authored source for off-system values.** Stylelint
   `declaration-strict-value` (ban raw color/spacing/radius; force `var(--token)`)
   + ESLint `no-restricted-imports` (block legacy/off-system components). Mature;
   assemble, don't build.
4. **Golden directory.** A small `src/design-system/examples/` of canonical
   implementations that compile in CI and break loudly when a component API drifts.
5. **One verification command.** A single `verify-ui` script (TS + ESLint +
   Stylelint + `design.md lint` + golden build) the agent must pass before a PR.
   Its value is the agent-loop: stderr re-enters the next prompt.

Prefer gating the **rendered output** where a live surface exists (framework-
agnostic; catches the violation however authored) over authoring-time-only
mechanisms. Reject a typed-prop compiler (StyleX `<Box>`, banned raw `<div>`)
unless a repo is already React-monorepo-locked at a scale that earns it. The
framework-agnostic rendered-DOM gate (Playwright → computed styles → diff vs
tokens) is net-new whitespace; evaluate it separately (backlog), do not block on it.

## Evidence Contract

Generate `design-contract.md` beside `DESIGN.md` when deriving a contract from
existing screens, screenshots, sites, docs, or user-provided examples.

| Source | Fact | Provenance | Confidence | Use | Evidence / Notes |
|---|---|---|---|---|---|
| `path-or-url` | Design fact to carry forward | `observed` / `provided` / `inferred` | high / medium / low | `keep` / `change` / `do-not-copy` | Artifact, quote, screenshot region, caveat |

- `observed` — visible in a rendered artifact, screenshot, code, token file, or
  committed product copy.
- `provided` — supplied by the user, stakeholder, brand guide, or repo docs.
- `inferred` — reasoned from weaker evidence; always include source + confidence.
- `keep` / `change` / `do-not-copy` — preserve / intentionally differ from /
  never clone (reference-brand, third-party, unlicensed, competitor UI).

Reference-brand material needs `do-not-copy` unless the user explicitly owns it
or provides a license-safe source. Never let an attractive external example
silently become the repo's brand.

## Generated Skill Shape

Write the project-local skill under the repo's shared skill root, then bridge per
harness. This skill carries the product framing the visual `DESIGN.md` omits.

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

[One sentence: what good design means in this project.]

## Product Feel
- Audience:
- Desired feel:
- Avoid:

## Surfaces
| Surface | Source | What Good Looks Like |
|---|---|---|

## Design System
- DESIGN.md: [path] — validate with `npx @google/design.md lint`.
- Component library / icons / motion / a11y tooling:

## Golden Examples
- [surface/path]: [why it is trusted]

## Completion Gate
- Intent, evidence (screenshot/render/visual diff), exact route exercised,
  DESIGN.md status, `design.md lint` result, residual risk.

## Red Lines
- [repo-specific things agents must not change]
```

`disable-model-invocation` is honored by Claude Code; other harnesses may ignore
it, so keep project-local trigger text specific.

## Quality Gate

Before declaring the scaffold complete:

- [ ] Surfaces are real paths/routes, not placeholders.
- [ ] Design-system facts are discovered from the repo, not invented.
- [ ] `DESIGN.md` uses the `@google/design.md` format and passes
      `npx @google/design.md lint` with `errors: 0`.
- [ ] Existing `DESIGN.md` was read and updated for durable visual changes or
      explicitly waived.
- [ ] `design-contract.md` records source, fact, provenance, confidence, and
      `keep`/`change`/`do-not-copy` for every load-bearing fact.
- [ ] Reference-brand material is marked `do-not-copy` unless ownership/license
      is explicit.
- [ ] Enforcement is wired where a consumer UI exists (lint + token export +
      stylelint/eslint + golden dir + one verify command), or its absence is
      named with reason.
- [ ] No bespoke DESIGN.md format, token engine, or framework was invented.
- [ ] No no-license external skill prose was copied into the repo.
