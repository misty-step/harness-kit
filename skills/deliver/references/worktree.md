# Worktree Behavior

`/deliver` supports concurrent invocations across git worktrees with zero
interference and no global locks.

## State Root Resolution

State root is computed per-invocation as:

```
$(git rev-parse --show-toplevel)/.harness-kit/deliver/
```

In a linked worktree, `git rev-parse --show-toplevel` returns the
**worktree's** root, not the primary clone's. Every worktree has its own
`.harness-kit/deliver/` tree.

## Concurrent Worktrees

Two worktrees A and B of the same repository can each run `/deliver` on
different branches concurrently:

- Separate `<ulid>`s (ULIDs are process-local and monotonic; collisions
  are not possible)
- Separate state directories under each worktree's root
- Separate receipts
- No cross-worktree file contention

This is the whole reason claims were dropped: one local workspace per
delivery, coordination via git branches, not file locks.

## What's Not Supported

- **Same-worktree concurrent `/deliver`:** running `/deliver` twice in the
  same worktree on the same item exits 41 (double-invoke). Running on
  different items simultaneously works but the branch logic gets
  confusing — prefer serial execution.
- **Network filesystems with weak rename semantics:** the atomic
  checkpoint protocol assumes POSIX rename. NFS without `close-to-open`
  consistency may lose writes. Not a supported config.

## Verification

The `/deliver` oracle verifies:

> worktree-A and worktree-B each run `/deliver` on different branches
> concurrently; both produce independent merge-ready receipts.

If that check fails, the state-root resolution is broken — likely
someone hardcoded `$HOME/.harness-kit/` or the primary clone's `.git/`.
