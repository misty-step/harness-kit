---
name: code-review
description: |
  Parallel multi-agent code review for Spellbook diffs — SKILL.md / agent md
  changes, Dagger gate changes, bootstrap/scripts/harness configs. Dagger
  check is tier 0; philosophy bench + thinktank + cross-harness add human
  judgment on top. Synthesize, fix blockers, loop until clean.
  Use when: "review this", "code review", "is this ready to ship",
  "check this diff", "review my changes".
  Trigger: /code-review, /review, /critique.
argument-hint: "[branch|diff|files]"
---

# /code-review

Multi-tier review for Spellbook. The product here is the **agent-shaped
coating** — skills, agents, hooks, lint gates. Reviews anchor to
`harnesses/shared/AGENTS.md` red flags and the invariants in
`.spellbook/repo-brief.md`. You are the marshal: read the diff, let Dagger
do the automated first pass, dispatch human-judgment reviewers in parallel,
synthesize, fix blockers, loop.

## Delegation Floor

When `.spellbook/agents.yaml` exists, `/code-review` starts by probing the
roster and dispatching two or more non-lead roster members for independent
review lanes. The internal philosophy bench and Thinktank are additive; they
do not replace the roster floor. Give reviewers the diff, acceptance criteria,
and risk lens, not the author's reasoning. The lead marshal synthesizes
findings, verifies fixes, records receipts, and only works directly for
mechanical command execution, emergency unblocks, explicit user-forbidden
delegation, or an explicit waiver when fewer than two roster members are
available.

## What Gets Reviewed Here

Almost every diff in this repo is one of:

1. **Skill changes** — `skills/<name>/SKILL.md` + `references/`. Judgment
   encoding. Triggerable frontmatter. Self-containment. <500 lines.
2. **Agent changes** — `agents/<name>.md`. Scoped persona. Tool restrictions.
3. **Dagger gate changes** — `ci/src/spellbook_ci/main.py`. A new gate is a
   load-bearing wall (Red Line: never lower quality gates).
4. **Bootstrap / scripts** — `bootstrap.sh`, `scripts/*.sh`, `scripts/*.py`.
   Cross-harness install paths. Portable paths. Pre-commit hooks.
5. **Harness config** — `harnesses/{claude,codex,pi,shared}/…`. Shared
   AGENTS.md is doctrine; per-harness settings must stay parity-equivalent.

If the diff looks like a typical "feature" change on app code, something is
wrong — this repo has no app. Check the diff scope before proceeding.

## Marshal Protocol

### 0. Let Dagger do the first pass

Before spawning any reviewer, run (or confirm the caller ran):

```bash
dagger call check --source=.
```

The 14 parallel gates catch everything mechanical. Named gates (see
`.spellbook/repo-brief.md` for the full table):

- `lint-yaml`, `lint-shell` (shellcheck), `lint-python` (py_compile)
- `check-frontmatter` — required fields, line limits
- `check-index-drift` — `index.yaml` must match generator output
- `check-vendored-copies`, `check-exclusions`
- `check-portable-paths` — no `/Users/<name>/` outside Claude-only paths
- `check-harness-install-paths` — seed/tailor writes must not be Claude-only
- `check-deliver-composition` — `/deliver` must compose via trigger syntax
- `check-no-claims` — no `claims.sh` under `skills/`

**Reviewers must not re-run or duplicate these checks.** If Dagger failed,
the fix is `dagger call heal --source=. --model=gpt-4.1 --attempts=2` or a
builder sub-agent — not a human reviewer restating what shellcheck already
said. Reviewer cycles are for judgment Dagger can't form.

### 1. Read the diff

`git diff $BASE...HEAD` (default base: `master`). Classify: which of the five
change categories above? What does this diff touch? What doesn't it touch
that it probably should (tests, references/, frontmatter, index regen)?

### 2. Select the philosophy bench (static map)

Run the selection algorithm in `references/bench-map.yaml`:

