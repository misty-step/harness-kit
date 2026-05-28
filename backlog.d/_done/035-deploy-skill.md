# `/deploy` — ship to environment

Priority: medium
Status: pending
Estimate: M (~2 dev-days)

## Goal

One skill ships merged code to a deployment environment. Invoked by
`/autopilot` (outer loop) after `/deliver` produces merge-ready code and
the merge happens. Also usable standalone for manual deploys.

## Contract

**Input:** A merged commit / branch to deploy. Deployment target (default
from repo config).

**Output:**
- Deployed artifact or service in the target environment
- Deploy ID / URL / version tag that downstream `/monitor` can anchor on
- Rollback handle (previous version) recorded in cycle manifest

**Stops at:** deploy reported healthy by target platform. Does not
monitor post-deploy (→ `/monitor`), does not triage anomalies (→
`/investigate`).

## Stance

1. **Per-repo wildly different.** Fly.io vs Vercel vs AWS vs self-hosted
   Docker vs static S3 — the skill is necessarily thin at the global
   level and most content lives in repo-local config.
2. **Global skill = router.** The global `/deploy` reads repo config
   (e.g. `.harness-kit/deploy.yaml` or detection heuristics), dispatches to
   the right deploy mechanism, and returns a canonical deploy receipt.
3. **Receipt is the contract.** Regardless of target, output a structured
   receipt: `{version, env, url, healthcheck_url, rollback_cmd, timestamp}`.
   `/monitor` keys on this.
4. **No in-skill secrets handling.** Skill delegates to repo-configured
   deploy commands; secrets stay in the deploy target's auth.

## Composition

```
/deploy [--env <name>] [--version <ref>]
    │
    ▼
  1. Detect target (config file, repo type heuristics)
    │
    ▼
  2. Validate: merged ref exists, CI green, target reachable
    │
    ▼
  3. Dispatch to platform-specific runner
     ├── fly: `flyctl deploy`
     ├── vercel: `vercel --prod`
     ├── docker: `docker push && ssh ... docker pull && restart`
     ├── static: `aws s3 sync` / `rsync`
     └── custom: shell out to configured command
    │
    ▼
  4. Capture deploy receipt
    │
    ▼
  5. Emit receipt JSON to stdout and append to cycle manifest
```

## Repo-Local Config

```yaml
# .harness-kit/deploy.yaml
target: fly
app: harness-kit-prod
healthcheck: https://harness-kit.fly.dev/health
rollback_grace_seconds: 300
```

When this file is absent, `/deploy` prompts for missing info on first use
and writes it. Interactive → self-persisting config.

## What `/deploy` Does NOT Do

- Build artifacts (→ CI in `/ci`, or implicit in platform deploy)
- Monitor post-deploy (→ `/monitor`)
- Triage failures (→ `/investigate`)
- Rollback automatically (→ `/monitor` decides, `/deploy --rollback` executes)
- Approve deploys — human or outer-loop caller decides; `/deploy` executes

## Oracle

- [ ] `skills/deploy/SKILL.md` exists
- [ ] Runs on at least 2 target types (e.g. static + containerized)
- [ ] Emits structured receipt JSON
- [ ] `.harness-kit/deploy.yaml` detection works; absent-file prompts
- [ ] Harness Kit itself has no deploy target — skill correctly no-ops or errors clearly
- [ ] Integrates with `/autopilot` outer loop: receipt lands in cycle manifest

## Non-Goals

- Replacing platform deploy CLIs — wraps them
- Multi-env promotion pipelines — one env per invocation
- Blue/green or canary logic — platform's job
- Deciding *when* to deploy — caller's job

## Related

- Blocks: 028 (`/autopilot` outer loop composition)
- Sibling: 036 (`/monitor` — consumes deploy receipt)
- Related: 025 (dagger merge gate — must pass before `/deploy` runs)
