# UI Surface Routing

Use this reference when a diff touches likely UI surfaces.

## Detector

Use `scripts/detect-ui-surfaces.sh` as an optional standardized routing
signal:

```bash
scripts/detect-ui-surfaces.sh --staged
scripts/detect-ui-surfaces.sh --unstaged
scripts/detect-ui-surfaces.sh --base <repo-default-base>
scripts/detect-ui-surfaces.sh --paths app/page.tsx components/Button.tsx
```

It prints JSON:

```json
{"ui_surface":true,"mode":"paths","matches":["components/Button.tsx"]}
```

`ui_surface: true` means run the design path. It does not mean the design is
good or bad. If the helper is unavailable or cannot resolve the base ref,
inspect the changed paths manually with the same pattern set.

## Workflow Composition

For UI diffs, compose:

- `/design` for hierarchy, taste, visual intent, and rendered artifact review.
- `/a11y` for WCAG, keyboard, labels, focus, and screen-reader behavior.
- `/qa` or `/browser` for running-surface behavior, viewports, console, and
  network checks.
- `/demo` for screenshots, GIFs, or before/after evidence when the change is
  user-visible.
- `/code-review` for code quality and architecture, with UI findings grounded
  in evidence rather than style preference.

For non-UI diffs, do not force the UI path. The detector should keep workflow
cost proportional to the change.

Some framework paths are ambiguous. API routes under UI frameworks may trigger
the detector; treat that as cheap extra review, not proof that pixels changed.

## Deliver And Settle

`/deliver` should check UI-surface paths before deciding whether QA is
skippable. Prefer the detector helper when available; if the detector returns
`ui_surface:true`, the clean loop includes `/design` and `/a11y` in addition
to the usual review, CI, refactor, and QA phases. If the detector errors, fall
back to manual path inspection.

`/deliver --polish-only` uses the same detector during its precondition and
review passes. A UI branch is not ship-ready until design/a11y evidence is
either recorded or explicitly waived with a repo-fit reason.

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
