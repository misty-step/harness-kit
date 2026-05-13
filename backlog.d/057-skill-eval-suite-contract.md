# Skill eval suite contract

Priority: P1
Status: pending
Estimate: M

## Goal

Make skill evaluation a first-class contract: every new or materially changed
skill has at least one representative eval case, an expected outcome artifact,
and a grader that can be rerun after edits or model upgrades.

## Non-Goals

- Do not require every existing legacy skill to gain evals in one pass.
- Do not build a heavyweight benchmark platform.
- Do not replace `dagger call check --source=.`; this adds behavioral signal
  for skills, not repository lint.
- Do not rely only on model self-confidence as a score.

## Oracle

- [ ] `skills/harness/references/mode-eval.md` defines the canonical eval
      shape: task, transcript, outcome, graders.
- [ ] `scripts/` or `skills/harness/scripts/` includes a small validator that
      checks each `skills/<name>/evals/` tree has a README, at least one case,
      and at least one grader.
- [ ] `/harness eval <skill>` can run a baseline-vs-skill comparison and write
      a short result artifact under the skill's `evals/results/`.
- [ ] `/tailor` refuses to install a domain-invented skill unless it includes
      an eval seed or explicitly records why no runnable eval exists yet.
- [ ] At least three high-leverage skills (`tailor`, `code-review`, `qa`, or
      `demo`) get initial eval cases proving the contract works.
- [ ] `dagger call check --source=.` green.

## Notes

Frontier agent teams treat evals as the mechanism that turns prompt and harness
changes from vibes into engineering. The useful eval unit is not just final
text: it includes the task, transcript/trajectory, final outcome, and graders.
For Spellbook, that maps naturally to skill invocation: prompt + repo fixture,
tool/evidence trace, produced artifact, and pass/fail or rubric grader.

This complements `backlog.d/053-skill-quality-audit-mode.md`: audit reports
which skills lack eval coverage; this ticket defines what valid eval coverage
means and gives `/harness eval` enough structure to run it.
