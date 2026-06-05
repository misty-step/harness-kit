# Context Packet: Missing Premise Source

Estimate: M

## PRD Summary
- User: Harness Kit operator.
- Problem: This fixture intentionally omits the premise source block.
- Why now: The checker must reject missing premise sources for M+ packets.
- UX enabled: none.
- Deliverable type: harness primitive.
- Success signal: checker rejects this fixture.

## Acceptance Evidence
- Acceptance source: checker fixture.
- Evidence that proves it: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test` rejects this packet.
- Exact command/path/route exercised: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test`.
- Oracle / acceptance artifact hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt
- Contract-change acknowledgment: this fixture intentionally proves acceptance evidence cannot replace premise source.
- Residual risk: fixture only proves the checker behavior, not source quality.
