# Context Packet: Missing Premise Path

Estimate: M

## Premise Source
Premise Source: sha256:0000000000000000000000000000000000000000000000000000000000000000 skills/shape/evals/cases/does-not-exist.txt

## Acceptance Evidence
- Acceptance source: checker fixture.
- Evidence that proves it: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test` rejects this packet.
- Exact command/path/route exercised: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test`.
- Oracle / acceptance artifact hash: none; this invalid fixture intentionally references a missing source path.
- Contract-change acknowledgment: this fixture intentionally models missing-path rejection.
- Residual risk: fixture only proves the checker behavior, not source quality.
