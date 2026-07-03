# Factory App Capability Audit - 2026-07-03

Scope: live local checkouts for Canary, Powder, Landmark, Aesthetic,
Bitterblossom, plus Harness Kit and the local Codex config. This is an audit
snapshot, not a product status source.

## Summary Matrix

| App | Role | Skills | MCP | SDK | Harness/system state | Gap |
|---|---|---|---|---|---|---|
| Canary | observability, uptime, incidents, health checks, error timelines | repo-local `canary-qa` and `canary-deploy` | implemented via `bin/canary mcp-server` | TypeScript SDK in `clients/typescript` | trusted project path exists; no Codex MCP server registered | missing root product skill; MCP not registered on this system |
| Powder | backlog, issues/cards, claims, relations, operator input | root product `SKILL.md`; repo-local `powder-qa` | implemented in `crates/powder-mcp` | no SDK observed | trusted project path exists; no Codex MCP server registered | SDK absent; MCP not registered; checkout was diverged |
| Landmark | release intelligence, versions, changelogs, release kit, fleet adoption | dogfood skill only | no MCP observed | no SDK observed | trusted project path exists; Harness preferred stack said Landfall | missing product skill, MCP, SDK; stale Harness naming |
| Aesthetic | UI/UX system, Misty Step law, tokens, static registry | no product skill observed | no MCP observed | package/static API via `@misty-step/aesthetic` | trusted project path exists; no app skill in Harness catalog | missing product skill; CLI/MCP intentionally later per local vision |
| Bitterblossom | ad-hoc supervised dispatch, Mode B reflex loops, durable runs | portable product skill in `skills/bitterblossom`; repo-local dogfood skill | read-only MCP via `bb --config <plane> mcp serve` | no SDK observed | trusted project path exists; no Codex MCP server registered | product skill not in Harness catalog before this router; MCP not registered |

## Evidence Read

- Canary:
  - `/Users/phaedrus/Development/canary/README.md`
  - `/Users/phaedrus/Development/canary/docs/factory-fleet-integration.md`
  - `/Users/phaedrus/Development/canary/docs/compatibility-policy.md`
  - `/Users/phaedrus/Development/canary/clients/typescript/package.json`
  - `/Users/phaedrus/Development/canary/.agents/skills/canary-qa/SKILL.md`
  - `/Users/phaedrus/Development/canary/.agents/skills/canary-deploy/SKILL.md`
- Powder:
  - `/Users/phaedrus/Development/powder/SKILL.md`
  - `/Users/phaedrus/Development/powder/AGENTS.md`
  - `/Users/phaedrus/Development/powder/README.md`
  - `/Users/phaedrus/Development/powder/crates/powder-mcp/Cargo.toml`
- Landmark:
  - `/Users/phaedrus/Development/landmark/README.md`
  - `/Users/phaedrus/Development/landmark/docs/agent-integration.md`
  - `/Users/phaedrus/Development/landmark/docs/fleet-integration-playbook.md`
  - `/Users/phaedrus/Development/landmark/skills/landmark-dogfood/SKILL.md`
  - `/Users/phaedrus/Development/landmark/package.json`
- Aesthetic:
  - `/Users/phaedrus/Development/aesthetic/README.md`
  - `/Users/phaedrus/Development/aesthetic/docs/ADOPTING.md`
  - `/Users/phaedrus/Development/aesthetic/docs/vision.md`
  - `/Users/phaedrus/Development/aesthetic/law/README.md`
  - `/Users/phaedrus/Development/aesthetic/package.json`
  - `/Users/phaedrus/Development/aesthetic/DESIGN.md`
- Bitterblossom:
  - `/Users/phaedrus/Development/bitterblossom/skills/bitterblossom/SKILL.md`
  - `/Users/phaedrus/Development/bitterblossom/README.md`
  - `/Users/phaedrus/Development/bitterblossom/AGENTS.md`
  - `/Users/phaedrus/Development/bitterblossom/docs/spine.md`
  - `/Users/phaedrus/Development/bitterblossom/.agents/skills/bb-dogfood/SKILL.md`
- Harness/system:
  - `/Users/phaedrus/Development/harness-kit/skills/harness-engineering/references/preferred-stack.md`
  - `/Users/phaedrus/.codex/config.toml` server names only; credential values were not copied
  - active Codex tool discovery for factory app MCP names

## System Configuration Finding

The local Codex config trusts the five app checkout paths, but the registered
MCP servers are unrelated general tools. No Canary, Powder, Landmark,
Aesthetic, or Bitterblossom MCP server was active in the audited session.

Do not register placeholder MCPs. Register only when the real instance and
auth source are known:

- Canary: command `bin/canary mcp-server`; needs endpoint and responder/read
  credentials from the service environment.
- Powder: `powder-mcp` or equivalent CLI wrapper; needs either a local
  `POWDER_DB_PATH` or `POWDER_API_BASE_URL` plus `POWDER_API_KEY`.
- Bitterblossom: `bb --config <plane> mcp serve`; needs the intended plane
  path. The audited MCP is read-only.
- Landmark: no MCP server observed; use CLI/action until the product exposes
  one.
- Aesthetic: no MCP server observed; use package/static API/law gate until the
  product exposes one.

## Remediated In Harness Kit

- Added first-party `factory-apps` skill so future agents have an app-visible
  router for Canary, Powder, Landmark, Aesthetic, and Bitterblossom.
- Updated Harness Engineering preferred stack defaults:
  - Powder is the default backlog/work-state system.
  - Landmark replaces stale Landfall naming for release intelligence.
  - Canary production-debugging and consumer integration expectations are
    explicit.
  - Aesthetic default references package/static API/law, not just prose taste.

## Remaining Product Gaps

These require clean product-repo branches or concrete deployment credentials:

- Add a Canary root product `SKILL.md` that tells consumers how to query and
  integrate Canary, distinct from QA/deploy skills for Canary contributors.
- Add a Landmark product skill. Decide whether Landmark needs an MCP or whether
  CLI/action remains the right agent surface.
- Add an Aesthetic product skill first; its own vision doc names skill as the
  cheapest missing agent surface. CLI and MCP can follow only if repeated
  adoption work proves they are needed.
- Decide whether Powder needs a small SDK or if API/CLI/MCP is sufficient.
- Register Canary, Powder, and Bitterblossom MCP servers in harness config
  only after the real endpoint/database/plane and non-interactive credential
  source are known.
