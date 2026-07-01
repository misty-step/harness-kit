---
name: skill-eval
description: |
  Prove a skill beats no-skill with a falsifiable A/B eval, or retire it.
  Design, generate, run, and maintain a skill-specific eval: name the one
  claim the skill must earn, run it skill-on vs raw same-model, grade blind
  with objective checks first, return a keep/adapt/cut verdict.
  Use when: "eval this skill", "does this skill help", "prove the skill beats
  no skill", "write an eval for", "benchmark a skill", "is this skill worth
  it", "skill A/B", "skill regression test", "generate skill evals".
  Trigger: /skill-eval, /eval-skill, /prove-skill.
argument-hint: "[skill-name] [--generate|--run|--smoke]"
---

# /skill-eval

Build the falsifier for a skill. A skill earns its place only when an agent
**with** it produces measurably better outcomes than the same agent **without**
it — otherwise it is context tax. This skill designs, runs, and maintains that
proof, and the proof must be able to come back **no-skill won**.

An eval is a *run with a grader*, never a directory shape. Structural eval
trees were deleted as theater in the 2026-06 consolidation (`backlog.d/103`);
do not rebuild them. The exemplar to copy is
`skills/design/evals/routing-eval.md`: an answer key, an objective grader, a
pass bar a no-op revision fails, a run log. The protocol contract is
`skills/harness-engineering/references/mode-eval.md`.

## The loop

