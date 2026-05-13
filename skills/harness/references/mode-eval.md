# /harness eval

Test whether a skill improves agent outcomes, not whether its prose sounds
reasonable.

## Protocol

Every eval has four pieces:

1. **Task** — representative prompt and repo fixture/context.
2. **Transcript** — tool calls, intermediate artifacts, and final answer.
3. **Outcome** — the final state or artifact the skill was supposed to create.
4. **Graders** — pass/fail commands, static checks, rubric judge, or human
   calibration notes.

Prefer objective outcome graders first: commands run, files created, tests
pass, evidence paths exist, forbidden edits absent. Use rubric/model judges
only for judgment-heavy outputs such as strategy, review quality, or demo
craft; calibrate them against human examples when possible.

### Baseline comparison

Spawn two sub-agents in parallel with the same representative prompt. One runs
without the skill loaded (baseline). The other runs with the skill active.
Both produce their output and confidence, but confidence is not the score.
Score the outcome and transcript with the same graders.

Then spawn a critic sub-agent to compare the two outputs: which is better?
By how much? Is the skill load-bearing or marginal?

If improvement is marginal, the skill isn't load-bearing. Delete it.

## Eval directory convention

Write eval prompts and graders to `evals/` in the skill directory. Rerun after
changes and after model upgrades.

Minimum tree:

```
skills/<name>/evals/
  README.md              # capability under test and expected failure mode
  cases/<case>.md        # prompt + fixture/context pointers
  graders/<grader>       # command, rubric, or script used to judge outcome
```

Invented repo-local skills created by `/tailor` must include at least one eval
seed before installation. The seed can be small, but it must name the expected
artifact and the grader that proves the skill helped.
