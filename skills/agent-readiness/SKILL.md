---
name: agent-readiness
description: |
  Assess and improve codebase readiness for AI coding agents across style,
  tests, docs, architecture, CI, observability, security, and dev setup.
  Produces a scored report, prioritizes remediation, then executes the
  highest-impact fixes.
  Use when: "agent readiness", "is this codebase agent-ready",
  "readiness report", "make this codebase agent-friendly",
  "agent-ready assessment", "readiness audit", "prepare for agents".
  Trigger: /agent-readiness, /readiness.
argument-hint: "[--assess-only] [--fix] [--pillar <name>]"
---

# /agent-readiness

Assess how well this codebase supports autonomous AI coding agents.
Then fix the highest-impact gaps.

**Target:** $ARGUMENTS

## Execution Stance

You are the executive orchestrator.
- Keep prioritization, remediation approval, and final readiness judgment on the lead model.
- Delegate pillar assessments and bounded fixes to focused subagents.
- Use parallel fanout by default for independent pillars.

## Core Insight

The agent is not broken. The environment is. A codebase with fast feedback
loops and clear instructions makes any agent dramatically more effective.
A codebase with poor feedback loops defeats any agent you throw at it.

## Workflow

### Phase 1: Assess

Dispatch **parallel subagents** — one per pillar — to evaluate the codebase.
Each subagent runs the checks from `references/pillar-checks.md` for its
assigned pillar and returns a structured verdict.

Launch all pillar assessments simultaneously:

| Subagent | Pillar | What it checks |
|----------|--------|---------------|
| 1 | Style & Validation | Linters, formatters, type checkers, pre-commit hooks |
| 2 | Build & CI | Build commands, dependency pinning, CI config, feedback speed |
| 3 | Testing | Coverage, speed, local execution, unit/integration/E2E layers |
| 4 | Documentation | CLAUDE.md/AGENTS.md, README, setup guide, architecture docs, ADRs |
| 5 | Dev Environment | Reproducibility, env templates, devcontainers, isolated workspaces |
| 6 | Code Quality | Modularity, complexity, file organization, coupling, dead code |
| 7 | Observability | Structured logging, error handling, metrics, tracing |
| 8 | Security & Governance | Branch protection, CODEOWNERS, secret scanning, dependency audit |

Each subagent uses **Explore** agent type and reads `references/pillar-checks.md`
for its pillar's specific pass/fail criteria. Output format per subagent:

```markdown
## [Pillar Name]
Score: X/Y checks passed
Maturity: L1-L5

### Passing
- [check]: [evidence]

### Failing
- [check]: [what's missing] → [recommended fix]

### Highest-Impact Fix
[The single change that would most improve this pillar]
```

### Phase 2: Report

Synthesize subagent results into a single readiness report:

```markdown
# Agent Readiness Report: [project-name]

## Overall: Level X — [Maturity Name] (XX%)

| Pillar | Score | Level | Top Fix |
|--------|-------|-------|---------|
| Style & Validation | 4/6 | L3 | Add pre-commit hooks |
| Testing | 2/7 | L1 | Add unit test runner |
| ... | ... | ... | ... |

## Maturity Levels
- L1 Functional: Code runs, but agents need hand-holding
- L2 Documented: Processes written down, some automation
- L3 Standardized: Enforced automation, agents handle routine work
- L4 Optimized: Fast feedback, data-driven improvement
- L5 Autonomous: Self-improving, sophisticated orchestration

## Top 5 Recommendations (by impact)
1. [highest impact fix across all pillars]
2. ...

## Detailed Findings
[per-pillar sections from subagents]
```

**Maturity level is gated:** must pass 80% of criteria at current level
and all previous levels before advancing. This prevents cherry-picking.

### Phase 3: Clarify

Present the report and top 5 recommendations to the user. Ask:

1. Which recommendations should we execute now?
2. Any pillars to skip or deprioritize?
3. Any constraints (don't change CI provider, keep current test framework, etc.)?

**One question at a time.** Don't dump all three at once.

### Phase 4: Fix

For each approved recommendation, spawn a **builder** subagent (or use
worktrees for parallel fixes on disjoint files). Each fix follows the
project's existing patterns — don't introduce new tools the team hasn't
chosen.

Typical fix categories:

| Fix type | Example | Subagent approach |
|----------|---------|-------------------|
| Config addition | Add `.editorconfig`, pre-commit hooks | Single builder, quick |
| Documentation | Create/update CLAUDE.md, AGENTS.md | Single builder |
| Test infrastructure | Add test runner, coverage config | Builder + TDD cycle |
| CI enhancement | Add lint/typecheck/test to CI | Single builder, verify locally |
| Architecture doc | Create ADR, architecture overview | Builder reads codebase first |
| Security hardening | Add CODEOWNERS, branch protection | Builder + gh CLI |

After fixes: re-run the failing checks to verify improvement. Report
the before/after delta.

### Phase 5: Re-assess (optional)

If `--fix` was used, re-run the full assessment to show the improved score.
Present the before/after comparison.

## Routing

| Argument | Behavior |
|----------|----------|
| (none) | Full assess → report → clarify → fix cycle |
| `--assess-only` | Assess and report only, no fixes |
| `--fix` | Skip clarification, fix all top 5 recommendations |
| `--pillar <name>` | Assess only the named pillar |

## Pillar Check Reference

All specific checks are in `references/pillar-checks.md`. Each check is:
- **Binary**: pass or fail (no subjective scoring)
- **Evidence-based**: the subagent must cite the file/config/output that proves pass/fail
- **Actionable**: every failing check has a concrete remediation

See `references/agent-readiness-principles.md` for the deeper "why" behind
each pillar — useful when explaining recommendations to the user.

## Gotchas

- **Scoring without fixing is theater.** The report is only useful if it
  leads to action. Always push toward Phase 4.
- **Don't introduce tools the team hasn't chosen.** If they use Jest, don't
  suggest Vitest. If they use ESLint, don't add Biome. Work within existing
  choices.
- **Pre-commit hooks are the highest-leverage single fix** for most codebases.
  They give agents instant feedback instead of waiting for CI.
- **CLAUDE.md/AGENTS.md is the highest-leverage documentation fix.** It's the
  file agents actually read. A perfect README that agents ignore is worth less
  than a scrappy CLAUDE.md they always load.
- **Don't conflate coverage percentage with test quality.** 80% coverage with
  shallow tests is worse than 50% coverage with behavior-focused tests.
  Check for assertion density, not just line coverage.
- **Monorepos need per-app assessment.** A monorepo score is the floor of
  its worst app, not the average.
- **Speed matters as much as existence.** A test suite that takes 20 minutes
  is nearly as bad as no tests for agent workflows. Measure execution time.
- **The codebase is the product, not the agent.** Every fix here improves
  the experience for ALL agents and ALL developers, not just one tool.
