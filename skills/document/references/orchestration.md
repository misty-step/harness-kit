# Orchestration — the multi-agent loop

The loop is a team of subagents converging on the oracles, not a fixed pipeline.
Lane cards below; compose `/groom`'s `references/investigation-bench.md` for the
recon muscle rather than reinventing it.

## Build-order discipline (read first)

Model-driven, ad-hoc fan-out **first**. By the harness-engineering contract,
do not pre-author a deterministic orchestration-workflow template —
*"build one only after telemetry shows the pattern recurring; never
pre-author."* Standing up the big deterministic harness on day one is the
deterministic-scaffold failure mode this repo keeps relapsing into.

Name the orchestration substrate by **capability**, so the skill degrades across
harnesses:

- Harness with a large-scale background-orchestration feature (e.g. a Workflow
  tool with `pipeline`/`parallel`/loop-until-dry): use it for the verify loop
  once the pattern has proven recurring; it is the natural fit for
  generate→verify→iterate over many pages.
- No such feature: parallel native subagents, or a `/sprites` fleet for heavy
  runs. A harness without the accelerator loses nothing it cannot do by hand.

Scale agent count to repo size and ambition; `log` any coverage cap (top-N
modules, sampled history) so silent truncation never reads as completeness.

## Lanes

Each lane is outcome-shaped; the lane agent owns its own decomposition. Give it
end state, scope boundary, and output shape — not atomized steps.

### Recon swarm (parallel, lens-blind)

One scout per lens, each blind to the others so no single search angle bounds
coverage:

- **entry-points** — main/handlers/CLI/jobs/cron; where execution begins.
- **data-flow** — how data moves and is persisted.
- **dependency-graph** — module/package coupling and boundaries.
- **config + infra** — env, flags, deploy topology, CI.
- **tests-as-spec** — what tests assert the system *should* do.
- **git-why** — history, ADRs, comments for non-obvious decisions and lore.

Output: a structured repo map per lens. Merge into one map for planning.

### IA planner (single synthesizer)

Input: merged repo map. Output: the page tree + diagram set, chosen from
`references/information-architecture.md` types and **adapted to the system
shape**. Decides which subsystems get a deep page vs a stub. The coverage oracle
checks this plan against the real surface before generation.

### Page generators (pipeline, facet-scoped)

One agent per page, scoped to its slice with just-enough context, produced in
dependency order (overview before deep-dives so deep pages can cross-link up).
Pipeline, not barrier: page B drafts while page A is being verified. Each
generator stamps provenance (`templates/page.md`).

### Verifiers (adversarial, per oracle)

- **Accuracy skeptics** — fresh-context, decorrelated model family, prompted to
  refute each claim against source. Only the page + repo, never the author's
  reasoning trail.
- **Cold-reader navigator** — only the docs + real tasks; names files it would
  open. Misses are IA bugs.

Findings feed back to generation. Iterate until the loop-until-dry stop rule in
`references/oracles.md` is met.

### Render + stamp (final)

One generation → md source + HTML + mermaid (`references/render-contract.md`).
Run the render oracle, refresh provenance/`verified` stamps, commit.

## Canonical shape (when the Workflow asset is eventually built)

Pipeline by default — each page verifies as soon as it is generated, so accuracy
checks on early pages overlap generation of later ones. Barrier only where a
stage genuinely needs all prior results: dedup the merged repo map before
planning; early-exit if recon found nothing to document. Keep this as guidance
for the future asset, not a script committed now.
