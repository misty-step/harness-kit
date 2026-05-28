# Durability & Resume

State is filesystem-backed, worktree-local, and resumable. SIGKILL, power
loss, and mid-phase crashes are recoverable via `--resume <ulid>`.

## State Layout

```
<state-dir>/
├── state.json      # current_phase, completed_phases, item_id, branch, ulid
├── receipt.json    # written at exit — see receipt.md
├── review/         # /code-review transcripts
└── ci/             # /ci logs
```

Default `<state-dir>` = `<worktree-root>/.harness-kit/deliver/<ulid>/`.

## Checkpoint Protocol (atomic)

After every phase completes, `/deliver` rewrites `state.json` atomically:

1. Serialize new state → `state.json.tmp`
2. `fsync` the file
3. `rename state.json.tmp state.json`
4. `fsync` the parent directory

This is the POSIX atomic-rename guarantee: on any crash, `state.json` is
either the previous consistent state or the new one — never a torn write.

## Phase Idempotency

Every phase skill must be idempotent on partial runs:

- `/implement` re-runs tests on already-green code cheaply
- `/code-review` re-reviews the current diff
- `/ci` re-runs dagger (cached where possible)
- `/qa` re-drives the running app

This is a contract on phase skills, not a guarantee `/deliver` provides.
See `/implement` and `/ci` skill docs for explicit idempotency clauses.

## `--resume <ulid>`

1. Load `<state-dir>/state.json`
2. Skip phases in `completed_phases`
3. Re-enter at `current_phase`
4. Continue normally

## `--abandon <ulid>`

1. Remove `<state-dir>` entirely
2. Leave the feature branch as-is (unpushed, uncommitted changes if any)
3. Exit 0

The human can delete the branch themselves, or re-use it for a fresh
`/deliver` invocation.

## Double-Invocation (exit 41)

`/deliver <item-id>` when a state-dir exists for that item with
`status: merge_ready`:

- **Exit 41** with message: "already delivered; use `--resume <ulid>` or
  `--abandon <ulid>` or switch to a fresh branch"
- No silent re-run. No no-op. Surfaces the ambiguity.

This catches the common footgun of re-invoking after a successful
delivery and getting a confusing "why is this doing nothing" experience.

## Interruption Guarantees

| Event | Guarantee |
|---|---|
| SIGINT | Trap writes `status: aborted`, exits 30 |
| SIGKILL | `state.json` remains last consistent checkpoint |
| Power loss | Same as SIGKILL — no torn writes |
| Mid-phase crash | `current_phase` records in-flight phase; resume re-runs it |

Resume-after-SIGKILL is part of the oracle — a test kills `/deliver` mid
`/code-review`, then `/deliver --resume <ulid>` completes delivery.
