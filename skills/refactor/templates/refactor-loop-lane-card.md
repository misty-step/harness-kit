# Refactor-Loop Lane Card

Role: bounded refactor tick worker
Objective: improve one named architecture pressure in one repo, then stop with proof
Provider target: <native | codex | claude | pi | opencode | goose | sprite>
Model override: none
Scope: <repo slug>, <absolute worktree/path>, <subsystem/files in bounds>
Inputs / oracle:
- Registry record: <path>#<slug>
- Proof contract: <path>#<slug>
- Architecture goal: <one responsibility/coupling/duplication pressure>
- Fitness tests: <canonical gate plus focused commands/routes>
- Stop rule: <what evidence means one tick is enough>
Allowed skills:
- refactor
- qa
- code-review
Allowed tools:
- git, repo-local test/build tools, browser/API/CLI route needed by the fitness test
Output shape:
- progress file update at <progress_file>
- evidence packet path with commands/routes, artifacts, reviewer findings, residual risk
- diff summary and exact verification output
Do not touch:
- unrelated dirty files
- production/external side effects unless explicitly approved
- Mode B scheduler/runner code
Stop conditions:
- one meaningful milestone is green and reviewed
- same failure/no-progress twice
- canonical gate or focused route fails without a small obvious fix
- required repo is dirty outside scope
- work requires product behavior or Mode B orchestration changes
Receipt expectation:
- lane receipt or delegation receipt
- evidence packet linked from closeout
Lane harness: none

## Launch

```sh
cargo run --locked -p harness-kit-checks -- dispatch-agent \
  --provider-target <provider> \
  --objective "Refactor <repo>/<subsystem> so <architecture property>; stop after one green milestone" \
  --input-ref "<registry-path>#<repo-slug>" \
  --prompt-file skills/refactor/templates/refactor-loop-lane-card.md \
  --repo <target-repo> \
  --backlog-ref <work-ref>
```

Remove `--provider-target` details that the active harness does not support.
For unattended schedules, do not launch from Harness Kit; create a Mode B handoff
packet using `harnesses/shared/references/loop-readiness.md`.
