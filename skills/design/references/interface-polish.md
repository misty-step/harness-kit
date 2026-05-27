# Interface Polish

Use this reference after the structure and intent are right. It distills
micro-polish checks from external design-engineering skills without vendoring
their prose or visual identity.

## Hierarchy

- The primary action is visually dominant without drowning secondary actions.
- Headings match the density of their container; compact panels do not use
  hero-scale type.
- Supporting text clarifies the next action or evidence, not generic value
  claims.
- Tables, lists, and cards preserve scan lines and comparison affordances.

## Typography

- Line length is readable for prose and compact for dense controls.
- Numbers that need comparison use tabular figures.
- Labels and values have distinct roles; labels should not compete with data.
- Long words, button labels, and headings fit their containers at mobile and
  desktop widths.

## Spacing And Alignment

- Repeated elements follow a consistent rhythm.
- Icons are optically aligned with text, not merely geometrically centered.
- Nested radii are concentric when containers sit inside each other.
- Fixed-format elements have stable dimensions so hover, labels, and dynamic
  content do not shift layout.

## Motion And Interaction

- Motion explains state change, spatial relationship, or feedback.
- Prefer transform and opacity for cheap transitions.
- Durations vary by interaction weight; do not use one default duration for
  every element.
- Press, hover, drag, loading, error, and disabled states are designed.
- Respect `prefers-reduced-motion` for non-essential animation.

## Visual Detail

- Shadows, borders, and backgrounds encode hierarchy; they are not decoration.
- Images and media reveal the real product, object, or state when inspection
  matters.
- Focus rings are visible and harmonious with the surface.
- Color is semantic enough that status, selection, and hierarchy survive theme
  changes.

## Final Pass

Ask:

1. What would a user notice first?
2. What would they do next?
3. What feels generic or template-derived?
4. What detail makes the surface feel intentionally made for this product?
5. What evidence proves the rendered result improved?
