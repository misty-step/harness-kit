---
name: design
description: |
  Artifact-backed interface critique and polish for hierarchy, typography,
  layout, density, IA, interaction feel, content, brand fit, and taste.
  Requires screenshot, URL, rendered artifact, or explicit file plus intent.
  Use when: "make this look better", "improve the design", "polish the UI",
  "critique this screen", "design pass", "art direction", "scaffold design".
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
4. Return ranked, specific design moves or implement a bounded polish pass.

Refuse to make a final design judgment from code alone when a rendered surface
can be inspected. If rendering is impossible, mark the design unverified.

## Delegation Floor

When a provider roster is available (repo `.harness-kit/agents.yaml` or system
`~/.harness-kit/agents.yaml`), use two or more roster members for substantive
redesign, external-facing polish, or final critique of visible UI changes. Use
one lane for proposed direction or implementation and another for cold review.
Direct lead-only design work is limited to mechanical capture, emergency
unblocks, explicit user-forbidden delegation, tiny one-off critiques, or fewer
than two available roster members.

## Routing

| Intent | Action |
|---|---|
| `/design audit` or `/design critique` | Read intent, inspect artifact, return ranked findings. |
| `/design polish` | Apply the smallest coherent improvement set, then verify render. |
| `/design redesign` | Propose 2-3 structurally different directions, get convergence, then implement. |
| `/design scaffold` | Read `references/scaffold.md` and generate project-local design guidance for recurring UI work. |

Use `/a11y` for WCAG compliance, `/qa` for behavior verification, `/demo` for
evidence packaging, and `/shape` when the product direction itself is unsettled.
For recurring UI work, use the references:

- `references/design-system.md` for token and component-system judgment.
- `references/taste-layer.md` for aesthetic direction and anti-generic critique.
- `references/anti-slop.md` for the checkable list of AI design tells, the
  VARIANCE/MOTION/DENSITY dials, and the pre-emit gate.
- `references/interface-polish.md` for micro-polish checks.
- `references/ui-surface-routing.md` for workflow composition.

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
- After visible changes, verify desktop and mobile render and report evidence.

## Completion Gate

```markdown
## Design Gate
- Direction chosen:
- Artifact/render inspected:
- Hierarchy/content changes:
- Typography/layout changes:
- Verification evidence:
- Residual design risk:
```

## Gotchas

- Design critique without an inspected artifact is speculation.
- Aesthetic preference is not blocking unless it hurts comprehension, trust,
  conversion, accessibility, or domain fit.
- Generic "modernize" moves are slop when they ignore the audience, density
  needs, or existing system.
- Never hide UI defects behind feature explanations. Point to the visible
  artifact and the concrete change.
