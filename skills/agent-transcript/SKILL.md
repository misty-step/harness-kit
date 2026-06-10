---
name: agent-transcript
description: |
  Redact and package local agent-session excerpts for PRs, issues, receipts,
  or review evidence. Use when: "add agent transcript", "attach session
  proof", "show agent provenance", "redact transcript", "PR transcript".
  Trigger: /agent-transcript.
argument-hint: "[render|preview] [session-log-or-text]"
---

# /agent-transcript

Package agent provenance without leaking the session.

Inspired by Peter Steinberger's `agent-transcript` contract: transcript
evidence is useful only when it is scoped, redacted, optional, and reviewable
before it leaves the machine.

## Contract

- Never upload raw logs.
- Never include system/developer prompts, raw tool output, secrets, cookies,
  auth URLs, broad local paths, or environment dumps.
- Ask before adding transcript evidence to a public PR or issue body.
- Fail closed when unresolved secrets remain.
- Keep only user intent, assistant-visible decisions, terse tool summaries,
  commands/tests run, and proof outcomes.
- Scope excerpts to the current PR/issue/branch/ticket; omit unrelated turns.
- Use a collapsed `<details>` block with stable markers so reruns update rather
  than duplicate the section.
- Best effort only: PR/issue creation continues when no safe transcript exists.

## Delegation Judgment

Rendering a scoped transcript preview is usually mechanical and may run direct
solo. delegate on judgment per the shared Roster contract: native subagents
by default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use specialized lanes for scope selector,
redaction-risk critic, provenance reviewer, and public-copy reviewer. Never
send raw transcripts to provider lanes; give rendered excerpts or sanitized
artifact refs only.

## Helper

Use the self-contained redactor for local previews:

```bash
cargo run --locked -p harness-kit-checks -- agent-transcript render \
  --input /path/to/session.log \
  --title "Agent Transcript" \
  --output /tmp/agent-transcript.md
```

Smoke test:

```bash
cargo run --locked -p harness-kit-checks -- agent-transcript --self-test
```

The helper reads plain text or JSONL-ish logs from stdin or `--input`. It writes
sanitized Markdown only. It performs no network calls and does not edit GitHub
bodies; callers pass the rendered block through the repo's normal PR/issue
body-file flow after human approval.

## Workflow

1. Identify scope: branch name, ticket/PR title, changed files, and goal.
2. Pick the smallest useful session excerpt.
3. Render locally with the helper.
4. Inspect the output for remaining secrets or unrelated turns.
5. If public insertion is desired, ask explicitly and use a temp body file.
6. Include the transcript path or PR section in the final receipt.

## Output Shape

```markdown
<!-- harness-kit-agent-transcript:start -->
<details>
<summary>Agent Transcript</summary>

...

</details>
<!-- harness-kit-agent-transcript:end -->
```

## Gotchas

- Raw transcripts are evidence-shaped secrets. Render first, inspect second,
  publish only after approval.
- Redaction is not summarization. Remove unrelated turns instead of hiding them
  behind broad `[REDACTED]` blocks.
- Public GitHub writes should use `--body-file`; never inline shell-expanded
  transcript text.
- A clean helper exit proves only the rendered file passed the redaction scan,
  not that the excerpt is relevant or appropriate.

## Verification

Run `cargo run --locked -p harness-kit-checks -- agent-transcript --self-test`
to prove rendering omits secret-like transcript content.
