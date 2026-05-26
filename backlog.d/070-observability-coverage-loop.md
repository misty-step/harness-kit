# Observability coverage loop

Priority: P1
Status: ready
Estimate: M

## Goal

Make observability coverage a first-class Spellbook health surface across
product and non-product repos. A repo should be able to answer: what behavior
changed, what telemetry or evidence would reveal whether it is working, and
which important paths are still invisible to agents and operators?

## Non-Goals

- Do not require a SaaS telemetry vendor.
- Do not log secrets, raw private transcripts, credentials, or customer data.
- Do not make `/monitor` diagnose or remediate; it observes and escalates.
- Do not turn every repo into a web dashboard project.

## Oracle

- [ ] Define an observability coverage report shape covering runtime telemetry,
      product analytics, agent traces, workflow receipts, command evidence,
      benchmark outputs, and release smoke artifacts.
- [ ] Extend `.spellbook/monitor.yaml` schema planning so repos can declare
      non-healthcheck signal sources such as delegation receipts, workflow
      events, evidence directories, local logs, and analytics coverage files.
- [ ] Teach `/monitor` and `/seed` to name a repo's dominant observability
      surfaces in repo-local config or vendored monitor guidance.
- [ ] Teach `/qa` or `/implement` to flag newly changed behavior with no
      observable signal as instrumentation debt.
- [ ] Provide one product-repo example and one Spellbook/library example.
- [ ] `dagger call check --source=.` passes.

## Notes

The n2parko-style nightly analytics scanner is the app-specific case. The
Spellbook-general version is broader: maximize useful context for future
agents through logs, traces, receipts, evidence, and coverage reports while
preserving privacy and thin-harness boundaries.
