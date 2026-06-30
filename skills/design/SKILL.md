---
name: design
description: |
  Artifact-backed interface design: critique, polish, redesign, generate, and a
  repo-owned design contract. One front door over a bench of design specialists —
  routes to exactly one primary per role; you pick the aesthetic.
  Requires screenshot, URL, rendered artifact, or explicit file plus intent.
  Use when: "make this look better", "improve the design", "polish the UI",
  "critique this screen", "design pass", "art direction", "make it premium",
  "make it brutalist/minimalist", "deslop this", "scaffold design", "DESIGN.md",
  "design system", "prototype this", "show me a few options", "mock up
  variations", "is this accessible", docs layout, report polish, generated
  diagrams/images, dashboards, charts, or any product-facing visual artifact.
  Trigger: /design, /prototype.
argument-hint: "[audit|polish|redesign|scaffold|prototype|<preset>|<verb>] <artifact-or-surface>"
---

# /design

Critique and improve a rendered artifact against its intent — and route the
specifics to the right specialist. `/design` is one front door over a bench of
design skills: it owns the contract and the taste call and dispatches each job
to exactly one primary per role. It is a **menu, not a pipeline** — every route
below stands alone; none waits on another's output.

## The core contract (every route)

These six are the method *within* a single design pass — how one route executes,
not a sequence across routes. Pick the route that fits the job and run this once.

1. Name the artifact: screenshot, URL, rendered file, route, or the source file
   that produces the surface.
2. Name the intent in one sentence: audience, job, desired feel.
3. Inspect the rendered result when possible.
4. State the design read: surface kind, audience, desired feel, constraints.
5. Set the dials (below) for the surface.
6. Return ranked, specific moves — or implement a bounded change — then verify
   the render.

Refuse to make a final design judgment from code alone when a rendered surface
can be inspected. If rendering is impossible, mark the design unverified.

## Set two dials first (choices, not steps)

**Aesthetic preset** — pick the vibe. Each is a distinct, vendored direction,
not a restyle of one template:

| Preset | Feel | Primary |
|---|---|---|
| default | neutral premium — Zinc/Slate, restrained | `leon-taste-skill` |
| soft | glossy, luxe, agency-tier | `leon-soft-skill` |
| minimalist | flat editorial (Notion/Linear), no gradient/shadow | `leon-minimalist-skill` |
| brutalist | raw industrial-terminal, `border-radius:0`, hazard accent | `leon-brutalist-skill` |

For greenfield generation, `nutlope-hallmark`'s genres (editorial / modern-
minimal / atmospheric / playful) are a parallel preset axis — pick one there.

**Variance / Motion / Density** — set per `references/anti-slop.md` before
building. A dashboard and a landing page sit at opposite ends; state the target.

## Route to one primary per role

Each row is an independent entry point. Invoke the one that matches the job; do
not chain them into a sequence.

| Need | `/design` verb | Primary |
|---|---|---|
| Critique / audit a rendered surface | `audit` | inspect + `impeccable audit` |
| Final meticulous polish pass | `polish` | `jakub-make-interfaces-feel-better` (exact values) + `impeccable polish` |
| Redesign an existing site, keep the stack | `redesign` | `impeccable audit`+`critique`; fold `leon-redesign-skill`'s a11y/SEO omissions checklist |
| Generate a page / identity from scratch | — | `nutlope-hallmark` (+ `leon-gpt-tasteskill` for GSAP/AIDA generation) |
| Build UI to match a design image | — | `leon-images-taste-skill` (image-gen runtime is live — `harnesses/shared/references/image-generation.md`) |
| Generate an image / diagram / contact sheet of options | — | image-gen direct — NB2 Lite ~$0.03/img, legible in-image text; `harnesses/shared/references/image-generation.md` |
| Motion — author it | `animate` | `emil-emil-design-eng` |
| Motion — review it | — | `emil-review-animations` |
| Accessibility / web-interface guidelines | — | `vercel-web-design-guidelines` |
| Page / route transitions (React) | — | `vercel-react-view-transitions` |
| Component architecture / prop API | — | `vercel-composition-patterns` |
| Repo design contract + tokens | `scaffold` / `document` | `@google/design.md` (`lint` / `export dtcg`) via `references/scaffold.md`; `leon-stitch-skill` for Stitch format |
| Typography / color / layout / distill / harden / adapt / clarify / onboard / optimize / delight / overdrive / live | `typeset` … | the matching `impeccable <verb>` |
| Slop detection before shipping | — | `references/anti-slop.md`; superset gate `nutlope` slop-test; deterministic `npx impeccable detect src/` (exit 2) |

