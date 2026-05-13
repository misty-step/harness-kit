---
name: ship
description: |
  Final mile. Take a merge-ready branch to shipped: squash-merge, archive
  the backlog ticket(s) with Closes-backlog trailers preserved into the
  merge commit, update touched docs, run /reflect, and apply its outputs.
  Assumes /settle has already left the branch merge-ready — ship does not
  run CI, code-review, or refactor itself. If those aren't done, run
  /settle first.
  Use when: "ship it", "merge and close out", "final mile", "land and
  reflect", "finish this ticket".
  Trigger: /ship.
argument-hint: "[branch-or-pr]"
---

# /ship

The final mile. Branch is merge-ready; `/ship` lands it, archives the
ticket(s), syncs docs, runs `/reflect`, and threads reflect's outputs back
into the repo. One command from "green" to "shipped and learned from."

## Stance

1. **Act, do not propose.** `/ship` has authority within its domain.
   Archive, merge, pull, reflect, apply. Escalate only on refuse conditions.
2. **Never lose trailer context.** `Closes-backlog` trailers must survive
   into the squash commit on master. `/groom` sweeps master by trailer —
   a dropped trailer is a ticket that never closes.
3. **Pre-merge prep belongs on the shipping branch.** Archive moves and
   doc syncs go on the feature branch before the squash so the merge
   commit itself carries a single, clean closure event.
4. **Reflect's harness edits never touch master.** They land on a
   `harness/reflect-outputs` branch for human review. This is a hard
   invariant from `reflect/SKILL.md`.
5. **Not a CI runner, not a reviewer, not a refactorer.** `/ship` assumes
   `/settle` already proved the branch clean. If `/settle` wasn't run,
   refuse and route the operator back.

## Prerequisites

Assert at start; refuse with a clear reason on any miss.

- On a feature branch (not `master` / `main` / default protected branch).
- Branch name matches `^(feat|fix|chore|refactor|docs|test|perf)/([0-9]+)-`.
  The numeric capture is the **primary backlog ID** being shipped.
- Working tree clean (`git status --short` empty).
- If a PR exists for the branch: `gh pr view --json mergeable,mergeStateStatus`
  reports mergeable. A conflicted or blocked PR means `/settle` isn't done.
- Landability evidence exists for this exact HEAD. Acceptable evidence:
  GitHub mode has a mergeable PR with green required checks; git-native
  mode has a `ship` or `conditional` verdict; or git-native mode has
  operator-provided/current-session local gate receipts from `/ci` or the
  repo's documented gate. Do not require a PR or verdict solely to land a
  locally verified git-native branch. A `dont-ship` verdict still blocks.

## Process

### 1. Extract backlog IDs

Primary ID from the branch name regex capture. Then scan branch commits:

```sh
git log --format=%B master..HEAD \
  | git interpret-trailers --parse --no-divider
```

Collect every `Closes-backlog:` and `Ships-backlog:` value (closing) plus
every `Refs-backlog:` value (reference-only). Merge with the primary ID.

- **Closing set:** primary ID ∪ Closes-backlog ∪ Ships-backlog.
- **Reference set:** Refs-backlog values. Noted in the final report, never
  archived.

Prefer `backlog_ids_from_range master..HEAD` from `scripts/lib/backlog.sh`
when available.

### 2. Archive backlog files on the shipping branch

For each ID in the closing set:

```sh
source scripts/lib/backlog.sh
backlog_archive "<id>"
```

This performs `git mv backlog.d/<id>-*.md backlog.d/_done/`. Stage the
moves. Idempotent — already-archived IDs exit 0 silently.

If the primary ID has no matching file AND no trailers were found,
**refuse** (see Refuse Conditions): the branch is shipping something with
no backlog association.

### 3. Sync touched docs

Inspect the diff to find docs that may have gone stale:

```sh
git diff master..HEAD --name-only
```

If the downstream repo has a drift contract (e.g.
`docs/context/DRIFT-WATCHLIST.md`), read it and cross-reference the
changed paths. When doc updates are required and not yet present,
dispatch a focused **general-purpose** subagent with:

- The exact list of changed source files.
- The exact doc paths to update.
- A bounded scope: "update X to reflect Y, no new docs."

**Do not invent docs that don't already exist.** If the repo has no drift
contract, skip this step and note it in the final report.

### 4. Create the archive commit on the feature branch

