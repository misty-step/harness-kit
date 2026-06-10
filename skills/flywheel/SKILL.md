---
name: flywheel
description: |
  Outer-loop shipping orchestrator. Composes /shape, /implement, /yeet,
  /deliver --polish-only, /ship, and /monitor per backlog item. Closure
  (archive, reflect, harness routing) lives in /ship; flywheel does not invoke
  /reflect directly.
  Use when: "flywheel", "run the outer loop", "next N items",
  "overnight queue", "cycle".
  Trigger: /flywheel.
argument-hint: "[--max-cycles N]"
---

# /flywheel

Compose cycles of: pick a backlog item → `/shape` (if unshaped) →
`/implement` → `/yeet` → `/deliver --polish-only` → `/ship` → `/monitor` →
loop.

Abbreviated form using the convenience composer:
pick → `/deliver` → `/yeet` → `/ship` → `/monitor` → loop.
(`/deliver` = `/shape` → `/implement` → clean loop; `/deliver --polish-only`
runs that same clean loop on an existing branch.)

> Migration: the polish step was `/settle` until backlog 080 collapsed it into
> `/deliver --polish-only`; `/settle` is a deprecated redirect for one release.

You already know how to do each of these. This skill exists only to
encode the invariants that aren't inferable from the leaf names.

## Invariants

- Flywheel composes. Phase logic lives in the leaf skill. Flywheel has none.
- State lives in leaf receipts, git, and `backlog.d/`. Flywheel has none.
- If `.harness-kit/flywheel.yaml` exists, load it once at cycle start with the
  Harness Kit config loader and use it only for cycle tuning: cadence,
  `max_cycles`, token budget, backlog include filters, and stop conditions.
  If absent, use invocation flags and built-in defaults.
- Provider-roster behavior lives in the leaf workflow skills. If the repo has
  `.harness-kit/agents.yaml`, `/flywheel` expects receipts for whatever
  lanes the leaf skills judged necessary, but it does not dispatch
  providers directly.
- The cycle closeout includes the roster delegation report from `/ship`,
  plus any provider-health follow-ups surfaced by failed or low-signal lanes.
- `/ship` owns closure: squash-merge, backlog archive, `/reflect`, and
  applying reflect's outputs. Flywheel does not invoke `/reflect` directly.
- Shared Closeout applies at cycle boundaries. A cycle is not complete while
  `git status --short --branch --untracked-files=all` shows paths or the
  shipped branch is unpushed/diverged from its remote. Verify remote sync with
  `git rev-list --left-right --count <branch>...<upstream>` or the branch's
  equivalent upstream check. Nonzero entries are action items for `/yeet`,
  `/ship`, a move-out, durable ignore, or an explicit blocker. See
  `harnesses/shared/AGENTS.md` (Closeout).
- Ship before deploy. Always.
- Harness edits from reflect never touch master. `/ship` routes them to
  `harness/reflect-outputs` for human review.

## Delegation Judgment

delegate on judgment per the shared Roster contract: native subagents
by default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Leaf skills own dispatch; /flywheel verifies phase receipts and uses lanes only for cycle strategy, risk critique, and closure-state review.

## Gotchas

- `/deliver`'s receipt is the contract — don't peer inside.
- An item can be open in `backlog.d/` but already shipped in git. Fix
  the stale entry before starting a cycle on it.
- Library repos still ship + reflect when deploy/monitor no-op.
- Two `/flywheel` runs in the same worktree collide on git state. Use
  separate worktrees for parallelism.
- Do not start the next cycle with leftover untracked evidence, backlog notes,
  generated docs, or unpushed commits from the last one. Resolve via shared
  Closeout before starting the next cycle.

## Non-Goals

- No cycle state machine, event enum, lock, or pick scoring.
- No direct `/reflect` invocation — that's `/ship`'s job.
- No USD tracking — the orchestrator runs under subscription. USD is a
  concern of systems that pay per token (e.g. ThinkTank itself).

## Verification

Semantic waiver: `/flywheel` composes other phase receipts rather than owning a
standalone deterministic transform. Verify a cycle by the `/deliver`, `/ship`,
`/deploy`, `/monitor`, and `/reflect` receipts it links, plus clean-tree
closeout and remote-sync evidence.
