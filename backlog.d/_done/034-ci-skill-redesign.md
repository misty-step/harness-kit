# `/ci` — audit + run Dagger gates

Priority: high
Status: pending
Estimate: M (~2 dev-days)

## Goal

One skill that audits a repo's CI, strengthens it where weak, then runs it
green. The skill is responsible for *confidence in correctness* — lint,
types, tests, coverage — and never returns until gates pass or fails loudly
with a diagnosis.

This absorbs CI responsibilities currently scattered across `/settle`
(polish-for-merge) and ad-hoc `dagger call check` invocations.

## Why New Skill vs Grow `/settle`

`/settle` blends three concerns: CI, review-comment-addressing, and
merge-coordination. That's not one skill — the merge-coordination part is
either a human concern or moves into `/autopilot` outer loop. Splitting
out `/ci` leaves `/settle` with an unclear remainder; evaluate
deprecating `/settle` as part of this work (see Phase 3).

## Contract

**Input:** A feature branch or working tree. No args required.

**Output:**
- CI passes green end-to-end
- If CI was weak or miscovered (no type check, no coverage gate, stale
  Dagger pipeline), audit report proposes strengthening, applies if
  user approves
- Exit non-zero on unfixable failures, with structured diagnosis (which
  gate failed, which test, why)

**Stops at:** green CI. Does not review code semantics (→ `/code-review`),
does not ship.

## Stance

1. **CI is load-bearing.** A green CI is a claim about code correctness.
   If CI is weak (no types, shallow tests, missing coverage), green means
   nothing. The skill audits *first*, then runs.
2. **Dagger-first.** Run CI via Dagger containers, not raw local shells.
   Hermetic, reproducible, matches what CI runners execute.
3. **Fix-until-green.** Don't report red and exit. Either fix the failure
   or produce a precise diagnosis the human can act on.
4. **No quality lowering.** Never weaken a threshold to make CI pass.
   Load-bearing thresholds (lint rules, type strictness, coverage floor)
   are walls.

## Composition

```
/ci [--audit-only | --run-only]
    │
    ▼
  1. Audit phase (unless --run-only)
     ├── Inventory: Dagger pipelines, lint config, type config, test config
     ├── Check: every concern covered? (lint, type, test, coverage, secrets)
     ├── Report: gaps
     └── Optionally apply strengthening (user-gated, diff preview)
    │
    ▼
  2. Run phase (unless --audit-only)
     ├── `dagger call check` (or equivalent pipeline entrypoint)
     ├── Capture structured output per gate
     ├── On failure: attempt fix (lint auto-fix, type errors, flaky test retry)
     └── On still-failing: exit non-zero with diagnosis
    │
    ▼
  Green → exit 0
  Red → exit non-zero with structured diagnosis
```

## Audit Checks

- Is there a Dagger pipeline? (`.dagger/` or `dagger.json`)
- Does it run lint + type + test + coverage? Any gap?
- Is coverage gated with a floor? Is the floor reasonable (>70%)?
- Are tests hermetic (no network, no external DBs)?
- Are secrets scanned (gitleaks / trufflehog)?
- Is the pipeline fast enough (<10 min)? If not, flag.
- Does the pre-push hook invoke the Dagger pipeline?

Each gap = audit finding. User can approve remediation or defer.

## What `/ci` Does NOT Do

- Code review (→ `/code-review`)
- Shape tickets (→ `/shape`)
- Merge PRs (→ human)
- Deploy (→ `/deploy`)
- QA against running app (→ `/qa`)
- Refactor for style (→ `/refactor`)

## `/settle` Deprecation Path

After `/ci` lands, `/settle`'s remaining responsibilities are:
- Address code review comments → move to `/code-review` or `/deliver`
- Coordinate merge → human, or `/autopilot` outer loop

Evaluate deprecating `/settle` entirely. If any residual remains, scope it
down and document. Track in a follow-up ticket after `/ci` lands.

## Oracle

- [ ] `skills/ci/SKILL.md` exists
- [ ] Runs on harness-kit itself: audit identifies any current CI gaps, runs Dagger pipeline green
- [ ] Audit output is structured (markdown table of gaps + severity + proposed fix)
- [ ] Fix-until-green loop: demonstrable on a branch with a fixable lint error
- [ ] Non-fixable failure: exit non-zero with diagnosis that points to file:line
- [ ] `/settle` SKILL.md updated or slated for removal in follow-up

## Non-Goals

- Writing new test frameworks — uses what repo has
- Replacing Dagger — uses Dagger as the pipeline runner
- Judging code quality beyond automated gates (→ `/code-review`)
- Multi-language CI beyond repo's stack

## Related

- Blocks: 032 (`/deliver` needs `/ci` as a composition step)
- Supersedes: parts of `/settle` (evaluate full deprecation)
- Related: 025 (dagger merge gate — `/ci` becomes the canonical runner)
