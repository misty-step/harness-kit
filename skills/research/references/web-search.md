# Web Search

Retrieval-first web research with citations and recency controls.

## Commands
- `/research web-search <query>`: fast top links
- `/research web-deep <query>`: fetch and summarize with citations
- `/research web-news <query>`: recency-biased search results
- `/research web-docs <query>`: library/docs-focused retrieval

Legacy aliases: `/web`, `/web-deep`, `/web-news`, `/web-docs`.
`meta.command` below stores the normalized internal routing value, not the full
user-facing slash command.

## Behavior Contract
- Return structured result envelope (see schema below)
- Include citation URL for every claim
- Prefer Context7 for docs/library lookups
- Prefer Exa as primary general provider
- Use xAI for social/real-time and grounded web with multimodal (see `xai-search.md`)
- Fallback to Brave on provider failure
- Optional Perplexity pass allowed only for synthesis, never source of truth

## Exa-First Patterns

### Reference Implementation Search
Enforces CLAUDE.md "reference architecture first" red line.
```
Query: "open source [problem domain] implementation [language/framework]"
Provider: Exa code context
```

### Academic/Technical Search
Formal specifications, algorithm papers, design patterns.
```
Query: "[algorithm/protocol] formal specification paper"
Provider: Exa neural search
```

### Recency-Filtered
Model releases, security advisories, deprecation notices.
```
Query: "[topic] [year]"
Provider: Exa with start_published_date filter
```

### Smart Routing Table

| Query signals | Route to |
|--------------|----------|
| Contains "code", "implementation", "example", "how to" | Exa code context |
| Contains "docs", "documentation", "API reference" | Context7 first, Exa fallback |
| Contains year, "latest", "current", "new" | Exa with recency filter |
| Contains "paper", "formal", "specification" | Exa neural search |
| Contains "people saying", "sentiment", "trending", "discourse" | xAI X Search (see `xai-search.md`) |
| Contains "X/Twitter", specific handles, social | xAI X Search |
| None of the above | Exa neural (default) |

## Output Schema
```json
{
  "results": [
    {
      "title": "string",
      "url": "string",
      "snippet": "string",
      "published_at": "ISO-8601 or null",
      "score": 0.0,
      "source_provider": "context7|exa|xai|brave|perplexity"
    }
  ],
  "meta": {
    "query": "string",
    "command": "web|web-deep|web-news|web-docs",
    "provider_chain": ["context7", "exa", "xai", "brave"],
    "provider_used": "context7|exa|xai|brave|null",
    "cache_hit": false,
    "time_sensitive": false,
    "recency_days": null,
    "confidence": "high|medium|low",
    "uncertainty": "string|null"
  },
  "synthesis": {
    "summary": "string",
    "citations": ["https://..."]
  }
}
```

`meta.command` is the normalized internal route selected by the umbrella skill,
not the full user-facing slash command.

## Safety and Quality
- Never fabricate URLs
- Mark uncertain facts as uncertain
- Apply recency filters for time-sensitive queries

## Runtime Notes
- Pi extension entrypoint: `~/.pi/agent/extensions/web-search/` (loaded via settings.json)
- Cache: `cache/web-search-cache.json` (TTL via `WEB_SEARCH_TTL_MS`)
- Logs: `logs/web-search.ndjson` (size-rotated)
- `PI_WEB_SEARCH_LOG_MAX_BYTES` / `PI_WEB_SEARCH_LOG_MAX_BACKUPS` / `PI_WEB_SEARCH_LOG_ROTATE_CHECK_MS`
- Provider HTTP requests are bounded by `WEB_SEARCH_PROVIDER_TIMEOUT_MS` (default 15000ms)
- Cost controls:
  - `WEB_SEARCH_MAX_RESULTS` caps results per query
  - Cache dedupe prevents repeated provider calls for same normalized query
