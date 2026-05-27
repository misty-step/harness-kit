# Bounded roster lane dispatch

Priority: P1
Status: ready
Estimate: S

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
   - Load the provider command from `.spellbook/agents.yaml`.
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
Spellbook. It records the same `.spellbook/traces/delegations.jsonl` receipts
that `probe-agent-roster.py` and `record-delegation.py` already use.

## Oracle

- [ ] `python3 -m unittest ci.tests.test_agent_roster` includes a fake provider
      that ignores `SIGTERM`; the dispatch helper times out, kills the process
      group, writes a transcript, and records a failed receipt.
- [ ] The same test suite covers unavailable providers refusing dispatch before
      a provider command is run.
- [ ] `python3 scripts/dispatch-agent.py --help` succeeds.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Non-Goals

- Do not build a provider scheduler, retry engine, ranking system, or semantic
  workflow DSL.
- Do not commit transcripts; runtime traces stay ignored under
  `.spellbook/traces/`.
- Do not change provider defaults or model choices.
- Do not solve provider auth or entitlement failures.

## Related

- Follow-up from: `067-positioning-boundary-for-client-facing-packages.md`