The dial and role axes are orthogonal, so a request never matches two rows
ambiguously: an **aesthetic preset** wins when the request names a *vibe*
(brutalist, luxe, minimalist); a **role** wins when it names a *surface to
improve or a job to do* (audit this, polish these, redesign that).

Use `/qa` for behavior verification and evidence capture, and `/shape` when the
product direction itself is unsettled.

## Anti-slop ban-core (one source)

`references/anti-slop.md` is HK's single ban-list: the slop tells, the
VARIANCE/MOTION/DENSITY dials, and the pre-emit quick gate. Do not re-derive it
per surface and do not paste the vendored skills' copies in — route to them and
let each carry its own. `nutlope`'s 58-gate slop-test is the superset oracle;
`npx impeccable detect` is the deterministic, no-LLM check.

## Accessibility is part of the pass, not a ceremony

Keyboard reachability and focus order on interactive changes, visible focus
states, contrast (WCAG AA), labels/alt on controls and images, reduced-motion
respect. Run axe or equivalent on web surfaces; a11y findings are design
findings and get fixed with the same minimal-change discipline.

## Review gate — earned approval, default-deny

Substantive or external-facing visual changes pass a gate before "done", not a
rubber stamp. Run two fresh-context lanes — a heuristic design-director read and
a deterministic scan (`npx impeccable detect` + the anti-slop quick gate) — and
**synthesize** them; do not just concatenate. Approval is earned. These are
presumptive blockers the author must justify, not nits:

- An off-system value (color / spacing / radius / type) not in the repo's tokens.
- A surviving slop tell from `references/anti-slop.md`.
- Contrast below WCAG AA, or an interactive control unreachable by keyboard.
- No distinctive decision — the surface is template-clean but anonymous.
- Meta-copy: UI that explains the agent's process instead of naming the thing.
- A generative/redesign request answered with a single option when the operator
  wanted to explore — or "variants" that are palette/font swaps of one layout
  rather than structurally distinct directions within the named constraints.

## Delegation Judgment

Delegate per the shared Roster contract: native subagents by default; one lane
for the proposed direction/implementation and a separate fresh-context lane for
cold review of substantive redesign, external-facing polish, or final critique.
Add cross-model critics or `/sprites` only when they answer a distinct question.

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

Directions must differ structurally, not by palette. Recommend one, and name
what each sacrifices: **minimal polish** (preserve structure, fix hierarchy and
rhythm), **editorial** (guide attention through a story), **workbench** (more
density, repeated-use affordances), **brand-forward** (make it unmistakable),
**inversion** (challenge the organizing metaphor).

## Divergence first (the standing stance for any generative work)

Models are strong at generating options and weak at choosing between them — so
use them that way. For any route that *creates or redesigns* a surface (generate,
redesign, prototype, scaffold a system, shape a flow), the default is to
**diverge, then let the operator decide** — never one-shot a single answer, never
auto-pick the aesthetic. The decision is the operator's; the spread is yours.

