---
name: flywheel
description: |
  Outer-loop shipping orchestrator. Composes /deliver, landing, /deploy,
  /monitor, and /reflect cycle per backlog item, then applies reflect
  outputs to the harness and backlog before looping.
  Use when: "flywheel", "run the outer loop", "next N items",
  "overnight queue", "cycle".
  Trigger: /flywheel.
argument-hint: "[--max-cycles N]"
---

# /flywheel

Compose cycles of: pick a backlog item → `/deliver` → land → `/deploy` →
`/monitor` → `/reflect cycle` → apply reflect's outputs → loop.

You already know how to do each of these. This skill exists only to
encode the invariants that aren't inferable from the leaf names.

## Invariants

- Flywheel composes. Phase logic lives in the leaf skill.
- State lives in leaf receipts, git, and `backlog.d/`. Flywheel has none.
- Provider-roster behavior lives in the leaf workflow skills. If the repo has
  `.spellbook/agents.yaml`, `/flywheel` expects two or more roster-member
  receipts or explicit exceptions in phase evidence, but it does not dispatch
  providers directly.
- The cycle closeout includes a roster delegation report, plus any
  provider-health follow-ups surfaced by failed or low-signal lanes.
- Land before deploy. Always.
- Reflect's mutations (backlog + harness branch) land before the cycle
  closes. Harness edits never touch main.

## Delegation Floor

When `.spellbook/agents.yaml` exists, `/flywheel` verifies that each
substantive lifecycle phase used two or more roster members or recorded a
valid exception. It remains the outer-loop manager; leaf skills own dispatch.

## Gotchas

- `/deliver`'s receipt is the contract — don't peer inside.
- An item can be open in `backlog.d/` but already shipped in git. Fix
  the stale entry before starting a cycle on it.
- Library repos still land + reflect when deploy/monitor no-op.
- Two `/flywheel` runs in the same worktree collide on git state. Use
  separate worktrees for parallelism.

## Non-Goals

- No cycle state machine, event enum, lock, or pick scoring.
- No USD tracking — the orchestrator runs under subscription. USD is a
  concern of systems that pay per token (e.g. ThinkTank itself).
