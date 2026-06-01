# Bounded roster lane dispatch

Priority: P1
Status: done
Estimate: S
Shipped: 2026-06-01

## Resolution

Shipped on `master` before this archive pass. Harness Kit now has
`scripts/dispatch-agent.py` as the CLI boundary and
`dispatch_provider_lane()` in `scripts/lib/agent_roster.py` as the reusable
helper. Dispatch loads configured roster commands, refuses unavailable/manual
providers before command execution, appends the prompt as a separate argv
element, writes transcript evidence under `.harness-kit/traces/provider-lanes`,
uses `start_new_session=True`, and kills the process group after timeout with
SIGTERM then SIGKILL.

Verification on `deliver/072-bounded-roster-lane-dispatch`:

- `python3 -m unittest ci.tests.test_agent_roster` — 23 tests OK, including the
  unavailable-provider refusal and SIGTERM-ignoring process-group timeout case.
- `python3 scripts/dispatch-agent.py --help` — CLI exposes provider, objective,
  input, prompt-file, timeout, grace, transcript-dir, receipt, and roster args.
- `git show HEAD:scripts/lib/agent_roster.py` confirmed
  `dispatch_provider_lane`, `start_new_session=True`, SIGTERM, and SIGKILL.

Closeout gate for this archive commit:
`python3 scripts/check-agent-roster.py` and `dagger call check --source=.`
both passed.

## Goal

Add a thin, test-backed provider dispatch boundary for roster lanes. The unit
is a small script plus shared helper that runs one configured provider command
with a wall-clock timeout, process-group cleanup, transcript evidence, and a
sanitized delegation receipt.

This codifies the failure from the `067-positioning-boundary` run: a provider
lane nested/hung, left background processes behind, and required manual cleanup
of raw transcripts.

## Design

1. Add a reusable dispatch helper in `scripts/lib/agent_roster.py`.
   - Load the provider command from `.harness-kit/agents.yaml`.
   - Refuse manual or unavailable providers.
   - Run with `start_new_session=True` so timeout cleanup can kill the whole
     process group, not just the parent PID.
   - Write stdout/stderr to an ignored transcript path.
   - On timeout: send `SIGTERM`, wait a short grace period, then `SIGKILL`.
   - Return and record a receipt with attempt status, provider status, summary,
     and transcript path as evidence.
2. Add `scripts/dispatch-agent.py` as the CLI entrypoint.
   - It accepts `--provider-target`, `--objective`, `--input-ref`,
     `--prompt-file`, `--timeout-s`, `--grace-s`, `--transcript-dir`, and the
     usual roster/receipt args.
   - It appends the prompt text to the provider dispatch argv rather than
     invoking through a shell.
3. Keep this as a boundary utility, not an orchestration engine.
   - No ranking, hidden supervision, retry loops, semantic scheduling, or
     cross-provider synthesis.

## Cross-Harness

The mechanism is filesystem and CLI based. Claude Code, Codex, Pi, and other
harnesses can call the same `scripts/dispatch-agent.py` from any repo using
Harness Kit. It records the same `.harness-kit/traces/delegations.jsonl` receipts
that `probe-agent-roster.py` and `record-delegation.py` already use.

## Oracle

- [x] `python3 -m unittest ci.tests.test_agent_roster` includes a fake provider
      that ignores `SIGTERM`; the dispatch helper times out, kills the process
      group, writes a transcript, and records a failed receipt.
- [x] The same test suite covers unavailable providers refusing dispatch before
      a provider command is run.
- [x] `python3 scripts/dispatch-agent.py --help` succeeds.
- [x] `python3 scripts/check-agent-roster.py` passes.
- [x] `dagger call check --source=.` passes.

## Non-Goals

- Do not build a provider scheduler, retry engine, ranking system, or semantic
  workflow DSL.
- Do not commit transcripts; runtime traces stay ignored under
  `.harness-kit/traces/`.
- Do not change provider defaults or model choices.
- Do not solve provider auth or entitlement failures.

## Related

- Follow-up from: `067-positioning-boundary-for-client-facing-packages.md`
