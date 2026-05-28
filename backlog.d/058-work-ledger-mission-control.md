# Work ledger mission-control view

Priority: P2
Status: pending
Estimate: M

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
