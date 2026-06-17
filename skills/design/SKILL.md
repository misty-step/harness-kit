---
name: design
description: |
  Artifact-backed interface critique and polish for hierarchy, typography,
  layout, density, IA, interaction feel, content, brand fit, and taste.
  Requires screenshot, URL, rendered artifact, or explicit file plus intent.
  Use when: "make this look better", "improve the design", "polish the UI",
  "critique this screen", "design pass", "art direction", "scaffold design",
  "prototype this", "show me a few options", "mock up variations",
  docs layout, report polish, generated diagrams/images, screenshots, decks,
  dashboards, charts, or any product-facing visual artifact.
  Trigger: /design, /prototype.
argument-hint: "[audit|redesign|polish|critique|scaffold|prototype] <artifact-or-surface>"
---

# /design

Critique and improve a rendered artifact against its intent. The core contract
is evidence, not generic advice:

1. Name the artifact: screenshot, URL, rendered file, route, or concrete source
   file that produces the surface.
2. Name the intent in one sentence: audience, job, and desired feel.
3. Inspect the rendered result when possible.
4. State the design read: surface kind, audience, desired feel, constraints.
5. Set VARIANCE / MOTION / DENSITY for the surface before building.
6. Return ranked, specific design moves or implement a bounded polish pass.

Refuse to make a final design judgment from code alone when a rendered surface
can be inspected. If rendering is impossible, mark the design unverified.

## Delegation Judgment

delegate on judgment per the shared Roster contract: native subagents
by default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use one lane for proposed direction or implementation and
another for cold review of substantive redesign, external-facing polish, or
final critique of visible UI changes.

## Routing

| Intent | Action |
|---|---|
| `/design audit` or `/design critique` | Read intent, inspect artifact, return ranked findings. For technical quality checks (a11y, performance, responsive), compose with `impeccable-impeccable`'s `/impeccable audit`. |
| `/design polish` | Apply the smallest coherent improvement set, then verify render. For the meticulous final pass, compose with `impeccable-impeccable`'s `/impeccable polish`. |
| `/design redesign` | Propose 2-3 structurally different directions, get convergence, then implement. |
| `/design scaffold` | Read `references/scaffold.md` and generate or update project-local `DESIGN.md` and `design-contract.md` when recurring or product-facing visual work earns them. |
| `/design study`, or the user supplies reference sites/screenshots as inspiration | Run the studied-DNA protocol in `references/external-design-references.md` (§ Reference-Driven Work). Extract DNA, never the dress; one primary donor per surface. The full extraction protocol is the installed `nutlope-hallmark` skill's `study` verb. |
| Greenfield page or identity generation | Compose the installed generation skills — `nutlope-hallmark` (macrostructure-first, theme rotation, slop gates) and `anthropic-frontend-design` (signature-element discipline) — then run `references/anti-slop.md`'s quick gate on the render. |
| `/design prototype`, "show me options", new feature UI where one-shot taste risk is high | Variation fan (below): 3–5 divergent options in one HTML file, operator picks, build the winner. |
| Iterative multi-issue prototyping: several named issues, verdict rounds, full-page compositions, viewport-dependent judgment | Lab registry (`references/lab-registry.md`): paged viewer with an adjustable viewport, one persistent section per issue, ≥6 options each, kill/mutate/seed across rounds. |
| Typography fixes (`/design typeset`) | Compose with `impeccable-impeccable`'s `/impeccable typeset` — font choices, hierarchy, sizing. |
| Color strategy (`/design colorize`) | Compose with `impeccable-impeccable`'s `/impeccable colorize` — strategic color without going garish. |
| Motion (`/design animate`) | Compose with `impeccable-impeccable`'s `/impeccable animate` and `emil-emil-design-eng` — purposeful motion tied to state. |
| Layout / spacing / rhythm (`/design layout`) | Compose with `impeccable-impeccable`'s `/impeccable layout`. |
| Simplify / strip to essence (`/design distill`) | Compose with `impeccable-impeccable`'s `/impeccable distill` — ruthless subtraction. |
| Production hardening (`/design harden`) | Compose with `impeccable-impeccable`'s `/impeccable harden` — edge cases, i18n, error states, overflow. |
| Responsive adaptation (`/design adapt`) | Compose with `impeccable-impeccable`'s `/impeccable adapt` — across screens and devices. |
| UX copy (`/design clarify`) | Compose with `impeccable-impeccable`'s `/impeccable clarify` — rewrite confusing UX copy. |
| Onboarding / empty states (`/design onboard`) | Compose with `impeccable-impeccable`'s `/impeccable onboard`. |
| UI performance (`/design optimize`) | Compose with `impeccable-impeccable`'s `/impeccable optimize` — LCP to bundle size. |
| Push bold (`/design bolder`) or tone down (`/design quieter`) | Compose with `impeccable-impeccable`'s `/impeccable bolder` or `/impeccable quieter`. |
| Delight moments (`/design delight`) | Compose with `impeccable-impeccable`'s `/impeccable delight` — small personality moments. |
| Extraordinary effects (`/design overdrive`) | Compose with `impeccable-impeccable`'s `/impeccable overdrive` — shaders, physics, 60fps. |
| Full shape-then-build (`/design craft`) | Compose with `impeccable-impeccable`'s `/impeccable craft` — design it then build it in one flow. |
| Design brief before building (`/design shape`) | Compose with `impeccable-impeccable`'s `/impeccable shape` — discovery, not guesswork. |
| Generate DESIGN.md (`/design document`) | Compose with `impeccable-impeccable`'s `/impeccable document` — Google Stitch DESIGN.md format. |
| Extract components/tokens (`/design extract`) | Compose with `impeccable-impeccable`'s `/impeccable extract` — pull into design system. |
| Live browser iteration (`/design live`) | Compose with `impeccable-impeccable`'s `/impeccable live` — pick element, get variants, accept into source. |
| Slop detection before shipping | Run `npx impeccable detect src/` — deterministic, no LLM, exit code 2 on findings. Wire as completion-gate check. |

