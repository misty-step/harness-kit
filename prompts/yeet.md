---
description: Tidy the worktree, split semantic conventional commits, push. Judgment over git, no approval gates.
argument-hint: "[--dry-run] [--single-commit] [--no-push]"
---

Take the current worktree to the remote: classify everything, commit what
belongs in reviewable pieces, push, end clean. Act — don't propose.

Classify every changed/untracked path: **signal** (commit), **debris**
(`.DS_Store`, swap files, stray logs — delete), **drift** (unrelated work —
own commit, move out, or durable ignore; never leave it), **evidence**
(route to the repo's evidence dir convention), **scratch** (move outside the
repo), **secret-risk** (`.env`-like content, key material, token patterns —
REFUSE and surface).

Group signal into semantic commits: one concern per commit; tests ride with
their code; config that enables a feature ships with it; refactors commit
before the features built on them; if you'd describe it as "X and also Y,"
it's two commits. Conventional Commits — type(scope): imperative subject;
body explains *why*, never restates the diff; match scope/co-author style to
`git log -20`, not a template.

Refuse instead of committing when: merge/rebase/cherry-pick in progress,
conflict markers in the diff, secret-risk files, detached HEAD, or pushing
would write a protected default branch absent explicit instruction.

Never: force-push, `--no-verify`, blind `git add -A`, delete unclassified
files, or declare success while `git status --short --untracked-files=all`
shows paths or `git rev-list --left-right --count <branch>...<upstream>`
isn't `0 0`. If a hook fails, fix the cause. If push rejects, pull-rebase and
retry once, then stop.

Report: one line per commit (sha, type, subject), what was deleted/moved/
ignored and why, push result, final worktree + remote status.
