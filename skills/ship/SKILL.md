---
name: ship
description: |
  Final mile from current committed work to shipped: get the work onto master,
  archive backlog tickets with trailers, update touched docs, run /reflect,
  apply outputs.
  Recovers detached HEAD or headless work by creating a shipping ref before
  squash-merging to master.
  Use when: "ship it", "merge and close out", "final mile", "land and
  reflect", "finish this ticket".
  Trigger: /ship.
argument-hint: "[branch-or-pr]"
---

# /ship

The final mile. `/ship` gets the current committed work onto `master`, archives
the ticket(s), syncs docs, runs `/reflect`, and threads reflect's outputs back
into the repo. One command from "green" to "shipped and learned from."

## Stance

1. **Act, do not propose.** `/ship` has authority within its domain.
   Archive, merge, pull, reflect, apply. Escalate only on refuse conditions.
2. **Never lose trailer context.** `Closes-backlog` trailers must survive
   into the squash commit on master. `/groom` sweeps master by trailer —
   a dropped trailer is a ticket that never closes.
3. **Pre-merge prep belongs on the shipping ref.** Archive moves and doc syncs
   go on the feature branch, temporary ship branch, or other normalized
   shipping ref before the squash so the merge commit itself carries a single,
   clean closure event.
4. **Ship always means get it onto master.** `/ship` always lands the current
   committed work by creating one squash commit on `master`, unless the work is
   already on `master`, in which case it verifies, pushes, reflects, and
   reports. No normal merge, no rebase-merge, no fast-forward-only landing, no
   destination inference from `origin/HEAD`, and no `main` fallback. GitHub mode
   may use GitHub's squash-merge operation, but the destination remains
   `master` and trailers must be passed explicitly.
5. **Reflect's harness edits never touch master.** They land on a
   `harness/reflect-outputs` branch for human review. This is a hard
   invariant from `reflect/SKILL.md`.
6. **Final-mile runner, not a refactorer.** `/ship` may run the repo's
   documented gate when same-HEAD landability evidence is missing. It does not
   redesign, refactor, or lower gates to pass.
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

- Current `HEAD` identifies the committed work to ship. It may be a feature
  branch, detached `HEAD`, or already on `master`.
- If `HEAD` is detached, create a temporary local branch before archive/doc
  prep:

  ```sh
  short="$(git rev-parse --short HEAD)"
  stamp="$(date -u +%Y%m%dT%H%M%SZ)"
  git switch -c "ship/${stamp}-${short}"
  ```

  Use that branch as the **shipping ref** in every later step and report it.
- If already on `master`, do not manufacture a squash merge. Verify the
  committed work is on `master`, push/sync it, then continue with trace and
  reflect. Refuse only when `master` has uncommitted work, failed gates, or
  remote divergence the operator has not authorized.
- `master` exists locally or can be fetched from `origin/master`; `master` is
  the only landing destination.
- If the shipping ref name matches `^(feat|fix|chore|refactor|docs|test|perf)/([0-9]+)-`,
  the numeric capture is the **primary backlog ID** being shipped. If it does
  not match, `/ship` continues with no primary backlog ID and closes only IDs
  found in trailers or explicit archive evidence.
- Working tree clean. See `harnesses/shared/AGENTS.md` (Closeout).
- If a PR exists for the branch: `gh pr view --json baseRefName,mergeable,mergeStateStatus`
  reports `baseRefName` as `master` and mergeable. A non-master PR base,
  conflicted PR, or blocked PR means `/deliver --polish-only` isn't done.
- Landability evidence exists for this exact HEAD or `/ship` can produce it by
  running the repo's documented gate before merge. Acceptable evidence:
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
- Trace handoff inputs exist or `/ship` records an explicit waiver: transcript refs, review
  receipt refs, QA/demo refs, or another durable trace source are available for
  `/trace` after merge. If no transcript or trace artifact is available, use a
  `Trace-waiver:` line in the trace record and final report. Raw session logs do
  not satisfy this prerequisite.
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

### 0. Normalize Shipping Ref

Capture:

```sh
PRE_SHIP_HEAD="$(git rev-parse HEAD)"
CURRENT_REF="$(git rev-parse --abbrev-ref HEAD)"
```

If `CURRENT_REF` is `HEAD`, create the temporary `ship/<timestamp>-<shortsha>`
branch described in prerequisites. If on `master`, set mode to `already_on_master`.
Otherwise use the current branch as the shipping ref.

