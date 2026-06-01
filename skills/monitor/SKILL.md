---
name: monitor
description: |
  Watch signals after deploy, release, local run, CI change, or repeated
  workflow. Uses telemetry when present; otherwise healthchecks, logs, CI,
  flaky tests, benchmark drift, daemon output, regressions, or agent traces.
  Emits events, escalates to /diagnose on trip, closes on green. Use when:
  "monitor signals", "watch the deploy", "watch production", "watch CI",
  "watch logs", "watch benchmark drift", "is it healthy".
  Trigger: /monitor.
argument-hint: "[<deploy-receipt-ref>] [--grace <duration>] [--config <path>]"
---

# /monitor

Watch signals after a change. Escalate to `/diagnose` on regression. Close
clean when signals stay green through the grace window.

**Every repo has a monitor path.** A production app may watch healthchecks,
logs, traces, and error rates. A CLI may watch CI failures, golden-command
transcripts, and release regressions. A library may watch consumer builds,
benchmark drift, flaky tests, and issue reports. A local daemon may watch
its logs and readiness endpoint. Absence of Sentry, Prometheus, or a
public deploy is not absence of monitoring; it means the signal source is
different.

This skill observes and escalates. It does not diagnose root cause
(`/diagnose` does). It does not rollback (caller decides). It does not
page humans (outer loop decides).

Monitoring also includes observability coverage. A repo with no product
analytics still has signals: agent traces, provider receipts, workflow ledger
events, CI history, command transcripts, logs, benchmark outputs, and release
smoke artifacts. If a change creates behavior that cannot be observed after
ship, flag that as a monitor finding and route it to `/reflect` or `/groom`
as instrumentation debt.

Repeated human corrections are also a local monitoring signal. When the same
workflow needs the same correction across repeated runs, stop passively
watching and escalate to `/reflect prompt-debt` with sanitized counts,
available receipt refs, and the workflow name. Do not store raw private
transcripts or Chronicle detail in the monitor event.

## Execution Stance

You are a thin watcher.
- Load config or fall back to healthcheck-only.
- Poll on a fixed cadence until the grace window elapses or a signal trips.
- On trip: emit one `monitor.alert` event with payload, exit, hand off.
- On clean: emit one `monitor.done` event, exit.
- Never analyze why a signal tripped. Never attempt remediation.
- Provider roster lanes are not part of steady-state polling. If a trip needs
  investigation, hand off to `/diagnose`, which uses `.harness-kit/agents.yaml`
  as its two-or-more roster-member floor.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use lanes for signal interpretation and false-positive critique; pure polling is mechanical, and investigations route to /diagnose.

Signal query syntax (Datadog PromQL, Grafana HTTP, log greps) lives in
`references/signals.md`. Judgment about what constitutes a real trip vs
noise lives here.
Observability coverage report shape, repo-local signal surfaces, and examples
live in `references/observability-coverage.md`.

## Inputs

| Input | Source | Default |
|-------|--------|---------|
| deploy receipt ref | positional arg from `/deploy` | optional; when absent, use configured project signals |
| grace window | `--grace` flag, else `.harness-kit/monitor.yaml`, else built-in | 5 minutes |
| signal config | `.harness-kit/monitor.yaml` | repo-specific signal path; absent → healthcheck-only if `$HARNESS_KIT_HEALTHCHECK_URL` is set |
| poll interval | config `poll_interval` | 30 seconds |

## Contract

**Emits exactly one terminal event per invocation.** Either `monitor.done`
or `monitor.alert` — never both, never zero.

### Event schema

Events extend the `/flywheel` envelope so the outer loop can consume them
directly. Append to the active cycle's `cycle.jsonl` when running under
`/flywheel`; otherwise write to `.harness-kit/monitor/<ulid>.jsonl`.