Use `/qa` for behavior verification and evidence capture, and `/shape` when
the product direction itself is unsettled.

Accessibility is part of the design pass, not a separate ceremony: keyboard
reachability and focus order on interactive changes, visible focus states,
contrast (WCAG AA), labels/alt on controls and images, reduced-motion
respect. Run axe or equivalent on web surfaces; a11y findings are design
findings and get fixed with the same minimal-change discipline.
For recurring or product-facing visual work, use the references:

- `references/scaffold.md` for repo-owned `DESIGN.md` and
  `design-contract.md` provenance scaffolds.
- `references/design-system.md` for token and component-system judgment.
- `references/taste-layer.md` for aesthetic direction and anti-generic critique.
- `references/anti-slop.md` for the checkable list of AI design tells, the
  VARIANCE/MOTION/DENSITY dials, and the pre-emit gate. The full 46-pattern
  catalog and deterministic detector are in the installed `impeccable-impeccable`
  skill; run `npx impeccable detect src/` for machine-checked slop findings.
- `references/interface-polish.md` for micro-polish checks.
- `references/external-design-references.md` for license-safe use of external
  design skills, DESIGN.md sources, and inspiration libraries.
- `references/ui-surface-routing.md` for workflow composition.

If a repo already has `DESIGN.md`, read it before any visual change and update
it when the change alters durable tokens, visual language, component grammar,
layout density, content voice, accessibility rules, or golden examples. If
recurring or product-facing visual work lacks `DESIGN.md`, scaffold it or record
an explicit one-off/internal waiver in the completion gate.

## Critique Shape

Lead with the highest-leverage issues. Avoid a laundry list.

```markdown
## Design Critique
- Intent:
- Artifact inspected:
- Primary issue:
- Recommended direction:
- Specific changes:
- Verification needed:
```

