# Cross-harness skill invocation telemetry

Priority: P2
Status: ready
Estimate: M

## Goal

Make skill invocation analytics cross-harness instead of Claude-only by adding
a shared event schema and adapters for Codex, Pi, Antigravity, and any harness
that can emit equivalent tool/skill events.

## Why Now

The existing tracker is passive and well-shaped but lives only in
`harnesses/claude/hooks/skill-invocation-tracker.py` and writes
`~/.claude/skill-invocations.jsonl`. That means any report over skill usage is
structurally biased toward Claude sessions. Harness Kit's doctrine is
cross-harness first; analytics must follow the same rule.

This ticket depends on or follows `088`, because the analyzer should already
label source coverage and missing harnesses before expanding collection.

## Non-Goals

- Do not force every harness to support identical hooks immediately.
- Do not add repo-local checked-in bridge directories to the source repo.
- Do not store raw prompts or tool outputs.
- Do not block agent execution if telemetry fails.
- Do not infer a skill invocation from natural-language text.

## Constraints / Invariants

- Event schema is shared; adapters are harness-specific and thin.
- Telemetry is best-effort, passive, no stdout, exit 0 on malformed input.
- Every row includes `harness` and `source_protocol`.
- Optional fields stay optional.
- The source repo does not commit generated `.codex/skills`, `.claude/skills`,
  `.pi/skills`, or `.antigravitycli/skills` bridges.

## Repo Anchors

- `harnesses/claude/hooks/skill-invocation-tracker.py` - implementation model.
- `harnesses/claude/hooks/test_skill_invocation_tracker.py` - test model.
- `harnesses/codex/`, `harnesses/pi/`, `harnesses/antigravity-cli/` - harness
  projection surfaces to inspect before adding adapters.
- `bootstrap.sh` - source of projected harness configuration.
- `AGENTS.md` - source-repo red line against committed bridge skill copies.
- `scripts/check-agent-roster.py` - likely place for adapter fixture validation.

## Prior Art

- Tessl docs say `tessl init` configures multiple coding agents for MCP support
  and that skills activate automatically when task descriptions match. The
  applicable lesson is cross-agent installation and activation visibility:
  https://docs.tessl.io/introduction-to-tessl/quickstart-skills-docs-rules.
- Langfuse exposes CLI and MCP access for agents and recommends allow-listing
  read-only tools when agents inspect observability data:
  https://langfuse.com/agents.
- OpenTelemetry's GenAI conventions include agent spans and model spans, which
  are useful naming prior art for eventual cross-harness event mapping:
  https://opentelemetry.io/docs/specs/semconv/gen-ai/.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Shared JSONL schema plus adapters | One schema, per-harness thin emitters | Cross-harness and local-first | Some harnesses may lack hooks | Choose |
| Claude-only analytics with coverage label | Analyze what exists | Quick | Misleading for cross-harness operator behavior | Interim only |
| Central wrapper around all provider CLIs | Capture every run centrally | Complete capture | Semantic workflow engine risk | Reject |
| MCP proxy for all skills | Instrument skill activation centrally | Theoretically uniform | Requires runtime everyone must use | Reject |
| OpenTelemetry SDK in each harness | Standard span export | Interoperable | Heavy for current local CLI hooks | Defer |

## Proposed Shape

Define a shared skill invocation event schema in a small reference or fixture:

```json
{
  "schema_version": 2,
  "event_type": "skill_invocation",
  "ts": "2026-06-03T16:00:00Z",
  "harness": "claude",
  "source_protocol": "post_tool_use",
  "skill": "code-review",
  "session_id": "abc",
  "repo": "harness-kit",
  "cwd": "/repo",
  "backlog_ref": "088",
  "work_id": "work-...",
  "usage": null
}
```

Implementation sequence:

1. Add schema fixture and update the Claude tracker to emit `harness:
   "claude"` and `source_protocol: "post_tool_use"` while preserving existing
   rows.
2. Inspect Codex and Pi harness configuration to determine whether hook/event
   adapters are currently possible.
3. Add only adapters with executable local smoke tests. For unsupported
   harnesses, add explicit coverage warnings in the analyzer from `088`.
4. Update bootstrap/projection only if the adapter has a verified target path.

## Agent Readiness

- Profile source: no dedicated profile; use existing harness projection tests.
- Stack feedback strength: Python hook unit tests plus bootstrap/check smoke.
- ADR decision: not required unless introducing a new hook protocol.
- Infrastructure path: hook scripts under `harnesses/<provider>/`.
- Gate: hook unit tests, `python3 scripts/check-agent-roster.py`, and
  `dagger call check --source=.`
- Evidence storage: `.harness-kit/examples/skill-invocations.jsonl` and
  harness-specific test fixtures.
- Mock policy impact: preserved; tests feed representative hook JSON.

## Delegation Evidence

- Roster providers used:
  - `claude`, receipt `c1bc871f-4122-4786-a2df-e04e62a03c91`, identified the
    Claude-only tracker blind spot.
  - `codex`, receipt `004cb27a-ff40-4918-9ed7-40478b196a7f`, warned against a
    central wrapper or hosted telemetry prerequisite.
- Native/Thinktank evidence:
  - Repo mapper and Thinktank both found cross-harness observability as a
    structural correctness gap.
- Rejected evidence:
  - Any adapter without a real hook/event surface and smoke path.
- Waivers:
  - No Codex/Pi hook API docs were externally researched in this shaping pass;
    implementation must inspect live harness docs/config before editing.

## Oracle

- [ ] Shared skill invocation fixture includes `harness` and
      `source_protocol`.
- [ ] Claude tracker continues to append existing fields and now includes
      `harness: claude`.
- [ ] Existing Claude hook tests pass and include backward-compatible input.
- [ ] Codex/Pi/Antigravity support is either implemented with a smoke test or
      explicitly marked unsupported/unavailable in analyzer coverage output.
- [ ] Telemetry failures never block skill/tool execution.
- [ ] No raw prompt/tool output is persisted by adapters.
- [ ] Source repo still has no committed generated harness skill bridges.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Observability Plan

- Changed behavior to watch: source coverage moves from Claude-only toward a
  labeled cross-harness view.
- Named signal or evidence surface: analyzer source coverage section and
  harness-specific hook test fixtures.
- Instrumentation debt if no signal exists: unsupported harness adapters remain
  explicit residual risk, not hidden omissions.

## Risk + Rollout

- Hook APIs differ: keep adapters small and independent.
- False cross-harness claims: require coverage report by harness.
- Runtime fragility: hooks must fail open.
- Privacy: metadata only.
- Rollback: remove adapter and fixture; analyzer still reports missing source.
