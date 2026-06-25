# Render contract — surfaces & sync

One generation, multiple renders, no drift between them. "Synced docs" means
markdown source of truth → consistent HTML + diagrams, all from the same run —
not three hand-maintained copies.

## Markdown is the source of truth

- Committed under `docs/` (operator picks the exact root if not `docs/`).
- Portable, diffable, blame-able, travels with forks and offline clones.
- Front-matter provenance on every page (`templates/page.md`).
- Mermaid fenced blocks embedded inline — text, so they diff and the render
  oracle can parse them.

## HTML render

The browsable, beautiful surface. Do not hand-roll a static-site generator or a
bespoke aesthetic — **compose**:

- `/design` + `anthropic-frontend-design` for the visual system, typography, and
  avoiding templated-default tells.
- The "think in HTML" doctrine: layout, hierarchy, tables, diagrams, and
  callouts make the docs easier to inspect than prose.
- `/showcase`'s publish machinery if the docs become a public site.

Constraints: self-contained and navigable (sidebar/TOC, search if cheap),
mermaid rendered to inline SVG, internal links resolved. If the repo already has
a docs toolchain (mkdocs, Docusaurus, rustdoc, a Mintlify site), render into it
instead of inventing a parallel one — match the repo.

## Diagrams

- Mermaid in markdown → rendered in HTML. Architecture, sequence, data-flow,
  dependency, state — only where they earn their place
  (`references/information-architecture.md`).
- The render oracle fails the build on any unparseable diagram. No silent drop.

## Output layout

```
docs/
  index.md                 # root / overview, the entry page
  <section>/<page>.md      # IA tree (provenance-stamped)
  assets/                  # generated images/screenshots if any
```

The built HTML target is the operator's call — published site, a gitignored
local `_site/`, or a GitHub Pages branch. Default to local render pending an
explicit publish decision; never push a public site without operator sign-off.

## "Synced across surfaces"

Markdown, HTML, and (optionally) a GitHub Wiki mirror all derive from the one
generation. If mirroring to GitHub Wiki, flatten the page tree and rewrite
internal links — but the committed `docs/` markdown remains canonical; mirrors
are derived, never edited in place.
