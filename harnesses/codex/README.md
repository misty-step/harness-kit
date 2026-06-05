# Codex CLI Harness Notes

Codex CLI is a primary roster lane for Harness Kit. Use it for independent
dynamic delegation when the lead is not already Codex, or when a separate
Codex process/worktree gives useful isolation.

## Dispatch Shape

Use non-interactive exec with the configured model and reasoning effort:

```sh
codex exec --dangerously-bypass-approvals-and-sandbox --model gpt-5.5 --config model_reasoning_effort="medium" "Role: critic. Objective: review the changed files. Output: blockers only."
```

The roster command launches the provider only. `harness-kit-checks
dispatch-agent` appends the prompt, bounds runtime, captures transcript
evidence, and records the receipt.

## Dynamic Delegation Notes

- Prefer isolated worktrees for competing implementation attempts.
- Keep prompts commissioned: role, objective, scope, output, boundaries.
- Give the shaped ticket, exact files, and commands to inspect; avoid hidden
  lead reasoning.
- Reject outputs that hard-code, hand-wave verification, or ignore boundaries.
- The lead remains responsible for final code, tests, and acceptance evidence.
