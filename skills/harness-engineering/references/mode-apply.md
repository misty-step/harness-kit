# /harness-engineering apply

Apply a `/reflect` learning packet to Harness Kit primitives. This is a
manual harness-mutation mode, not a default `/deliver` step.

## Boundary

`/reflect` emits proposals. `/harness-engineering apply` validates, plans, and
applies accepted proposals on a harness branch.

Use this before creating a standalone `/harness-apply` skill. Promote only
after at least three successful apply runs plus one evidence-cited refusal.

## Inputs

Accept a packet from `/deliver`, `/reflect distill`, `/reflect cycle`, or
`/reflect prompt-debt`.

```yaml
source: deliver|distill|cycle|prompt-debt
packet_id: <id>
packet_path: <path>
operations:
  - action: create|update|delete|move|backlog-create|gate-add|eval-add
    path: <repo-relative path>
    evidence_ref: <commit|diff|receipt|log|artifact>
    rationale: <why this mutation prevents recurrence>
    codification_target: type|lint|hook|test|ci|skill|agents|memory
```

## Outputs

```yaml
branch: harness/apply-<packet-id>
applied:
  - path:
    action:
    verification:
rejected:
  - path:
    reason:
    evidence_ref:
deferred:
  - path:
    reason:
    followup:
receipts_dir: .harness-kit/traces/apply/<packet-id>/
```

## Refuse Conditions

Refuse instead of mutating when any condition holds:

- target path is outside `skills/`, `agents/`, `harnesses/`, root
  `AGENTS.md`, `CLAUDE.md`, hook scripts, or `backlog.d/`;
- target is under `backlog.d/_done/`;
- current branch is `master`/`main` or detached;
- packet lacks an evidence ref;
- operation deletes without proof of obsolescence;
- operation edits a skill used in the same cycle before that cycle is fully
  closed;
- a higher codification layer fits but the proposal targets prose;
- new mechanism lacks a gate, eval, smoke path, or explicit follow-up;
- worktree is dirty before apply starts.

## Protocol

1. Validate packet schema and evidence refs.
2. Create or switch to `harness/apply-<packet-id>`.
3. Classify each operation: apply, reject, or defer.
4. For skill create/update/delete, prefer
   `cargo run --locked -p harness-kit-checks -- skillify-skill-crud ...`
   when it fits.
5. For gates/evals/hooks, implement the smallest enforceable layer first.
6. Dispatch fresh critic lanes for hierarchy and regression risk before final
   acceptance.
7. Run the narrow validators, then Dagger before merge.

## Verification

Run the checks matching touched surfaces:

```bash
cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .
cargo run --locked -p harness-kit-checks -- check-skill-evals --repo .
cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .
cargo run --locked -p harness-kit-checks -- check-runtime-primitives --repo .
dagger call check --source=.
```

Emit the Post-Sync Acceptance block from `/harness-engineering` with packet ref,
applied/rejected/deferred operations, exact commands, and residual risk.
