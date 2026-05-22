---
name: research
description: |
  Web research, multi-AI delegation, and multi-perspective validation.
  /research [query], /research delegate [task], /research thinktank [topic].
  Triggers: "search for", "look up", "research", "delegate", "get perspectives",
  "web search", "find out", "investigate", "introspect", "session analysis",
  "check readwise", "saved articles", "reading list", "highlights",
  "what are people saying", "X search", "social sentiment", "trending".
argument-hint: "[query] or [web-search|web-deep|web-news|web-docs|delegate|thinktank|introspect|readwise|xai] [args]"
---

# Research

Retrieval-first research, multi-AI orchestration, and expert validation.

## Execution Stance

You are the executive orchestrator.
- Keep query framing, source weighting, and final synthesis on the lead model.
- Delegate source retrieval and specialized analysis to focused subagents/tools.
- Run multi-source fanout in parallel for independent evidence streams.
- When `.spellbook/agents.yaml` exists, treat it as the external coding-agent
  roster for repo-aware research lanes. Probe availability, choose bounded
  provider tasks, and record meaningful lanes with the receipt tool.

## Absorbed Skills

This skill consolidates: `web-search`, `delegate`, `thinktank`, `introspect`.

## Routing

### Explicit sub-capability (user names one)

If first argument matches a keyword, route directly to that reference:

| Keyword | Reference |
|---------|-----------|
| `web-search`, `web-deep`, `web-news`, `web-docs` | `references/web-search.md` |
| `delegate` | `references/delegate.md` |
| `thinktank` | `references/thinktank.md` |
| `introspect` | `references/introspect.md` |
| `readwise` | `references/readwise.md` |
| `xai` | `references/xai-search.md` |
| `exemplars` | `references/exemplars.md` |

### No sub-capability (default): MANDATORY PARALLEL FANOUT

**You MUST fan out to multiple sources. A single WebSearch is NOT research.**

Research means gathering signal from many vantage points simultaneously.
WebSearch alone is a lookup, not research. The whole point of this skill is
multi-source triangulation.

**REQUIRED: Launch ALL of these in a single message (parallel Agent/Bash calls):**

1. **Exa search** — Bash: `curl -s https://api.exa.ai/search -H "x-api-key: $EXA_API_KEY" ...`
   See `references/exa-tools.md` for request format. WebSearch is fallback ONLY if curl fails.
2. **Thinktank** — Run the native research bench only when local repository context materially matters.
   Default fanout path: `thinktank run research/quick --input "$QUERY" --output /tmp/thinktank-out --json --no-synthesis`
   Deep path: `thinktank research "$QUERY" --output /tmp/thinktank-out --json`
   Add `--paths ...` when local files or directories should be pointed out as starting places.
   Before waiting, record the mode (`quick` or `deep`), output directory, and expected runtime band in your own notes.
   Default budgets: `quick` targets `60-180s` with a hard cap of `300s`; `deep` targets `3-8m` with a hard cap of `900s`.
   Thinktank launches a real multi-agent Pi bench, so even `research/quick` can take a few minutes.
   With `--json`, stdout stays reserved for the final envelope; a healthy run may look quiet until completion.
   Inspect the output directory while it runs: `trace/events.jsonl`, `manifest.json`, `task.md`, and `prompts/` are the current partial-progress artifacts.
   Current limitation: if you stop a run early, completed agent reports may not exist yet. Treat that as a current Thinktank limitation, not as evidence that nothing happened.
   Skip Thinktank for pure external/web-only research and state that explicitly in the synthesis.
3. **xAI / social pulse** — Bash: `curl -s https://api.x.ai/v1/responses -H "Authorization: Bearer $XAI_API_KEY" ...`
   Model MUST be `grok-4.20-beta-latest-non-reasoning` (only grok-4 supports tool use).
   See `references/xai-search.md` for request format. Skip ONLY for purely technical/code queries.
4. **Codebase** — Grep/Glob for what the project already does (skip only if query is unrelated to codebase)

**Then produce a sourced report** using the mandatory structure below.

### Report Format (mandatory for all default fanout runs)

Every research report MUST have one labeled section per source that ran,
followed by a synthesis. Omit a section only if that source was explicitly
skipped — and state why it was skipped in the Synthesis section.

```
## Exa (neural search)
[Findings with inline URLs. What did Exa specifically surface?]

## xAI / Grok ([web_search | x_search | both])
[Findings with citations from response.citations. What did Grok surface?
For X Search: quotes or paraphrases from X posts, authors, dates.]

## Thinktank (Pi bench)
[What did the thinktank bench surface? Label this section `Thinktank (complete)` or `Thinktank (partial)`. Note any disagreements between agents. If skipped, say the query was not repo-aware enough to justify the bench.]

## Codebase
[What relevant patterns, implementations, or prior art exist locally?
"None found" is a valid answer — write it explicitly.]

## Synthesis
[Consensus across sources. Conflicts or contradictions between them.
Recommendations grounded in the evidence above. Every claim cites a source.]
```

**Discipline rule**: if a section is missing, you either skipped that tool
(state why) or you failed to run it (go back and run it). A report that
collapses all sources into one unlabeled blob has failed the fanout goal.
Readers must be able to see what each tool contributed independently.

If Thinktank is still running when you need to respond, say that plainly.
Name the output directory and summarize only the artifacts that actually exist
so far. Do not present an incomplete run as a finished Thinktank result.

**Narrow to a single source ONLY when:**
- The user explicitly names one (e.g., "/research web-search [query]")
- It's a version/fact lookup (e.g., "what version is X?")

**If you catch yourself about to return results from only WebSearch — STOP.
That means you skipped the fanout. Go back and launch the other sources.**

## Use When

- Before implementing any system >200 LOC (reference architecture search)
- Before choosing a library, framework, or approach (current best practices)
- When training data may be stale (model releases, API changes, deprecations)
- When you need to verify a fact before asserting it
- When the user asks about something outside the codebase
- During `/groom` architecture critique (reference implementations)
- During `/shape` technical exploration (how others solve this)
- During `/build` understand step (existing patterns and examples)
- When another skill says "web search first" or "research before implementing"

## Decision Framework

**If you're about to assert something from training data that could be wrong,
invoke `/research web-search` first.** The cost of a search is negligible;
the cost of hallucination is high.

This applies especially to:
- Model names and versions (stale within months)
- Library APIs and best practices (change with major versions)
- Pricing, availability, feature comparisons
- Security advisories, CVEs, deprecation notices

## Provider Routing

| Query Type | Primary Provider | Why |
|------------|-----------------|-----|
| Code examples, reference implementations | Exa (code context) | Finds actual code, not blog posts |
| Academic papers, formal specs | Exa (search) | Strong academic indexing |
| Library/framework docs | Context7 | Semantic doc search |
| Current events, model releases | Exa (with recency filter) | Real-time indexing |
| Social sentiment, public discourse, trending | xAI X Search | Real-time X/Twitter data |
| Web pages with image/video analysis | xAI Web Search | Grounded web + multimodal |
| General knowledge fallback | WebSearch / Brave | Broad coverage |

**Exa-first** for code and technical content. **xAI for social/real-time pulse.**
Default to Exa unless the query is clearly better suited to another provider.
Use xAI X Search for "what are people saying about", sentiment, discourse,
trending topics. Use xAI Web Search when you need grounded web results with
optional image understanding. See `references/xai-search.md` for API details.

## Anti-Patterns

- Asserting model versions from training data without searching
- Using WebSearch for code examples (Exa code context is better)
- Skipping research because "I'm pretty sure" (you're not)
- Research without citations (every claim needs a URL)
