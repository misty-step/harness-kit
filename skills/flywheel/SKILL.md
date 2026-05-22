---
name: flywheel
description: |
  Outer-loop shipping orchestrator. Composes /shape, /implement, /yeet,
  /settle, /ship, and /monitor per backlog item. Closure (archive,
  reflect, harness routing) lives in /ship; flywheel does not invoke
  /reflect directly.
  Use when: "flywheel", "run the outer loop", "next N items",
  "overnight queue", "cycle".
  Trigger: /flywheel.
argument-hint: "[--max-cycles N]"
---

# /flywheel

Compose cycles of: pick a backlog item → `/shape` (if unshaped) →
`/implement` → `/yeet` → `/settle` → `/ship` → `/monitor` → loop.

Abbreviated form using the convenience composer:
pick → `/deliver` → `/yeet` → `/ship` → `/monitor` → loop.
(`/deliver` = `/shape` → `/implement` → `/settle`.)

You already know how to do each of these. This skill exists only to
encode the invariants that aren't inferable from the leaf names.

## Invariants

- Flywheel composes. Phase logic lives in the leaf skill. Flywheel has none.
- State lives in leaf receipts, git, and `backlog.d/`. Flywheel has none.
- Provider-roster behavior lives in the leaf workflow skills. If the repo has
  `.spellbook/agents.yaml`, `/flywheel` expects roster receipts to appear in
  phase evidence, but it does not dispatch providers directly.
- `/ship` owns closure: squash-merge, backlog archive, `/reflect`, and
  applying reflect's outputs. Flywheel does not invoke `/reflect` directly.
- Ship before deploy. Always.
- Harness edits from reflect never touch master. `/ship` routes them to
  `harness/reflect-outputs` for human review.

## Gotchas

- `/deliver`'s receipt is the contract — don't peer inside.
- An item can be open in `backlog.d/` but already shipped in git. Fix
  the stale entry before starting a cycle on it.
- Library repos still ship + reflect when deploy/monitor no-op.
- Two `/flywheel` runs in the same worktree collide on git state. Use
  separate worktrees for parallelism.

## Non-Goals

- No cycle state machine, event enum, lock, or pick scoring.
- No direct `/reflect` invocation — that's `/ship`'s job.
- No USD tracking — the orchestrator runs under subscription. USD is a
  concern of systems that pay per token (e.g. ThinkTank itself).
