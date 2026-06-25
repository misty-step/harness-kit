# Divergence First

The standing method for any design work that *creates or redesigns*. Models
brainstorm well and decide badly — so make the model diverge and the operator
choose. This reference carries the part that is not obvious: **what "a variation"
means in each design mode**, and how to keep wide differentiation *inside* the
constraints instead of sliding into noise or reskins.

## Step zero — name the constraint set

Before generating anything, state what is FIXED for this job. Wide creative
distinction is only meaningful relative to a fence; without one you get either
timid reskins or off-brief noise. The fence's source differs by mode (below).
Say it out loud: *"Fixed: the system's tokens + the brand voice. Varying:
layout, hierarchy, density."* Then diverge hard inside it.

A variant that breaks a constraint is **noise** for a within-system task — but
may be **signal** for a system-design task, where the constraints themselves are
what you're exploring. Knowing which mode you're in is the whole game.

## Variation axes by design mode

| Mode | Fixed (the constraints) | Varies (diverge across these) | Navigable artifact |
|---|---|---|---|
| **Design system** (inventing it) | brand brief, product premise, platform, a11y floor | type-scale strategy, color architecture, density philosophy, motion language, shape language, voice | a **sampler**: the same 3–4 representative surfaces (card, form, page header, data row) rendered under each candidate system, paged or side-by-side |
| **Component / page** (given a system) | the system's tokens + components — never re-invent them | layout, composition, hierarchy, emphasis, information density, entry/empty/error states | the one-shot fan or the lab-registry paged viewer — full-viewport pages, real chrome |
| **UX flow** (not UI) | the user's job-to-be-done, the required steps/states, entry & exit points | flow structure (wizard vs single-screen vs progressive disclosure vs inline), step count, where branches/decisions sit, optimistic vs confirmed, recovery/error paths | clickable step sequences or flow diagrams (mermaid/boxes) per variant — the artifact shows the *path*, not one pretty screen. Route to `/shape` when the product direction itself is unsettled |
| **Motion** | the surface + its tokens + the reduced-motion rule | easing curve, duration, choreography (stagger/sequence), the triggering moment, what moves vs. what holds | side-by-side live motion demos (`emil` author/review primaries) |
| **Copy / voice** | product truth, the surface, the audience | register, length, what leads, framing, the CTA verb | candidates rendered IN the real chrome (type only reads in context), never bare strips |

"And much more" resolves the same way: name what's fixed, pick the axes that
genuinely change the experience, render variants so they can be compared fast.

## Fan width — scale to render cost, not to a number

Count is never the point; distinctness within constraints is. Six reskins are
worse than three genuinely different directions.

- **3–5** structurally distinct directions when each is expensive to build or the
  call is coarse (the `/design redesign` direction menu sits here).
- **≥6** is the lab-registry floor for real, rounds-based prototyping.
- **up to ~12–20** when variants are cheap to render and the operator wants a wide
  sweep ("20 variants, one button each"). The lab-registry paged viewer (arrow-key
  nav, one full-viewport page per option) is the artifact for this — it is the
  operator's confirmed default for almost all prototyping.

## Generate distinct directions, not reskins

Route the ideation through `nous-creative-ideation`'s named methods so the spread
has real spread:

- **SCAMPER** — mutate a base direction systematically (substitute, combine,
  adapt, modify, put-to-other-use, eliminate, reverse).
- **lateral-provocations / analogy-and-blending** — break frame; import structure
  from a remote domain (a transit map, a darkroom, a trading terminal).
- **volume-generation** — when you need many fast.

Anti-slop rule, same as everywhere: refuse the first obvious option, and make at
least one variant **invert a load-bearing assumption** of the brief.

## Then converge

Divergence and convergence are different phases — do not blend them. Once the
operator picks:

- switch to convergent: build the winner properly with the chosen preset + the
  anti-slop gate;
- update `DESIGN.md` if a durable fact changed;
- never ship the prototype/sampler file — it was a sketch.

## Failure modes

- One answer when the operator asked to explore. (The default model reflex; the
  Review Gate blocks it.)
- "Variants" that are palette/font swaps of one layout — one variation wearing
  costumes.
- Diverging *outside* the constraints on a within-system task (brilliant, off-system, useless).
- Prose descriptions of options instead of a rendered, navigable artifact.
- Endless fans, no build — divergence that never converges.
- Auto-deciding the aesthetic for the operator. The pick is theirs.
