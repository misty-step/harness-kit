# Voice And Raw-Transcript Premise Metadata

Use this shape only when a `/shape` premise source is voice-derived or a raw
transcript excerpt. It preserves provenance and uncertainty without storing
raw audio or pretending transcript text is an ordinary hand-written note.

Place the block inside `## Premise Source`, after the `Premise Source:` line:

```markdown
Voice Transcript Metadata:
- source_kind: voice
- source_hash: sha256:<digest>
- transcript_model: unknown
- transcript_confidence: unknown
- audio_duration_seconds: unknown
- redaction_status: redacted
- redaction_tool: agent-transcript
- created_at: 2026-06-04T00:00:00Z
- residual_risk: Transcript accuracy is unverified.
```

## Fields

| Field | Required | Allowed values | Privacy class | Notes |
|---|---|---|---|---|
| `source_kind` | yes | `voice`, `raw_transcript` | internal | Use only for voice-derived or raw-transcript premise artifacts. |
| `source_hash` | yes | `sha256:<64 hex>` | internal | Must match the `Premise Source:` digest. |
| `transcript_model` | yes | model id or `unknown` | internal | Unknown is acceptable only when explicit. |
| `transcript_confidence` | yes | `0..1` or `unknown` | sensitive | This is metadata, not accuracy proof. |
| `audio_duration_seconds` | yes | non-negative number or `unknown` | sensitive | Do not retain raw audio just to compute this. |
| `redaction_status` | yes | `redacted`, `sanitized` | internal | Raw/unredacted transcript text is not acceptable. |
| `redaction_tool` | yes | tool name or `unknown` | internal | Redact private transcript excerpts before inclusion; never embed raw logs. |
| `created_at` | yes | ISO-8601 timestamp, not future | internal | Timestamp for the transcript/premise artifact metadata. |
| `residual_risk` | yes | substantive text | internal | Name uncertainty, especially transcript accuracy and omitted context. |

## Rules

- Missing metadata fields fail. Unknown model, confidence, or duration must be
  spelled `unknown`; silence is not allowed.
- Raw audio paths such as `.wav`, `.mp3`, `.m4a`, `.flac`, `.aac`, `.aiff`,
  or `.ogg` fail closed. This ticket does not permit raw audio retention in the
  repo, even with a waiver.
- `source_hash` must match the digest in the `Premise Source:` line.
- The block proves provenance and uncertainty only. It does not prove transcript
  accuracy, speaker identity, consent, or completeness.
