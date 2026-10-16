# Run-isolated static previews: bounded lifetime

As a visitor, I can continue using the accepted public static reference at its
canonical hostname while preview cleanup runs. Cleanup cannot modify its bytes,
Worker, domain, access policy or deployment history.

As a reviewer, I receive a distinct URL for each native CI run and attempt. The
URL is never reused for different bytes. Ordinary previews expire after two newer
completed previews exist; the accepted migration review URL is retained explicitly.

As a maintainer, I can run CI repeatedly without accumulating preview Workers.
Before allocating another preview, CI retains the accepted review identity, the
two newest completed previews and its current attempt, and removes only exact
repository/workflow-owned superseded or failed previews. An ownership, API or
deletion-readback failure stops allocation. Recovery is a normal rerun after the
failure is resolved, not a bypass. Canonical and retained review identities are
read back unchanged, and deleted URLs must return HTTP 404.

Scope: this slice specifies public static-reference hosting only. Application
behavior outside that reference is not changed or inferred here.
