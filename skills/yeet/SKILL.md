---
name: yeet
description: |
  End-to-end "ship it to the remote": read worktree, classify changes, tidy
  debris, split semantic commits, and push. Judgment layer over git, not a
  wrapper; decides what belongs and how reviewers should read the diff.
  Use when: "yeet", "yeet this", "commit and push", "ship it", "tidy and
  commit", "wrap this up and push", "get this off my machine".
  Trigger: /yeet, /ship-local (alias).
argument-hint: "[--dry-run] [--single-commit] [--no-push]"
---

# /yeet

Take the current worktree state → one or more conventional commits → remote.
One command. Executive authority. No approval gates.

## Stance

1. **Act, do not propose.** The skill has authority within its domain. Stage
   what belongs, leave out what doesn't, delete debris, split logically, push.
   Escalate only on red-flag state (see Refuse Conditions).
2. **Clean tree is the deliverable.** `/yeet` is not done while
   `git status --short` still shows modified, staged, or untracked paths.
   Resolve every path by commit, ignore, move out of the repo, or delete.
3. **Reviewability is the product.** A stack of three focused commits beats
   one 2,000-line "wip" commit, every time. Split on semantic boundaries
   even if the tree was built in one session.
   Provider-roster receipts are part of classification evidence when present.
   If classification requires substantive judgment and no receipts exist,
   dispatch two or more roster members before packaging completed work.
4. **Never lose work.** Untracked scratch that might be the user's in-flight
   thinking gets moved, not deleted, unless it's unambiguous debris.
5. **Conventional Commits, always.** Type, optional scope, imperative subject.
   Body explains *why*, not *what*.

## Delegation Floor

When a provider roster is available (repo `.harness-kit/agents.yaml` or system `~/.harness-kit/agents.yaml`), `/yeet` uses two or more roster members
for substantive worktree classification, commit-boundary design, or risky
cleanup decisions. Use lanes for diff classification and risk review; give
them scoped status, diff, and backlog context while withholding final commit
intent until synthesis. The lead owns final staging, commit messages,
verification, and receipts. Direct lead-only yeet work is limited to
mechanical git commands, emergency unblocks, explicit user-forbidden
delegation, or an explicit waiver when fewer than two roster members are
available.

## Modes

- Default: stage → split into commits → push.
- `--dry-run`: report the plan (commit boundaries, messages, skips), do not execute.
- `--single-commit`: skip the split pass; one commit for everything that belongs.
- `--no-push`: commit locally but don't push. Useful when the user wants to
  amend before going remote.

## Process

### 1. Read the worktree holistically

