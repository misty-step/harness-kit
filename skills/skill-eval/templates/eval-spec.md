# /<skill> eval

> Template. Copy to `skills/<skill>/evals/<skill>-eval.md` and fill. Delete this
> line and the bracketed guidance. Keep it a run-with-a-grader, not a directory
> shape (`skills/design/evals/routing-eval.md` is the worked exemplar).

The one claim `<skill>` must earn: **[one falsifiable sentence — the thing the
skill makes true that raw prompting does not]**.

This is a `mode-eval` A/B run. Arms: A = skill installed + invoked; B = raw
same-model; C = [alternative primitive, or "n/a"]. Grade blind, objective first,
judge a different model family than the workers.

## Fixtures

[2–3. Each = frozen prompt + repo SHA + forbidden edits, stressing the claim
differently — include the boring/manual path and one that inverts a load-bearing
assumption.]

| # | Prompt | Repo @ SHA | Forbidden edits | What it stresses |
|---|---|---|---|---|
| 1 | … | … | … | … |
| 2 | … | … | … | … |

## Objective checks (scriptable, pass/fail, ~free)

[Mechanical, no judge. Each must be able to fail on a real artifact.]

- [ ] …
- [ ] …

## Rubric (1–5, blind, justify each)

[Only the judgment-heavy delta no objective check can see. Every dimension ties
to the claim. Drop any dimension a strong B-arm couldn't lose.]

| Dimension | What 5 looks like | What 1 looks like |
|---|---|---|
| … | … | … |

## Pass condition

A beats B paired on [the claim metric] across ≥2 of [N] fixtures, AND
ties-or-wins every objective check. [State the bar a no-op skill fails — what
the raw arm reliably omits.]

## Human anchor

[The operator blind-grades ≥1 fixture; record their verdict here and whether the
agent grader matched it. The automated verdict is provisional until a recent
human grade agrees. On disagreement, fix the rubric — don't overrule the human.]

## Cadence

[Edit-time smoke / contract-change full run / model-upgrade re-audit.]

## Run log

[Append-only. Date, arms, model families, score matrix, variance, decision label
(keep/adapt/cut/needs-more-tasks/graduate). A run that didn't fire both arms +
a falsifiable grader is not a result.]
