---
name: design
description: |
  Artifact-backed interface critique and polish for hierarchy, typography,
  layout, density, IA, interaction feel, content, brand fit, and taste.
  Requires screenshot, URL, rendered artifact, or explicit file plus intent.
  Use when: "make this look better", "improve the design", "polish the UI",
  "critique this screen", "design pass", "art direction", "scaffold design",
  docs layout, report polish, generated diagrams/images, screenshots, decks,
  dashboards, charts, or any product-facing visual artifact.
  Trigger: /design.
argument-hint: "[audit|redesign|polish|critique|scaffold] <artifact-or-surface>"
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

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use one lane for proposed direction or implementation and
another for cold review of substantive redesign, external-facing polish, or
final critique of visible UI changes.

## Routing

| Intent | Action |
|---|---|
| `/design audit` or `/design critique` | Read intent, inspect artifact, return ranked findings. |
| `/design polish` | Apply the smallest coherent improvement set, then verify render. |
| `/design redesign` | Propose 2-3 structurally different directions, get convergence, then implement. |
| `/design scaffold` | Read `references/scaffold.md` and generate or update project-local `DESIGN.md` and `design-contract.md` when recurring or product-facing visual work earns them. |

Use `/a11y` for WCAG compliance, `/qa` for behavior verification, `/demo` for
evidence packaging, and `/shape` when the product direction itself is unsettled.
For recurring or product-facing visual work, use the references:

- `references/scaffold.md` for repo-owned `DESIGN.md` and
  `design-contract.md` provenance scaffolds.
- `references/design-system.md` for token and component-system judgment.
- `references/taste-layer.md` for aesthetic direction and anti-generic critique.
- `references/anti-slop.md` for the checkable list of AI design tells, the
  VARIANCE/MOTION/DENSITY dials, and the pre-emit gate.
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
- Residual risk: remaining design, a11y, or QA risk after inspection.
```

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
