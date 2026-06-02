# Observability coverage loop

Priority: P1
Status: ready
Estimate: M

## Goal

Make observability coverage a first-class Harness Kit health surface across
product and non-product repos. A repo should be able to answer: what behavior
changed, what telemetry or evidence would reveal whether it is working, and
which important paths are still invisible to agents and operators?

## Non-Goals

- Do not require a SaaS telemetry vendor.
- Do not log secrets, raw private transcripts, credentials, or customer data.
- Do not make `/monitor` diagnose or remediate; it observes and escalates.
- Do not turn every repo into a web dashboard project.

## Oracle

- [x] Define an observability coverage report shape covering runtime telemetry,
      product analytics, agent traces, workflow receipts, command evidence,
      benchmark outputs, and release smoke artifacts.
- [x] Extend `.harness-kit/monitor.yaml` schema planning so repos can declare
      non-healthcheck signal sources such as delegation receipts, workflow
      events, evidence directories, local logs, and analytics coverage files.
- [x] Teach `/monitor` and `/seed` to name a repo's dominant observability
      surfaces in repo-local config or vendored monitor guidance.
- [x] Teach `/qa` or `/implement` to flag newly changed behavior with no
      observable signal as instrumentation debt.
- [x] Provide one product-repo example and one Harness Kit/library example.
- [x] `dagger call check --source=.` passes.

## Notes

The n2parko-style nightly analytics scanner is the app-specific case. The
Harness Kit-general version is broader: maximize useful context for future
agents through logs, traces, receipts, evidence, and coverage reports while
preserving privacy and thin-harness boundaries.

`/seed` is retired in current Harness Kit. This delivery treats that checkbox
as repo-local generated/vendored monitor guidance through `/create-repo-skill`
and `.harness-kit/monitor.yaml`, without reviving the old minimal-global
`/seed` model.

## Progress

- Added `skills/monitor/references/observability-coverage.md` with the coverage
  report shape, config keys, product-repo example, Harness Kit/library example,
  and lifecycle hand-off.
- Extended `/monitor`'s `.harness-kit/monitor.yaml` example to name delegation
  receipts, workflow events, evidence dirs, local logs, benchmarks, release
  smoke, and analytics coverage.
- Added an Observability Plan to `/shape` context packets.
- Added `Observability / instrumentation debt` to `/implement` and `/qa`
  completion gates.
- Taught `/create-repo-skill` to preserve repo-local observable surfaces when a
  generated skill verifies behavior that should be watched after ship.

## Delegation Evidence

- `claude` receipt `975884e4-3a7f-467d-be44-195eafc245be`: accepted the
  monitor-owned coverage reference, `/implement` and `/qa` hooks, and explicit
  `/seed` non-revival. Kept examples in one observability reference rather than
  splitting multiple files.
- `grok-build` receipt `80743d55-e1ba-46cb-8a70-2c94c87a916a`: accepted the
  no-new-skill/no-dashboard contract, terminal-event `coverage` shape, and
  completion-gate debt field. Rejected "no tests needed" as too weak for final
  verification, but kept the implementation prose-only.
- `pi` receipt `146c5758-eef0-45c2-b788-34c0aaba2c8f`: accepted the new
  observability reference and config shape. Rejected required-signal CI gates,
  freshness scoring, and secret-pattern validators as over-scope for this thin
  contract.

## Verification

- `git diff --check`
- `bash scripts/build-docs-site.sh`
- `bash scripts/check-docs-site.sh`
- `python3 scripts/summarize-delegations.py --backlog-ref backlog.d/070-observability-coverage-loop.md --format text`
- `dagger call check --source=.` -> 15 passed, 0 failed
