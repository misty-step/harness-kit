# Context Packet: Valid Premise Source

## PRD Summary
- User: Harness Kit operator.
- Problem: Future implementers need the original premise.
- Why now: This fixture proves a valid local source path.
- UX enabled: The source can be inspected.
- Deliverable type: harness primitive.
- Success signal: premise-source checker passes.

## Premise Source
Premise Source: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt

## Acceptance Evidence
- Acceptance source: checker fixture.
- Evidence that proves it: self-test.
- Exact command/path/route exercised: `bash skills/shape/evals/check-premise-source.sh`.
- Oracle / acceptance artifact hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt.
- Contract-change acknowledgment: this fixture intentionally models the new premise-source contract.
- Residual risk: fixture only proves the checker behavior, not source quality.
