Role: adversarial projection contract critic.
Objective: try to refute the backlog 101 implementation direction before code
is written.

Scope: read-only. Use backlog.d/101-focused-lane-harness-projection.md and
current Rust harness-check code. Focus on invariants that would embarrass us:
global skill leakage, real HOME mutation, receipt schema drift, and fake proof
that does not actually test skill visibility.

Output shape: <=45 lines with:
- blocking concerns, if any;
- required acceptance tests;
- non-blocking deferrals;
- verdict: implement / reshape / reject.

Do not edit files. Do not browse. Do not propose a semantic workflow engine.
