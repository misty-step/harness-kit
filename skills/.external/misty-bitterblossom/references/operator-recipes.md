# Bitterblossom Operator Recipes

Use these recipes when the top-level skill points here. Replace `<plane>` with a
directory containing `plane.toml`.

## Resolve `bb`

```bash
bb --version
```

If `bb` is not installed and you are in the Bitterblossom source checkout:

```bash
cargo run --quiet -- --config <plane> check
```

After building:

```bash
cargo build
./target/debug/bb --config <plane> check
```

## Inventory

```bash
bb --config <plane> check
bb --config <plane> check --json
bb --config <plane> status --json
bb --config <plane> task list --json
bb --config <plane> runs list --json
bb --config <plane> dlq list --json
```

Read `status --json` first when triaging. It clusters tasks, recent run states,
cost, in-flight cap enforcement, queue age, parked reasons, DLQ rows, and safe
next actions. Use raw `task list`, `runs list`, and `dlq list` when you need the
underlying rows.

## Manual Dispatch

```bash
bb --config <plane> run <task> \
  --idempotency-key "<stable-key>" \
  --payload '{"key":"value"}' \
  --json
```

Use an idempotency key for replayable operator actions. Treat a returned run id
as the receipt, then inspect it:

```bash
bb --config <plane> runs show <run-id> --json
```

For human-supervised long runs, omit `--json`: `bb run` prints the run id
immediately and emits heartbeat lines on stderr while the run is pending or
running. Keep `--json` for agents and scripts; it stays quiet until the final
`run`/`attempts`/`events` bundle is ready.

## Builder Dispatch

Use the checked-in `build` task only for shaped implementation work. It binds a
manual API-auth OMP/GLM builder agent, so it must not be attached to webhook or
cron triggers.

```bash
OPENROUTER_API_KEY=<key> GH_TOKEN=$(gh auth token) bb --config <plane> run build \
  --idempotency-key "build:<backlog-or-packet>:<date>" \
  --payload '{"repo":"misty-step/bitterblossom","backlog":"backlog.d/060-builder-dispatch-role.md","branch_slug":"builder-dispatch-role"}' \
  --json
```

Expected side effects: a pushed `bb/build/<slug>` branch and a builder report
in the run result. The builder never merges. Submit and gate the produced rev
with the normal submission loop.

## Scoped OpenRouter Keys

For agents with `[policy] provider_key_name` and
`provider_spend_cap_usd`, mint a child key with a provider-side cap before
dispatching. The management key must come from the environment by name and must
not be copied into argv, payloads, cards, or logs:

```bash
bb --config <plane> keys mint <agent> --json
bb --config <plane> keys list --remote --json
bb --config <plane> keys sync --all --check --json
```

The remote list output should show the minted key hash/name with `limit` equal
to the agent policy cap. The sync command refreshes local non-secret usage/cap
metadata and exits non-zero if a provider-side limit, disabled state, or missing
key drifts from policy. `bb keys rotate <agent> --json` creates a replacement key
with the current cap and revokes the old one; `bb keys revoke <agent> --json`
disables the stored child key and clears local key material. Policy-bound
OpenRouter agents do not fall back to a shared `OPENROUTER_API_KEY`.

## In-Flight Cap Enforcement

For harnesses that stream usage, `bb` meters partial attempt cost while the
process is still running. A `max_cost_per_run_usd` breach follows the agent's
`[policy].side_effect_policy`:

```toml
[policy]
side_effect_policy = "kill" # kill | quarantine | log
```

`kill` terminates the running harness and emits a
`run_in_flight_cap_killed` notification outbox row. `quarantine` moves the run
to `awaiting_recovery` and emits `run_in_flight_cap_quarantined`. `log` records
the observed breach and lets the run continue. Check `bb status --json` for
`guards.in_flight` and `tasks[].budget.cost_enforcement`.

## Ledger Export

Export run and attempt telemetry as versioned JSONL for downstream analysis:

```bash
bb --config <plane> runs export
```

Each line uses the `bb.run_telemetry.v1` contract documented in
`docs/run-telemetry-export-v1.md`. `runs export` currently takes only
`--config`; use `runs list --json` and `runs show --json` for filtered
interactive inspection.

## Review Workload

