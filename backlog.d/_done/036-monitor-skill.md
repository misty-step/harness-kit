# `/monitor` — post-deploy signal watch

Priority: medium
Status: pending
Estimate: S (~1 dev-day)

## Goal

Watch signals after a deploy. Escalate to `/investigate` if signals
indicate regression. Close cleanly if signals stay green through a
configurable grace window. Invoked by `/autopilot` (outer loop) after
`/deploy` reports success.

## Contract

**Input:** Deploy receipt (from `/deploy`). Grace window (default 5 min,
configurable). Signal sources (from repo config).

**Output:**
- `monitor.done` event if signals stay green through grace window
- `monitor.alert` event with payload if any signal trips
- Signal snapshot appended to cycle manifest

**Stops at:** grace window elapsed clean, OR signal trip. Does not fix
problems (→ `/investigate`), does not rollback (→ caller decides).

## Stance

1. **Thin watcher, not diagnostician.** Monitor observes and escalates.
   Diagnosis is `/investigate`'s job. Remediation is caller's.
2. **Signal sources are per-repo.** Global skill reads config; no
   baked-in assumptions about what "signal" means for this repo.
3. **Grace window is load-bearing.** Most deploy issues surface in first
   few minutes. Watching for 5 minutes catches obvious breakage without
   blocking outer-loop for hours.
4. **Escalation is one-shot.** On trip, emit `monitor.alert` once and
   exit. Let the outer loop decide whether to triage, rollback, or abort.

## Composition

```
/monitor <deploy-receipt-ref> [--grace <duration>]
    │
    ▼
  1. Load signal config (.harness-kit/monitor.yaml or defaults)
    │
    ▼
  2. Establish baseline: healthcheck 200, error rate, latency p99
    │
    ▼
  3. Poll every 30s until grace window elapses
     ├── Any signal trips threshold → emit monitor.alert, exit
     └── All green through window → emit monitor.done, exit
    │
    ▼
  Exit 0 (clean) or exit 2 (alerted, not a failure per se)
```

## Repo-Local Config

```yaml
# .harness-kit/monitor.yaml
healthcheck:
  url: https://harness-kit.fly.dev/health
  expected_status: 200
signals:
  - name: error_rate
    source: datadog
    query: "sum:errors{service:harness-kit}.as_rate()"
    threshold: "> 0.01"
  - name: latency_p99
    source: grafana
    url: https://grafana.internal/api/...
    threshold: "> 2000"
grace_window: 5m
```

Absent config → fall back to healthcheck-only monitoring using deploy
receipt's `healthcheck` URL.

## Signal Sources

Initial: healthcheck polling only (zero-config). Pluggable backends for
Datadog / Grafana / CloudWatch / Sentry as repos need them. Each backend
= small adapter that takes a query + threshold and returns bool.

## What `/monitor` Does NOT Do

- Diagnose root cause of alerts (→ `/investigate`)
- Rollback (→ `/deploy --rollback`, called by outer loop on triage decision)
- Long-term observability — this is short-horizon deploy verification
- Page humans — log the alert; caller decides paging

## Oracle

- [ ] `skills/monitor/SKILL.md` exists
- [ ] Healthcheck-only mode works with zero config
- [ ] Grace window elapsed clean → `monitor.done` event
- [ ] Failed healthcheck → `monitor.alert` event with structured payload
- [ ] Integrates with `/autopilot` outer loop: alerts trigger `/investigate`
- [ ] At least one custom signal backend working (Datadog OR Grafana)

## Non-Goals

- Long-term monitoring — this is deploy-verification, minutes not days
- Alerting humans — structured log only; outer loop decides escalation
- Replacing incident tooling — complements, doesn't compete
- Writing dashboards — reads them

## Related

- Blocks: 028 (`/autopilot` outer loop composition)
- Depends on: 035 (`/deploy` — consumes deploy receipt)
- Escalates to: existing `/investigate` on `monitor.alert`
