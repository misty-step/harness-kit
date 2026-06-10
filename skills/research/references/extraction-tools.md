# Extraction Tools

Use extraction tools when the research task names a URL, site, crawl, sitemap,
or page content target. Do not route ordinary broad research here by default.

## Capability Routing

| Need | Prefer | Fallback |
|---|---|---|
| Fetch known URL into context | Exa `web_fetch_exa` or `/contents` | browser snapshot |
| Extract clean markdown from a page | Firecrawl scrape/extract | Exa fetch |
| Map a site or docs corpus | Tavily Map or Firecrawl Map | manual link walk |
| Crawl a bounded docs site | Firecrawl Crawl | targeted search + fetch |
| Dynamic or logged-in page | browser automation (Playwright/Chrome MCP) | manual operator-provided artifact |

## Harness Contract

- Bound crawls by domain, path, max pages, and time budget.
- Emit source URLs and extraction status; an extracted summary without the
  source URL is not evidence.
- Prefer official docs or repo-local files before paid extraction.
- Treat browser agents as the fallback for dynamic pages, not the default for
  static pages.

## Smoke Shape

Live provider smoke tests are env-gated. Offline evals should assert routing,
not hit paid APIs:

- `map this docs site` routes to map/crawl capability.
- `extract this URL` routes to fetch/extract capability.
- `what are people saying` does not route to extraction.
- `latest model pricing` does not route to extraction unless a target URL is
  named.
