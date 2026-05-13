# Signal Sources

Catalog of common post-deploy signal sources and how to query each. This
reference is query syntax and wire format — judgment about what counts as
a trip lives in `SKILL.md` and `grace-window.md`.

Every signal adapter must satisfy the same contract:

```
input:  { query | url, threshold }
output: { observed_value, threshold_ok (bool), sample_ts }
```

Keep adapters thin. They parse a response and compare to a threshold. They
do not retry (the poll loop handles cadence), do not page, do not cache.

---

## 1. HTTP Healthcheck (zero-config default)

Simplest signal. The deploy receipt's `healthcheck.url` is polled for a
2xx response.

```bash
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 5 "$URL"
```

Parse: integer status code.

Trip conditions:
- Non-2xx response code
- Connection refused, DNS failure, TLS handshake failure, timeout
- `expected_status` mismatch when the config pins a specific code

Hard-fail (skip debounce): any 5xx, any network error. Configured via
`hard_fail_on_5xx: true` (default).

---

## 2. Error Rate — Datadog

```bash
curl -sS \
  -H "DD-API-KEY: $DD_API_KEY" \
  -H "DD-APPLICATION-KEY: $DD_APP_KEY" \
  "https://api.datadoghq.com/api/v1/query?from=$(( $(date +%s) - 300 ))&to=$(date +%s)&query=$QUERY"
```

Response shape: `{ series: [{ pointlist: [[ts, value], ...] }] }`.
Take the last point's value. Compare to threshold.

Typical query: `sum:errors{service:app}.as_rate()` — errors per second
over the last 5 min.

Trip when observed value breaches the threshold. Soft signal → requires
two consecutive trips.

---

## 3. Latency p95/p99 — Prometheus / Grafana

```bash
curl -sS "$PROM_URL/api/v1/query" \
  --data-urlencode 'query=histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))'
```

Response shape: `{ data: { result: [{ value: [ts, "1.23"] }] } }`.
Parse the string value as float.

Threshold units: milliseconds. Convert if the metric is in seconds.

Soft signal → two-poll debounce.

---

## 4. Structured Log Greps

For apps that don't have a proper metrics pipeline. Query the log backend
(Cloud Logging, Loki, Papertrail, etc.) for error-shaped records in the
post-deploy window.

```bash
# Loki example
curl -sS -G "$LOKI_URL/loki/api/v1/query_range" \
  --data-urlencode 'query={app="web"} |= "level=error"' \
  --data-urlencode "start=$(( $(date +%s) - 60 ))000000000" \
  --data-urlencode "end=$(date +%s)000000000"
```

Parse: count matching records.

Threshold: count over an interval (e.g. `> 10` in 1 min). Be generous —
log-based thresholds are noisy.

Soft signal → two-poll debounce.

---

## 5. RUM 5xx Counts (real-user monitoring)

Client-side observation of server errors as seen by browsers. Useful
because it catches CDN, edge, and middleware failures that healthcheck
misses.

Query varies by vendor (Datadog RUM, Sentry, New Relic Browser). Same
contract: return a count for a time window, compare to threshold.

Soft signal.

---

## 6. Custom URL Probes

For signals that don't fit the above: any URL that returns a simple
status (2xx good, else trip) or a JSON document with a known field.

```yaml
signals:
  - name: feature_flag_service
    source: custom
    url: https://flags.internal/healthz
    jq: ".status == \"ok\""
```

Hard-fail when the URL is on the same host as healthcheck (loss of host
means loss of users). Soft otherwise.

---

## Adapter Registry

Backends currently expected:

| `source` | Adapter |
|----------|---------|
| `healthcheck` | built-in (curl) |
| `datadog` | `scripts/adapters/datadog.sh` |
| `prometheus` | `scripts/adapters/prometheus.sh` |
| `grafana` | `scripts/adapters/grafana.sh` |
| `loki` | `scripts/adapters/loki.sh` |
| `custom` | `scripts/adapters/custom.sh` (URL + optional `jq` filter) |

Adding a backend means writing one small script that takes the config
block on stdin and prints one JSON line on stdout:

```json
{"observed": "0.023", "threshold_ok": false, "ts": "2026-04-15T12:03:00Z"}
```

If the adapter fails (auth, network, parse), print to stderr and exit
non-zero. The poll loop treats adapter failures as `phase.failed`, not as
signal trips — a broken adapter should not trigger `/diagnose`.

---

## What NOT to Put Here

- Judgment about whether a given metric is a meaningful signal for this
  service. That is repo config, not the adapter.
- Diagnostic context. Adapters return numbers, not theories.
- Alert routing. The poll loop emits `monitor.alert`; routing is the
  outer loop's job.
- Historical baselining. This skill is short-horizon deploy verification,
  not trend detection.
