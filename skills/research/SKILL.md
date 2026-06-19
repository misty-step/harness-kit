---
name: research
description: |
  Web research, multi-AI delegation, and multi-perspective validation.
  /research [query], /research delegate [task].
  Use when: "search for", "look up", "research", "delegate",
  "get perspectives", "web search", "find out", "investigate",
  "introspect", "check readwise", "saved articles", "reading list",
  "what are people saying", "X search", "trending", "which model",
  "compare models", "best model for", "model selection".
argument-hint: "[query] or [web-search|web-deep|web-news|web-docs|delegate|introspect|readwise|xai|exemplars] [args]"
---

# Research

Evidence-backed research. The lead owns framing, source weighting, synthesis,
and residual uncertainty.

`/research` routes by capability, not vendor. Exa, xAI, Brave, Perplexity,
Context7, Tavily, Firecrawl, browser agents, and provider lanes are acquisition
tools behind a small set of evidence jobs: repo truth, docs lookup, web
retrieval, code/context examples, extraction, social/discourse, recency
verification, and synthesis.

## Route

| Need | Load |
|---|---|
| broad research, comparison, architecture prior art, or discourse scan | `references/default-fanout.md` |
| `web-search`, `web-deep`, `web-news`, `web-docs` | `references/web-search.md` |
| Exa search/fetch/deep/MCP/code context | `references/exa-tools.md` |
| extraction, site maps, crawls | `references/extraction-tools.md` |
| `delegate` | `references/delegate.md` |
| `introspect` | `references/introspect.md` |
| `readwise` | `references/readwise.md` |
| `xai` | `references/xai-search.md` |
| `exemplars` | `references/exemplars.md` |

If the user names a sub-capability, load that reference. Otherwise use the
default fanout for substantive research; narrow to one source only for explicit
single-source requests or simple fact/version lookups.

Model selection/comparison is research, not memory: model facts rot in
weeks. Start from the installed roster skill's
`references/model-provider-harness-index.md` for current local facts, then
verify against live sources — availability on the target platform, pricing,
context, tool-calling support — and return a ranked recommendation with
dates on every claim.

## Contract

- Read the live repo first for repo facts.
- Use current external sources for drift-prone facts.
- Keep provider CLIs and web tools thin: launch, bound, record.
- Prefer acquisition surfaces in this order when available: MCP tool first,
  local CLI wrapper second, direct REST/API call third, built-in WebSearch last.
- Treat web search, extraction, X/social search, provider lanes, and
  local grep as evidence inputs, not substitutes for lead synthesis.
- Do not let synthesis stand in for retrieval. A grounded answer may summarize
  sources, but the source URLs/artifacts remain the proof.
- Separate source evidence from conclusions; cite URLs, files, commands,
  receipts, or artifacts for claims.
- Label skipped, failed, stale, in-flight, and partial sources.
- Report residual uncertainty instead of smoothing over missing evidence.

## Delegation Judgment

delegate on judgment per the shared Roster contract: native subagents
by default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use lanes with distinct sources, methods, or model families; web search and provider CLIs are evidence inputs, not substitutes for lead synthesis.

## Completion Evidence

- Research objective and scope.
- Sources/tools queried and why.
- Provider lanes, receipt ids, accepted/rejected outputs, and failures.
- Claims tied to URLs, local files, commands, receipts, or artifacts.
- Source coverage gaps, stale facts, skipped tools, and residual risk.

## Gotchas

- A single WebSearch is a lookup, not substantive research.
- Mandatory source structure belongs in `references/default-fanout.md`; keep
  vendor command recipes in tool references.
