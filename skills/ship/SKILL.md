---
name: ship
description: |
  Final mile from merge-ready branch to shipped: squash-merge to master,
  archive backlog tickets with trailers, update touched docs, run /reflect,
  apply outputs.
  Assumes /deliver or /deliver --polish-only already made the branch ready.
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
4. **Ship always means squash to master.** `/ship` always lands the current
   merge-ready branch by creating one squash commit on `master`. No normal
   merge, no rebase-merge, no fast-forward-only landing, no destination
   inference from `origin/HEAD`, and no `main` fallback. GitHub mode may use
   GitHub's squash-merge operation, but the destination remains `master` and
   trailers must be passed explicitly.
5. **Reflect's harness edits never touch master.** They land on a
   `harness/reflect-outputs` branch for human review. This is a hard
   invariant from `reflect/SKILL.md`.
6. **Not a CI runner, not a reviewer, not a refactorer.** `/ship` assumes
   `/deliver --polish-only` already proved the branch clean. If it wasn't
   run, refuse and route the operator back.
7. **Roster receipts are required evidence.** In repos with
   `.harness-kit/agents.yaml`, verify that `/deliver --polish-only` or the
   documented landability evidence includes two or more roster-member receipts
   or an explicit exception before final-mile merge work.
8. **Shipped means tidy and synced.** Shared Closeout applies: no visible
   paths, no unpushed commits, and `master...origin/master` is `0 0`.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Normally verify upstream roster receipts; if final-mile work surfaces substantive judgment, route back to /deliver --polish-only or dispatch release-risk and closure-state review lanes.

## Prerequisites

Assert at start; refuse with a clear reason on any miss.

- On a feature branch (not `master` / `main` / default protected branch).
- `master` exists locally or can be fetched from `origin/master`; `master` is
  the only landing destination.
- If the branch name matches `^(feat|fix|chore|refactor|docs|test|perf)/([0-9]+)-`,
  the numeric capture is the **primary backlog ID** being shipped. If it does
  not match, `/ship` continues with no primary backlog ID and closes only IDs
  found in trailers or explicit archive evidence.
- Working tree clean. See `harnesses/shared/AGENTS.md` (Closeout).
- If a PR exists for the branch: `gh pr view --json baseRefName,mergeable,mergeStateStatus`
  reports `baseRefName` as `master` and mergeable. A non-master PR base,
  conflicted PR, or blocked PR means `/deliver --polish-only` isn't done.
- Landability evidence exists for this exact HEAD. Acceptable evidence:
  GitHub mode has a mergeable PR with green required checks; git-native
  mode has a `ship` or `conditional` verdict; or git-native mode has
  operator-provided/current-session local gate receipts from `/ci` or the
  repo's documented gate. Do not require a PR or verdict solely to land a
  locally verified git-native branch. A `dont-ship` verdict still blocks.
  For Harness Kit and any repo with `dagger.json`, the documented gate is
  `dagger call check --source=.` on the exact HEAD being landed.
- Acceptance evidence exists for this exact HEAD (`git rev-parse HEAD`):
  exact behavior changed, live repo evidence read, acceptance source,
  command/path exercised, repo-fit check, and residual unverified paths.
  Refuse stale evidence that names a different SHA or omits the SHA when the
  upstream phase could have produced one. When the oracle depends on a fixture, contract, golden
  file, screenshot, Gherkin feature, transcript, or equivalent acceptance
  artifact, the evidence includes its `sha256` hash. If acceptance criteria,
  artifact contents, or assertion strength changed, the evidence includes an
  explicit `Contract-change acknowledgment:` line. If `.evidence/<branch>/<date>/`
  is non-empty, `/ship` carries it into the final commit as `QA-Evidence:`.
- Trace handoff inputs exist or a waiver is explicit: transcript refs, review
  receipt refs, QA/demo refs, or another durable trace source are available for
  `/trace` after merge. If no transcript or trace artifact is available, the
  operator provides a `Trace-waiver: <reason>` line. Raw session logs do not
  satisfy this prerequisite.
- If `.harness-kit/agent-readiness.yaml` exists, the branch evidence includes
  readiness impact: improved, preserved, or regressed. Regressions must include
  a contract-change reference and a valid future-expiring waiver.

## Work Ledger

When `.harness-kit/work/ledger.jsonl` is available, `/ship` consumes the latest
completed `/deliver` record for the closing backlog/branch. It calls
`cargo run --locked -p harness-kit-checks -- work-ledger append` for `phase_started` at final-mile start,
`next_action_changed` after merge while trace/reflect remains, `blocker_added`
on a refuse condition, and `phase_completed` with `status=completed` after
trailers, trace handoff, reflect, and follow-up mutations are done.

