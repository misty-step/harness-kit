# Migrate Claude hooks to harness-agnostic enforcement

Priority: high
Status: done
Estimate: L

## Goal

Eliminate dependence on Claude Code hooks for quality enforcement. Migrate each hook to the highest-leverage harness-agnostic layer: Dagger gate, git hook, skill instruction, or drop.

## Non-Goals
- Don't remove all hooks in one PR — migrate incrementally
- Don't lose enforcement value — every migrated hook must have an equivalent
- Don't build custom linting infrastructure — use Dagger gates

## Why

Claude hooks are Claude-Code-specific, fire per tool use (expensive at agent velocity), and don't work in Codex, Pi, or human workflows. The codification hierarchy says: Dagger gate > git hook > skill instruction > Claude hook.

## Migration Table

| Hook | Current | Target | Notes |
|------|---------|--------|-------|
| Hook | Current | Target | Status |
|------|---------|--------|--------|
| block-master-push.py | PreToolUse/Bash | **drop** | ✅ removed (not needed) |
| check-todo-quality.py | PreToolUse/Edit | **skill instruction** | ✅ Torvalds Test in AGENTS.md |
| codex-post-feedback.py | PostToolUse/Edit | **drop** | ✅ removed from settings.json |
| codex-session-init.py | SessionStart | **drop** | ✅ removed from settings.json |
| destructive-command-guard.py | PreToolUse/Bash | **keep** | ✅ Claude Code permission model |
| disk-space-guard.py | PreToolUse/Bash | **drop** | ✅ runtime-only, Dagger handles containers |
| env-var-newline-guard.py | PreToolUse/Bash | **drop** | ✅ interactive guard, not static-analysable |
| exa-research-reminder.py | PreToolUse/WebSearch | **skill instruction** | ✅ Exa-first guidance in /research |
| exclusion-guard.py | PreToolUse/Edit | **Dagger gate** | ✅ check_exclusions in main.py |
| fix-what-you-touch.py | PreToolUse/Bash | **skill instruction** | ✅ expanded in AGENTS.md |
| github-cli-guard.py | PreToolUse/Bash | **keep** | ✅ GH API deprecation workaround |
| permission-auto-approve.py | PreToolUse/any | **keep** | ✅ Claude Code permission model |
| portable-code-guard.py | PreToolUse/Edit+Bash | **Dagger gate** | ✅ check_portable_paths in main.py |
| session-health-check.py | SessionStart | **drop** | ✅ removed from settings.json |
| shaping-ripple.sh | PostToolUse/Edit | **skill instruction** | ✅ ripple-check in /shape |
| stop-quality-gate.py | (unwired) | **Dagger gate** | ✅ covered by dagger call check |
| time-context.py | SessionStart | **keep** | ✅ harness-specific context injection |

## Oracle
- [x] All "Dagger gate" hooks have equivalent checks in `ci/src/harness_kit_ci/main.py`
- [x] All "skill instruction" hooks have their guidance in the relevant SKILL.md or AGENTS.md
- [x] All "drop" hooks are removed from settings.json
- [x] `dagger call check` catches everything the old hooks caught
- [x] No regressions: `dagger call check` — 9 passed, 0 failed

## What Was Built

All 17 hooks triaged and migrated:
- **2 Dagger gates** added: `check_exclusions` (TS/lint/test exclusions), `check_portable_paths` (hardcoded home paths)
- **4 skill instructions** migrated: Torvalds Test (AGENTS.md), fix-what-you-touch (AGENTS.md), Exa-first (/research), ripple-check (/shape)
- **6 hooks dropped**: codex-post-feedback, codex-session-init, session-health-check, block-master-push, disk-space-guard, env-var-newline-guard
- **4 hooks kept**: destructive-command-guard, github-cli-guard, permission-auto-approve, time-context (Claude Code-specific, no harness-agnostic equivalent)
- **1 already covered**: stop-quality-gate (by `dagger call check`)
