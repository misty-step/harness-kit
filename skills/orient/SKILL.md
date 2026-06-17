---
name: orient
description: |
  Fast session-start repository orientation from live local evidence. Use when:
  "orient yourself", "start of session", "new session", "where are we",
  "catch me up before acting", after compaction,
  after switching worktrees, or before choosing a Harness Kit workflow. Trigger:
  /orient, /ground, /session-start.
argument-hint: "[scope|--deep]"
---

# /orient

Ground the session in repo truth — not memory, not a stale summary. Read-only,
fast.

## What it produces

A short, natural read of where things stand and the one move that follows. You
are writing to stop a wrong first step, not to fill a form. In plain prose,
cover only what the evidence supports:

- where we are (repo, branch, clean vs dirty) and what's in flight;
- the single best next move, why it wins, and what would make it wrong.

Punchy beats complete. A sentence or two when the state is obvious; a short list
only when it's genuinely tangled. No fixed template, no field-by-field rundown,
no history dump. If the user already gave a precise command and the state is
clear, that sentence is the whole report. `--deep` is the only license to dig.

Example, obvious state: *"On `master`, clean, CI green. Shaped ticket 058 is the
only thing in flight and nothing blocks it — next is `/deliver 058`."*

## Where the signal lives

Read the smallest set that explains the workspace; skip whatever the prompt
already settled. Beyond the obvious (`git status`, branch, recent commits):

- scoped then repo `AGENTS.md`; `project.md` for focus (`README.md` only if still unclear)
- `backlog.d/*.md` for active work; `backlog.d/_done/*` and `Closes-backlog:` trailers for closure
- `.harness-kit/agents.yaml` / roster probe — only when the next move is a delegation
- `.harness-kit/agent-readiness.yaml` — only to name the profile state, never to score the repo

## Routing

Recommend one next skill. If the signal is mixed, name the missing evidence
rather than guess from vibes.

| Signal | Next |
|---|---|
| Shaped ticket, clean branch | `/deliver <ticket>` |
| Dirty branch, intended changes | `/deliver --polish-only` |
| Empty or stale backlog | `/groom` |
| Failure, broken gate, unclear root cause | `/diagnose` |
| Finished work needing closeout | `/ship`, `/yeet`, or `/reflect` |
| Readiness or profile question | `/qa` or the repo's readiness surface |
| "What happened / why does it matter" | `/orient --deep` or `/shape` |
| Skill or harness primitive change | `/harness-engineering` |

## Stay in lane

Orient reads; it never acts. Don't deliver, groom, refactor, reflect, debrief,
mine transcripts, or mutate state. Don't label the repo "ready" or "validated" —
route that to the owning skill. Don't store session memory; durable state lives
in backlog, commits, and receipts. Don't spin up provider lanes unless scope is
broad, stale, or contested. Scoped `AGENTS.md` governs — respect it.

## Verification

Editing this skill in the Harness Kit source repo:

```sh
cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .
```

Invoked in another repo, that command isn't required — acceptance is the
report's cited live evidence and a next move the operator can act on.
