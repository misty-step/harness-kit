---
name: document
description: |
  Generate world-class, source-verified reference documentation for a codebase:
  a multi-agent loop that surveys the repo, plans the information architecture,
  writes facet-scoped pages, and adversarially verifies every claim against live
  source before committing markdown + HTML + diagrams to docs/. Always runs the
  full verify loop; scope is incremental by provenance. Use when: "document this
  codebase", "generate the docs", "build a codebase wiki", "write architecture
  docs", "onboarding docs", "documentation site", "keep the docs in sync",
  "world-class docs". Trigger: /document, /docs, /wiki.
argument-hint: "[scope-path | --full | --check]"
---

# /document

Documentation that earns trust because **every claim is verified against live
source** — not a plausible summary that drifts.

This skill owns the gap between "the code exists" and "a new contributor or
maintainer can navigate, understand, and safely change this system from the
docs." It is committed, human-facing reference documentation living in `docs/`,
rendered to markdown + HTML + diagrams from one source. The wedge over every
auto-wiki product (Factory AutoWiki, Cognition DeepWiki) is the one thing they
skip: **adversarial, source-grounded verification of each claim** — the antidote
to authoritative-looking lies committed into the repo.

The named phases (survey → plan → write → critique → iterate) are SDLC the model
already knows. Do not narrate them. The skill's value is the **oracles, the IA
taste, the provenance contract, and the orchestration discipline** below.

## Route

| Need | Load |
|---|---|
| The verification system — how "comprehensive / accurate / well-organized" become falsifiers | `references/oracles.md` |
| Information architecture — Diátaxis mapping, page taxonomy, which diagrams earn their place | `references/information-architecture.md` |
| Committed-docs provenance, freshness falsifier, incremental scope, Mode B handoff | `references/provenance-and-freshness.md` |
| The multi-agent loop topology, lane cards, build-order discipline | `references/orchestration.md` |
| Output surfaces: md source → HTML + mermaid, "synced across surfaces", publish target | `references/render-contract.md` |
| Per-page provenance front-matter + world-class page skeleton | `templates/page.md` |
| Freshness oracle driver (stale-page detector) | `scripts/freshness.py` |

`--check` runs only the freshness + render oracles against existing `docs/` (no
generation). `--full` forces full-repo regeneration instead of incremental scope.

## The loop, where the judgment lives

A table of lanes, not a procedure. Each lane is outcome-shaped; the lead owns
decomposition. Detail and lane cards in `references/orchestration.md`.

| Lane | Non-obvious judgment | Oracle it feeds |
|---|---|---|
| Recon swarm | lens-blind agents (entry-points / data-flow / dep-graph / config+infra / tests-as-spec / git-*why*); each blind to the others so no single search angle bounds coverage | — |
| IA plan | structure **adapts to the system** — a CLI ≠ a service ≠ a library; pick pages and diagrams from what recon found, not a fixed template | coverage |
| Generation | one agent per page/facet, just-enough context, dependency order (overview before deep-dives); pipeline, not barrier | — |
| Verify | adversarial skeptics on a **decorrelated model family** refute each claim against source; cold-reader proves navigability | accuracy, navigability |
| Iterate | loop-until-dry: regenerate flagged pages until K consecutive rounds raise no blocking oracle failure | all |
| Render + stamp | one generation → md + HTML + diagrams; stamp provenance; commit | render, freshness |

**Build-order discipline:** model-driven ad-hoc fan-out first. Do **not**
pre-author a deterministic orchestration-workflow template — by the
harness-engineering contract, crystallize that asset only once telemetry shows
the pattern recurring. Standing up the machinery on day one is the
deterministic-scaffold failure mode.

## Contract

- **Source-grounded or it does not ship.** Every architectural claim maps to
  specific source lines a skeptic verified, or it is cut/flagged. Unverifiable
  prose is the failure this skill exists to prevent.
- **Committed pages carry provenance.** Each page stamps `generated-at-sha` and
  `covers:` globs (`templates/page.md`). This makes drift *detectable* instead
  of silent — the load-bearing mitigation for committing docs into the repo.