- `git diff --name-only $BASE...HEAD` → changed files
- Start from `default` (`critic`, `ousterhout`, `grug`)
- Union `add` agents for every rule whose glob matches any changed file
- De-dup; cap at 5 with `critic` pinned

Available bench (philosophy agents under `/Users/phaedrus/Development/spellbook/agents/`):

| Agent | Lens | Best on Spellbook diffs |
|---|---|---|
| **critic** | correctness/depth/simplicity/craft rubric | every review |
| **ousterhout** | deep modules, information hiding | SKILL.md restructuring, new skill interfaces, Dagger gate composition — catches *shallow modules* and *pass-through layers* |
| **grug** | complexity demon | catches *semantic workflow DSLs around general agents*, *speculative abstractions*, and scaffolding that compensates for strong models |
| **carmack** | shippability, direct implementation | bootstrap/scripts, bash harnesses, "is this the simplest working thing" |
| **beck** | TDD, YAGNI | new test coverage for scripts/, python modules, eval fixtures |

No a11y triad. This repo has no UI surface — if an a11y reviewer shows up in
a bench for this repo, the map is broken.

See `references/internal-bench.md` for lens detail and prompt guidance.

### 3. Dispatch the three judgment tiers in parallel

| Tier | What | How |
|---|---|---|
| Philosophy bench | 3-5 Explore sub-agents with named lenses | Agent tool, tailored prompts. Read-only. |
| Thinktank review | 10 reviewers across 8 model providers | `thinktank review` CLI. See `references/thinktank-review.md` |
| Cross-harness | Two or more available non-lead providers from `.spellbook/agents.yaml` | See `references/cross-harness.md` |

Parallel dispatch only — sequential here wastes wall time. Thinktank gotcha:
wait for the process to exit, or for `trace/summary.json` to reach `complete`
or `degraded` with a `run_completed` event in `trace/events.jsonl`, before
you consume the run. Mid-run directories are not final artifacts.

### 4. Synthesize

Collect all outputs. Deduplicate findings. Rank by severity:

- **Blocking** — correctness, security, cross-harness parity violation,
  self-containment violation (`../..`, `$REPO_ROOT/…`), lowered quality gate,
  broken frontmatter, direct `dagger call check` inside `skills/deliver/SKILL.md`
  (fails `check-deliver-composition`).
- **Important** — shallow module, pass-through layer, speculative abstraction,
  SKILL.md over 500 lines, description not assertive/triggerable, untested
  script, regex over agent prose, semantic workflow DSL.
- **Advisory** — naming, style, ordering, doc polish.

### 5. Verdict

No blocking findings → **Ship**. Any blocking → fix loop.

## Fix Loop

For each blocking finding, spawn a **builder** sub-agent with file:line and
the specific fix instruction. Builder applies strategic (Ousterhout) and
simple (grug) fixes, runs `dagger call check --source=.`, commits atomically.

After all fixes land, **re-dispatch all three judgment tiers** (full, not
spot-check). Loop until clean. Max 3 iterations — escalate to human if still
blocked; repeated blocks usually mean the shape was wrong, not the code.

## Review Judgment — What This Repo's Reviewer Actually Adds

Dagger catches syntax, frontmatter, portable paths, composition, drift.
The reviewer's cycles go to things Dagger cannot evaluate. Named red flags
from `harnesses/shared/AGENTS.md` to cite verbatim:

> Shallow modules, pass-through layers, hidden coupling, large diffs,
> untested branches, speculative abstractions, stale context,
> responding to agent errors with prose instead of structural fixes,
> regexes over agent prose, semantic workflow DSLs around general agents.

Additional Spellbook-specific checks:

- **Does the skill encode JUDGMENT, not procedure?** If the model already
  knows how, delete the skill. A SKILL.md that reads like a runbook for a
  strong model is dead weight. Reference form: `skills/flywheel/SKILL.md`
  (43 lines). (Memory: *feedback_bitter_lesson_skill_design*.)