- `git status --short` (untracked, modified, staged — full picture, don't truncate)
- `git diff --stat` + `git diff --stat --cached` (sizes + files)
- `git log -5 --oneline` (recent commit style)
- `git rev-parse --abbrev-ref HEAD` (branch, for push target)
- `git status` for rebase/merge/cherry-pick in progress (see Refuse Conditions)

If the tree is clean, say so and exit.

### 2. Classify every file

For each changed / untracked path, assign one of:

| Class | Meaning | Action |
|---|---|---|
| **signal** | Work the user meant to do | Include in a commit |
| **debris** | Unambiguous trash (`.DS_Store`, `*.log` scratch, `.orig`, editor swap files, `node_modules` that slipped the gitignore) | Delete outright |
| **drift** | Unrelated work from another concern / earlier session | Separate commit, move out of repo, or ignore with explicit rationale; never leave it unresolved |
| **evidence** | Logs, screenshots, walkthrough artifacts relevant to the feature | Include if the branch convention tracks them; otherwise move to the evidence dir |
| **scratch** | Half-written notes, TODO files, planning docs | Move to a vault/scratch dir outside the repo, or delete if trivial |
| **secret-risk** | Contains plausible credentials / tokens / .env-like content | REFUSE the commit, surface to user |

**Heuristics:**
- Filename in `.gitignore` templates (node_modules, __pycache__, *.pyc, dist/) → debris.
- `.DS_Store`, `Thumbs.db`, `*.swp`, `*.swo`, `*~`, `.#*` → debris.
- Files matching `.env*` not in .gitignore → secret-risk, refuse.
- grep diff for `-----BEGIN.*PRIVATE KEY-----`, `api[_-]?key.*=.*["'][^"']{20,}`,
  `(AKIA|ghp_|github_pat_|sk-)[A-Za-z0-9]{16,}` → secret-risk, refuse.
- New untracked dirs with only logs / timestamped filenames → evidence; route to
  the repo's established evidence dir if it has one (`walkthrough/`, `.evidence/`, etc.).
- If the user has working-tree changes that DON'T trace to commits already
  on this branch (e.g. random edits in an unrelated module), they're drift —
  split them into their own commit, move them out of the repo, or add a
  durable ignore rule. Do not leave them behind in the worktree.

### 3. Group signals into semantic commits

Group rules:
- **One concern per commit.** Separate feature from refactor from chore.
- **Co-changed tests belong with their code.** A new feature + its tests is one commit; don't split them.
- **Config that enables the feature goes with the feature.** The env.ts change
  that adds a knob for the new lane ships with the lane.
- **Cross-cutting infrastructure changes are their own commit.** Dagger
  scaffolding, formatter config, CI wiring — separate from feature work.
- **Refactors before features.** If the diff contains a pure refactor AND a
  feature that builds on it, commit the refactor first (makes bisect sane).
- **Carmack's stapled-PR rule**: if you'd describe the change as "X and also
  Y," it's two commits.

If the user passed `--single-commit`, skip grouping; everything signal-class
becomes one commit.

### 4. Write commit messages

Conventional Commits. Format:

```
<type>(<scope>): <imperative subject under 72 chars>

<optional body: why, not what. Wrap at 72.>

<optional footer: BREAKING CHANGE, refs, co-author>
```

**Types:** `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`,
`chore`, `style`. No `wip`, no `misc`, no `update`.

**Scope:** match the convention in the last 20 commits on the branch. If
the log shows `feat(attacker): …`, use `attacker`, not `red-team-attacker`
or `orchestrator`.

**Subject rules:**
- Imperative ("add", not "added" or "adds").
- No trailing period.
- Don't reference PR numbers or issue IDs unless the project convention does.

**Body rules:**
- Omit entirely if the subject is self-explanatory.
- When present, explain the *why* — the constraint, the incident, the reason
  this was the right call over alternatives.
- Do NOT restate the file-level diff.

**Co-author:** Append when the session did real implementation work. Match
the project's existing co-author line.

### 5. Stage, commit, push

- `git add` only the signal paths for each commit (path-by-path, not `git add -A`).
- `git commit` per group. Allow hooks to run (`lefthook`, `pre-commit`). If
  a hook fails, investigate and fix the underlying issue — do not `--no-verify`.
- After the final commit: `git push`. If the upstream isn't set, `git push -u origin <branch>`.
- If `git push` is rejected (upstream moved), pull-rebase (if linear) and retry
  once. Do NOT force-push on retry.
- After push, rerun `git status --short --untracked-files=all`. If any path
  still appears, continue classifying and resolve it. `/yeet` exits only on a
  clean worktree; ignored files are acceptable, visible status entries are not.

### 6. Report

What got committed (one line per commit: sha, type, subject).
What got removed, ignored, or moved and why.
Push target + result.
Final worktree status (`clean` or refuse).

## Refuse Conditions

Stop and surface to the user instead of committing:

- `.git/MERGE_HEAD`, `.git/CHERRY_PICK_HEAD`, or `rebase-*` dir exists — mid-operation.
- Diff contains unresolved conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`).
- Any file classified `secret-risk`.
- Current branch is `main` / `master` / default protected branch AND push would
  write to that branch (absent explicit user request).
- HEAD is detached.
- The worktree has >500 files changed AND no obvious semantic grouping — ask
  the user if this is really one session's work or if something unexpected happened.

## Safety rails (never)

- Never force-push.
- Never `--no-verify` to bypass hooks.
- Never `git add -A` at the repo root without classifying first.
- Never `git clean -fdx` or delete directories without individual-file classification.
- Never commit files whose content matches known secret patterns (above).
- Never declare success while `git status --short` still shows paths.

## Gotchas

- **"Tidy" is not refactor.** This skill stages and commits — it does not
  edit source code to make it prettier. If the diff is messy, that's a
  `/refactor` concern, not `/yeet`.
- **Match the log, not a template.** If the project writes `chore:` commits
  without scopes, don't retroactively add `chore(orchestrator):`. Read
  `git log -20` and match.
- **Untracked dirs are commonly overlooked.** `git add` doesn't recurse into
  new dirs by default unless you pass the dir path. Do classify new dirs
  directory-by-directory.
- **Evidence needs a home.** If a repo keeps walkthrough artifacts, commit them
  there. If it treats them as local review material, add or reuse a durable
  ignore rule and keep the worktree clean anyway.
- **Pre-commit hooks can reformat.** If lefthook's `stage_fixed: true`
  mutates files during commit, they're still part of that commit — good.
  Don't panic and re-stage.
- **Large diffs tempt single-commit laziness.** Resist. A reviewer reading
  20 files across 3 concerns will miss bugs in all three.
- **Don't describe what the diff already says.** "update attacker-dispatcher.ts
  to add kill switch check" is filler. "kill switch was inert because dedup
  lanes don't supply runId; move to ledger-backed scheduled ingress" is a
  reason.
- **Co-author lines match project style.** Some repos use Anthropic's line,
  some use the user's, some use neither. Grep recent log for `Co-Authored-By`
  before deciding.
- **Push rejection on first try is usually benign**: upstream moved.
  Rebase-pull + push once. If it rejects again, stop — something weirder is
  happening.

## Output

```markdown
## /yeet Report

Classified 42 paths: 35 signal, 3 debris, 2 drift, 2 evidence.
Deleted: .DS_Store, orchestrator/scratch.md, test.log
Ignored: orchestrator/walkthrough/qa-audit/ (local QA evidence)
Moved out of repo: notes/todo.md → ~/vault/vulcan/todo.md

Commits:
  abc1234 refactor(attacker): consolidate NetworkPolicyRule type
  def5678 fix(attacker): ledger-backed scheduled task + kill-switch
  9012345 ci(dagger): scaffold Dagger pipeline; thin GHA to pointer

Pushed feat/014-red-team-attacker-sprite → origin (3 new commits).
Worktree: clean
```

On refuse:

```markdown
## /yeet — REFUSED
Reason: orchestrator/.env.prod contains plausible secret
  (matches /sk-[A-Za-z0-9]{32}/ at line 12).
Action: remove or gitignore the file before re-running.
```
