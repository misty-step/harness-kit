# Visual Surface Routing

Use this reference when a diff touches likely UI or visual surfaces.

## Detection

Inspect the diff paths directly: `.tsx`/`.jsx`/`.vue`/`.svelte`, `pages/`,
`app/`, `routes/`, component directories, CSS/design tokens, docs/report
layouts, generated diagrams or site assets, screenshots, and decks all count
as visual surfaces. API routes living under UI frameworks are ambiguous —
treat a match as cheap extra review, not proof that pixels changed.

## Workflow Composition

For visual diffs, compose:

- `/design` for hierarchy, taste, visual intent, rendered artifact review,
  and accessibility (WCAG, keyboard, labels, focus) when the artifact is
  interactive or web-rendered.
- `/qa` for running-surface behavior, viewports, console/network checks, and
  screenshot or before/after evidence capture.
- `/code-review` for code quality and architecture, with UI findings grounded
  in evidence rather than style preference.

For non-visual diffs, do not force the design path. The detector should keep
workflow cost proportional to the change.

Some framework paths are ambiguous. API routes under UI frameworks may trigger
the detector; treat that as cheap extra review, not proof that pixels changed.

## Deliver

`/deliver` should check visual-surface paths before deciding whether QA is
skippable. If the diff touches a visual surface, the delivery includes a
`/design` pass (accessibility included) alongside the usual review, CI, and
QA judgment. When detection is ambiguous, inspect paths manually.

`/deliver --polish-only` uses the same detector during its precondition and
review passes. A visual branch is not ship-ready until design evidence is either
recorded or explicitly waived with a repo-fit reason.

## Evidence

Use the lightest proof that matches the change:

- static UI copy or docs page: screenshot at desktop and mobile widths;
- interactive component: screenshot plus interaction evidence or GIF;
- dashboard/workbench: screenshots showing dense states, empty state, and error
  state when touched;
- visual regression-prone repo: deterministic Playwright screenshots;
- uncertain visual critique: annotated screenshot from agent-browser or browser
  tooling.

Evidence can live in `/tmp/qa-{slug}/`, a PR description, a demo artifact, or
the phase receipt. Do not commit screenshots unless the repo already owns that
artifact class.