From this point on, every range that used `<branch>` means the normalized
shipping ref. Never lose `PRE_SHIP_HEAD`.

### 1. Extract backlog IDs

Primary ID is optional. If the branch name matches
`^(feat|fix|chore|refactor|docs|test|perf)/([0-9]+)-`, use the numeric capture.
Then scan branch commits:

```sh
git log --format=%B master.."$PRE_SHIP_HEAD" \
  | git interpret-trailers --parse --no-divider
```

Collect every `Closes-backlog:` and `Ships-backlog:` value (closing) plus
every `Refs-backlog:` value (reference-only). Merge with the primary ID when
one exists.

- **Closing set:** optional primary ID ∪ Closes-backlog ∪ Ships-backlog.
- **Reference set:** Refs-backlog values. Noted in the final report, never
  archived.

Prefer `cargo run --locked -p harness-kit-checks -- backlog ids-from-range master.."$PRE_SHIP_HEAD"`
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
git diff master.."$PRE_SHIP_HEAD" --name-only
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

### 4. Create the archive commit on the shipping ref

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

If mode is `already_on_master`, skip the squash step, run the documented gate
on `master`, push/sync `master`, and continue at "Pull master and verify
trailers." Report `Mode: already_on_master`.

The only acceptable landing result is one new squash commit on `master`.
Never use plain `git merge`, `git merge --ff-only`, `git rebase` as the landing
operation, or any destination other than `master`.

**GitHub mode** (PR exists, `gh` available):

Construct a squash body that carries every closing trailer. GitHub's
default squash template often drops commit trailers, so pass the body
explicitly:

```sh
body="$(git log --format=%B master.."$PRE_SHIP_HEAD" \
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
git merge --squash <shipping-ref>
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

- Shipping ref name (pre-merge, including generated `ship/...` refs).
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
- On `main` or another non-`master` protected/default branch directly.
- On `master` directly with uncommitted work, failed documented gate, or
  unresolved `master...origin/master` divergence.
- Verdict ref reads `dont-ship` (`harness-kit-checks verdict check-landable` returns 2).
- Same-HEAD landability cannot be produced: no green PR checks, no landable
  verdict, no operator-provided/current-session local gate receipt, and the
  documented gate cannot be run or fails.
- In a repo with `dagger.json`, `dagger call check --source=.` has not passed
  on the exact HEAD being landed.
- In GitHub mode, `gh pr view --json baseRefName` does not report `master`,
  or `gh pr checks` is red. Do not add a `--force` flag; refuse.
- If a PR exists, it is not mergeable per
  `gh pr view --json mergeable,mergeStateStatus`.
- Trace handoff cannot be recorded and no explicit `Trace-waiver:` can be
  included in the trace/final report.
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

- **Upstream:** `/deliver --polish-only` usually leaves the branch merge-ready.
  `/ship` can also normalize detached/headless committed work and produce the
  documented gate receipt itself. It still does not refactor or lower gates.
- **Invokes:** `/reflect cycle` for retro, backlog mutations, and
  harness proposals.
- **Invoked by:** `/flywheel` as the landing + reflection stage of each
  cycle. `/flywheel` reads `/ship`'s final report to decide the next
  cycle.
- **Complements `/yeet`:** `/yeet` pushes the working tree; `/ship` lands the
  branch on master, archives, and reflects.

## Gotchas

- GitHub squash bodies can drop trailers; always pass the trailer block
  explicitly with `--body`.
- Keep all trailers in one contiguous final block. Inject with
  `git interpret-trailers --if-exists addIfDifferent --trailer ...`.
- Archive before merge so closure is part of the squash, not a follow-up
  master commit.
- Primary IDs without backlog files are valid trailer-only work; note missing
  files instead of failing.
- Reflect harness edits never mutate master; use `harness/reflect-outputs`.
- Re-running on an already-shipped branch should exit early.
- Deduplicate repeated closing trailers through `interpret-trailers`.

## Output

Single plain-text report: merged SHA/PR, closed and referenced IDs, docs,
trace, reflect outputs, harness branch, accepted evidence, residual risk,
workspace status, remote sync, and roster delegation summary. On refuse, emit
the blocking reason and the exact action that re-enables shipping.

## Verification

Run `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`
and `cargo run --locked -p harness-kit-checks -- check-evidence-blocks skills`;
semantic proof is the
actual merge/backlog/archive/reflect receipt for the shipped branch.