Each finding names evidence from the artifact and one concrete change. If the
issue is only preference, say so; if it blocks comprehension or trust, say that.

## Redesign Directions

Directions must differ structurally, not just by palette:

- Minimal polish: preserve structure, improve hierarchy and rhythm.
- Editorial/narrative: guide attention through a story.
- Operational/workbench: increase density and repeated-use affordances.
- Brand-forward: make the product, client, or object unmistakable.
- Inversion: challenge the current organizing metaphor.

For each direction, name what it sacrifices. Recommend one.

## Variation Fan (prototype)

For new feature UI or any surface where one-shot taste risk is high, don't
one-shot: build 3–5 variations in **one self-contained HTML file** (inline
CSS/JS, no build step), labeled side-by-side, and let the operator pick.

- Variations must differ structurally — use the Redesign Directions axes
  above as divergence prompts. Five palette swaps of one layout is one
  variation.
- Each variation uses real content from the product, not lorem ipsum;
  fake content hides hierarchy failures.
- The fan is a sketch, not the implementation. After the pick, build the
  winner properly in the real stack with the generation skills and the
  anti-slop gate; never ship the prototype file.
- The fan is for one decision. When the work is several named issues
  iterated over verdict rounds — or judgment depends on viewport size —
  graduate to the lab registry (`references/lab-registry.md`).
- When the field is big and the operator doesn't want to judge it all, a
  harness with large-scale orchestration can run a tournament against a
  rubric to pre-filter — but the final pick stays human.

## Implementation Guardrails

- Change the fewest surfaces that can create a coherent improvement.
- Do not add a framework, animation system, or design token layer for a one-off
  surface.
- Scaffold a project-local design skill before enforcing tokens or component
  grammar across recurring UI surfaces.
- Prefer clearer hierarchy and better content structure over decoration.
- Preserve domain truth; design polish must not launder weak claims.
- Keep process out of the UI. Visible copy must not explain internal
  implementation notes, source uncertainty, artifact review/publication status,
  future work, or design rationale. Put that in docs or handoff; write the
  screen as the finished surface.
- After visible changes, verify desktop and mobile render and report evidence.

## Completion Gate

See `harnesses/shared/AGENTS.md` (Completion Evidence) for the shared evidence
core; this phase keeps design-specific local fields.

```markdown
## Completion Gate
- Direction chosen: critique, polish, redesign, or scaffold decision applied.
- Design read: surface kind, audience, desired feel, constraints.
- Dials: VARIANCE / MOTION / DENSITY values chosen for this surface.
- Evidence that proves it: screenshot, render, artifact, or visual diff inspected.
- Exact command/path/route exercised: URL, screenshot path, render command, or artifact path inspected.
- DESIGN.md status: read, created, updated, not present with waiver, or not applicable with reason.
- Hierarchy/content changes: specific hierarchy or content issue changed or recommended.
- Typography/layout changes: specific type, spacing, density, or layout issue changed or recommended.
- Copy provenance: visible copy inspected for product truth versus agent process leakage.
- Distinctive decision: the intentional design choice that prevents template sameness.
- Slop detector: `npx impeccable detect` result on changed files (clean, findings listed, or not applicable with reason).
- Residual risk: remaining design, a11y, or QA risk after inspection.

## Gotchas

- Design critique without an inspected artifact is speculation.
- Aesthetic preference is not blocking unless it hurts comprehension, trust,
  conversion, accessibility, or domain fit.
- Generic "modernize" moves are slop when they ignore the audience, density
  needs, or existing system.
- Never hide UI defects behind feature explanations. Point to the visible
  artifact and the concrete change.
- Meta-copy in UI is a design defect: headings like "if published, should be
  rounded and opt-in" describe the agent's caution, not the user's page. Real
  product policy, privacy notices, draft-state badges, or compliance disclosures
  are fine when the user is meant to see them.
