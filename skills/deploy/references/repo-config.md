# Repo Deploy Config

How a repo declares its deploy target to `/deploy`. Two mechanisms,
one authoritative, the other heuristic.

## Authoritative: `.harness-kit/deploy.yaml`

If present, this file is the single source of truth. `/deploy` will
not probe further.

Validate with `scripts/load-harness-kit-config.py deploy --repo <repo>`.
The executable schema is mirrored in
`meta/config-schemas/deploy.schema.yaml`; malformed config fails before
target detection or deploy commands run. `--optional` returns `{}` when the
file is absent so `/deploy` can fall back to heuristics deliberately.

### Schema

```yaml
# Required
target: fly | vercel | cloudflare | aws | s3 | docker | k8s | custom

# Required unless target is inferable from platform config (e.g. fly.toml
# already names the app). Duplicate here if you want explicit control.
app: my-app-prod

# Required if the repo deploys to more than one env. When multiple envs
# are declared, /deploy --env <name> is mandatory; no default.
envs:
  prod:
    app: my-app-prod                  # overrides top-level `app`
    healthcheck: https://my-app.com/health
    rollback_grace_seconds: 300
    require_ci_green: true            # default true
  staging:
    app: my-app-staging
    healthcheck: https://staging.my-app.com/health
    rollback_grace_seconds: 120
    require_ci_green: false

# Optional (single-env shorthand; used when `envs` is absent)
healthcheck: https://my-app.com/health
rollback_grace_seconds: 300

# Optional: skip deploy if sha on target matches. Default true.
idempotent: true

# Custom target only
deploy_cmd: ./scripts/deploy.sh
current_sha_cmd: ./scripts/current-sha.sh
rollback_handle_cmd: ./scripts/current-release.sh
rollback_cmd: "./scripts/rollback.sh {{handle}}"
```

### What NEVER goes in this file

- API tokens, deploy keys, platform credentials
- Secret URLs (tokens embedded as query params)
- Per-user state (local paths, personal env vars)

Secrets live in the platform CLI's auth store (`flyctl auth login`,
`vercel login`, `~/.aws/credentials`, etc.). `/deploy` never reads
them, never writes them.

### First-run bootstrap

If `.harness-kit/deploy.yaml` is absent and `/deploy` runs interactively
(TTY attached):

1. Run detection heuristics (below). If exactly one target is
   plausible, show what was inferred and ask: "Create
   `.harness-kit/deploy.yaml` with target=<X>, app=<Y>? [Y/n]"
2. Ask for `healthcheck` URL (required, no default)
3. Ask for `rollback_grace_seconds` (default 300)
4. Write the file, commit it in a separate step (skill does not auto-commit)

If non-interactive and the file is missing: abort with a clear error
showing the exact YAML to paste.

---

## Heuristic: detection from existing files

Ordered probe. First hit wins. Stop at the first match.

| Priority | Marker                                       | Inferred target |
|----------|----------------------------------------------|-----------------|
| 1        | `.harness-kit/deploy.yaml`                     | (authoritative) |
| 2        | `fly.toml`                                   | `fly`           |
| 3        | `vercel.json` or `.vercel/project.json`      | `vercel`        |
| 4        | `wrangler.toml` or `wrangler.jsonc`          | `cloudflare`    |
| 5        | `serverless.yml`                             | `aws`           |
| 6        | `samconfig.toml` or `template.yaml` + SAM    | `aws`           |
| 7        | `Chart.yaml` or `kustomization.yaml` or `k8s/` dir | `k8s`     |
| 8        | `Dockerfile` (alone, no above)               | ambiguous — prompt |

Ambiguity rules:
- Multiple markers (e.g. both `fly.toml` and `vercel.json`) → require
  `.harness-kit/deploy.yaml` to disambiguate. Do not guess.
- `Dockerfile` alone → could be fly, k8s, self-hosted, ECS, Railway,
  Render, etc. Always prompt or require config.

### Inferring `app` / scope

When heuristic detection succeeds:
- `fly.toml` → parse `app = "..."` field
- `vercel.json` → parse `name` or use `.vercel/project.json` → `projectId`
- `wrangler.toml` → parse `name = "..."` field
- `serverless.yml` → parse `service:` field
- `k8s` → require `.harness-kit/deploy.yaml` (no universal convention)

### What heuristic detection CANNOT infer

Always require `.harness-kit/deploy.yaml` for:
- `healthcheck` URL (platform defaults are insufficient — they often
  return 200 on a root path that does not exercise the deployed code)
- Multi-env routing (platforms handle envs differently; explicit
  config is the only safe path)
- `rollback_grace_seconds` (platform defaults vary wildly)
- Any `custom` target

---

## Evolving config

When `/deploy` runs and notices drift (e.g. config says `target: fly`
but `fly.toml` is gone), abort with an explanation rather than
falling back to heuristics. Config is authoritative; drift is a bug
in the repo, not in the skill.

When a new env is added, the operator edits this file. `/deploy` never
writes new envs on its own — only the initial bootstrap for a
previously-unconfigured repo.
