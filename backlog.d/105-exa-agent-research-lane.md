# Context Packet: Optional Exa Agent research lane

Priority: P2
Status: ready
Estimate: M

## Goal

Make `/research web-deep` better at broad, multi-hop web research by adding an
optional Exa Agent acquisition lane that returns structured, cited evidence
without replacing the existing fast Exa Search path.

## Premise Challenged

The hit-list premise says Exa Agent "might come for free" from the current
research skill. It partly does: `/research` already routes Exa as a primary
general retrieval provider and documents MCP, search, fetch, deep, and code
context usage in `skills/research/references/exa-tools.md`.

It does not come fully for free. The current runtime provider interface is a
flat `search(request) -> SearchResult[]` chain that returns the first nonempty
provider result. Exa Agent is a long-running, async, multi-step research API
with structured output, grounding, run lifecycle, cost breakdown, continuation,
and events. Treating that as just another flat search provider would hide the
parts that make it useful and expensive.

The user outcome is not "add a vendor." The outcome is: when a research task is
open-ended enough that single search results are too shallow, the agent should
know to use a stronger acquisition lane, preserve citations and cost evidence,
and still synthesize with lead judgment.

## Non-Goals

- Do not create a standalone `exa-agent` skill.
- Do not replace Exa Search, Context7, xAI, Brave, or Perplexity synthesis.
- Do not use Exa's deprecated `/research/v1` API for new work.
- Do not make live Exa Agent calls part of the default gate or CI.
- Do not store raw prompts, full external transcripts, API keys, or provider
  internals beyond sanitized run metadata and cited outputs.
- Do not use Exa Agent for low-latency docs lookup, news lookup, single-page
  extraction, or ordinary top-link retrieval.

## Constraints

- Keep `/research` capability-shaped, not vendor-shaped.
- Exa remains an acquisition surface behind the research skill.
- MCP remains preferred when it has the needed capability; REST/API is allowed
  for Agent because the current MCP docs expose search and fetch tools, not the
  Agent run lifecycle as the default path.
- CI must be deterministic without paid provider access.
- Live Exa smoke tests must be opt-in and skipped cleanly without
  `EXA_API_KEY`.
- Every Agent run must be bounded by effort, timeout, and output schema/list
  limits where possible.
- Cost and provenance must be first-class evidence, not invisible side effects.

## Repo Anchors

- `skills/research/SKILL.md` - routes research by capability and says Exa is
  one acquisition tool behind retrieval, code examples, extraction, recency,
  and synthesis.
- `skills/research/references/default-fanout.md` - the source matrix and
  capability lane contract for substantive research.
- `skills/research/references/web-search.md` - command semantics, provider
  routing, response schema, cache/log notes, and cost controls.
- `skills/research/references/exa-tools.md` - Exa-specific recipes and the
  right place to document Agent use and fallback.
- `skills/research/provider-adapter.ts` - current flat provider contract.
- `skills/research/providers.ts` - current `ExaProvider` implementation,
  timeout helper, and provider error style.
- `skills/research/orchestrator.ts` - current first-success provider chain,
  cache, dedupe, and logging.
- `skills/research/cli.ts` - provider construction, `web-deep` synthesis, and
  output envelope assembly.
- `skills/research/__tests__/runtime-hardening.test.ts` - existing Bun test
  style for provider and runtime fault behavior.
- `cargo run --locked -p harness-kit-checks -- check --repo .` - canonical
  repo gate.

## External Evidence

- Exa announced Agent on 2026-06-16 as a single API for frontier web research.
  The blog describes deep research, list building, entity enrichment, parallel
  subtasks, model fusion, structured outputs, input data, fixed effort levels,
  and direct API availability.
- Exa Agent docs describe async runs for deep research/list-building/enrichment
  with natural-language answers, structured outputs, citations, metadata, cost
  breakdown, event replay, and continuation.
- Exa Agent lifecycle is create run, queue/start, poll or stream events, then
  read terminal output. Terminal output can include text, structured JSON, and
  grounding citations.
- Exa docs say the older `/research/v1` API is deprecated as of May 1, 2026 and
  recommend search with `type: "deep-reasoning"` for equivalent legacy
  functionality. New work should not build on `/research/v1`.
- Exa MCP docs list default `web_search_exa` and `web_fetch_exa`, optional
  `web_search_advanced_exa`, and deprecated deep researcher tools. That supports
  using MCP for search/fetch while using Agent API only when lifecycle and
  structured run evidence are required.
- Pricing docs and Agent docs make cost a real design input: fixed effort modes
  range from minimal through xhigh; Agent compute and search calls are billed,
  and Agent is not zero-data-retention.

## Alternatives