The checked-in `review` task is a Cerberus-backed command workload. `bb` owns
trigger filtering, dedupe, dispatch, budget/parking, and receipts; Cerberus owns
the review request/artifact contract and GitHub projection. Org-wide rollout is
controlled in `plane/tasks/review/task.toml`: owner allowlist, draft/action
filters, bot denial, size/file caps, one Cerberus run per PR head SHA, and task
budget. The submission-storm gate is a separate workflow; the org review reflex
does not fan out storm members.

For a real review comment:

```bash
GH_TOKEN=$(gh auth token) bb --config <plane> run review \
  --payload '{"repo":"owner/repo","pr":123}' \
  --json
```

For measurement without posting:

```bash
GH_TOKEN=$(gh auth token) bb --config <plane> run review \
  --payload '{"repo":"owner/repo","pr":123,"measurement":true}' \
  --json
```

Evidence is both the ledger row and the external effect: PR comment for normal
mode, artifact/result output for measurement mode.

## CI Diagnose Workload

For a failed GitHub Actions check suite, run the report-only diagnoser manually:

```bash
GH_TOKEN=$(gh auth token) bb --config <plane> run ci-diagnose \
  --idempotency-key "ci-diagnose:<repo>:<sha>" \
  --payload '{"repo":"owner/repo","head_sha":"<sha>","workflow":"verify"}' \
  --json
```

Webhook mode is `POST /hooks/ci-diagnose` for GitHub `check_suite` deliveries.
The task filters to failed completed GitHub Actions suites for the configured
repo allowlist. The lane writes `REPORT.json` with diagnosis evidence and may
recommend an exact builder command, but it does not edit code, comment, merge,
deploy, park tasks, resolve runs, or invoke follow-up runs.

## Model Evaluation Loop

When the best model/config for a flow is uncertain, run the flow's first-class
cohort from [`docs/model-evals/`](../../../docs/model-evals/README.md): at
least three candidate tasks against the same payload, then one evaluator run.
For example, the CI-diagnose cohort is:

```bash
payload='{"repo":"owner/repo","head_sha":"<sha>","workflow":"verify","dry_run":true}'

GH_TOKEN=$(gh auth token) bb --config <plane> run ci-diagnose \
  --idempotency-key "model-eval:<sha>:deepseek" \
  --payload "$payload" --json
GH_TOKEN=$(gh auth token) bb --config <plane> run ci-diagnose-kimi \
  --idempotency-key "model-eval:<sha>:kimi" \
  --payload "$payload" --json
GH_TOKEN=$(gh auth token) bb --config <plane> run ci-diagnose-glm \
  --idempotency-key "model-eval:<sha>:glm" \
  --payload "$payload" --json
```

Extract each run's `REPORT.json` and pass a compact candidate packet to the
evaluator:

```bash
bb --config <plane> run model-eval \
  --idempotency-key "model-eval:<flow>:<sha>:judge" \
  --payload '{"flow":"ci-diagnose","objective":"compare reports","candidates":[...],"reference_context_path":"docs/model-evals/ci-diagnose/YYYY-MM-DD.md"}' \
  --json
```

The evaluator is report-only. Save accepted conclusions under
[`docs/model-evals/`](../../../docs/model-evals/README.md) with run ids, model
ids, cost, latency, and residual risk.
Do not promote a model default from malformed candidate output or same-context
self-review.

Candidate variants are manual-only. Review variants force measurement mode;
gardener variants force dry-run; build variants default to dry-run unless the
payload explicitly asks for a live branch; storm variants use eval-only verdict
kinds and do not change gate arithmetic.

## Submission Gate

Open a submission:

```bash
bb --config <plane> submit open \
  --change "<change-id>" \
  --rev "<git-rev>" \
  --json
```

For the common submit-and-storm path, prefer the checked-in recipe over a
hand-built shell wrapper. It validates a JSON payload file before any ledger
mutation, runs `bb preflight --storm --json` before opening the submission,
relies on `bb submit open`'s CAS refusal for duplicate open submissions,
dispatches storm lanes with `--payload-file` (so the payload is not visible in
argv), and prints a machine-readable receipt plus the safe next gate command:

```bash
cat > /tmp/bb-storm-payload.json <<'JSON'
{"repo":"misty-step/bitterblossom","change":"<change-id>","rev":"<git-rev>","backlog":"backlog.d/086-first-class-operator-dispatch-recipes.md","base_ref":"origin/master"}
JSON

scripts/bb-submit-storm \
  --config <plane> \
  --bb "target/debug/bb" \
  --payload-file /tmp/bb-storm-payload.json \
  --require-field backlog \
  --require-field base_ref \
  --json
```