## Process

### 1. Extract backlog IDs

Primary ID is optional. If the branch name matches
`^(feat|fix|chore|refactor|docs|test|perf)/([0-9]+)-`, use the numeric capture.
Then scan branch commits:

```sh
git log --format=%B master..HEAD \
  | git interpret-trailers --parse --no-divider
```

Collect every `Closes-backlog:` and `Ships-backlog:` value (closing) plus
every `Refs-backlog:` value (reference-only). Merge with the primary ID when
one exists.

- **Closing set:** optional primary ID ∪ Closes-backlog ∪ Ships-backlog.
- **Reference set:** Refs-backlog values. Noted in the final report, never
  archived.

Prefer `cargo run --locked -p harness-kit-checks -- backlog ids-from-range master..HEAD`
when available.

### 2. Archive backlog files on the shipping branch

For each ID in the closing set:

```sh
cargo run --locked -p harness-kit-checks -- backlog archive "<id>"
```

This performs `git mv backlog.d/<id>-*.md backlog.d/_done/`. Stage the
moves. Idempotent — already-archived IDs exit 0 silently.

If the closing set is empty, skip archive work and report `Closed: none`.
Do not refuse solely because a branch has no backlog association; non-backlog
design, docs, infrastructure, and emergency branches still squash-merge to
master when all landability and acceptance evidence exists.

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
evidence="$(cargo run --quiet --locked -p harness-kit-checks -- evidence trailer 2>/dev/null || true)"
if [ -n "$evidence" ]; then
  msg="$(printf '%s' "$msg" \
    | git interpret-trailers \
        --if-exists addIfDifferent \
        --trailer "$evidence")"
fi
git commit -m "$msg"
```

Body stays minimal. The trailers are the contract; prose is optional.

### 5. Squash-merge to master

Before merging, fetch and update `master`:

```sh
if git remote get-url origin >/dev/null 2>&1; then
  git fetch origin master
fi
git checkout master
if git remote get-url origin >/dev/null 2>&1; then
  git pull --ff-only origin master
fi
git checkout -
```

The only acceptable landing result is one new squash commit on `master`.
Never use plain `git merge`, `git merge --ff-only`, `git rebase` as the landing
operation, or any destination other than `master`.

**GitHub mode** (PR exists, `gh` available):

Construct a squash body that carries every closing trailer. GitHub's
default squash template often drops commit trailers, so pass the body
explicitly:

```sh
body="$(git log --format=%B master..HEAD \
        | git interpret-trailers --parse --no-divider \
        | grep -E '^(Closes-backlog|Ships-backlog|Refs-backlog|QA-Evidence):' \
        | sort -u || true)"
gh pr merge --squash --body "$body"
```

Include a one-line subject summarizing the shipped work above the trailer
block. Match the repo's squash-subject convention (look at recent
`git log master --merges`).

**Git-native mode** (no PR, no `gh`, or no GitHub remote):

```sh
dagger call check --source=.
git checkout master
git merge --squash <branch>
git commit -F <constructed-message-file>
```

Detect mode by: remote URL + `gh` on PATH + `gh pr view` exit code.
GitHub mode is preferred when available because it records the merge in
the PR timeline. Both modes still produce exactly one squash commit on
`master`. The Dagger command is explicit here because squash merges do not
invoke Git's `pre-merge-commit` hook.

### 6. Pull master and verify trailers

```sh
git checkout master
git pull --ff-only
git log -1 --format=%B | git interpret-trailers --parse --no-divider
```

The output must contain `Closes-backlog: <id>` for every ID in the
closing set. If any are missing, **stop and escalate** — the squash body
construction dropped them and the fix must happen before `/groom` next
sweeps. If the shipping branch had a non-empty `.evidence/<branch>/<date>/`,
the output must also contain `QA-Evidence: <path>`.

### 7. Record trace handoff

After the merge SHA is known, write or link the final work record. Preferred
local form:

```sh
cargo run --locked -p harness-kit-checks -- trace-record append \
  --backlog "<primary-id>" \
  --branch "<pre-merge-branch>" \
  --commit "<branch-head-before-merge>" \
  --reviewer-verdict-ref "<receipt-or-verdict-ref>" \
  --qa-ref "<qa-evidence-ref>" \
  --demo-ref "<demo-ref-if-any>" \
  --transcript-ref "<redacted-transcript-ref>" \
  --shipped-ref "master@<merged-sha>"
```

If no safe transcript exists, use `--waiver-reason "<why no transcript was
available>"`. If no local JSONL store is appropriate, the final report must
name another durable trace ref such as a Git note or PR body section. Do not
persist raw session logs.

### 8. Invoke `/reflect cycle`

Bounded scope: the just-shipped work only. Pass as context:

