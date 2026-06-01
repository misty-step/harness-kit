---
name: code-review
description: |
  Parallel multi-agent code review. Launch reviewer team, synthesize findings,
  auto-fix blocking issues, loop until clean.
  Use when: "review this", "code review", "is this ready to ship",
  "check this code", "review my changes".
  Trigger: /code-review, /review.
argument-hint: "[branch|diff|files]"
---

# /code-review

Multi-provider, multi-harness code review. You are the marshal — read the diff,
select reviewers, craft prompts, dispatch everything in parallel, synthesize
results, fix blockers, loop until clean.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use independent adversarial review lanes calibrated to what would embarrass us in production; give reviewers the diff, acceptance criteria, and risk lens, not author reasoning.

## Marshal Protocol

1. **Read the diff.** `git diff $BASE...HEAD` (default base: `main` or `master`).
   Classify: what changed? (API, UI, tests, infra, security, perf, data model, etc.)

2. **Select internal reviewers (static map).** Do NOT hand-pick. Run the
   selection algorithm in `references/bench-map.yaml`:
   - `git diff --name-only $BASE...HEAD` → changed files
   - Start from `default`; union `add` agents for every rule whose glob
     matches any changed file; de-dupe; cap at 5 (critic pinned)
   - Bench size always in [3, 5]; selection is deterministic per diff
   Read `references/bench-map.md` for the full algorithm and
   `references/internal-bench.md` for each agent's lens. Then craft a
   tailored prompt per selected reviewer. Load
   `references/deep-review-lens.md` when the diff needs root-cause,
   provenance, or long-running autoreview discipline.

3. **Dispatch all three tiers in parallel:**

   | Tier | What | How |
   |------|------|-----|
   | Internal bench | 3-5 Explore sub-agents with philosophy lenses | Agent tool, tailored prompts |
   | Thinktank review | 10 agents, 8 model providers | `thinktank review` CLI. See `references/thinktank-review.md` |
   | Cross-harness | Two or more available non-lead providers from `.harness-kit/agents.yaml` | See `references/cross-harness.md` |

   Thinktank-specific rule: wait for the process to exit, or for
   `trace/summary.json` to reach `complete` or `degraded` with a
   `run_completed` event in `trace/events.jsonl`, before you consume the run.
   Mid-run output directories are not final artifacts.

4. **Synthesize.** Collect all outputs. Deduplicate findings across tiers.
   Rank by severity: blocking (correctness, security) > important (architecture,
   testing) > advisory (style, naming).

5. **Verdict.** If no blocking findings → **Ship**. If blocking findings exist →
   fix loop (below).

## Fix Loop

For each blocking finding, spawn a **builder** sub-agent with the specific
file:line and fix instruction. Builders fix strategically (Ousterhout) and
simply (grug). Builder fixes, runs tests, commits.

After all fixes land, **re-dispatch all three review tiers.** Full re-review,
not a spot-check. Loop until no blocking findings remain. Max 3 iterations —
escalate to human if still blocked.

## Live Verification

**Trigger:** the diff touches user-facing patterns — `.tsx`, `.jsx`, `pages/`,
`app/`, `routes/`, `api/`, `endpoints/`, or component directories.

**Rule:** at least one reviewer must exercise the affected routes/components.
**Ship** verdict is blocked until live verification passes.

**Skip:** pure refactors, config-only, test-only, backend-only with no
user-facing surface.

## Executable Path Verification

**Trigger:** the diff adds or materially changes an executable path —
script entrypoint, CLI, `package.json` / `make` target, Dagger function,
migration, runner, responder, or job entrypoint.

**Rule:** the marshal or a reviewer must either:

- run the exact entrypoint once
- or cite a gate or artifact that invokes that exact path

Helper tests, fixture-only critics, and adjacent lanes do not count.

**Verdict:** if the path was not exercised, label it an
`unverified runtime path`. That is a blocking finding for a **Ship**
verdict unless the diff is explicitly framed as scaffolding only.

## Plausible-but-Wrong Patterns

LLMs optimize for plausibility, not correctness. Reviewers must hunt for:
- Wrong algorithm complexity (O(n²) where O(log n) is needed)
- Unnecessary abstractions (82K lines vs 1-line solution)
- Stub implementations that pass tests but don't actually work
- Adjacent green lanes that never invoked the new entrypoint
- "Specification-shaped" code — right module names, wrong behavior
- Missing invariant checks that only matter at scale

## Simplification Pass

After review passes, if diff > 200 LOC net:
- Look for code that can be deleted
- Collapse unnecessary abstractions
- Simplify complex conditionals
- Remove compatibility shims with no real users

## Thermo / Deslop Lens

For meaningful implementation diffs, apply a harsh maintainability lens before
issuing a Ship verdict:

- structural simplification beats local polish; look for a code-judo move that
  deletes branches, helpers, modes, or layers while preserving behavior
