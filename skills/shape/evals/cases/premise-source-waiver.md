# Context Packet: Waived Premise Source

## Premise Source
Premise Source Waiver: Private raw transcript is unavailable because only a scoped redacted excerpt may be stored.
Residual risk: Future implementers cannot inspect the full conversation; they must rely on the shaped packet and redacted excerpt refs.

## Acceptance Evidence
- Acceptance source: checker fixture.
- Evidence that proves it: `bash skills/shape/evals/check-premise-source.sh` accepts this packet only because the waiver includes residual risk.
- Exact command/path/route exercised: `bash skills/shape/evals/check-premise-source.sh`.
- Oracle / acceptance artifact hash: none; this fixture proves waiver structure.
- Contract-change acknowledgment: this fixture intentionally models the premise-source waiver path.
- Residual risk: checker proves waiver shape, not whether the waiver is wise.
