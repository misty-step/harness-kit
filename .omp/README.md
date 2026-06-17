# harness-kit oh-my-pi (omp) harness

Repo-local config for the [`omp`](https://github.com/can1357/oh-my-pi) coding
agent. omp reads this `.omp/` dir at priority 100 (above inherited `.claude/`
and `.codex/` config), walking up from the cwd so it works from any subdir.

## Run it

```bash
omp                       # interactive TUI, in repo root
omp "wire up X"           # interactive with a starting prompt
omp -p "run the gate"     # non-interactive (print and exit)
```

## What's configured

**Models (global — `~/.omp/agent/config.yml`).** Barbell composition: heavy
models at SDLC start (plan) and end (advisor), lighter models for execution.
Everything routes through OpenRouter (uses `OPENROUTER_API_KEY`).

| Role | Model | Why | Cost (in/out per 1M) |
|---|---|---|---|
| default | GLM 5.2:high | TB 81.0, SWE Pro 62.1, MCP 76.8 — best open-weight | $1.40 / $4.40 |
| plan | GLM 5.2:high | "When in doubt" pick; coherent plan→execution | $1.40 / $4.40 |
| slow | DeepSeek V4 Pro | LiveCodeBench 93.5 (#1) — strongest algorithmic | $0.435 / $0.87 |
| advisor | Claude Opus 4.8 | MCP 82.2, SWE Pro 69.2 — heaviest review, decorrelated | $5 / $25 |
| task | Kimi K2.7 Code | MCP Mark 81.1 (#1) — best tool use for subagents | $0.95 / $4 |
| smol | DeepSeek V4 Flash | $0.14/$0.28 — unbeatable for text | $0.14 / $0.28 |
| commit | DeepSeek V4 Flash | Mechanical task | $0.14 / $0.28 |
| designer | GLM 5.2:medium | Same coding strength, lower thinking | $1.40 / $4.40 |
| vision | Gemini 3.5 Flash | MCP 83.6 (#1), multimodal, 4x faster | $1.50 / $9 |

Six model families for decorrelated failure: Z.ai (GLM), DeepSeek, Anthropic,
Moonshot, Google, and (via fallback) Alibaba/Qwen. Switch live with `/model`,
`Ctrl+P`, or `--slow`/`--plan`.

**Context.** omp natively loads `AGENTS.md` + `CLAUDE.md` + `CODEBASE.md`.
`.omp/RULES.md` holds the critical-few invariants and is re-injected near every
turn so they survive long sessions. Compaction uses `snapcompact` with
`systemPrompt: agents-md` to preserve AGENTS.md through compaction.

**Memory.** `mnemopi` per-project local memory: retains facts between sessions,
recalls them on startup. No external server needed.

**Advisor.** Claude Opus 4.8 passively reviews each turn (Anthropic, different
family from the default GLM 5.2). MCP Atlas 82.2, SWE-bench Pro 69.2 — catches
coding mistakes with decorrelated judgment. ~$0.04/turn.

**Scoped rules (`.omp/rules/`).** Fire only when you touch the matching paths:
- `rust-core.mdc` → `crates/**` — Rust-by-default, no mixed-language seams
- `skills.mdc` → `skills/**` — self-contained, no `$REPO_ROOT`, no bridges
- `backlog.mdc` → `backlog.d/**` — closure trailers, shape contract

**Slash commands (`.omp/commands/`).**
- `/gate` — run `cargo run --locked -p harness-kit-checks -- check --repo .`
- `/backlog` — show `backlog.d/` queue + closure protocol
- `/bootstrap` — run bootstrap + verify skill sync

**LSP.** On for the Rust core (diagnostics on edit); format-on-write off so it
never fights `cargo fmt` or the gate. Requires `rustup component add
rust-analyzer`.

## Harness-agnosticism

This `.omp/` directory is OMP-specific config, parallel to `.claude/` and
`.codex/` dirs for their respective harnesses. The shared doctrine lives in
`AGENTS.md`, `harnesses/shared/AGENTS.md`, and `skills/` — all of which are
harness-agnostic and portable across omp, Claude Code, Codex, Pi, Goose, and
OpenCode.

## Extending

- Add a command: drop `.omp/commands/<name>.md` (file body = the prompt).
- Add a scoped rule: `.omp/rules/<name>.mdc` with `globs:` frontmatter.
- Unpack omp's bundled task agents: `omp agents unpack --project`.
