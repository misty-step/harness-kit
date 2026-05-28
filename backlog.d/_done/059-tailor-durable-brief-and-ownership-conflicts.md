# Tailor durable brief + ownership conflict contract

Priority: high
Status: fixed
Estimate: S

## Outcome

Superseded by Tailor retirement. `.harness-kit/repo-brief.md`, Tailor ownership
markers, and `skills/tailor/` were removed; bootstrap now exposes the canonical
skill catalog directly instead of generating repo-specific rewrite spines.

## Problem

`/tailor` required `.harness-kit/repo-brief.md` as the shared rewrite spine but
did not say what to do when a target repo ignored `.harness-kit/`. It also told
agents to copy shared scripts verbatim while forbidding overwrites of unmarked
scripts, leaving no explicit resolution path.

## Fix

- Write a tracked compatibility copy of the repo brief when `.harness-kit/` is
  ignored.
- Treat unmarked divergent shared scripts as ownership conflicts requiring
  `preserve / replace / diff`, not as silent self-audit failures.
- Add an eval case for ignored brief storage and unmarked script conflicts.

## Oracle

- [x] `skills/tailor/SKILL.md` names the tracked brief fallback.
- [x] `skills/tailor/SKILL.md` names the shared-script conflict prompt.
- [x] `skills/tailor/evals/cases/ignored-harness-kit-brief.md` covers the
  regression.
