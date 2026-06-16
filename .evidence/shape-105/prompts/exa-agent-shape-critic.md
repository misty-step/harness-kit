Role: adversarial shape critic.

Objective: review the shaped packet and HTML plan for `backlog.d/105-exa-agent-research-lane.md`.

Artifacts to inspect:

- `backlog.d/105-exa-agent-research-lane.md`
- `.evidence/shape-105/exa-agent-research-lane.html`

Do not edit files. Do not use the author's chat context. Treat these artifacts
as the whole handoff an executor would receive.

Focus only on blocking or material issues:

- Does the plan accidentally create a vendor-shaped Exa workflow instead of
  preserving `/research` as capability-shaped?
- Can ordinary lookup/docs/news research accidentally trigger paid Exa Agent
  runs?
- Does the oracle actually catch missing citations, schema mismatch, deprecated
  API usage, cost leaks, and degraded provider failures?
- Is the non-ZDR/private-data boundary explicit enough for future executors?
- Does the design hide Exa Agent's async lifecycle or cost metadata behind the
  existing flat search provider contract?
- Is there a better simpler shape that would satisfy the hit-list premise with
  less repo surface?

Output shape:

```
BLOCKING: yes|no

Findings:
- [severity] file:line - issue, why it matters, exact fix

Verdict:
1-3 sentences on whether this packet is safe to hand to `/deliver`.
```

Keep the review under 1200 words. If there are no blockers, say so plainly and
name any non-blocking watchpoints.
