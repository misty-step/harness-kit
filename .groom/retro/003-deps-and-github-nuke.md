# Retro: /deps skill + GitHub Issues nuke (2026-03-28)

## What went well

- **/autopilot pipeline worked end-to-end.** (Historical note: `/autopilot` was renamed to `/deliver` in 032.) Planner produced a complete context packet,
  builder shipped in one pass, 5-reviewer bench caught real issues (missing Phase 0
  baseline, over-decomposed sub-agents, dead weight coordination table, procedural
  grep patterns). 2 Don't Ship verdicts led to concrete fixes. This validates the
  planner → builder → critic pipeline design.

- **Code review convergence.** 3 of 5 reviewers independently flagged the same issue
  (sub-agent over-decomposition). Cross-reviewer convergence is the strongest signal.

## Operational learnings: bulk GitHub Issues cleanup

- `gh issue close --reason "not planned"` silently fails on some repos. The flag
  isn't universally supported. Use `gh issue close` without `--reason` for reliability.
- **Rate limiting:** GraphQL limit is 5000 points/hour. `gh issue close` costs ~6 points
  each. At ~800 closes you'll exhaust the budget. Throttle with `sleep 0.2` between calls.
- `open_issues_count` in the GitHub REST API **includes PRs**, not just issues. Don't
  use it as verification — use `gh issue list --state open` instead.
- **Archived repos are read-only.** Can't close issues without unarchiving first. 50
  archived repos across legacy personal + misty-step ownership still have open issues.
- Initial repo scan used `isArchived == false` filter, missing archived repos. Scan
  ALL repos first, then filter by archived status.

## What was codified

- 8 files updated to fix fallback chains and bridge descriptions (see commit)
- This retro file for `/groom` context loading
