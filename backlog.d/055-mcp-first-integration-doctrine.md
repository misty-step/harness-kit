# MCP-first integration doctrine

Priority: P2
Status: pending
Estimate: S

## Goal

Document the integration order for any future external system harness-kit (or a downstream repo consuming harness-kit) touches: **MCP server first, skill atop MCP second, CLI for local dev third.** Lands as a reference doc, not a skill. Prevents architectural mistakes before harness-kit's first external integration arrives.

## Non-Goals

- NOT building an MCP server. This is doctrine, not code.
- NOT ripping out any existing integration. Harness Kit has zero external-system integrations today; this is forward-looking.
- NOT advocating MCP over alternatives in every case. The doctrine names when MCP wins (multi-harness, multi-session, protocol-level auth and elicitation) and when simpler is fine (one-off local scripts, inner-loop tooling).
- NOT a skill with runtime behavior. Pure reference.

## Oracle

- [ ] `meta/INTEGRATION_GUIDE.md` exists. Structure:
  1. When to reach for MCP (symptoms: multi-harness, multi-repo, auth negotiation, interactive elicitation, tool-heavy)
  2. When NOT to reach for MCP (symptoms: one-off script, inner-loop only, local-dev-only)
  3. The three-layer order (MCP → skill → CLI) with rationale
  4. Pointers: Anthropic MCP SDK, Model Context Protocol spec, prior-art references
  5. Harness Kit-specific note: the doctrine is about integrations that reach *external systems*. Internal harness-kit tooling (bootstrap, generate-index) does not need MCP-ification.
- [ ] `harnesses/shared/AGENTS.md` L3 routing (from 051) contains a decision-table entry: "integrating an external system → read `meta/INTEGRATION_GUIDE.md` before choosing shape."
- [ ] Document cites specific downstream pain that motivated this: bitterblossom's sprite orchestration, canary's harness dispatch, cerberus's reviewer patterns (all have opaque dispatch without a shared integration model).
- [ ] `dagger call check --source=.` green.

## Notes

### Provenance

/groom session 2026-04-23 reading "Building agents that reach production systems with MCP" (twice — duplicate clipping). Strategist investigator found concrete downstream demand: bitterblossom, canary, and cerberus all have opaque harness-dispatch / sprite / MCP patterns with no reference doc. Zero guidance today means every downstream repo reinvents the shape.

### Why forward-looking doctrine, not a ticket with code

Harness Kit has zero external-system integrations today. Writing a skill before there's a system to integrate violates thin-harness-first (pre-solving). Writing doctrine before the first integration prevents the first integration from setting a bad precedent that future integrations copy.

### Investigator disagreement noted

Archaeologist said "N/A without evidence — no MCP drift visible in harness-kit." True locally. Strategist saw the downstream signal the Archaeologist couldn't see from harness-kit alone. Weight toward Strategist's view because MCP's value proposition is inherently cross-system, which harness-kit-only reading can't measure.

### The 100M → 300M MCP SDK download YoY growth

Cited in the clipping. Not load-bearing for the doctrine itself (arguments from usage volume are weak), but load-bearing for the timing: by the time harness-kit has its first external integration, the surrounding ecosystem will have established enough MCP patterns that rolling our own non-MCP integration will look archaeological. Better to set the convention now.

### What this isn't

Not a manifesto. Not a "why MCP is great." Not a tutorial. It's a decision-support reference for "I'm about to integrate X; should I use MCP, a skill, or a CLI?" The doc answers that and stops.

### Composition

- Depends on 051 (for the AGENTS.md L3 routing entry to land cleanly).
- Independent of 052, 053, 054.
- Pairs well with any future backlog item that actually ships an integration — the doctrine becomes testable at that point.

### Execution sketch

Single PR, single commit: `docs(meta): add MCP-first integration doctrine reference`. One file (`meta/INTEGRATION_GUIDE.md`) + one edit to the L3 routing section of `harnesses/shared/AGENTS.md` (which lands after 051).
