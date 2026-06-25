---
model_reference_review_due: 2026-06-15
last_researched: 2026-06-14
substrate_reference_review_due: 2026-06-26
substrate_reference_last_researched: 2026-06-19
speech_reference_review_due: 2026-06-27
speech_reference_last_researched: 2026-06-20
---

# Model / Provider / Harness Index

Factual context for composition design. This reference is evidence input for a
lead agent, not a routing policy. It must not prescribe role fit, preferred
team shapes, or "best for X" judgments. The lead agent chooses compositions
from the current task, current repo evidence, runtime probes, receipts, and
this factual sheet.

## Freshness Contract

- Review due: 2026-06-15.
- Treat model facts as stale after the review due date.
- Verify exact model ids, availability, prices, context windows, and benchmark
  claims from live provider docs or catalogs before changing defaults.
- Record local smoke evidence in delegation receipts; this file may point at
  receipts, but receipts remain the proof that a local harness invocation ran.

## Live model facts: the OpenRouter MCP

The OpenRouter MCP is the live source for everything in this file that rots.
It is a remote HTTP server, user-scoped so it is available in every repo:

```sh
claude mcp add --scope user --transport http openrouter https://mcp.openrouter.ai/mcp
claude mcp login openrouter   # one-time OAuth (browser); issues a 7-day, $10-cap key
```

Tools: `models-list` / `model-get` / `model-endpoints` (catalog, providers,
live pricing), `benchmarks` (third-party quality scores), `rankings-daily`,
`credits-get` (balance), `generation-get` (per-call cost/tokens), `chat-send`
(billable test inference), `docs-search`, `ping`. Use it to verify slugs,
prices, and context windows before changing defaults, and to compose a
`/council` bench from current top diverse families. Quote prices at dispatch
time; never hardcode them into gates.

## Local Harness Roster

Source: `.harness-kit/agents.yaml`, probed with
`cargo run --locked -p harness-kit-checks -- probe-agent-roster` on 2026-06-14.

| Provider target | Harness / CLI | Active model id | Dispatch surface | Local probe status |
|---|---|---|---|---|
| `codex` | OpenAI Codex CLI | `gpt-5.5` | `codex exec --model gpt-5.5 --config model_reasoning_effort="medium"` | available |
| `pi` | Pi coding agent via OpenRouter | `openrouter/moonshotai/kimi-k2.7-code` | `pi -p --no-extensions --provider openrouter --model moonshotai/kimi-k2.7-code --thinking xhigh` | available |
| `goose` | Goose CLI via OpenRouter | `openrouter/moonshotai/kimi-k2.7-code` | `goose run --provider openrouter --model moonshotai/kimi-k2.7-code --text` | available |
| `opencode` | OpenCode CLI via OpenRouter | `openrouter/moonshotai/kimi-k2.7-code` | `opencode run --model openrouter/moonshotai/kimi-k2.7-code --variant max --format json` | available |
| `claude` | Claude Code CLI | `claude-opus-4-8` | `claude -p --model claude-opus-4-8 --effort xhigh` | available |
| `agy` | Antigravity CLI | `gemini-3.5-flash` | `agy --dangerously-skip-permissions --print` | available |
| `cursor-agent` | Cursor Agent CLI | `composer-2.5` | `cursor-agent -p --model composer-2.5` | available |
| `grok-build` | xAI Grok CLI | `grok-4.3` | `grok --model grok-4.3 --effort max --reasoning-effort xhigh -p` | available |
| `oracle` | Oracle browser consult | `gpt-5.5-pro-browser` | `npx -y @steipete/oracle --engine browser --model gpt-5.5-pro -p` | available via `npx`; dry-run smoke passed 2026-06-16 |
| `manual` | Human/imported evidence | none | manual summary | manual |

Local probe status proves only command discovery. It does not prove task
quality, current billing, tool-call reliability, or benchmark performance.
Oracle status proves only the browser-mode dry-run path; Harness Kit forbids
Oracle API mode in its skill and roster defaults.

## Realtime And Speech Substrate Snapshot

Source: primary provider docs checked on 2026-06-20. This section is factual
input for product boundary decisions; it is not a default-provider policy.

OpenAI:

