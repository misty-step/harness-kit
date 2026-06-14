---
roster_review_due: 2026-06-15
---

# Open-Model Roster Notes

Last researched: 2026-06-14.

Use this when choosing open-model defaults and variants for Harness Kit roster
lanes. Treat this as a one-day operating snapshot. Re-check OpenRouter and run
live local smokes before each default change.

## Current Defaults

| Lane | Default | Why |
|---|---|---|
| Pi | `openrouter/moonshotai/kimi-k2.7-code` | Current coding-focused Kimi line on OpenRouter, 262K context, tool parameters, image input, and code positioning. |
| Goose | `openrouter/moonshotai/kimi-k2.7-code` | Bespoke on-machine agent surface with first-class OpenRouter provider support. |
| OpenCode | `openrouter/moonshotai/kimi-k2.7-code` | Bespoke open-source coding agent with built-in OpenRouter provider support and JSON event output. |

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

Pi stays the first open-model lane because Harness Kit already has dispatch
receipts and model override support for it. Roster dispatch uses
`--no-extensions` so optional personal Pi extensions cannot make a successful
model response exit nonzero. Pi also supports custom OpenAI-compatible
providers/models through `~/.pi/agent/models.json`.

Source: https://pi.dev/docs/latest/models.

### Goose

Goose is now a primary open-model harness candidate. Official docs list
OpenRouter as a supported provider requiring `OPENROUTER_API_KEY`, and the
local CLI exposes:

```sh
goose run --no-session --quiet --provider openrouter --model moonshotai/kimi-k2.7-code --text "task"
```

Source: https://block.github.io/goose/docs/getting-started/providers.

### OpenCode

OpenCode is now a primary open-model harness candidate. OpenRouter's official
integration docs say OpenCode supports OpenRouter as a built-in provider via
`/connect`, `/models`, or `opencode.json`, and accepts OpenRouter model ids
through the `openrouter/<model>` form.

Source: https://openrouter.ai/docs/cookbook/coding-agents/opencode-integration.

## Operating Rules

- Prefer a three-surface open-model spread for peer lanes: Pi, Goose, and
  OpenCode. The model family may be the same; the harness behavior is not.
- Promote a default only with: live OpenRouter catalog evidence, local binary
  probe, and at least one real Harness Kit smoke receipt.
- Keep model facts in `skills/roster/references/model-provider-harness-index.md`.
  Keep role-fit policy here and in shared doctrine.
- Do not add a new provider wrapper if Pi/Goose/OpenCode plus model variants
  cover the failure mode.
