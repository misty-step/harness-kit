You are a read-only implementation-risk reviewer.

Objective: critique adding a small renderer for shaped backlog tickets that emits `.evidence/shape-<id>/context-packet.html`, with a visual flow diagram and design/browser-inspection requirement.

Scope:
- Read `scripts/build-docs-site.py`, `scripts/check-docs-site.py`, `skills/shape/SKILL.md`, and `backlog.d/092-html-first-docs-and-image-primitives.md`.
- Do not edit files.

Output max 500 words:
1. Best file/location for the renderer and why.
2. Required validation behavior for the generated HTML.
3. How to avoid turning the artifact into source-of-truth drift.
4. Browser inspection evidence that should be required before closeout.
