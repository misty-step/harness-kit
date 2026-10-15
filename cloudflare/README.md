# Public static hosting adapter

Foundations: https://foundations.mistystep.io/

## Behavior

Preserve every public object in `files.sha256` from the exact existing Pages
source in `site.json`, including project-prefixed URLs. Bytes are not rewritten.
The domain root redirects to that prefix, except Mint, whose root stays `/`.
Explicit `.html` URLs can redirect to the same canonical content under Workers
assets' automatic HTML handling. Unknown paths return 404, never an SPA fallback.
This is a public frontend artifact, not revival of its dormant application backend.

## Build and verify

The native `Cloudflare static site` workflow checks out the pinned source into
its original repository-root layout, uses the original build where needed,
stages only the emitted allowlist, checks every SHA256, uploads the exact artifact,
deploys a public isolated Workers-assets preview, and verifies every HTTP object,
the homepage and a missing path. Mint's CI artifact stays private in this private
repository; the deployed allowlist contains only its already-public site bytes.
No credential or source checkout is included in the artifact.

## Promotion and recovery

The first migration is NOT accepted by this PR or a green preview. Keep GitHub
Pages and DNS unchanged until independent review, link/browser checks and native
preview rollback proof pass. `CLOUDFLARE_SITE_PROMOTED` must be absent/false before
that acceptance. After acceptance, set it true: default-branch CI promotes the
same verified staging directory to `harness-kit-site.mistystep.io`. It never rebuilds between preview
and promotion. Retain the verified artifact for 90 days; the canonical source and
hash allowlist persist here. Use native Wrangler rollback for the recorded prior
version and reverify all bytes. Record original Pages source/build_type/cname and
archive flag before cutover; restoring Pages uses that exact configuration, not
a new source branch. Mint's original DNS remains until Cloudflare proof and its
own cutover readback; never substitute a workers.dev URL as final acceptance.

Preview deployments leave DNS unchanged. The gated production deployment
attaches the configured Cloudflare Custom Domain and can create its DNS record;
that attachment is the reviewed cutover, not a preview side effect. Mint has an
existing CNAME: chief-only scoped removal precedes production Custom Domain
attachment after live CF proof; restore it only on rollback, after detaching
the Custom Domain. No DNS-write privilege
is added to CI. No Pages deletion, archive toggle, credential or Access mutation
is automated. Archived status is restored after the
migration, without deleting the repository. Public sites remain public. Assets
only: no billed Worker invocation or mutable data store is introduced.

Each run/attempt has a distinct preview Worker name. Reruns cannot replace a
previous review URL. Record its URL and native version in the candidate proof;
remove only owned superseded previews after durable release evidence. Repository
writers are trusted with repository Actions secrets, as in the authorized CD
binding. Fork workflows are denied; this is not a claim to sandbox a malicious
repository administrator. Promotion requires the current default branch and
never relies on a manual approval environment or a broader credential.
