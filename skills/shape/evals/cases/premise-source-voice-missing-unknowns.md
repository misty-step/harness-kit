# Context Packet: Missing Unknown Voice Fields

Estimate: M

## Premise Source
Premise Source: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt

Voice Transcript Metadata:
- source_kind: voice
- source_hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71
- audio_duration_seconds: unknown
- redaction_status: redacted
- redaction_tool: agent-transcript
- created_at: 2026-06-04T00:00:00Z
- residual_risk: Transcript accuracy is unverified and model metadata was omitted.

## Acceptance Evidence
- Acceptance source: invalid voice metadata checker fixture.
- Evidence that proves it: `bash skills/shape/evals/check-premise-source.sh` rejects this packet.
- Exact command/path/route exercised: `bash skills/shape/evals/check-premise-source.sh`.
- Oracle / acceptance artifact hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt.
- Contract-change acknowledgment: this fixture intentionally models omitted model/confidence rejection.
- Residual risk: fixture proves checker behavior only.
