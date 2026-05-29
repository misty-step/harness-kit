# Collapse settle into deliver: one merge-readiness owner

Priority: P1
Status: done
Estimate: M
Shipped: 2026-05-29

## Resolution

Collapsed `/settle` into `/deliver --polish-only` — `/deliver` is now the
single owner of "existing branch → merge-ready." Net −181 LOC.

- New `--polish-only <branch|PR>` entry shim (`skills/deliver/SKILL.md` +
  `references/polish-only.md`): validates an existing branch/PR, skips
  shape+implement, enters the same clean loop / receipt / 3-iteration cap.
  Does not inline phase logic (deliver's hard invariant holds).
- Absorbed settle behaviors into the shared clean loop: **hindsight sanity
  pass** (renamed from "adversarial self-review", distinct from `/critique`) +
  verdict-ref freshness (`clean-loop.md`). PR-mode comment triage + full-fetch
  + `gh pr checks` live in `polish-only.md` (kept out of the generic loop).
- Moved settle's owned files into deliver: `references/pr-fix.md`,
  `references/pr-polish.md`, `scripts/fetch-pr-reviews.sh`; deleted the
  `simplify.md` moved-stub.
- `/settle`, `/pr-fix`, `/pr-polish` are now a deprecation redirect (one
  release). Rewired `/flywheel` (migration note only) + prose pointers in
  ci/code-review/deploy/design/implement/ship.

Lanes: codex (`a50b6837`) + grok-build (`7f6464da`) ran read-only
design+regression critique with strong convergence (entry-shim not 2nd loop;
cap stays 3; move pr files; redirect-then-delete is the right migration). A
fresh adversarial critic on the diff caught 3 blocking gaps — stale
`check-agent-roster` (settle still in workflow lists), stale `docs/site`, and
leftover `/settle` Phase 1/2 vocab in the moved docs — all fixed. dagger check
15/15.

## Goal

Resolve the clearest MECE violation in the harness: `/settle` and `/deliver`
both own the merge-readiness polish loop (`/ci` + `/code-review` + `/refactor`).
Collapse `/settle`'s polish loop into `/deliver` as a `--polish-only` mode so
`/deliver` is the single owner of "branch → merge-ready," whether starting from
a backlog item or picking up an existing dirty branch.

## Non-Goals

- Do NOT change what merge-ready means or lower any gate.
- Do NOT make `/deliver` merge, push, or deploy — those boundaries hold.
- Do NOT delete `/settle` immediately. Ship a redirect for one release first
  (no silent breakage for muscle-memory `/settle`, `/pr-fix`, `/pr-polish`).
- Do NOT re-open the deliver-vs-flywheel inner-loop question here. Honor it as a
  constraint (below); a separate ticket can reconcile flywheel's composition.

## Constraints / Invariants

- `/deliver` composes atomic phase skills; it must not inline polish logic. The
  `--polish-only` path reuses the existing clean-loop phases, just entered on an
  existing branch instead of after `/implement`.
- Aliases `/pr-fix` and `/pr-polish` must keep working (route to
  `/deliver --polish-only`) — they are the operator's PR-fix ergonomics.
- The clean-loop iteration cap (3) and dirty-detection semantics are unchanged.
- **Flywheel constraint:** `/flywheel` currently composes `settle` directly
  (`skills/flywheel/SKILL.md`). After collapse, flywheel's polish step must call
  `/deliver --polish-only` (or the thinned phase), not the deleted `/settle`.

## Authority Order
tests > type system > code > docs > lore

## Repo Anchors
- `skills/deliver/SKILL.md:105-148` — the clean loop to make reusable as
  `--polish-only` (enter on existing branch, skip shape+implement).
- `skills/settle/SKILL.md:62-192` — the polish loop being absorbed; check for
  anything deliver lacks (hindsight pass, PR-mode, design/a11y routing).
- `skills/flywheel/SKILL.md` — composes `settle`; rewire to `--polish-only`.
- `skills/deliver/references/clean-loop.md` — iteration cap + dirty-detection.

## Alternatives Considered
| Option | Shape | Strength | Failure mode | Verdict |
|---|---|---|---|---|
| `/deliver --polish-only` + redirect (this) | One owner; settle redirects 1 release then deleted | Single merge-readiness owner; no MECE overlap; safe migration | Polymorphic deliver entry must cleanly distinguish fresh-ticket vs pick-up-branch | **choose** |
| Delete settle now | Hard cut | Fastest reduction | Breaks muscle memory + flywheel composition same release | reject |
| Keep both, dedupe loop only | Shared clean-loop reference | Less churn | Two entry points persist; MECE violation remains | reject |

## Oracle (Definition of Done)
- [ ] `/deliver --polish-only [branch|PR]` runs the clean loop on an existing
      branch (skips shape + implement), stops at merge-ready, writes the same
      `receipt.json` contract.
- [ ] Anything `/settle` did that `/deliver` did not (hindsight/fresh-eyes pass,
      PR-mode, design+a11y routing on UI diffs) is present in the `--polish-only`
      path. Step-6 "adversarial self-review hindsight" is renamed "hindsight
      sanity pass" to distinguish from `/critique` lens dispatch.
- [ ] `skills/settle/SKILL.md` becomes a redirect: invoking `/settle`,
      `/pr-fix`, or `/pr-polish` prints the `/deliver --polish-only` mapping and
      routes there. Marked for deletion next release in the file.
- [ ] `skills/flywheel/SKILL.md` composes `/deliver --polish-only`, not
      `/settle`; `grep -rn "/settle" skills/flywheel/SKILL.md` shows only the
      migration note, no live composition.
- [ ] `scripts/generate-index.sh` green; no dangling `/settle` references in
      other skills.
- [ ] `dagger call check --source=.` passes; existing `/deliver` mock runs and
      receipt-schema tests unaffected.

## Implementation Sequence
1. Extract the clean loop into a `--polish-only` entry on `/deliver` (enter on
   branch, skip shape/implement); reuse `references/clean-loop.md`.
2. Port any settle-only behavior (hindsight pass, PR-mode, UI routing) into the
   `--polish-only` path; rename the hindsight step.
3. Replace `skills/settle/SKILL.md` with a redirect; keep `/pr-fix`,
   `/pr-polish` aliases pointing at `/deliver --polish-only`.
4. Rewire `/flywheel` polish step.
5. `generate-index.sh`, `dagger call check`, deliver mock run.

## Risk + Rollout
- **Polymorphic deliver confusion** (fresh ticket vs pick-up branch). Mitigation:
  `--polish-only` is an explicit flag; default `/deliver` behavior unchanged.
- **Lost settle nuance.** Mitigation: the oracle's explicit settle-parity check
  before the redirect lands.
- Rollback: restore `skills/settle/SKILL.md` from the redirect commit; revert
  the flywheel rewire. One revert.

## Delegation Evidence
- All four lanes (codex, agy, grok-build + lead) named settle/deliver as the #1
  MECE violation and the #1 reduction. codex: keep `/pr-fix`,`/pr-polish` as
  redirects. Accepted.

## Related
- Part of the MECE/DRY consolidation line with `051` (delegation-floor DRY) and
  `076` (trigger-collision lint).
- Reduces composer count under the 30-item backlog cap.
