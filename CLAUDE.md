# CLAUDE.md

## What This Repo Is

**Harness Kit** — focused workflow skills, specialized agents, and harness
configs for AI-assisted development. One repo, all harnesses, symlinked
to ~/.claude, ~/.codex, ~/.pi via bootstrap.sh.

## Structure

```
harness-kit/
├── skills/        # Leaf skills (qa, demo, diagnose, research, ...) and
│                  #   orchestrators (deliver, flywheel, code-review, settle, ...)
├── agents/        # Specialized agents: planner, builder, critic,
│                  #   ousterhout, carmack, grug, beck
├── harnesses/     # Per-harness configs, hooks, shared principles
├── registry.yaml  # External skill sources (for embeddings)
└── bootstrap.sh   # Discovers skills/agents from filesystem, symlinks to harness dirs
```

## Issue Tracking

**`backlog.d/` is the single source of truth.** Open work lives in
`backlog.d/NNN-<slug>.md`; closed work moves to `backlog.d/_done/`. No
external distributed tracker.

Closure is trailer-driven. Feature branches carry `Closes-backlog: NNN` /
`Ships-backlog: NNN` / `Refs-backlog: NNN` trailers on the squash-merge
commit to master — `/ship` injects them and archives the ticket file into
`_done/` before merge. `/groom` sweeps recent master commit trailers
against active `backlog.d/` to detect stale tickets; see
`.agents/skills/groom/SKILL.md`.

## Workflow

```
backlog.d/ → /groom → /shape → /deliver → ship
                              └─ /flywheel (outer loop: cycles of
                                  /deliver → /deploy → /monitor → /reflect)
```

`/deliver` is the inner-loop composer: one ticket → merge-ready code via
`/shape` → `/implement` → clean loop over `/code-review` + `/ci` +
`/refactor` + `/qa`. It stops at merge-ready; humans merge.

`/flywheel` is the outer-loop orchestrator (028): continuous, unattended,
budgeted cycles that compose `/deliver` as a black-box merge-readiness step
then `/deploy`, `/monitor`, `/diagnose` (on alert), `/reflect`, and
backlog/harness mutation.

## Principles

See `harnesses/shared/AGENTS.md` — one file, symlinked to every harness.

- **Cross-harness first** — every new mechanism works on Claude, Codex,
  AND Pi. Harness-native runtime features are optimizations on top of a
  filesystem-level base, not the base itself. Red Line. Primary layer is
  SKILL.md + what bootstrap symlinks into each harness's skills dir
- **Focused set of skills and agents** — resist bloat, justify additions
- **Thin harness, strong models** — don't compensate for weak models with scaffold
- **Gotchas > instructions** — enumerate what goes wrong
- **Description is the trigger** — write it assertively
- **Strip non-load-bearing scaffold** — stress-test after model upgrades
- **Map, not manual** — AGENTS.md and CLAUDE.md point to skills, not contain them

## Gotchas for Contributing to This Repo

- Skills encode judgment, not procedures. If the model already knows how, delete the skill.
- SKILL.md must be < 500 lines. Extract deep content to references/.
- Regexes over agent prose and semantic workflow DSLs are strong smells.
- Run `/harness lint` on any skill you create or modify.
- Run `/harness eval` to verify the skill actually improves output vs baseline.
- The pre-commit hook regenerates index.yaml — don't edit it manually.
- bootstrap.sh has two modes: symlink (local checkout) and download (remote).
  Test both paths if you change it.
- If you bootstrap from a temporary worktree, prefer pinning
  `HARNESS_KIT_DIR` to a stable checkout. `HARNESS_KIT_DIR` remains a legacy
  alias. Disposable worktree symlinks make global harness skills disappear later.
- `harnesses/claude/settings.json` is COPIED by bootstrap (Claude modifies it
  at runtime), not symlinked. Changes there need a re-bootstrap to take effect.
