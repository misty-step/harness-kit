# Delegate

> You orchestrate. Sub-agents do the work.

Reference pattern for dispatching work to sub-agents and synthesizing results.

## Your Role

You don't investigate/review/implement yourself. You:
1. **Route** — Send work to appropriate sub-agents
2. **Collect** — Gather their outputs
3. **Curate** — Validate, filter, resolve conflicts
4. **Synthesize** — Produce unified output

## Sub-Agent Archetypes

| Archetype | When to use |
|-----------|-------------|
| **planner** | Decompose work, write specs, scope decisions |
| **builder** | Implement, test, fix, gather evidence |
| **critic** | Evaluate output quality, grade against criteria |
| **Explore** | Codebase research, file discovery, pattern mapping |
| **philosophy bench** | Design review — spawn ousterhout, carmack, grug, beck in parallel |

### External tools (non-agent)

| Tool | Best for |
|------|----------|
| Thinktank CLI | Multi-model consensus, architecture validation |
| /research | Web search, prior art, reference implementations |

## How to Delegate

State goals, not steps. Give the sub-agent the objective and let it figure
out the path. Include constraints and verify commands, but don't micromanage.

**Good:** "Investigate this stack trace. Find root cause. Propose fix with file:line."

**Bad:** "Step 1: Read file X. Step 2: Check line Y. Step 3: ..."

## Parallel Execution

Spawn independent sub-agents simultaneously — they run concurrently. Use this
when tasks don't depend on each other: one reviews the backend API, another
audits frontend components, a third analyzes test coverage. All in one message.

## When to use which pattern

| Signal | Parallel sub-agents | Agent teams | Single agent |
|--------|-------------------|-------------|--------------|
| Independent tasks | YES | overkill | too slow |
| Workers must discuss | no | YES | no |
| Competing hypotheses | no | YES | no |
| Simple implementation | no | no | YES |

## Dependency-Aware Orchestration

For large work (10+ subtasks, multiple phases), use DAG-based scheduling:

```
Phase 1 (no deps):    Tasks 01, 02, 03 → spawn in parallel
Phase 2 (deps on P1): Tasks 04, 05     → blocked until P1 complete
Phase 3 (deps on P2): Tasks 06, 07, 08 → blocked until P2 complete
```

Use task tracking to manage phases: decompose into atomic tasks with
dependencies, spawn all unblocked tasks simultaneously, mark completed,
check newly-unblocked, spawn next phase.

## Curation (Your Core Job)

For each sub-agent finding:

- **Validate**: Real issue or false positive?
- **Filter**: Generic advice? Style preference contradicting conventions?
- **Resolve conflicts**: When sub-agents disagree, explain tradeoff, recommend

## Output Template

```markdown
## [Task]: [subject]

### Critical
- [ ] `file:line` — Issue — Fix: [action] (Source: [agent])

### Important
- [ ] `file:line` — Issue — Fix: [action] (Source: [agent])

### Synthesis
**Agreements** — Multiple agents flagged: [issue]
**Conflicts** — [Agent A] vs [Agent B]: [your recommendation]
```

## Related

- `/harness-engineering` — Harness engineering and context lifecycle
- `/code-review` — Multi-agent review implementation
- `/research thinktank` — Multi-model synthesis
