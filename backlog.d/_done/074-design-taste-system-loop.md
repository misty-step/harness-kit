# Design taste and system loop for UI work

Priority: P1
Status: pending
Estimate: M

## Goal

Give Harness Kit a thin, cross-harness design-quality loop for UI work:
detect when a change touches user-facing interface surfaces, route those
changes through artifact-backed design critique, scaffold project-local design
system guidance when a repo has recurring UI work, and verify the skill with a
small eval.

This is design-process infrastructure, not a shared UI framework.

## Research Notes

External research converged on the same failure mode: coding agents can build
functional interfaces, but underspecified UI work regresses toward common
training-data defaults. The useful systems separate four concerns:

1. **Design system constraints**: tokens, component grammar, typography scale,
   spacing, motion, density, states, and accessibility.
2. **Taste/judgment layer**: references, mood boards, distinctive direction,
   anti-generic critique, and micro-polish rules.
3. **Rendered verification**: screenshots, viewports, visual diffs, browser
   walks, and before/after evidence.
4. **Scaffolded repo memory**: project-local SKILL.md files that name the
   actual product surfaces, tokens, brand voice, golden examples, and things
   agents must not change.

Sources checked during shaping:

- Anthropic frontend-design guidance: skills load specialized frontend design
  context on demand and fight distributional convergence without permanently
  bloating every task.
- OpenAI frontend/Codex guidance: define design system constraints up front,
  use visual references or mood boards, and verify rendered work with browser
  tooling such as Playwright.
- Vercel/v0 guidance: high-fidelity UI generation improves when prompts include
  concrete component, style, source, and project-context constraints.
- Taste / taste-skill: design taste can be treated as a portable profile or
  skill layer across Cursor, Claude Code, Codex, and project surfaces.
- Jakub Krehel's make-interfaces-feel-better skill: concrete micro-details
  such as text wrapping, concentric radii, tabular numbers, animation
  interruptibility, and optical alignment make better review checklists than
  generic "make it polished" prose.
- Emil Kowalski's design engineering skill: motion/timing and component feel
  need decision rules, not just "add animation".
- Public design-system exemplars such as GOV.UK, USWDS, Carbon, Material,
  Fluent, and Atlassian are useful for vocabulary and component-page anatomy,
  not for importing their visual language.

Local research found Harness Kit already has most of the raw ingredients:

- `skills/design/SKILL.md` requires rendered artifacts, explicit intent,
  ranked critique, structural redesign directions, and a completion gate.
- `skills/a11y/SKILL.md` has the stronger triad pattern:
  audit -> remediate -> critique.
- `skills/demo/SKILL.md` and `skills/qa/references/evidence-capture.md` cover
  screenshots, GIFs, videos, and evidence packaging.
- `skills/browser/SKILL.md` already frames browser verification as a pyramid
  where findings harden into deterministic tests.
- `registry.yaml` already tracks Anthropic, Vercel, OpenAI, Jakub, Emil, and
  Leon taste-skill sources.
- `backlog.d/061-public-docs-companion.md` already asks for restrained layout,
  excellent typography, high-contrast diagrams, simple filters, workflow cards,
  and a generated catalog.

## Problem Challenge

Adding a design token system alone will not solve AI slop. Tokens can make a
generic interface consistent. They do not make it distinctive, trustworthy, or
appropriate for the product.

Conversely, importing aesthetic skills wholesale can create taste theater:
font bans, purple-gradient avoidance, dramatic motion, or landing-page
composition where the target repo actually needs a quiet operational console.

The Harness Kit-sized solution is a routing and evidence loop:

- detect UI-relevant changes;
- load repo-local design-system facts when they exist;
- use external taste skills as references, not as global doctrine;
- inspect rendered output;
- package evidence; and
- harden recurring findings into repo-local checks.

## Design

### 1. Add a UI surface detector

Add a small script such as `scripts/detect-ui-surfaces.sh`.

It should inspect the current diff or explicit path list and return whether the
change touches likely UI surfaces:

