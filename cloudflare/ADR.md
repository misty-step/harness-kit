# ADR: preserve the existing public site on Cloudflare assets

Status: accepted for implementation; release still requires independent review
and per-site live/retirement evidence. Decision date: 2026-10-15.

The scope is the public static reference, not the repository's executable
application or dormant data. `site.json` owns its immutable serving revision;
`files.sha256` owns the expected emitted bytes. Static sources are staged without
inventing an application build. Existing generated sites use their original
build, dependency locks, and recorded reproducibility inputs. Updating the
published snapshot later requires an intentional reviewed source/hash update.

Workers static assets preserve the project prefix and public access. An unknown
path is a 404, not SPA fallback. Workers' native HTML canonicalization may redirect
an explicit .html URL to the same content while retaining its query. Mutable
application resources and new paid features are not introduced. Site-specific
exceptions and retained defects are recorded in README.md.

One preview Worker uses native immutable version URLs instead of a Worker per
run. CI verifies every versioned object and isolates A from live synthetic B.
Accepted migration review identity (retained unchanged):
https://harness-kit-site-preview-38798173356-1.misty-step.workers.dev
One-time exact Actions cleanup receipts live with task t_efbebddf. No perpetual
pruner, TTL, cron or fleet CD service remains. Native preview URL reference:
https://developers.cloudflare.com/workers/configuration/previews/
CI prepares the artifact once, verifies all hashes, exercises real A/B/A native
rollback, and promotes the same directory
only from the current default branch with the promotion variable enabled. That
production Custom Domain attachment is an intentional DNS-affecting cutover.
It is not enabled by a preview or by merely merging this adapter. Repository
writers retain their existing trusted Actions-secret boundary; forks receive no
secret-bearing deployment. No broader token or manual release-approval lane is
introduced.

Old Pages remains live through preview, review, rollback and canonical proof.
Then disable only this repository's Pages, verify its github.io URL becomes 404,
and retain the repository and original settings for recovery. Restore the
original archive flag after necessary edits. Mint additionally preserves its
custom hostname and requires chief-only reversible CNAME handling after live
Cloudflare proof. Source/default-branch merge alone is not migration acceptance.

Shared rules: https://foundations.mistystep.io/
