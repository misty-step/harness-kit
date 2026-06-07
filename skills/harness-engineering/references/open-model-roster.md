---
roster_review_due: 2026-06-13
---

# Pi Open-Model Roster Notes

Last researched: 2026-06-06.

Use this when choosing open-model defaults and variants for the Harness Kit Pi
roster lane. Treat this as an operating snapshot, not a permanent ranking.
Re-check model lists and live smokes before changing defaults.

## Current Default

| Lane | Default | Why |
|---|---|---|
| Pi | `openrouter/moonshotai/kimi-k2.6` | Current Kimi model line in the local roster, with OpenRouter listing 262K context and tool parameters. |

Pi is the open-model CLI lane. Get independent failure modes by rotating Pi
models, not by adding another CLI wrapper. A second open-model provider lane
must earn entry with a live roster smoke, at least one real Harness Kit task,
and a clear failure mode that Pi model variants cannot cover.

## Model Notes

### Kimi

#### Current default — K2.6

`moonshotai/kimi-k2.6` is the current dispatch-floor default. OpenRouter listed
it on 2026-06-06 with 262,144 context tokens, 262,144 max completion tokens,
text+image input, tools, structured outputs, and reasoning parameters.

Evidence source: `curl -s https://openrouter.ai/api/v1/models` filtered to
`moonshotai/kimi-k2.6` on 2026-06-06.

#### Previous default — K2.5

`moonshotai/kimi-k2.5` remains as `previous_kimi` for explicit comparison or
rollback. OpenRouter listed it on 2026-06-06 with the same 262,144 context
window, lower listed token prices, and no `parallel_tool_calls` parameter in
the model record returned by the API.

Provider-specific K2.5 deprecation/replacement behavior has been reported; do
not restore K2.5 as default without a fresh model-catalog check and local Pi
smoke.

Sources: https://openrouter.ai/moonshotai/kimi-k2.6,
https://forums.developer.nvidia.com/t/kimi-k2-5-replacement/368480

### MiniMax

`minimax/minimax-m2.7` is the first rotation candidate. OpenRouter lists it
as released 2026-03-18 with 205K context, agentic workflow positioning, and
reported benchmark signals including SWE-Pro 56.2, Terminal Bench 2 57.0, and
GDPval-AA ELO 1495. Pi did not list it in the filtered model table but accepted
the model id as a custom OpenRouter id and returned a successful smoke.

Use as an alternate open-model comparison when the lead wants a non-Kimi and
non-DeepSeek result.

Source: https://openrouter.ai/minimax/minimax-m2.7

### DeepSeek

`deepseek/deepseek-v4-pro` is the Pi long-context comparison variant.
OpenRouter lists 1M context, 1.6T total parameters, 49B active
parameters, and support for `high`/`xhigh` reasoning. NIST CAISI evaluated
DeepSeek V4 Pro in May 2026 and found it was the most capable PRC model CAISI
had evaluated, but that its aggregate capability lagged leading U.S. frontier
models by about 8 months; CAISI also found it cost-efficient on several
benchmarks relative to a U.S. reference model.

Use through Pi when the lead explicitly wants a DeepSeek-family comparison.

Sources: https://openrouter.ai/deepseek/deepseek-v4-pro,
https://www.nist.gov/news-events/news/2026/05/caisi-evaluation-deepseek-v4-pro

### Qwen

`qwen/qwen3.5-397b-a17b` is a major open-weight reference model, not the
current default. OpenRouter lists it as released 2026-02-16 with 262K context
and describes strong language, reasoning, code-generation, agent, image, video,
and GUI interaction capabilities. The family is broad and fast-moving; test
Qwen when multimodal/GUI interaction is first-order or when we need an Alibaba
lineage comparator.

Keep in the candidate pool for multimodal, GUI, or self-hosting experiments.

Source: https://openrouter.ai/qwen/qwen3.5-397b-a17b

### GLM / Z.ai

`z-ai/glm-5.1` is a credible long-horizon coding contender. OpenRouter lists it
as released 2026-04-07 with 203K context and describes 8+ hour autonomous
coding-task capability. Keep it in the candidate pool, but do not make it a
default until we have provider-specific tool-call smokes and at least one real
Harness Kit task comparison.

Keep in the candidate pool for long-running autonomy and model-diversity
experiments.

Source: https://openrouter.ai/z-ai/glm-5.1

## Invoking A Variant

Use the committed Pi provider id and override only the model:

```sh
cargo run --locked -p harness-kit-checks -- dispatch-agent --provider-target pi --model-override long_context --objective "long-context review" --input-ref "path/or/ticket" --prompt-file /tmp/prompt.md
```

`--model-override` accepts a key from `.harness-kit/agents.yaml` `model_variants`
or a direct model id. The receipt stays attached to provider `pi` and records
the resolved model override in its summary.

## Operating Rules

- Prefer one Pi lane with complementary model variants: one clean default, one
  long-context model, and one non-Kimi agentic-productivity model.
- Re-check provider model lists before each default change. Model family names
  drift faster than harness docs.
- A model-list hit is not enough. Required evidence is a live CLI smoke using
  the exact roster command and at least one real Harness Kit task before promotion.
- Record default changes in `.harness-kit/agents.yaml`, the relevant harness
  config, local config if needed, and this note.
