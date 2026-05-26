---
name: settle
description: |
  Polish loop. Take a feature branch with code and tests and iterate on it
  via /ci, /code-review, and /refactor until the branch is lean, green, and
  ship-ready. Stops at merge-ready. Does not merge. Does not archive tickets.
  Hands off to /ship for the final mile.
  Use when: "polish this", "get this merge-ready", "unblock", "clean up",
  "address reviews", "fix CI", "make this shippable".
  Trigger: /settle, /pr-fix, /pr-polish.
argument-hint: "[PR-number|branch-name]"
---

# /settle

Take a feature branch from "works" to ship-ready. Iterate `/ci` →
`/code-review` → `/refactor` until all four exit gates pass in one pass.
Then report ship-ready and hand the operator off to `/ship`.

## Stance

1. **Polish until green, stop at merge-ready.** `/settle` does not merge,
   does not archive tickets, does not reflect. When the loop exits clean,
   the operator runs `/ship`.
2. **Compose, do not replace.** `/ci`, `/code-review`, and `/refactor` own
   their domains. `/settle` sequences them and decides when to loop.
3. **Executive orchestrator.** Keep review-comment disposition, risk
   tradeoffs, and ship-readiness judgment on the lead model. Delegate
   bounded fixes and evidence gathering to focused subagents.
   When a provider roster is available (repo `.spellbook/agents.yaml` or system `~/.spellbook/agents.yaml`), require `/code-review`,
   `/refactor`, or `/diagnose` to produce two or more roster-member receipts
   or an explicit exception before `/settle` calls the branch ship-ready.
4. **Bounded iteration.** The loop has a safety cap (max 6 iterations).
   If polish isn't converging by then, escalate rather than thrash.
5. **Fresh-eyes self-review is load-bearing.** The final gate is reading
   the full diff one last time and asking "would I approve this?" —
   same-model bias is real; counter it with explicit hindsight.

## Delegation Floor

When a provider roster is available (repo `.spellbook/agents.yaml` or system `~/.spellbook/agents.yaml`), `/settle` verifies that each
substantive polish loop used two or more roster members or recorded a valid
exception. It sequences leaf skills; leaf skills own dispatch. Direct
lead-only settle work is limited to mechanical command execution, emergency
unblocks, explicit user-forbidden delegation, or an explicit waiver when
fewer than two roster members are available.

## Prerequisites

Assert at start; refuse with a clear reason on any miss.

- On a feature branch (not `master` / `main` / default protected branch).
- Branch has commits beyond the base branch (`git log base..HEAD` non-empty).
- Working tree clean, or dirty in a way the operator acknowledges —
  `/settle` will not stage random debris.
- No rebase / merge / cherry-pick in progress.

## The Polish Loop

Run the six steps in order. If any step produces changes, return to step 2
(`/ci`). The loop exits only when all four exit gates pass in the same
iteration without requiring further changes.

### 1. Assert preconditions

Check the prerequisites above. Detect mode:

- **PR mode** — `$ARGUMENTS` is a PR number, or `gh pr view` succeeds for
  the current branch. Use `skills/settle/scripts/fetch-pr-reviews.sh` for
  review bodies; check remote checks via `gh pr checks`.
- **Local mode** — no PR. Rely on local `/ci` and `/code-review` output.

Mode only changes *where* findings come from, not *what* the loop does.

### 2. Ci

Invoke `/ci`. If green, proceed to step 3. If red, classify:

- **Self-healable** (lint drift, stale lockfile, trivial import/typo):
  `/ci` handles it directly per its own fix-vs-escalate contract.
- **Logic/algorithm failure:** dispatch a **builder** subagent with the
  specific failure (file:line, gate, excerpt) and a bounded fix scope.
  Commit, then return to step 2.

Do not declare green while checks are pending. In PR mode, also check
`gh pr checks` before proceeding.

### 3. Code-review

Invoke `/code-review` with a must-ship lens. Synthesize the verdict:

- **ship / conditional** → proceed to step 4.
- **dont-ship** or blockers present → fix loop. For each blocking finding:
  dispatch a **builder** subagent with the specific file:line and fix
  scope. Fix → commit. Then return to step 2.

**In PR mode**, also read every PR comment in full via
`skills/settle/scripts/fetch-pr-reviews.sh <PR>`. Do not preview with
truncated `gh api` output. Triage per `references/pr-fix.md`: fix (in
scope), defer (out of scope → `backlog.d/`), or reject (with specific
reasoning, after steelman). Address one at a time, not in batches.

**Reviewer dispositions go on the lead model**, not a subagent. Deciding
whether a reviewer's concern is valid is judgment work.

### 4. Refactor

Invoke `/refactor`. It runs in feature-branch mode against the detected
base.

- **Mandatory** when net branch diff is >200 LOC.
- **Recommended** otherwise — cheap to run, often finds something.

If `/refactor` applies changes, commit and return to step 2.

Exit criterion: one `/refactor` pass in this iteration produced no
applied changes (or the changes it applied were already reverified by a
subsequent loop).

### 5. Self-review hindsight

Read the full branch diff one last time with fresh eyes:

```sh
git diff $(git merge-base HEAD master)...HEAD
```

Ask: **"Would I approve this if I were the reviewer?"** Look for:

- Shallow modules, pass-through layers, hidden coupling.
- Tests that assert implementation instead of behavior.
- Stale comments or docs in changed areas.
- Debug artifacts, commented-out code, TODO placeholders the loop missed.

