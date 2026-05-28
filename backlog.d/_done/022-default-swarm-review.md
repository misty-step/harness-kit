# Make agent swarm review the mandatory default

Priority: high
Status: done
Estimate: M

## Goal

Wire `/code-review` into `/deliver` and `/settle` so that agent swarm review
is mechanically enforced, not optional. No branch can land without a verdict.

## Changes

- `/deliver` must run `/code-review` after build, before declaring "ready"
- `/settle` (git-native mode from 021) must require a verdict ref
- Pre-merge git hook validates verdict ref exists
- `/settle` must trigger `/code-review` if no verdict exists for the branch

## Multi-Provider Default

The recent multi-provider review (thinktank + codex + gemini) should be the
default path, not single-model review. This gives diverse perspectives and
catches model-specific blind spots.

## Oracle

- [x] `/deliver` pipeline includes code review step that produces verdict
- [x] `/land` refuses without verdict ref
- [x] Pre-merge hook blocks `git merge` without verdict
- [x] Skipping review requires explicit `HARNESS_KIT_NO_REVIEW=1` env var (escape hatch)

## Non-Goals

- Requiring human review (agent review is sufficient for merge)
- Blocking on individual reviewer disagreement (synthesis decides)

## What Was Built

- `scripts/land.sh` — thin `/land` entrypoint. Validates verdict
  via `verdict_check_landable`, keeps Dagger CI active even under review bypass,
  resolves the target branch dynamically, and merges `--no-ff`.
  Exit codes: 0 (success), 2 (missing/stale verdict), 3 (dont-ship), 4 (dagger fail).
- `.githooks/pre-merge-commit` — git hook blocking merge commits
  without valid verdict. Fail-open on branch-name ambiguity (learned from def0cb9).
- `verdict_check_landable` function in `scripts/lib/verdicts.sh` — returns 0/1/2
  for landable/missing-stale-or-invalid/dont-ship. Centralizes logic used by both callers.
- Escape hatch: `HARNESS_KIT_NO_REVIEW=1` env var, respected by both hook and script.
- 30 tests total: 17 verdict, 7 land, 6 hook — all green.
- Skill prose updates: `/deliver` gotcha bullet, `/settle` /land pointer,
  `/code-review` escape-hatch documentation.

## Design Decisions

- Env var (`HARNESS_KIT_NO_REVIEW=1`) over CLI flag — single harness-agnostic
  mechanism that works for both hook and script.
- `pre-merge-commit` hook only (not `pre-push`) — matches ticket scope
  ("blocks git merge"). FF-merge gap accepted; `/land` is the canonical path.
- Fail-open on branch-name ambiguity in hook — blocking on unclear state is
  worse than letting one merge through unguarded.
