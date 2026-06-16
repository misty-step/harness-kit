# Loop engineering and HarnessX placement lane

Provider target: `opencode`
Delegation ID: `6f3eab49-bc12-44ad-92eb-70b8892932c3`
Runtime receipt: `.harness-kit/traces/provider-lanes/20260616T204302.444723Z-opencode-f1dc30ef.txt`

## Accepted Evidence

- Loop engineering belongs across the Mode A/Mode B boundary: Harness Kit owns
  loop design, readiness checks, skills, gates, lane cards, and state contracts;
  Bitterblossom owns scheduled/event triggers and unattended workers.
- HarnessX is too speculative for core Harness Kit. The safe shape is a
  review-only evaluation using sanitized traces, held-out tasks, and
  human-reviewed diffs.
- Both packets must preserve hard stops: max iterations, no-progress detection,
  token/dollar budget, and fresh-context verifier separation.

## Resulting Packets

- `backlog.d/109-loop-engineering-mode-b-bridge.md`
- `backlog.d/110-harnessx-evolution-eval.md`

