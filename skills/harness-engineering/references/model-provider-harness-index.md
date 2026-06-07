---
model_reference_review_due: 2026-06-14
last_researched: 2026-06-07
---

# Model / Provider / Harness Index

Factual context for composition design. This reference is evidence input for a
lead agent, not a routing policy. It must not prescribe role fit, preferred
team shapes, or "best for X" judgments. The lead agent chooses compositions
from the current task, current repo evidence, runtime probes, receipts, and
this factual sheet.

## Freshness Contract

- Review due: 2026-06-14.
- Treat model facts as stale after the review due date.
- Verify exact model ids, availability, prices, context windows, and benchmark
  claims from live provider docs or catalogs before changing defaults.
- Record local smoke evidence in delegation receipts; this file may point at
  receipts, but receipts remain the proof that a local harness invocation ran.

## Local Harness Roster

Source: `.harness-kit/agents.yaml`, probed with
`cargo run --locked -p harness-kit-checks -- probe-agent-roster` on 2026-06-07.

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

## Pi / OpenRouter Catalog Snapshot

Pi can attempt OpenRouter model ids through its configured dispatch surface:
`pi -p --provider openrouter --model <openrouter-id> ...`. The rows below are
OpenRouter catalog facts captured with
`curl -fsSL https://openrouter.ai/api/v1/models` on 2026-06-07. A row here
does not mean the model has been smoke-tested through Pi, and it is not a
recommendation. Record a delegation receipt before treating a non-roster model
as locally proven. OpenRouter rows describe OpenRouter listings only; do not
infer local Codex, Claude Code, Antigravity, Cursor, or Grok CLI pricing or
limits from them. `~...latest` ids are OpenRouter catalog aliases. Detailed
sections below carry extra source notes for selected rows; this table is the
scannable catalog snapshot.

| OpenRouter id | Created | Context | Max completion | Input | Output | Cache read | Modalities | Supported parameters excerpt |
|---|---:|---:|---:|---:|---:|---:|---|---|
| `~moonshotai/kimi-latest` | 2026-04-27 | 262,144 | 262,144 | `$0.684/M` | `$3.42/M` | `$0.144/M` | text+image -> text | `tools`, `tool_choice`, `parallel_tool_calls`, `structured_outputs`, `reasoning`, `reasoning_effort` |
| `moonshotai/kimi-k2.6` | 2026-04-20 | 262,144 | 262,144 | `$0.684/M` | `$3.42/M` | `$0.144/M` | text+image -> text | `tools`, `tool_choice`, `parallel_tool_calls`, `structured_outputs`, `reasoning`, `reasoning_effort` |
| `moonshotai/kimi-k2.6:free` | 2026-04-20 | 262,144 | unknown | `$0/M` | `$0/M` | unknown | text+image -> text | `tools`, `tool_choice`, `reasoning` |
| `moonshotai/kimi-k2.5` | 2026-01-27 | 262,144 | 262,144 | `$0.40/M` | `$1.90/M` | `$0.09/M` | text+image -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `deepseek/deepseek-v4-pro` | 2026-04-24 | 1,048,576 | 384,000 | `$0.435/M` | `$0.87/M` | `$0.003625/M` | text -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `deepseek/deepseek-v4-flash` | 2026-04-24 | 1,048,576 | 131,072 | `$0.0983/M` | `$0.1966/M` | `$0.0197/M` | text -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `minimax/minimax-m3` | 2026-05-31 | 1,048,576 | 512,000 | `$0.30/M` | `$1.20/M` | `$0.06/M` | text+image+video -> text | `tools`, `tool_choice`, `reasoning` |
| `minimax/minimax-m2.7` | 2026-03-18 | 204,800 | 196,608 | `$0.279/M` | `$1.20/M` | unknown | text -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `qwen/qwen3.7-plus` | 2026-06-03 | 1,000,000 | 65,536 | `$0.40/M` | `$1.60/M` | `$0.08/M` | text+image -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `qwen/qwen3.7-max` | 2026-05-21 | 1,000,000 | 65,536 | `$1.25/M` | `$3.75/M` | `$0.25/M` | text -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `qwen/qwen3.6-flash` | 2026-04-27 | 1,000,000 | 65,536 | `$0.1875/M` | `$1.125/M` | unknown | text+image+video -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `qwen/qwen3.5-397b-a17b` | 2026-02-16 | 262,144 | 65,536 | `$0.39/M` | `$2.34/M` | unknown | text+image+video -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `z-ai/glm-5.1` | 2026-04-07 | 202,752 | unknown | `$0.98/M` | `$3.08/M` | `$0.182/M` | text -> text | `tools`, `tool_choice`, `parallel_tool_calls`, `structured_outputs`, `reasoning`, `reasoning_effort` |
| `z-ai/glm-5v-turbo` | 2026-04-01 | 202,752 | 131,072 | `$1.20/M` | `$4.00/M` | `$0.24/M` | image+text+video -> text | `tools`, `tool_choice`, `reasoning` |
| `x-ai/grok-4.3` | 2026-04-30 | 1,000,000 | unknown | `$1.25/M` | `$2.50/M` | `$0.20/M` | text+image -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `x-ai/grok-4.20` | 2026-03-31 | 2,000,000 | unknown | `$1.25/M` | `$2.50/M` | `$0.20/M` | text+image+file -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `x-ai/grok-build-0.1` | 2026-05-20 | 256,000 | unknown | `$1.00/M` | `$2.00/M` | `$0.20/M` | text+image -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `openai/gpt-5.5` | 2026-04-24 | 1,050,000 | 128,000 | `$5.00/M` | `$30.00/M` | `$0.50/M` | file+image+text -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `anthropic/claude-opus-4.8` | 2026-05-27 | 1,000,000 | 128,000 | `$5.00/M` | `$25.00/M` | `$0.50/M` | text+image+file -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |

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
- OpenRouter pricing on 2026-06-07: input `$0.684/M`, output `$3.42/M`,
  cache read `$0.144/M`.
