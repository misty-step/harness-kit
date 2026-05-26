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

## Delegation Floor

If a provider roster is available (repo `.spellbook/agents.yaml` or system
`~/.spellbook/agents.yaml`), `/research` starts by probing the roster and
dispatching two or more available roster
members for independent research lanes. Use lanes with distinct sources,
methods, or model families; web search, Thinktank, and provider CLIs are
evidence inputs, not substitutes for lead synthesis. Record meaningful
external attempts with the repo receipt tool. Direct lead-only research is
limited to mechanical lookup, emergency unblocks, explicit user-forbidden
delegation, or an explicit waiver when fewer than two roster members are available.
Do not check only the repo-local path. Run the resolver-backed probe, which
falls back from repo roster to the system Spellbook roster:
`python3 $SPELLBOOK_ROOT/scripts/probe-agent-roster.py`.
Native in-thread subagents are useful for fresh context, but they are not
external roster members. They do not count toward the two-provider floor.

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

**REQUIRED: Launch ALL of these in a single message (parallel Agent/Bash calls). Missing any default source makes the run incomplete. Do not intentionally skip Thinktank or xAI/Grok in default mode:**

1. **Exa search** — Bash: `curl -s https://api.exa.ai/search -H "x-api-key: $EXA_API_KEY" ...`
   See `references/exa-tools.md` for request format. WebSearch is fallback ONLY if curl fails.
2. **Thinktank** — Mandatory repo-aware bench for the default fanout.
   Run Thinktank from the repo you want it to inspect. Thinktank's workspace is context; `--paths` only orient the agents and do not replace the workspace root. If the query points at another repo, `cd` there before launch.
   Default fanout path: `thinktank run research/quick --input "$QUERY" --output /tmp/thinktank-out --json --no-synthesis`
   Deep path: `thinktank research "$QUERY" --output /tmp/thinktank-out --json`
   Add `--paths ...` when local files or directories should be pointed out as starting places.
   Before waiting, record the mode (`quick` or `deep`), output directory, and expected runtime band in your own notes.
   Default budgets: `quick` targets `60-180s` with a hard cap of `300s`; `deep` targets `3-8m` with a hard cap of `900s`.
   Thinktank launches a real multi-agent Pi bench, so even `research/quick` can take a few minutes.
   With `--json`, stdout stays reserved for the final envelope; a healthy run may look quiet until completion.
   Canonical inspection flow: `thinktank runs show /tmp/thinktank-out` and `thinktank runs wait /tmp/thinktank-out`. Inspect the output directory directly only when you need deeper detail: `trace/events.jsonl`, `manifest.json`, `task.md`, and `prompts/` are the current partial-progress artifacts.
   Current limitation: if you stop a run early, completed agent reports may not exist yet. Treat that as a current Thinktank limitation, not as evidence that nothing happened.
   If Thinktank fails or times out, keep the Thinktank section and label it `partial` or `failed` with the output directory and observed error. That is an incomplete fanout, not a justified skip.
3. **xAI / Grok** — Bash: `curl -s https://api.x.ai/v1/responses -H "Authorization: Bearer $XAI_API_KEY" ...`
   Model MUST be `grok-4.20-beta-latest-non-reasoning` (only grok-4 supports tool use).
   Use xAI for grounded web retrieval, recency verification, contradiction checks, X-native discourse, and multimodal evidence. See `references/xai-search.md` for request format.
4. **Codebase** — Grep/Glob for what the project already does (skip only if query is unrelated to codebase)

**Then produce a sourced report** using the mandatory structure below.

### Report Format (mandatory for all default fanout runs)

Every research report MUST have one labeled section per default source,
followed by a synthesis. Include the section even when a provider returns
partial output or fails; mark that status explicitly and carry the failure into
the synthesis.

```
## Exa (neural search)
[Findings with inline URLs. What did Exa specifically surface?]

## xAI / Grok ([web_search | x_search | both])
[Findings with citations from response.citations. What did Grok surface?
For X Search: quotes or paraphrases from X posts, authors, dates.]

## Thinktank (Pi bench)
[What did the thinktank bench surface? Label this section `Thinktank (complete)`, `Thinktank (partial)`, or `Thinktank (failed)`. Note any disagreements between agents, the output directory, and the specific failure mode when applicable.]

## Codebase
[What relevant patterns, implementations, or prior art exist locally?
"None found" is a valid answer — write it explicitly.]

## Synthesis
[Consensus across sources. Conflicts or contradictions between them.
Recommendations grounded in the evidence above. Every claim cites a source.]
```

**Discipline rule**: if a section is missing, you failed the fanout. A failed
provider still gets a section. A report that collapses all sources into one
unlabeled blob has failed the fanout goal.
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
| Repo-local architecture, tradeoffs, contradiction checks | Thinktank | Fresh multi-agent second voice against the workspace |
| Code examples, reference implementations | Exa (code context) | Finds actual code, not blog posts |
| Academic papers, formal specs | Exa (search) | Strong academic indexing |
| Library/framework docs | Context7 | Semantic doc search |
| Current events, model releases, fast recency checks | xAI Web Search | Grounded live web search with citations |
| Social sentiment, public discourse, trending | xAI X Search | Native X/Twitter search and discourse retrieval |
| Web pages with image/video analysis | xAI Web Search | Grounded web + multimodal |
| General knowledge fallback | WebSearch / Brave | Broad coverage |

Default fanout uses Exa, Thinktank, xAI/Grok, and Codebase together. This
routing table only tells you which source to lean on most heavily in synthesis;
it does not authorize skipping the others. Exa remains strongest for code and
technical indexing. xAI/Grok is broader than social pulse: use it for grounded
web retrieval, recency verification, contradiction checks, and X-native
discourse. Thinktank is the repo-aware second voice. See
`references/xai-search.md` and `references/thinktank.md` for the concrete
operator contract.

## Anti-Patterns

- Asserting model versions from training data without searching
- Using WebSearch for code examples (Exa code context is better)
- Skipping research because "I'm pretty sure" (you're not)
- Research without citations (every claim needs a URL)
