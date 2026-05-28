# Deploy Targets

Catalog of deploy targets. Each entry owns its CLI invocation, rollback
handle capture, healthcheck shape, and log tail budget. `/deploy` is a
router; this file is where the actual commands live.

Adding a new target: append a section with the same eight fields. If
the target lacks any of them (especially a rollback handle), flag it —
`/deploy` refuses targets without reversible deploys.

---

## Fly.io

- **Detect:** `fly.toml` at repo root
- **Auth check:** `flyctl auth whoami`
- **Current sha:** `flyctl status --json | jq -r '.Deployment.ImageRef'`
  (parse sha from image tag) OR store sha as a Fly metadata label on
  each deploy
- **Rollback handle:** `flyctl releases --json | jq -r '.[0].Version'`
  (the integer release number, e.g. `v42`)
- **Deploy:** `flyctl deploy --app <app> --image-label sha-<shortsha>
  --strategy rolling`
- **Healthcheck:** `flyctl status --json | jq -r '.Allocations[].Checks'`
  — all checks `passing`; plus configured `healthcheck_url`
- **Rollback:** `flyctl releases rollback <handle> --app <app>`
- **Log budget:** tail last 50 lines on failure; nothing on success

---

## Vercel

- **Detect:** `vercel.json` OR `.vercel/project.json`
- **Auth check:** `vercel whoami`
- **Current sha:** `vercel inspect <url> --json | jq -r '.meta.githubCommitSha'`
- **Rollback handle:** the previous deployment URL —
  `vercel ls <project> --json | jq -r '.[1].url'`
- **Deploy:** `vercel deploy --prod --yes --meta
  githubCommitSha=<sha>` (for `prod`); drop `--prod` for previews
- **Healthcheck:** poll deployment URL + configured `healthcheck_url`;
  Vercel reports state via `vercel inspect <url> --json | jq -r '.readyState'`
  — wait for `READY`
- **Rollback:** `vercel promote <previous-url>` (promotes the prior
  deployment to production alias)
- **Log budget:** `vercel logs <url>` tail 50 on failure; none on success

---

## Cloudflare (Workers / Pages)

- **Detect:** `wrangler.toml` or `wrangler.jsonc`
- **Auth check:** `wrangler whoami`
- **Current sha:** Workers — `wrangler deployments list --json | jq
  -r '.[0].metadata.commitHash'` (requires `--commit-hash` on deploy)
- **Rollback handle:** deployment ID from
  `wrangler deployments list --json | jq -r '.[0].id'`
- **Deploy:** Workers — `wrangler deploy --commit-hash <sha>`;
  Pages — `wrangler pages deploy <dist> --commit-hash <sha> --branch main`
- **Healthcheck:** configured `healthcheck_url` (Cloudflare does not
  expose a native "ready" state for Workers beyond 2xx on the route)
- **Rollback:** `wrangler rollback <deployment-id>`
- **Log budget:** `wrangler tail` for 30s on failure; none on success

---

## AWS (Lambda via SAM/Serverless)

- **Detect:** `serverless.yml` OR `samconfig.toml` / `template.yaml`
- **Auth check:** `aws sts get-caller-identity`
- **Current sha:** stack tag —
  `aws cloudformation describe-stacks --stack-name <name> --query
  'Stacks[0].Tags[?Key==\`git-sha\`].Value' --output text`
- **Rollback handle:** previous CloudFormation change-set ID OR prior
  Lambda version alias (`aws lambda list-versions-by-function`)
- **Deploy:** SAM — `sam deploy --stack-name <name> --tags git-sha=<sha>
  --no-confirm-changeset`; Serverless — `serverless deploy --stage <env>`
- **Healthcheck:** configured `healthcheck_url` (typically API Gateway
  endpoint); CloudFormation status must be
  `UPDATE_COMPLETE`/`CREATE_COMPLETE`
- **Rollback:** SAM — `aws cloudformation continue-update-rollback`
  or redeploy prior sha; Lambda — `aws lambda update-alias --name prod
  --function-version <prev>`
- **Log budget:** CloudFormation events last 20 + CloudWatch last 30s on failure

---

## Static S3 + CloudFront

- **Detect:** `.harness-kit/deploy.yaml` with `target: s3` (no universal
  marker file) OR `s3-deploy.json` / similar project convention
