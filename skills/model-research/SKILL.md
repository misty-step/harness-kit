---
name: model-research
description: |
  Research, compare, and select LLM models for AI-powered apps and workflows.
  Finds latest models per family, verifies availability on target platform,
  compares pricing/benchmarks/tool-calling, produces ranked recommendations.
  Use when: "which model", "compare models", "find a model", "model research",
  "best model for", "cheapest model", "tool calling models", "model selection",
  "upgrade model", "swap model", "fallback models", "model chain".
  Trigger: /model-research.
argument-hint: "[use-case description] [--platform openrouter|anthropic|openai|vertex]"
---

# /model-research

Find the right model for a specific use case. Live data, not training data.

## The Iron Laws

```
1. NEVER RECOMMEND MODELS FROM TRAINING DATA ALONE.
   Every model name, price, and benchmark MUST come from a live source.

2. EVERY MODEL HAS A RELEASE DATE. USE IT.
   The current date matters. A model released 3+ months ago may already be
   superseded. A model released last week may be unstable. Factor age into
   every recommendation.
```

LLM model landscapes change weekly. Training data is stale within months.
A model recommended from memory may be discontinued, superseded, or repriced.

## Delegation Judgment

delegate on judgment per the shared Roster contract: native subagents
by default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use specialized lanes for platform catalog verification,
benchmark/source review, social/reliability pulse, integration/codebase fit,
and cost/latency risk. Prefer different model/provider families for independent
judgment; the lead owns synthesis and final recommendation.

### The Freshness Rule

**Today's date is always available in system context. Use it.**

For every model under consideration, determine its release date and compute age:

| Age | Classification | Implication |
|-----|---------------|-------------|
| < 2 weeks | **Bleeding edge** | May have bugs, API instability, sparse benchmarks. Flag risk. |
| 2 weeks – 2 months | **Fresh** | Sweet spot: stabilized but still current-gen. Prefer these. |
| 2 – 4 months | **Aging** | Check if a successor exists in the same family. Often superseded. |
| 4+ months | **Stale** | Almost certainly superseded. Do NOT recommend without verifying it's still the latest in its family. |

**In a field moving this fast, "a few months old" IS old.** When two models
are otherwise comparable, prefer the fresher one — it reflects more recent
training data, RLHF tuning, and architecture improvements.

When presenting models, ALWAYS include the release date (or "unknown" if
unfindable) and age classification. This lets the user calibrate trust.

## Protocol

### Phase 1: Scope the Use Case

Before searching, answer these questions (ask the user if unclear):

1. **Task type**: tool calling, classification, generation, code, chat, embedding?
2. **Platform**: OpenRouter, Anthropic, OpenAI, Vertex, Bedrock, direct?
3. **Budget**: cost ceiling per million tokens? Per request?
4. **Quality floor**: what's the minimum acceptable reliability?
5. **Constraints**: context window, latency, streaming, multimodal, structured output?
6. **Volume**: requests per day/month? (affects rate limit viability)

### Phase 2: Live Research (MANDATORY PARALLEL FANOUT)

Launch ALL of these in parallel:

1. **Platform catalog** — fetch the actual model list from the target platform
   - OpenRouter: `https://openrouter.ai/api/v1/models` (JSON endpoint)
   - Filter by capability (tool calling, structured output, etc.)
   - This is the AUTHORITATIVE source for model IDs, pricing, and availability

2. **Exa search** — find recent benchmarks, comparisons, known issues
   - Search: `"[model family] tool calling benchmark 2026"`
   - Search: `"[model family] vs [competitor] [use case]"`
   - Search: `"openrouter [model name] issues OR problems OR reliability"`

3. **xAI / social pulse** — what practitioners are saying
   - Search: `"[model name] production" OR "[model name] tool calling"`
   - Catches reliability issues, rate limit complaints, silent deprecations

4. **Codebase** — what the project currently uses and why
   - Check existing model config, ADRs, policy files
   - Understand current integration shape (AI SDK version, provider setup)

