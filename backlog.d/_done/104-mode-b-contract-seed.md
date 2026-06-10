# Backlog: Mode B Contract Seed — Event Plane Boundary + First Workload

Priority: P1
Status: ready
Type: harness architecture / cross-repo contract

## Problem

Event-driven workflows (code review on PR-ready, production-error
diagnose/fix/postmortem, review-comment response, the flywheel outer loop)
do not belong in the ad-hoc operator harness, but Harness Kit currently
carries them as chat skills. The destination plane now exists: bitterblossom
is unarchived as the event-driven control plane (Fly Sprites substrate),
Daedalus is the eval-driven lab that designs/tunes the agents, and Cerberus
(archived ×3) was a prior attempt at exactly the first workload.

What killed Cerberus: parallel specialized reviewers with no coordinator,
no cost controls — expensive and noisy. Cloudflare's production system
(blog.cloudflare.com/ai-code-review; IndyDevDan breakdown clipped in
daybook: "I Ranked Cloudflare's Software Factory") fixes precisely those
failures:

- **Coordinator as filter:** sub-reviewers emit structured XML; coordinator
  dedupes, kills nitpicks/speculation/false positives, verifies uncertain
  findings by reading source, posts ONE structured review. Bias toward
  approval; `break glass` human override.
- **Tokenomics ($1.19/review median):** tiered model stack (state-of-the-art
  / workhorse / lightweight e.g. Kimi-class), risk-tiered compute (trivial
  <10 lines → coordinator + 1 generalist; light <100 lines → 4 agents; full
  pipeline only >100 lines / >50 files), shared context file + per-domain
  patch reads (avoids 7× diff duplication), aggressive prompt caching.
- **Resilience:** JSONL streaming output, step-finish events for retry /
  model fallback chains, error classification, incremental re-review with
  prior context.
- **Prompts name what to IGNORE** per specialty, not just what to find.

Stripe's complement (minions): activation energy beats execution — Slack
emoji reaction spawns a one-shot agent in a cloud devbox (Goose), ~1,300
PRs/wk, humans gate merges. Their devboxes ≈ our sprites.

## Goals

1. **One-page contract doc in Harness Kit** defining the shared disk surface
   both modes read/write: `backlog.d/` format + closure trailers, lane-card
   shape, delegation-receipt JSONL, evidence dir conventions. Harness Kit
   defines; bitterblossom consumes. Every Mode B flow must also be runnable
   ad hoc from a terminal (platform webhook is just one trigger — local-first,
   GitHub-compatible, not GitHub-dependent).
2. **First workload:** resurrect Cerberus's mission as a bitterblossom
   workload spec — coordinator + specialized reviewers with the Cloudflare
   economics above. Reuse thinktank where it fits (thin Pi bench launcher,
   parallel agents, artifacts — already the right shape for the reviewer
   fan-out layer).
3. **Migration hooks:** when (2) is live, Harness Kit's code-review skill
   shrinks to the thin dispatch prompt (per 103) and monitor/deploy/flywheel
   follow as later workloads.

## Portfolio boundary (anti-sprawl rule)

Each repo holds exactly one slot; two repos in one slot → merge:

| Slot | Repo |
|---|---|
| Ad-hoc judgment layer + shared contracts | harness-kit |
| Event-driven control plane (runtime) | bitterblossom |
| Eval-driven agent design lab | daedalus |
| Workload specs (review, incident, …) | bitterblossom workloads (Cerberus mission absorbed) |
| Parallel-bench launcher library | thinktank |

Open research before building: why Cloudflare chose OpenCode (open source,
SDK for programmatic sessions, internal familiarity) and Stripe chose Goose —
run the same evaluation for our substrate (Pi vs OpenCode vs Goose vs
provider CLIs) through Daedalus's arena loop rather than picking by vibe.

## Non-Goals

- No daemon/fleet machinery in Harness Kit (sprites stays one primitive).
- No rebuilding Cerberus as parallel-only reviewers (the coordinator and the
  cost tiers are the lesson).
- No always-on requirement for v1 (CI/event-triggered is enough; ZTE later).

## Acceptance Oracle

Scoped to Harness Kit per the portfolio boundary above — the workload build
itself belongs to the bitterblossom plane and is tracked there as
`bitterblossom/backlog.d/028-code-review-factory.md` (carries the
Cloudflare economics constraints and the end-to-end/cost/local-trigger
oracle).

- [x] `meta/CONTRACTS.md` exists (≤ 1 page): Mode A/B boundary, backlog +
      trailer canon, lane cards, receipts, evidence paths, Mode B roadmap.
- [x] `/code-review` reduced to the ad-hoc dispatch form; the orchestrated
      event form is named as bitterblossom's first workload.
- [x] monitor/deploy/flywheel removed from Mode A and routed to the Mode B
      roadmap in the contract doc.
- [x] bitterblossom ticket 028 filed with design constraints and oracle.
