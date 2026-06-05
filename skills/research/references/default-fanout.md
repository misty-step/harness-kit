# Default Fanout

Multi-source triangulation for substantive research.

## Use

Load this reference when the user asks broad research, comparison, architecture
prior art, model/provider investigation, "what are people saying", or any
question where a single lookup would overfit one source.

Single-source research is allowed only when the user names the source or the
task is a narrow fact/version lookup.

## Context Packet

Capture the packet before launching lanes:

- Objective: fact lookup, prior-art scan, architecture comparison, discourse
  scan, or decision support.
- Scope: repos, files, domains, products, dates, jurisdictions, or excluded
  areas.
- Freshness tolerance: latest/current, date-bounded, or stable background.
- Acceptance oracle: decision to support, risk to refute, artifact to produce,
  or explicit absence.
- Tool constraints: unavailable credentials, provider limits, offline mode, or
  user-named sources.

## Source Invariant

Default research requires independent evidence from these capability lanes:

| Lane | Capability | Primary refs |
|---|---|---|
| Retrieval | web, docs, papers, reference implementations | `web-search.md`, `exa-tools.md` |
| Recency / discourse | current web, X/social discourse, contradiction checks | `xai-search.md` |
| Repo-aware critique | local fit, architecture tradeoffs, second opinion | `thinktank.md`, `delegate.md` |
| Codebase | live repo patterns, existing contracts, local prior art | `rg`, `git`, local files |

If a lane fails, times out, lacks credentials, or is intentionally skipped by
scope, keep its section and label the status. Do not silently collapse failed
lanes into synthesis.

Capability lanes do not replace the roster floor. For substantive research,
also dispatch the provider lanes required by `harnesses/shared/AGENTS.md`
(Roster) unless a documented waiver applies.

## Capability Routing

| Intent | Prefer | Fallback |
|---|---|---|
| Code examples or reference implementations | Exa code context | GitHub/source search, web search |
| Library docs or API usage | docs capability such as Context7 | official docs via web search |
| Model releases, pricing, CVEs, deprecations | recency-filtered web/xAI | Exa recency, official sources |
| Social sentiment or public discourse | xAI X Search | web results that cite public posts |
| Repo architecture or local tradeoffs | Thinktank / roster lanes | scoped grep plus lead analysis |
| Saved user reading/highlights | Readwise | local notes or explicit web search |

Route by capability. Vendor names are implementation details; if a named
provider is unavailable, use the closest source and report the substitution.

## Thinktank State

Thinktank is an in-flight bench, not an instant lookup. Before waiting, record:

- Mode: `quick` or `deep`.
- Output directory.
- Time budget.
- Starting paths or repo root.

If incomplete, report `Thinktank (partial)` with the output directory and only
the artifacts that exist, such as `manifest.json`, `trace/events.jsonl`,
`task.md`, `prompts/`, or completed `agents/` reports.

## Report Shape

Use this shape for default fanout reports:

```markdown
## Synthesis
[Lead conclusion, confidence, decision impact, and residual uncertainty.]

## Source Matrix
| Source lane | Status | What it contributed | Key refs |
|---|---|---|---|
| Retrieval | complete/partial/failed/skipped | ... | URLs/artifacts |
| Recency / discourse | complete/partial/failed/skipped | ... | URLs/citations |
| Repo-aware critique | complete/partial/failed/skipped | ... | receipt ids/output dir |
| Codebase | complete/partial/failed/skipped | ... | file:line/commands |

## Conflicts
[Disagreements across sources and the lead's resolution.]

## Evidence
[Grouped citations, commands, receipts, local files, or artifacts.]

## Residual Risk
[Stale facts, missing providers, unqueried sources, or none with reason.]
```

For source-heavy reports, keep per-source detail below the matrix. Readers must
be able to tell what each lane contributed independently.

## Failure Labels

- `complete`: lane produced usable evidence inside scope.
- `partial`: lane started but only some artifacts or results exist.
- `failed`: lane was attempted and produced no usable evidence.
- `skipped`: lane was out of scope, user-forbidden, or impossible due missing
  credentials/tooling.
- `stale`: evidence may be outdated for the requested freshness tolerance.

Every recommendation should survive removing the weakest source. If it does
not, label the recommendation low confidence.