### Phase 3: Model Family Audit

For EACH model family the user mentions or that appears promising:

1. **List ALL current models** in the family (not just the one you remember)
2. **Identify the latest** — check release dates, version numbers
3. **Check supersession** — is there a newer model that replaces this one?
4. **Verify availability** — is it actually on the target platform right now?
5. **Get exact pricing** — from the platform API, not from memory

### Phase 4: Head-to-Head Comparison

Build a comparison table with ONLY verified data:

| Field | Source |
|-------|--------|
| Model ID | Platform API |
| **Release date** | **Platform page, announcement post, or changelog** |
| **Age vs today** | **Computed from release date and current date** |
| Pricing | Platform API |
| Context window | Platform API |
| Tool calling support | Platform docs + benchmark |
| Benchmark scores | Recent (<3 month) benchmarks with citations |
| Known issues | Exa + xAI search results |
| Status | GA / Preview / Deprecated |

**Rules:**
- Every cell must have a source. No "probably" or "likely."
- If a benchmark score can't be found, write "No data" — don't interpolate.
- If pricing is from a blog post, verify against the platform API.
- Flag preview/beta models explicitly — they have different reliability profiles.
- **Release date is mandatory.** If you can't find it, write "Unknown (treat as stale)."
  A model with unknown release date gets the same skepticism as a 4+ month old model.

### Phase 5: Recommendation

Produce ranked recommendations for the specific use case:

1. **Primary model**: best quality-per-dollar for the task
2. **Fallback chain**: 2-3 models ordered by reliability, increasing cost acceptable
3. **Not recommended**: models that looked promising but have specific disqualifiers

For each recommendation, state:
- **Why this model** (specific evidence, not vibes)
- **Risk** (preview status, known issues, rate limits)
- **Cost impact** vs current model

## Gotchas

- **Models age fast**: A model from 3 months ago is likely superseded. ALWAYS compute
  age from release date. MiniMax M2.5 → M2.7 happened in ONE month. Gemini generations
  overlap. If you're recommending something >2 months old, you're probably behind.
- **Superseded models**: ALWAYS check if a newer version exists before recommending.
  Search "[model family] latest model [current year]" to catch recent releases.
- **Preview ≠ GA**: Preview models can change behavior without notice. Flag them.
  Preview models are especially likely to have intermittent reliability issues.
- **"Free tier" rate limits**: Many OpenRouter models have free tiers with 1 concurrent
  request. Not viable for production. Check the paid tier pricing.
- **Thinking tokens**: Models with reasoning (DeepSeek, Qwen) may generate hidden
  thinking tokens that add latency and cost. Check if thinking can be disabled.
- **Tool calling ≠ good tool calling**: A model may "support" tool calling but fail
  on complex schemas, parallel tools, or multi-step tool chains. Look for benchmarks
  specific to tool calling (tau2-bench, PinchBench, Toolathlon), not just general benchmarks.
- **OpenRouter routing**: Same model ID may route to different providers with different
  latency/reliability. Check if provider can be pinned.
- **Output token limits**: Some cheap models cap output at 4K-8K tokens. Check max output.
- **Stale benchmark data**: A benchmark from 6 months ago may not reflect current model
  performance, especially for preview models that get updated.
- **"Latest" in family ≠ best for task**: Newer isn't always better for a specific use
  case. A flash-lite model may be newer but weaker at tool calling than the older flash.
  Match the model's DESIGN PURPOSE to your task.

## Anti-Patterns

- Recommending a model you "know" without live verification
- Using general benchmarks (MMLU, HumanEval) for tool-calling selection
- Ignoring model family versions (recommending M2.5 when M2.7 exists)
- Treating all "flash/lite/mini" models as equivalent
- Recommending based on pricing alone without tool-calling quality check

## Verification

Semantic waiver: "latest model" facts are time-sensitive and require live
provider/docs checks. Each run must cite current availability, release date or
version evidence, pricing/benchmark source, and the exact platform where the
model will be used.
