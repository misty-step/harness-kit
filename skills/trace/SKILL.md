---
name: trace
description: |
  Capture agent-session work records as local JSONL audit evidence. Links a
  backlog/spec, branch, commits, review verdicts, QA/demo evidence, transcript
  refs, and shipped ref without storing raw private transcripts. Use when:
  "trace this work", "write work record", "agent session trace", "journal
  this delivery", "link transcript evidence". Trigger: /trace, /journal.
argument-hint: "append [--backlog <id>] [--transcript-ref <path>|--waiver-reason <reason>]"
---

# /trace

Capture the work record, not the raw conversation.

`/trace` is the durable session-lifecycle primitive. It writes sanitized JSONL
records that link a unit of work to its backlog/spec, branch, commits, reviewer
verdicts, QA evidence, demo evidence, transcript refs, and shipped commit/ref.
The default store is `.harness-kit/traces/work-records.jsonl`.

## Delegation Judgment

Trace append/preview commands are usually mechanical and may run direct solo.
When the trace requires substantive evidence classification, provenance
judgment, redaction-risk review, or conflicting artifact synthesis, delegate
on judgment per the shared Roster contract: native subagents by default; add
cross-model critics, roster providers, or sprite lanes (`/sprites`) only when
they answer a distinct question. See `harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use specialized lanes for evidence classifier,
redaction-risk critic, and provenance verifier. Store refs and receipt ids,
not raw provider transcripts.

## Contract

- Local-first. No hosted transcript database and no network writes.
- Append-only. Do not rewrite commit history to add trace evidence.
- Store refs, not raw transcripts. Use `/agent-transcript` to render a scoped,
  redacted transcript excerpt before attaching it.
- Persist a transcript ref or a clear waiver reason. "No transcript available"
  is acceptable only when the work record says that plainly.
- Fail closed on obvious secrets: token/API-key/password/credential names,
  bearer strings, private customer-data labels, or secret-like metadata.
- Link evidence by path, PR section, Git note ref, receipt id, or URL. Do not
  paste raw provider transcripts into the JSONL row.

## Helper

```bash
cargo run --locked -p harness-kit-checks -- trace-record append \
  --backlog 056 \
  --branch "$(git branch --show-current)" \
  --commit "$(git rev-parse --short HEAD)" \
  --reviewer-verdict-ref ".harness-kit/traces/delegations.jsonl#<id>" \
  --qa-ref ".evidence/qa/<id>.md" \
  --demo-ref ".evidence/demo/<id>.gif" \
  --transcript-ref ".harness-kit/traces/transcripts/<id>.md"
```

If transcript evidence is unavailable:

```bash
cargo run --locked -p harness-kit-checks -- trace-record append \
  --backlog 056 \
  --branch "$(git branch --show-current)" \
  --commit "$(git rev-parse --short HEAD)" \
  --waiver-reason "No safe transcript export available from this harness run."
```

Smoke test:

```bash
cargo run --locked -p harness-kit-checks -- trace-record --self-test
```

## Ship Handoff

`/ship` requires a final work record linking the shipped commit to trace
evidence, or an explicit waiver when no transcript or trace artifact exists.
The trace may be a `.harness-kit/traces/work-records.jsonl` row, a Git note, a
PR body section, or another named durable store. Raw session logs do not count.

## Bootstrap Exposure

`/trace` is a normal first-party skill under `skills/`, so `bootstrap.sh`
installs it system-wide through the existing cross-harness skill scan. No
special harness-native bridge is required.

## Gotchas

- Trace refs are audit breadcrumbs; they do not replace commit trailers, PR
  bodies, backlog closure, QA receipts, or demos.
- Redaction happens before trace storage. `/trace` rejects obvious leaks but
  does not summarize or sanitize a raw transcript for you.
- Keep bulky artifacts out of Git unless the repo already has an evidence
  convention for them.