- **Cross-harness first (Red Line).** What does this do on Codex? On Pi?
  Anchoring a design on Claude's `enabledPlugins` or Codex's `/plugins`
  without a filesystem-level base is a parity violation. Prior art:
  `harnesses/pi/settings.json:skills[]`.
- **Self-containment.** No `../..`, no `$REPO_ROOT/…` sourcing. Scripts
  resolve via `readlink -f` + `$SCRIPT_DIR/lib/…`. State roots anchor to
  the *invoking* project's `git rev-parse --show-toplevel`. Symlink-install
  + invoke from a foreign project is the canonical test.
- **Thin harness over semantic orchestration.** Scaffold that tells a strong
  model *how* to think generically is a grug target. Keep invariants it
  cannot infer; delete the rest.
- **Description is the trigger.** Frontmatter `description` must be
  assertive — not "this skill helps with X" but "Use when: <concrete
  signals>. Trigger: /foo."
- **`/deliver` composition.** Any diff under `skills/deliver/` must compose
  atomic phase skills (`/code-review`, `/ci`, `/qa`, `/implement`,
  `/refactor`, `/shape`) via trigger syntax. Raw `dagger call check` inside
  `/deliver` fails `check-deliver-composition` — flag blocking.
- **No `references/<repo-name>.md` sidecars.** Repo-specific content belongs
  in SKILL.md body. Stack-topic references (`references/convex-patterns.md`)
  are fine; `references/bitterblossom.md` is not.
- **No `index.yaml` edits.** Pre-commit regenerates it. A human-edited
  `index.yaml` in the diff is either churn or drift — flag.
- **Plausible ≠ correct.** SKILL.md prose that sounds right but fails the
  subtractive test (could describe any Next.js or Elixir repo verbatim) is
  shallow. Reviewer must demand repo-specific anchors.

## Spellbook-Shaped Review Examples

**Diff: new skill `skills/rebase/SKILL.md`.**
Bench (via map): `critic`, `ousterhout`, `grug`. Judgment calls: does it
encode judgment or procedure? Does the description trigger? Does it touch
`../..`? Cross-harness: does it work on Codex/Pi? Red flags to hunt:
*speculative abstractions*, *semantic workflow DSLs*.

**Diff: new Dagger gate in `ci/src/spellbook_ci/main.py`.**
Bench: `critic`, `ousterhout`, `carmack` (shell/python), `beck` (test
coverage). Judgment calls: is this gate load-bearing or ceremony? Does it
compose with existing parallel gates without serializing? Does it have a
deterministic failure mode? Does `heal` have a recipe for it? Red Line:
lowering any existing threshold is a blocker.

**Diff: bootstrap.sh change.**
Bench: `critic`, `carmack`, `grug`. Judgment calls: does it preserve
symlink + download parity? If it adds a skill, does it belong in
`GLOBAL_SKILLS` or is it per-repo content for `/tailor` / `/seed`?
Does it honor the COPIED-not-symlinked exception for
`harnesses/claude/settings.json`? Red flag: *hidden coupling* between
bootstrap and harness-specific settings.

**Diff: agent persona update under `agents/`.**
Bench: `critic`, `ousterhout`, `grug`. Judgment calls: tool restrictions
coherent with scope? Does the persona add structural guarantees a
general-purpose ad-hoc subagent wouldn't? Named agents exist for structural
reasons (shared AGENTS.md) — if it's just prose, it should be ad-hoc.

## Thermo / Deslop Lens

For meaningful diffs, add the Cursor-kit style structural lens before a Ship
verdict:

- look for the code-judo move that deletes branches, helpers, modes, or layers
  while preserving behavior
- treat new ad-hoc conditionals in busy flows as design findings, not style nits
- force wrappers, casts, optionality churn, magic fallbacks, and bespoke helpers
  to justify themselves
- flag AI slop: unnecessary comments, abnormal defensive try/catch, internal
  mocks, `any` casts, deep nesting, and codegen residue

