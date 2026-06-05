# Polish-Only Mode

`/deliver --polish-only <branch|PR>` drives an **existing** branch or PR to
merge-ready. It is the single owner of "branch → merge-ready," whether the
branch came from `/implement` or from a human. It absorbs the former `/settle`
(backlog 080). It is an *entry shim*, not a second loop: it validates the
target, skips `/shape` + `/implement`, and enters the standard clean loop
(`references/clean-loop.md`) with the same `receipt.json`, exit codes, and
3-iteration cap.

## Preconditions (refuse, don't loop)

Assert at start; refuse with a clear reason on any miss:

- On a feature branch (not `master`/`main`/default protected branch).
- Branch has commits beyond base (`git log base..HEAD` non-empty) — nothing
  to polish otherwise.
- No rebase/merge/cherry-pick in progress (`.git/MERGE_HEAD`,
  `.git/CHERRY_PICK_HEAD`, `rebase-*`).
- No unresolved conflict markers.

A working tree dirty with debris the operator hasn't acknowledged is a refuse,
not a "stage everything" — polish-only never commits random debris.

## Mode detection

- **PR mode** — `$ARGUMENTS` is a PR number, or `gh pr view` succeeds for the
  branch. Findings come from `/ci` + `gh pr checks` + full PR review bodies.
- **Local mode** — no PR. Findings come from local `/ci` + `/code-review`.

Mode changes only *where findings come from*, never *what the loop does*.
Both modes read and write QA/demo/review artifacts under
`.evidence/<branch>/<date>/`; GitHub release assets are optional PR mirrors,
not the source of truth.

## PR mode: review ingestion before `/code-review`

Do this in the loop's review step, before synthesizing a verdict:

1. **Read every comment in full.** Run
   `cargo run --locked -p harness-kit-checks -- fetch-pr-reviews <PR>` — it deterministically
   fetches all review bodies, inline comments, and conversation. Never preview
   with truncated `gh api`/jq/`head`. Automated reviewers (Gemini, Codex,
   CodeRabbit) get the same rigor — their suggestion blocks hide in truncated
   views.
2. **Check remote checks.** `gh pr checks` — a pending check is not a passing
   check. Do not declare green while checks are non-terminal.
3. **Disposition each comment** per `references/pr-fix.md`: fix (in scope) /
   defer (out of scope → `backlog.d/`) / reject (only after steelman, with
   specifics). One at a time — fix → commit → reply → next. Reviewer
   disposition is lead judgment, not a subagent's.

Fixes route through the composed phases (builder subagents, `/code-review`,
`/refactor`) — polish-only never inlines phase logic.

## Settle-parity checklist

These former-`/settle` behaviors now live in the shared clean loop, so
polish-only inherits them automatically:

- **Hindsight sanity pass** — `clean-loop.md` step 6 (the renamed adversarial
  self-review; distinct from a `/critique` lens dispatch).
- **Verdict-ref freshness** — `clean-loop.md` step 7 (when `harness-kit-checks verdict`
  exists).
- **Design/a11y routing** on UI diffs — `clean-loop.md` step 4.
- **Deep hindsight reference** — `references/pr-polish.md` (architectural smell
  catalog + confidence assessment). Test depth/coverage is `/qa` + `/hardening`,
  not duplicated here.

## Semantic change vs old `/settle`

Polish-only ends with the **full `/deliver` closeout: brief + `/reflect`**. The
old `/settle` stopped at merge-ready and left reflection to `/ship`. This is
intentional — one merge-readiness contract, deliberately heavier. `/pr-fix`
and `/pr-polish` callers get the same receipt + brief + reflect every run.

## Cap

3 iterations, same as default deliver. Settle's old 6-pass cap is retired — a
second merge-readiness definition would defeat the collapse. PR-comment churn
is handled by disposition triage, not extra loop passes.