One commit. Subject: `chore(backlog): archive shipped tickets`.

Inject every closing ID as a separate trailer — do not hand-format:

```sh
msg="chore(backlog): archive shipped tickets"
for id in $CLOSING_IDS; do
  msg="$(printf '%s' "$msg" \
    | git interpret-trailers \
        --if-exists addIfDifferent \
        --trailer "Closes-backlog: $id")"
done
git commit -m "$msg"
```

Body stays minimal. The trailers are the contract; prose is optional.

### 5. Squash-merge

**GitHub mode** (PR exists, `gh` available):

Construct a squash body that carries every closing trailer. GitHub's
default squash template often drops commit trailers, so pass the body
explicitly:

```sh
body="$(git log --format=%B master..HEAD \
        | git interpret-trailers --parse --no-divider \
        | grep -E '^(Closes-backlog|Ships-backlog|Refs-backlog):' \
        | sort -u)"
gh pr merge --squash --body "$body"
```

Include a one-line subject summarizing the shipped work above the trailer
block. Match the repo's squash-subject convention (look at recent
`git log master --merges`).

**Git-native mode** (no PR, no `gh`, or no GitHub remote):

```sh
git checkout master
git merge --squash <branch>
git commit -F <constructed-message-file>
```

Detect mode by: remote URL + `gh` on PATH + `gh pr view` exit code.
GitHub mode is preferred when available because it records the merge in
the PR timeline.

### 6. Pull master and verify trailers

```sh
git checkout master
git pull --ff-only
git log -1 --format=%B | git interpret-trailers --parse --no-divider
```

The output must contain `Closes-backlog: <id>` for every ID in the
closing set. If any are missing, **stop and escalate** — the squash body
construction dropped them and the fix must happen before `/groom` next
sweeps.

### 7. Invoke `/reflect cycle`

Bounded scope: the just-shipped work only. Pass as context:

- Branch name (pre-merge).
- Merged SHA on master.
- Closing backlog IDs.
- Reference IDs (non-closing).

Capture reflect's outputs:

- Backlog mutations (new tickets, edits, reprioritizations).
- Harness-tuning proposals (skill/agent/hook/AGENTS.md edits).
- Retro notes and coaching output.

### 8. Apply reflect's backlog mutations on master

Reflect may propose new tickets, edits to open tickets, or deletions. Apply
them in-tree: add files to `backlog.d/`, edit existing tickets. Commit to
master:

```
chore(backlog): apply reflect outputs from shipping <primary-id>
```

If reflect proposed no backlog mutations, skip this commit.

### 9. Apply harness-tuning outputs to a harness branch

Reflect's harness proposals **never** land on master. Create or checkout
the branch:

```sh
git checkout -B harness/reflect-outputs master
```

Apply the harness edits there. Commit per-concern (match `/yeet` commit
discipline). Push:

```sh
git push -u origin harness/reflect-outputs
```

If the branch already exists with prior suggestions, rebase onto master
first, then add the new commits. Report the branch name so a human can
review.

Return to master before finishing:

```sh
git checkout master
```

### 10. Final report

Emit a single block covering:

- Merged SHA on master and PR number (if GitHub).
- Closing IDs archived.
- Reference IDs noted.
- Docs touched (path list) or "none required."
- Reflect outputs grouped by category: backlog mutations applied, harness
  proposals on `harness/reflect-outputs`, retro notes, coaching.
- Harness branch name.
- Residual risk or follow-ups, if any.

## Refuse Conditions

Stop and surface to the user instead of shipping:

- Branch name doesn't match `^(type)/(\d+)-` — no primary ID extractable.
- Working tree dirty.
- On `master` / `main` directly.
- Verdict ref reads `dont-ship` (`verdict_check_landable` returns 2).
- No same-HEAD landability evidence exists: no green PR checks, no
  landable verdict, and no operator-provided/current-session local gate
  receipt.
- In GitHub mode, `gh pr checks` is red. Do not add a `--force` flag;
  refuse.
- If a PR exists, it is not mergeable per
  `gh pr view --json mergeable,mergeStateStatus`.
- Primary ID has no `backlog.d/<id>-*.md` file AND no closing trailers on
  any branch commit — shipping with no backlog association. Operator must
  add a ticket or add a marker commit and re-run.
