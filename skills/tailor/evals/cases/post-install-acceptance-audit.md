# Case: post-install acceptance audit rejects generic-but-valid harness

## Prompt

Run `/tailor` after installing a repo-local harness. The deterministic install
checks are green, but the installed harness contains these facts in its
post-tailor evidence packet:

- `/trace` is missing from the shared skill root.
- `/monitor` still treats a `/deploy` receipt as its primary input even though
  the repo brief says the repo has no deploy target.
- `/ci` says the gate has 12 sub-gates while the repo brief says the gate is
  `dagger call check --source=.` with 19 sub-gates.
- One installed workflow skill is byte-identical to its Spellbook source.
- A rewritten skill contains a hardcoded `/Users/<name>/...` checkout path.
- `AGENTS.md` has a P0 known-debt entry marked `(unfiled)`.

Produce only the post-install acceptance decision and repair plan. Do not
modify files.

## Expected Outcome

- Runs deterministic evidence collection first and cites the
  `.spellbook/tailor/audit/<run-id>/evidence.json` path.
- Does not treat the evidence collector as a quality verdict or numeric score.
- Dispatches a critic subagent to judge bespoke fit from the evidence packet,
  repo brief, `AGENTS.md`, rewritten workflow skills, and source workflow
  skills.
- Fails the acceptance audit with concrete blockers for missing `/trace`,
  deploy lifecycle leakage, stale gate count, byte-identical workflow content,
  hardcoded user path, and unfiled debt.
- Persists the critic decision to
  `.spellbook/tailor/audit/<run-id>/verdict.json`.
- Gives repair directives tied to evidence refs.
- Does not declare `/tailor` successful until blockers are repaired and the
  collector plus critic are rerun.
- Does not introduce deterministic bespoke-quality scores, percentages, or
  regex-based semantic verdicts.
