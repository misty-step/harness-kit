# Loop Readiness

Harness Kit designs loop handoffs. Bitterblossom or another Mode B event plane
runs unattended schedules, webhooks, queues, and recurring workers. The
canonical Mode A / Mode B boundary remains `meta/CONTRACTS.md`.

## Strategic Test

A workflow may become a loop only when all are true:

1. The work repeats.
2. A verifier can decide progress or done without the worker's self-judgment.
3. The runner can reproduce the environment it must change or inspect.
4. Token, dollar, time, and blast-radius budgets can absorb failed attempts.

If any answer is no, keep it as an ad-hoc Harness Kit session or a shaped
ticket.

## 30-Second Check

- Trigger: event, schedule, PR-ready state, incident, or manual run.
- State file: the durable place progress is written between ticks.
- Gate: the command, probe, or artifact that proves progress.
- Hard stops: max iterations, no-progress detection, token/dollar budget.
- Review boundary: the fresh verifier or human approval point before
  irreversible action.

## Minimum Viable Loop

One automation, one skill/lane card, one state file, one gate, one halt rule.
Anything more is earned by a concrete failure.

## Handoff Fields

- Owner repo and Mode B system.
- Trigger and cadence.
- Lane card path or embedded card.
- State file path.
- Verifier command.
- Evidence/receipt path.
- Human review boundary.
- Halt behavior for failure, no progress, and budget exhaustion.

## Reject By Default

Reject loops for architecture rewrites, vague "keep improving" goals,
one-off research, work with no automated verifier, or tasks where the worker
must grade its own output.
