# Misty Step Factory Fleet Integration Standard

This standard applies to active `misty-step` organization projects. Harness Kit
owns the routing doctrine; the product repos own their actual Canary, Powder,
and Landmark integration surfaces.

## Required Evidence

Every active project must have a Powder representation:

- Repos with `backlog.d` import active backlog files into Powder under the
  full repo label, for example `misty-step/canary`.
- Repos without a local backlog root get a Powder seed card titled
  `Adopt Canary, Powder, and Landmark factory stack`.
- Open GitHub issues are imported or mirrored as Powder cards so agents do not
  have to discover durable work from GitHub alone.

Runtime projects must have Canary evidence:

- A stable Canary service name. If the runtime service differs from the repo
  name, record the service mapping in `.canary/integration.json`.
- HTTP runtimes expose or identify one production health URL and have live
  Canary target readback.
- Worker, scheduler, CLI, or event-plane runtimes use Canary check-in
  monitors.
- Application error paths report to Canary or carry an explicit gap card in
  Powder. Sentry-only coverage is not enough for Factory operation.

Release-managed projects must have Landmark evidence:

- Repos with existing release tooling or an existing release surface keep
  Landmark in manifest-only or synthesis-only mode unless the Landmark fleet
  plan recommends full release ownership.
- Repos without release automation start with a `.landmark.yml` manifest and
  backfill-first plan. Do not add release mutation before operator-approved
  initial version/tag policy exists.
- If `fleet plan` reports missing secret metadata, do not add a broken
  workflow. Track the secret gap in Powder and use manifest-only adoption until
  the secret policy is fixed.

## Service Names

Use the Canary service name from live readback, not the repo name guessed from
the path:

| Repo | Canary service |
|---|---|
| `misty-step/bitterblossom` | `bitterblossom-plane` |
| `misty-step/brainrot` | `brainrot-publishing-house` |
| `misty-step/chrondle` | `chrondle` |
| `misty-step/linejam` | `linejam` |
| `misty-step/memory-engine` | `memory-engine-api` |
| `misty-step/misty-step` | `misty-step` |
| `misty-step/powder` | `powder` |
| `misty-step/sploot` | `sploot-web` |
| `misty-step/vibe-machine` | `vibe-machine` |

## Verification Commands

Use these commands from local checkouts with credentials supplied by the
environment or the Agents vault:

```sh
/Users/phaedrus/Development/canary/bin/canary integrate status /path/to/repo \
  --service <canary-service> \
  --production-url <health-url> \
  --json

/Users/phaedrus/Development/canary/bin/canary errors <canary-service> \
  --window 1h --json

/Users/phaedrus/Development/landmark/target/debug/landmark setup \
  --repo-root /path/to/repo \
  --dry-run --error-format json

curl -fsS -H "Authorization: Bearer $POWDER_API_KEY" \
  "$POWDER_API_BASE_URL/api/v1/cards?repo=misty-step%2F<repo>&limit=100"
```

As of the 2026-07-04 cutover, the canonical Powder instance is Sanctum Powder
on the `phrazzld-bastion` Fly app: use
`https://bastion.tail5f5eb4.ts.net:10001` as `$POWDER_API_BASE_URL` (reachable
from any device on the tailnet). The old standalone `powder` Fly app and its
`127.0.0.1:14030` local proxy are retired.

## Waivers

A waiver is allowed only when the project is not an active runtime or release
surface. The waiver must live as a Powder card or repo file and say:

- why Canary uptime/health/error logging does not apply;
- where backlog state lives in Powder;
- how Landmark release intelligence is intentionally deferred or not
  applicable.

Do not call a project integrated because it has one of the three surfaces.
Factory integration means all applicable surfaces are present and queryable.
