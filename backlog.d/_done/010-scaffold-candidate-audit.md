# Identify scaffold candidates across all skills

Priority: low
Status: done
Estimate: S

## Goal

Review all global skills for repo-specificity. Classify each as: stays global, becomes scaffold, or reads project config.

## Classification

| Skill | Generic | Repo-specific | Decision | Rationale |
|-------|---------|---------------|----------|-----------|
| investigate | 70% | 30% | Global + config | Methodology (4-phase protocol, iron law, flaky test taxonomy) is universal. Monitoring tooling (Sentry, Vercel, Convex CLI) and domain fix guides (Bitcoin, Stripe) are hardcoded in triage.md and fix.md. Config file for tooling; excise domain guides. |
| agent-readiness | 90% | 10% | Global | Pillar checks are already polyglot (enumerate alternatives per ecosystem). Fixes use runtime detection, not hardcoded tools. Optional config for threshold overrides only. |
| deps | 95% | 5% | Global | Ecosystem routing via lockfile detection is universal. Reachability analysis and behavioral diff are per-ecosystem, not per-repo. No changes needed. |
| code-review | 70% | 30% | Global + config | Reviewer bench and synthesis workflow are universal. Live verification trigger patterns (`.tsx`, `pages/`, `routes/`) are Next.js-specific. Config for file patterns and dev command. |
| settle | 95% | 5% | Global | All three phases (fix, polish, simplify) are universal Git/GitHub mechanics. Only `backlog.d/`/`git-bug` references are harness-kit-specific; low severity. No changes needed. |
| autopilot | 60% | 40% | Global + config | Orchestration skeleton is universal. Backlog source (`backlog.d/` vs GitHub Issues vs Linear), CI runner (`dagger` vs `make` vs `npm`), and observability stack (Canary/Sentry/PostHog) are hardcoded. Config for backlog/CI; observability block is the strongest scaffold sub-component candidate. |

## Key Finding

**None of the 6 skills warrant full scaffold templates.** The consistent pattern: methodology stays global, tooling integration points become config-driven. Only autopilot's observability block has enough project-specific substance to consider scaffolding, and even that is a sub-component.

## Follow-Up Work

### Config-driven skills (create backlog items if pursued)

- **investigate**: Add `.investigate.yaml` schema (monitoring tool, log commands, health endpoints, required env vars). Extract hardcoded Sentry/Vercel/Convex from `triage.md` and `investigation-protocol.md`. Excise domain fix guides from `fix.md` (Bitcoin, Lightning, Stripe, etc.) — they belong in project-local audit checklists.
- **code-review**: Add config for live verification trigger patterns and dev command. Current hardcoded patterns only work for JS/TS web apps.
- **autopilot**: Add config for backlog source and CI runner detection. Observability block in `references/qa-and-demo.md` could become a scaffold sub-template.

### Tech debt discovered

- `investigate/references/triage.md` references 3 non-existent files: `verify-fix-protocol.md`, `postmortem-template.md`, `incident-response-workflow.md`.
- `investigate/references/fix.md` lines 48-80 contain project-domain checklists (Bitcoin, Lightning, Stripe, Docs, Observability, Bun) that belong in project-local references, not a global skill.

## Oracle
- [x] Classification table complete: each skill marked as global / scaffold / config-driven
- [x] For each "scaffold" or "config" candidate: specific rationale for what's repo-specific
- [x] Recommendations don't bloat the skill count — prefer config over scaffold where possible

## What Was Built

Research-only deliverable. Analyzed all 6 candidate skills via parallel sub-agent investigation. Each skill was read in full (SKILL.md + all references/) and classified by what percentage of its content is generic vs repo-specific.

The audit confirms the scaffold system's current scope (qa + demo) is correct — those are the skills where project-specific content IS the skill. The remaining global skills have universal methodology with thin tooling integration points that config files handle cleanly.
