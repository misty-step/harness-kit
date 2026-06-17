# OMP Harness Notes

OMP (oh-my-pi) is the primary lead harness for Harness Kit. It is not a roster
delegation target — it is the harness the operator drives directly. The roster
(codex, pi, goose, opencode) provides cross-model peer lanes that OMP dispatches
through `task` subagents or the `dispatch-agent` receipt system.

## Config Surfaces

**Global (`~/.omp/agent/config.yml`).** Model roles, fallback chains, LSP,
compaction, memory, advisor. The global config is user-owned and shared across
all repos.

**Repo-local (`.omp/`).** Harness Kit's repo-local OMP config:
- `RULES.md` — critical-few invariants, re-injected every turn (survive
  compaction via `snapcompact.systemPrompt: agents-md`)
- `commands/` — `/gate`, `/backlog`, `/bootstrap` slash commands
- `rules/` — scoped TTSR rules for `crates/`, `skills/`, `backlog.d/`

## Model Composition

OMP's role-based routing assigns different models to different work types. See
`.omp/README.md` for the current role split and rationale. The barbell strategy
uses GLM 5.2:high for default/plan, DeepSeek V4 Pro for slow, Claude Opus 4.8
for advisor (decorrelated passive review), and Kimi K2.7 Code for task subagents.

## What's Different from Codex / Claude Code

- **TTSR rules** — regex-matched invariants injected mid-stream; survive
  compaction. The `.omp/rules/` scoped rules and `.omp/RULES.md` are the
  mechanical guardrails that AGENTS.md prose alone cannot enforce.
- **Hashline edits** — content-hash anchored; rejects stale patches before
  corrupting files. No string-not-found loops.
- **In-process search** — ripgrep, glob, find linked into the binary. No
  fork-exec per search call.
- **LSP wired into writes** — rename, diagnostics, go-to-def through the
  language server. Requires `rustup component add rust-analyzer` for Rust.
- **Persistent Python + JS kernels** — kernels can call back into agent tools
  (`read`, `search`, `task`) over a loopback bridge.
- **Memory** — `mnemopi` per-project local recall between sessions.
- **Advisor** — passive per-turn review by a different model family.

## Harness-Agnostic Principle

The `.omp/` directory is OMP-specific, parallel to `.claude/` and `.codex/`
for their harnesses. The shared doctrine (`AGENTS.md`,
`harnesses/shared/AGENTS.md`, `skills/`) is harness-agnostic and portable
across all harnesses. OMP-specific config does not leak into shared doctrine.
