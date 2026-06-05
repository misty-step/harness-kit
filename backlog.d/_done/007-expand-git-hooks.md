# Expand git hooks — harness-agnostic enforcement

Priority: medium
Status: done
Estimate: S

## Goal

Add git hooks that enforce quality at commit/push time, replacing Claude-specific hooks. Install via bootstrap.sh.

## Hooks to Add

### git-hooks/pre-commit
- Regenerate `index.yaml` (CLAUDE.md says pre-commit does this — verify and ensure it's actually installed)
- Stage the regenerated file so it's included in the commit

### git-hooks/pre-push (enhancement)
- Add branch protection: block pushes to main/master (replaces the Claude `harness-kit-checks claude-hook block-master-push` fallback)
- Keep existing Dagger gate check

## Non-Goals
- Don't add hooks that slow down commits significantly (keep pre-commit < 2s)
- Don't duplicate what Dagger already checks

## Oracle
- [x] `git commit` in harness-kit repo regenerates index.yaml automatically — .githooks/pre-commit
- [x] `git push` runs Dagger CI gates via pre-push hook — .githooks/pre-push
- [x] bootstrap.sh ensures core.hooksPath is set to .githooks/
- ~~`git push origin master` blocked~~ — dropped per user preference (not needed)

## What Was Built

Consolidated two hook directories into one:
- `.githooks/` (via `core.hooksPath`) is the single source of truth
- Moved pre-push (Dagger gates) from dead `git-hooks/` dir to `.githooks/`
- Removed `git-hooks/` directory (bootstrap was installing to `.git/hooks/` which git ignores when `core.hooksPath` is set)
- Updated bootstrap.sh to set `core.hooksPath` instead of symlinking
- Full hook inventory: pre-commit (index.yaml), post-commit (re-link), post-merge (bootstrap), post-rewrite (bootstrap), pre-push (Dagger gates)
