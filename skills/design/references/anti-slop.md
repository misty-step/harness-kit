# Anti-Slop

Concrete, checkable rules for stripping the "AI-generated" tells out of a
surface. Use alongside `taste-layer.md` (direction) and `design-system.md`
(tokens). Distilled from three public design skills — credit and further
reading:
- **impeccable** (`impeccable.style`, installed as `impeccable-impeccable`) —
  desloppification; a deterministic 46-pattern detector with `npx impeccable
  detect` (exit code 2 on findings); respect the project's existing
  tokens/components. PRODUCT.md + DESIGN.md context system. 23 commands for
  discipline-specific work (typeset, colorize, animate, layout, distill, etc.).
- **hallmark** (`github.com/Nutlope/hallmark`, vendored as
  `nutlope-hallmark`) — macrostructure variance, a 58-gate slop test,
  pre-emit self-critique, "two briefs should feel like different sites, not
  colour-swaps." Run the installed skill for builds, audits, and `study`.
- **taste-skill** (`github.com/Leonxlnx/taste-skill`, vendored as
  `leon-taste-skill`) — the VARIANCE / MOTION / DENSITY dials and an
  anti-slop pre-flight.

## The slop tells (reject on sight)

These are the defaults every model drifts into. Treat each as a bug:

- **Gradient text** and rainbow/`bg-clip-text` headings.
- **Purple→indigo hero gradients** and the generic "AI palette" (one violet
  accent on near-black). Pick a palette from the product, not the model's prior.
- **Glassmorphism** (`backdrop-blur` frosted cards) used as decoration.
- **Decorative blobs, bokeh, mesh gradients** standing in for hierarchy.
- **Side-stripe borders** (a coloured 4px left border on every card).
- **Inter + fully-rounded cards + soft shadow** as the entire visual language.
- **Uniform motion** — everything fades/floats the same way, untied to meaning.
- **Over-large headings** inside dense/operational surfaces.
- **Em-dashes in product copy** — a giveaway of generated text; prefer plain
  punctuation in UI strings.
- **Placeholder/stock content** — charts, avatars, or numbers that do not show
  real product state.
- **Reference cloning** — a surface traceable to one inspiration source:
  its hero composition, palette, and type voice reproduced together. DNA
  travels (macrostructure, type roles, colour-anchor bands); the dress does
  not. See `external-design-references.md` § Reference-Driven Work.
- **Meta-copy as UI** — headings, cards, buttons, nav, or empty states that
  explain internal implementation notes, artifact review/publication status,
  future work, or agent uncertainty instead of naming the thing. "Top holdings,
  if published, should be rounded and opt-in" is not a heading; it is a handoff
  note. Legitimate user-facing policy, privacy, draft-state, or compliance copy
  is not the defect; leaking the agent's process is.

## Three dials — set them before you build

State the target explicitly; a dashboard and a landing page sit at opposite ends.

- **VARIANCE** (conventional ↔ experimental layout). Tools/dashboards: low —
  the work surface leads, not a centered hero. Marketing: higher.
- **MOTION** (still ↔ kinetic). Operational UI: minimal, and only on
  interaction (hover, expand, state change). Never animate idle data.
- **DENSITY** (airy ↔ packed). Operator consoles: high, scannable rows; landing
  pages: airy. Pick one and keep it consistent within a surface.

## Respect what exists

Before adding anything, scan the project's tokens, component library, and
conventions and inherit them. Do not introduce a second accent, a second type
scale, or a new radius/shadow language next to an existing one. New work should
look like it was always there.

## Make it distinctive, not just clean

"Clean" is necessary, not sufficient. A surface should have one memorable
decision a competitor's default would not make — a deliberate type pairing, a
structural layout choice, a restrained signature colour used with intent. Two
surfaces you build for two different products should not be colour-swaps of one
template.

## Quick gate (run before declaring done)

1. No gradient text, no purple-on-black default, no decorative glass/blobs.
2. Palette and type scale come from the product's tokens, used consistently.
3. Light and dark both legible; contrast meets WCAG AA (verify with axe or manual check).
4. Motion is tied to interaction, not ambient.
5. Density matches the surface's job; headings are sized for it.
6. Copy has no em-dashes or placeholder text.
7. No visible copy says "should", "could", "if published", "opt-in",
   "public-safe", "placeholder", "future layer", or "metric to confirm" unless
   the actual product domain is about those terms.
8. One distinctive, intentional decision — name it.