Step zero is naming the **constraint set** — what is FIXED for this job (the
repo's `DESIGN.md` tokens, brand, platform, a11y floor, the product premise).
Wide creative distinction *within* those constraints is the goal; differentiation
that breaks them is noise, and five palette swaps of one layout is one variation,
not five. What varies vs. what stays fixed differs by design mode — system,
component/page, UX flow, motion, copy — and `references/divergence.md` maps the
axes and the right navigable artifact for each.

Present the spread as a **navigable artifact**, not prose: a one-shot fan
(several structurally distinct options in one self-contained HTML file, real
content, labeled, switchable) for "pick one of these"; the paged
`references/lab-registry.md` viewer (≥6 options, full-viewport pages, arrow-key
nav, adjustable viewport — the operator's confirmed default for almost all
prototyping) for rounds or viewport-dependent calls. Generate genuinely distinct
directions by routing the ideation through `nous-creative-ideation`'s named
methods (SCAMPER, lateral provocations, analogy) rather than reskinning one idea.
Then **converge**: build the chosen direction properly with the preset and the
anti-slop gate; never ship the prototype file.

## Implementation Guardrails

- Change the fewest surfaces that make a coherent improvement; prefer hierarchy
  and content structure over decoration.
- No framework, animation system, or token layer for a one-off surface. Scaffold
  a project-local contract before enforcing tokens across recurring UI.
- Preserve domain truth; polish must not launder weak claims. Keep process out
  of the UI — copy reads as the finished surface, not internal notes.
- After visible changes, verify desktop and mobile render and report evidence.
- If a repo has `DESIGN.md`, read it before any visual change and update it when
  durable facts change; else scaffold it (`references/scaffold.md`) or waive as
  one-off in the completion gate.

## References

- `references/scaffold.md` — repo-owned `DESIGN.md` in the `@google/design.md`
  format (`lint` + DTCG export) and `design-contract.md` provenance.
- `references/design-system.md` — token and component-system judgment.
- `references/taste-layer.md` — aesthetic direction and anti-generic critique.
- `references/aesthetic-library.md` — six operator-endorsed aesthetic
  directions (neo-brutalist, terminal-TUI, soft-luxe, memphis, web-1.0,
  instrument-panel), each with DNA + a runnable HTML example to pull from.
- `references/divergence.md` — the divergence-first method: per-mode variation
  axes (system / component / UX flow / motion / copy), constraint-naming,
  navigable artifacts, fan width, and the converge handoff.
- `references/anti-slop.md` — the single ban-core: slop tells, dials, quick gate.
- `references/interface-polish.md` — micro-polish checks.
- `references/external-design-references.md` — license-safe use of the vendored
  design skills and inspiration libraries.
- `references/ui-surface-routing.md` — composing `/design` + `/qa` + `/code-review`.
- `references/lab-registry.md` — paged adjustable-viewport prototyping viewer.
- `evals/routing-eval.md` — the routing oracle (this skill's verification system).

## Completion Gate

See `harnesses/shared/AGENTS.md` (Completion Evidence) for the shared core; this
phase keeps design-specific fields.

```markdown
## Completion Gate
- Direction chosen: critique, polish, redesign, generate, or scaffold applied.
- Route taken: which preset and which role-primary were invoked.
- Design read: surface kind, audience, desired feel, constraints.
- Dials: VARIANCE / MOTION / DENSITY values chosen for this surface.
- Evidence that proves it: screenshot, render, artifact, or visual diff inspected.
- Exact command/path/route exercised: URL, screenshot path, render command, or artifact path.
- DESIGN.md status: read, created, updated, not present with waiver, or n/a with reason.
- Hierarchy/content + type/layout changes: the specific issues changed or recommended.
- Copy provenance: visible copy inspected for product truth vs. agent-process leakage.
- Distinctive decision: the intentional choice that prevents template sameness.
- Slop detector: `npx impeccable detect` result on changed files (clean, listed, or n/a).
- Residual risk: remaining design, a11y, or QA risk after inspection.
```

## Gotchas

- Design critique without an inspected artifact is speculation.
- The routes are a flat menu: never sequence them into a phase pipeline. No
  route depends on another route's output — that drift is the workflow engine
  VISION.md forbids.
- Aesthetic preference is not blocking unless it hurts comprehension, trust,
  conversion, accessibility, or domain fit.
- Generic "modernize" moves are slop when they ignore audience, density, or the
  existing system.
- Never hide UI defects behind feature explanations. Point to the visible
  artifact and the concrete change.
- Meta-copy in UI is a design defect: real product policy, privacy, draft-state,
  or compliance copy is fine when the user is meant to see it; leaking the
  agent's caution is not.
