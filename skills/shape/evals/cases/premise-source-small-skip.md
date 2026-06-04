# Context Packet: Small Shape Without Premise Source

Estimate: S

## PRD Summary
- User: Harness Kit operator.
- Problem: Tiny copy edits should not need source-artifact ceremony.
- Why now: The checker must not require premise sources for explicitly small shapes.
- UX enabled: Small maintenance stays light.
- Deliverable type: cleanup.
- Success signal: checker skips this fixture.

## Acceptance Evidence
- Acceptance source: checker fixture.
- Evidence that proves it: `bash skills/shape/evals/check-premise-source.sh` accepts this explicit small-shape packet without premise source.
- Exact command/path/route exercised: `bash skills/shape/evals/check-premise-source.sh`.
- Oracle / acceptance artifact hash: none; this fixture proves size-scoped behavior.
- Contract-change acknowledgment: this fixture intentionally preserves the non-goal of avoiding premise-source ceremony for small work.
- Residual risk: qualitative shape size still requires operator judgment outside fixtures.
