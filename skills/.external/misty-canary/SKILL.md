---
name: canary
description: |
  Use when an agent needs production observability from Canary: uptime, health
  checks, check-ins, incidents, error timelines, service evidence, webhook
  responder context, API keys, SDK integration, or agent-readable operational
  truth. Trigger phrases: "Canary", "production debugging", "incident",
  "health check", "error log", "monitor", "observability".
argument-hint: "[query|integrate|health|incident|errors|mcp|sdk]"
---

# Canary

Canary is the owned observability substrate for agents. Query Canary before
making a repo-local production hypothesis. Integrate new services with Canary
unless there is a named reason not to.

Read `VISION.md` before changing product scope, responder boundaries, or
agent-facing surfaces.

## Route

| Need | Surface |
|---|---|
| Production health, incidents, recent errors | Canary API query routes or CLI query helpers |
| New service integration | `docs/factory-fleet-integration.md` |
| API contract | `GET /api/v1/openapi.json` and `priv/openapi/` |
| TypeScript consumers | `clients/typescript/` |
| MCP consumers | `bin/canary mcp-server` |
| Local or CI verification | `./bin/validate` |
| Live post-deploy signal | Query Canary itself for service/window evidence |

## Operating Rules

- Production debugging starts with Canary state: health, incidents, checks,
  recent errors, and service evidence.
- Canary owns ingest, health, correlation, timelines, queries, and webhooks.
  Repo mutation, issue creation, LLM triage, and repair live downstream.
- Use scoped keys. Do not reuse admin authority for read-only or ingest-only
  work.
- Webhook responders must query back into Canary for context instead of treating
  one payload as complete truth.
- Do not add app-specific responder logic to Canary. Consumers subscribe via
  generic webhooks.

## MCP

`bin/canary mcp-server` is the product-owned MCP server. Configure it with the
real Canary endpoint and non-interactive credentials supplied by the target
environment. Do not bake secrets into skill text, argv, or repository files.

## Verification

In the Canary repo:

```sh
./bin/validate
```

For consumer repos, prove both sides: the consumer emits or queries the expected
Canary signal, and Canary returns the expected service evidence.
