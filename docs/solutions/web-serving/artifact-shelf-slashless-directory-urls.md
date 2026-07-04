---
title: "Redirect slashless directory artifact URLs before serving relative-link pages"
tags: [artifact-shelf, static-serving, slashless-url, relative-links, redirect-308, dot-paths]
module: "bastion/apps/artifacts"
problem_type: bug-track
applies_when:
  - "A static or artifact shelf serves generated HTML directories under both slash and slashless URLs."
  - "Pages use relative navigation, relative images, or nested assets."
  - "Published trees can include dot-prefixed normal components such as .github."
severity: high
date: 2026-07-04
repo_anchor: "misty-step/bastion@559d2791ab7fe59401a1b5f0612a8df37c3a7319"
pr: "misty-step/bastion#38"
---

## Context

Bastion's artifacts shelf served directory artifacts at slashless URLs such as
`/artifacts/a/site/crates`. Browser relative-link resolution treats that URL as
a file path, so child links and images resolve one level too high. The same
incident exposed an over-strict sanitizer: normal dot-prefixed path components
such as `.github/index.html` were rejected even though traversal through `.` or
`..` was already blocked by path component parsing.

## Learning

When serving generated HTML trees or artifact shelves, canonicalize directory
URLs to their trailing-slash form before serving `index.html`. Use a permanent
redirect for slashless directory requests, then serve the slash form directly.
Keep path sanitization strict against traversal, but allow dot-prefixed normal
components that real published sites use.

Do not verify this only at the shelf index. Exercise both the slashless URL and
the slash URL, then exercise at least one relative child path or dot-prefixed
tree path.

## Evidence

- Anchor: `misty-step/bastion@559d2791ab7fe59401a1b5f0612a8df37c3a7319`
- PR: `misty-step/bastion#38`
- Fix: `apps/artifacts/src/main.rs:167` redirects slashless directories with
  `Redirect::permanent`.
- Regression: `apps/artifacts/src/main.rs:691` checks slashless directory URLs
  redirect to `/artifacts/a/site/crates/`.
- Regression: `apps/artifacts/src/main.rs:715` checks `.github/index.html`
  publish/read round-trips.

## Retrieval Terms

artifact shelf, static serving, slashless URL, trailing slash, relative links,
redirect 308, dot path, `.github`, generated HTML, published site
