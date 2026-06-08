Role: fresh-context verifier / release-risk critic.
Objective: Verify that the current diff for backlog 101 implements focused lane harness projection without introducing a semantic workflow engine.

Scope:
- Read only. Do not edit files.
- Inspect the current git diff, backlog.d/101-focused-lane-harness-projection.md, .harness-kit/examples/lane-harness.yaml, and the Rust tests touching lane_harness, agent_roster, summarize_delegations, and check_agent_roster.
- Treat the acceptance oracle as: the primary can materialize a bespoke projected harness for one provider lane, only explicitly allowed skills are visible, broad globally installed skills do not leak into that lane, provider/model mismatches are rejected before dispatch, provider failures such as credits/auth/missing binary/timeouts are classified in receipts, and no provider CLI wrapper becomes a semantic workflow engine.

Output shape, <=45 lines:
1. Verdict: pass or block.
2. Blocking gaps, if any, with exact file paths and line references.
3. Non-blocking risks or follow-ups.
4. One sentence on whether this stays harness-engineering-simple.

Do not summarize the author's intent. Refute the diff against the oracle.
