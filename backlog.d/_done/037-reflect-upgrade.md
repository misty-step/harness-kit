# `/reflect` upgrade — session + bucket + harness critique

Priority: high
Status: pending
Estimate: M (~2 dev-days)

## Goal

Upgrade `/reflect` to be the learning engine of the outer loop. Today it's
a session retrospective. Needed: a skill that reads a cycle's events,
artifacts, and diffs, and produces:

1. **Session critique** — what went well, what rabbit-holed, what was wasted
2. **Bucket mutations** — consolidate / delete / create backlog items based on learnings
3. **Harness suggestions** — skill / agent / hook improvements, emitted to a branch only

## Why Upgrade vs Build New

`/reflect` already exists and is the right name. What exists today is a
subset; the outer-loop caller needs more structured output (typed findings,
backlog mutation authority, harness suggestions). Grow the current skill
rather than fork it.

## Contract

**Input:**
- Cycle directory: `backlog.d/_cycles/<ulid>/` (events, evidence, manifest)
- Backlog path: `backlog.d/`
- Harness pointers: CLAUDE.md, AGENTS.md, active skills/agents used

**Output:**
- `reflect.done` event with structured findings
- Mutations applied to `backlog.d/` (new items, edits, consolidations, deletions)
- `harness.suggested` patch emitted to `harness/auto-tune` branch only —
  never main, never current feature branch

**Stops at:** event written + mutations committed + harness branch pushed (if any).

## Stance

1. **Evidence-driven.** Read the cycle's event log, diffs, review transcripts, QA artifacts. Don't hallucinate learnings.
2. **Backlog mutation authority.** `/reflect` is the only skill allowed to write bucket-wide changes (consolidate N tickets into one, delete stale items). Scope: `backlog.d/` only.
3. **Harness changes go to a branch, never main.** Prevents runaway meta-loop from rewriting its own skills mid-cycle.
4. **Structured output.** Each finding has: kind (`wasted_effort` | `rabbit_hole` | `missing_skill` | `harness_gap` | `spec_drift`), severity, evidence refs, recommended action.

## Composition

```
/reflect <cycle-ulid>
    │
    ▼
  1. Load cycle artifacts
     ├── cycle.jsonl events
     ├── evidence/* (QA, review transcripts, diffs)
     └── manifest.json (timings, costs, outcomes)
    │
    ▼
  2. Critique phases
     ├── Which phases took longer than expected?
     ├── Which phases retried / looped / backtracked?
     ├── Was the shaping accurate? (spec vs what shipped)
     └── Were the right skills/agents used?
    │
    ▼
  3. Emit findings (structured)
    │
    ▼
  4. Apply bucket mutations (git commit to current branch or backlog branch)
     ├── Consolidate duplicates
     ├── Delete stale items
     ├── Create new items from discovered work
     └── Update item priorities based on learnings
    │
    ▼
  5. Emit harness suggestions to harness/auto-tune branch (if any)
    │
    ▼
  Write reflect.done event, exit
```

## Finding Schema

```json
{
  "kind": "wasted_effort",
  "severity": "medium",
  "evidence_refs": ["cycle.jsonl:42-48", "evidence/review_iter_2.md"],
  "summary": "Built feature X; spec changed mid-cycle and half the work was scrapped",
  "recommended_action": "Shape phase needs a contract-freeze step before build",
  "mutation": {
    "kind": "new_backlog_item",
    "title": "Add contract-freeze step to /shape"
  }
}
```

## What `/reflect` Does NOT Do

- Merge harness suggestions to main — humans review the branch
- Edit skills / agents outside the `harness/auto-tune` branch
- Delete backlog items without evidence
- Run more cycles — that's the caller's concern
- Self-critique (reflecting on reflect's own mistakes) — out of scope

## Oracle

- [ ] `skills/reflect/SKILL.md` updated to new contract
- [ ] Given a cycle directory with ≥6 events, produces ≥3 structured findings
- [ ] Backlog mutations land as git commits on the appropriate branch
- [ ] Harness suggestions land on `harness/auto-tune`, never main
- [ ] `reflect.done` event carries findings array
- [ ] Dogfoods: reflect on this harness-kit repo's autopilot Phase 1 cycle, produces useful findings

## Non-Goals

- Automatic harness adoption — humans review branch
- Cross-cycle meta-reflection — single cycle per invocation
- Multi-reflector ensemble (MAR-style) — single pass
- Self-healing skills — evidence-driven suggestions only, no auto-edit

## Related

- Blocks: 028 (`/autopilot` outer loop needs upgraded `/reflect`)
- Consumes: cycle event log (already shipped via `events.sh`)
- Produces signal for: 031 (harness auto-tune — consumes structured findings)
- Related: 029 (tailor — may surface as `missing_skill` findings)