If anything non-trivial surfaces, fix it and return to step 2. The
self-review gate is not optional — it is the last defense against
same-model blind spots.

### 6. Verdict-ref check (if supported)

If the repo uses verdict refs (`scripts/lib/verdicts.sh` exists), confirm
the current verdict at `refs/verdicts/<branch>` reads `ship` or
`conditional` and its SHA matches HEAD:

```sh
source scripts/lib/verdicts.sh
verdict_validate "$(git rev-parse --abbrev-ref HEAD)"
```

A stale verdict (SHA mismatch) means changes landed after the last
review; return to step 3 and re-run `/code-review`. A `dont-ship`
verdict means exit criteria are not met — return to step 3.

Repos without `scripts/lib/verdicts.sh` skip this step.

## Exit Criteria

The loop exits and `/settle` reports **ship-ready** only when all four
gates pass in the *same* iteration:

- [ ] `/ci` green (all gates, no pending checks)
- [ ] `/code-review` verdict `ship` or `conditional` (no open blockers)
- [ ] `/refactor` ran and applied no further changes this iteration
- [ ] Self-review hindsight pass produced no follow-ups
- [ ] (If applicable) verdict ref is fresh and not `dont-ship`

Safety cap: **max 6 iterations**. If the loop has not converged by the
sixth pass, stop and emit a structured diagnosis — the branch likely has
a deeper issue that needs human judgment, not more loop churn.

On clean exit, emit:

```
/settle complete — ship-ready.

Iterations: 3
CI:          green (dagger call check, 4m12s)
Code-review: ship (3 reviewers, 0 blockers)
Refactor:    no further simplification found (diff: 187 LOC net)
Self-review: clean
Verdict ref: refs/verdicts/<branch> → ship (SHA matches HEAD)

Next: run /ship to merge, archive, and reflect.
```

## What /settle Does NOT Do

These are hard non-goals. Surface them to the operator if asked.

- **Does not merge.** Squash-merge is `/ship`'s job. `/settle` stops at
  merge-ready.
- **Does not archive backlog tickets.** `Closes-backlog` trailers and
  `backlog.d/_done/` moves happen in `/ship`.
- **Does not run `/reflect`.** Retro, backlog mutations, and
  harness-tuning outputs are `/ship`'s responsibility.
- **Does not push.** Use `/yeet` to ship commits to the remote, or let
  `/ship` handle the final push as part of the merge.
- **Does not scaffold CI.** If CI is absent or weak, `/ci` handles it
  per its own contract; `/settle` does not duplicate that logic.

## PR Mode vs Local Mode

| Concern | PR mode | Local mode |
|---|---|---|
| Detection | `$ARGUMENTS` is a PR number, or `gh pr view` succeeds | no PR for branch |
| CI signal | `/ci` + `gh pr checks` | `/ci` only |
| Review input | `fetch-pr-reviews.sh <PR>` + `/code-review` | `/code-review` only |
| Verdict proof | approving review on PR + verdict ref (if supported) | verdict ref only |

Both modes run the same six-step loop. PR mode adds remote-checks and
reviewer-comment sources to the existing gates.

## Refuse Conditions

Stop and surface to the operator instead of looping:

- On `master` / `main` / default protected branch directly.
- Branch has no commits beyond base (nothing to polish).
- Rebase / merge / cherry-pick in progress (`.git/MERGE_HEAD`,
  `.git/CHERRY_PICK_HEAD`, or `rebase-*` dir).
- Working tree has unresolved conflict markers.
- Safety cap hit (6 iterations without convergence) — emit diagnosis,
  do not loop further.
- Escape-hatch environment variable `SPELLBOOK_NO_REVIEW=1` does **not**
  apply here — it's for `/ship`'s pre-merge check, not `/settle`'s loop.

## Interactions

- **Invoked by:** `/flywheel` as the polish stage of each cycle;
  `/deliver` as the final step before handing back to the outer loop.
- **Invokes:** `/ci`, `/code-review`, `/refactor`. Dispatches
  **builder** subagents for bounded fixes.
- **Hands off to:** `/ship` for merge + archive + reflect. `/settle`'s
  exit report always names `/ship` as the next step.
- **Complements `/yeet`:** `/yeet` handles commit/push discipline on the
  feature branch; `/settle` handles polish; `/ship` handles the merge.
  All three are imperative finals at different layers.

## Gotchas

- **Truncated review comments.** `gh api` + jq previews hide the body of
  long reviewer comments, especially automated reviewers with code
  suggestions. Always run `fetch-pr-reviews.sh` for full text.
- **Reflexive dismissal.** "By design" and "established pattern" are not
  valid rejection reasons on their own. Steelman first; cite specifics.
  See `references/pr-fix.md` disposition criteria.
- **Declaring ship-ready while checks are pending.** A pending check is
  not a passing check. Wait for terminal state before exiting the loop.
- **Polishing without re-running CI.** Every loop step that commits must
  return to `/ci`. Never exit with refactor changes unverified by CI.
- **Skipping the hindsight pass.** "Tests pass, review is clean" is not
  the exit gate — the exit gate is *also* that fresh eyes see nothing
  worth fixing. The hindsight pass catches what the sub-gates miss.
- **Infinite polish.** The safety cap exists because some branches
  genuinely need human judgment, not more loop iterations. At 6 passes,
  stop and diagnose.
- **Treating `/settle` as the merge step.** It isn't. If the operator
  says "ship this" after `/settle` reports ship-ready, the answer is
  `/ship`, not another `/settle` pass.