- Realtime guide positions `gpt-realtime-2` for low-latency voice agents and
  `gpt-realtime-whisper` for streaming transcription.
- Realtime conversations support function calling and out-of-band responses
  (`conversation: "none"`), which fits side-channel classification/proposal
  work that should not speak into the main conversation.
- `gpt-4o-transcribe-diarize` supports `diarized_json` speaker-aware segments
  through `/v1/audio/transcriptions`; OpenAI docs state it is not yet supported
  in the Realtime API.
- Sources:
  <https://developers.openai.com/api/docs/guides/realtime>,
  <https://developers.openai.com/api/docs/guides/realtime-conversations>,
  <https://developers.openai.com/api/docs/guides/speech-to-text#speaker-diarization>.

Google Gemini:

- Gemini Live API supports low-latency realtime voice/vision interactions,
  tool use, audio transcriptions, and proactive audio controls.
- Gemini model docs list Gemini 3.1 Flash Live Preview for high-quality
  low-latency audio-to-audio dialogue and Gemini 2.5 Flash Live Preview for
  low-latency bidirectional voice/video agents with native audio reasoning.
- Sources:
  <https://ai.google.dev/gemini-api/docs/live-api>,
  <https://ai.google.dev/gemini-api/docs/models>.

Deepgram:

- Flux is positioned as conversational speech recognition for voice agents with
  model-integrated end-of-turn detection, configurable turn-taking dynamics, and
  ultra-low latency.
- Source: <https://developers.deepgram.com/docs/models-languages-overview>.

ElevenLabs:

- Scribe v2 supports speech recognition across 90+ languages, word timestamps,
  dynamic audio tagging, and speaker diarization up to 32 speakers.
- Scribe v2 Realtime is documented for realtime low-latency transcription and
  word-level timestamps; verify current diarization support separately before
  relying on realtime speaker labels.
- Source: <https://elevenlabs.io/docs/overview/capabilities/speech-to-text>.

Design implication for AI-first meeting products: deterministic code should
own approval, policy, event logs, sandboxing, schemas, and evals; semantic
proposal generation, speech understanding, and diarization should be shaped
against a model/provider capability surface first, with deterministic
heuristics only as explicit fallback or fixture paths.

## Substrate Assessment Addendum

Source: user-provided research report, "Modern coding-agent systems as
execution substrates", current through 2026-06-19.

Factual substrate distinctions to preserve in composition design:

- OpenCode is server/session-shaped: clients sit on top of a service with
  programmatic sessions, SDK/API access, plugins, tools, permissions, MCP, and
  structured event output. It is the strongest open candidate for custom
  code-centric review runners, but it is not a durable queue, sandbox, policy
  service, publisher, model gateway, or eval warehouse by itself.
- Goose is a portable MCP-driven workflow runtime with CLI/headless operation,
  recipes, subagents, broad provider support, and a large MCP extension
  surface. Prefer it when a lane spans code plus trackers, docs, browsers,
  chat, or internal tools.
- Pi is a minimal coding-agent harness with CLI/RPC/SDK, tree sessions,
  provider adapters, extensions, and skills. It is valuable for local
  hackability and peer lanes, but its default process permissions and lack of a
  built-in permission system mean unattended production use needs an external
  sandbox and control plane.
- OMP is a heavier local engineering environment built around Pi-style
  workflows, with LSP/debugger/worktree/subagent affordances. Treat it as an
  expert local surface, not the organization-wide production control plane.
- OpenHands is the heavier open self-hosted platform option for remote
  workspaces and issue-to-PR work; use it when operating a multi-user agent
  platform is the point, not for a lightweight review-only lane.
- Continue's official repository was reported read-only / no longer actively
  maintained; do not make it a new strategic dependency without fresh contrary
  evidence.
- Managed review products such as Cursor Bugbot, Greptile, CodeRabbit, Codex
  review, and Copilot are bake-off comparators before building commodity review
  machinery.

The security boundary is external to every row above: do not put model-provider
keys or GitHub write credentials inside a sandbox that can execute
repository-controlled code.

Kimi K2.7 Code sentinel dispatch receipts on 2026-06-14:

- Pi: `efd464ab-bed2-465c-9a89-b644822733ae`, succeeded after roster command
  added `--no-extensions`.
