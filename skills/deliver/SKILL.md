---
name: deliver
description: |
  Inner-loop composer for one backlog item to merge-ready code. Composes
  /shape, /implement, /code-review, /ci, /refactor, and /qa; stops before
  push, merge, or deploy. Emits receipt.json plus operator brief + /reflect.
  Use for "deliver this", "make it merge-ready", shaped-ticket builds, and
  `--polish-only <branch|PR>` for existing branch/PR cleanup.
  Trigger: /deliver.
argument-hint: "[backlog-item|issue-id] [--polish-only <branch|PR>] [--resume <ulid>] [--abandon <ulid>] [--state-dir <path>]"
---

# /deliver

Inner-loop composer. One backlog item → merge-ready code. **Delivered ≠
shipped.** The outer loop (`/flywheel`) consumes the receipt and
decides whether to deploy. Humans merge.

## Invariants

- Compose atomic phase skills. Never inline phase logic.
- Fail loud. Never swallow a phase failure into a "best effort" pass.
- Clean closeout is part of merge-readiness. Before writing `merge_ready` or
  presenting delivery as complete, shared Closeout applies; every visible path
  must be classified into a follow-up commit, deletion, move-out, durable
  ignore, or explicit handoff. See `harnesses/shared/AGENTS.md` (Closeout).

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Phase skills dispatch their own lanes; /deliver verifies roster-member receipts or explicit exceptions before calling work merge-ready.

## Completion Evidence

Completion evidence core applies: use `harnesses/shared/AGENTS.md`
(Completion Evidence) as the universal evidence shape, then fill the local
fields in the merge-ready block below.

## Work Ledger

When `.harness-kit/work/ledger.jsonl` is available, `/deliver` calls
`scripts/work-ledger.py append` at transition points: `phase_started` for
`shape`, `implement`, `review`, `ci`, `qa`, and `reflect`; `blocker_added`
when a phase fails; `phase_completed` with `status=completed` when the branch
is merge-ready. On `--resume`, consume the latest record for the same
backlog/branch/work id to report current phase, blockers, evidence refs,
spawned agents, trace refs, and next action without parsing chat history.

## Closeout Contract

Every `/deliver` run ends with two operator-facing outputs, in this order:
1. A tight delivery brief.
2. A full `/reflect` session.

This does not replace the machine contract. `receipt.json` remains the source
of truth for callers and automation. The brief and reflection are for the
human operator.

The delivery brief is short and punchy. It is not a file inventory, a raw
changelog, or a generic "green tests" note. Default shape: 1-2 short
paragraphs or 4-6 flat bullets.
When a public PR or issue needs agent provenance, use `/agent-transcript` to
render a redacted local excerpt and ask before publishing it. Never paste raw
session logs into the brief or receipt.

The delivery brief must answer:
- What ticket was worked and what changed.
- What roster lanes ran: providers dispatched and why, whether they ran in
  parallel or as competing worktree attempts, what each did well or poorly,
  what was accepted into the final synthesis, what failed or was rejected,
  and any waiver/exception. Ground this in
  `scripts/summarize-delegations.py --format text` plus receipt ids or
  evidence paths, not raw transcripts.
- What exact end-user behavior changed; for internal-only work, what
  developer/operator behavior changed.
- What value the ticket adds, and why making it merge-ready is useful and
  important now.
- What alternatives to the implemented design existed.
- Why the implemented design is best under the current constraints. If it is
  not clearly best, say so plainly and explain why it was still the right
  delivery choice.
- What value the change creates for developers and operators.
- What value the change creates for users or customers once it ships.
- What was verified, and what residual risk remains before merge or deploy.
- What hardening triggers were found by implementation, review, CI, or QA; if
  `/hardening` did not run, why the route was waived.

Every merge-ready brief includes this gate:

Completion evidence core applies: behavior changed or verified, live evidence,
exact command/path/route, repo-fit check, and residual risk. See
`harnesses/shared/AGENTS.md` (Completion Evidence).

Local fields include oracle hash, contract-change acknowledgment, and
hardening run / waiver.

```markdown
## Completion Gate
- Exact end-user behavior changed: behavior or internal operator behavior delivered by this branch.
- Evidence that proves it: test output, QA artifact, gate result, or receipt proving the behavior.
- Exact command/path/route exercised: command, URL, route, file path, or tool call actually run.
- Oracle / acceptance artifact hash: sha256 digest for any fixture, contract, golden file, transcript, screenshot, or equivalent acceptance source used by the oracle.
- Contract-change acknowledgment: explicit reason when the branch changes acceptance criteria or weakens an assertion surface.
- Repo-fit check: live repo pattern, contract, or boundary this branch follows.
- Hardening run / waiver: hardening mode run, blocking recommendation, or waiver reason.
- Residual risk: unverified path, accepted survivor, or none with reason.
```

