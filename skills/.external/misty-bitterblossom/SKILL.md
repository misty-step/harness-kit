---
name: bitterblossom
description: |
  Operate Bitterblossom's `bb` event-plane CLI for agent workloads. Use when
  Codex needs to inspect or run `bb`, configure `plane.toml`, `agents/`, or
  `tasks/`, dispatch or audit tasks, handle runs, dead letters, recovery,
  parked tasks, submissions, gates, review-factory workflows, or help another
  repo consume Bitterblossom. Trigger phrases: "Bitterblossom", "bb",
  "event plane", "agent workload", "run a task", "inspect runs", "DLQ",
  "parked task", "submission loop", "review factory".
---

# Bitterblossom

Operate the event plane. Do not move workload judgment into the plane.

Bitterblossom is `bb`: tasks + agents + triggers as files, with a durable run
ledger, budgets, retries, dead letters, and optional webhook/cron serving.
Agents are CLI users, so prefer stable `--json` surfaces over prose parsing.
When working in the Bitterblossom source checkout, read `VISION.md` for the
runtime-vs-workload boundary before adding or reshaping tasks.

## Stance

- Treat `bb` as runtime and ledger, not an agent brain.
- Workloads are config: `plane.toml`, `agents/<name>.toml`,
  `tasks/<name>/task.toml`, and `tasks/<name>/card.md`.
- New workload behavior should be a task/card/agent change, not a Rust branch.
- Use `--config <plane>` explicitly unless the user has set `BB_PLANE_DIR`.
- Use `--json` for agent-readable output. Text is for humans.
- Human-mode `bb run` prints an early run id and periodic stderr heartbeats;
  `bb run --json` stays quiet until it emits the final run bundle.
- If `bb` is not on `PATH`, use `cargo run --quiet --` from the source repo or
  `./target/debug/bb` after `cargo build`.

## First Probe

Run these before changing or dispatching anything:

```bash
bb --config <plane> check
bb --config <plane> task list --json
bb --config <plane> runs list --json
```

Read the output for:

- loaded tasks and agent versions;
- parked tasks and budget ceilings;
- recent failures, dead letters, in-flight cap kills, and costs;
- whether a task is reflex (`webhook`/`cron`) or dispatch (`manual`).

## Authority And Readiness

Classify the run before dispatch. Mode B work belongs in Bitterblossom, but the
allowed authority depends on trigger source, auth class, and task policy.

| Mode | Allowed | Refuse | Readiness |
|---|---|---|---|
| Supervised dispatch | Operator-initiated `bb run`/submission work with the task's declared side effects; subscription-auth builders only when the task is manual and the operator asked for that authority. | Hidden cron/webhook dispatch, unbounded continuation, merge/deploy/recovery authority not named by the task or operator. | `bb --config <plane> check`; `bb --config <plane> task list --json`; `bb --config <plane> preflight <task> --json`; `bb --config <plane> status --json`. |
| Unsupervised reflex | Webhook/cron API-auth work inside declared repo/org filters, budgets, required artifacts, authority, and side-effect policy. | Subscription auth, broad repo discovery, secret values in payloads, merge/deploy/run-resolution authority, or any action beyond the task card. | `bb --config <plane> check`; inspect task policy and budget in `task list --json`; run `preflight`; inspect `status --json`, `dlq list --json`, and `notify list --json`. |
| Read-only inspection | MCP/CLI/API reads for status, config, runs, gates, dead letters, and artifacts. | Any `tools/call` mutation, dispatch, replay, resolve, park/unpark, submit, merge, or credential provisioning over MCP. | Prefer MCP tools when available; fall back to `bb ... --json`; use `bb_artifacts_list`/`bb_artifact_read` or `bb artifacts list/read` for evidence. |

A closeout receipt is incomplete until it names the exact command/surface used,
run id or submission id, ledger state, cost/budget status, side effects,
artifact reads via MCP or `bb artifacts list/read`, and residual risk.

## Rollout Scorecards

Autonomous task families ship at less than full authority (read-only,
report-only, dry-run, PR-only) and climb an authority ladder only on evidence.
The single reusable scorecard template and promotion doctrine live in
[`docs/rollout-scorecards.md`](../../docs/rollout-scorecards.md).

Refuse autonomy expansion from vibes. Do not recommend or take a higher
authority action (open a branch/PR, merge, deploy, resolve, unpark) for a task
family unless that level's scorecard is green *and* an operator has approved the
promotion. "It has been working" is not evidence; the scorecard is. Green
metrics only make the next-level ticket eligible for explicit operator approval;
they never flip authority by themselves. Merge, unpark, production mutation, and
broad rollout stay operator authority until a scorecard plus an operator
decision says otherwise.

## Route