- Goose: `4f0b6928-7abc-4080-a0cb-1b195a7dd74a`, succeeded.
- OpenCode: `9601cf81-428f-4718-980f-15ee161b7b6e`, succeeded.

## Focused Lane Harness Projection

Roster dispatch can optionally use a `lane_harness.v1` manifest to project a
minimal child harness before launching a provider. This is for context hygiene:
the primary lead can give a lane only the local skills and external aliases
needed for its role instead of inheriting every globally installed Harness Kit
skill.

Use it when a lane has a narrow responsibility and extra skills would be
misleading, such as a CI-only critic, a docs-only verifier, or an implementation
lane that should not see shaping or grooming skills. Do not use it as a semantic
workflow engine, a permission system, or a substitute for the lead's judgment.

Minimum operating path:

```sh
cargo run --locked -p harness-kit-checks -- materialize-lane-harness \
  --manifest crates/harness-kit-checks/tests/fixtures/lane-harness.yaml

cargo run --locked -p harness-kit-checks -- dispatch-agent \
  --provider-target codex \
  --objective "bounded lane objective" \
  --input-ref "path/or/ticket" \
  --prompt-file /tmp/lane.md \
  --repo . \
  --lane-harness crates/harness-kit-checks/tests/fixtures/lane-harness.yaml
```

Manifest constraints:

- `provider_target` must match a provider id in the roster and the dispatch
  provider target.
- `model_override`, when present, must match the provider's roster model or one
  of its configured `model_variants` keys or values.
- `allowed_local_skills` must name existing first-party skills and cannot
  escape the repo `skills/` root.
- `allowed_external_aliases` must resolve to pinned aliases in `registry.yaml`.
- `fallback.on_provider_failure` is `record_and_return`; a failed Codex, Pi,
  Goose, OpenCode, Claude, Antigravity, Cursor, or Grok lane should produce
  evidence for the lead, not crash the whole composition.
- `fallback.replacement_policy` is `lead_explicit`; replacing a failed lane is
  a lead decision, not an automatic provider loop.

Runtime projection creates an ignored root under
`.harness-kit/tmp/lane-harness/<id>/`, links the allowed skills into the
known harness skill locations, sets child environment variables (`HOME`,
`CODEX_HOME`, `CLAUDE_CONFIG_DIR`, `PI_HOME`, `GEMINI_CONFIG_DIR`,
`GOOSE_CONFIG_DIR`, `OPENCODE_CONFIG_DIR`, `XDG_CONFIG_HOME`), and removes the
root after dispatch unless
`--keep-lane-root` is supplied for debugging.

Receipt fields make projection auditable:

- `lane_harness_ref`: manifest path.
- `lane_harness_sha256`: manifest hash at dispatch time.
- `projection_status`: `projected` or `failed`.
- `failure_kind`: typed provider or projection failure such as
  `credits_exhausted`, `auth_required`, `missing_binary`, `probe_timeout`,
  `dispatch_timeout`, `nonzero_exit`, `sentinel_mismatch`, or
  `projection_failed`.
- `output_check`: optional sentinel verdict when `--expect-output` is used.

## Open-Model / OpenRouter Catalog Snapshot

Pi, Goose, and OpenCode can attempt OpenRouter model ids through their
configured dispatch surfaces. The rows below are OpenRouter catalog facts
captured with `curl -fsSL https://openrouter.ai/api/v1/models` on 2026-06-14.
A row here does not mean the model has been smoke-tested through every harness,
and it is not a recommendation. Record a delegation receipt before treating a
non-roster model as locally proven. OpenRouter rows describe OpenRouter
listings only; do not infer local Codex, Claude Code, Antigravity, Cursor, or
Grok CLI pricing or limits from them. `~...latest` ids are OpenRouter catalog
aliases. Detailed sections below carry extra source notes for selected rows;
this table is the scannable catalog snapshot.

