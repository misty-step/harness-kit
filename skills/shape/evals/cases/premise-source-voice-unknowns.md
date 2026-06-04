# Context Packet: Explicit Unknown Voice Metadata

Estimate: M

## Premise Source
Premise Source: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt

Voice Transcript Metadata:
- source_kind: raw_transcript
- source_hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71
- transcript_model: unknown
- transcript_confidence: unknown
- audio_duration_seconds: unknown
- redaction_status: sanitized
- redaction_tool: unknown
- created_at: 2026-06-04T00:00:00Z
- residual_risk: Transcript model and confidence are unknown, so accuracy is not proven.

## Acceptance Evidence
- Acceptance source: explicit unknown metadata checker fixture.
- Evidence that proves it: `bash skills/shape/evals/check-premise-source.sh` accepts this packet.
- Exact command/path/route exercised: `bash skills/shape/evals/check-premise-source.sh`.
- Oracle / acceptance artifact hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt.
- Contract-change acknowledgment: this fixture intentionally models explicit unknown metadata.
- Residual risk: fixture proves unknowns are explicit, not that the transcript is reliable.
