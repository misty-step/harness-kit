# Factory App Capability Audit - 2026-07-03

Scope: live local checkouts for Canary, Powder, Landmark, Aesthetic,
Bitterblossom, plus Harness Kit and the local Codex config. This is an audit
snapshot, not a product status source.

## Summary Matrix

| App | Role | Skills | MCP | SDK | Harness/system state | Gap |
|---|---|---|---|---|---|---|
| Canary | observability, uptime, incidents, health checks, error timelines | product root `SKILL.md`, imported as `misty-canary`; repo-local `canary-qa` and `canary-deploy` | implemented via `bin/canary mcp-server`; registered in factory MCP `global` profile | TypeScript SDK in `clients/typescript` | trusted project path exists; Harness Kit imports product skill and installs factory MCP registry | none for current skill/MCP registry; full profile-aware MCP materialization is tracked separately in backlog `135` |
| Powder | backlog, issues/cards, claims, relations, operator input | root product `SKILL.md`, imported as `misty-powder`; repo-local `powder-qa` | implemented in `crates/powder-mcp`; profile-gated for non-Adminifi/non-r90 repos; active Codex launcher reads `op://Agents/POWDER_ENDPOINT/URL` and `op://Agents/POWDER_API_KEY__bridge/credential` | no SDK observed | trusted project path exists; Harness Kit imports product skill and records MCP profile policy | SDK absent; full profile-aware MCP materialization is tracked separately in backlog `135` |
| Landmark | release intelligence, versions, changelogs, release kit, fleet adoption | product root `SKILL.md`, imported as `misty-landmark`; dogfood skill remains contributor-facing | no MCP observed | no SDK observed | trusted project path exists; Harness preferred stack now says Landmark | no MCP/SDK; current product-owned surface is skill + CLI/action |
| Aesthetic | UI/UX system, Misty Step law, tokens, static registry | product root `SKILL.md`, imported as `misty-aesthetic` | no MCP observed | package/static API via `@misty-step/aesthetic` | trusted project path exists; Harness Kit imports product skill | CLI/MCP intentionally later per local vision |
| Bitterblossom | ad-hoc supervised dispatch, Mode B reflex loops, durable runs | portable product skill in `skills/bitterblossom`, imported as `misty-bitterblossom`; repo-local dogfood skill | read-only MCP via `bb --config <plane> mcp serve`; registered in factory MCP `factory-ops` profile | no SDK observed | trusted project path exists; Harness Kit imports product skill and installs factory MCP registry | mutating MCP tools remain intentionally absent |

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

The local Codex config trusts the five app checkout paths. Harness Kit now
ships `.harness-kit/factory-mcps.yaml` as the managed factory MCP registry and
bootstrap links it to `~/.harness-kit/factory-mcps.yaml`.

Do not register placeholder MCPs. Register only when the real instance and
auth source are known:

- Canary: command `bin/canary mcp-server`; registered in the `global` profile
  through a secret-free launcher that inherits env or reads
  `op://Agents/CANARY_ENDPOINT/credential` and
  `op://Agents/CANARY_API_KEY/credential`.
- Powder: `powder-mcp`; registered for the `non-adminifi-non-r90` profile and
  configured from the Agents vault with `POWDER_API_BASE_URL` from
  `op://Agents/POWDER_ENDPOINT/URL` and `POWDER_API_KEY` from
  `op://Agents/POWDER_API_KEY__bridge/credential`.
- Bitterblossom: `bb --config <plane> mcp serve`; registered for the
  `factory-ops` profile using the local plane. The audited MCP is read-only.
- Landmark: no MCP server observed; use CLI/action until the product exposes
  one.
- Aesthetic: no MCP server observed; use package/static API/law gate until the
  product exposes one.

## Remediated In Harness Kit

- Added first-party `factory-apps` skill so future agents have an app-visible
  router for Canary, Powder, Landmark, Aesthetic, and Bitterblossom.
- Added product-owned external skill imports in `registry.yaml`:
  `misty-canary`, `misty-powder`, `misty-landmark`, `misty-aesthetic`, and
  `misty-bitterblossom`.
- Added `.harness-kit/factory-mcps.yaml` plus `check-mcp-registry` so MCP
  profile policy is data, validated, and bootstrapped.
- Updated Harness Engineering preferred stack defaults:
  - Powder is the default backlog/work-state system.
  - Landmark replaces stale Landfall naming for release intelligence.
  - Canary production-debugging and consumer integration expectations are
    explicit.
  - Aesthetic default references package/static API/law, not just prose taste.

## Remaining Product Gaps

These require clean product-repo branches or concrete deployment credentials:

- Decide whether Powder needs a small SDK or if API/CLI/MCP is sufficient.
- Decide whether Landmark earns an MCP or whether CLI/action remains the right
  agent surface.
- Decide whether Aesthetic earns an MCP after repeated adoption work proves it
  needs one beyond skill/package/static API.
- Build the profile-aware MCP materializer tracked in backlog `135`; until
  then the registry is installed system-wide and this machine's Codex config is
  manually aligned with it.