| OpenRouter id | Created | Context | Max completion | Input | Output | Cache read | Modalities | Supported parameters excerpt |
|---|---:|---:|---:|---:|---:|---:|---|---|
| `moonshotai/kimi-k2.7-code` | 2026-06-12 | 262,144 | 262,144 | `$0.75/M` | `$3.50/M` | `$0.16/M` | text+image -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning`, `response_format` |
| `~moonshotai/kimi-latest` | 2026-04-27 | 262,144 | 262,142 | `$0.68/M` | `$3.41/M` | `$0.34/M` | text+image -> text | `tools`, `tool_choice`, `parallel_tool_calls`, `structured_outputs`, `reasoning`, `reasoning_effort` |
| `moonshotai/kimi-k2.6` | 2026-04-20 | 262,144 | 262,142 | `$0.68/M` | `$3.41/M` | `$0.34/M` | text+image -> text | `tools`, `tool_choice`, `parallel_tool_calls`, `structured_outputs`, `reasoning`, `reasoning_effort` |
| `moonshotai/kimi-k2.6:free` | 2026-04-20 | 262,144 | unknown | `$0/M` | `$0/M` | unknown | text+image -> text | `tools`, `tool_choice`, `reasoning` |
| `moonshotai/kimi-k2.5` | 2026-01-27 | 262,144 | unknown | `$0.375/M` | `$2.025/M` | unknown | text+image -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `deepseek/deepseek-v4-pro` | 2026-04-24 | 1,048,576 | 384,000 | `$0.435/M` | `$0.87/M` | `$0.003625/M` | text -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `deepseek/deepseek-v4-flash` | 2026-04-24 | 1,048,576 | 65,536 | `$0.09/M` | `$0.18/M` | `$0.02/M` | text -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `minimax/minimax-m3` | 2026-05-31 | 1,048,576 | 512,000 | `$0.30/M` | `$1.20/M` | `$0.06/M` | text+image+video -> text | `tools`, `tool_choice`, `reasoning` |
| `minimax/minimax-m2.7` | 2026-03-18 | 204,800 | 131,072 | `$0.25/M` | `$1.00/M` | `$0.05/M` | text -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
| `qwen/qwen3-coder-next` | 2026-02-04 | 262,144 | 262,144 | `$0.11/M` | `$0.80/M` | `$0.07/M` | text -> text | `tools`, `tool_choice`, `structured_outputs` |
| `qwen/qwen3.7-plus` | 2026-06-03 | 1,000,000 | 65,536 | `$0.32/M` | `$1.28/M` | `$0.064/M` | text+image -> text | `tools`, `tool_choice`, `structured_outputs`, `reasoning` |
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

### Moonshot Kimi K2.7 Code

- Active local id for Pi, Goose, and OpenCode: `openrouter/moonshotai/kimi-k2.7-code`.
- OpenRouter id: `moonshotai/kimi-k2.7-code`.
- OpenRouter created date: 2026-06-12.
- OpenRouter context length: 262,144 tokens.
- OpenRouter max completion tokens: 262,144.
- OpenRouter API catalog pricing on 2026-06-14: input `$0.75/M`, output
  `$3.50/M`, cache read `$0.16/M`.
- OpenRouter model page excerpt on 2026-06-14 summarized `$0.95/M` input and
  `$4/M` output. Treat API/page price disagreement as live provider drift and
  verify before quoting spend.
- OpenRouter modalities: text+image input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `structured_outputs`, `reasoning`, and `response_format`.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `moonshotai/kimi-k2.7-code` on 2026-06-14, plus
  https://openrouter.ai/moonshotai/kimi-k2.7-code.

### Moonshot Kimi K2.6

- Retained local variant id: `openrouter/moonshotai/kimi-k2.6`.
- OpenRouter id: `moonshotai/kimi-k2.6`.
- OpenRouter created date: 2026-04-20.
- OpenRouter context length: 262,144 tokens.
- OpenRouter max completion tokens: 262,142.
- OpenRouter pricing on 2026-06-14: input `$0.68/M`, output `$3.41/M`,
  cache read `$0.34/M`.
- OpenRouter modalities: text+image input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `parallel_tool_calls`, `structured_outputs`, `reasoning`, and
  `reasoning_effort`.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `moonshotai/kimi-k2.6` on 2026-06-14.

### Moonshot Kimi K2.5

- Retained local variant id: `openrouter/moonshotai/kimi-k2.5`.
- OpenRouter id: `moonshotai/kimi-k2.5`.
- OpenRouter created date: 2026-01-27.
- OpenRouter context length: 262,144 tokens.
- OpenRouter max completion tokens: 262,144.
- OpenRouter pricing on 2026-06-14: input `$0.375/M`, output `$2.025/M`;
  cache read was not listed in the API row.
- NVIDIA forum reports provider-specific K2.5 deprecation/replacement pressure
  around K2.6. Treat provider behavior as platform-specific until verified.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `moonshotai/kimi-k2.5` on 2026-06-14, plus
  https://forums.developer.nvidia.com/t/kimi-k2-5-replacement/368480.

### DeepSeek V4 Pro

- Local Pi variant id: `openrouter/deepseek/deepseek-v4-pro`.
- OpenRouter id: `deepseek/deepseek-v4-pro`.
- OpenRouter created date: 2026-04-24.
- OpenRouter context length: 1,048,576 tokens.
- OpenRouter max completion tokens: 384,000.
- OpenRouter pricing on 2026-06-14: input `$0.435/M`, output `$0.87/M`,
  cache read `$0.003625/M`.
- OpenRouter modalities: text input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `structured_outputs`, and `reasoning`.
- DeepSeek docs list `deepseek-v4-pro` with 1M context and pricing details;
  prior discount notes may have changed, so verify live before quoting
  non-OpenRouter prices.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `deepseek/deepseek-v4-pro` on 2026-06-14, and
  https://api-docs.deepseek.com/quick_start/pricing.

### MiniMax M3

- Local open-model variant id: `openrouter/minimax/minimax-m3`.
- OpenRouter id: `minimax/minimax-m3`.
- OpenRouter created date: 2026-05-31.
- OpenRouter context length: 1,048,576 tokens.
- OpenRouter max completion tokens: 512,000.
- OpenRouter pricing on 2026-06-14: input `$0.30/M`, output `$1.20/M`,
  cache read `$0.06/M`.
- OpenRouter modalities: text+image+video input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `structured_outputs`, and `reasoning`.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `minimax/minimax-m3` on 2026-06-14.

### Qwen3 Coder Next

- Local open-model variant id: `openrouter/qwen/qwen3-coder-next`.
- OpenRouter id: `qwen/qwen3-coder-next`.
- OpenRouter created date: 2026-02-04.
- OpenRouter context length: 262,144 tokens.
- OpenRouter max completion tokens: 262,144.
- OpenRouter pricing on 2026-06-14: input `$0.11/M`, output `$0.80/M`,
  cache read `$0.07/M`.
- OpenRouter modalities: text input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  and `structured_outputs`.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `qwen/qwen3-coder-next` on 2026-06-14.

### Z.ai GLM 5.1

- Candidate id: `openrouter/z-ai/glm-5.1`.
- OpenRouter id: `z-ai/glm-5.1`.
- OpenRouter created date: 2026-04-07.
- OpenRouter context length: 202,752 tokens.
- OpenRouter pricing on 2026-06-14: input `$0.98/M`, output `$3.08/M`,
  cache read `$0.182/M`.
- OpenRouter modalities: text input to text output.
- OpenRouter supported parameters include `tools`, `tool_choice`,
  `parallel_tool_calls`, `structured_outputs`, `reasoning`, and
  `reasoning_effort`.
- Source: `curl -fsSL https://openrouter.ai/api/v1/models` filtered to
  `z-ai/glm-5.1` on 2026-06-14.

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

Use `/harness-engineering models` or `/research` when this file is stale
or a user asks for current model/provider/harness choices.

1. Read `.harness-kit/agents.yaml`, harness settings, and this file.
2. Probe local providers with `cargo run --locked -p harness-kit-checks -- probe-agent-roster`.
3. Query live provider catalogs/docs for exact model ids, context windows,
   max output, pricing, tool support, release dates, and deprecation notes.
4. Update this file with hard facts only.
5. Update `.harness-kit/agents.yaml` and harness settings only when changing a
   runnable default or variant.
6. Run `cargo run --locked -p harness-kit-checks -- probe-agent-roster --validate-only`.

Do not add subjective labels such as role fit, taste, or task suitability to
this file. Put task-specific composition rationale in the run's receipts,
context packet, or final debrief.
