# Token and cost observability schema

Priority: P1
Status: ready
Estimate: M

## Goal

Extend Harness Kit's receipt, ledger, and skill-event schemas so provider,
model, token, duration, and cost data can be captured when available and
reported without guessing when unavailable.

## Why Now

The user wants to know token consumption per skill and whether skills are
effective enough to justify their cost. Today:

- `scripts/lib/agent_roster.py` records provider attempts but not duration,
  token counts, cost, or structured model details from the actual run.
- `scripts/work-ledger.py` records `owning_skill` and phase status but not
  model/cost fields.
- `harness-kit-checks claude-hook skill-invocation-tracker` records skill name and
  args but not harness, model, outcome, or usage.
- `meta/config-schemas/flywheel.schema.yaml` already has `budget_tokens`, but
  there is no measured token feed to compare against.

retired bench's own run envelope already reports model, token counts, and USD cost
by model. Harness Kit should capture analogous optional fields for its own
provider lanes and skill events.

## Non-Goals

- Do not require every harness to provide exact token counts in this ticket.
- Do not infer costs from raw transcripts or prompt text.
- Do not call provider billing APIs.
- Do not hard-code pricing tables into the first implementation.
- Do not export to a vendor backend yet.
- Do not store prompt/completion content.

## Constraints / Invariants

- Usage fields are optional and nullable.
- Unknown usage is `null`/`unknown`, never `0`.
- Existing schema-version-1 rows remain readable.
- Cost source must be explicit: `provider_reported`, `estimated`, `manual`, or
  `unknown`.
- Model ids come from the actual receipt/runtime when available, then roster
  config as fallback.
- Secret-like values remain rejected by existing trace/receipt guards.

## Authority Order

provider response usage > provider receipt envelope > explicit operator entry >
roster default > unknown

## Repo Anchors

- `scripts/lib/agent_roster.py` - receipt builder and dispatch wrapper.
- `scripts/record-delegation.py` - manual receipt entrypoint.
- `scripts/summarize-delegations.py` - operator-facing aggregation report.
- `scripts/work-ledger.py` - phase lifecycle event store.
- `harness-kit-checks claude-hook skill-invocation-tracker` - skill event source.
- `.harness-kit/examples/delegation-receipt.jsonl` and
  `.harness-kit/examples/work-ledger.jsonl` - schema fixtures.
- `scripts/check-agent-roster.py` - current fixture/schema gate.

## Prior Art

- Langfuse prioritizes ingested usage/cost over inferred usage and can map
  OpenAI-style `prompt_tokens`, `completion_tokens`, and `total_tokens` into
  its usage schema:
  https://langfuse.com/docs/observability/features/token-and-cost-tracking.
- Langfuse metrics expose cost, latency, quality, and volume slices over trace
  dimensions:
  https://langfuse.com/docs/metrics/overview.
- OpenTelemetry GenAI conventions define GenAI metrics, spans, and events but
  remain in development, so Harness Kit should map toward them without making
  them the canonical local schema yet:
  https://opentelemetry.io/docs/specs/semconv/gen-ai/.
- Phoenix accepts OpenTelemetry traces and evaluates traces/spans, making OTLP
  export a plausible later adapter:
  https://arize.com/docs/phoenix.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Optional local schema fields | Add nullable usage fields to receipts/events | Simple, private, backward compatible | Not a full billing report | Choose |
| Pricing-table estimator | Maintain local model pricing map | Useful for providers without usage | Stale quickly and can mislead | Defer |
| Langfuse usage export first | Emit Langfuse-compatible generation usage | Strong dashboards | Requires backend + auth + privacy review | Defer |
| OpenTelemetry spans first | Emit `gen_ai.*` span attributes | Interoperable | Spec still development; collector burden | Defer |
| Transcript byte approximation | Use transcript bytes as token proxy | Works everywhere | Too imprecise for cost claims | Use only as diagnostic field |
| Provider billing API import | Pull real bills | Accurate | High auth/security burden | Reject now |

