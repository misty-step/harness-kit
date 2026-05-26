# SDLC workflow map audit

Priority: P1
Status: ready
Estimate: S

## Goal

Re-examine Spellbook's software-development lifecycle map and verify that the
workflow skills are split at the right boundaries. The audit should identify
over-split phases, underrepresented phases, and cross-cutting responsibilities
that should be carried by existing skills instead of becoming new skills.

## Non-Goals

- Do not create a new orchestration DSL.
- Do not add a new lifecycle skill unless the audit proves an existing phase
  cannot own the work cleanly.
- Do not rename skills for aesthetics.

## Oracle

- [ ] Produce a lifecycle table from idea to post-ship learning:
      `groom -> shape -> implement -> instrument/observe -> review ->
      refactor -> qa -> demo -> yeet -> settle -> ship -> monitor -> reflect`.
- [ ] For each phase, name the owning skill, inputs, outputs, evidence, and
      whether the boundary is too broad, too narrow, or right-sized.
- [ ] Explicitly evaluate whether instrument/observe belongs in `/implement`,
      `/qa`, `/monitor`, `/tailor`, a new skill, or a shared contract.
- [ ] Compare the audit against at least one real shipped Spellbook cycle and
      one downstream app repo cycle.
- [ ] Emit at most three follow-up backlog items, each with an oracle.
- [ ] `dagger call check --source=.` passes if any skill docs change.

## Notes

This comes from the May 26 strategy discussion around Cursor Team Kit, Codex
goal-mode automations, prompt-debt reflection, and observability coverage. The
strong signal was not "add more skills"; it was "check whether the lifecycle
surface is carved correctly and whether completion evidence is consistent
across phases."