- **Always world-class quality bar; incremental scope.** The full verify loop
  runs every time. On re-run, regenerate only pages whose covered source changed
  (plus cross-link neighbors) — quality constant, work proportional to the diff.
- **IA adapts to the repo.** Borrow page *types*, never a fixed tree. Let the
  system's shape choose the structure.
- **Compose, don't reinvent.** Aesthetic/HTML via `/design` +
  `anthropic-frontend-design`; publish machinery via `/showcase`; recon muscle
  via `/groom`'s investigation bench. This skill owns the oracles and IA taste.
- **Freshness is checkable.** `scripts/freshness.py` is the driver; a page
  covering changed files past its stamped SHA is stale, full stop.
- **Operator owns IA and publish choices.** Page tree, depth, diagram set, and
  where HTML publishes are the operator's call when ambiguous.

## Boundaries (what this is not)

- **Not the agent's context substrate.** `/orient` owns ephemeral, live-evidence
  session grounding. Agents read source, not a possibly-stale wiki *about*
  source. These docs are for humans.
- **Not marketing/demo.** `/showcase` owns external proof and launch copy. This
  is technical reference.
- **Not external-repo lookup.** The vendored `deepwiki` skill queries
  third-party OSS wikis. This documents *your* repo.
- **Auto-refresh-on-push is Mode B → bitterblossom.** This skill is on-demand
  (Mode A). The freshness script is the trigger contract for that future loop;
  see `harnesses/shared/references/loop-readiness.md`.

## Delegation Judgment

Delegate per the shared Roster contract in `harnesses/shared/AGENTS.md`
(Roster). This skill is parallel-by-default and a heavy token spender; route
heavy/long runs to `/sprites`. Lanes:

- **Recon scouts** — one per lens, blind to the others (compose `/groom`'s bench).
- **Page generators** — one per facet, just-enough context, dependency order.
- **Accuracy skeptics** — fresh-context, **different model family**, prompted to
  *refute* each claim against source. Give them only the page + the repo, never
  the author's reasoning trail.
- **Cold-reader navigator** — sees only the generated docs + a real task; must
  land in the right files. Wrong landing = bad IA.

## Gotchas

- **Plausible ≠ correct, and here it commits lies.** A confident wrong claim,
  committed into `docs/`, is worse than no doc — it carries the repo's authority.
  The accuracy oracle is not optional; it is the whole reason to build this.
- **Committed docs that drift are the silent killer.** No error, just steady
  divergence. Provenance stamps + the freshness oracle are mandatory given the
  committed-in-repo choice, not polish.
- **Coverage is not accuracy.** Documenting every symbol while one flow is
  described wrong still fails. Run both oracles; they catch different errors.
- **Too much context degrades agents** — which is exactly why these docs target
  humans and `/orient` keeps agents on live source. Do not repurpose this output
  as the agent's primary context feed.
- **A verifier inheriting the author's context rationalizes the author.** Hand
  critics the artifact and the repo, nothing else.
- **Restating code is not documentation.** A page that paraphrases functions
  adds drift surface and no understanding. Capture intent, flow, and *why*.
- **Diagrams that do not parse are worse than none.** The render oracle fails the
  build on broken mermaid or dead internal links — no silent ship.
- **Copied IA is generic IA.** A fixed page tree applied to every repo reads as
  autogenerated. Let recon choose the structure.
- **Silent truncation reads as completeness.** If scope was bounded (top-N
  modules, sampled history), the run must `log` what it skipped.

## Completion Gate

See `harnesses/shared/AGENTS.md` (Completion Evidence) for the shared core.
`/document` adds:

```markdown
## Document Gate
- Coverage: real export/route/entry surface vs documented surface; named gaps or waiver.
- Accuracy: each architectural claim source-grounded by a fresh-context skeptic; refuted/flagged claims listed.
- Navigability: cold-reader landed on the right files for the test task(s), or where it failed.
- Render: HTML built, mermaid parsed, zero broken internal links — command + result.
- Provenance: every committed page stamped with generated-at-sha + covers globs.
- Freshness: scripts/freshness.py result against HEAD; stale pages or clean.
- Scope: full vs incremental, and what was intentionally not regenerated.
- Publish: where HTML renders, or that it stayed local pending operator choice.
```
