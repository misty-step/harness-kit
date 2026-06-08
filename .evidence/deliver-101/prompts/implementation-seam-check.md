Role: implementation seam reviewer.
Objective: inspect the current Harness Kit repo and identify the smallest
Rust implementation plan for backlog 101.

Scope: read-only. Use backlog.d/101-focused-lane-harness-projection.md and
live files under crates/harness-kit-checks/src plus .harness-kit/examples.

Output shape: <=40 lines with:
- exact modules/functions to touch;
- tests that should fail first;
- one implementation risk;
- one simplification to avoid a semantic workflow engine.

Do not edit files. Do not browse. Do not suggest provider ranking, retry
orchestration, or global harness mutation.
