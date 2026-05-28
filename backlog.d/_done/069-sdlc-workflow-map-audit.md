# SDLC workflow map audit

Priority: P1
Status: done
Estimate: S

## Goal

Re-examine Harness Kit's software-development lifecycle map and verify that the
workflow skills are split at the right boundaries. The audit should identify
over-split phases, underrepresented phases, and cross-cutting responsibilities
that should be carried by existing skills instead of becoming new skills.

## Non-Goals

- Do not create a new orchestration DSL.
- Do not add a new lifecycle skill unless the audit proves an existing phase
  cannot own the work cleanly.
- Do not rename skills for aesthetics.

## Oracle

- [x] Produce a lifecycle table from idea to post-ship learning:
      `groom -> shape -> implement -> instrument/observe -> review ->
      refactor -> qa -> demo -> yeet -> settle -> ship -> monitor -> reflect`.
- [x] For each phase, name the owning skill, inputs, outputs, evidence, and
      whether the boundary is too broad, too narrow, or right-sized.
- [x] Explicitly evaluate whether instrument/observe belongs in `/implement`,
      `/qa`, `/monitor`, `/seed`, a new skill, or a shared contract.
- [x] Compare the audit against at least one real shipped Harness Kit cycle and
      one downstream app repo cycle.
- [x] Emit at most three follow-up backlog items, each with an oracle.
- [x] `dagger call check --source=.` passes if any skill docs change.

## Notes

This comes from the May 26 strategy discussion around Cursor Team Kit, Codex
goal-mode automations, prompt-debt reflection, and observability coverage. The
strong signal was not "add more skills"; it was "check whether the lifecycle
surface is carved correctly and whether completion evidence is consistent
across phases."

## Closure

Audit artifact: `docs/skill-catalog-audit.md`.

Accepted outcome:

- Keep `/create-repo-skill` broad as a thin generator router.
- Do not add `/observe`; carry instrument/observe as a shared lifecycle
  contract across `/shape`, `/implement`, `/qa`, `/monitor`, `/reflect`, and
  `/groom`.
- Use existing follow-ups instead of duplicating backlog:
  `070-observability-coverage-loop.md`, `053-skill-quality-audit-mode.md`, and
  `070-clean-copied-skill-reference-quality.md`.

Provider receipts:

- `629723ff-a563-42d2-ac78-e25be98fd2ba` (`codex`)
- `ce043681-18d6-4bb2-803a-486eb7d0c3c2` (`claude`)
