# Exa Agent shape critic

Provider: `pi`
Model: `openrouter/moonshotai/kimi-k2.7-code`
Delegation ID: `3490d817-3515-4c3a-afd5-2ce87f089a4c`
Runtime receipt: `.harness-kit/traces/provider-lanes/20260616T203431.595439Z-pi-f15e560b.txt`

## Verdict

BLOCKING: no

The critic judged the packet capability-shaped rather than vendor-shaped:
Exa Agent is framed as an optional `/research` acquisition lane, Exa Search
is preserved, fallback chains are defined, and lead synthesis remains
authoritative.

## Findings Accepted

- The Agent trigger needed deterministic gates. The packet now requires
  `EXA_AGENT_ENABLED=1` or enumerated positive signals, with Agent defaulting
  off when no positive signal matches.
- Expensive effort modes needed an explicit control surface. The packet now
  requires `EXA_AGENT_ALLOW_EXPENSIVE=1` for `high`, `xhigh`, or `auto`.
- The response shape should not remain a fork. The packet now chooses an
  explicit `agentic` block for Agent run metadata, citations, structured
  output, cost, and degraded status.
- The non-ZDR/private-context boundary needed a testable acceptance mechanism.
  The packet now requires `EXA_AGENT_PRIVATE_CONTEXT_OK=1` before private repo,
  customer, or local-file context is sent to Agent.

## Residual Watchpoints

- Delivery should keep the deterministic routing function small and covered by
  tests rather than letting vague "broad research" heuristics grow.
- Live smoke tests should stay low-effort unless the operator explicitly opts
  into expensive modes.
