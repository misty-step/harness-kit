# /reflect cycle

Invoked by `/flywheel` (or re-run manually) after a cycle closes. Read
what actually happened — commits, diff, leaf receipts, backlog changes —
and extract learnings worth keeping.

## What reflect owns that nothing else does

1. **Backlog mutations beyond a single item** — create, consolidate,
   delete, reprioritize. Cite cycle evidence for each.
2. **Harness-edit suggestions** for `/harness-engineering apply` — never in
   place. Skills, agents, hooks, AGENTS.md, CLAUDE.md are all in scope.

## When to escalate to a harness branch

Harness apply branches are expensive — humans must review. Raise something when:

- The same failure appears in ≥2 cycles.
- A documented contract was followed and still produced wrong output
  (the contract is wrong, not the agent).
- An existing rule was ambiguous enough to be bypassed rationally.

Otherwise leave a finding in the retro and move on.

## Judgment

- **No speculation.** If you can't point at a commit, diff, receipt, or
  log line, drop the finding.
- **Consolidate** when two items share root cause and target.
  **Split** when one accreted two goals mid-cycle.
  **Delete** only when evidence proves obsolescence — "looks stale" is
  not evidence; reprioritize to low if unsure.
- **Never edit a skill that was used in the cycle being reflected on.**
  Wait for full close before changing the tool you just used.
- **Self-reflection is out of scope.** File a backlog item if reflect
  itself has a gap.

## Gotchas

- Harness branches live until a human merges or deletes them — no
  auto-gc.
- Re-running on the same cycle is fine; skip mutations already applied
  by diffing against git history.
