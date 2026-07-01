# Blind grader prompt (skill-eval)

> Fill the brackets, hand to a fresh agent of a *different model family* than the
> arms. The grader must not be told which artifact is the skill arm.

```
You are grading two attempts (X and Y) at the same task. You do not know how
either was produced. Be skeptical; default to "no meaningful difference" unless
the artifacts earn otherwise.

TASK GIVEN TO BOTH:
[paste the exact fixture prompt + repo context + forbidden edits]

THE ONE QUESTION:
Which attempt better achieves: [the skill's one claim, as a testable question —
e.g. "could a stranger build the feature from this packet without the author's
context?"]

ARTIFACT X:
[paste]

ARTIFACT Y:
[paste]

RETURN:
1. Objective checks — for X and Y, pass/fail each with the evidence line:
   [list the eval spec's objective checks]
2. Rubric — score X and Y 1–5 on each, one-line justification each:
   [list the eval spec's rubric dimensions]
3. Verdict — X, Y, or tie on the one question, plus the single most important
   reason. If tie, name what neither did that would have broken it.
4. Forbidden-edit check — did either modify files it shouldn't have?
```
