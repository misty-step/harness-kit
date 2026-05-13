# Case: repo-specific lifecycle plan

## Prompt

Run `/tailor` for a TypeScript CLI repository with:

- `package.json` exposing `bin: {"acme": "./dist/cli.js"}`
- `vitest.config.ts`
- no browser routes
- no deploy target
- recurring changelog and release-note work in git history

Produce only the skill portfolio and rewrite brief. Do not modify files.

## Expected Outcome

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
- Invented skills, if any, include an eval seed.
