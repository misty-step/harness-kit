# Adopt Lucide toolbox as the harness-kit wordmark icon

Priority: P2 · Status: open · Estimate: S

## Goal

Adopt the Lucide `toolbox` icon as the canonical wordmark/logo for
harness-kit — agent primitives — skills, doctrine, gates, evals, receipts.

The icon was selected through a visual design proposal playground
(see `aesthetic/prototypes/icon-logo-playground.html`, 2026-07-02)
as part of a unified Lucide-icon-as-logo system across the Misty Step
umbrella. All projects using the aesthetic design system converge on
Lucide icons for their logos; this is one of 12 in the set.

## Oracle

- [ ] The `toolbox` Lucide icon is rendered in the repo's primary
      identity surface (README hero, site header, or app chrome)
      using the aesthetic `.ae-icon` treatment (1.5px stroke, round
      caps, sized to ride alongside text).
- [ ] The icon is used consistently wherever the project identity
      appears: README, docs site, CLI help, generated artifacts.
- [ ] The icon source is the Lucide SVG, inlined or imported — no
      rasterized favicon-only adoption.
- [ ] In dark mode the icon uses `--ae-ink` (or `--ae-accent` if the
      project's steering calls for an accent logo); in light mode the
      same.

## Context

- Playground: `aesthetic/prototypes/icon-logo-playground.html`
- Lucide icon: https://lucide.dev/icons/toolbox
- Aesthetic icon treatment: `aesthetic.css` `.ae-icon` class
- Selected alongside: orbit (misty-step), aperture (aesthetic),
  flower (bitterblossom), flask-conical (crucible), kanban (powder),
  bird (canary), scroll-text (counterspell), toolbox (harness-kit),
  eye (cerberus), milestone (landmark), layers (weave), gauge (curb)
