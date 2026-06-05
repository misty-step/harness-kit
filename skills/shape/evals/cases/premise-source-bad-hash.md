# Context Packet: Bad Premise Hash

Estimate: M

## Premise Source
Premise Source: sha256:0000000000000000000000000000000000000000000000000000000000000000 skills/shape/evals/cases/premise-source-valid-source.txt

## Acceptance Evidence
- Acceptance source: checker fixture.
- Evidence that proves it: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test` rejects this packet.
- Exact command/path/route exercised: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test`.
- Oracle / acceptance artifact hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt.
- Contract-change acknowledgment: this fixture intentionally models stale-digest rejection.
- Residual risk: fixture only proves the checker behavior, not source quality.
