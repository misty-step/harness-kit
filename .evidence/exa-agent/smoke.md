# Exa Agent Live Smoke

Backlog: `105`
Date: 2026-06-16

Command:

```sh
if [ "${EXA_AGENT_LIVE_SMOKE:-}" = "1" ] && [ -n "${EXA_API_KEY:-}" ]; then
  EXA_AGENT_ENABLED=1 EXA_AGENT_EFFORT=low WEB_SEARCH_MAX_RESULTS=5 \
    bun run cli.ts web-deep "prior art landscape for agent skill marketplaces"
else
  printf 'SKIPPED: live Exa Agent smoke requires EXA_AGENT_LIVE_SMOKE=1, EXA_API_KEY, and an explicit cost budget.\n'
fi
```

Result: skipped. No live Exa Agent call was made because the run did not have
`EXA_AGENT_LIVE_SMOKE=1` and an explicit cost budget.

Deterministic acceptance evidence:

- `bun test` in `skills/research` passed.
- Tests cover default-off routing, opt-in/signal selection, high-effort guard,
  explicit `agentic` output mapping, and failure degradation to ordinary
  retrieval.
