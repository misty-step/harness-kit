---
roster_review_due: 2026-06-30
---

# Pi Open-Model Roster Notes

Last researched: 2026-05-31.

Use this when choosing open-model defaults and variants for the Harness Kit Pi
roster lane. Treat this as an operating snapshot, not a permanent ranking.
Re-check model lists and live smokes before changing defaults.

## Current Default

| Lane | Default | Why |
|---|---|---|
| Pi | `openrouter/moonshotai/kimi-k2.5` | Clean Pi registry hit with thinking + tools and a 262K context window. |

Pi is the open-model CLI lane. Get independent failure modes by rotating Pi
models, not by adding another CLI wrapper. A second open-model provider lane
must earn entry with a live roster smoke, at least one real Harness Kit task,
and a clear failure mode that Pi model variants cannot cover.

## Model Notes

### Kimi

#### Default — K2.5

`moonshotai/kimi-k2.5` is the clean dispatch-floor default because Pi's native
model registry recognizes it and returns a warning-free smoke with thinking and
tools. The dispatch-floor default stays on K2.5 until a newer Kimi id resolves
cleanly in Pi's registry.

Use for: default Pi delegation, long-horizon coding, UI generation,
multi-agent decomposition, workflow synthesis.

Evidence: `backlog.d/_done/082-pi-roster-model-id-stale.md` and live Pi roster
smokes on 2026-05-29 and 2026-05-31.

#### Variant — K2.6

`moonshotai/kimi-k2.6` is newer and may be better for long-horizon coding, but
it remains an opt-in variant until Pi's registry resolves it without a
custom-model warning.

Current caveat: Pi did not list K2.6 during the 2026-05-26 filtered model
check. It may still answer through a custom OpenRouter id, but that warning is
not clean enough for the roster floor.

Sources: https://openrouter.ai/moonshotai/kimi-k2.6,
https://replicate.com/moonshotai/kimi-k2.6

### MiniMax

`minimax/minimax-m2.7` is the first rotation candidate. OpenRouter lists it
as released 2026-03-18 with 205K context, agentic workflow positioning, and
reported benchmark signals including SWE-Pro 56.2, Terminal Bench 2 57.0, and
GDPval-AA ELO 1495. Pi did not list it in the filtered model table but accepted
the model id as a custom OpenRouter id and returned a successful smoke.

Use for: general agentic productivity, document-heavy workflows, debugging and
root-cause work where we want a non-Kimi/non-DeepSeek vote.

Use as the alternate default when Kimi is degraded, too verbose, or too
expensive for the task mix.

Source: https://openrouter.ai/minimax/minimax-m2.7

### DeepSeek

`deepseek/deepseek-v4-pro` is the Pi long-context comparison variant.
OpenRouter lists 1M context, 1.6T total parameters, 49B active
parameters, and support for `high`/`xhigh` reasoning. NIST CAISI evaluated
DeepSeek V4 Pro in May 2026 and found it was the most capable PRC model CAISI
had evaluated, but that its aggregate capability lagged leading U.S. frontier
models by about 8 months; CAISI also found it cost-efficient on several
benchmarks relative to a U.S. reference model.

Use through Pi for: full-codebase analysis, large-context synthesis,
cost-sensitive reasoning, and a DeepSeek-family counterpoint to Kimi/MiniMax.

Sources: https://openrouter.ai/deepseek/deepseek-v4-pro,
https://www.nist.gov/news-events/news/2026/05/caisi-evaluation-deepseek-v4-pro

### Qwen

`qwen/qwen3.5-397b-a17b` is a major open-weight reference model, not the
current default. OpenRouter lists it as released 2026-02-16 with 262K context
and describes strong language, reasoning, code-generation, agent, image, video,
and GUI interaction capabilities. The family is broad and fast-moving; test
Qwen when multimodal/GUI interaction is first-order or when we need an Alibaba
lineage comparator.

Use for: multimodal/GUI-heavy work, broad agent evaluation, and self-hosting
experiments.

Source: https://openrouter.ai/qwen/qwen3.5-397b-a17b

### GLM / Z.ai

`z-ai/glm-5.1` is a credible long-horizon coding contender. OpenRouter lists it
as released 2026-04-07 with 203K context and describes 8+ hour autonomous
coding-task capability. Keep it in the candidate pool, but do not make it a
default until we have provider-specific tool-call smokes and at least one real
Harness Kit task comparison.

Use for: long-running coding/autonomy trials and model-diversity experiments.

Source: https://openrouter.ai/z-ai/glm-5.1

## Invoking A Variant

Use the committed Pi provider id and override only the model:

```sh
python3 scripts/dispatch-agent.py --provider-target pi --model-override long_context --objective "long-context review" --input-ref "path/or/ticket" --prompt-file /tmp/prompt.md
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