Route behavior-preserving cleanup to `/refactor`; block Ship when the slop or
spaghetti makes the changed behavior harder to reason about.

## Completion Gate

Every Ship or Conditional verdict includes:

```markdown
## Completion Gate
- Exact end-user behavior changed:
- Live evidence that proves it:
- Exact command/path/route exercised:
- Repo-fit check:
- Residual unverified paths:
```

For Spellbook-internal diffs, replace end-user behavior with the
developer/operator behavior that changed.

## Review Scoring

After the final verdict, append one JSON line to `.groom/review-scores.ndjson`
(create `.groom/` if needed):

```json
{"date":"2026-04-20","pr":null,"correctness":8,"depth":7,"simplicity":9,"craft":8,"verdict":"ship","providers":["claude","thinktank","codex","gemini"]}
```

Scores (1-10) are cross-provider consensus, not any single reviewer. `pr`
is the PR number or `null` for a branch. `verdict` is `ship`, `conditional`,
or `dont-ship`. Committed to git; `/groom` reads it for quality trends.

**Current reality (backlog.d/023):** the feedback loop on
`.groom/review-scores.ndjson` is operationally empty — scores are written
but not yet consumed by `/groom`. Write them anyway; closing that loop is
tracked work.

## Verdict Ref (git-native review proof)

After scoring, record the verdict as a git ref so `/settle` and pre-merge
hooks can enforce review without depending on GitHub PRs:

```bash
source scripts/lib/verdicts.sh
verdict_write "<branch>" '{"branch":"<branch>","base":"<base>","verdict":"<ship|conditional|dont-ship>","reviewers":[...],"scores":{...},"sha":"<HEAD-sha>","date":"<ISO-8601>"}'
```

- Write on every review, not just "ship" — a `dont-ship` verdict blocks
  `/settle --land`.
- `sha` MUST be `git rev-parse HEAD` at review time. New commits after
  review make the verdict stale; `/settle` re-triggers review.
- Verdict refs live under `refs/verdicts/<branch>` and sync via `git
  push/fetch`.
- Also write `.evidence/<branch>/<date>/verdict.json` for browsability.
- Escape hatch `SPELLBOOK_NO_REVIEW=1` is handled at the caller
  (`scripts/land.sh`, `pre-merge-commit`), never inside this skill.

## Gotchas

- **Duplicating Dagger.** Reviewers restating frontmatter errors or shellcheck
  warnings is wasted cycles. If `dagger call check` is red, run `heal` or
  spawn a builder — don't spawn a human-judgment bench to re-read the error.
- **Self-review leniency.** Models overrate their own work. Reviewers MUST
  be separate sub-agents, not the builder evaluating itself.
- **Reviewing the whole repo.** Review the diff, not the catalog. Scope is
  `git diff $BASE...HEAD`.
- **Skipping tiers for speed.** Philosophy bench alone is same-model depth,
  not diversity. Thinktank + roster-driven cross-harness review provide real
  foundation diversity. Same-model self-critique is theater
  (see AGENTS.md "Diverge Before You Converge").
- **Misreading a live Thinktank run.** `review.md`, `summary.md`, and
  `agents/*.md` may appear only late. Watch stderr JSON progress or
  `trace/summary.json`, not just the directory listing. `thinktank review
  eval` is broken in 6.3.0 — consume final stdout JSON or finished files.
- **Treating all concerns equally.** Blocking = correctness, security,
  cross-harness parity, self-containment, lowered gate. Style preferences
  don't gate shipping.
- **A11y bench on this repo.** If the map adds an a11y reviewer, the map is
  broken — no UI surface exists here. Fix the map.
- **Over-prescribing prompts.** You are the marshal. Craft prompts that fit
  the diff. The references describe lenses, not scripts.
- **Reviewing tailor-owned files.** `/tailor`-written files carry a
  `.spellbook` marker with `installed-by: tailor`. A diff that modifies
  tailor-owned content by hand is a scope question — confirm with the user
  before reviewing as a normal change.
