# Public static hosting adapter

Foundations: https://foundations.mistystep.io/

## Behavior

Preserve every public object in `files.sha256` from the exact existing Pages
source in `site.json`, including project-prefixed URLs. Bytes are not rewritten.
The domain root redirects to that prefix, except Mint, whose root stays `/`.
Explicit `.html` URLs can redirect to the same canonical content under Workers
assets' automatic HTML handling. Unknown paths return404, never an SPA fallback.
This is a public frontend artifact, not revival of its dormant application backend.

## Build and verify

The native `Cloudflare static site` workflow checks out the pinned source into
its original repository-root layout, builds with existing locked dependencies,
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
and promotion. Retain the verified artifact for90days; the canonical source and
hash allowlist persist here. Use native Wrangler rollback for the recorded prior
version and reverify all bytes. Record original Pages source/build_type/cname and
archive flag before cutover; restoring Pages uses that exact configuration, not
a new source branch. Mint's original DNS remains until Cloudflare proof and its
own cutover readback; never substitute a workers.dev URL as final acceptance.

No Pages deletion, archive toggle, DNS write, provider credential or Access
mutation is automated by this adapter. Archived status is restored after the
migration, without deleting the repository. Public sites remain public. Assets
only: no billed Worker invocation or mutable data store is introduced.
