# Pi Harness Notes

Pi is the primary open-model roster lane for Harness Kit. Use it for independent
dynamic delegation when model diversity is more valuable than another
proprietary coding-agent opinion.

## Dispatch Shape

Use print mode with explicit provider/model/thinking/tool settings from the
roster:

pi -p --provider openrouter --model moonshotai/kimi-k2.7-code --thinking xhigh --tools read,bash,edit,write,grep,find,ls "Role: investigator. Objective: inspect this oracle. Output: risks and proof."
```

The command stays a thin launch surface. `harness-kit-checks dispatch-agent`
appends the commission, applies the timeout, stores transcript evidence, and
records the receipt.

## Model Variants

Keep one Pi provider id and switch models inside Pi when the work needs another
open-model failure mode:
| `default` | `moonshotai/kimi-k2.7-code` | Current Kimi roster dispatch target, thinking + tools, 256K context. |
| `previous_kimi` | `moonshotai/kimi-k2.6` | Previous Kimi default retained only for explicit comparison or rollback. |
| `thinking_kimi` | `moonshotai/kimi-k2-thinking` | Explicit thinking variant for hard reasoning lanes. |
| `long_context` | `deepseek/deepseek-v4-pro` | Full-codebase or large-document analysis where context length dominates. |
| `budget_long_context` | `deepseek/deepseek-v4-flash` | Ultra-cheap long-context for bulk analysis. |
| `alternate_agentic` | `minimax/minimax-m3` | Non-Kimi comparison for planning, debugging, and document-heavy work. |
| `qwen_coder` | `qwen/qwen3-coder-next` | Qwen coding model for additional model diversity. |

Invoke variants through the same roster provider:

```sh
cargo run --locked -p harness-kit-checks -- dispatch-agent --provider-target pi --model-override long_context --objective "long-context review" --input-ref "path/or/ticket" --prompt-file /tmp/prompt.md
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
- Use roster lanes for multi-model Pi benches instead of hand-rolling parallel
  Pi commands.
- The lead verifies Pi output against files, commands, tests, and receipts
  before accepting it.
