# /harness-engineering eval

Test whether a skill improves agent outcomes, not whether its prose sounds
reasonable.

Load `../../../harnesses/shared/references/verification-system-first.md` when
designing the eval. The eval is the verification system for an agent-behavior
claim: task, transcript, outcome, grader, evidence packet, and cadence must all
be explicit.

## Protocol

Every eval has four pieces:

1. **Task** — representative prompt and repo fixture/context.
2. **Transcript** — tool calls, intermediate artifacts, and final answer.
3. **Outcome** — the final state or artifact the skill was supposed to create.
4. **Graders** — pass/fail commands, static checks, rubric judge, or human
   calibration notes.
5. **Cadence** — when this eval reruns: one-off shape evidence, pre-merge
   gate, model-upgrade audit, or recurring Mode B benchmark lane.

Prefer objective outcome graders first: commands run, files created, tests
pass, evidence paths exist, forbidden edits absent. Use rubric/model judges
only for judgment-heavy outputs; calibrate against human examples.

## How to run one

A/B in worktrees: spin one agent on the task with the skill installed and
one without (or with the candidate revision), then a fresh comparison agent
grades both outputs against the rubric — it must not know which is which.
Two or three task instances beat one; decorrelate the grader's model family
from the workers'.

## Boundaries

- Structural eval trees are theater; they were deleted in the 2026-06
  consolidation. An eval is a run with a grader, not a directory shape.
- A benchmark with no baseline, variance note, or threshold is not an eval
  result; it is a transcript waiting for a grader.
- The cheapest valid eval is live telemetry plus judgment: did the skill
  trigger when it should, and did sessions that loaded it end better?
  (`harness-kit-checks telemetry`.)
- HarnessX-style trace evolution is review-only here: a Mode B/eval lane may
  propose typed harness edits from sanitized traces, but no source edit ships
  without held-out tasks, the full Harness Kit gate, fresh critic review, and
  human approval. Candidate patches are artifacts, not self-merging workers.
- Serious, repeated eval work (benchmarking agent compositions, model
  selection for a recurring workflow) belongs in Daedalus's arena loop, not
  ad-hoc here.
