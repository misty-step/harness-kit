# /compound eval

The one claim `/compound` must earn: **after a solved problem, the skill
causes an agent to write exactly one grep-ready, anchored learning or update an
overlapping one, where a raw closeout prompt either loses the lesson in chat or
creates duplicative prose without retrieval fields.**

This is a `mode-eval` A/B run. Arms: A = `/compound` installed and invoked; B =
raw same-model closeout prompt with the same solved-problem evidence. Grade
blind, objective first, judge a different model family than the workers.

## Fixtures

| # | Prompt | Repo @ SHA | Forbidden edits | What it stresses |
|---|---|---|---|---|
| 1 | Capture the lesson from an artifact shelf slashless-directory redirect fix with existing code and PR evidence. | `harness-kit@HEAD` + `misty-step/bastion@559d2791ab7fe59401a1b5f0612a8df37c3a7319` evidence packet | editing skills or schema | new bug-track learning, anchored evidence, grep-ready fields |
| 2 | Capture a lesson whose tags overlap an existing `docs/solutions/web-serving/*.md` learning but whose evidence does not change the rule. | `harness-kit@HEAD` + seeded overlapping learning | adding a second near-duplicate | overlap check and no-duplicate behavior |
| 3 | Capture a stable integration constraint learned during research, not a bug. | `harness-kit@HEAD` + short research note | using `problem_type: bug-track` | knowledge-track classification |

## Objective checks

- [ ] Output creates or updates exactly one file under `docs/solutions/`.
- [ ] Frontmatter includes `title`, `tags`, `module`, `problem_type`,
      `applies_when`, `severity`, and `date`.
- [ ] `applies_when` has 2-4 concrete list items.
- [ ] Body cites at least one checkable `repo@SHA`, PR, command, route, or
      `file:line` anchor.
- [ ] The run reports the overlap query used:
      `rg -n --glob '*.md' '^(title|tags|applies_when):|<module>|<failure-mode>' docs/solutions`.
- [ ] Fixture 2 does not create a near-duplicate learning.

## Rubric

| Dimension | 5 | 1 |
|---|---|---|
| Reusability | The lesson changes future agent behavior in a similar task. | The note is a session summary. |
| Retrieval quality | Title, tags, and applies_when are concrete grep handles. | Future agents would not find it by grep. |
| Evidence discipline | Claims are anchored to checkable repo evidence. | Claims rely on memory or chat. |

## Pass condition

Arm A beats Arm B on retrieval quality and evidence discipline across at least
2 of 3 fixtures, and ties-or-wins every objective check. A no-op skill fails
because raw closeout reliably omits overlap checks or grep-ready frontmatter.

## Human anchor

Pending. The operator should blind-grade fixture 2 because duplicate avoidance
is the easiest place for plausible but harmful corpus growth.

## Cadence

Edit-time smoke on any `skills/compound/**` or `harnesses/shared/references/learnings.md`
change. Full 3-fixture A/B when the frontmatter schema or duplicate policy
changes. Re-audit after major model releases.

## Run log

Seeded 2026-07-04 for `harness-kit-905`; no full A/B run yet.