| Option | What it buys | Where it fails | Verdict |
|---|---|---|---|
| Do nothing | No new surface; current Exa Search is already wired and documented. | Misses Agent's async multi-step, structured, costed, grounded run shape; broad research still depends on first-result provider behavior plus optional synthesis. | Reject. The hit-list item identifies a real capability gap. |
| Standalone `exa-agent` skill | Easy user invocation and vendor-specific docs. | Violates route-by-capability; duplicates `/research`; increases catalog surface for a vendor choice; invites users to bypass source matrix and lead synthesis. | Reject. |
| Add Agent as a normal `ProviderAdapter.search()` implementation | Minimal code change; fits current chain. | Collapses async lifecycle, structured output, cost, grounding, and stop reasons into flat links; risks accidentally calling expensive Agent for ordinary queries. | Reject. |
| Extend `/research` with an optional Exa Agent lane for broad `web-deep` tasks | Preserves capability routing, keeps Exa Search fast path, exposes Agent's real lifecycle and cost evidence, and remains testable with mocks. | Requires a small provider-interface extension and careful routing heuristics. | Choose. |
| Use Exa `deep` or `deep-reasoning` search only | Lower surface than Agent; aligns with Exa's search API guide and deprecated research migration. | Good for synthesized search, but weaker for long-running list-building/enrichment, continuation, run events, and costed research tasks. | Include as fallback, not the primary shape for this hit. |

## Design

Add an optional "agentic acquisition" branch to `/research`, scoped to
substantive `web-deep` work where a single search is structurally weak.

### Routing

- Keep `buildProviders()` as the fast retrieval chain for `web`, `web-news`,
  `web-docs`, and ordinary `web-deep`.
- Add a separate Agent-capable acquisition path, for example
  `AgenticResearchProvider`, not a replacement `SearchProvider`.
- Enable it only when all are true:
  - command is `web-deep`;
  - `EXA_API_KEY` is set;
  - `EXA_AGENT_ENABLED=1` is set, or a deterministic routing function matches
    enumerated positive signals such as explicit "use Exa Agent", "build/list
    prospects", "enrich these entities", "compare options across sources",
    "prior art landscape", or "multi-entity research". Absent a positive signal,
    Agent defaults off;
  - the task is not docs-only, social/discourse-only, single URL extraction, or
    simple top-link lookup;
  - cost controls are present (`EXA_AGENT_EFFORT`, timeout, max output/entity
    limits).
- Default effort should be `medium` for standard single-topic research and
  `low` or `minimal` for narrow lookups. Require `EXA_AGENT_ALLOW_EXPENSIVE=1`
  before accepting `EXA_AGENT_EFFORT=high`, `xhigh`, or `auto`.
- Do not include private repo excerpts, customer data, or local file contents in
  Agent input unless `EXA_AGENT_PRIVATE_CONTEXT_OK=1` is set and the response
  records that acknowledgment as run metadata.

### Data/control flow

1. Build the normal source matrix from `default-fanout.md`.
2. If Agent is selected, create an Agent run with a schema that returns:
   `summary`, `findings[]`, `citations[]`, `open_questions[]`, and optional
   `entities[]` only when requested.
3. Poll or stream with a bounded timeout.
4. Persist sanitized run evidence in the response envelope and log:
   provider, run id, status, stop reason, effort, cost if returned, citations,
   and degraded/error messages. Do not log API keys or full private prompts.
5. Put Agent output in an explicit `agentic` block on the response envelope:
   `provider`, `run_id`, `status`, `effort`, `cost`, `citations`,
   `structured_output`, and `degraded`. Do not masquerade Agent findings as
   ordinary flat `SearchResult` rows. Existing `results` may still carry normal
   retrieval/fallback links.
6. Lead synthesis still owns the final answer. Agent output is evidence, not
   authority.
7. On timeout, budget stop, missing citations, schema violation, or provider
   failure, degrade to Exa Search/Deep Search plus existing synthesis/fallback
   rather than failing the whole research command when usable retrieval exists.

### Documentation

- Update `skills/research/references/exa-tools.md` with:
  - when to use Agent;
  - when not to use Agent;
  - REST endpoint/run lifecycle;
  - cost and ZDR caveats;
  - deprecation warning for `/research/v1`;
  - fallback to `type: "deep"`/`deep-reasoning` search.
- Update `skills/research/references/web-search.md` and
  `default-fanout.md` so the source matrix can label Exa Agent as a completed,
  skipped, partial, or failed acquisition lane.
- Keep the docs vendor-specific only where command recipes or API surfaces
  require it.

## Oracle

The work is done when these executable checks pass:

