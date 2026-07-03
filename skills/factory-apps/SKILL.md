---
name: factory-apps
description: |
  Route Misty Step factory application capabilities. Use when choosing,
  auditing, integrating, or operating Canary, Powder, Landmark, Aesthetic, or
  Bitterblossom: production observability, incidents, health checks, error
  logging, backlog/work-card state, release intelligence, UI/UX system
  adoption, or supervised/unsupervised agent dispatch. Trigger: /factory-apps,
  /factory-stack.
argument-hint: "[canary|powder|landmark|aesthetic|bitterblossom|audit]"
---

# /factory-apps

Use the owned factory app before inventing local state, bespoke glue, or a
generic third-party workflow. Product repos own the concrete skills and MCP
servers; Harness Kit imports those skills under `misty-*` aliases and manages
MCP policy in `.harness-kit/factory-mcps.yaml`.

## Router

| Need | App | First surface | Fallback |
|---|---|---|---|
| uptime, incidents, error timelines, health checks, service evidence, production debugging | Canary | Canary MCP from the `global` profile when registered | `misty-canary`, `/Users/phaedrus/Development/canary/bin/canary`, API |
| backlog, issue cards, claims, relations, operator input requests, work status | Powder | Powder MCP from the `non-adminifi-non-r90` profile when configured | `misty-powder`, CLI, API |
| release intelligence, versions, changelogs, release notes, release kit, fleet adoption | Landmark | `misty-landmark` and `landmark describe --json` / dry-run CLI/action paths | `docs/agent-integration.md`, `docs/fleet-integration-playbook.md` |
| UI/UX, Misty Step design law, tokens, static design registry, rendered design gate | Aesthetic | `misty-aesthetic`, `@misty-step/aesthetic` package, static API, law gate | `docs/ADOPTING.md`, `DESIGN.md` |
| ad-hoc supervised dispatch, event-triggered agents, reflex loops, durable runs | Bitterblossom | `misty-bitterblossom`, `bb` CLI, read-only MCP from `factory-ops` profile | product plane config |

## Operating Rule

- Production debugging starts with Canary state. Query service health,
  incidents, checks, and recent errors before making a repo-local hypothesis.
- Backlog or issue state lives in Powder. Do not keep durable card state in
  chat, TODO prose, or an ad-hoc markdown list when Powder is available.
- Release questions start with Landmark. Do not hand-write release
  intelligence from memory when the release app can describe the repo.
- UI and artifact design starts with Aesthetic. Use its tokens, recipes,
  registry, and law gate before adding one-off CSS vocabulary.
- Dispatch architecture starts with Bitterblossom only when the work is Mode B:
  triggered, scheduled, durable, reflexive, or event-driven. Ad-hoc operator
  work remains Harness Kit / Mode A.

## Current Audit

The live 2026-07-03 audit is in
`references/capability-audit-2026-07-03.md`. Load it when the question is
"are these configured?" or "what gaps remain?" before changing product repos
or system config.

## Fleet Integration Standard

For active Misty Step repos, load
`references/fleet-integration-standard.md` before claiming a project is
factory-integrated. The standard defines the repo-level Canary receipt,
Powder backlog, and Landmark manifest/workflow evidence expected for runtime
apps, libraries, and non-release support repos.

## Gotchas

- A product repo having an MCP implementation does not mean this harness has
  that MCP registered. Check `.harness-kit/factory-mcps.yaml` and the active
  harness config before claiming MCP availability.
- Do not add placeholder MCP servers. A broken registered tool is worse than a
  clear CLI/API fallback.
- Root product skills (`SKILL.md`) and portable product skills under
  `<repo>/skills/<name>/SKILL.md` are for consumers of the app. Repo-local
  `.agents/skills/*` are usually QA/deploy/dogfood runbooks for work inside
  that repo. Do not treat one as a substitute for the other.
- Bitterblossom's MCP is read-only in the audited checkout. Mutating dispatch
  and run control still go through the CLI/API unless the product changes.
