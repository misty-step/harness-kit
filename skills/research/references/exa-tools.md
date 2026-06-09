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

## MCP Tool Names

When Exa MCP is configured, prefer these capability-shaped tools:

- `web_search_exa` — broad web search.
- `web_search_advanced_exa` — filtered/deeper search.
- `web_fetch_exa` — fetch known URLs into context.
- Company, LinkedIn, GitHub, and competitor tools are specialized retrieval
  lanes; do not invoke them for generic research.

## Integration with Research Skill

The `/research` default fanout calls Exa for retrieval, code/context examples,
known URL fetch, and deep/structured search. Exa results include URLs — always
cite them.

Provider chain: Exa MCP → `exa-search` / `exa-fetch` → Exa REST/curl →
WebSearch (fallback only)