- `bun test` from `skills/research` passes and includes mocked coverage for:
  - routing skips Agent for `web`, `web-news`, `web-docs`, simple docs queries,
    and missing `EXA_API_KEY`;
  - routing selects Agent for a broad `web-deep` query only when
    `EXA_AGENT_ENABLED=1` is set or an enumerated positive signal matches;
  - absent a positive signal, Agent defaults off even for `web-deep`;
  - Agent create/poll success maps structured output, grounding citations, run
    id, effort, status, and cost metadata into an explicit `agentic` block;
  - Agent timeout/failure/schema-mismatch degrades to existing retrieval or
    records a structured degraded reason;
  - `high`, `xhigh`, and `auto` effort modes require
    `EXA_AGENT_ALLOW_EXPENSIVE=1`;
  - private local/repo/customer context is rejected unless
    `EXA_AGENT_PRIVATE_CONTEXT_OK=1` is set and logged.
- The premise hash check passes:
  `test "$(shasum -a 256 HIT-LIST.md | awk '{print $1}')" = "ba4b4b28887bd991ea21311c4f9c0c9b38a8d0d4c5f27535b941c62dd50b8663"`.
- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.
- With `EXA_API_KEY` available, an opt-in live smoke is run and recorded:
  `EXA_AGENT_ENABLED=1 EXA_AGENT_EFFORT=low WEB_SEARCH_MAX_RESULTS=5 bun run <research-test-or-cli> web-deep "<broad test query>"`
  and the output contains citations plus bounded run metadata. If no key is
  available, the live smoke is explicitly skipped and mocked tests remain the
  acceptance gate.

## Verification Harness

The repo's proof loop is:

1. `bun test` in `skills/research` for runtime behavior.
2. `test "$(shasum -a 256 HIT-LIST.md | awk '{print $1}')" = "ba4b4b28887bd991ea21311c4f9c0c9b38a8d0d4c5f27535b941c62dd50b8663"`
   for premise provenance. The current `harness-kit-checks` binary does not
   expose the `premise-source` subcommand named in `skills/shape/SKILL.md`, so
   delivery should not depend on that stale command until a separate
   housekeeping fix restores it.
3. `cargo run --locked -p harness-kit-checks -- check --repo .` for the full
   Harness Kit gate.
4. Optional live Exa smoke only when an API key and cost budget are available.

## Premise Source

Premise Source: sha256:ba4b4b28887bd991ea21311c4f9c0c9b38a8d0d4c5f27535b941c62dd50b8663 HIT-LIST.md

External source URLs checked on 2026-06-16:

- https://exa.ai/blog/exa-agent
- https://exa.ai/docs/reference/agent-api-guide
- https://exa.ai/docs/reference/agent-api/overview
- https://exa.ai/docs/reference/exa-mcp
- https://exa.ai/docs/reference/search-api-guide
- https://exa.ai/pricing

## HTML Plan

HTML Plan: `.evidence/shape-105/exa-agent-research-lane.html`

The plan should be opened before delivery. Its hero is the work contract:
optional Exa Agent lane inside `/research`, not a standalone skill.

## Risks + Rollout

| Risk | Mitigation |
|---|---|
| Expensive accidental provider calls | Require `web-deep`, API key, explicit enablement or strong heuristic, effort cap, timeout, and opt-in for high/xhigh/auto. |
| Vendor lock-in inside `/research` | Keep capability routing; make Agent one acquisition lane; preserve fallback to Exa Search, Context7, xAI, Brave, and lead synthesis. |
| Hidden provider hallucination | Require grounding citations and schema validation; classify missing citations as degraded evidence. |
| Deprecated API usage | Do not call `/research/v1`; document it only as historical/deprecated. |
| Sensitive data exposure | Document that Exa Agent is not ZDR; reject private repo/customer/local-file context unless `EXA_AGENT_PRIVATE_CONTEXT_OK=1` is set and recorded. |
| Interface bloat | Add the smallest explicit agentic result block or adapter trait; do not force all providers into Agent lifecycle methods. |

Rollback is simple: remove the optional Agent branch and docs additions; the
existing Exa Search provider and provider chain continue to work.

## Adversarial Review Focus

Ask the critic only from the artifact:

- Is this a vendor-shaped skill in disguise?
- Does the oracle catch accidental paid calls and degraded citation quality?
- Is the routing narrow enough that normal research does not become slower or
  more expensive?
- Does the design preserve `/research` as lead-owned synthesis over evidence?
- Is any claim relying on deprecated Exa Research API behavior?
- Are the deterministic Agent trigger, `EXA_AGENT_ALLOW_EXPENSIVE`, and
  `EXA_AGENT_PRIVATE_CONTEXT_OK` guardrails implemented and tested?
