# Observability Coverage

Observability coverage answers one question: after a change ships, what signal
would tell an agent or operator whether the changed behavior is working?

This is not a telemetry product, dashboard, score, or historical trend system.
It is a report shape and repo-local config convention that make existing
signals visible.

## Report Shape

`/monitor` includes a `coverage` object in its terminal event when config,
deploy receipt, or repo evidence names observable surfaces.

```json
{
  "coverage": {
    "runtime_telemetry": ["healthcheck", "error_rate", "latency_p95"],
    "product_analytics": ["analytics_coverage:docs/analytics-coverage.md"],
    "agent_traces": [".harness-kit/traces/delegations.jsonl"],
    "workflow_receipts": [".harness-kit/work/*.jsonl"],
    "command_evidence": [".evidence/cli-smoke/*.txt"],
    "benchmark_outputs": ["benchmarks/results/*.json"],
    "release_smoke": [".evidence/release-smoke/*.json"],
    "missing_for_changed_behavior": [
      {
        "behavior": "checkout coupon validation",
        "debt": "no log, metric, receipt, smoke artifact, or analytics event named"
      }
    ]
  }
}
```

Only include categories that exist for the repo. Empty coverage is not an
error by itself; changed behavior with no named signal is instrumentation debt.

## Config Shape

Use `.harness-kit/monitor.yaml` to name signal sources. Paths are repo-relative
unless absolute.

```yaml
observability:
  delegation_receipts: .harness-kit/traces/delegations.jsonl
  workflow_events: .harness-kit/work/*.jsonl
  evidence_dirs: [".evidence", ".harness-kit/monitor"]
  local_logs: ["logs/*.log"]
  benchmark_outputs: ["benchmarks/results/*.json"]
  release_smoke: [".evidence/release-smoke/*.json"]
  analytics_coverage: docs/analytics-coverage.md
```

These keys are descriptive. A repo may use a subset. Backends under `signals:`
remain the active poll targets; `observability:` explains the broader evidence
surface agents should inspect before claiming a behavior is invisible.

## Product Repo Example

```yaml
healthcheck:
  url: https://app.example.com/health
  expected_status: 200
signals:
  - name: checkout_error_rate
    source: datadog
    query: "sum:checkout.errors{env:prod}.as_rate()"
    threshold: "> 0.01"
observability:
  product_analytics: docs/analytics-coverage.md
  release_smoke: [".evidence/checkout-smoke/*.json"]
  local_logs: ["logs/web/*.log"]
```

Dominant surfaces: healthcheck, checkout error-rate metric, analytics coverage
doc, release-smoke artifacts, and web logs.

## Harness Kit / Library Example

```yaml
observability:
  delegation_receipts: .harness-kit/traces/delegations.jsonl
  workflow_events: .harness-kit/work/*.jsonl
  evidence_dirs: [".evidence", ".harness-kit/monitor"]
  benchmark_outputs: ["ci/benchmarks/*.json"]
  release_smoke: [".evidence/cli-smoke/*.txt"]
```

Dominant surfaces: roster receipts, workflow events, command transcripts, CI
benchmark output, and release-smoke artifacts. No SaaS telemetry is required.

## Phase Hand-Off

- `/shape` names the expected observable surface for behavior that must be
  watched after ship.
- `/implement` adds instrumentation only when the packet requires it; otherwise
  it names instrumentation debt in the Completion Gate.
- `/qa` verifies observable surfaces that are part of the changed behavior and
  records missing surfaces as residual risk.
- `/monitor` observes and escalates. It does not diagnose or remediate.
- `/reflect` and `/groom` turn repeated missing-signal findings into backlog.
