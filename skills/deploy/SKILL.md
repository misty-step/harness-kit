---
name: deploy
description: |
  Ship merged code to one deploy target. Thin router: detect target, run the
  platform recipe, capture receipt (sha, version, URL, rollback handle), stop
  when healthy. Does not monitor, triage, or decide whether to deploy.
  Use when: "deploy", "deploy to prod", "release", "push to staging",
  "deploy this branch", "release cut".
  Trigger: /deploy, /release.
argument-hint: "[--env <name>] [--version <ref>] [--rollback] [--dry-run]"
---

# /deploy

Ship merged code to an environment. One invocation, one target, one
receipt. The global skill is a router; the real work lives in the
platform-specific recipe (`references/targets.md`) and repo-local config
(`references/repo-config.md`).

## Execution Stance

You are the executive orchestrator for a narrow, high-stakes action.
- Keep the abort/ship decision on the lead model. Do not delegate go/no-go.
- Delegate detection, artifact validation, and log tailing to subagents.
- Run validation steps in parallel; the deploy call itself is serial.

## Delegation Floor

Delegation floor applies for substantive deploy preparation, release-risk
review, config changes, rollback planning, or unhealthy deploy diagnosis:
probe the roster first; dispatch two or more providers; direct solo only for
mechanical deploy commands, emergency preservation, user-forbidden delegation,
or fewer-than-two-providers cases. See `harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use specialized lanes for target detection, artifact/CI
validation, rollback-handle verification, release-risk critique, and log
interpretation. The lead owns the final deploy/abort decision and the actual
deploy command remains serial.

## Contract

**Input:** merged ref to deploy (default: current `HEAD` on primary
branch). Optional `--env` (default from repo config). Optional
`--version` (default: `HEAD` sha).

**Output:** a deploy receipt (schema below) emitted to stdout as JSON
and appended to the cycle manifest if one exists
(`.harness-kit/cycle-manifest.json`, see `/flywheel`).

**Stops at:** target reports healthy (platform-native healthcheck OR
configured `healthcheck` URL returns 2xx within `rollback_grace_seconds`).

**Does NOT:** monitor post-deploy, triage failures, rollback
automatically, build artifacts, manage secrets, promote across envs.

## Protocol

### 1. Detect target

Check in order and stop at first hit:
1. `.harness-kit/deploy.yaml` → authoritative repo-local config
2. `fly.toml` → `target: fly`
3. `vercel.json` or `.vercel/project.json` → `target: vercel`
4. `wrangler.toml` → `target: cloudflare`
5. `Dockerfile` + `.harness-kit/deploy.yaml` missing → prompt for target
6. `serverless.yml` or `sam.yaml` → `target: aws`
7. None of the above → abort with actionable error pointing to
   `references/repo-config.md`

If detection finds a config but `--env` was not supplied and the config
declares multiple envs, abort and require `--env`. Fail closed.

See `references/repo-config.md` for the full detection table and config
schema. When `.harness-kit/deploy.yaml` is present, validate it with
`scripts/load-harness-kit-config.py deploy --repo <repo> --optional` from the
Harness Kit source repo or an equivalent installed loader before using values.

### 2. Validate (parallel)

Dispatch these checks in parallel. All must pass before deploy fires:
- **Ref exists:** `git rev-parse --verify <version>` resolves
- **Ref is merged:** commit is an ancestor of the primary branch
  (unless `--force` is set — only makes sense for hotfix rollforward)
- **CI green:** if `gh` is available and a PR/commit check exists,
  require the Dagger merge gate (`/ci`) to be passing for this sha
- **Target reachable:** `flyctl auth whoami`, `vercel whoami`, etc.
  (per-target liveness check — see `references/targets.md`)
- **No secrets in diff:** quick grep of `git show <sha>` for obvious
  token/credential patterns; abort if found
- **Current state:** query target for its currently-deployed sha

### 3. Idempotence check

If target currently-deployed sha == `<version>` sha: skip deploy. Emit
a receipt with `action: "no-op"` and the existing rollback handle. This
is not a failure — it is the success path when the outer loop re-invokes
`/deploy` on a sha that already shipped.

### 4. Capture rollback handle BEFORE deploy

Query the target for its current deployment ID / release tag / previous
image. Store it in the receipt as `rollback_handle` (opaque string the
platform CLI can consume for a rollback). If the platform cannot surface
a rollback handle: abort. You must be able to reverse this deploy before
you make it.

### 5. Dispatch

Hand off to the target-specific recipe in `references/targets.md`. The
recipe owns the actual CLI invocation and log streaming. The recipe
returns: `{deploy_id, url, version, healthcheck_url}`.

### 6. Wait for healthy

Poll `healthcheck_url` (from config or platform-native) with exponential
backoff up to `rollback_grace_seconds` (default 300). Healthy = 2xx
response AND platform reports deploy status as `ready`/`running`/`live`.

If not healthy within the grace window: emit receipt with
`status: "unhealthy"` and `rollback_handle` prominent. Do **not**
auto-rollback — emit a clear call to the operator naming the rollback
command. `/monitor` may trigger rollback as a separate action.

### 7. Emit receipt

Write JSON to stdout. Append to `.harness-kit/cycle-manifest.json` if it
exists (as `deploy_receipts[]`). Also write to
`.evidence/deploys/<date>/<sha-short>.json` for browsability.

## Receipt Schema

```json
{
  "version": "abc1234",
  "sha": "abc1234567890...",
  "env": "prod",
  "target": "fly",
  "app": "myapp-prod",
  "url": "https://myapp.fly.dev",
  "healthcheck_url": "https://myapp.fly.dev/health",
  "deploy_id": "dep_01HX...",
  "rollback_handle": "v42",
  "status": "healthy",
  "action": "deployed",
  "timestamp": "2026-04-15T14:32:10Z",
  "duration_seconds": 94,
  "operator": "misty-step"
}
```

Field rules:
- `status` ∈ {`healthy`, `unhealthy`, `timeout`}
- `action` ∈ {`deployed`, `no-op`, `rolled-back`, `aborted`}
- `rollback_handle` MUST be present and non-empty unless
  `action == "aborted"` and the abort happened before step 4
- `sha` is the full 40-char sha; `version` is the short form or the
  platform-native version tag if the target mints one

## Rollback Mode

`/deploy --rollback [--to <handle>]` — reverse the most recent deploy.

- Default `<handle>`: the `rollback_handle` from the most recent receipt
  in `.evidence/deploys/`
- Emit a new receipt with `action: "rolled-back"` and the new current
  state captured
- Do NOT chain rollbacks. If the operator wants to reverse further,
  require an explicit `--to` with a concrete handle

## Harness Kit Self-Deploy

Harness Kit itself has no deploy target (it is a symlinked-into-home
config repo). If invoked from the Harness Kit repo: emit a clear no-op
receipt explaining bootstrap.sh is the "deploy" mechanism and exit 0.
Detection: `git rev-parse --show-toplevel` resolves to a path
containing `bootstrap.sh` AND `skills/` AND no `.harness-kit/deploy.yaml`.

## Gotchas

- **Deploying unmerged code:** the caller (`/flywheel`) promises merged
  input, but validate it anyway. Ancestor check is cheap.
- **Missing rollback handle:** if the platform does not expose one,
  refuse the deploy rather than shipping irreversibly.
- **Healthcheck that always returns 200:** a healthcheck that does not
  actually exercise the deployed code is worse than none. Document in
  `references/repo-config.md`; warn if the configured healthcheck is
  the root path.
- **Re-deploying same sha:** idempotence check in step 3 prevents
  wasted deploys and misleading receipts. Do not skip it.
- **Silent CI bypass:** if `gh` is unavailable, do not silently skip the
  CI-green check — warn loudly and require `--force-no-ci` to proceed.
- **Secrets in repo-local config:** `.harness-kit/deploy.yaml` holds
  target names, URLs, grace windows — NEVER tokens. If the repo needs
  secrets to deploy, they live in the platform CLI's auth, not here.
- **Multi-env ambiguity:** if config lists `prod` and `staging` and the
  caller did not pass `--env`, fail closed. Never guess.
- **Log firehose:** platform deploy logs can be thousands of lines. The
  recipe in `references/targets.md` specifies a log tail budget; do not
  dump full logs into the receipt.
- **Outer-loop re-entry:** `/flywheel` may call `/deploy` on every cycle.
  The no-op path must be fast (< 5s) and side-effect-free.
- **Interactive prompts in CI:** when repo config is missing and the
  invocation is non-interactive (no TTY), abort with instructions
  rather than hanging on prompt.

## Related

- `/flywheel` — outer-loop caller; passes merged sha + env
- `/monitor` — consumes this receipt, decides on rollback
- `/diagnose` — triages anomalies post-deploy
- `/deliver --polish-only` — merge gate that must pass before `/deploy` runs
- `references/targets.md` — platform-specific recipes
- `references/repo-config.md` — config schema and detection rules

## Verification

Semantic waiver: deploy correctness depends on the target platform and live
receipt. When `.harness-kit/deploy.yaml` exists, validate it with
`scripts/load-harness-kit-config.py deploy --repo <repo>`; final proof is the
deploy receipt sha/version/URL plus platform log tail.
