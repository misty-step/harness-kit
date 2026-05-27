# Case: rendered dashboard critique

## Prompt

Run `/design audit` for a rendered screenshot of an operational dashboard.

Intent: help an on-call operator scan incidents quickly and choose the next
action. The screenshot shows:

- a top heading that is larger than the data table;
- five equal visual-weight cards before the incident list;
- status colors applied directly as Tailwind utility classes;
- inconsistent spacing between rows;
- buttons whose labels wrap on mobile;
- no visible focus state on icon-only actions.

Produce the design critique and completion gate.

## Expected Outcome

- Names the rendered screenshot and operational intent.
- Identifies hierarchy, density, typography, contrast/state, and interaction
  issues with evidence from the artifact.
- Recommends a coherent operational/workbench direction.
- Suggests bounded fixes rather than a new UI framework, component library, or
  global token engine.
- Calls out that `/a11y` or keyboard/focus verification is needed for icon-only
  actions.
- Emits a `## Design Gate` block.
