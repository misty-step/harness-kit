---
model_reference_review_due: 2026-06-13
last_researched: 2026-06-06
---

# Model / Provider / Harness Index

Factual context for composition design. This reference is evidence input for a
lead agent, not a routing policy. It must not prescribe role fit, preferred
team shapes, or "best for X" judgments. The lead agent chooses compositions
from the current task, current repo evidence, runtime probes, receipts, and
this factual sheet.

## Freshness Contract

- Review due: 2026-06-13.
- Treat model facts as stale after the review due date.
- Verify exact model ids, availability, prices, context windows, and benchmark
  claims from live provider docs or catalogs before changing defaults.
- Record local smoke evidence in delegation receipts; this file may point at
  receipts, but receipts remain the proof that a local harness invocation ran.

## Local Harness Roster

Source: `.harness-kit/agents.yaml`, probed with
`cargo run --locked -p harness-kit-checks -- probe-agent-roster` on 2026-06-06.

| Provider target | Harness / CLI | Active model id | Dispatch surface | Local probe status |
|---|---|---|---|---|
| `codex` | OpenAI Codex CLI | `gpt-5.5` | `codex exec --model gpt-5.5 --config model_reasoning_effort="medium"` | available |
| `claude` | Claude Code CLI | `claude-opus-4-8` | `claude -p --model claude-opus-4-8 --effort xhigh` | available |
| `pi` | Pi coding agent via OpenRouter | `openrouter/moonshotai/kimi-k2.6` | `pi -p --provider openrouter --model moonshotai/kimi-k2.6 --thinking xhigh` | available |
| `agy` | Antigravity CLI | `gemini-3.5-flash` | `agy --dangerously-skip-permissions --print` | available |
| `cursor-agent` | Cursor Agent CLI | `composer-2.5` | `cursor-agent -p --model composer-2.5` | available |
| `grok-build` | xAI Grok CLI | `grok-4.3` | `grok --model grok-4.3 --effort max --reasoning-effort xhigh -p` | available |
| `manual` | Human/imported evidence | none | manual summary | manual |

Local probe status proves only command discovery. It does not prove task
quality, current billing, tool-call reliability, or benchmark performance.

## Verified Model Facts

### Anthropic Claude Opus 4.8

- Active local id: `claude-opus-4-8`.
- Official API id: `claude-opus-4-8`.
- Release: 2026-05-28.
- Provider claim: Anthropic describes Opus 4.8 as its most capable generally
  available model at release.
- Context / output: Anthropic docs state Opus 4.8 supports 1M context on the
  Claude API, Amazon Bedrock, and Vertex AI; Microsoft Foundry lists 200k.
  Docs state 128k max output tokens.
- Platform surface: Anthropic docs state Opus 4.8 supports the same tools and
  platform features as Opus 4.7.
- Source: https://www.anthropic.com/news/claude-opus-4-8 and
  https://platform.claude.com/docs/en/about-claude/models/whats-new-claude-4-6.

### Moonshot Kimi K2.6

- Active local id: `openrouter/moonshotai/kimi-k2.6`.
- OpenRouter id: `moonshotai/kimi-k2.6`.
- OpenRouter created date: 2026-04-20.
- OpenRouter context length: 262,144 tokens.
- OpenRouter max completion tokens: 262,144.
- OpenRouter pricing on 2026-06-06: input `$0.684/M`, output `$3.42/M`,
  cache read `$0.144/M`.
- OpenRouter modalities: text+image input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `parallel_tool_calls`, `structured_outputs`, `reasoning`, and
  `reasoning_effort`.
- Source: `curl -s https://openrouter.ai/api/v1/models` filtered to
  `moonshotai/kimi-k2.6` on 2026-06-06.

### Moonshot Kimi K2.5

- Retained local variant id: `openrouter/moonshotai/kimi-k2.5`.
- OpenRouter id: `moonshotai/kimi-k2.5`.
- OpenRouter created date: 2026-01-27.
- OpenRouter context length: 262,144 tokens.
- OpenRouter max completion tokens: 262,144.
- OpenRouter pricing on 2026-06-06: input `$0.40/M`, output `$1.90/M`,
  cache read `$0.09/M`.
- NVIDIA forum reports provider-specific K2.5 deprecation/replacement pressure
  around K2.6. Treat provider behavior as platform-specific until verified.
- Source: `curl -s https://openrouter.ai/api/v1/models` filtered to
  `moonshotai/kimi-k2.5` on 2026-06-06, plus
  https://forums.developer.nvidia.com/t/kimi-k2-5-replacement/368480.

### DeepSeek V4 Pro

- Local Pi variant id: `openrouter/deepseek/deepseek-v4-pro`.
- OpenRouter id: `deepseek/deepseek-v4-pro`.
- OpenRouter created date: 2026-04-24.
- OpenRouter context length: 1,048,576 tokens.
- OpenRouter max completion tokens: 384,000.
- OpenRouter pricing on 2026-06-06: input `$0.435/M`, output `$0.87/M`,
  cache read `$0.003625/M`.
