You are a read-only test/oracle and implementation-risk reviewer shaping Harness Kit backlog 092.

Objective: define acceptance evidence that prevents docs-image slop, stale generated output, accessibility regressions, and accidental API/runtime dependencies.

Scope:
- Read `backlog.d/092-html-first-docs-and-image-primitives.md`.
- Inspect `scripts/check-docs-site.py`, `scripts/build-docs-site.py`, existing docs checks, and design eval patterns if useful.
- Check current official OpenAI docs only if you need to discuss model/API persistence; otherwise keep external facts out.
- Do not edit files.

Output, max 700 words:
1. Required negative tests/self-test cases.
2. Manifest fields that should be mandatory vs optional.
3. Acceptance artifact hashes or fixtures that should be named in the context packet.
4. Formal-spec ladder: required or not, and why.
5. Risks that should block implementation until the packet resolves them.
