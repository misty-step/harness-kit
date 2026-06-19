---
roster_review_due: 2026-06-26
---

# Open-Model Roster Notes

Last researched: 2026-06-19.

Use this when choosing open-model defaults and variants for Harness Kit roster
lanes. Treat this as a one-day operating snapshot. Re-check OpenRouter and run
live local smokes before each default change.

Substrate positioning was refreshed from a 2026-06-19 coding-agent substrate
report. The model catalog rows below still depend on the 2026-06-14
OpenRouter snapshot unless a later live probe is cited.

## Current Defaults

| Lane | Default | Use first when |
|---|---|---|
| OpenCode | `openrouter/moonshotai/kimi-k2.7-code` | Code-centric review, diff analysis, event-stream capture, or future runner-adapter work where session/service shape matters. |
| Pi | `openrouter/moonshotai/kimi-k2.7-code` | Quick open-model peer lanes, model-family variants, and small decorrelated critiques where minimal harness overhead wins. |
| Goose | `openrouter/moonshotai/kimi-k2.7-code` | MCP-heavy workflows that span code plus trackers, docs, browsers, chat, or internal tools. |

Claude, Antigravity, Cursor, and Grok remain useful conditional tools. They are
not the default composition bias for Harness Kit peer lanes when a
smoke-tested open-model lane can answer the same question.

## Local Smoke Evidence

Sentinel objective: `open-model-roster-smoke`, expected output
`HARNESS_OPEN_MODEL_OK`, run through `harness-kit-checks dispatch-agent` on
2026-06-14.

| Lane | Receipt | Verdict | Note |
|---|---|---|---|
| Pi | `efd464ab-bed2-465c-9a89-b644822733ae` | succeeded | Passed after adding `--no-extensions`; previous attempt matched output but exited 1 due personal `ops-watchdog` extension. |
| Goose | `4f0b6928-7abc-4080-a0cb-1b195a7dd74a` | succeeded | `goose run --provider openrouter --model moonshotai/kimi-k2.7-code`. |
| OpenCode | `9601cf81-428f-4718-980f-15ee161b7b6e` | succeeded | `opencode run --model openrouter/moonshotai/kimi-k2.7-code --format json`. |

## Model Notes

### Kimi K2.7 Code

`moonshotai/kimi-k2.7-code` is the current open-model dispatch-floor default.
OpenRouter listed it on 2026-06-14 with:

- 262,144 context tokens.
- 262,144 max completion tokens.
- text+image input to text output.
- prompt `$0.75/M`, completion `$3.50/M`, cache read `$0.16/M` in the API
  catalog; the model page summarized `$0.95/M` input and `$4/M` output.
- supported parameters including tools, tool choice, structured outputs,
  reasoning, and response format.

Treat the price mismatch between the API catalog and model page as a live
provider drift signal. Quote prices from the catalog/page at dispatch time; do
not hard-code them into gates.

Sources: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
`moonshotai/kimi-k2.7-code` on 2026-06-14, and
https://openrouter.ai/moonshotai/kimi-k2.7-code.

### Kimi rollback and reasoning variants

- `moonshotai/kimi-k2.6` remains `previous_kimi` for rollback and A/B checks.
- `moonshotai/kimi-k2-thinking` remains `thinking_kimi` when the lead wants
  the Kimi family but a different reasoning surface.

Do not restore K2.6 as default without a fresh OpenRouter catalog check and a
local task smoke.

### DeepSeek

- `deepseek/deepseek-v4-pro` remains `long_context`: OpenRouter listed 1,048,576
  context tokens, 384,000 max completion tokens, tools, structured outputs,
  and reasoning on 2026-06-14.
- `deepseek/deepseek-v4-flash` is `budget_long_context`: same 1M context class,
  lower catalog price, smaller max completion.

Use through Pi/Goose/OpenCode when long context or cheap large-context review
matters more than Kimi-family continuity.

### MiniMax

`minimax/minimax-m3` is the `alternate_agentic` candidate. OpenRouter listed it
on 2026-06-14 with 1,048,576 context tokens, 512,000 max completion tokens,
text+image+video input, tools, structured outputs, and reasoning. Prefer it
over stale M2.x defaults unless a smoke shows a regression.

### Qwen

`qwen/qwen3-coder-next` is `qwen_coder`: a coding-family comparator with 262K
context and tool parameters in the 2026-06-14 OpenRouter catalog. Use it when
we need a non-Kimi, non-DeepSeek coding lane.

## Harness Notes

### Pi

Pi stays the smallest open-model peer lane because Harness Kit already has
dispatch receipts and model override support for it. Roster dispatch uses
`--no-extensions` so optional personal Pi extensions cannot make a successful
model response exit nonzero. Pi also supports custom OpenAI-compatible
providers/models through `~/.pi/agent/models.json`.

Source: https://pi.dev/docs/latest/models.

### Goose

Goose is a primary open-model harness candidate for MCP-heavy work. Official
docs list OpenRouter as a supported provider requiring `OPENROUTER_API_KEY`,
and the local CLI exposes:

```sh
goose run --no-session --quiet --provider openrouter --model moonshotai/kimi-k2.7-code --text "task"
```

Source: https://block.github.io/goose/docs/getting-started/providers.

### OpenCode

OpenCode is the preferred open substrate candidate for code-centric review
runner experiments. OpenRouter's official integration docs say OpenCode
supports OpenRouter as a built-in provider via `/connect`, `/models`, or
`opencode.json`, and accepts OpenRouter model ids through the
`openrouter/<model>` form. The 2026-06-19 substrate report's core distinction:
OpenCode is session/service-shaped, which fits coordinator/specialist review
lanes and structured event collection better than wrapping terminal-first tools.

Source: https://openrouter.ai/docs/cookbook/coding-agents/opencode-integration.

## Operating Rules

- Prefer OpenCode first for code-review substrate experiments; prefer Goose
  first for cross-system MCP workflows; prefer Pi first for small, cheap,
  decorrelated peer critiques. The model family may be the same; the harness
  behavior is not.
- Promote a default only with: live OpenRouter catalog evidence, local binary
  probe, and at least one real Harness Kit smoke receipt.
- Keep model facts in `skills/roster/references/model-provider-harness-index.md`.
  Keep role-fit policy here and in shared doctrine.
- Do not add a new provider wrapper if Pi/Goose/OpenCode plus model variants
  cover the failure mode.
- Do not treat any CLI as a production control plane. Durable queueing,
  sandboxing, policy, publication credentials, budget/circuit breakers, and
  eval storage live outside the per-job agent kernel.
