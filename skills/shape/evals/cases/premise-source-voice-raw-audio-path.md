# Context Packet: Raw Audio Path

Estimate: M

## Premise Source
Premise Source: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt

Voice Transcript Metadata:
- source_kind: voice
- source_hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71
- transcript_model: whisper-large-v3
- transcript_confidence: 0.82
- audio_duration_seconds: 321.5
- redaction_status: redacted
- redaction_tool: agent-transcript
- created_at: 2026-06-04T00:00:00Z
- residual_risk: Transcript accuracy is unverified and raw audio was intentionally retained.
- raw_audio_path: evidence/raw/meeting.m4a
- audio_retention_waiver: Raw audio was intentionally retained for review.

## Acceptance Evidence
- Acceptance source: invalid voice metadata checker fixture.
- Evidence that proves it: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test` rejects this packet.
- Exact command/path/route exercised: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test`.
- Oracle / acceptance artifact hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt.
- Contract-change acknowledgment: this fixture intentionally proves raw audio paths fail closed, even with waiver-like text.
- Residual risk: fixture uses extension heuristics, not media inspection.
