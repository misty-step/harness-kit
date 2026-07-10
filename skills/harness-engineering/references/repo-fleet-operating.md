# Repo-Fleet Operating Primitives

Use this reference when the operator is managing many local repos as one
operating surface: inventory, VISION coverage, proof contracts, supervised loop
readiness, or bounded refactor ticks. This is a Mode A reference and template
set, not an engine.

## Primitive Test Result

Repo-fleet work earns Harness Kit source only at the primitive layer:

- **Registry format:** reference/template. It is machine-readable shared state
  that lets agents orient without re-inventing inventory on every run.
- **VISION scanner guidance:** skill/reference. It changes how `/vision` and
  `/harness-engineering` audit many repos; it is not a repo-local prompt.
- **Quality/evidence packet:** shared reference. Use `quality-system.md` and
  `verification-system-first.md`; do not add a separate ceremony tree.
- **Loop-readiness route:** shared handoff reference. Harness Kit decides and
  shapes; Mode B runs recurring/event work.
- **Refactor-loop lane card:** template. One bounded lane card is useful; a
  semantic workflow engine around agents is not.

Reject repo-fleet additions that require Harness Kit to schedule, retry,
prioritize, or semantically supervise many repos. That belongs in the Mode B
plane after `harnesses/shared/references/loop-readiness.md` passes.

## Registry Shape

Keep the registry boring JSON. The format is descriptive state for agents and
scripts, not a governance database.

```json
{
  "version": 1,
  "generated_at": "<ISO-8601 timestamp>",
  "source_documents": ["path/to/source-packet.md"],
  "projects": [
    {
      "slug": "harness-kit",
      "tier": "active",
      "canonical_path": "/abs/path/to/repo",
      "path_exists": true,
      "flags": {
        "active": true,
        "backburner": false,
        "archive": false,
        "private": false,
        "routine_loop_candidate": true
      },
      "gate_commands": ["cargo run --locked -p harness-kit-checks -- check --repo ."],
      "live_qa_routes": ["CLI happy path plus generated-doc drift check"],
      "side_effect_constraints": ["no deploys or external writes without explicit approval"],
      "progress_file": "/tmp/refactor-harness-kit.md",
      "source_of_truth_owner": {
        "project_contract": "VISION.md and AGENTS.md",
        "gate_contract": "repo-owned scripts and package metadata",
        "registry_record": "this JSON file"
      }
    }
  ]
}
```

Useful companion projection: a proof-contract file keyed by `slug` with
`canonical_local_gate_commands`, `focused_commands`, `live_qa_route`,
`evidence_path_receipt_policy`, `backlog_work_item_surface`, and
`external_side_effect_constraints`. Generate it from live repo evidence; do not
handwave missing gates as `ok`.

## VISION Coverage Scanner

For a fleet audit:

1. Read the registry source, then the live filesystem. Training data and prior
   packets are stale.
2. For each active project, check only the canonical root first:
   `<canonical_path>/VISION.md`.
3. If root `VISION.md` is missing, inspect nearby high-signal sources before
   proposing creation: `README*`, `AGENTS.md`, `CLAUDE.md`, package manifests,
   docs index, roadmap/backlog, product pages, and sibling `docs/VISION.md`.
4. Record dirty worktrees before edits. Do not create or normalize vision files
   in a dirty repo unless the operator explicitly scopes that repo.
5. Emit both human and machine receipts: Markdown summary plus JSON with counts,
   present roots, missing roots, stale non-root pointers, dirty repos, sources
   read, and exact verification commands.

The scanner may recommend cards or edits. It must not silently generate generic
VISION stubs across a fleet.

## Loop-Readiness Route

Harness Kit may shape three Mode A artifacts:

- inspect-only dry run: read registry + contracts, run no repo edits, produce an
  evidence packet and blockers;
- bounded refactor tick: one repo, one architectural pressure, one gate, one
  progress file, one stop rule;
- Mode B handoff packet: trigger, state path, verifier, evidence path, human
  boundary, halt behavior, and budget.

Harness Kit must not own the recurring runner. If the trigger is schedule,
webhook, PR-ready, incident, or unattended outer loop, hand off to the Mode B
plane with the fields from `loop-readiness.md`.

## Evidence Packet Minimum

For fleet operations, leave a compact receipt another agent can trust:

```markdown
# Repo-Fleet Evidence Packet

- Goal:
- Registry path and digest:
- Projects in scope:
- Sources read:
- Commands/routes exercised:
- Artifacts written:
- Changes made / none:
- Blockers and dirty repos:
- Mode A/Mode B decision:
- Residual risk:
```

## Refactor-Loop Tick Shape

A refactor-loop tick is a lane card, not a daemon:

- Target one repo and one branch/worktree.
- Name the architecture pressure and stop rule before edits.
- Use the registry gate plus any focused command as the fitness test.
- Write progress to the repo's configured progress file.
- Leave a lane receipt and an evidence packet.
- Stop after one meaningful milestone, repeated no-progress, broken gate, dirty
  unrelated user work, or reviewer-blocked architecture concern.

Use `skills/refactor/templates/refactor-loop-lane-card.md` as the lane-card
starter when a supervised tick is actually requested.
