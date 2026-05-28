# Bootstrap minimal globals with no global agents

Priority: P1
Status: superseded
Estimate: S

## Outcome

Superseded by the simpler opposite design: bootstrap installs every first-party
skill system-wide and keeps the core agents available. The minimal-global
premise did not justify its complexity.

## Goal

Make machine bootstrap install only the minimal global skills needed to bring
the harness library into a repo: `/tailor` and `/seed`. Stop linking or copying
global agents by default.

This is the mechanical first step behind the dynamic-delegation pivot.

## Non-Goals

- Do not delete `agents/*.md` in this ticket. Their content is handled by
  `061-retire-global-static-agents.md`.
- Do not change per-repo `/tailor` output except to stop assuming global agents
  exist.
- Do not remove runtime-native support for project-local agent files where a
  downstream repo explicitly asks for them.

## Oracle

- [ ] `bootstrap.sh` sets `GLOBAL_AGENTS=()` and does not fail when the list is
      empty.
- [ ] Remote bootstrap mode does not call GitHub to enumerate agents.
- [ ] Local bootstrap removes stale Harness Kit-managed global agent symlinks
      from supported harness dirs, while preserving human-authored files.
- [ ] Bootstrap output clearly says: "Global agents are not installed; skills
      now provide dynamic delegation guidance."
- [ ] `skills/seed/SKILL.md` and `skills/tailor/SKILL.md` stop promising global
      agent installation.
- [ ] `dagger call check --source=.` passes.

## Notes

This can land before the full content migration. It makes the default install
match the desired direction immediately, while the richer philosophy-lens work
continues in 061 and 063.
