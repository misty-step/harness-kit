# Phase 1: Fix — unblock the branch (spellbook)

Get from blocked to green. Conflicts resolved, `dagger call check` passing,
every `/code-review` finding addressed, verdict ref valid.

Spellbook has no GitHub PR review gate. This reference is git-native.

## Self-review protocol

Before touching review findings:

1. Read the entire diff as if seeing it for the first time.
2. Each file change must serve the shape's stated goal — remove what doesn't.
3. Strip debug, commented-out code, stale TODO placeholders.
4. Verify commit messages describe the "why" accurately.

Self-review catches problems before the verdict does. Fix anything found
here before dispatching fixes to subagents.

## Conflict resolution

```text
git fetch origin
  |
  +- Few commits, linear history matters, no downstream branches -> rebase
  |    git rebase origin/master
  |
  +- Many commits, shared branch, complex conflicts            -> merge
       git merge origin/master
```

**Always:**
- `git fetch origin` first.
- Never rebase a published shared branch.
- Read the backlog shape (`backlog.d/NNN-*.md`) for intent before choosing
  a side.
- Prefer the version aligned with the shape's goal. When unclear, keep
  the more defensive or better-tested version.

## Gate failure diagnosis

The gate is `dagger call check --source=.`. 14 sub-gates, parallel. Read
the actual failure output — don't guess from the gate name.

### Root-cause categories

| Category | Signal | Action |
|---|---|---|
| Real regression | Fails deterministically on this branch, green on `master` | Find the breaking commit (`git bisect`), fix root cause |
| Flake | Passes on retry without code change | Re-run once. If green, file a git-bug issue or `backlog.d/` entry. Never ignore. |
| Environment | Dagger engine mismatch, cache miss, Docker pull failure | `dagger core container` version; clear cache; re-run |
| Dependency | Upstream Python/Node package changed | Pin or bump, verify lockfile, re-run |

### Diagnosis sequence

1. `dagger call check --source=.` — read the failure output in full.
2. Identify which sub-gate failed (`lint-yaml`, `lint-shell`,
   `check-frontmatter`, `check-portable-paths`, etc.) and the message.
3. Does this pass on `master`?
   ```bash
   git stash && git checkout master
   dagger call check --source=.
   git checkout - && git stash pop
   ```
4. If flaky: re-run once.
5. If real: find the breaking commit, fix root cause, commit.

### Heal mode

For a single failing lint-class gate (yaml/shell/python/frontmatter):

```bash
dagger call heal --source=. --model=gpt-4.1 --attempts=2
```

Bounded LLM repair. Not a substitute for understanding the failure —
review every change it produces before committing.

### Never

- Disable tests or sub-gates to turn the gate green.
- Lower a threshold or strictness level.
- Mark a failing gate "expected" or "flaky" without an issue tracking the
  root cause.

These aren't fixes. They're debt with compound interest and they violate
the "NEVER lower quality gates" Red Line.

## Review-finding triage

`/code-review` produces a verdict ref at `refs/verdicts/<branch>` and
synthesis output (typically under `.evidence/<branch>/review-synthesis.md`
or the current code-review scratch path — check the skill output).

### Reading protocol

Read every finding in full. Never skim on truncated previews. If a
finding references a file/line, open the actual code at that location.
If it includes a `suggestion` block, evaluate the suggested code on its
merits.

Automated reviewers are treated with the same rigor as humans.

### Disposition criteria

For each finding:

1. **Real problem?** Not "could be better" — "violates a contract,
   duplicates information, introduces a bug, creates a maintenance
   hazard." Yes -> fix it.
2. **Concrete suggestion?** Evaluate on merits. If correct and improving,
   take it. Don't reject working code because you'd "rather do it
   differently."
3. **Steelman test.** Articulate the strongest version of the reviewer's
   argument before rejecting. If you can't, you haven't understood it.
4. **Pattern check.** Does the finding point at inconsistency with
   existing spellbook patterns? Default to fixing the inconsistency, not
   justifying it. ("By design" without citing the specific design
   decision is not a valid rejection.)

### Classification

**In scope, valid:** fix it, commit, reference the finding in the message.

**Valid but out of scope:** create a git-bug issue or a
`backlog.d/NNN-<slug>.md` entry; note it in the final report. Don't
silently drop it.

**Invalid (after steelman):** document the reasoning. Cite code, tests,
or the harness brief. Be specific.

**Questions (not requests):** answer. If the answer reveals a doc gap,
fix the doc in the same branch.

### Ordering

Address findings one at a time. Fix -> commit -> next finding. Batched
disposition encourages bulk dismissal.

## Re-gate after fixes

Spellbook CI is synchronous. No async bots, no status rollup to poll.

```bash
dagger call check --source=.
```

If green and all findings are addressed, move to verdict validation.

## Merge-readiness gate

```bash
source scripts/lib/verdicts.sh
verdict_check_landable "$branch"
# rc=0 ship/conditional  -> landable
# rc=1 missing/stale     -> run /code-review, or the branch SHA moved after verdict
# rc=2 dont-ship         -> Phase 1 isn't done; address findings and re-review

(cd "$(git rev-parse --show-toplevel)" && dagger call check --source=.)
```

Verify: verdict exists, SHA matches HEAD, gate green. If any condition
fails, address the gap. Do not proceed to Phase 2.

## Exit criteria

- [ ] No merge conflicts against `master`.
- [ ] `dagger call check --source=.` green (14/14 sub-gates).
- [ ] Every `/code-review` finding fixed, deferred with an issue, or
      rejected with specific reasoning.
- [ ] `verdict_check_landable "$branch"` returns 0.
- [ ] Verdict SHA matches branch HEAD.

If already green and settled on entry, skip to Phase 2.
