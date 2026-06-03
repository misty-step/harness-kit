# Work ledger mission-control view

Priority: P2
Status: done
Estimate: M

## What Was Built

- Added `scripts/work-ledger.py` with append, summary, and self-test commands
  for local mission-control events.
- Added `.harness-kit/work/ledger.jsonl` as the gitignored runtime store and
  `.harness-kit/examples/work-ledger.jsonl` as the committed fixture.
- Added ledger fixture validation and store checks to
  `scripts/check-agent-roster.py`.
- Added a Dagger `test-work-ledger` lane.
- Updated `/deliver`, `/ship`, `/qa`, `/demo`, `/code-review`, `/monitor`, and
  `/reflect` to name exact ledger event transitions they emit or consume.

## Goal

Create a local-first work ledger that lets an operator see all active agentic
work in one place: backlog/spec, branch, owning skill, current phase, latest
evidence, open blockers, spawned agents, and next action.

## Non-Goals

- Do not build a web dashboard.
- Do not replace `backlog.d/`, git branches, or `.evidence/`.
- Do not require GitHub, Linear, or any SaaS tracker.
- Do not introduce a semantic workflow database.

## Oracle

- [ ] A simple ledger format exists under `.harness-kit/work/` or
      `.evidence/_work/` with one JSONL record per phase transition.
- [ ] `/deliver`, `/ship`, `/qa`, `/demo`, `/code-review`, `/monitor`, and
      `/reflect` each name the ledger event they emit or consume.
- [ ] A CLI helper prints the current mission-control summary: active branch,
      backlog ID, phase, evidence path, blockers, and next action.
- [ ] Ledger records link to agent-session trace refs from
      `backlog.d/056-agent-session-trace-lifecycle.md` when available.
- [ ] The format works offline and survives context compaction.
- [ ] `dagger call check --source=.` green.

## Notes

Sophisticated agent workflows are converging on a mission-control shape: many
agents may work in parallel, but humans need one compact surface to steer,
pause, resume, audit, and communicate status. Harness Kit already has most of the
pieces (`backlog.d/`, `.evidence/`, verdict refs, reflect outputs). The missing
piece is a tiny ledger that ties them together without turning the repo into a
platform.

The first version should be boring: append-only JSONL plus a print command.
The value is operational coherence, not UI.