- OpenRouter modalities: text+image input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `parallel_tool_calls`, `structured_outputs`, `reasoning`, and
  `reasoning_effort`.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `moonshotai/kimi-k2.6` on 2026-06-07.

### Moonshot Kimi K2.5

- Retained local variant id: `openrouter/moonshotai/kimi-k2.5`.
- OpenRouter id: `moonshotai/kimi-k2.5`.
- OpenRouter created date: 2026-01-27.
- OpenRouter context length: 262,144 tokens.
- OpenRouter max completion tokens: 262,144.
- OpenRouter pricing on 2026-06-07: input `$0.40/M`, output `$1.90/M`,
  cache read `$0.09/M`.
- NVIDIA forum reports provider-specific K2.5 deprecation/replacement pressure
  around K2.6. Treat provider behavior as platform-specific until verified.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `moonshotai/kimi-k2.5` on 2026-06-07, plus
  https://forums.developer.nvidia.com/t/kimi-k2-5-replacement/368480.

### DeepSeek V4 Pro

- Local Pi variant id: `openrouter/deepseek/deepseek-v4-pro`.
- OpenRouter id: `deepseek/deepseek-v4-pro`.
- OpenRouter created date: 2026-04-24.
- OpenRouter context length: 1,048,576 tokens.
- OpenRouter max completion tokens: 384,000.
- OpenRouter pricing on 2026-06-07: input `$0.435/M`, output `$0.87/M`,
  cache read `$0.003625/M`.
- OpenRouter modalities: text input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `structured_outputs`, and `reasoning`.
- DeepSeek docs list `deepseek-v4-pro` with 1M context and pricing details;
  prior discount notes may have changed, so verify live before quoting
  non-OpenRouter prices.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `deepseek/deepseek-v4-pro` on 2026-06-07, and
  https://api-docs.deepseek.com/quick_start/pricing.

### MiniMax M2.7

- Local Pi variant id: `openrouter/minimax/minimax-m2.7`.
- OpenRouter id: `minimax/minimax-m2.7`.
- OpenRouter created date: 2026-03-18.
- OpenRouter context length: 204,800 tokens.
- OpenRouter top-provider context length: 196,608 tokens.
- OpenRouter max completion tokens: 196,608.
- OpenRouter pricing on 2026-06-07: input `$0.279/M`, output `$1.20/M`.
- OpenRouter modalities: text input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `structured_outputs`, and `reasoning`.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `minimax/minimax-m2.7` on 2026-06-07.

### Qwen3.5 397B A17B

- Candidate id: `openrouter/qwen/qwen3.5-397b-a17b`.
- OpenRouter id: `qwen/qwen3.5-397b-a17b`.
- OpenRouter created date: 2026-02-16.
- OpenRouter context length: 262,144 tokens.
- OpenRouter max completion tokens: 65,536.
- OpenRouter pricing on 2026-06-07: input `$0.39/M`, output `$2.34/M`.
- OpenRouter modalities: text+image+video input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `structured_outputs`, and `reasoning`.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `qwen/qwen3.5-397b-a17b` on 2026-06-07.

### Z.ai GLM 5.1

- Candidate id: `openrouter/z-ai/glm-5.1`.
- OpenRouter id: `z-ai/glm-5.1`.
- OpenRouter created date: 2026-04-07.
- OpenRouter context length: 202,752 tokens.
- OpenRouter pricing on 2026-06-07: input `$0.98/M`, output `$3.08/M`,
  cache read `$0.182/M`.
- OpenRouter modalities: text input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `parallel_tool_calls`, `structured_outputs`, `reasoning`, and
  `reasoning_effort`.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `z-ai/glm-5.1` on 2026-06-07.

### xAI Grok 4.3

- Active local id: `grok-4.3`.
- xAI docs page title includes Grok 4.3 under the Grok 4 model family.
- xAI docs describe external tool/system connection support for Grok 4.
- Source: https://docs.x.ai/developers/models/grok-4.
- Local CLI probe status on 2026-06-07: available.

### OpenAI GPT-5.5 Through Codex

- Active local id: `gpt-5.5`.
- Local dispatch surface: Codex CLI `codex exec --model gpt-5.5`.
- Source for local availability: `.harness-kit/agents.yaml` plus
  `probe-agent-roster` on 2026-06-07.
- Public model-card/pricing/context facts were not verified in this refresh.
  Do not infer pricing, context, or benchmark facts from the local model id.

### Google Gemini 3.5 Flash Through Antigravity

- Active local id: `gemini-3.5-flash`.
- Local dispatch surface: Antigravity CLI `agy --print`.
- Source for local availability: `.harness-kit/agents.yaml` plus
  `probe-agent-roster` on 2026-06-07.
- Public model-card/pricing/context facts were not verified in this refresh.
  Do not infer pricing, context, or benchmark facts from the local model id.

### Cursor Composer 2.5

- Active local id: `composer-2.5`.
- Local dispatch surface: Cursor Agent CLI `cursor-agent -p --model composer-2.5`.
- Source for local availability: `.harness-kit/agents.yaml` plus
  `probe-agent-roster` on 2026-06-07.
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
