---
name: settle
description: |
  Land a topic branch into `master` on spellbook. Git-native by default:
  Dagger gate, verdict ref, non-FF merge, backlog close-out. GitHub PR mode
  is optional (no `.github/` here — no PR review gate exists).
  /land alias: validate verdict + run `dagger call check` + `scripts/land.sh`.
  Use when: "land this", "settle this branch", "merge it", "get this
  mergeable", "close the backlog item", "ship it", "clean up and land".
  Trigger: /settle, /land (alias).
argument-hint: "[branch-name]"
---

# /settle — spellbook

Land a `feat/*`, `fix/*`, `chore/*`, `docs/*`, or `refactor/*` branch into
`master`. Spellbook has no deploy step and no PR review gate (`.github/` is
absent). Settle IS the ship — merged code propagates to every harness the
next time `./bootstrap.sh` runs (post-merge hook fires it automatically).

Plain `/settle` stops at merge-ready. `/land` continues through the merge.

## Spellbook reality

- **Base branch:** `master`. Topic: `feat/*`, `fix/*`, `chore/*`, `docs/*`,
  `refactor/*`.
- **The single shipping gate:** `dagger call check --source=.` (14 parallel
  sub-gates). Without this green, settle refuses. No exceptions.
- **Verdict gate:** non-FF merges into `master` trip
  `.githooks/pre-merge-commit`, which requires a ship/conditional verdict ref
  at `refs/verdicts/<branch>` whose SHA matches HEAD. Generate verdicts via
  `/code-review`. FF merges bypass this gate by construction — settle
  prefers non-FF so history records the merge.
- **Escape hatch:** `SPELLBOOK_NO_REVIEW=1` bypasses the verdict gate. Use
  only when the gate is broken, not when you dislike the verdict.
- **Land tool:** `scripts/land.sh <branch>` is the canonical merge driver.
  It validates the verdict, runs `dagger call check`, checks out the target
  branch, and executes `git merge --no-ff <branch>`.
- **Backlog close-out:** if the branch implements a `backlog.d/NNN-*.md`
  shape, `mv backlog.d/NNN-*.md backlog.d/_done/` and commit that move as
  part of settling. Recent close-outs live in `backlog.d/_done/`.
- **Post-merge auto-bootstrap:** `.githooks/post-commit`, `post-merge`, and
  `post-rewrite` re-run `./bootstrap.sh` when `skills/` or `agents/` changed.
  Global harness symlinks refresh without explicit invocation.
- **GitHub PRs are optional.** There's no `.github/`, no branch-protection
  rule, no CI workflow. `git-bug push origin` can mirror issues when
  desired; settle's git-native mode is the primary mode.
- **No deploy, no QA evidence step, no PR screenshots.** Merge is the ship.

## Role

Senior engineer who owns the lane end-to-end. Not done until the branch is
genuinely clean — architecturally sound, tested, simple — and merged.

## Execution stance

Executive orchestrator:
- Keep merge-readiness judgment, verdict disposition, and risk calls on the
  lead model.
- Delegate bounded fixes (one failing gate, one review finding, one narrow
  patch) to general-purpose subagents.
- Parallel fanout for independent fixes; serialize when fixes share files.
- Compose `/ci`, `/code-review`, `/refactor`; never reimplement their
  domain contracts inline.
- In repos with `.spellbook/agents.yaml`, let `/code-review`, `/refactor`,
  or `/diagnose` use roster lanes for non-trivial evidence or fixes; `/settle`
  consumes their receipts rather than dispatching providers directly.

## Mode detection

1. `$ARGUMENTS` is a branch name → git-native mode against that branch.
2. No argument → current branch, git-native mode.
3. `/land <branch>` or `/land` → always git-native; proceed through merge.
4. A PR exists (`gh pr view` succeeds) → GitHub mode is available but
   **non-primary here**. Use it only if the user explicitly asks to route
   through a PR; otherwise git-native is correct because spellbook has no
   PR review gate.

## Process

Three phases, looped until a full pass produces no changes.

### Phase 1: Fix — unblock