1. **Name the one claim.** Every skill earns exactly one load-bearing,
   falsifiable claim — the thing it makes true that raw prompting does not
   (design: "routes to the right specialist first try"; shape: "a stranger
   builds the right thing from the packet"). Write it as a sentence that could
   be wrong. A skill whose claim you can't name, or whose claim raw prompting
   already meets, is a deletion candidate — say so and stop. That is a valid,
   cheap eval result, not a failure to produce one.
2. **Fix the task.** 2–3 fixtures that stress the claim *differently*: a frozen
   prompt + a repo pinned at a SHA + a forbidden-edits list. One fixture is
   noise; cover the claim's failure modes, not one happy path.
3. **Run the A/B.** Same model, same fixture, two arms:
   - **A** — the skill installed and invoked.
   - **B** — raw: the bare instruction a sharp operator would type, same model,
     same repo access, no skill.
   - **C (optional)** — a credible alternative primitive (external skill,
     Ponytail for simplicity pressure). Add only when one exists; A-vs-B is the
     floor (backlog 112's three-condition shape).
   Drive it per `references/run-recipe.md` (native-subagent smoke = free;
   `council.sh` + decorrelated families = the serious run).
4. **Grade blind, objective first.** The grader sees the artifacts and the
   fixture, never which arm is which, and is a *different model family* than the
   workers. Mechanical checks before taste (below).
5. **Verdict + disposition.** Pass = A beats B paired on the claim across ≥2 of
   3 fixtures. Then label the skill: `keep` / `adapt` / `cut` /
   `needs-more-tasks` / `graduate-to-Daedalus`. The verdict is about the skill's
   right to exist, not a vanity score.

## Graders — three tiers, and the human is the gold

- **Objective first** (scriptable, ~free, every edit): sections present and
  non-empty, oracle is a runnable command not "it should work", cited paths
  resolve at the SHA, gate passes, forbidden edits absent, artifacts render.
  These fail without a judge — push every check you can down to this tier.
- **Human judgment is the ground truth** for the judgment-heavy delta. A spec's
  buildability, a design's taste, a critique's bite — these are the operator's
  call, and for most skills that verdict *needs human input*, not just a model's.
  The keep/adapt/cut decision on a taste-heavy skill is signed off by the human,
  or by a grader carrying a *recent* human anchor (see Cadence) — never by an
  unvalidated model judge alone. At minimum, the operator blind-grades one fixture
  per eval; that grade is the anchor everything else is checked against.
- **The agent rubric is a calibrated proxy, not the judge.** A blind,
  decorrelated model grader is how you afford to run the rubric often and cheaply
  — but it *approximates* the operator's taste and drifts. 1–5 per dimension tied
  to the claim, one-line justification, scored blind, a different family than the
  workers (a worker grading its own family flatters itself; same-family smokes
  prove the loop *fires*, not the margin). Trust its margin only while it matches
  the human anchor. When proxy and human disagree, the rubric is broken (or the
  claim is) — fix the grader; do not overrule the human.

## Cadence — match cost to stakes

A full pass is ~15–18 runs (paid, slow). Do not gate every edit on it.

- **Every `skills/<skill>/**` edit** → cheap objective checks + a 1-fixture
  native-subagent smoke. Catches gross regressions for free.
- **Contract-level change** (the skill's claim or output shape moved) → full
  A/B, decorrelated, all fixtures. The skill changed; re-earn the claim.
- **Major model release** → re-audit. *This is the point of the eval.* A
  stronger bare model erodes every skill's edge; the skill that beat raw on the
  old model may be railroading on the new one. The eval is how you find the
  skills to retire.
- **Continuous** → telemetry (`harness-kit-checks telemetry`): did it trigger
  when it should, did loaded sessions cost more than they returned. Context, not
  proof (backlog 112).
- **Calibration (the human anchor)** → every eval carries ≥1 fixture the operator
  graded blind; the agent grader is trusted only while its verdict matches that
  anchor. Re-anchor when the rubric changes, the worker model upgrades, or
  proxy-vs-human last diverged. No recent anchor → the automated verdict is
  unvalidated; say so rather than quoting the margin.

## Gotchas

- **Falsifier that can't fail.** A pass bar the skill always clears is theater.
  Before running, ask: what result tells me to delete this skill? If nothing
  can, the eval is decoration.
- **Rubric laundering.** Vague dimensions ("is it good") let the grader
  rubber-stamp the skill arm. Tie every dimension to the claim; prefer an
  objective check to a rubric line wherever one exists.
- **Self-graded family.** Grader shares the worker's model family → inflated
  margin. Smoke-only waiver, never the serious verdict.
- **One fixture.** A single task proves nothing generalizes. ≥2, spanning the
  claim's distinct failure modes.
- **Eval bloat.** This skill stays minimal. Serious, repeated arena work
  (composition sweeps, model selection) graduates to Daedalus; it does not
  expand Harness Kit into a benchmark platform.
- **Grading prose, not outcomes.** "The packet reads well" is not the claim. "A
  cold lane built the feature from the packet" is. Grade the outcome the skill
  promises.
- **Agent judge ≠ ground truth.** The rubric grader is a proxy for the operator's
  taste; an unanchored proxy rubber-stamps. Anchor it to a blind human grade or
  don't quote the margin.

## Route

| Need | Load |
|---|---|
| generate a new skill's eval | `templates/eval-spec.md` |
| blind grader prompt | `templates/grader-prompt.md` |
| drive the A/B (smoke + serious) | `references/run-recipe.md` |
| eval protocol contract | `skills/harness-engineering/references/mode-eval.md` |
| canonical worked eval | `skills/design/evals/routing-eval.md` |
| first instance | `skills/shape/evals/shape-eval.md` |
| parent backlog spec | `backlog.d/112-harness-eval-bench.md` |

## Verification

The eval spec lands at `skills/<skill>/evals/<skill>-eval.md` (mirrors the design
exemplar). Run evidence lands at `.evidence/harness-evals/<skill>/<date>/` —
sanitized artifacts + scored receipts only, never raw transcripts with secrets.
A run is real only when it produced both arms and a grader verdict that *could*
have gone the other way.

`cargo run --locked -p harness-kit-checks -- check-eval-coverage --repo .`
(folded into the repo gate) enforces that every first-party skill has either
an eval spec or a live, unexpired `skills/<skill>/evals/WAIVER.md` — see
`skills/harness-engineering/references/skill-design-principles.md`'s "New
Skill: Eval Scaffold Is Not Optional" for the exact new-skill scaffolding
steps this gate enforces.