## Proposed Shape

Add optional fields to delegation receipts:

```json
{
  "model_id": "gpt-5.5",
  "duration_ms": 12345,
  "usage": {
    "input_tokens": 1000,
    "output_tokens": 200,
    "total_tokens": 1200,
    "cost_usd": 0.0123,
    "cost_source": "provider_reported"
  },
  "transcript_bytes": 54321
}
```

Add optional usage fields to work-ledger events and skill invocation rows when
the invoking harness can provide them. Use the same nested `usage` shape.

Extend reports:

- `scripts/summarize-delegations.py --format text` prints usage coverage,
  total known tokens, total known cost, and unknown-count by provider.
- JSON format includes machine-readable `usage_by_provider`.
- Reports must label partial coverage rather than implying completeness.

## Agent Readiness

- Profile source: no dedicated profile; use existing Python fixtures.
- Stack feedback strength: deterministic Python CLI self-tests.
- ADR decision: not required for optional schema extension.
- Infrastructure path: local scripts only.
- Gate: `python3 scripts/check-agent-roster.py`, receipt/work-ledger self-tests,
  and `dagger call check --source=.`
- Evidence storage: `.harness-kit/examples/*.jsonl`.
- Mock policy impact: preserved; tests use fixture rows and public CLI outputs.

## Delegation Evidence

- Roster providers used:
  - `claude`, receipt `c1bc871f-4122-4786-a2df-e04e62a03c91`, identified
    missing token/cost fields in receipt/ledger schemas.
  - `codex`, receipt `004cb27a-ff40-4918-9ed7-40478b196a7f`, challenged
    build/buy/hybrid options.
- Native/retired bench evidence:
  - Repo mapper and retired bench both found that receipt and ledger streams have
    useful accountability fields but no cost/timing/token data.
- External evidence:
  - Langfuse usage/cost docs support prioritizing ingested usage over inferred.
  - OpenTelemetry/Phoenix support a later vendor-neutral export path.
- Rejected evidence:
  - Broad xAI claims about vendor-specific "skill analytics" unless confirmed
    by official docs.
- Waivers:
  - No live provider billing API inspected; this ticket avoids that dependency.

## Oracle

- [ ] Delegation receipt fixtures accept v1 rows and new rows with `model_id`,
      `duration_ms`, `usage`, and `transcript_bytes`.
- [ ] `scripts/record-delegation.py` accepts optional model/usage/duration
      arguments for manual rows.
- [ ] `scripts/dispatch-agent.py` records `duration_ms` and
      `transcript_bytes` for every dispatch attempt.
- [ ] Provider-reported token/cost fields are recorded when available; otherwise
      usage is absent/unknown without failing the dispatch.
- [ ] `scripts/summarize-delegations.py --format text` reports known usage and
      unknown coverage by provider.
- [ ] `scripts/summarize-delegations.py --format json` includes
      `usage_by_provider`.
- [ ] Work-ledger and skill-invocation fixtures tolerate optional `usage`
      fields.
- [ ] No report renders missing usage as zero.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Observability Plan

- Changed behavior to watch: provider/skill usage coverage and cost attribution
  become visible as explicit known/unknown counts.
- Named signal or evidence surface:
  `.harness-kit/traces/delegations.jsonl`, `.harness-kit/work/ledger.jsonl`,
  `~/.claude/skill-invocations.jsonl`, and summary output.
- Instrumentation debt if no signal exists: exact per-skill token attribution
  remains unavailable until harnesses expose usage at skill boundaries.

## Risk + Rollout

- False precision: label estimates and unknowns aggressively.
- Schema churn: keep fields optional and tolerate unknown fields.
- Cost privacy: store numeric usage/cost only, not prompt content.
- Provider inconsistency: normalize common fields but preserve source.
- Rollback: remove optional fields/report sections; old receipts remain valid.
