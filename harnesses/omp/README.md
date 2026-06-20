# OMP Harness Notes

OMP (oh-my-pi) is the primary lead harness for Harness Kit. It is not a roster
delegation target — it is the harness the operator drives directly. The roster
(codex, pi, goose, opencode) provides cross-model peer lanes that OMP dispatches
through `task` subagents or the `dispatch-agent` receipt system.

## Config Surfaces

**Global (`~/.omp/agent/config.yml`).** Model roles, fallback chains, LSP,
compaction, memory, advisor. The global config is user-owned and shared across
all repos.

**Harness Kit source (`harnesses/omp/`).** Keep OMP-specific notes and future
bootstrap inputs here. Harness Kit does not commit a source-root `.omp/`
projection; operator-local `.omp/` files are user-owned unless a shaped ticket
proves bootstrap support is worth adding.

## Model Composition

OMP's role-based routing assigns different models to different work types in
the user's global config. Keep repo source limited to portable doctrine and
bootstrap inputs; model rosters live in Harness Kit's roster files.

## What's Different from Codex / Claude Code

- **TTSR rules** — regex-matched invariants injected mid-stream; survive
  compaction. If Harness Kit needs source-controlled OMP rules later, they
  should be generated from shared doctrine or shaped as explicit bootstrap
  inputs, not committed as root `.omp/` drift.
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

The shared doctrine (`AGENTS.md`, `harnesses/shared/AGENTS.md`, `skills/`) is
harness-agnostic and portable across all harnesses. OMP-specific config does
not leak into shared doctrine.
