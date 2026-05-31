# Pi Harness Notes

Pi is the primary open-model roster lane for Harness Kit. Use it for independent
dynamic delegation when model diversity is more valuable than another
proprietary coding-agent opinion.

## Dispatch Shape

Use print mode with explicit provider/model/thinking/tool settings from the
roster:

```sh
pi -p --provider openrouter --model moonshotai/kimi-k2.5 --thinking xhigh --tools read,bash,edit,write,grep,find,ls "Role: investigator. Objective: inspect this oracle. Output: risks and proof."
```

The command stays a thin launch surface. `scripts/dispatch-agent.py` appends the
commission, applies the timeout, stores transcript evidence, and records the
receipt.

## Model Variants

Keep one Pi provider id and switch models inside Pi when the work needs another
open-model failure mode:

| Variant | Model | Use |
|---|---|---|
| `default` | `moonshotai/kimi-k2.5` | Clean roster dispatch, thinking + tools, 262K context. |
| `latest_kimi_custom` | `moonshotai/kimi-k2.6` | Opt-in only until Pi's registry resolves it without a custom-id warning. |
| `long_context` | `deepseek/deepseek-v4-pro` | Full-codebase or large-document analysis where context length dominates. |
| `alternate_agentic` | `minimax/minimax-m2.7` | Non-Kimi comparison for planning, debugging, and document-heavy work. |

Invoke variants through the same roster provider:

```sh
python3 scripts/dispatch-agent.py --provider-target pi --model-override long_context --objective "long-context review" --input-ref "path/or/ticket" --prompt-file /tmp/prompt.md
```

For direct one-off use, keep the same Pi shape and swap only `--model`.

## Dynamic Delegation Notes

- Use Pi for alternative implementation plans, research synthesis, and critique
  of assumptions.
- Give scoped paths, expected output, and explicit boundaries because open-model
  lanes can drift when the prompt is loose.
- Keep provider/model defaults in `.harness-kit/agents.yaml`; do not bake them
  into workflow skills.
- Pi settings are symlinked by bootstrap, so a fresh `bash bootstrap.sh` exposes
  the current `harnesses/pi/settings.json` default.
- Use the prompt shape `Role: ... Objective: ... Scope: ... Output: ...`.
- Run from the target workspace. Paths orient the lane; cwd is the workspace.
- Use Thinktank for multi-model Pi benches instead of hand-rolling parallel Pi
  commands.
- The lead verifies Pi output against files, commands, tests, and receipts
  before accepting it.
