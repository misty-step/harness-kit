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

**Models (global — `~/.omp/agent/config.yml`).** Everything routes through
OpenRouter (built-in; uses `OPENROUTER_API_KEY`). Default anchor is
`z-ai/glm-5.2:high`; on quota/rate-limit it fails over through the chain
`deepseek-v4-pro → grok-4.3 → kimi-k2.7-code → gemini-3.5-flash → minimax-m3 →
glm-5.1`, then reverts when the primary's cooldown expires.

Role split: `slow`→deepseek-v4-pro, `plan`→grok-4.3, `task`→minimax-m3,
`smol`/`commit`→deepseek-v4-flash, `vision`→gemini-3.5-flash,
`designer`→glm-5.2:medium, `advisor`→grok-4.3. Switch live with `/model`,
`Ctrl+P`, or `--slow`/`--plan`.

**Context.** omp natively loads `AGENTS.md` + `CLAUDE.md` + `CODEBASE.md`.
`.omp/RULES.md` holds the critical-few invariants and is re-injected near every
turn so they survive long sessions. Compaction uses `snapcompact` with
`systemPrompt: agents-md` to preserve AGENTS.md through compaction.

**Memory.** `mnemopi` per-project local memory: retains facts between sessions,
recalls them on startup. No external server needed.

**Advisor.** Grok 4.3 passively reviews each turn (different model family from
the default GLM 5.2 for decorrelated judgment). Catches drift before it
compounds.

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