- Branch name (pre-merge).
- Merged SHA on master.
- Closing backlog IDs.
- Reference IDs (non-closing).
- Accepted Acceptance Evidence refs for the exact shipped HEAD.

Capture reflect's outputs:

- Backlog mutations (new tickets, edits, reprioritizations).
- Harness-tuning proposals (skill/agent/hook/AGENTS.md edits).
- Retro notes and coaching output.

### 9. Apply reflect's backlog mutations on master

Reflect may propose new tickets, edits to open tickets, or deletions. Apply
them in-tree: add files to `backlog.d/`, edit existing tickets. Commit to
master:

```
chore(backlog): apply reflect outputs from shipping <primary-id>
```

If reflect proposed no backlog mutations, skip this commit.

### 10. Apply harness-tuning outputs to a harness branch

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

### 11. Final report

Before reporting completion, run shared Closeout: final `git status` plus
`git rev-list --left-right --count master...origin/master`; keep working or
stop blocked on any visible path or nonzero divergence.

Emit a single block covering:

- Merged SHA on master and PR number (if GitHub).
- Closing IDs archived, or `none`.
- QA evidence trailer path, or "none" when the branch had no committed
  `.evidence/<branch>/<date>/` artifacts.
- Reference IDs noted.
- Trace/work-record ref, or explicit no-trace/no-transcript waiver.
- Docs touched (path list) or "none required."
- Reflect outputs grouped by category: backlog mutations applied, harness
  proposals on `harness/reflect-outputs`, retro notes, coaching.
- Accepted Acceptance Evidence refs carried into `/reflect` and final report.
- Harness branch name.
- Final workspace status and remote sync (`master...origin/master`, expected
  `0 0`).
- Roster delegation report: provider lanes used upstream and in final-mile
  work, why each was dispatched, parallel/split/competing-worktree pattern,
  provider_status and attempt_status totals, lead_verdict totals, accepted
  synthesis, rejected or failed lanes, and any waiver/exception. Prefer
	  `cargo run --locked -p harness-kit-checks -- summarize-delegations --format text` scoped to the closing
  backlog ref when receipts exist.
- Residual risk or follow-ups, if any.

## Refuse Conditions

Stop and surface to the user instead of shipping:

- `master` does not exist locally and `origin/master` cannot be fetched.
- Working tree dirty.
- `master` and `origin/master` diverge after fetch/pull/push and the operator
  has not authorized the required corrective action.
- On `master` / `main` directly.
- Verdict ref reads `dont-ship` (`harness-kit-checks verdict check-landable` returns 2).
- No same-HEAD landability evidence exists: no green PR checks, no
  landable verdict, and no operator-provided/current-session local gate
  receipt.
- In a repo with `dagger.json`, `dagger call check --source=.` has not passed
  on the exact HEAD being landed.
- In GitHub mode, `gh pr view --json baseRefName` does not report `master`,
  or `gh pr checks` is red. Do not add a `--force` flag; refuse.
- If a PR exists, it is not mergeable per
  `gh pr view --json mergeable,mergeStateStatus`.
- No trace handoff inputs and no explicit `Trace-waiver: <reason>` line.
  Operator must provide a transcript/ref source or a waiver reason before
  merge.
- Closing IDs are present but their matching backlog files cannot be archived
  and the missing files are not already archived. A branch with no closing IDs
  is allowed; report `Closed: none`.
- Acceptance criteria, oracle artifacts, golden files, fixtures, Gherkin
  features, CLI transcripts, screenshots with asserted data, or assertion
  surfaces changed without an explicit `Contract-change acknowledgment:`
  explaining why the contract changed. Route back to `/deliver --polish-only`.
- Rebase / merge / cherry-pick in progress (`.git/MERGE_HEAD`,
  `.git/CHERRY_PICK_HEAD`, `rebase-*` dir).

## Trailer Conventions

Every ticket closure flows through git trailers. Keys recognized by
`harness-kit-checks backlog trailer-keys`:

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

## Interactions

- **Upstream:** `/deliver --polish-only` leaves the branch merge-ready.
  `/ship` assumes that work is done; it does not re-run CI, code-review, or
  refactor.
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
  the last block, so downstream `harness-kit-checks backlog ids-from-commit`
  returns empty.
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
            harness/reflect-outputs, retro in .harness-kit/reflect/<cycle>/
Residual:   none
Workspace:  clean
Remote:     master...origin/master 0 0
```

On refuse, emit the reason and the action the operator must take to
re-enable shipping.

## Verification

Run `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`
and `cargo run --locked -p harness-kit-checks -- check-evidence-blocks skills`;
semantic proof is the
actual merge/backlog/archive/reflect receipt for the shipped branch.
