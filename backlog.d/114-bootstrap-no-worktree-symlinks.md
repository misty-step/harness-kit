# Context Packet: Bootstrap must not leave symlinks into disposable worktrees

Priority: P2
Status: shaped
Estimate: S

## PRD Summary
- User: anyone whose `harness-kit-checks bootstrap` was ever run from inside a
  Codex worktree (`~/.codex/worktrees/<id>/harness-kit`).
- Problem: an old bootstrap symlinked `~/.harness-kit/scripts/*` and
  `~/.claude/hooks/*` into the worktree it was run from. When that worktree was
  cleaned up, **23 dangling symlinks** were left behind (4 roster scripts + 19
  Claude hooks, all → a deleted `ed05` worktree). The Python scripts/hooks were
  also rewritten into the `harness-kit-checks` Rust binary, so the symlinks are
  doubly obsolete. Active guardrails were unaffected (settings.json wires hooks
  via `harness-kit-checks claude-hook <name>`; roster validates via the binary),
  but the dangling cruft misled a diagnosis into thinking the roster was broken
  and violates the "no Codex-worktree dependency" doctrine.
- Goal: bootstrap leaves no dangling symlinks and never sources installed links
  from a disposable worktree; the migration self-heals on next run.
- Why now: surfaced live 2026-06-17 while diagnosing "agent feels stuck" in the
  Olympus repo. Already cleaned locally (trashed the 23 dangling symlinks); this
  ticket is the durable repo fix so it cannot recur / self-heals elsewhere.
- Success signal: after a bootstrap run, `find ~/.harness-kit ~/.claude -type l !
  -exec test -e {} \;` returns nothing, and no installed symlink resolves under
  `~/.codex/worktrees/`.

## Product Requirements
- P0: bootstrap prunes dangling symlinks in the dirs it manages
  (`~/.harness-kit/`, `~/.claude/hooks/`) — only dangling symlinks, never real
  files.
- P0: bootstrap refuses to (or warns + canonicalizes when it would) source an
  installed symlink from a path under `*/.codex/worktrees/*`; resolve the source
  to the canonical checkout instead.
- P1: drop the now-obsolete `link_dir_entries_if_present(harnesses/claude/hooks
  → ~/.claude/hooks)` step — canonical ships zero `.py` hooks (all moved into the
  `harness-kit-checks claude-hook` binary), so the step only creates stale links.
- Verification: a bootstrap unit/integration check that, given a managed dir
  seeded with a dangling symlink, asserts it is pruned; run
  `cargo run --locked -p harness-kit-checks -- check --repo .`.

## Notes
- Root cause file: `crates/harness-kit-checks/src/bootstrap.rs` (install_system_roster ~210, claude harness link ~348-359, the `link_or_replace` / `link_dir_entries_if_present` helpers, and the line-475 "old clone or worktree, or dangling" handling).
- Settings.json is COPIED (line 354), not symlinked, so re-running bootstrap overwrites `~/.claude/settings.json` — factor that into the migration so user customizations are not clobbered (or document it).