```json
{
  "schema_version": 1,
  "ts": "2026-04-15T12:00:00Z",
  "cycle_id": "01HQ...",
  "kind": "monitor.alert",
  "phase": "monitor",
  "agent": "monitor",
  "refs": ["deploy-receipt:<ref>"],
  "findings": [
    {
      "signal": "healthcheck",
      "observed": "503",
      "expected": "200",
      "first_trip_ts": "2026-04-15T12:02:13Z",
      "consecutive_trips": 3,
      "samples": [
        {"ts": "2026-04-15T12:01:43Z", "value": "503"},
        {"ts": "2026-04-15T12:02:13Z", "value": "503"},
        {"ts": "2026-04-15T12:02:43Z", "value": "503"}
      ]
    }
  ],
  "note": "healthcheck returned 503 three consecutive polls; escalating"
}
```

On `monitor.done` the `findings` array holds the final sample per signal
(for audit) and `note` summarizes the clean window.

### Exit codes

| Exit | Meaning |
|------|---------|
| 0 | `monitor.done` emitted — signals green through grace window |
| 2 | `monitor.alert` emitted — signal tripped, escalating (not a failure) |
| 1 | Tooling failure (config parse, network, auth) — `phase.failed` emitted |

Exit 2 is distinct from exit 1 so the outer loop can route: `/diagnose`
on 2, retry-or-abort on 1.

## Escalation Rule

A signal **trips** when BOTH hold:
1. The observed value violates its threshold.
2. The violation is confirmed on the next poll (i.e. `consecutive_trips >= 2`).

**Hard failures skip the confirm step** (one-shot trip):
- Healthcheck 5xx flood (any 5xx counts as a trip; do not require two polls)
- Healthcheck connection refused / DNS failure / TLS error
- Deploy receipt's canary URL returns non-2xx on first poll

**Slow-burn failures require two consecutive trips:**
- Error rate over threshold
- Latency p95/p99 over threshold
- RUM 5xx counts over threshold
- Log-grep hit counts over threshold

This asymmetry is deliberate. A hard failure means the deploy is already
broken in an obvious way — delaying escalation by one poll interval wastes
30 seconds of users seeing 500s. A slow-burn signal can flap from a single
bad minute; one confirmation keeps noise from dragging `/diagnose` out
of bed.

Details in `references/grace-window.md`.

## Configuration

```yaml
# .harness-kit/monitor.yaml
grace_window: 5m          # total watch duration
poll_interval: 30s        # time between polls
observability:
  delegation_receipts: .harness-kit/traces/delegations.jsonl
  workflow_events: .harness-kit/work/*.jsonl
  evidence_dirs: [".evidence", ".harness-kit/monitor"]
  local_logs: ["logs/*.log"]
  benchmark_outputs: ["benchmarks/results/*.json"]
  release_smoke: [".evidence/release-smoke/*.json"]
  analytics_coverage: docs/analytics-coverage.md
healthcheck:
  url: https://app.example.com/health
  expected_status: 200
  hard_fail_on_5xx: true  # default true; disable for services that legitimately return 5xx
signals:
  - name: error_rate
    source: datadog
    query: "sum:errors{service:app}.as_rate()"
    threshold: "> 0.01"
    hard_fail: false      # default false for non-healthcheck signals
  - name: latency_p95
    source: prometheus
    url: https://prom.internal/api/v1/query?query=histogram_quantile(0.95,...)
    threshold: "> 2000"
```

Absent config → healthcheck-only using the deploy receipt's `healthcheck`
field or `$HARNESS_KIT_HEALTHCHECK_URL`. Absent receipt, config, and env
healthcheck → refuse to run, emit `phase.failed` with note
`monitor: no signal source available`. For repos with a vendored harness, this
is a harness failure: the local config or skill copy should name at least one
signal path.

Signal backend query syntax and response parsing live in
`references/signals.md`.

When `observability:` is present, `/monitor` names the repo's dominant
observable surfaces in the terminal event's `coverage` object. It does not
score the repo or require every key. Missing coverage for newly changed
behavior is a monitor finding and should route to `/reflect` or `/groom` as
instrumentation debt.

