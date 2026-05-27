# AGENTS.md — Spellbook

Spellbook is the harness source repo. Keep this file terse: repo-specific
contracts only. Put workflow detail in skills; put generated/runtime state on
disk; do not restate obvious filesystem facts.

## Non-Negotiables

- Base branch: `master`.
- Gate: `dagger call check --source=.`. Green means all 15 Spellbook CI lanes
  pass. `/ci` owns the exact lane list in `ci/src/spellbook_ci/main.py`.
- Clean-tree closeout: no run is complete while
  `git status --short --untracked-files=all` shows paths. Commit, delete,
  move out, or ignore every file. Treat untracked `backlog.d/NNN-*.md` as
  signal unless the user explicitly says scratch/delete.
- `index.yaml` is generated. Never edit it by hand.
- `harnesses/claude/settings.json` is copied by bootstrap; changes require
  re-bootstrap.
- Spellbook source skills live only in `skills/`. Do not commit source-repo
  `.agents/skills/`, `.codex/skills/`, `.claude/skills/`, or `.pi/skills/`
  bridges; those are `/seed` output for consumer repos and duplicate the global
  install here.

## Roster Floor

The provider roster is repo-local at `.spellbook/agents.yaml` when present and
system-wide at `~/.spellbook/agents.yaml` otherwise. For substantive research,
design, implementation, QA, diagnosis, review, backlog, reflection, or harness
work:

- probe the roster;
- dispatch two or more available providers;
- record sanitized receipts in `.spellbook/traces/delegations.jsonl`;
- synthesize as lead; provider output is evidence, not authority.

Direct solo work is allowed only for mechanical commands, emergency unblocks,
explicit user-forbidden delegation, or fewer than two available providers.
Every final report includes a short roster report grounded in receipts, never
raw transcripts.

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
nontechnical team, read `docs/positioning.md`. Spellbook is implementation
substrate for technical operators, not the buyer-facing governed workflow
package or admin-control plane.

## Root Skills

Use these for harness work:

- `/harness`: mutate Spellbook primitives, gates, roster, doctrine.
- `/yeet`: classify, commit, push; clean tree is the deliverable.
- `/ship`: final-mile landing and backlog closure.
- `/deliver`: one shaped item to merge-ready; no push, no merge.

Do not define static project subagents here. Spawn roster/ad-hoc lanes from
the active skill with a role, scope, output shape, and boundaries.

## Hot Paths

- `harnesses/shared/AGENTS.md` — shared cross-harness doctrine.
- `.spellbook/agents.yaml` / `~/.spellbook/agents.yaml` — provider roster and
  default commands.
- `scripts/check-agent-roster.py` — roster + delegation-floor gate.
- `docs/copy/site.json` — public-facing docs companion copy and icon map.
- `scripts/build-docs-site.sh` — regenerates `docs/site/` from live repo sources.
- `scripts/check-docs-site.sh` — generated docs companion drift and oracle gate.
- `scripts/record-delegation.py` / `scripts/summarize-delegations.py` —
  receipt capture and operator report.
- `skills/harness/SKILL.md` — harness mutation contract.
- `bootstrap.sh` — system-wide install; all first-party skills are global.

## Red Lines

- Cross-harness first: Claude, Codex, Pi. Harness-native features are
  optimizations, not the primary layer.
- Skills are self-contained: scripts/libs/references live under the skill.
- No claim primitives under `skills/`.
- No semantic workflow engine around provider CLIs.
- No generated repo harness layer unless a shaped ticket proves it earns its
  complexity.
