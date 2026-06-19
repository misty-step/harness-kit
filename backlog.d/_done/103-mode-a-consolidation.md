# Backlog: Harness Kit v2 — Mode A Consolidation

Priority: P0
Status: ready
Type: harness architecture / deletion

## Problem

Harness Kit conflates two modes of agent work and pays for it in bloat:

- **Mode A (this repo's actual job):** ad-hoc, operator-driven sessions. The
  harness loads judgment + context into a session a frontier model drives.
- **Mode B (not this repo's job):** event-driven workflows (review on
  PR-ready, diagnose on production error, respond to review comments). Those
  belong to a separate CI-native/event plane (bitterblossom, with Daedalus as
  the eval-driven design lab). Stripe (minions: Slack-reaction → one-shot
  agent in cloud devbox, ~1,300 PRs/wk) and Cloudflare (CI-native OpenCode
  orchestration, 7 specialized reviewers + coordinator, $1.19/review, 0.6%
  override) both prove the pattern: the authoring agent never runs these.

Audit evidence (2026-06-10 session, mined from ~939 Codex sessions / 60d and
~981 Claude sessions / 90d):

- 36 first-party skills, ~22k lines of prose (6.6k SKILL.md + 15k references).
- Usage is a power law: groom (593 sessions), code-review (304), trace (274),
  deliver (192), then a cliff; ~15 skills effectively unused.
- 32.7k-line Rust crate across ~55 modules, most enforcing process prose
  rather than catching real failures.
- 363 commits to this repo in 90 days — meta-work ratio inverted.
- Anthropic's own skill taxonomy (June 2026, "How we use skills") has NO
  SDLC-phase-orchestration category; highest-impact category is per-product
  verification. Their top lessons: "don't state the obvious," "avoid
  railroading." Dynamic workflows make static generic pipelines the inferior
  option.
- Root cause of catalog shape: slash commands were collapsed into skills when
  skills arrived. Many "workflow skills" are saved prompts, not skills.

## Goals

Three primitives, ruthlessly sorted:

| Primitive | Test |
|---|---|
| Prompt | "Is this just what I'd retype to a strong model?" One file, no machinery. |
| Skill | "Does this change what a frontier model does, for the better, repeatedly?" |
| Doctrine | "Worth paying for in every session?" One page. |

Disposition table (judgment, revisit per item during execution):

- **Keep as skills, rewritten (≈8):** deliver (rewrite as the delivery
  philosophy: context-first → docs→tests→code → lenses (Ousterhout, Carmack,
  Kent C. Dodds, Uncle Bob) → QA the live thing → refactor at three altitudes
  (diff/codebase/backlog) → semantic commits → diverse-provider review fan-out
  → adversarial pre-ship "what breaks and how will we know" → squash-merge →
  monitor → ta-da; ~150 lines, receipt shrinks to a few fields, 15-field
  completion gate dies); groom (slim, less prescriptive; absorbs
  agent-readiness as harness-gap auditing: missing per-repo verification
  skill / runbook / weak CI → tickets); research (keystone
  capability-awareness: Exa, Firecrawl, context7, retired-bench — beats hardcoded
  library-reference skills); qa/verify (absorbs browser + demo evidence +
  hardening judgment; expects per-repo verification skill, files gap via
  groom); sprites (keep as-is — new, well-shaped, Mode B substrate too);
  harness-engineering (absorbs skillify, create-repo-skill, lint/eval/sync);
  diagnose (ad-hoc form only); design (keep but resolve: aggregator of
  external taste skills vs picking one — currently frustrating, evaluate
  externals head-to-head on a real surface and keep the winner(s)).
- **Demote to prompts (≈6):** yeet, orient+debrief (merge), ship (squash-merge
  + trailers + archive + docs touch), critique (lens prompt), reflect
  (artifact-shaped: "mine session for rules → write into AGENTS/CLAUDE.md").
- **Split:** code-review → (a) thin dispatch prompt: fan out to diverse roster
  providers, synthesize, fix blockers; (b) orchestrated multi-reviewer system
  → Mode B (see 104). Do not delete (a)-as-used before (b) exists.
- **Move to Mode B roadmap:** monitor, deploy, flywheel (the outer loop is an
  event system pretending to be a chat skill).
- **Fold:** implement/refactor/hardening/demo → deliver+qa prose;
  agent-readiness → groom; agent-transcript → trace; model-research →
  research.
- **Delete:** settle, karpathy-guidelines (→ ~4 doctrine lines), dispatch
  (→ Roster doctrine + receipt script), create-repo-skill, skillify, a11y
  (→ paragraph in design/qa), deps (→ 2 doctrine lines).
- **Re-evaluate after receipts decision:** trace (high usage is machinery
  driven; operator never invokes it).

Supporting work in the same lane:

- **Truth pass first:** rewrite CLAUDE.md/AGENTS.md to match reality (agents/
  contents, bootstrap.sh role, current commands); land in-flight Dagger→Rust
  CI migration against the *shrunken* lane list.
- **Rust shrink:** keep modules that catch real failures (bootstrap,
  frontmatter/structure lint, index gen, external sync, delegation receipts,
  telemetry summarizer ≈ 3–5k lines). For each other module ask: "what real
  failure did this catch in the last 90 days?" No answer → delete.
- **Telemetry that runs:** hook-based skill/prompt load logging to JSONL
  (Claude PreToolUse hook; Codex session-log mining fallback) + one
  summarizer (usage, undertriggering, staleness). `/groom audit` reads it.
- **Externals:** vendor (commit) the curated set actually loaded; registry
  stays as provenance ledger (pin, license, upstream); resolve every
  `allow_floating` TODO by pinning; edited external = marked fork; drop
  embeddings discovery unless a real outcome is named.
- **Cross-harness posture:** Codex + Claude first-class. Install to
  `~/.agents/skills/` (converged ecosystem standard) and symlink
  harness-specific dirs from it. Parity gates die; parity format is free.
  Pi stays installed; its future is Mode B substrate.

## Non-Goals

- No Mode B implementation here (104).
- No new orchestration machinery, receipts schemas, or eval trees.
- No tailor-style per-repo full-harness rewriting (tried, was awkward;
  per-repo gaps flow through groom instead).

## Acceptance Oracle

- Skill count ≤ 12; prompts split out; total first-party prose ≤ ~6k lines.
- CLAUDE.md/AGENTS.md contain no statements contradicted by the live tree.
- Rust crate ≤ ~8k lines with every surviving module justified by a named
  failure it catches.
- Telemetry summarizer answers "what was used last 30 days" in one command.
- `registry.yaml` has zero unpinned non-default sources.
- Bootstrap installs and both Claude and Codex resolve the catalog.
