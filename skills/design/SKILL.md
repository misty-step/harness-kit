---
name: design
description: |
  Artifact-backed interface design critique and polish. Use when an existing
  screen, report, dashboard, website, game, or tool should feel better:
  visual hierarchy, typography, layout, density, information architecture,
  interaction feel, content structure, brand fit, taste, or aesthetic quality.
  Requires a screenshot, live URL, rendered artifact, or explicit file plus
  a one-sentence intent. Triggers on "make this look better", "improve the
  design", "more tasteful", "better hierarchy", "polish the UI",
  "critique this screen", "reimagine this page", "layout", "typography",
  "visual design", "aesthetic", "design pass", "art direction",
  "scaffold design", "generate design skill".
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
