# Project-local skill resolution doesn't override global skills

Priority: high
Status: done
Estimate: M

## What Was Built

Commit `6d3944d "fix: scaffold templates use name: qa/demo to properly override
global skills"` resolved the root cause (Hypothesis #1): project-local resolution
keys on the `name` field, not the directory name. Scaffold templates (`/qa scaffold`,
`/demo scaffold`) now write skills with `name: qa` / `name: demo` so project-local
versions take precedence over the global fallback stubs.

Supplementary work: `3418134` moved scaffold references from harness/ into domain
skills; `977b4eb` converted qa and demo to scaffold-first thin redirects.

Remaining structural hardening (prevent shadowing global *workflow* skills —
groom, shape, autopilot, etc.) is covered by backlog item 029 via
`scripts/tailor-lint.sh` pre-commit enforcement.

## Goal

Project-local skills in `.claude/skills/` should override global skills in
`~/.claude/skills/` when both define the same skill name. Currently the global
fallback always wins.

## Evidence

Scaffolded `/qa` and `/demo` for retired-bench (retired bench repo#286):
- Wrote `.claude/skills/qa/SKILL.md` (name: `retired-bench-qa`)
- Wrote `.claude/skills/demo/SKILL.md` (name: `retired-bench-demo`)
- Invoked `/qa` and `/demo` from the retired-bench project directory
- Both resolved to the global `~/.claude/skills/qa/` and `~/.claude/skills/demo/`
  fallback stubs instead of the project-local versions

## Hypotheses

1. **Name mismatch:** Global skill is `name: qa`, project-local is `name: retired-bench-qa`.
   Resolution may key on the `name` field, not the directory name. Fix: use `name: qa`
   in the project-local skill.
2. **Directory not searched:** Claude Code may not search `.claude/skills/` at project
   root — may need `.claude/commands/` or a different convention.
3. **Load order:** Global skills load first and short-circuit before project-local
   skills are checked.

## Oracle

- [ ] `/qa` invoked from retired-bench root resolves to `.claude/skills/qa/SKILL.md`
- [ ] `/demo` invoked from retired-bench root resolves to `.claude/skills/demo/SKILL.md`
- [ ] Global fallback still fires in projects without a scaffolded skill

## Non-Goals

- Changing the global fallback skill content
- Supporting nested skill overrides (project > org > global)
