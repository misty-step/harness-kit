# Receipt Contract

`/deliver` communicates with its caller — human or `/flywheel` outer
loop — exclusively via exit code and `receipt.json`. No stdout parsing,
no heuristic sniffing.

## Path

- Default: `<worktree-root>/.harness-kit/deliver/<ulid>/receipt.json`
- Override: `--state-dir <path>` (outer loop uses
  `backlog.d/_cycles/<ulid>/evidence/deliver/`)

The entire `.harness-kit/` tree is gitignored.

## Schema

```json
{
  "schema_version": 1,
  "ulid": "01JXXX...",
  "item_id": "<item-id>",
  "branch": "feat/<item-slug>",
  "base_sha": "abc123...",
  "head_sha": "def456...",
  "status": "merge_ready | clean_loop_exhausted | phase_failed | aborted",
  "phases": [
    {"name": "shape",     "status": "ok"},
    {"name": "implement", "status": "ok"},
    {"name": "code-review", "status": "dirty", "iteration": 3, "blocking_count": 2},
    {"name": "ci",        "status": "ok"},
    {"name": "qa",        "status": "p1", "findings": 1}
  ],
  "evidence": {
    "review_dir": "<state-dir>/review/",
    "evidence_dir": ".evidence/<branch>/<date>/",
    "ci_dir": "<state-dir>/ci/",
    "qa_dir": ".evidence/<branch>/<date>/",
    "demo_mirror": "https://github.com/org/repo/releases/tag/qa-evidence-..."
  },
  "remaining_work": ["code-review: 2 blocking findings in auth.py"],
  "recommended_next": "fix-and-resume | abandon | human-review",
  "cost_usd": 2.80
}
```

## Exit Codes

| Code | Meaning | Receipt `status` |
|---|---|---|
| 0 | Merge-ready | `merge_ready` |
| 10 | Phase handler hard-failed (tool/infra error, not dirty output) | `phase_failed` |
| 20 | Clean loop exhausted (3 iterations, still dirty) | `clean_loop_exhausted` |
| 30 | User/SIGINT abort | `aborted` |
| 40 | Invalid args / missing dependency skill | `phase_failed` |
| 41 | Double-invoke: item already delivered (state exists, merge_ready) | `phase_failed` |

## State Lifecycle

```text
  ┌────────┐   phase ok    ┌────────────┐
  │ shape  │──────────────▶│ implement  │──────────▶ (clean loop)
  └────────┘               └────────────┘                │
       │                        │                        │
       │ phase fail             │ phase fail             ▼
       ▼                        ▼                   all green
  exit 10                  exit 10                  exit 0 (merge_ready)
                                                         │
                                 clean loop, cap hit     │
                                 exit 20                 │
                                                         ▼
                                                   receipt.json
```

`status` transitions are monotonic — once `merge_ready` is written, the
state is frozen. Re-invoking on the same item returns exit 41.

## Caller Consumption

- **Human:** `cat <state-dir>/receipt.json | jq` — read `status`,
  `remaining_work`, `recommended_next`.
- **`/flywheel` outer loop:** reads `receipt.json`, emits one
  `deliver.done` event. Exit 0 → proceed to deploy phase. Non-zero →
  halt cycle with `phase.failed` event.
