# Delegate

> You orchestrate. Sub-agents do the work.

Reference pattern for dispatching focused work to sub-agents and synthesizing
their evidence.

## Your Role

You remain the lead. You frame the work, dispatch lanes, compare evidence,
decide, verify, and report.

1. **Frame** — State the claim, question, or artifact each lane must test
2. **Route** — Send bounded work to appropriate sub-agents or providers
3. **Collect** — Gather outputs, receipts, files, commands, and artifacts
4. **Curate** — Validate, filter, resolve conflicts
5. **Synthesize** — Produce the lead conclusion and residual risk

## Research Lane Types

| Lane | Use |
|---|---|
| Source scout | Find URLs, papers, docs, examples, or saved reading |
| Repo scout | Map local files, contracts, tests, and prior art |
| Contradiction critic | Try to refute the leading claim or recommendation |
| Lens critic | Evaluate a design through a named lens such as ousterhout, carmack, grug, or beck |

## Dispatch Packet

Every lane gets:

- Role and objective.
- Scope: files, domains, commands, sources, and boundaries.
- Output shape and maximum length.
- Evidence requirement: URLs, file:line, command output, receipt id, or artifact.
- What not to touch.

State the goal, not a step-by-step script. Give the lane room to choose the
path inside the packet constraints.

**Good:** "Investigate this stack trace. Find root cause. Propose fix with file:line."

**Bad:** "Step 1: Read file X. Step 2: Check line Y. Step 3: ..."

## Dependency-Aware Orchestration

For large work (10+ subtasks, multiple phases), use DAG-based scheduling:

```text
Phase 1 (no deps):    Tasks 01, 02, 03 -> spawn in parallel
Phase 2 (deps on P1): Tasks 04, 05     -> blocked until P1 complete
Phase 3 (deps on P2): Tasks 06, 07, 08 -> blocked until P2 complete
```

Use task tracking to manage phases: decompose into atomic tasks with
dependencies, spawn all unblocked tasks simultaneously, mark completed,
check newly-unblocked, spawn next phase.

## Curation

For each sub-agent finding:

- **Validate**: Does the cited evidence support the claim?
- **Filter**: Generic advice, stale facts, source-quality gaps, or style
  preferences that contradict local conventions.
- **Resolve conflicts**: When sub-agents disagree, explain the tradeoff,
  recommend which evidence to trust, and name what would change the verdict.

## Output Template

```markdown
## [Task]: [subject]

### Accepted
- [claim] — evidence: [URL/file:line/command/receipt/artifact] — why it matters

### Rejected
- [claim] — reason rejected: [unsupported/stale/out of scope/conflicts with repo]

### Synthesis
**Agreements** — Multiple lanes support: [claim]
**Conflicts** — [Lane A] vs [Lane B]: [lead resolution]
**Residual risk** — [missing source, stale evidence, skipped provider, or none]
```

## Related

- `/harness-engineering` — Harness engineering and context lifecycle
- `/code-review` — Multi-agent review implementation
