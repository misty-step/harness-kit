# git-bug: distributed git-native issue tracking + agent coordination

Priority: high
Status: done
Estimate: M

## Goal
Issues live in git as objects, not in GitHub's database. Agents read/write issues via CLI.
GitHub Issues becomes a read-only bridge for human visibility. Agents coordinate via
atomic claims so two agents never work the same issue.

## Non-Goals
- Don't migrate all historical GitHub Issues
- Don't build a custom issue tracker
- Don't break existing backlog.d/ pattern (git-bug complements it)

## Oracle
- [x] `git bug` CLI installed and configured
- [x] Bridge to GitHub configured (git-bug push/pull syncs with GitHub Issues)
- [x] Agents can create, query, and close issues via `git bug` commands
- [x] Claim protocol works: `git update-ref refs/claims/<id>` atomic CAS
- [x] `/autopilot` claims item before spawning builder; `/groom` skips claimed items
- [x] New `/investigate` findings auto-create git-bug issues
- [x] Issues travel with repo clone (no API calls needed to read)

## What Was Built
- `scripts/setup-git-bug.sh` — Idempotent setup: install via brew, configure user identity
  from git config, configure GitHub bridge with token from `gh auth token`. Safe to re-run.
- `scripts/lib/claims.sh` — Atomic claim protocol: `claim_acquire`, `claim_release`,
  `claim_check`, `claim_owner`, `claim_audit`. Uses `git update-ref refs/claims/<id>` CAS.
- `skills/autopilot/SKILL.md` — Step 1 (Pick work) now acquires atomic claim; Step 9 (Ship)
  and Stopping Conditions release claim. Also reads git-bug issues alongside backlog.d/.
- `skills/autopilot/references/issue.md` — `/issue lint/enrich/decompose` now support git-bug
  issues alongside GitHub Issues. Routing: hex prefix → git-bug, `#N` → GitHub.
- `skills/groom/SKILL.md` — Context Loading reads git-bug issues, filters claimed items.
  WRITE phase creates git-bug issues for raw findings. Tidy audits claims (stale >24h).
- `skills/groom/references/backlog-doctrine.md` — Three-tier backlog: `backlog.d/` (shaped),
  git-bug (raw issues), `.groom/BACKLOG.md` (icebox). GitHub is read-only bridge.
- `skills/groom/references/git-bug-conventions.md` — Label taxonomy, query recipes,
  lifecycle commands, claim protocol reference.
- `skills/investigate/references/log-issues.md` — Creates git-bug issues (preferred) with
  GitHub Issues fallback. Includes sync step (`git-bug push origin`).
- `harnesses/claude/settings.json` — Added `git-bug` and `git bug` to permission allowlist.
- `CLAUDE.md` — Added Issue Tracking section documenting git-bug usage.

## Workarounds
- git-bug requires initial setup per clone: `bash scripts/setup-git-bug.sh`. The setup
  script is idempotent but must be run once.
- All skills gracefully degrade when git-bug is absent — fall back to `gh issue` or
  `backlog.d/` patterns.
- Claims are local (atomic within one `.git`). For multi-machine coordination, push claims
  to origin: `git push origin refs/claims/<id>`. Not implemented yet.

## Notes
- git-bug stores issues as git objects (not files), with Lamport timestamps for CRDT merging
- git-bug has NO assignee concept — use labels (`claimed:<agent-id>`) + `refs/claims/` for coordination
- Bridges to GitHub/GitLab for human visibility
- Offline-first — works in CI, sandboxes, locally without network
- backlog.d/ = shaped work ready to build. git-bug = raw issues, bugs, requests
- Coordination: `git update-ref refs/claims/<id> $HASH ""` is atomic CAS within shared .git.
  All worktrees see claims instantly. For multi-machine: push-gate the ref.
- Research: https://github.com/git-bug/git-bug
- 2026-03-28: GitHub Issues cleared across all active legacy personal + misty-step repos
  (~1,630 issues closed). backlog.d/ is now the sole active backlog surface.
  Bridge still functional for optional sync. 50 archived repos untouched (read-only).

## 2026-04-22 — retirement

git-bug was retired because the tooling was installed but never seeded — zero
`refs/bugs/*` across spellbook and all downstream repos. The CLI shipped, the
adoption loop didn't. `backlog.d/` is now the single-tier source of truth for
shaped work.

Associated primitives deleted:
- `scripts/setup-git-bug.sh`
References purged:
- `/diagnose` (`skills/diagnose/references/log-issues.md` — dropped git-bug
  reading/creation/sync paths, `backlog.d/` is now the only issue surface)
- `/deliver` (`skills/deliver/SKILL.md` — pick-source no longer mentions
  git-bug; `backlog.d/` highest-priority is the sole default)
Harness permissions:
- `harnesses/claude/settings.json` — removed `Bash(git-bug:*)` and
  `Bash(git bug:*)` entries

Note: `/groom`, `/settle`, and other surfaces still carrying git-bug references
will be cleaned up by in-flight rewrites on separate tracks.

If distributed issue tracking is revisited later, consider an MCP-server-backed
alternative rather than a CLI-only tool — the adoption failure here was that a
CLI requires explicit seeding and per-clone setup, whereas an MCP surface would
be ambient to every agent session.