| Need | Use |
|---|---|
| Validate config and see loaded agents/tasks | `bb --config <plane> check` |
| Decision-ready task/run/DLQ health | `bb --config <plane> status --json` |
| Task inventory, parked state, budgets | `bb --config <plane> task list --json` |
| Trigger manual work | `bb --config <plane> run <task> --payload '<json>' --json` |
| Dispatch a manual builder lane | `bb --config <plane> run build --payload '{"backlog":"<id-or-path>"}' --json` |
| Diagnose failed CI | `bb --config <plane> run ci-diagnose --payload '{"repo":"owner/repo","head_sha":"<sha>"}' --json` |
| Run a report-only lifecycle reflex | `bb --config <plane> run <reflex> --payload-file EVENT.json --json` where `<reflex>` is `fix-prompt`, `deploy-prod-verify`, `canary-triage`, `backlog-chewer-dry-run`, or `lifecycle-orchestrator` (each writes `REPORT.json`, no mutation; see `docs/rollout-scorecards.md`) |
| Compare candidate model configs | Run at least three candidate tasks, then `bb --config <plane> run model-eval --payload '<json>' --json` |
| Inspect ledger | `bb --config <plane> runs list --json`; `bb --config <plane> runs show <id> --json` |
| Export run telemetry | `bb --config <plane> runs export` (`bb.run_telemetry.v1` JSONL) |
| Inspect run artifacts | `bb --config <plane> artifacts list <run-id> --json` (top-level artifact files); `bb --config <plane> artifacts read <run-id> REPORT.json` (safe text/JSON read, including known nested relative paths; binary, oversized, and traversal paths refused) |
| Handle pre-execute failures | `bb --config <plane> dlq list --json`; `bb --config <plane> dlq replay <id> --json`; `bb --config <plane> dlq ack <id> --reason <text> --json` to close a superseded DLQ |
| Handle notification failures | `bb --config <plane> notify list --json`; `bb --config <plane> notify retry --json`; `bb --config <plane> notify ack <id> --reason <text> --json` |
| Provision scoped OpenRouter child keys | `bb --config <plane> keys mint <agent> --json`; `bb --config <plane> keys rotate <agent> --json`; `bb --config <plane> keys revoke <agent> --json`; `bb --config <plane> keys list --remote --json`; `bb --config <plane> keys sync --all --check --json` |
| Park or unpark workload dispatch | `bb --config <plane> task park|unpark <task>` |
| Classify inherited running rows after host restart | `bb --config <plane> recover` |
| Run webhook/cron plane | `bb --config <plane> serve` |
| Submission storm / review factory | `bb submit ...`, verdict `bb run <kind> ...`, then `bb gate --json` |
| Read-only inspection for agents over MCP | `bb --config <plane> mcp serve` (stdio JSON-RPC; `bb_status`, `bb_check`, `bb_tasks`, `bb_runs_list`, `bb_runs_show`, `bb_artifacts_list`, `bb_artifact_read`, `bb_dlq_list`, `bb_preflight`, `bb_gate`) |

Detailed command recipes: `references/operator-recipes.md`. Authority-promotion
rules and per-task scorecards: [`docs/rollout-scorecards.md`](../../docs/rollout-scorecards.md).

## Dispatch Rules

- A `bb run` can have external side effects. Do not blindly re-run a successful
  or executing task.
- Secrets travel through declared env/secrets and stdin plumbing. Never put
  tokens in argv, task cards, or payload JSON unless the task contract explicitly
  says the value is non-secret.
- For GitHub-backed runs, prefer `GH_TOKEN=$(gh auth token) bb ...` over copying
  tokens into shell history.
- Reflex triggers must use API-auth agents. Subscription-auth agents belong to
  manual dispatch only.
- The checked-in `build` task is a manual API-auth OMP/GLM builder lane. Use
  it for shaped implementation work only; it creates/pushes a branch and report
  but does not merge or replace the submission gate.
- For uncertain model fit, run a cohort: at least three materially different
  candidate tasks for the same flow and payload, then the `model-eval` task.
  First-class cohorts for `build`, `review`, `gardener`, `ci-diagnose`, and
  submission-storm members are listed under
  [`docs/model-evals/`](../../docs/model-evals/README.md). Record accepted
  findings there before promoting a new default.
- A parked task is intentionally blocked. Inspect the reason before `unpark`.
- Dead letters are pre-execute failures. At/after execute, use operator
  resolution paths because the run may have side effects.
- Streaming harness usage is metered while the run is executing. If
  `max_cost_per_run_usd` is breached, the agent policy chooses `kill`,
  `quarantine`, or `log`; `kill` and `quarantine` must emit notification
  outbox rows such as `run_in_flight_cap_killed`.

## Submission Storm

Verdict tasks (`correctness`, `security`, `product`, `simplification`,
`arbiter`, `verify`) expect a submission payload. Do not call them with an
arbitrary repo/rev payload.

Shape:

```bash
bb --config <plane> submit open --change <change> --rev <rev> --json
bb --config <plane> run correctness --payload '{"submission":"<submission>"}' --json
bb --config <plane> gate --submission <submission> --json
```

If a verdict task fails with `payload has no 'submission' field`, the plane is
correct; the invocation was wrong.

If `bb gate --json` reports a canonical member as `run:failure`, read that
member's `safe_next_command`. Replays prove the failed pre-execute path can run,
but they do not count for the canonical gate key; the safe recovery is a clean
replacement submission after fixing the operator or infrastructure issue. The
command includes the loaded plane's `--config` path so an agent can run it from
another cwd.

