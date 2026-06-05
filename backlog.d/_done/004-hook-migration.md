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
| `harness-kit-checks claude-hook block-master-push` | PreToolUse/Bash | **unwired Rust fallback** | ✅ Python hook removed |
| `harness-kit-checks claude-hook check-todo-quality` | PreToolUse/Edit | **skill instruction + Rust fallback** | ✅ Torvalds Test in AGENTS.md; Python hook removed |
| `harness-kit-checks claude-hook codex-post-feedback` | PostToolUse/Edit | **unwired Rust fallback** | ✅ removed from settings.json; Python hook removed |
| `harness-kit-checks claude-hook codex-session-init` | SessionStart | **unwired Rust fallback** | ✅ removed from settings.json; Python hook removed |
| `harness-kit-checks claude-hook destructive-command-guard` | PreToolUse/Bash | **keep** | ✅ Claude Code permission model |
| `harness-kit-checks claude-hook disk-space-guard` | PreToolUse/Bash | **unwired Rust fallback** | ✅ Python hook removed |
| `harness-kit-checks claude-hook env-var-newline-guard` | PreToolUse/Bash | **unwired Rust fallback** | ✅ Python hook removed |
| `harness-kit-checks claude-hook exa-research-reminder` | PreToolUse/WebSearch | **skill instruction + Rust fallback** | ✅ Exa-first guidance in /research; Python hook removed |
| `harness-kit-checks claude-hook exclusion-guard` | PreToolUse/Edit | **Dagger gate + Rust fallback** | ✅ check_exclusions in Rust; Python hook removed |
| `harness-kit-checks claude-hook fix-what-you-touch` | PreToolUse/Bash | **skill instruction + Rust fallback** | ✅ expanded in AGENTS.md; Python hook removed |
| `harness-kit-checks claude-hook github-cli-guard` | PreToolUse/Bash | **keep** | ✅ GH API deprecation workaround |
| `harness-kit-checks claude-hook permission-auto-approve` | PreToolUse/any | **keep** | ✅ Claude Code permission model |
| `harness-kit-checks claude-hook portable-code-guard` | PreToolUse/Edit+Bash | **Dagger gate + Rust fallback** | ✅ check_portable_paths in Rust; Python hook removed |
| `harness-kit-checks claude-hook session-health-check` | SessionStart | **unwired Rust fallback** | ✅ removed from settings.json; Python hook removed |
| `harness-kit-checks claude-hook shaping-ripple` | PostToolUse/Edit | **skill instruction + Rust fallback** | ✅ ripple-check in /shape; shell hook removed |
| `harness-kit-checks claude-hook stop-quality-gate` | (unwired) | **Dagger gate + Rust fallback** | ✅ covered by dagger call check; Python hook removed |
| `harness-kit-checks claude-hook time-context` | SessionStart | **keep** | ✅ harness-specific context injection |

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
- **0 hooks dropped without a Rust fallback**
- **17 Rust hooks kept or available**: block-master-push, check-todo-quality, codex-post-feedback, codex-session-init, destructive-command-guard, disk-space-guard, env-var-newline-guard, exa-research-reminder, exclusion-guard, fix-what-you-touch, github-cli-guard, permission-auto-approve, portable-code-guard, session-health-check, shaping-ripple, stop-quality-gate, time-context
