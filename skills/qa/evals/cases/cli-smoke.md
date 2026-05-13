# Case: CLI smoke QA

## Prompt

Run QA for a Go CLI repository with:

- `cmd/acme/main.go`
- `go test ./...` already passing
- README examples for `acme render input.yaml`
- no browser, API server, or Playwright config

Describe the QA path and evidence to capture.

## Expected Outcome

- Does not skip QA.
- Does not use browser tooling.
- Runs or requests the CLI help and representative README invocation.
- Includes malformed-input or missing-file checks.
- Captures terminal transcript evidence.
- States that passing `go test ./...` is not sufficient QA.