- `*.tsx`, `*.jsx`, `*.vue`, `*.svelte`, `*.css`, `*.scss`
- `app/**`, `pages/**`, `components/**`, `src/components/**`
- `stories/**`, `*.stories.*`
- design-token files such as `tokens.*`, `tailwind.config.*`,
  `theme.*`, `components.json`

The detector is a router input, not a quality verdict.

### 2. Teach `/deliver` and `/settle` to route UI diffs

When the detector fires, the clean loop should add:

- `/design audit` or `/design polish` for visual hierarchy, taste, and intent;
- `/a11y` for WCAG and keyboard/screen-reader concerns;
- `/qa` or `/browser` for rendered behavior and viewport evidence; and
- `/demo` for screenshot/GIF packaging when the change is user-visible.

This should be soft routing in the workflow skills, not a hard global CI gate.

### 3. Add `/design scaffold`

Add a scaffold mode to `skills/design/SKILL.md`, mirroring the existing `/qa`
and `/demo` scaffold patterns.

The scaffold should generate a project-local design skill only for repos with
recurring UI work. It should capture:

- product type and audience;
- primary surfaces and golden flows;
- existing tokens, type scale, radii, spacing, motion, and component library;
- approved components and composition grammar;
- brand voice and aesthetic direction;
- visual anti-patterns to avoid;
- screenshot/evidence path; and
- repo-specific non-goals and red lines.

It must not generate a framework, component library, or token engine.

### 4. Add design references, not prose bloat

Create `skills/design/references/` with concise loaders:

- `design-system.md`: token hierarchy, component grammar, state coverage,
  accessibility, and repo-local ownership.
- `taste-layer.md`: how to use visual references, mood boards, and taste
  profiles without copying an external aesthetic wholesale.
- `interface-polish.md`: distilled micro-polish checks from Jakub/Emil style
  sources, preserving license boundaries by linking to externals rather than
  copying no-license content.
- `ui-surface-routing.md`: how `/deliver`, `/settle`, `/code-review`, `/qa`,
  `/a11y`, `/browser`, and `/demo` compose for UI diffs.

Keep `skills/design/SKILL.md` terse. The references carry depth.

### 5. Add one eval for `/design`

Add a small eval under `skills/design/evals/` that checks a design critique
response against a fixture screenshot or rendered artifact description.

The grader should verify that the candidate:

- named the inspected artifact and intent;
- refused code-only final judgment when render evidence was available;
- identified hierarchy, typography, density, contrast, and interaction issues
  with evidence;
- recommended a coherent direction rather than a laundry list;
- did not propose a framework/token system for a one-off surface; and
- emitted the `## Design Gate` shape when implementing or closing a polish pass.

## Oracle

- [ ] `scripts/detect-ui-surfaces.sh` returns a stable machine-readable result
      for staged diff, unstaged diff, and explicit path-list modes.
- [ ] `/deliver` or its clean-loop reference routes detected UI diffs through
      `/design` and `/a11y` without requiring that path for non-UI changes.
- [ ] `/design scaffold` is documented and produces a repo-local design skill
      only when a repo has recurring UI surfaces.
- [ ] `skills/design/references/` exists and carries the design-system,
      taste-layer, interface-polish, and UI-surface routing details.
- [ ] `skills/design/evals/` contains at least one runnable eval and grader.
- [ ] The implementation preserves cross-harness behavior: filesystem
      `SKILL.md` remains primary, with Claude/Codex/Pi bridges treated as
      runtime optimizations.
- [ ] `dagger call check --source=.` passes.

## Non-Goals

- No shared Harness Kit UI framework.
- No global token engine.
- No hard CI fail on subjective design taste.
- No static `designer` agent in `agents/`.
- No design doctrine in `harnesses/shared/AGENTS.md` beyond routing if a later
  shaped item proves it necessary.
- No vendoring no-license external skill prose into first-party skills.
- No Chromatic/Percy/Storybook requirement at the shared harness layer.
- No claim that design tokens alone solve aesthetic quality.

## Roster / Research Receipt

This item was shaped from a `/research` + `/harness` run on 2026-05-27.
Evidence included Exa, xAI/Grok web search, retired bench, Codex, Claude, Pi, web
search, and live codebase inspection. Provider output was treated as evidence,
not authority.
