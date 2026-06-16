# Skill-market prior-art lane

Provider target: `pi`
Delegation ID: `59a64ed9-6e1c-4c4d-8475-075871ae1465`
Runtime receipt: `.harness-kit/traces/provider-lanes/20260616T204302.618011Z-pi-2c43ea82.txt`

## Accepted Evidence

- Ponytail fits as an external synced skill rather than a first-party rewrite:
  portable skill folder, MIT license, and a concrete YAGNI/stdlib/native ladder.
- Start with only the core `ponytail` skill; defer `ponytail-review`,
  `ponytail-audit`, `ponytail-debt`, and `ponytail-help` until telemetry or a
  probe proves they add value beyond existing Harness Kit review/refactor
  skills.
- The top-skills leaderboard should become a dry-run scanner/report, not a
  bulk registry import. Popularity is a discovery signal, not a fit oracle.

## Resulting Packets

- `backlog.d/106-ponytail-simplicity-skill.md`
- `backlog.d/107-agent-skill-market-scout.md`

