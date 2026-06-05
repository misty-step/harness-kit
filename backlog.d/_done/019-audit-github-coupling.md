# Audit remaining GitHub coupling

Priority: high
Status: done
Estimate: S

## Goal

Identify every remaining dependency on GitHub (API, Actions, PR state, releases,
`gh` CLI calls) across the entire Harness Kit codebase and downstream skills.
Produce a concrete removal plan for each dependency.

## Why

Thinktank research revealed disagreement about how much GitHub coupling remains.
The `ci/` module may still reference `GITHUB_ACTIONS` and `GITHUB_STEP_SUMMARY`
env vars. Skills like `/settle` and `/demo` may still call `gh` for PR checks
and evidence upload. Until we know the full surface, we can't plan the removal.

## Audit Scope

- `ci/` Dagger module — env var assumptions, GitHub API calls
- All skills — `gh` CLI usage, GitHub URL construction, PR references
- `bootstrap.sh` — GitHub release download path
- Hooks — any GitHub-specific logic
- CLAUDE.md / AGENTS.md — GitHub-assuming instructions

## Oracle

- [x] Grep report: every file referencing `github`, `gh `, `GITHUB_`, `pulls/`, `issues/`
- [x] Each reference categorized: required (keep) vs removable (plan)
- [x] Removal plan filed as backlog items or amendments to existing items

## Non-Goals

- Actually removing anything — this is audit only
- Removing GitHub as a git remote (keep as mirror)

## What Was Built

Full audit completed 2026-04-06 via three parallel agents scanning ci/, skills/,
scripts/, harnesses/, bootstrap.sh, hooks, agents, and documentation.

### Findings Summary

**ci/ Dagger module: CLEAN.** Zero GitHub coupling. No env vars, no API calls,
no `gh` CLI, no `.github/` assumptions. Fully vendor-agnostic.

**Skills: Moderate coupling in 3 areas:**

| Area | Files | Classification | Removal Path |
|------|-------|---------------|--------------|
| **PR review aggregation** | `settle/scripts/fetch-pr-reviews.sh` | required (GitHub mode) | 021: `/settle` git-native mode uses verdict refs instead |
| **Evidence upload** | `demo/references/pr-evidence-upload.md`, `demo/SKILL.md` | required (GitHub mode) | 024: git-native evidence storage |
| **Merge-readiness check** | `settle/SKILL.md:82` (`gh pr view --json`) | required (GitHub mode) | 021: verdict refs replace PR status checks |
| **Issue fallback** | `autopilot/references/issue.md` | fallback | Already lowest priority in chain (git-bug > backlog.d/ > gh) |

**Infrastructure: 2 removable couplings:**

| Area | Files | Classification | Removal Path |
|------|-------|---------------|--------------|
| **GitHub-specific URL parsing** | `bootstrap.sh:42-43`, `scripts/setup-git-bug.sh:42-43` | removable | Parse any git remote, not just github.com |
| **GitHub API for remote sync** | `bootstrap.sh:286,293` (`curl api.github.com`) | required (remote mode) | Shallow clone fallback instead of API listing |
| **GitHub CLI guard hook** | `harness-kit-checks claude-hook github-cli-guard` | fallback | Remove when `gh` usage is eliminated |

**Documentation-only references (no action needed):**
- `settle/references/pr-fix.md` — `gh run view/rerun` examples
- `investigate/references/triage.md` — CI investigation examples
- `agent-readiness/references/pillar-checks.md` — example check
- `research/references/exemplars.md` — example URLs
- `CLAUDE.md` — git-bug push description (optional sync)
- `ci/sdk/` generated docstrings — `.github/` path examples

### Coupling Map

```
                    ┌─────────────────────────────────┐
                    │         GitHub Coupling          │
                    └─────────────────────────────────┘
                                   │
          ┌────────────────────────┼────────────────────────┐
          │                        │                        │
    ┌─────▼──────┐          ┌─────▼──────┐          ┌─────▼──────┐
    │  CLEAN ✓   │          │  MODERATE   │          │  REMOVABLE │
    │  ci/ module │          │  Skills     │          │  Infra     │
    │  agents/    │          │             │          │            │
    │  .githooks/ │          │  /settle    │          │  bootstrap │
    │  harnesses/ │          │  /demo      │          │  setup-    │
    │  (configs)  │          │  /autopilot │          │  git-bug   │
    └────────────┘          │  (fallback) │          │  gh-guard  │
                            └─────────────┘          └────────────┘
                                   │                        │
                            ┌──────┴──────┐          ┌──────┴──────┐
                            │ Tracked by: │          │ Tracked by: │
                            │ 021-settle  │          │ new item or │
                            │ 024-evidence│          │ inline fix  │
                            └─────────────┘          └─────────────┘
```

### Removal Plan

All GitHub coupling is already tracked by existing backlog items:

1. **021-settle-git-native** — `/settle` learns dual-mode (GitHub + git-native).
   Covers `fetch-pr-reviews.sh` and `gh pr view` usage.
2. **024-offline-evidence-storage** — `/demo` stores evidence in Git, not
   GitHub releases. Covers `pr-evidence-upload.md`.
3. **Bootstrap URL parsing** — Minor inline fix. Change `sed` regex to parse
   any git remote URL, not just github.com. Can be done as part of any
   bootstrap touch.
4. **Bootstrap remote sync** — Replace `curl api.github.com` with shallow
   clone for skill discovery. Can be done as part of any bootstrap touch.
5. **`harness-kit-checks claude-hook github-cli-guard`** — Remove once `gh` CLI usage is eliminated from
   skill workflows (after 021 + 024 complete).

### Key Insight

GitHub coupling is **narrower than feared**. The ci/ module is completely clean.
The coupling lives in exactly two skill workflows (`/settle` PR review flow and
`/demo` evidence upload) plus two `sed` regexes in bootstrap scripts. Both skill
workflows are already tracked for git-native evolution (021, 024). The infra
fixes are trivial inline changes.
