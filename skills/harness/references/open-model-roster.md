# Open-Model Roster Notes

Last researched: 2026-05-26.

Use this when choosing complementary open-weight provider defaults for
Harness Kit roster lanes such as Pi and OpenCode. Treat this as an operating
snapshot, not a permanent ranking. Re-check model lists and live smokes before
changing defaults.

## Current Default

| Lane | Default | Why |
|---|---|---|
| Pi | `openrouter/moonshotai/kimi-k2.6` | Latest Kimi family member; live-smoked through Pi despite missing from Pi's filtered list. |
| OpenCode | `openrouter/deepseek/deepseek-v4-pro` | Long-context reasoning/coding complement to Pi's Kimi default; live-smoked through OpenCode. |

Do not pin both open lanes to the same family unless a live benchmark clearly
dominates. The point of the roster is independent failure modes.

## Model Notes

### Kimi

`moonshotai/kimi-k2.6` supersedes K2.5 for this harness. OpenRouter lists it
as released 2026-04-20 with 262K context, and Replicate describes K2.6 as a
1T-parameter model for long-horizon coding, agent swarms, and autonomous
software engineering. Replicate reports K2.6 benchmark signals including
Terminal-Bench 2.0 66.7, SWE-Bench Pro 58.6, SWE-Bench Verified 80.2, and
LiveCodeBench v6 89.6.

Use for: long-horizon coding, UI generation, multi-agent decomposition,
workflow synthesis.

Current caveat: Pi did not list K2.6 during the 2026-05-26 filtered model
check, but accepted the exact id as a custom OpenRouter id and returned a
successful smoke.

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

`deepseek/deepseek-v4-pro` is the OpenCode default and long-context comparison
lane. OpenRouter lists 1M context, 1.6T total parameters, 49B active
parameters, and support for `high`/`xhigh` reasoning. NIST CAISI evaluated
DeepSeek V4 Pro in May 2026 and found it was the most capable PRC model CAISI
had evaluated, but that its aggregate capability lagged leading U.S. frontier
models by about 8 months; CAISI also found it cost-efficient on several
benchmarks relative to a U.S. reference model.

Use for: full-codebase analysis, large-context synthesis, cost-sensitive
reasoning, and a DeepSeek-family counterpoint to Kimi/MiniMax.

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

## Operating Rules

- Prefer two complementary open lanes: one agentic-productivity model and one
  long-horizon coding model.
- Re-check provider model lists before each default change. Model family names
  drift faster than harness docs.
- A model-list hit is not enough. Required evidence is a live CLI smoke using
  the exact roster command and at least one real Harness Kit task before promotion.
- Record default changes in `.harness-kit/agents.yaml`, the relevant harness
  config, local config if needed, and this note.