- OpenRouter modalities: text input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `structured_outputs`, and `reasoning`.
- DeepSeek docs list `deepseek-v4-pro` with 1M context and pricing details;
  prior discount notes may have changed, so verify live before quoting
  non-OpenRouter prices.
- Source: `curl -s https://openrouter.ai/api/v1/models` filtered to
  `deepseek/deepseek-v4-pro` on 2026-06-06, and
  https://api-docs.deepseek.com/quick_start/pricing.

### MiniMax M2.7

- Local Pi variant id: `openrouter/minimax/minimax-m2.7`.
- OpenRouter id: `minimax/minimax-m2.7`.
- OpenRouter created date: 2026-03-18.
- OpenRouter context length: 204,800 tokens.
- OpenRouter top-provider context length: 196,608 tokens.
- OpenRouter max completion tokens: 196,608.
- OpenRouter pricing on 2026-06-06: input `$0.279/M`, output `$1.20/M`.
- OpenRouter modalities: text input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `structured_outputs`, and `reasoning`.
- Source: `curl -s https://openrouter.ai/api/v1/models` filtered to
  `minimax/minimax-m2.7` on 2026-06-06.

### Qwen3.5 397B A17B

- Candidate id: `openrouter/qwen/qwen3.5-397b-a17b`.
- OpenRouter id: `qwen/qwen3.5-397b-a17b`.
- OpenRouter created date: 2026-02-16.
- OpenRouter context length: 262,144 tokens.
- OpenRouter max completion tokens: 65,536.
- OpenRouter pricing on 2026-06-06: input `$0.39/M`, output `$2.34/M`.
- OpenRouter modalities: text+image+video input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `structured_outputs`, and `reasoning`.
- Source: `curl -s https://openrouter.ai/api/v1/models` filtered to
  `qwen/qwen3.5-397b-a17b` on 2026-06-06.

### Z.ai GLM 5.1

- Candidate id: `openrouter/z-ai/glm-5.1`.
- OpenRouter id: `z-ai/glm-5.1`.
- OpenRouter created date: 2026-04-07.
- OpenRouter context length: 202,752 tokens.
- OpenRouter pricing on 2026-06-06: input `$0.98/M`, output `$3.08/M`,
  cache read `$0.182/M`.
- OpenRouter modalities: text input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `parallel_tool_calls`, `structured_outputs`, `reasoning`, and
  `reasoning_effort`.
- Source: `curl -s https://openrouter.ai/api/v1/models` filtered to
  `z-ai/glm-5.1` on 2026-06-06.

### xAI Grok 4.3

- Active local id: `grok-4.3`.
- xAI docs page title includes Grok 4.3 under the Grok 4 model family.
- xAI docs describe external tool/system connection support for Grok 4.
- Source: https://docs.x.ai/developers/models/grok-4.
- Local CLI probe status on 2026-06-06: available.

### OpenAI GPT-5.5 Through Codex

- Active local id: `gpt-5.5`.
- Local dispatch surface: Codex CLI `codex exec --model gpt-5.5`.
- Source for local availability: `.harness-kit/agents.yaml` plus
  `probe-agent-roster` on 2026-06-06.
- Public model-card/pricing/context facts were not verified in this refresh.
  Do not infer pricing, context, or benchmark facts from the local model id.

### Google Gemini 3.5 Flash Through Antigravity

- Active local id: `gemini-3.5-flash`.
- Local dispatch surface: Antigravity CLI `agy --print`.
- Source for local availability: `.harness-kit/agents.yaml` plus
  `probe-agent-roster` on 2026-06-06.
- Public model-card/pricing/context facts were not verified in this refresh.
  Do not infer pricing, context, or benchmark facts from the local model id.

### Cursor Composer 2.5

- Active local id: `composer-2.5`.
- Local dispatch surface: Cursor Agent CLI `cursor-agent -p --model composer-2.5`.
- Source for local availability: `.harness-kit/agents.yaml` plus
  `probe-agent-roster` on 2026-06-06.
- Public model-card/pricing/context facts were not verified in this refresh.
  Do not infer pricing, context, or benchmark facts from the local model id.

## Refresh Procedure

Use `/harness-engineering models` or `/model-research` when this file is stale
or a user asks for current model/provider/harness choices.

1. Read `.harness-kit/agents.yaml`, harness settings, and this file.
2. Probe local providers with `cargo run --locked -p harness-kit-checks -- probe-agent-roster`.
3. Query live provider catalogs/docs for exact model ids, context windows,
   max output, pricing, tool support, release dates, and deprecation notes.
4. Update this file with hard facts only.
5. Update `.harness-kit/agents.yaml` and harness settings only when changing a
   runnable default or variant.
6. Run `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`.

Do not add subjective labels such as role fit, taste, or task suitability to
this file. Put task-specific composition rationale in the run's receipts,
context packet, or final debrief.
