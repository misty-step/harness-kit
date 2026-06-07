# Claude Code Harness Notes

Claude Code is a primary roster lane for Harness Kit. Use it for independent
dynamic delegation when the lead is not already Claude.

## Dispatch Shape

Use print mode with permissions bypassed and pass one bounded commission as the
prompt:

```sh
claude -p --dangerously-skip-permissions --permission-mode bypassPermissions --model claude-opus-4-8 --effort xhigh "Role: reviewer. Objective: inspect this diff. Output: 5 bullets."
```

The roster entry should stay thin: launch Claude, provide the scoped prompt,
and let `harness-kit-checks dispatch-agent` capture transcript evidence and
write a sanitized receipt. Do not encode workflow semantics in the command.

## Dynamic Delegation Notes

- Commission the lane with role, objective, scope, output shape, and boundaries.
- Give only the relevant repo paths, diff, oracle, receipts, or logs.
- Ask Claude for concrete findings or decisions, not a general chat.
- Treat the output as evidence; the lead owns synthesis and verification.
- Record failed, rejected, or partially accepted attempts like successful ones.