If any phase cannot fill the block with live evidence, `/deliver` is not
merge-ready. "Gate passed" is necessary evidence, not the whole acceptance
argument. Hardening recommendations are recorded in the receipt; they extend
the clean loop only when a phase verdict makes the test-strength gap blocking.

`/reflect` remains mandatory. Do not collapse reflection into the delivery
brief. The brief explains the delivered result; `/reflect` captures the
learnings, harness changes, and follow-on mutations.

When `/deliver` is invoked under `/flywheel`, keep the same content shape but
let the outer loop own the final session-level shipping brief.

## Composition

```
/deliver [backlog-item|issue-id] [--resume <ulid>] [--state-dir <path>]
    │
    ▼
  pick (if no arg) — backlog.d/ highest-priority
    │
    ▼
  /shape            → context packet (goal + oracle + sequence)
    │
    ▼
  /implement        → TDD build on feature branch
    │
    ▼
┌── CLEAN LOOP (max 3 iterations) ──────────────┐
│  /code-review     → critic + bench             │
│  /ci              → dagger audit + run         │
│  /refactor        → diff-aware simplify        │
│  /design + /a11y  → only when UI surfaces exist│
│  /qa              → running-surface evidence   │
│  capture evidence → see references/evidence.md │
└──────────────────────────────────────────────┘
    │ all green → merge-ready (exit 0)
    │ cap hit or hard fail → fail loud (exit 20/10)
    ▼
  receipt.json written; stop. No push, no merge, no deploy.
```

## Phase Routing

| Phase | Skill | What it owns | Skip when |
|---|---|---|---|
| shape | `/shape` | context packet, oracle, sequence | packet already has executable oracle |
| implement | `/implement` | TDD red→green→refactor + per-chunk milestone critic gate (AGENTS.md L2), commits on feature branch | — |
| review | `/code-review` | parallel bench review, synthesized findings | — |
| ci | `/ci` | dagger audit + green pipeline | `/ci` itself decides — do not pre-filter |
| hardening | `/hardening` | property, mutation, acceptance, CRAP/SCRAP, or DRY hardening evidence | no phase issued a blocking hardening requirement, or an explicit waiver is recorded |
| refactor | `/refactor` | diff-aware simplification | trivial diffs (<20 LOC, single file) |
| design | `/design` + `/a11y` | visual intent, taste, and accessibility evidence | no UI paths by detector or equivalent diff inspection |
| qa | `/qa` | browser-driven exploratory test, evidence | no user-facing surface (pure library/refactor) |

Each skill has its own contract and receipt. `/deliver` reads those
receipts; it never re-implements the phase.

## Polish-Only Mode

`/deliver --polish-only <branch|PR>` is the single owner of "existing branch →
merge-ready." It is the absorbed former `/settle` (collapsed per backlog 080).
Use it to pick up a branch or PR that already has code and drive it green —
no fresh ticket, no `/shape`, no `/implement`.

- **Entry, not a second loop.** Polish-only resolves and validates the target
  (feature branch with commits beyond base; clean tree; no rebase/merge in
  progress), **skips `/shape` + `/implement`**, and enters the *same* clean loop
  with the *same* `receipt.json` contract, exit codes, and 3-iteration cap.
  It never re-implements a phase — the hard invariant still holds.
- **PR mode.** When the target is a PR number (or `gh pr view` succeeds), the
  clean loop ingests full PR review bodies via
  `scripts/fetch-pr-reviews.sh` and remote check state via `gh pr checks`
  before `/code-review`, and dispositions every comment (fix / defer to
  `backlog.d/` / reject-after-steelman, one at a time). Full protocol:
  `references/polish-only.md` + `references/pr-fix.md`.
- **Same closeout.** Unlike the old `/settle`, polish-only ends with the full
  `/deliver` closeout: brief + `/reflect`. This is intentional — one
  merge-readiness contract, deliberately heavier than the old settle loop.
- **Aliases.** `/pr-fix` and `/pr-polish` route here (via the `/settle`
  redirect during the deprecation window).

See `references/polish-only.md` for the entry protocol, PR-mode detail, and
settle-parity checks (hindsight sanity pass, verdict-ref freshness).

## Cross-Cutting Invariants

- **No claims.** Dropped per operating principle. Single local workspace.
  Concurrent worktrees coordinate via state-dir isolation (see
  `references/worktree.md`).
- **Never re-deliver stale backlog.** If the target item already carries
  `## What Was Built` or current-branch history contains an explicit closure
  marker like `Closes backlog:<item-id>` / `Ships backlog:<item-id>`, stop
  and route to `/groom tidy`. That is backlog drift, not fresh delivery work.
- **Never push.** Delivery ≠ shipping. `git push` is the outer loop's call.
- **Never merge.** `gh pr merge` is a human decision.
- **Never deploy.** `/deploy` is the outer loop's concern.
- **Never commit to default.** Feature branch only; see `references/branch.md`.
- **Fail loud.** A dirty phase is a dirty phase — do not mask it, do not
  retry past the cap, do not write `status: merge_ready` when anything is
  red.
