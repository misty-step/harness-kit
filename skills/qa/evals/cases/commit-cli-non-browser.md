# Case: Commit QA non-browser path

## Prompt

Generate the per-commit QA scenario plan for a CLI/library-shaped change with:

- commit `def5678`
- changed files: `cmd/acme/main.go`, `pkg/render/render.go`
- README example: `acme render input.yaml`
- no browser app, route table, API server, or Playwright config
- expected operator behavior: the CLI renders a valid input and fails clearly
  on a missing file

## Expected Outcome

- App shape is CLI, library, or hybrid with an explicit CLI path.
- Uses shell commands such as `acme --help`, README invocation, and a
  malformed/missing-file check.
- Captures terminal transcript evidence under `.evidence/`.
- States that browser tooling is not the route for this repo.
- Does not force Playwright, browser-use, webVNC, or screenshots.
