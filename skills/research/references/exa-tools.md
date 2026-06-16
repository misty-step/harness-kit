# Exa Research Tools

Exa provides neural search optimized for code and technical content.

## Access

**Official remote MCP plus REST API via local CLI wrappers or curl.**

Auth: `x-api-key: $EXA_API_KEY` header. Key is set in shell env.

MCP endpoint: `https://mcp.exa.ai/mcp`. Use it when the active harness has MCP
tool support. Use `exa-search` / `exa-fetch` when MCP is unavailable or a
script needs deterministic JSON. Use raw REST only when no wrapper is present.

Local wrappers, when installed:

```bash
exa-search --num 5 --chars 1000 "YOUR QUERY HERE"
exa-fetch --chars 2000 https://example.com/page1 https://example.com/page2
```

## Search

```bash
curl -s https://api.exa.ai/search \
  -H "x-api-key: $EXA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "YOUR QUERY HERE",
    "type": "auto",
    "numResults": 5,
    "useAutoprompt": true,
    "contents": { "text": { "maxCharacters": 1000 } }
  }'
```

### Deep / Structured Search

Use Exa deep search when the task needs stronger source gathering before
synthesis, or structured output that a downstream script can validate.

```bash
curl -s https://api.exa.ai/search \
  -H "x-api-key: $EXA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "best practices for browser agent visual regression",
    "type": "deep",
    "numResults": 8,
    "contents": { "text": { "maxCharacters": 2000 } }
  }'
```

### Legacy Research API

Do not build new Harness Kit work on Exa's legacy `/research/v1` API. Treat it
as deprecated for new integration design; prefer the Agent API for async
multi-step research, or Exa Search with `type: "deep"` / `"deep-reasoning"` for
search-shaped fallback work.

### Agent API

Use Exa Agent only for broad `web-deep` work where flat search is structurally
weak: multi-entity research, list-building, enrichment, prior-art landscapes,
or comparisons across many sources. It is async, costed, and not zero-data-
retention; do not send private repo/customer context unless the operator has
explicitly allowed it for the run.

Harness Kit routes Agent output into `response.agentic`, never into the flat
`results[]` list.

```bash
EXA_AGENT_ENABLED=1 EXA_AGENT_EFFORT=low \
  bun run cli.ts web-deep "prior art landscape for agent skill marketplaces"
```

Runtime controls:

- `EXA_AGENT_ENABLED=1` opts in explicitly.
- Enumerated broad-research signals can select Agent for `web-deep`; ordinary
  queries default off.
- `EXA_AGENT_EFFORT=minimal|low|medium|high|xhigh|auto` defaults to `medium`.
- `high`, `xhigh`, and `auto` require `EXA_AGENT_ALLOW_EXPENSIVE=1`.
- `EXA_AGENT_TIMEOUT_MS` and `EXA_AGENT_POLL_INTERVAL_MS` bound the async run.
- `EXA_AGENT_PRIVATE_CONTEXT_OK=1` is required before private local/repo/
  customer context may be included in Agent input.
- `response.agentic.private_context_allowed` records that consent in the
  returned artifact. Keep it false for public-web-only runs.

Current official endpoint family: `https://api.exa.ai/agent/runs`, with
separate run retrieval and event endpoints under Exa's Agent API docs.
The docs checked for this change show create/poll/list/event/continue flows,
but no verified cancel endpoint. Treat timeout after run creation as a residual
cost risk until a cancel endpoint is confirmed.

### Code Context Search

Find reference implementations — highest-leverage research for engineers.

```bash
curl -s https://api.exa.ai/search \
  -H "x-api-key: $EXA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "TLA+ PlusCal payment state machine example",
    "type": "code",
    "numResults": 5,
    "useAutoprompt": true,
    "contents": { "text": { "maxCharacters": 2000 } }
  }'
```

### Recency-Filtered

For time-sensitive queries (model releases, security advisories).

```bash
curl -s https://api.exa.ai/search \
  -H "x-api-key: $EXA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Claude API latest model versions",
    "type": "auto",
    "numResults": 5,
    "startPublishedDate": "2026-01-01",
    "contents": { "text": { "maxCharacters": 1000 } }
  }'
```

### Find Similar

Find pages similar to a known URL.

```bash
curl -s https://api.exa.ai/findSimilar \
  -H "x-api-key: $EXA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/good-reference",
    "numResults": 5,
    "contents": { "text": { "maxCharacters": 1000 } }
  }'
```

### Get Contents

Extract content from known URLs.

```bash
curl -s https://api.exa.ai/contents \
  -H "x-api-key: $EXA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "ids": ["https://example.com/page1", "https://example.com/page2"],
    "text": { "maxCharacters": 2000 }
  }'
```

## When to Use Each Mode

| Need | Type | Example |
|------|------|---------|
| "How does X implement Y?" | `code` | Reference architecture search |
| "What's the current best practice for Z?" | `auto` + recency | Library/framework decisions |
| "Is X still recommended?" | `auto` + `startPublishedDate` | Model currency, deprecation |
| "Find papers on X" | `auto` | Academic/formal specs |
| "Pages like this one" | `findSimilar` | Expand from known good source |
| "Build/enrich/compare many entities" | Agent API | Async structured research |

## MCP Tool Names

When Exa MCP is configured, prefer these capability-shaped tools:

- `web_search_exa` — broad web search.
- `web_search_advanced_exa` — filtered/deeper search.
- `web_fetch_exa` — fetch known URLs into context.
- Company, LinkedIn, GitHub, and competitor tools are specialized retrieval
  lanes; do not invoke them for generic research.

## Integration with Research Skill

The `/research` default fanout calls Exa for retrieval, code/context examples,
known URL fetch, and deep/structured search. Optional Agent runs are an
agentic acquisition lane for broad `web-deep` only. Exa results include URLs —
always cite them.

Provider chain: Exa MCP → `exa-search` / `exa-fetch` → Exa REST/curl →
WebSearch (fallback only)