Read `references/pr-fix.md` for conflict resolution, CI diagnosis, and
review-finding triage.

1. **Conflicts.** `git fetch origin`, rebase or merge against `master`,
   resolve.
2. **Gate.** Invoke `/ci`. The gate is `dagger call check --source=.` —
   14 sub-gates run in parallel. Read the actual failure output. Never
   lower a threshold, skip a gate, or mark a failure "expected." If a
   lint-class gate fails (yaml/shell/python/frontmatter), consider
   `dagger call heal --source=. --model=gpt-4.1 --attempts=2` — bounded
   LLM repair of one failing gate.
3. **Self-review.** Read the whole diff as if seeing it cold. Strip
   debug, commented-out code, stale TODOs. Each file must earn its line
   count against the shape's stated goal.
4. **Review findings.**
   - Run `/code-review` if no verdict ref exists for the branch.
   - Read `.evidence/<branch>/review-synthesis.md` (or equivalent output).
   - For each finding: fix (in scope), defer (out of scope → git-bug or
     `backlog.d/`), or reject with specific reasoning (never "by design"
     without citing the decision).
5. **Re-gate.** After any change, re-run `/ci`. Synchronous — no async
   bots to wait on.
6. **Merge-readiness verification.**
   ```bash
   source scripts/lib/verdicts.sh
   verdict_check_landable "$branch"   # rc=0 ship/conditional, 1 missing/stale, 2 dont-ship
   (cd "$(git rev-parse --show-toplevel)" && dagger call check --source=.)
   ```

**Exit gate:** conflicts clean, `dagger call check` green, verdict ref
exists and matches HEAD, every review finding disposed.

If already clean on entry, skip to Phase 2.

### Phase 2: Polish — elevate

Read `references/pr-polish.md`.

**Goal:** from "works" to "exemplary."

1. **Hindsight review.** "Would we build it the same way starting over?"
   Look for shallow modules, pass-through layers, hidden coupling,
   temporal decomposition, missing or premature abstractions, tests that
   couple to implementation. Spellbook-specific smells: SKILL.md over
   500 lines (must extract to `references/`), skills that escape their
   own tree (`../..`, `$REPO_ROOT/…`), claim-coordination primitives
   under `skills/` (guarded by `check-no-claims`), raw
   `dagger call check` inside `skills/deliver/SKILL.md` (guarded by
   `check-deliver-composition`), hardcoded `/Users/<name>/` paths
   outside `harnesses/claude/` (guarded by `check-portable-paths`).
2. **Agent-first assessment.** Run `assess-review` (strong tier) and
   `assess-tests`. Address every `fail` before exiting this phase.
3. **Architecture edits.** Commit fixes atomically.
4. **Test audit.** Coverage gaps on error paths, boundary values, newly
   added branches. Assertions specific, not `assert x is not None`.
5. **Docs current.** `harnesses/shared/AGENTS.md`, skill `SKILL.md`
   bodies, `CLAUDE.md` map entries — update what the change touched.
6. **Cross-harness parity (Red Line).** If the change adds a new
   mechanism, confirm it works on Claude, Codex, AND Pi. One-harness-only
   designs fail the doctrine even if every sub-gate is green.
7. **Confidence.** State it with evidence — per concern if the branch
   spans more than one.

**Exit gate:** blocking smells fixed, assess-* `fail` findings resolved,
confidence stated, cross-harness parity verified.

If Phase 2 produced commits, return to Phase 1.

### Phase 3: Refactor — reduce complexity

Invoke `/refactor` as the simplification engine.

**Mandatory when net diff > 200 LOC.** For smaller diffs, do a manual
module-depth pass against the Ousterhout checklist (shallow modules,
information leakage, pass-throughs, shims with no active contract).

1. `/refactor` with auto-detected base branch; pass `--base master` only
   if detection is ambiguous.
2. **Select one bounded change.** Deletion > consolidation > state
   reduction > naming > abstraction. Every line fights for its life.
3. **Preserve behavior.** Run `dagger call check --source=.` after each
   refactor commit.
4. **Validate.** `assess-simplify` (strong tier) when available —
   `complexity_moved_not_removed` must be false to exit.

