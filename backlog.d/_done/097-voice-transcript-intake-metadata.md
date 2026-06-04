# Voice and raw-transcript intake metadata

Priority: P2
Status: shaped
Estimate: S

## Goal

Standardize the minimal metadata for voice-derived or raw-transcript premise
artifacts so `/shape`, `/reflect`, and transcript mining can consume them
without pretending they are ordinary hand-written notes.

## Source Evidence

- Pasted article hash:
  `sha256:ff278d4ed3965ca36f2eb50dbac6712afee2ea6b3060a5a119cd39825198139c`
  `/Users/phaedrus/.codex/attachments/57bc8d3e-9224-4126-b3ec-298e9fe1cb15/pasted-text.txt`.
- Article themes: voice-to-agent is now practical; raw Granola transcripts and
  note tools become high-value agent context; do not summarize prematurely.
- Pi critic recommendation: keep the first voice slice to metadata/schema
  extension, not a voice management system.

## Non-Goals

- Do not build audio transcription, recording, or meeting-note integrations.
- Do not store raw audio in the repo.
- Do not require one transcription provider.
- Do not mine private transcripts by default.
- Do not create a new voice skill before proving the metadata contract.

## Constraints / Invariants

- Artifacts are local-first and opt-in.
- Store refs and sanitized text, not raw private transcripts or audio.
- Metadata must make uncertainty visible: transcript confidence/model/source
  may be unknown, but must not be silently omitted.
- This ticket should compose with 095; it should not block 095.

## Authority Order

redacted artifact > metadata fields > source hash > agent summary > lore

## Repo Anchors

- `backlog.d/095-shape-premise-source-artifact.md` - premise artifact contract
  this can feed.
- `skills/agent-transcript/SKILL.md` - safe transcript excerpt rendering.
- `skills/trace/SKILL.md` - refs and waiver reasons instead of raw transcripts.
- `backlog.d/091-transcript-mining-effectiveness-loop.md` - later mining loop.
- `harnesses/codex/config.toml` - current Codex harness has a
  `voice_transcription` config line, but no repo contract for voice artifacts.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Ignore voice | Treat all premise artifacts as text | No new surface | Loses source/confidence/provenance for voice inputs | Reject |
| Full voice-intake script | Normalize audio/transcript into markdown | Useful workflow | Premature provider/tool assumption | Defer |
| Metadata-only contract | Define fields for voice/raw transcript artifacts | Small and composable | Requires later scripts to use it | Choose |
| Meeting-service CLI | Pull Granola/Bear/etc. directly | Convenient | External auth and provider sprawl | Reject |
| Raw audio storage | Keep original audio for full fidelity | Maximum audit | Privacy and repo bloat risk | Reject |
| Transcript-mining only | Wait for 091 to handle everything | Avoids duplicate code | Premise artifacts need metadata before mining | Reject |

## Agent Readiness

- Profile source: not applicable.
- Stack feedback strength: medium; fixture markdown/JSON validation is enough.
- ADR decision: not required.
- Infrastructure path: schema/reference first, optional script later.
- Gate: metadata fixture self-test, `python3 scripts/check-agent-roster.py`,
  then `dagger call check --source=.`
- Evidence storage: fixture artifacts under `skills/agent-transcript/evals/` or
  `skills/shape/evals/`.
- Mock policy impact: preserved; fixtures are local text artifacts.

## Delegation Evidence

- Roster providers used:
  - `claude` repo investigator, receipt
    `c5a1708e-e046-4590-8141-1d08412317a5`.
  - `pi` premise critic, receipt `fe9a2a9a-c48e-41ce-a4a2-9feea8338884`.
  - `codex` oracle critic, receipt `2920ae5b-d21c-46a6-9202-0861490134fa`.
- Accepted evidence: Pi's recommendation to avoid a broad voice workflow and
  start with optional metadata fields; Claude's voice-intake idea is narrowed
  to a schema/reference slice.
- Rejected evidence: remote-control automation, always-on email/phone launch,
  and provider-specific meeting transcript integrations.
- Waivers: no provider-specific voice/transcription docs were researched; this
  is a provider-agnostic metadata shape.

## Oracle

- [ ] A reference defines the minimum metadata for voice/raw-transcript premise
      artifacts: `source_kind`, `source_hash`, `transcript_model`,
      `transcript_confidence`, `audio_duration_seconds`, `redaction_status`,
      `redaction_tool`, `created_at`, and `residual_risk`.
- [ ] A fixture with `source_kind: voice` and no `source_hash` fails.
- [ ] A fixture with raw audio path inside the repo fails or requires an
      explicit waiver stating the audio is intentionally retained.
- [ ] A fixture with unknown model/confidence passes only when the fields are
      explicitly set to `unknown`.
- [ ] 095's premise source guidance references this metadata shape as the path
      for voice-derived premise artifacts.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Acceptance Evidence

- Acceptance source: metadata fixture artifacts.
- Evidence that proves it: checker accepts explicit unknowns and rejects missing
  source hash/raw-audio cases.
- Exact command/path/route exercised: implementation should add a small
  deterministic self-test command, such as
  `bash skills/agent-transcript/evals/check-intake-metadata.sh`.
- Oracle / acceptance artifact hash:
  `sha256:ff278d4ed3965ca36f2eb50dbac6712afee2ea6b3060a5a119cd39825198139c`
  `/Users/phaedrus/.codex/attachments/57bc8d3e-9224-4126-b3ec-298e9fe1cb15/pasted-text.txt`.
- Contract-change acknowledgment: no existing contract changes until a
  reference/checker is added.
- Residual risk: metadata does not prove transcript accuracy; it only prevents
  silent provenance loss.

## Observability Plan

- Changed behavior to watch: voice/raw-transcript premise artifacts carry
  provenance and uncertainty before shape/reflect/mining consume them.
- Named signal or evidence surface: metadata checker output.
- Instrumentation debt: no cross-session aggregation until 091 lands.

## Implementation Sequence

1. Add the metadata reference under the smallest owning skill surface.
2. Add valid/invalid fixture artifacts.
3. Add a checker/self-test for required fields, explicit unknowns, and raw audio
   retention.
4. Link the reference from 095's `/shape` premise-source guidance.
5. Run the self-test, roster check, and Dagger gate.

## Risk + Rollout

- Risk: scope expands into provider integrations. Mitigate by forbidding
  transcription/integration code in this ticket.
- Risk: metadata looks like accuracy proof. Mitigate by naming confidence and
  residual risk explicitly.
- Rollback: remove the reference/checker; premise-source artifacts still work
  as plain text.
