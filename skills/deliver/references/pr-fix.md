# PR Fix — Unblock the PR

Polish-only PR-mode comment-triage reference (absorbed from the former
`/settle`). Get a blocked PR to green; then the clean loop polishes it.

Get from blocked to green. Conflicts resolved, CI passing, every review comment addressed.

## Self-Review Protocol

Before touching reviewer comments:

1. Read the entire diff as if seeing it for the first time
2. Each file change must serve the PR's stated goal — remove what doesn't
3. Strip debug artifacts, commented-out code, TODO placeholders
4. Verify commit messages accurately describe what changed

Self-review catches problems before reviewers do. Fix anything found here
before responding to comments.

## Conflict Resolution

```text
fetch origin
  │
  ├─ Few commits, linear history matters, no downstream consumers → rebase
  │    git rebase origin/main
  │
  └─ Many commits, shared branch, complex conflicts → merge
       git merge origin/main
```

**Always:**
- `git fetch origin` first
- Never rebase published shared branches
- Understand both sides before choosing a resolution
- Read the PR description and commit messages for intent
- Prefer the version that matches the spec

**Conflict resolution heuristic:** When both sides changed the same code,
the version aligned with the PR's stated goal wins. When unclear, preserve
the more defensive / more tested version.

## CI Failure Diagnosis

Read the actual logs. Don't guess from the check name.

### Root Cause Categories

| Category | Signal | Action |
|----------|--------|--------|
| Real regression | Test fails deterministically, passes on base branch | Find breaking commit (`git bisect` if needed), fix root cause |
| Flaky test | Passes on retry, no code change | Re-run once. If green, file a `backlog.d/` item for the flake. Don't ignore. |
| Environment | Runner version mismatch, cache miss, Docker pull failure | Check runner versions, dependency caches, base images |
| Dependency | Upstream package broke, yanked version, incompatible update | Pin or update dependency, verify lockfile |

### Diagnosis Sequence

**GitHub mode:**
1. `gh run view RUN-ID --log-failed` — read the actual failure
2. Identify which test/step failed and the error message
3. Check: does this test pass on the base branch? (`git stash && git checkout main && run test`)
4. If flaky: re-run once via `gh run rerun RUN-ID --failed`
5. If real: find the breaking commit, fix the root cause, push

**Git-native mode:**
1. `dagger call check` — read the actual failure output
2. Identify which gate failed and the error message
3. Check: does this pass on the base branch? (`git stash && git checkout main && dagger call check`)
4. If flaky: re-run `dagger call check` once
5. If real: find the breaking commit, fix the root cause, commit

### Never

- Disable tests to make CI green
- Add retry loops around flaky assertions
- Lower coverage thresholds or lint strictness
- Mark failing checks as "expected failure"

These are not fixes. They are debt with compound interest.

## Review Comment Triage

### Reading Protocol

**Run `skills/deliver/scripts/fetch-pr-reviews.sh <PR>` first.** This script
deterministically fetches ALL review bodies, inline comments, and PR
conversation in full. Read the complete output before classifying anything.

Do NOT use ad-hoc `gh api` calls with jq truncation, python slicing, or
`head`/`tail` to preview comments. The script exists to prevent this.

Automated reviewers (Gemini, Codex, CodeRabbit, etc.) are treated with
the same rigor as human reviewers — their comments often contain specific
code suggestions that are invisible in truncated views.

For each comment:
1. Read the full text including any code suggestions, diffs, or expandable sections
2. If the comment references a file/line, read the actual code at that location
3. If the comment includes a `suggestion` block, evaluate the suggested code directly

### Disposition Criteria

Before classifying a comment, answer these questions:

1. **Does the comment identify a real problem?** Not "could this be better" but
   "does this violate a contract, duplicate information, introduce a bug, or
   create a maintenance hazard?" If yes → fix it.

2. **Does the comment include a concrete code suggestion?** Evaluate the suggestion
   on its merits. If the suggested code is correct and improves the PR, accept it.
   Don't reject working code suggestions because you'd "rather do it differently."

3. **Steelman test:** Before rejecting, articulate the strongest version of the
   reviewer's argument. If you can't explain *why* they think this matters,
   you haven't understood the comment yet.

4. **Pattern check:** Does the comment point out an inconsistency with existing
   patterns in the codebase? If yes, the default is to fix the inconsistency,
   not to justify it.

**Rejection requires specificity.** "By design" and "established pattern" are
not valid rejection reasons on their own. You must cite the specific design
decision or pattern, explain why it applies here, and explain why the
reviewer's concern doesn't override it.

### Classification

After applying the disposition criteria, classify and act:

#### In scope, valid

Fix it. Commit with a message referencing the feedback. Reply **inline on
the comment thread** confirming the fix with the commit SHA.

```
Fixed in abc1234 — switched to the connection pool pattern you suggested.
```

#### Valid but out of scope

Create a `backlog.d/` item. Reply linking the issue. Explain why it's
deferred — scope boundary, not dismissal.

```
Good catch. Filed as backlog.d/042-connection-pool-config.md — out of scope
for this PR but should ship before the next release.
```

#### Invalid (after steelman test passes)

Reply with clear, specific reasoning. Reference code, tests, or docs that
support your position. Be respectful but firm.

```
The current approach handles this via X (see src/pool.ts:42). The suggested
change would break the invariant documented in ARCHITECTURE.md § Connection
Lifecycle. Happy to discuss further if I'm missing context.
```

#### Questions (not requests)

Answer clearly. If the answer reveals a documentation gap, fix the gap
in the same PR.

### Ordering

Address comments **one at a time**, not in batches. Fix → commit → reply
→ next comment. Batch replies encourage bulk dismissal and skip evaluation.

## Wait for async automation

After pushing fixes, do not declare the fix pass complete. Automation may
still be running:

**GitHub mode:**
1. Wait for all CI checks to reach terminal state (pass/fail, not pending)
2. Wait for bot reviewers (linters, security scanners, coverage bots) to post
3. Re-check: `gh pr view --json statusCheckRollup,reviews`
4. If new findings appeared, triage them — loop back to the relevant section

**Git-native mode:**
1. Re-run `dagger call check` after all fixes
2. No async bots to wait for — local CI is synchronous

### Merge-Readiness Gate

Before declaring the branch unblocked:

**GitHub mode:**
```bash
gh pr view --json reviews,statusCheckRollup
```
Verify: at least one approving review, all checks passing, no unresolved threads.

**Git-native mode:**
```bash
source scripts/lib/verdicts.sh && verdict_validate <branch>
dagger call check
```
Verify: verdict ref exists with SHA matching HEAD, Dagger CI green.
If no verdict exists, run `/code-review` first to generate one.

If any condition fails, address the gap or escalate before polishing.

## Exit Criteria

The PR-fix pass is complete when ALL of:

- [ ] No merge conflicts
- [ ] CI green (all checks passing, not pending)
- [ ] Every review comment addressed (fixed, deferred with issue, or rejected with reasoning)
- [ ] No open/unresolved review threads
- [ ] Merge-readiness gate passes

If already green on entry, skip straight to the polish pass (`pr-polish.md`).