- new ad-hoc conditionals in busy flows are design findings, not style nits
- thin wrappers, casts, optionality churn, magic fallbacks, and bespoke helpers
  need to earn their keep
- files pushed toward giant-module territory require decomposition rationale
- AI slop is review debt: unnecessary comments, abnormal defensive
  try/catch, internal mocks, `any` casts, deep nesting, and codegen residue

Route behavior-preserving cleanup to `/refactor`; block Ship when the slop or
spaghetti makes the changed behavior harder to reason about.

## Hardening Lens

Recommend `/hardening` when the diff exposes a test-strength gap:

- `/hardening property` for broad input domains, parser/serializer laws,
  normalization, state transitions, totals, allocation, scheduling, sorting, or
  round trips;
- `/hardening mutation` when branch-heavy changed code is covered only by
  shallow examples or assertion-light tests;
- `/hardening acceptance` when user-facing fixtures, Gherkin, API examples,
  CLI transcripts, or golden paths should fail if important values change;
- `/hardening risk` when complexity and coverage need to pick the next target
  before a Ship verdict is credible.

Do not block Ship merely because a hardening mode exists. Block only when the
changed behavior is high-risk, the current tests are visibly weak, or the
review cannot name a live evidence path that would catch the likely failure.

## Completion Gate

Every Ship or Conditional verdict must include:

```markdown
## Completion Gate
- Exact end-user behavior changed: behavior or internal operator behavior the diff changes.
- Evidence that proves it: review finding, test output, QA artifact, or command result behind the verdict.
- Exact command/path/route exercised: command, route, file path, artifact, or tool call inspected.
- Repo-fit check: live repo pattern or contract the verdict compared against.
- Hardening recommendation / waiver: mode recommended, mode run, or waiver reason.
- Residual risk: unverified path, accepted survivor, or none with reason.
```

If there is no end-user behavior because the diff is internal, say so and name
the developer/operator behavior that changed instead.

## Review Scoring

After the final verdict, append one JSON line to `.groom/review-scores.ndjson`
in the target project root (create `.groom/` if needed):

```json
{"date":"2026-04-06","pr":42,"correctness":8,"depth":7,"simplicity":9,"craft":8,"verdict":"ship","providers":["claude","thinktank","codex","gemini"]}
```

- Scores (1-10) reflect cross-provider consensus, not any single reviewer.
- `pr` is the PR number, or `null` when reviewing a branch without a PR.
- `verdict`: `"ship"`, `"conditional"`, or `"dont-ship"`.
- `providers`: which review tiers contributed.
- This file is committed to git (not gitignored). `/groom` reads it for quality trends.

## Verdict Ref (git-native review proof)

After scoring, record the verdict as a git ref so `/deliver --polish-only` and
pre-merge hooks can enforce review requirements without GitHub PRs.

```bash
source scripts/lib/verdicts.sh
verdict_write "<branch>" '{"branch":"<branch>","base":"<base>","verdict":"<ship|conditional|dont-ship>","reviewers":[...],"scores":{...},"sha":"<HEAD-sha>","date":"<ISO-8601>"}'
```

- Write on every review, not just "ship" — "dont-ship" verdicts block pre-merge callers (`/deliver --polish-only` and the pre-merge hook).
- The `sha` field MUST be `git rev-parse HEAD` at the time of review. If the branch
  gets new commits after review, the verdict is stale and `/deliver --polish-only` will re-trigger review.
- Verdict refs live under `refs/verdicts/<branch>` and sync via `git push/fetch`.
- Also write a copy to `.evidence/<branch>/<date>/verdict.json` for browsability.
- The escape hatch (`HARNESS_KIT_NO_REVIEW=1`) is handled at the caller (`pre-merge-commit` hook), never inside `/code-review`.

Run the snippet from the target project root. Skip this step if
`scripts/lib/verdicts.sh` does not exist there (Harness Kit-only feature, not
expected in downstream repos).

## Gotchas

- **Self-review leniency:** Models overrate their own work. Reviewers must be separate sub-agents, not the builder evaluating itself.
- **Reviewing the whole codebase:** Review the diff, not the repo. `git diff main...HEAD` is the scope.
- **Skipping tiers:** Internal bench alone is same-model groupthink. Thinktank + cross-harness provide genuine model and harness diversity.
- **Misreading a live Thinktank run:** `review.md`, `summary.md`, and
  `agents/*.md` may not exist until late. Watch stderr progress or
  `trace/summary.json`, not just the directory listing. `thinktank review eval`
  is broken in 6.3.0, so consume the final stdout JSON or the finished files.
- **Treating all concerns equally:** Blocking issues (correctness, security) gate shipping. Style preferences don't.
- **Monoculture:** The whole point of three tiers is provider diversity. Don't skip external tiers for speed.
- **Over-prescribing prompts:** You are the marshal. Craft prompts that fit the diff. The references describe lenses, not scripts.