## Grace Window Judgment

The default 5 minutes is a compromise: long enough to catch obvious
breakage, short enough not to block the outer loop. Override per-repo, not
per-invocation, unless the deploy is unusually risky.

**Do not extend the grace window on soft trips.** If a slow-burn signal
flaps green after one trip, keep polling but do not reset the window. The
point of the window is a bounded watch.

**Do extend the grace window when the deploy ramp is gated.** If the
deploy receipt reports a staged rollout (`ramp: 10% → 50% → 100%`),
align the grace window to finish after the final ramp step plus two polls.
Otherwise you declare victory before real traffic hits the new version.

More in `references/grace-window.md`.

## Control Flow

```
/monitor [<deploy-receipt-ref>] [--grace <duration>]
    │
    ▼
  1. Load config (.harness-kit/monitor.yaml) or fall back to receipt healthcheck
  2. Compute deadline = now + grace_window (adjusted for ramp if present)
  3. Poll loop (every poll_interval):
       ├── For each signal: query, compare to threshold, record sample
       ├── Hard trip on healthcheck 5xx/refused → emit monitor.alert, exit 2
       ├── Soft trip confirmed (2 consecutive) → emit monitor.alert, exit 2
       └── All green AND now >= deadline → emit monitor.done, exit 0
    │
    ▼
  Emit terminal event. Release nothing — skill holds no global state.
```

## Invocation

```bash
# Outer loop: receipt from /deploy, config from repo
/monitor deploy:01HQ...

# Ad-hoc: custom grace window, healthcheck-only on a URL
/monitor --grace 10m --config /tmp/monitor.yaml

# Smoke test: zero-config, env healthcheck
HARNESS_KIT_HEALTHCHECK_URL=https://app.example.com/health /monitor

# Local/library repo: config watches CI or benchmark artifacts
/monitor --config .harness-kit/monitor.yaml
```

## Gotchas

- **Every repo has a signal path.** If there is no production telemetry,
  point this skill at the repo's real feedback surface: CI, logs,
  release smoke, benchmark drift, flaky tests, local daemons, or
  agent-session audit trails.
- **One terminal event per invocation.** Never emit both `monitor.done` and
  `monitor.alert`. If the loop somehow trips after the deadline, trust the
  first terminal condition and exit.
- **Exit 2 is not a failure.** It is an escalation signal. Callers that
  treat nonzero as failure will mistake alerts for tooling bugs.
- **Flapping signals are the common case, not the exception.** Single-poll
  threshold violations on non-healthcheck signals almost always resolve.
  Require the second confirmation.
- **Do not diagnose in the alert payload.** The payload is samples plus
  threshold info. Root cause belongs to `/diagnose`. A `note` field of
  "db connection pool likely exhausted" is out of scope and misleads the
  next skill.
- **Do not rollback.** Even if the signal is catastrophic. The outer loop
  may have reasons (forward-fix pending, ramp not yet at 100%) that this
  skill cannot know.
- **Do not extend the grace window to avoid escalation.** If the signal
  tripped, it tripped. Kicking the can wastes time and teaches the caller
  to distrust monitor output.
- **Do not normalize repeated correction loops.** If the same workflow needs
  the same human correction repeatedly, emit a monitor finding and hand off to
  `/reflect prompt-debt` instead of silently continuing the watch.
- **Healthcheck connection refused is a hard trip.** Do not treat it as
  transient. If the host is unreachable two polls in a row you have
  already lost users.
- **Config changes mid-flight are ignored.** The config loaded at start is
  the config for the invocation. A repo editing `.harness-kit/monitor.yaml`
  during a watch does not alter thresholds for the running instance.
- **Poll interval is a floor.** Slow backends may stretch the actual
  cadence. The grace window is wall-clock, not poll-count.
- **Never page humans.** Write the alert event. The outer loop routes
  escalation (Slack, PagerDuty, `/diagnose`). Paging from inside this
  skill duplicates channels and creates noise.
