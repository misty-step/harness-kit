---
name: roster
description: |
  Enumerates the peer AI agent CLIs installed on this machine (codex, pi,
  goose, opencode, claude, cursor-agent, grok, agy, hermes, thinktank)
  and how to invoke each headlessly. A capability map, not a quota: useful
  for fresh-context adversarial review on a different model family, second
  opinions, competing attempts, and wide benches. Use when: "ask codex",
  "ask another model", "second opinion", "cross-model review", "what AI
  tools do I have", "other agents", "different model family", "adversarial
  critique from another provider". Trigger: /roster.
argument-hint: "[provider] [task]"
---

# /roster

You are not the only frontier agent on this machine. These CLIs are
installed and each runs headlessly. They are options, not obligations:
native subagents remain the default delegation path, and a peer harness
earns a lane only when you can name what it adds — usually a decorrelated
model family or a genuinely fresh context.

## When a peer harness beats a native subagent

- **Adversarial critique of your own work.** A reviewer from a different
  model family has decorrelated failure modes and no loyalty to your
  reasoning. Give it the artifact (diff + oracle) only — never your
  reasoning trail.
- **Second opinion on a contested judgment** — architecture call, risk
  assessment, "is this idiomatic" — where one model's taste shouldn't
  decide alone.
- **Competing attempts** at the same bounded problem, graded blind.
- **Wide bench** — `thinktank` fans one task across many models and
  collates.

A native subagent is still better for exploration, scoped builds, and
anything where harness identity doesn't matter — it shares your tools,
needs no cold start, and the orchestrator is trained on it.

## The CLIs

Verified installed and probed 2026-06-14. Each row is the headless form;
add the prompt as the argument or via stdin.

| CLI | Stack | Headless invocation |
|---|---|---|
| `codex` | OpenAI Codex (gpt-5.5) | `codex exec "<task>"` (`--model`, `--config model_reasoning_effort=`) |
| `pi` | Pi over OpenRouter (Kimi, DeepSeek, …) | `pi -p --no-extensions --provider openrouter --model <id> "<task>"` |
| `goose` | Goose over OpenRouter | `goose run --no-session --quiet --provider openrouter --model <id> --text "<task>"` |
| `opencode` | OpenCode over OpenRouter | `opencode run --model openrouter/<id> --format json "<task>"` |
| `claude` | Claude Code (Opus/Fable) | `claude -p "<task>"` (`--model`, `--effort`) |
| `cursor-agent` | Cursor (composer) | `cursor-agent -p "<task>"` |
| `grok` | xAI Grok | `grok -p "<task>"` (`--model`, `--effort`) |
| `agy` | Antigravity (Gemini) | `agy --print "<task>"` |
| `hermes` | Hermes agent | `hermes -z "<task>"` (`-m <model>`) |
| `thinktank` | Multi-model bench | `thinktank review` · `thinktank run <bench> --input "<task>"` |

Current model ids, pricing, context windows, and freshness dates:
`references/model-provider-harness-index.md`. Open-model facts rot in days —
check the review-due date before quoting them.

## Judgment

- One well-aimed critic beats three vague ones. Aim it at the claim that
  would embarrass us in production, with explicit "ignore style/naming"
  bounds.
- Peer output is evidence, not authority. You weigh it, accept or reject
  it, and own the result.
- A failed or rambling lane is a result too — report it, don't re-roll
  silently.
- For a bounded lane whose evidence should outlive the session,
  `harness-kit-checks dispatch-agent` wraps the invocation and writes a
  delegation receipt (`.harness-kit/traces/delegations.jsonl`). Optional
  for quick second opinions; useful when the lane's verdict feeds a ship
  decision.
- Heavy, long-running, or isolation-needing lanes run on sprites
  (`/sprites`) regardless of which CLI executes them.

## Gotchas

- Peer CLIs run cold: no conversation history, no local skills unless the
  harness loads them itself. Inline everything the lane needs.
- Don't fan out to N providers to look thorough. Decorrelation is the
  point; two well-chosen families usually saturate it.
- Auth rots independently per CLI. A lane failing instantly with an auth
  error means re-login locally, not a provider verdict.
