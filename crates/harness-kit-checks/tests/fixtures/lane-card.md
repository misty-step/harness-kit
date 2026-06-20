# Lane Card

Role: implementation critic
Objective: Review the focused lane harness projection diff for blocking gaps.
Provider target: codex
Model override: none
Scope: diff plus backlog oracle only.
Inputs / oracle: `backlog.d/_done/101-focused-lane-harness-projection.md` and
the changed Rust/tests.
Allowed skills: code-review, critique
Allowed tools: read, grep, git diff
Output shape: <=600 words: blocking findings, evidence, verdict.
Do not touch: no edits, no broad repo audit, no provider replacement policy.
Receipt expectation: record receipt id, accepted/rejected findings, and any
projection failure.
Lane harness: `crates/harness-kit-checks/tests/fixtures/lane-harness.yaml`