- **Auth check:** `aws sts get-caller-identity`
- **Current sha:** object metadata on `index.html` —
  `aws s3api head-object --bucket <b> --key index.html
  --query 'Metadata."git-sha"'`
- **Rollback handle:** S3 object version IDs from bucket versioning
  (REQUIRE bucket versioning; refuse deploy if disabled). Store the
  set of version IDs for all synced keys as the handle, or use a
  pre-deploy `aws s3 sync` to an archive prefix
- **Deploy:** `aws s3 sync <build-dir> s3://<bucket>/ --delete
  --metadata git-sha=<sha>` then `aws cloudfront create-invalidation
  --distribution-id <id> --paths '/*'`
- **Healthcheck:** fetch `healthcheck_url` (configured; typically
  `<site>/version.json` or `<site>/health`) and verify it reports the
  deployed sha
- **Rollback:** restore prior versions via S3 versioning API OR sync
  from archive prefix; always re-invalidate CloudFront
- **Log budget:** sync output last 20 lines; invalidation ID

---

## Self-hosted Docker (SSH + registry)

- **Detect:** `Dockerfile` + `.harness-kit/deploy.yaml` with
  `target: docker` and `host`, `image`, `registry` fields
- **Auth check:** `docker login <registry>` succeeds; `ssh <host> true`
- **Current sha:** `ssh <host> 'docker inspect <container> --format
  "{{index .Config.Labels \"git-sha\"}}"'`
- **Rollback handle:** previous image tag, e.g. `<registry>/<image>:sha-<prev>`
  — capture from current container's image reference before pushing new
- **Deploy:**
  1. `docker build --label git-sha=<sha> -t <registry>/<image>:sha-<short> .`
  2. `docker push <registry>/<image>:sha-<short>`
  3. `ssh <host> 'docker pull <registry>/<image>:sha-<short> &&
     docker stop <container> && docker rm <container> &&
     docker run -d --name <container> --label git-sha=<sha> ...
     <registry>/<image>:sha-<short>'`
- **Healthcheck:** configured `healthcheck_url`; optionally
  `docker inspect <container> --format '{{.State.Health.Status}}'`
- **Rollback:** re-run step 3 with prior image tag
- **Log budget:** `docker logs <container> --tail 50` on failure

---

## Kubernetes

- **Detect:** `k8s/` dir, `kustomization.yaml`, `Chart.yaml` (helm),
  OR `.harness-kit/deploy.yaml` with `target: k8s`
- **Auth check:** `kubectl auth can-i update deployments -n <ns>`
- **Current sha:** deployment annotation —
  `kubectl get deployment <name> -n <ns> -o
  jsonpath='{.metadata.annotations.git-sha}'`
- **Rollback handle:** revision number from
  `kubectl rollout history deployment/<name> -n <ns>` (use the current
  revision before deploy; rollback returns to it)
- **Deploy:** kustomize — `kustomize build k8s/<env> | kubectl apply -f -`
  with image tag patched to `sha-<short>`; helm —
  `helm upgrade <release> <chart> --set image.tag=sha-<short>
  --set annotations.git-sha=<sha> -n <ns>`
- **Healthcheck:** `kubectl rollout status deployment/<name> -n <ns>
  --timeout=<grace>s` + configured `healthcheck_url`
- **Rollback:** `kubectl rollout undo deployment/<name> -n <ns>
  --to-revision=<handle>`
- **Log budget:** `kubectl logs deployment/<name> -n <ns> --tail=50` on failure

---

## Custom

Last-resort escape hatch. Repo declares:

```yaml
# .harness-kit/deploy.yaml
target: custom
deploy_cmd: "./scripts/deploy.sh"
current_sha_cmd: "./scripts/current-sha.sh"
rollback_handle_cmd: "./scripts/current-release.sh"
rollback_cmd: "./scripts/rollback.sh {{handle}}"
healthcheck_url: "https://..."
```

Contract: each command exits 0 on success, prints the relevant value
to stdout. `/deploy` runs them and treats the output as opaque
strings. The repo owns correctness.

Refuse this target if `rollback_cmd` or `rollback_handle_cmd` is
missing — non-negotiable.