**Exit gate:** no obvious complexity to remove, or explicit justification
for keeping it.

If Phase 3 produced commits, return to Phase 1.

## Loop until done

```text
Phase 1 (fix) → Phase 2 (polish) → Phase 3 (refactor)
       ^                                      |
       +--------- if changes pushed ----------+
```

Termination: a full pass produces no changes.

## Landing (the `/land` mode)

Plain `/settle` stops at the end of the loop above with the branch
merge-ready. `/land` continues through the merge.

**Preferred path — `scripts/land.sh`:**

```bash
scripts/land.sh <branch>   # defaults to current branch
```

That script: resolves the target via `origin/HEAD` (falls back to
main/master/trunk), validates the verdict via `verdict_check_landable`,
runs `dagger call check` (when `dagger.json` + `dagger` are present),
then `git checkout <target> && git merge --no-ff <branch>`. It exits
non-zero on missing verdict (rc=2), `dont-ship` (rc=3), failing Dagger
(rc=4), or unresolved target (rc=5).

**Manual fallback** (when `scripts/land.sh` is unavailable or blocked):

```bash
source scripts/lib/verdicts.sh
verdict_check_landable "$branch"                   # must be rc=0
(cd "$(git rev-parse --show-toplevel)" \
   && dagger call check --source=.)                # must be green
git checkout master
git merge --no-ff "$branch"                        # trips pre-merge-commit gate
git push origin master                             # post-push, global bootstrap re-runs
```

**Backlog close-out** (if the branch implements a shape):

```bash
git mv backlog.d/NNN-<slug>.md backlog.d/_done/
git commit -m "close(NNN): <shape title>"
```

Include the close-out commit on the topic branch before merging, or
amend onto `master` immediately after the merge commit — before pushing.

**Emergency bypass:**

```bash
SPELLBOOK_NO_REVIEW=1 scripts/land.sh <branch>
```

Use only when the verdict mechanism itself is broken. Never as a reflex.
Document why in the merge message.

**Post-merge:** `.githooks/post-merge` re-runs `./bootstrap.sh`
automatically when skills/ or agents/ changed. Global harness symlinks
refresh without explicit action. Sanity-check on a consuming project
before walking away.

## What not to do

- **Declare "done" while `dagger call check` is running.** The gate is
  synchronous; wait for the exit code.
- **Lower a gate to make it green.** Thresholds and strictness are
  load-bearing. Fix root cause, never patch the gate.
- **Merge without a verdict.** `SPELLBOOK_NO_REVIEW=1` is the documented
  escape — document the reason when you use it. "I don't want to run
  `/code-review`" is not a reason.
- **Fast-forward into master to sidestep the verdict gate.** The hook
  only fires on non-FF; a silent FF merge is worse than a deliberate
  bypass.
- **Ignore the cross-harness Red Line.** A green gate + a one-harness
  design still violates the doctrine. Fail it in Phase 2.
- **Leave the backlog file in-place after shipping.** A merged shape
  that still lives in `backlog.d/` (not `_done/`) is a synchronization
  bug. Move it or re-open the work.
- **Edit `index.yaml` by hand to resolve `check-index-drift`.** The
  pre-commit hook regenerates it; hand-edits create churn or drift.
- **Skip re-bootstrap on another machine.** After pushing, global
  symlinks on *this* machine update automatically; clones on other
  machines need `./bootstrap.sh` manually.
- **Route through GitHub PRs when git-native is sufficient.** There's
  no review gate on GitHub here. A PR is overhead unless you explicitly
  want the external discussion surface.

## Output

Per phase:
- **Fix:** conflicts resolved, gate failures fixed, findings addressed
  (count + dispositions), verdict ref SHA.
- **Polish:** architecture changes, test gaps filled, cross-harness
  parity confirmed, confidence + evidence.
- **Refactor:** LOC delta, modules consolidated, abstractions deleted.
- **Land** (if `/land`): merge SHA, backlog item moved to `_done/`
  (yes/no), `dagger call check` rerun on `master` (optional sanity),
  bootstrap side-effects (if the post-merge hook re-ran it).
