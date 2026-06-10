# AGENTS.md — Harness Kit

Harness Kit is the harness source repo. Keep this file terse: repo-specific
contracts only. Put workflow detail in skills; put generated/runtime state on
disk; do not restate obvious filesystem facts.

## Non-Negotiables

- Base branch: `master`.
- Gate: `cargo run --locked -p harness-kit-checks -- check --repo .`. Green
  means the Rust-owned local gate passed; `/ci` owns the exact lane list in
  `crates/harness-kit-checks/src/ci_check.rs`.
- Clean-tree closeout: shared Closeout applies; see
  `harnesses/shared/AGENTS.md` (Closeout). Harness Kit additionally treats
  untracked `backlog.d/NNN-*.md` as signal unless the user explicitly says
  scratch/delete.
- `index.yaml` is generated. Never edit it by hand.
- `harnesses/claude/settings.json` is copied by bootstrap; changes require
  re-bootstrap.
- Durable tooling is Rust in `crates/harness-kit-checks`. The only allowed
  non-Rust implementation surface is `bootstrap.sh` as the curl-compatible
  Rust launcher. Every gate must name a real failure it catches; gates that
  enforce prose structure are the historical failure mode here.
- Harness Kit source skills live only in `skills/`. Do not commit source-repo
  `.agents/skills/`, `.codex/skills/`, `.claude/skills/`, `.pi/skills/`, or
  `.antigravitycli/skills/` bridges; those duplicate the global install here.

## Roster

The delegation floor lives in `harnesses/shared/AGENTS.md` (Roster). Harness Kit
resolves providers from `.harness-kit/agents.yaml` or
`~/.harness-kit/agents.yaml`, records sanitized receipts in
`.harness-kit/traces/delegations.jsonl`, and reports receipt-grounded roster
evidence instead of raw transcripts.

## Backlog

- Active: `backlog.d/NNN-*.md`.
- Closed: `backlog.d/_done/NNN-*.md`.
- Closure signal: `Closes-backlog:` / `Ships-backlog:` trailers, or an
  explicit backlog move committed with the work.
- Open high-signal debt starts at `backlog.d/023-*.md`; do not mirror the debt
  table here. Read the directory.

## Positioning

Before answering whether to hand this repo to a client, enterprise,
department, executive, procurement reviewer, security reviewer, or
nontechnical team, read `docs/positioning.md`. Harness Kit is implementation
substrate for technical operators, not the buyer-facing governed workflow
package or admin-control plane.

## Harness Work

This repo is Mode A only: the ad-hoc operator harness. Event-driven
workflows (CI-native review, incident response, outer loops) belong to
bitterblossom; see `meta/CONTRACTS.md`. Do not define static project
subagents here. Spawn roster/ad-hoc lanes from the active skill with a
role, scope, output shape, and boundaries. New primitives pass the
primitive test in `skills/harness-engineering/SKILL.md` (prompt vs skill vs
doctrine line vs Mode B).

## Hot Paths

- `harnesses/shared/AGENTS.md` — shared cross-harness doctrine.
- `meta/CONTRACTS.md` — Mode B boundary, trailer canon, shared disk state.
- `cargo run --locked -p harness-kit-checks -- bootstrap` — system-wide
  install (skills, prompts, configs); externals are vendored at registry pins.
- `cargo run --locked -p harness-kit-checks -- telemetry` — skill/prompt
  usage summarizer; consult before catalog changes.
- `bootstrap.sh` — curl-compatible launcher for the Rust bootstrap command.

## Red Lines

Harness Kit architecture constraints:

- Cross-harness first: Claude, Codex, Pi, Antigravity. Harness-native features are
  optimizations, not the primary layer.
- Skills are self-contained: scripts/libs/references live under the skill.
- No claim primitives under `skills/`.
- No semantic workflow engine around provider CLIs.
- No generated repo harness layer unless a shaped ticket proves it earns its
  complexity.
