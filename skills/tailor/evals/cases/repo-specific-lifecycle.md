# Case: repo-specific lifecycle plan

## Prompt

Run `/tailor` for a TypeScript CLI repository with:

- `package.json` exposing `bin: {"acme": "./dist/cli.js"}`
- `vitest.config.ts`
- no browser routes
- no deploy target
- recurring changelog and release-note work in git history
- registry-backed externals synced under `skills/.external/`

Produce only the skill portfolio and rewrite brief. Do not modify files.

## Expected Outcome

- Names four install buckets: workflow, universal, external, agents.
- Includes always-core workflow skills: research, groom, shape, implement,
  qa, demo, code-review, refactor, ci, diagnose, monitor, deliver, settle,
  ship, yeet, flywheel.
- Does not skip QA/demo/monitor because the repo lacks browser or deploy
  infrastructure.
- Defines QA as CLI smoke plus malformed-input checks.
- Defines demo as terminal transcript or release-note artifact.
- Defines monitor as CI, release smoke, benchmark drift, or issue/regression
  signal.
- Does not install deploy unless a real deploy/release surface is named.
- Picks zero frontend externals for this CLI repo.
- Includes stack-neutral externals such as `karpathy-*` and `julius-caveman`.
- External installs are absolute symlinks from shared root
  (`.agents/skills/<alias>` or `.agent/skills/<alias>`) to
  `skills/.external/<alias>`.
- External marker is sibling `<alias>.spellbook` (not inside symlink target),
  with category `external` and source/alias/target metadata.
- Per-harness bridges (`.claude/skills/`, `.codex/skills/`, `.pi/skills/`)
  are relative symlinks back to the shared root.
- Reconcile semantics: kept externals re-resolve symlink; dropped externals
  remove symlink + sibling marker; target content is never modified.