- Rebase / merge / cherry-pick in progress (`.git/MERGE_HEAD`,
  `.git/CHERRY_PICK_HEAD`, `rebase-*` dir).

## Trailer Conventions

Every ticket closure flows through git trailers. Keys recognized by
`scripts/lib/backlog.sh`:

- `Closes-backlog: <id>` — closes the ticket (archival intent).
- `Ships-backlog: <id>` — synonym for Closes-backlog, closes the ticket.
- `Refs-backlog: <id>` — references the ticket without closing it.

Example trailer block on a squash merge commit:

```
feat(lane): add adaptive backoff to dispatcher

Closes-backlog: 029
Closes-backlog: 031
Refs-backlog: 024
```

IDs are bare numeric strings (`029`, not `BACKLOG-029`). Trailers are
injected via `git interpret-trailers --trailer`, never hand-formatted, to
avoid whitespace and key-casing drift.

## GitHub Mode vs Git-Native Mode

| Mode | Detection | Merge command |
|---|---|---|
| GitHub | remote URL + `gh` on PATH + `gh pr view` succeeds | `gh pr merge --squash --body "<trailers>"` |
| Git-native | no PR, no `gh`, or no GitHub remote | `git merge --squash <branch> && git commit -F <msg>` |

GitHub mode is preferred when available because the PR timeline records
the merge. Behavior is otherwise identical: squash-only, trailer-preserving.

## Interactions

- **Upstream:** `/settle` leaves the branch merge-ready. `/ship` assumes
  that work is done; it does not re-run CI, code-review, or refactor.
- **Invokes:** `/reflect cycle` for retro, backlog mutations, and
  harness proposals.
- **Invoked by:** `/flywheel` as the landing + reflection stage of each
  cycle. `/flywheel` reads `/ship`'s final report to decide the next
  cycle.
- **Complements `/yeet`:** `/yeet` ships the working tree to the remote
  (commits + push). `/ship` ships the branch to master (merge + archive
  + reflect). Both are imperative finals; they operate at different
  layers.

## Gotchas

- **GitHub default squash body drops trailers.** `gh pr merge --squash`
  with no `--body` often uses the PR title + description, not commit
  trailers. Always pass `--body` with the trailer block explicitly.
- **All trailers live in ONE contiguous block at the end of the message.**
  A blank line between `Closes-backlog: NNN` lines and `Co-Authored-By:`
  splits the block; `git interpret-trailers --parse` only recognizes
  the last block, so downstream `backlog_ids_from_commit` returns empty.
  Use `git interpret-trailers --if-exists addIfDifferent --trailer "..."`
  to inject programmatically — it handles block boundaries correctly.
- **Archive before merge, not after.** Archiving on master after the
  merge splits the closure event across two commits and muddies `/groom`
  sweeps. One commit on the feature branch; one squash commit on master.
- **Primary ID without a file is a real case.** When the ticket was added
  via trailers only (hotfix, spike), there may be no `backlog.d/<id>-*.md`
  to move. Trust the trailers; don't fail the archive step on a missing
  file, but do note it.
- **Reflect must not mutate master's harness.** This is not a style
  preference — `reflect/SKILL.md` encodes it as an invariant. Harness
  edits go to `harness/reflect-outputs`, full stop. A `/reflect` run that
  writes to master's `.claude/`, `.agents/`, `AGENTS.md`, or `CLAUDE.md`
  is a bug; surface it.
- **Re-running `/ship` on an already-shipped branch.** The branch is
  gone, the PR is closed. Detect and exit early; do not attempt to
  re-archive or re-reflect.
- **Trailer deduplication.** `Closes-backlog: 029` appearing in three
  branch commits must squash to one trailer on master, not three. The
  `interpret-trailers --if-exists addIfDifferent` flag handles this;
  don't sort-and-paste manually.
- **Library repos.** No deploy target, but `/ship` still merges and
  reflects. `/flywheel` decides whether `/deploy` runs after.

## Output

Single report, plain text:

```
/ship complete

Merged:     <sha> on master (PR #<n>)
Closed:     029, 031
Referenced: 024
Docs:       docs/context/lane-runtime.md (synced)
Reflect:    2 backlog mutations applied, 3 harness proposals on
            harness/reflect-outputs, retro in .spellbook/reflect/<cycle>/
Residual:   none
```

On refuse, emit the reason and the action the operator must take to
re-enable shipping.
