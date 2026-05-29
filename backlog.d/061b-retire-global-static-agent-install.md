# Retire the global static-agent install (bootstrap allowlist + file deletion)

Priority: P1
Status: ready
Estimate: M

## Goal

Finish `061` (which shipped the roles-not-files doctrine + lens rubric). Stop
`bootstrap.sh` from installing `agents/*.md` globally **except the a11y trio**,
delete the now-redundant philosophy + role agent files, and migrate the
remaining `subagent_type: <role>` usages to ad-hoc dispatch. This is the
mechanical retirement that `061` deliberately deferred because it mutates
`~/.claude/agents` globally and removes files that back live `subagent_type`
usage.

## Non-Goals
- Do NOT re-litigate the doctrine (shipped in `061`).
- Do NOT remove the a11y trio (`a11y-auditor`, `a11y-fixer`, `a11y-critic`) —
  permission-isolation carve-out (`skills/a11y/SKILL.md`).
- Do NOT delete a role file until every live `subagent_type:` reference to it is
  migrated to ad-hoc dispatch (otherwise Claude-harness skills break).

## Constraints / Invariants
- Cross-harness: one bootstrap change covers claude/codex/pi/antigravity.
- User-owned agent files must be preserved (never delete a non-symlink,
  non-byte-identical file under `~/.*/agents`).
- `subagent_type: Explore|Plan|general-purpose` are native types — leave them.

## Repo Anchors (codex design lane 5d9d6443, captured)
- `bootstrap.sh:369-371` (local) + `:406-408` (remote) — `GLOBAL_AGENTS`
  discovery loops. Add an `is_global_agent_allowed` allowlist filter
  (`a11y-auditor|a11y-fixer|a11y-critic`) inside both.
- `bootstrap.sh:500` — `link_parent_dir "$HARNESS_KIT/agents"` symlinks the
  WHOLE dir. This is the crux: replace with `prepare_agents_dir` (turn a
  dir-symlink into a real dir) + filtered per-file `ln -sfn` over `GLOBAL_AGENTS`.
- `bootstrap.sh:504-507` (local per-agent), `:567-583` (remote) — install loops;
  same allowlist applies.
- New helper `cleanup_retired_agents`: remove stale Harness-Kit-managed symlinks
  (`ousterhout/carmack/grug/beck/cooper/planner/builder/critic`) and byte-identical
  copies (`cmp -s`); preserve modified/user-owned files with a warning.
- `bootstrap.sh:665` — the `Agents (...)` report then naturally shows only the trio.

## subagent_type migration (do FIRST)
- `grep -rn "subagent_type: \(critic\|builder\|planner\)" skills/` — migrate each
  to ad-hoc dispatch (name the role + scope + evidence; for critic, the milestone
  gate / lens rubric). Then the files are safe to delete.
- Philosophy personas already migrated in `061` (groom). Their name references in
  `code-review/references/{internal-bench.md,bench-map.yaml}`,
  `shape/references/critique-personas.md`, `research/references/delegate.md` are
  lens references — confirm each reads the rubric, not a deleted file.

## Oracle (Definition of Done)
- [ ] Fresh `bash bootstrap.sh` then `ls ~/.claude/agents` shows ONLY the a11y
      trio (+ any user-owned files) — not ousterhout/carmack/grug/beck/cooper/
      planner/builder/critic.
- [ ] Re-running bootstrap is idempotent and removes previously-installed
      non-allowlisted agent symlinks; user-owned files preserved.
- [ ] `agents/{ousterhout,carmack,grug,beck,cooper}.md` deleted (content lives in
      `harnesses/shared/references/lenses.md`); role files disposed per `061`.
- [ ] `grep -rn "subagent_type: \(critic\|builder\|planner\|ousterhout\|carmack\|grug\|beck\|cooper\)" skills/`
      returns nothing.
- [ ] `scripts/generate-index.sh` green; no dangling links to a deleted agent.
- [ ] `scripts/check-agent-roster.py` green (it counts agents); `dagger call
      check --source=.` passes.
- [ ] Smoke: a Claude skill that used a migrated `subagent_type` still dispatches
      a working ad-hoc critic.

## Risk + Rollout
- Wrong cleanup logic destroys user-owned agent files → use symlink-target check
  + `cmp -s` before any delete.
- Deleting a role file before migrating its `subagent_type` callers breaks the
  Claude harness → migration is step 1, deletion is last.
- Rollback: restore `agents/*.md` + the bootstrap loop (one revert).

## Related
- Completes `061` (doctrine + rubric shipped there). codex design lane
  `5d9d6443` has the exact bootstrap shell.
