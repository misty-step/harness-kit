# Integration Guide

Harness Kit integrations that reach external systems should default to this
order:

1. MCP server for the external-system boundary.
2. Skill atop MCP for operator judgment and workflow guidance.
3. CLI for local development, debugging, and one-off maintenance.

This guide is for external systems: SaaS APIs, internal services,
authenticated data stores, design tools, calendars, mailboxes, issue trackers,
CI providers, and production systems. Internal Harness Kit tooling such as
`bootstrap.sh`, `scripts/generate-index.sh`, local Dagger gates, and one-repo
maintenance helpers do not need MCP-ification.

## Reach For MCP

Use MCP first when the integration has one or more of these symptoms:

- **Multi-harness use:** Claude, Codex, Pi, Antigravity, and future clients
  should all call the same capability without bespoke wrappers.
- **Multi-repo reuse:** more than one downstream repo needs the same external
  system.
- **Auth negotiation:** the system needs OAuth, scoped credentials, user
  consent, or per-operator identity.
- **Interactive elicitation:** the server needs to ask the user for missing
  inputs, permissions, or third-party authorization during a tool call.
- **Tool-heavy surface:** the integration exposes multiple actions, resources,
  prompts, progress updates, cancellation, or structured errors.
- **Audit boundary:** the external system needs a narrow, inspectable contract
  instead of scattered shell scripts.

MCP is the boundary when the hard part is not local execution but safe,
portable access to external context and actions.

## Do Not Reach For MCP

Use a simpler shape when the work is local and single-purpose:

- **One-off script:** a migration, cleanup, report, or data transform that will
  run once or a few times.
- **Inner-loop only:** a repository-local test, formatter, index generator,
  Dagger gate, or bootstrap helper.
- **Local-dev-only:** a command whose value is direct terminal feedback and
  whose inputs are local files.
- **No external trust boundary:** the code does not cross auth, user-consent,
  remote API, or production-data boundaries.
- **One operation, one repo:** a shell or Python helper is clearer than a
  protocol server.

Do not turn internal Harness Kit scripts into MCP servers just because MCP
exists. Protocol surface is a liability unless it hides a real integration
boundary.

## Three-Layer Order

### 1. MCP Server

The server owns external-system mechanics: authentication, authorization,
resource discovery, tool schemas, prompt/resource exposure, progress,
cancellation, error reporting, and safety checks. It should be usable from any
MCP-capable client.

### 2. Skill Atop MCP

The skill owns human and agent judgment: when to use the integration, what
evidence to capture, what not to do, and how to interpret results. It should
name the MCP tools and resources it expects, but it should not reimplement
auth, pagination, retries, or protocol behavior in prose.

### 3. CLI For Local Dev

The CLI is for build, test, debug, and emergency local operation. It may share
library code with the MCP server, but it is not the primary agent contract
when multiple harnesses or repos need the integration.

## Work Source Receipts

When a work item, acceptance oracle, review, or operator prompt originates
outside local `backlog.d/`, record its identity as `work_source_refs` in the
local ledger, trace, or delegation receipt. These refs are evidence metadata:
they name `local_backlog`, `local_file`, `mcp_resource`, `cli_resource`, `url`,
or `manual` sources, plus optional snapshot and closure descriptors.

`work_source_refs` do not fetch, mutate, close, or synchronize external
systems. MCP servers own authenticated external resources and actions; CLIs
remain local/debug surfaces. Local `backlog_ref` remains the required
join/closeout key until a shaped migration replaces the numeric `backlog.d`
contract.

## Pointers

- [Model Context Protocol specification](https://modelcontextprotocol.io/specification/)
  defines the protocol boundary for connecting LLM applications to external
  tools, data, and workflows. Use it for the host/client/server model,
  capability negotiation, transports, authorization, resources, prompts,
  tools, sampling, roots, elicitation, and safety guidance.
- [MCP SDKs](https://modelcontextprotocol.io/docs/sdk) list official SDKs for
  building servers and clients. Use an official SDK before hand-rolling
  JSON-RPC plumbing.
- [Anthropic MCP documentation](https://docs.anthropic.com/en/docs/mcp) points
  to Anthropic-supported MCP usage and the current SDK path for supported
  clients and servers.
- [MCP introduction](https://modelcontextprotocol.io/docs/getting-started/intro)
  frames MCP as a standard way for AI applications to connect to external
  systems, data sources, tools, and workflows.

## Prior Art Signals

This doctrine comes from recurring downstream pain:

- **bitterblossom sprite orchestration:** external tool orchestration became
  opaque because the integration shape was not separated from workflow prose.
- **canary harness dispatch:** review and harness dispatch patterns needed a
  stable integration boundary instead of repo-local convention alone.
- **cerberus reviewer patterns:** external reviewer behavior needed a portable
  contract for recurring findings rather than scattered local glue.

The exact systems differ, but the failure mode is the same: once external
state, tools, identity, and agent workflows mix without a protocol boundary,
every consuming repo invents a slightly different integration model.

## Decision Checklist

Before adding an external integration, answer:

- What is the external system of record?
- Which users or agents need access?
- Does access require identity, consent, scopes, or authorization refresh?
- Which capabilities are tools, which are resources, and which are prompts?
- What evidence proves the integration worked?
- What local CLI command helps debug the server without becoming the primary
  agent interface?

If those answers span more than one harness or repo, start with MCP.