## Recovery

- `bb recover` classifies inherited `running` rows after a host restart.
- `bb recover --json` exposes `probe_state`, `probe_reason`,
  `lease_disposition`, and `operator_action`; unknown probes retain the host
  lease.
- `bb runs resolve` is for `awaiting_recovery` after side-effect inspection.
- `bb status --json` marks `awaiting_recovery` older than one hour as
  `escalate_stale_recovery`, with age fields, but the operator still resolves
  the run only after inspecting side effects.
- `bb dlq replay --json` mints a new run linked to a pre-execute dead letter
  and returns the replayed run bundle.
- `bb dlq ack <id> --reason <text> --json` acknowledges a superseded
  pre-execute dead letter without replaying it, recording reason + timestamp.
  Acknowledged DLQs cannot be replayed; `bb status --json` no longer counts
  them as open operator work.
- `bb preflight <task> | --storm --json` checks missing declared secrets,
  missing policy-bound provider keys, and unspawnable `command`-harness
  binaries before dispatch creates run rows.

## Serving

`bb serve` exposes webhook ingress, cron scheduling, the HTML operator view, and
read APIs. Do not bind publicly without `BB_API_TOKEN`; the server refuses
non-loopback binds without it.

Useful API mirrors:

- `GET /api/tasks`
- `GET /api/status`
- `GET /api/runs`
- `GET /api/runs/<id>`
- `GET /api/dlq`
- `GET /api/notify`
- `GET /api/submissions`

## MCP (read-only)

`bb --config <plane> mcp serve` runs a read-only MCP stdio server: JSON-RPC
2.0 over stdin/stdout, no network listener, no external credentials for
local-plane inspection. Consume it MCP-first where a host agent supports it;
fall back to `bb ... --json` for anything the MCP tool table does not yet
cover. The registered read tools (`bb_status`, `bb_check`, `bb_tasks`,
`bb_runs_list`, `bb_runs_show`, `bb_artifacts_list`, `bb_artifact_read`,
`bb_dlq_list`, `bb_preflight`, `bb_gate`) return the same shapes as their
CLI/API counterparts — MCP is a typed adapter, not a second implementation.

This slice ships read-only tools only. No mutating MCP tool exists; a
`tools/call` for an unknown or would-be mutating name is rejected with a
JSON-RPC `-32602` error. Do not expect `bb runs cancel`, `bb dlq replay`,
`bb run`, or submission/gate mutations over MCP until a separate
writable-MCP backlog lands its graduation signals.

Routing:

| Need | First choice | Fallback |
|---|---|---|
| Decision-ready plane health | MCP `bb_status` | `bb status --json` |
| Config/task inventory | MCP `bb_check`, MCP `bb_tasks` | `bb check --json`; `bb task list --json` |
| Runs | MCP `bb_runs_list`, MCP `bb_runs_show` | `bb runs list --json`; `bb runs show <id> --json` |
| Run artifacts | MCP `bb_artifacts_list`, MCP `bb_artifact_read` | `bb artifacts list <id> --json`; `bb artifacts read <id> <path> --json` |
| Dead letters | MCP `bb_dlq_list` | `bb dlq list --json` |
| Pre-dispatch readiness | MCP `bb_preflight` | `bb preflight <task> --json` |
| Submission gate evaluation | MCP `bb_gate` | `bb gate --change <key> --json` |
| Submission mutation | (not yet MCP) | `bb submit ... --json` |

## Operator Dispatch Loop

Use the ergonomic dispatch surface when the operator wants `bb` to own a
supervised lane instead of running a bare shell:

```bash
bb --config <plane> dispatch --repo <path> --brief <file> [--model <slug>] [--label <text>]
bb --config <plane> logs -f <run-id>
```

`dispatch` prints only the accepted run id and exits. It chooses the task from
`BB_DISPATCH_TASK`, then `dispatch`, then `build`, then a single manual task if
that is the only unambiguous option. The brief file is read at dispatch time and
persisted into the run payload as `bb.dispatch_job.v1` with `repo`, canonical
`prompt`, optional `model`, `label`, and `branch_slug`; the brief file path
itself is not persisted. `model`, when provided, overrides the selected task
agent's model for that run. It does not run synchronously; a running local or
deployed plane drains the pending run. `logs -f` follows ledger events plus
released text artifacts until the run reaches a terminal state.

## Distribution

The portable artifact is this whole folder: `skills/bitterblossom/`. Consumers
should copy or symlink the folder, not just `SKILL.md`, so references and agent
metadata travel with it.

Harness Kit integration should keep one source of truth. Prefer a source entry,
bootstrap projection, or explicit symlink from Harness Kit to this folder over a
manual copied skill that can drift. The durable decision is
`docs/adr/006-skill-projection.md`.

## Closeout Evidence

When using Bitterblossom, report:

- exact plane path and `bb` binary used;
- commands run and relevant run ids;
- ledger state, costs, parked/DLQ status, and external side effects;
- generated artifacts, inspected through `bb artifacts list/read` rather than local path spelunking, plus any API/CLI JSON read;
- residual risk, including failed probes that remain in the ledger.