- **Evidence is out-of-band.** `/deliver` writes zero artifacts itself;
  per-phase skills emit; receipt records pointers only. See
  `references/evidence.md`.
- **No auto-invoke of `/hardening`.** Downstream phases may flag hardening
  opportunities; `/deliver` records them and any waiver. `/hardening` runs only
  when explicitly included in the delivery path or when a phase issues a
  blocking verdict naming the test-strength gap.

## Contract (exit code + receipt)

`/deliver` communicates exclusively via its exit code and
`<state-dir>/receipt.json`. Callers — human or `/flywheel` outer loop —
do not parse stdout.

| Exit | Meaning | Receipt `status` |
|---|---|---|
| 0 | merge-ready | `merge_ready` |
| 10 | phase handler hard-failed (tool/infra error) | `phase_failed` |
| 20 | clean loop exhausted (3 iterations, still dirty) | `clean_loop_exhausted` |
| 30 | user/SIGINT abort | `aborted` |
| 40 | invalid args / missing dep skill | `phase_failed` |
| 41 | double-invoke on an already-delivered item | `phase_failed` |

Full receipt schema + state lifecycle: `references/receipt.md`.

## Resume & Durability

State is filesystem-backed and resumable.

- **State root:** `<worktree-root>/.harness-kit/deliver/<ulid>/` (gitignored).
  Override via `--state-dir <path>` (the outer loop uses this to land state
  under its cycle's evidence tree).
- **Checkpoint:** after each phase, `state.json` rewritten atomically
  (write → fsync → rename).
- **`--resume <ulid>`:** loads `state.json`, skips completed phases,
  re-enters at `current_phase`. Phase handlers must be idempotent.
- **`--abandon <ulid>`:** removes state-dir; leaves branch as-is.
- **Double-invoke:** `/deliver <already-delivered-item>` → exit 41, not
  silent re-run.

Full protocol: `references/durability.md`.

## Gotchas (judgment, not procedure)

- **Retry vs escalate.** Dirty on iteration 1 → retry (normal). Dirty on
  iteration 3 → exit 20, write receipt, hand to human. Do not invent a 4th
  iteration. The cap is load-bearing: loops without one produce slop.
- **What counts as "dirty".** `/code-review` blocking verdict, `/ci`
  non-zero, `/qa` P0/P1. P2 QA findings are documented in the receipt and
  do NOT block. Review "nit" and "consider" are not blocking.
- **Inlining a missing phase.** `/implement` missing → exit 40. Do NOT
  fall back to your own TDD build — inlined fallbacks become permanent.
- **Silent push.** A phase skill that "helpfully" runs `git push` is a bug
  in that phase skill. Surface it; do not suppress it in the composer.
- **Re-shaping mid-delivery.** If `/implement` or `/qa` reveals the shape
  is wrong, stop the clean loop and exit with remaining_work pointing at
  re-shape. Do not spin.
- **Skipping shape.** Building without a context packet yields plausible
  garbage. If the item has no oracle, `/shape` runs first. Always.
- **Review without verdict = dirty.** If `/code-review` runs but no `refs/verdicts/<branch>` points at HEAD afterward, treat the review phase as failed.
- **Merging.** Never. End-state is merge-ready, not merged.
- **Stale active item.** An item can be "open" in `backlog.d/` and still be
  already shipped in git history because a human landed it outside `/flywheel`.
  Refuse to treat that as new work; fix the backlog state first.

## References

- `references/clean-loop.md` — iteration cap, dirty-detection per phase,
  escalation protocol
- `references/receipt.md` — full JSON schema, exit-code table, state
  lifecycle
- `references/durability.md` — state.json atomic checkpoint protocol,
  `--resume` / `--abandon` semantics, double-invoke
- `references/evidence.md` — per-phase emission paths, gitignored
  `.harness-kit/deliver/` conventions
- `references/branch.md` — branch-naming, HEAD-detection, no-push rule
- `references/worktree.md` — state-root resolution, concurrent worktrees
- `references/polish-only.md` — `--polish-only` entry protocol, PR-mode
  detection, settle-parity checks (hindsight sanity pass, verdict-ref)
- `references/pr-fix.md` — PR comment triage + disposition (moved from
  `/settle`; used by polish-only PR mode)
- `references/pr-polish.md` — deep hindsight smell catalog + confidence
  assessment (moved from `/settle`; `/qa` + `/hardening` own test depth)

## Non-Goals

- Deploying — `/flywheel` outer loop's concern
- Merging — humans merge
- Multi-ticket operation — one ticket per invocation
- Claim-based coordination — explicitly dropped
- Version-controlled evidence — gitignored under `.harness-kit/`

## Related

- Consumer: `/flywheel` — outer loop passes `--state-dir` under its cycle tree and reads `receipt.json`
- Phases: `/shape`, `/implement`, `/code-review`, `/ci`, `/refactor`, `/qa`
