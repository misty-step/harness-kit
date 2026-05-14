# Tailor durable brief + ownership conflict contract

Priority: high
Status: fixed
Estimate: S

## Problem

`/tailor` required `.spellbook/repo-brief.md` as the shared rewrite spine but
did not say what to do when a target repo ignored `.spellbook/`. It also told
agents to copy shared scripts verbatim while forbidding overwrites of unmarked
scripts, leaving no explicit resolution path.

## Fix

- Write a tracked compatibility copy of the repo brief when `.spellbook/` is
  ignored.
- Treat unmarked divergent shared scripts as ownership conflicts requiring
  `preserve / replace / diff`, not as silent self-audit failures.
- Add an eval case for ignored brief storage and unmarked script conflicts.

## Oracle

- [x] `skills/tailor/SKILL.md` names the tracked brief fallback.
- [x] `skills/tailor/SKILL.md` names the shared-script conflict prompt.
- [x] `skills/tailor/evals/cases/ignored-spellbook-brief.md` covers the
  regression.
