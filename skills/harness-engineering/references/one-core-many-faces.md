# One Core, Many Faces

Use when a factory repo should expose one product through API, CLI, MCP, SDK,
human UI, deploy runtime, and a shipped skill without duplicating business
logic.

**Greenfield-only.** This template is for a new product's first commit, not a
retrofit of an existing repo. The 2026-07-01 groom teardown estimated
60-116 engineer-hours per repo to retrofit an existing fleet repo (canary is a
7-crate workspace, crucible a 2-crate one — neither matches the template
topology) and judged retrofit uneconomical. If a repo already ships a working
product in a different shape, do not migrate it to this layout; the shape
argument in `skills/harness-engineering/SKILL.md` and this file's own doctrine
still apply without the file layout.

## Source Signals

- In-house exemplar: `~/Development/bastion/apps/cairn` ships web, JSON API,
  CLI, and MCP from one Rust app. The useful shape is: domain functions behind
  one state boundary; API and web call those functions; CLI and MCP share a
  client surface.
- Dagger reference: one engine/API core, CLI as a client, SDKs generated from
  the API type system.
- Stripe reference: one official AI surface combines SDK-style packages, MCP,
  and agent skills so agents do not have to rediscover integration patterns.

## Target Shape

```text
AGENTS FIRST                                             HUMANS THIN
api   cli   mcp   sdk   web/control   deploy   SKILL.md   AGENTS.md
 |     |     |     |        |          |          |          |
 +-----+-----+-----+--------+----------+----------+----------+
                  shell / ports / use cases
                         |
                      core crate
```

The rule that proves the design: a business-rule change lands in `core` or the
shell use-case layer once, and every face inherits it.

## Crate Roles

`core`
: Pure domain types and rules. No HTTP, filesystem, database, environment,
clock, subprocess, model provider, or harness imports.

`shell`
: Use cases plus effect ports. Owns persistence, clocks, model/provider calls,
remote APIs, approval checks, budgets, and audit events through traits or small
adapters.

`api`
: HTTP transport. Parses requests, calls shell use cases, serializes responses,
returns structured errors. No product policy.

`cli`
: Local operator and CI surface. Calls shell directly for local-first tools or
calls the API/SDK when the command is explicitly remote. No separate rules.

`mcp`
: Agent-native intent surface. Tools are task-shaped and risk-separated. It
calls shell use cases or the same client surface as CLI. Prefer a current MCP
SDK over hand-rolled JSON-RPC.

`sdk`
: Consumer package over the stable API or generated schema. Keep non-Rust SDKs
tiny unless a real consumer requires them. Generate when the schema is stable
enough; otherwise ship a typed minimal client.

`deploy`
: Process edge: Docker image, Fly runtime config, `/data` mount, Litestream
restore/replicate wrapper, and typed config-from-env. No product policy and no
business logic.

`SKILL.md`
: Agent judgment: when to use the tool, what evidence to capture, safety
boundaries, and exact commands. It should not reimplement API docs.

`AGENTS.md`
: Repo contract: gate, base branch, red lines, lifecycle, and where depth
lives. Short map, not manual.

## MCP Tool Design

Do not translate endpoints 1:1. Design tools around agent intent:

- Prefer `prepare_release_packet(repo, since)` over `get_commits`,
  `get_tags`, `write_notes`, `open_release`.
- Keep one risk level per tool: read-only, write, or destructive.
- Keep the default server small enough to scan. Five to fifteen tools is a
  useful target; split or hide rare tools behind resources/prompts.
- Tool descriptions are prompts. Include when to use the tool, argument format,
  and what evidence the agent should read after the call.
- Return structured errors the model can correct from; do not crash the server
  on validation errors.

## Structure At The AI Seam

Only impose schema where deterministic code branches, persists, dedupes,
checks policy, or renders a fixed UI slot. If the next reader is an LLM, carry
rich prose or loosely shaped context. The model needs the information, not a
taxonomy.

## Template

Copy `skills/harness-engineering/templates/one-core-many-faces/` into a new
repo or planning branch and replace:

- `{{project}}` with the product name.
- `{{client_class}}` with a TypeScript-safe client class prefix.
- `{{crate_prefix}}` with the Rust crate prefix, for example `landmark`.
- `{{binary}}` with the CLI binary name.
- `{{repo}}` with `owner/repo`.
- `{{base_branch}}` with the repository base branch, for example `main` or
  `master`.
- `{{npm_scope}}` with the npm scope, for example `misty-step`.
- `{{fly_app}}` with the Fly.io app name.
- `{{fly_region}}` with the Fly.io primary region.
- `{{description}}` with one concrete product sentence.

Then delete every face the first slice cannot verify. The template is a menu,
not a mandate. A repo earns a face by naming its proof loop.

The template itself cannot drift silently: `check-template`
(`crates/harness-kit-checks/src/template_check.rs`, part of this repo's own
`check --repo .`) materializes every `.tmpl` file with sample tokens into a
temp workspace, checks no token survives substitution, and runs
`cargo generate-lockfile && cargo build --locked --workspace` against it. A
broken template file fails Harness Kit's own gate, not just a consumer's.

## Verification System

Minimum proof for the scaffold itself:

- Core: unit tests over public domain functions.
- Shell: fixture-backed integration test with fake ports only at external
  boundaries.
- API: representative request replay, including one validation error.
- CLI: happy path plus malformed input path with stdout/stderr assertions.
- MCP: harness registration or protocol replay for `initialize`, `tools/list`,
  a read tool, and one structured error.
- SDK: throwaway consumer build that imports the package and calls one public
  method.
- Deploy: materialized Docker build, `fly.toml` validation, `/healthz` and
  `/readyz` smoke, and Litestream restore drill against a non-production
  database or explicit pre-production waiver.
- Skill: cold-agent smoke: ask the agent to use the product from only the
  skill and repo docs, then inspect the evidence it leaves.

## Anti-Patterns

- 1:1 REST-to-MCP wrapping.
- Shallow adapter pass-throughs that add surface without simplifying a user or
  agent workflow.
- Duplicated business logic across API, CLI, MCP, SDK, and UI.
- Reflexive schemas at model-facing seams.
- MCP tools that mix read and destructive behavior.
- Generated SDKs or clients treated as done without a consumer build.
- Skills that restate CLI help and omit judgment, evidence, or safety
  boundaries.
- Human UI that tries to match every API operation instead of owning oversight,
  approval, trace, and recovery.
- Deploy files copied without typed env parsing, readiness probes, and restore
  evidence.
