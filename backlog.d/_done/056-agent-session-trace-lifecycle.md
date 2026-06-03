# Agent session trace lifecycle

Priority: P1
Status: done
Estimate: M

## What Was Built

- Added `/trace` as a first-party work-record primitive for agent-session
  lifecycle evidence.
- Added a self-contained JSONL appender at
  `skills/trace/scripts/trace_record.py` with transcript-or-waiver enforcement,
  obvious-secret rejection, and a focused self-test.
- Added `.harness-kit/examples/work-record.jsonl` plus roster validation for
  the trace work-record schema and runtime store convention.
- Extended `/ship` so final-mile shipping records a trace handoff after the
  merged SHA exists, or refuses without trace inputs or an explicit waiver.
- Added a Dagger lane for the trace helper self-test.

## Goal

Make the conversation between a developer and coding agents a durable
work artifact, linked to the spec, commits, review evidence, QA result,
demo artifact, and shipped change.

## Non-Goals

- Do not build a hosted conversation database.
- Do not require one harness's private transcript format as the only
  source of truth.
- Do not store secrets or raw credentials in traces.
- Do not replace commit history, PR descriptions, or backlog closure.
  Session traces augment those records.

## Oracle

- [ ] A workflow primitive exists (`/trace`, `/journal`, or an extension
      to `/reflect` + `/ship`) that captures agent-session metadata for
      a unit of work: backlog/spec ID, branch, commits, reviewer verdicts,
      QA evidence, demo artifact, and transcript refs.
- [ ] The primitive supports at least one durable attachment mechanism
      that does not rewrite history: Git notes, a `.harness-kit/traces/`
      JSONL index, PR body links, or another explicitly named store.
- [ ] `/ship` requires a final work record linking the shipped commit to
      the trace artifact or records why no transcript was available.
- [ ] Bootstrap exposes the trace primitive system-wide once it exists; `/seed`
      vendors any repo-local trace config when checked-in harness state is
      explicitly requested.
- [ ] The design names redaction rules and refuses to persist obvious
      secrets (`*_TOKEN`, API keys, credentials, private customer data).
- [ ] `dagger call check --source=.` green.

## Notes

Agent conversation is becoming part of the audit trail for software
work. Commit history records the resulting state; the agent transcript
records decisions, corrections, failed hypotheses, tool evidence, and
review context that explain how the work arrived there.

The storage contract should be boring and local-first. A minimal viable
shape is a small JSONL record under `.harness-kit/traces/` plus optional
Git notes on the landing commit. The JSONL record can point at external
transcript exports when a harness provides them, without making that
harness the primary architecture.

This should compose with existing lifecycle skills:

- `/reflect` distills lessons and harness improvements.
- `/qa` produces verification evidence.
- `/demo` produces a communicable artifact.
- `/code-review` produces review verdicts.
- `/ship` ties backlog closure, commits, evidence, and trace refs into
  the final work record.

Do not overload `/reflect` until it becomes a transcript archive. The
trace primitive should preserve evidence; `/reflect` should synthesize
lessons from it.
