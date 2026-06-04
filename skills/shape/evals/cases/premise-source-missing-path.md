# Context Packet: Missing Premise Path

Estimate: M

## Premise Source
Premise Source: sha256:0000000000000000000000000000000000000000000000000000000000000000 skills/shape/evals/cases/does-not-exist.txt

## Acceptance Evidence
- Acceptance source: checker fixture.
- Evidence that proves it: `bash skills/shape/evals/check-premise-source.sh` rejects this packet.
- Exact command/path/route exercised: `bash skills/shape/evals/check-premise-source.sh`.
- Oracle / acceptance artifact hash: none; this invalid fixture intentionally references a missing source path.
- Contract-change acknowledgment: this fixture intentionally models missing-path rejection.
- Residual risk: fixture only proves the checker behavior, not source quality.
