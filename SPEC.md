# Immutable static previews without accumulating Workers

As a visitor, I can continue using the accepted public static reference at its
canonical hostname while preview cleanup runs. Cleanup cannot modify its bytes,
Worker, domain, access policy or deployment history.

As a reviewer, I receive the actual immutable native version URL and version ID
for my CI run and attempt. Another run, a synthetic rollback drill or a rerun
cannot change its bytes. The shared mutable preview hostname is not a review URL.
The accepted migration review URL remains available at its original identity.

As a maintainer, I can run CI repeatedly without accumulating preview Workers.
CI reuses one preview Worker and creates native versions, not a Worker per run.
Alongside the retained migration review Worker, this lane needs only two preview
Workers per repository, independently of the number of runs. The isolated A/B/A
native rollback drill and every published-byte/missing-path check remain real.
Superseded run Workers are removed only after exact Actions-receipt attribution,
provider absence and HTTP404 readbacks. Canonical and accepted review deployment
identities remain unchanged during cleanup. No fleet cleanup service is needed.

Scope: this slice specifies public static-reference hosting only. Application
behavior outside that reference is not changed or inferred here.