Cron supervisors that need runtime secrets must pass the BB secret helper as
part of `--bb`, for example
`--bb "plane/.bb/dogfood-loop/with-bb-secrets target/debug/bb"`. The helper
keeps secrets in environment, not argv; the payload file must not contain
secrets.

Run verdict members with the returned submission id:

```bash
bb --config <plane> run correctness \
  --idempotency-key "storm:<submission>:correctness" \
  --payload '{"submission":"<submission>"}' \
  --json
```

Evaluate:

```bash
bb --config <plane> gate --submission <submission> --json
```

If the gate blocks, fix the underlying issue and open the next round. Do not
delete or rewrite prior verdict rows. If a canonical member failed before
producing a verdict, `gate --json` escalates and the failed member carries
`safe_next_command`; fix the operator or infrastructure issue, then run that
clean replacement submission command instead of trying to make a replay count.
The command includes the plane `--config` path used for the gate evaluation.

## Dead Letters and Recovery

List pre-execute failures:

```bash
bb --config <plane> dlq list --json
```

Replay only when the pre-execute failure is understood. Use `--json` when an
agent or script needs the replayed run, attempt, and event bundle:

```bash
bb --config <plane> dlq replay <id> --json
```

When a pre-execute dead letter is superseded (a replacement submission or run
already passed), close it with an explicit reason instead of replaying it.
`dlq list --json` reports each row's `status` (`open`, `replayed`, or
`acknowledged`); acknowledged rows keep reason + timestamp and cannot be
replayed, and `status --json` no longer counts them as open operator work:

```bash
bb --config <plane> dlq ack <id> --reason <text> --json
```

Before dispatching a storm, preflight missing declared secrets and unspawnable
`command`-harness binaries for one task or the gate-required storm member set:

```bash
bb --config <plane> preflight <task> --json
bb --config <plane> preflight --storm --json
```

After a host restart:

```bash
bb --config <plane> recover
bb --config <plane> recover --json
bb --config <plane> runs list --json
```

Resolve `awaiting_recovery` only after inspecting side effects:

```bash
bb --config <plane> runs resolve <run-id> success --reason "<why>"
```

`recover --json` reports `attempt_phase`, legacy human `probe`,
`probe_state`, `probe_reason`, `lease_disposition`, `operator_action`, and
`disposition` for each inherited run. `probe_state: "unknown"` means the
substrate could not prove the agent process is dead; inspect
`bb runs show <run-id> --json` for the `boot_probe` event and leave the host
lease in place until side effects are understood. Missing or malformed
pidfiles and probe command failures are unknown, not dead.

`bb status --json` is the ongoing recovery queue. Fresh recovery rows suggest
`resolve_after_side_effect_inspection`; after one hour the action becomes
`escalate_stale_recovery` and includes `age_seconds` plus
`stale_after_seconds`. Escalation is visibility, not automatic replay.

## Serve and Read APIs

Loopback development:

```bash
bb --config <plane> serve
curl http://127.0.0.1:7077/health
curl http://127.0.0.1:7077/api/status
curl http://127.0.0.1:7077/api/tasks
```

Non-loopback serving needs `BB_API_TOKEN`. Query with:

```bash
{
  printf '%s\n' 'fail'
  printf '%s\n' 'silent'
  printf '%s\n' 'show-error'
  printf 'url = "%s/api/runs"\n' "$BB_URL"
  printf 'header = "Authorization: Bearer %s"\n' "$BB_API_TOKEN"
} | curl --config -
```

Do not put `BB_API_TOKEN` in a query string. Read APIs and the HTML view accept
only the bearer header when a token is configured. Prefer curl config on stdin
so the bearer value is not exposed in process argv.

## Parked Tasks

```bash
bb --config <plane> task list --json
bb --config <plane> task park <task> --reason "<operator reason>"
bb --config <plane> task unpark <task>
```

Never unpark just to make a command succeed. Read the parked reason, inspect
recent runs, and name why the underlying condition is gone.

## Autonomy Promotion

Autonomous task families run at a fixed authority level (read-only,
report-only, dry-run, PR-only) and climb only on cited evidence. Before
recommending or taking any higher-authority action, read
[`docs/rollout-scorecards.md`](../../../docs/rollout-scorecards.md): confirm the
task family's scorecard is green for the target level and that an operator has
approved the promotion. Green metrics make the next-level ticket eligible for
operator approval; they never flip authority automatically. A promotion ticket
that does not cite the lower-authority evidence packet is not ready.
