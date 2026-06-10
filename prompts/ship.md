---
description: Final mile — squash-merge committed work to master, archive backlog tickets via trailers, sync docs, push.
argument-hint: "[branch-or-pr]"
---

Land the current committed work on `master` and close it out. Act — don't
propose. Master is the only destination; one squash commit is the only
landing shape (no plain merge, no rebase-merge, no `main` fallback).

Preconditions — refuse with the exact blocker otherwise: clean tree; the
repo's documented gate green on this exact HEAD (for Harness Kit:
`cargo run --locked -p harness-kit-checks -- check --repo .`); if a PR
exists it is mergeable against master. Detached HEAD is fine: create
`ship/<timestamp>-<shortsha>` first and ship that.

Backlog closure (trailer canon: `meta/CONTRACTS.md`):
1. Collect closing IDs from the branch name (`feat/NNN-…`) and from
   `Closes-backlog:` / `Ships-backlog:` trailers in branch commits
   (`Refs-backlog:` is reference-only, never archived).
2. Archive each on the shipping ref BEFORE merging —
   `harness-kit-checks backlog archive <id>` where available, else
   `git mv backlog.d/<id>-*.md backlog.d/_done/` — and commit with one
   `Closes-backlog: <id>` trailer per ID via `git interpret-trailers`
   (never hand-formatted).
3. Update docs the diff made stale; don't invent new ones.
4. Squash-merge. GitHub mode: `gh pr merge --squash --body "<trailer block>"`
   — pass trailers explicitly; GitHub's default squash body drops them.
   Git-native: `git merge --squash` + commit with the trailer block.
5. Verify on master: `git log -1 --format=%B | git interpret-trailers
   --parse` shows every closing ID. Missing trailer = stop and fix before
   anything else.
6. Push master; confirm `master...origin/master` is `0 0`.

A branch with no backlog IDs still ships; report `Closed: none`. Already on
master? Verify, push, report — don't manufacture a merge. End with: merged
SHA, closed/referenced IDs, docs touched, final status + remote sync, and
anything learned worth folding back into the harness (propose, don't
self-apply doctrine edits on master).
