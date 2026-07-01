# /deliver eval

The oracle for the `/deliver` skill. Tests the one claim an end-to-end delivery
skill must earn: **given one ticket or raw idea, `/deliver` produces a
merge-ready change with docs+tests+code in sync, live QA evidence (not just
"tests pass"), a three-altitude refactor pass, and diverse-provider review
resolved to zero blockers — that bare "implement this ticket" does not.**

This is a `mode-eval` A/B run, not a directory shape. Arms: A = `/deliver`
installed and invoked; B = raw same-model ("implement this ticket end to end
and make it mergeable", no skill); C = n/a (no credible alternative primitive
covers the full arc). Grade blind, objective checks first, judge a different
model family than the workers. Protocol: `skills/skill-eval/references/run-recipe.md`.

## Fixtures

Each fixture is a small, self-contained ticket landed in a scratch repo/branch
so the arm can actually build, test, and review it end to end.

| # | Prompt | Repo @ SHA | Forbidden edits | What it stresses |
|---|---|---|---|---|
| 1 | "Deliver: add a `--since <duration>` flag to `harness-kit-checks telemetry` that filters rows older than the duration (reuse the existing `--since` parser in `skill_invocation_analytics.rs`)." | `harness-kit@c6e01b9` (`crates/harness-kit-checks/src/{main.rs,skill_invocation_analytics.rs}`) | any unrelated crate | docs→tests→code discipline, live CLI QA, review loop on a real small feature |
| 2 | "Deliver: fix the bug where `check-godfiles` reports a false failure on a file that shrank below the baseline (confirm by writing a failing test first)." | `harness-kit@c6e01b9` (`crates/harness-kit-checks/src/quality_gates.rs`) | any file outside `quality_gates.rs` and its tests | TDD red→green→refactor, three-altitude refactor pass on a bug fix (not a feature) |
| 3 | "Deliver: the `check-eval-coverage` gate has no CLI flag to list which skills are covered vs missing without failing the process — add a `--report` mode." | seeded fixture repo with a stub gate (no harness-kit SHA) | any file outside the stub gate module | oracle-less ticket (no acceptance criteria given) — tests whether the arm writes one before building, per the skipping-shape gotcha |

Two of three must show A>B for a pass; the fixtures span a clean feature, a
regression fix requiring TDD, and an underspecified ticket that should trigger
the "no oracle, no delivery" gate.

## Objective checks (scriptable, pass/fail, ~free — run on every `skills/deliver/**` edit)

- [ ] A failing test existed before the fix/feature landed (visible in commit
      history: test commit precedes or is paired with the implementation commit).
- [ ] The change is on a feature branch, not committed directly to the default
      branch.
- [ ] Docs (README/doc-comment/CLI usage string) were updated in the same
      change as the behavior they describe.
- [ ] A live QA artifact exists: an actual command invocation and its output
      (not "tests pass" alone) — e.g. the new flag was run once against real
      data and the output is shown.
- [ ] Commit message(s) are semantic/conventional and explain why, not just what.
- [ ] Fixture 3 only: the arm produces or requests an oracle (acceptance
      criteria) before writing implementation code — does not build blind.
- [ ] No unrelated files touched (forbidden-edit list respected).
- [ ] Final state report names exact verification command/output, review
      findings and resolution, and residual risk — not a bare "done."

## Rubric (1–5, blind, one-line justification each — judgment-heavy delta only)

| Dimension | 5 | 1 |
|---|---|---|
| Verification depth | live command run + output shown, not just green test suite | "tests pass," no live evidence |
| Refactor discipline | diff, codebase, and backlog altitudes all visibly considered | code merged as first draft, no refactor pass |
| Review rigor | at least one blocking-severity finding surfaced and fixed, or an honest "none found" with reasoning | no review pass, or a rubber-stamp |
| Oracle discipline (fixture 3) | writes/derives an acceptance oracle before implementing | implements the first plausible interpretation with no stated oracle |
| Closeout completeness | clean tree, exact evidence cited, residual risk named | vague "should be good to go" |

## Pass condition

Arm A beats arm B on aggregate rubric **and** ties-or-wins every objective
check, across **≥2 of 3** fixtures. A no-op "deliver" (equivalent to raw
prompting) fails because the raw arm reliably stops at "code compiles, tests
pass" — it skips live QA evidence, the refactor sweep, and the review loop,
and on fixture 3 it typically builds without ever naming an oracle.

## Human anchor

The operator blind-grades ≥1 fixture (recommend fixture 2 — the TDD bug fix,
since "did they actually write the failing test first" is easy to verify and
easy for a rubric to fake). Record the verdict and match/mismatch with the
agent grader here once run. **PENDING — no run yet.**

## Cadence

- Edit-time: 1-fixture native-subagent smoke (fixture 1) on any
  `skills/deliver/**` change.
- Contract change (the docs→tests→code loop, the review-loop rule, or the
  closeout bar moves): full A/B, all 3 fixtures, decorrelated families.
- Major model release: re-audit — a stronger bare model may already default to
  TDD and live QA, closing `/deliver`'s edge.

## Run log

**No run yet.** Spec seeded 2026-07-01 under backlog.d/128 (EVALS-PER-SKILL);
`/deliver` is the highest-usage first-party skill (36 recorded invocations per
the 2026-07-01 groom telemetry read) and had no eval coverage before this. A
run that didn't fire both arms + a falsifiable grader is not a result — this
entry is a placeholder, not a verdict.
