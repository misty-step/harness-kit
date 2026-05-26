# GEPA-style harness auto-tune from cycle events

Priority: low
Status: parked
Estimate: L

Parked during 2026-04-14 grooming. Do not start until `/flywheel` (028) has
produced ≥20 cycles of event data. Without signal, there's nothing to tune
on, and prompt-evolution infrastructure ahead of signal is the same failure
mode that ate `/focus`.

## Goal

Close the reflect → harness-improvement loop. Retros emit structured signals
(`reflect.signals.json`) naming a harness target (skill / agent / hook /
AGENTS.md section) plus textual feedback. A `harness-tuner` agent evolves
the named artifact and proposes the edit on a dedicated branch — never
auto-merged.

## Why GEPA, Not MIPROv2

DSPy's GEPA (Genetic-Pareto reflective optimizer, arxiv 2507.03620) accepts
**textual feedback** from evals, not just scalar scores. The retro phase
produces *reasons* ("review missed the auth regression because the bench
didn't include securitron"), and GEPA can evolve prompts using those
reasons. MIPROv2 only optimizes on scalar metrics. For a harness where
the feedback signal is prose, GEPA is the correct backend.

## Signal Schema (to be defined against real cycle event data)

```json
{
  "cycle_id": "01HQ...",
  "target": "skills/code-review/references/bench-map.yaml",
  "feedback": "review missed SQL injection on /users endpoint; bench did not include securitron; auth glob pattern was too narrow",
  "evidence_refs": [".evidence/cycle-01HQ.../review-synthesis.md"],
  "proposed_edit_hint": "add glob for /api/**/*.py → [securitron]"
}
```

## Flow (v2)

```
/reflect emits reflect.signals.json
    │
    ▼
harness-tuner agent reads signal + current artifact
    │
    ▼
proposes edit on branch harness/auto-tune
    │
    ▼
opens PR for human review (NEVER auto-merge)
    │
    ▼
next /flywheel cycle picks up the PR as a backlog item
```

## Entry Criteria (when to unpark)

- [ ] `/flywheel` has run ≥20 cycles on real work (not dogfooding alone)
- [ ] `backlog.d/_cycles/*.jsonl` files contain reflect.done events with
      actionable targets in ≥50% of cycles
- [ ] At least one concrete harness improvement has been identified
      manually from cycle event review (proving signal exists before automating)

## Oracle (for eventual v2)

- [ ] `reflect.signals.json` schema is defined and validated
- [ ] `harness-tuner` agent exists and is invoked by `/flywheel` step 9
- [ ] Proposed edits land on `harness/auto-tune` branch, never main
- [ ] ≥3 tuner-generated PRs over 30 days show measurable improvement
      (A/B via harness eval or human verdict)

## Non-Goals

- Auto-merging tuner proposals (humans decide)
- Tuning the tuner itself (meta-meta)
- Touching global AGENTS.md without human approval
- Starting this work before 028 produces event data

## Related

- Depends on: 028 (`/flywheel` produces the event signal this consumes)
- Related pattern: 023 (review score feedback loop — similar retro→improvement
  arc, different target surface)
