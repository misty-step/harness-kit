# Grace Window

The grace window is the bounded wall-clock period during which `/monitor`
watches signals after a deploy. This reference spells out what counts as a
trip, how debouncing works, and how the window interacts with deploy ramps.

## Default

5 minutes. Poll every 30 seconds → 10 polls minimum per signal.

Rationale: most deploy regressions surface within the first few minutes.
Stretching to 30+ minutes catches a few rare issues but blocks the outer
loop and invites flapping-signal noise. 5 minutes is the Pareto sweet
spot for short-horizon verification.

Override per-repo in `.spellbook/monitor.yaml`. Override per-invocation
with `--grace` only for unusually risky deploys.

## What Counts as a Trip

A signal **trips** when its observed value violates its threshold AND the
violation is confirmed. The confirmation rule depends on signal class:

### Hard-fail signals (one-shot)

No debounce. First poll that violates the threshold is a trip.

- Healthcheck non-2xx response
- Healthcheck connection refused, DNS failure, TLS error
- Healthcheck any 5xx (unless `hard_fail_on_5xx: false`)
- Any signal explicitly configured with `hard_fail: true`

Rationale: when the app is unreachable or returning 500s, every
additional poll is another wave of users seeing errors. Delay costs
humans, not just CI minutes.

### Soft signals (two-poll debounce)

Violation must be confirmed on the next poll.

- Error rate thresholds
- Latency p95/p99 thresholds
- RUM 5xx counts
- Log-grep match counts
- Any signal explicitly configured with `hard_fail: false` (default)

Rationale: single-minute spikes in these metrics are common and usually
resolve. One confirmation cuts the noise floor to near-zero without
meaningfully slowing response to real regressions.

### Debounce semantics

A signal carries per-instance state: `consecutive_trips`. Reset to 0 on
any poll where the threshold is met. Increment on every violation. Trip
fires when `consecutive_trips >= 2` (soft) or `>= 1` (hard).

**One flap resets the counter.** A pattern of `bad, good, bad, good` does
not trip. This is intentional — flapping is noise, not signal.

**Do not accumulate debounce across the window.** A signal that is bad
for 1 poll, good for 8 polls, bad for 1 poll should not trip. If the
signal is actually degraded, it will stay bad for at least two polls.

## What Does NOT Count as a Trip

- A single bad poll on a soft signal
- A flapping signal that resolves
- An adapter error (treat as `phase.failed`, not `monitor.alert`)
- A timeout on a single poll (retry next poll; two timeouts in a row on a
  hard-fail signal IS a trip)
- Threshold violations before the first successful baseline poll (the
  first poll establishes the baseline; count trips from poll 2 onward
  for soft signals)

## Grace Window Extension

### Do extend for staged ramps

If the deploy receipt reports `ramp: [10%, 50%, 100%]` and each ramp step
takes N minutes:

```
effective_grace = sum(ramp_step_durations) + (2 * poll_interval)
```

You want at least two polls of observation after the final ramp step, so
real production traffic exercises the new version before you declare
victory.

Default grace is 5 minutes; ramp-aware grace is typically 15-45 minutes
depending on ramp cadence.

### Do NOT extend for soft trips

A single flapped signal is not grounds to extend the watch. The window is
bounded on purpose.

### Do NOT extend to "wait and see"

If the deploy looks worrying but no signal has tripped, emit `monitor.done`
at the deadline. The outer loop decides whether to keep watching via a
separate invocation. Stretching the window silently invites operators to
treat `/monitor` as indefinite production monitoring, which it is not.

## Interaction with the Outer Loop

`/monitor` is invoked by `/flywheel` after `/deploy` reports success. The
outer loop's contract:

| Monitor exit | Outer loop action |
|--------------|-------------------|
| 0 (`monitor.done`) | Proceed to `/reflect`, close cycle |
| 2 (`monitor.alert`) | Invoke `/diagnose` with the alert payload |
| 1 (`phase.failed`) | Emit `phase.failed`, abort cycle, require operator |

Exit 2 is reserved. Do not reuse it for tooling failures.

## Poll Cadence Invariants

- `poll_interval` is a floor, not a ceiling. A slow signal backend may
  stretch the actual cadence to 45s even with a 30s interval.
- The grace window is measured in wall-clock seconds, not poll counts. If
  a slow backend turns 10 polls into 6, you still stop at the deadline.
- Do not block the poll loop on a single slow signal. Run signal queries
  in parallel within a poll; the poll completes when all signals return
  or the poll times out (default 10s).

## Known Failure Modes

- **Time-skewed hosts.** If the signal backend reports timestamps from
  before the deploy, the freshest sample may predate the change. Filter
  by `deploy.completed_at` in the query window.
- **Cold caches.** First 1-2 polls after deploy may show elevated latency
  from cache warmup. Either (a) use a longer `poll_interval` for the
  first minute, or (b) configure latency thresholds to accept the
  warmup. Do not special-case this in the skill — it is repo config.
- **DNS TTL lag.** If the deploy involves DNS changes, the healthcheck
  may hit old IPs for the TTL duration. Monitor-from-inside (i.e. app
  self-report) or extend `--grace` manually.
- **Flapping during ramp.** Error rate can legitimately spike during a
  ramp step as more traffic hits the new version. Ramp-aware grace
  smooths this out; manual `--grace` does not, which is one reason to
  prefer ramp-aware configuration.
