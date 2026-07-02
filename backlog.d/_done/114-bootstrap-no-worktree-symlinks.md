# Context Packet: Bootstrap must not leave symlinks into disposable worktrees

Priority: P2 (shipped)
Status: done
Estimate: S
Shipped: 2026-07-02

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
- [x] P0: bootstrap prunes dangling symlinks in the dirs it manages
  (`~/.harness-kit/`, `~/.claude/hooks/`) — only dangling symlinks, never real
  files. (Skills/agents dirs already had this via `cleanup_symlinks_under_prefix`;
  extended to `~/.claude/hooks/` — the actual dir the 23 dangling links lived in.
  `~/.harness-kit/scripts/*` linking no longer exists in the current Rust
  bootstrap at all — verified via grep, nothing to prune there anymore.)
- [x] P0: bootstrap refuses to (or warns + canonicalizes when it would) source an
  installed symlink from a path under `*/.codex/worktrees/*`; resolve the source
  to the canonical checkout instead. (Chose the documented "warns" alternative
  over silent auto-redirect: `is_disposable_worktree_path` detects the pattern
  and prints a loud warning naming the risk, rather than guessing which
  checkout is "canonical" — no registry of that exists and guessing wrong would
  be worse than warning. A hard refusal was rejected too: it would block
  legitimate worktree-based CI/testing of bootstrap itself.)
- [x] P1: drop the now-obsolete `link_dir_entries_if_present(harnesses/claude/hooks
  → ~/.claude/hooks)` step — canonical ships zero `.py` hooks (all moved into the
  `harness-kit-checks claude-hook` binary), so the step only creates stale links.
  (Function deleted entirely — its only call site is what got replaced by the
  self-healing cleanup above.)
- [x] Verification: a bootstrap unit/integration check that, given a managed dir
  seeded with a dangling symlink, asserts it is pruned; run
  `cargo run --locked -p harness-kit-checks -- check --repo .`.
  (3 new tests: `detects_disposable_codex_worktree_paths`,
  `cleanup_symlinks_under_prefix_prunes_dangling_hook_links` — exactly the
  seeded-dangling-symlink fixture this line asks for — plus live end-to-end
  proof against a scratch `$HOME`: seeded a real dangling symlink pointing at
  a nonexistent `.codex/worktrees/.../harnesses/claude/hooks/...` path,
  compiled binary bootstrap against that `$HOME` removed it and printed
  `removed stale <name>`; separately, running bootstrap with `--repo` pointed
  at a real copied checkout under a `.codex/worktrees/` path printed the
  WARNING line verbatim.)

## Notes
- Root cause file: `crates/harness-kit-checks/src/bootstrap.rs` (install_system_roster ~210, claude harness link ~348-359, the `link_or_replace` / `link_dir_entries_if_present` helpers, and the line-475 "old clone or worktree, or dangling" handling).
- Settings.json is COPIED (line 354), not symlinked, so re-running bootstrap overwrites `~/.claude/settings.json` — factor that into the migration so user customizations are not clobbered (or document it).

**2026-07-02 — shipped; settings.json clobber note deliberately NOT addressed
here.** That note describes different, pre-existing `copy_if_present`
behavior unrelated to dangling worktree symlinks and wasn't in this ticket's
P0/P1 scope — changing copy-vs-preserve semantics for a config file agents
and operators both edit deserves its own shaped ticket with real design
(merge? diff-and-confirm? `<!-- keep -->` fences?), not a rushed addition
here. `crates/harness-kit-checks/src/bootstrap.rs` root-cause line numbers in
this ticket are stale post-refactor (backlog 133 split `install_cli` into
`cli_install.rs` earlier tonight, shifting everything below it); current
locations: `install_system_roster` ~243, claude harness link block ~336,
`link_or_replace`/`cleanup_symlinks_under_prefix` ~570s/~494, the "old clone
or worktree, or dangling" comment ~502.
