# Factory MCP Materializer Critic Verdict

Provider target: `pi` (`openrouter/moonshotai/kimi-k2.7-code`)
Backlog: `135`
Prompt: `.harness-kit/traces/factory-mcp-materializer-critic-prompt.md`
Transcript: `.harness-kit/traces/provider-lanes/20260703T220329.096906Z-pi-95bfbe90.txt`

## Verdict

BLOCKING: no

## Findings

- Public surface: `apply-factory-mcps` is wired and documented in the CLI usage
  path, with the expected profile/project/codex-home/dry-run/env flags.
- Human workflow: the command satisfies the backlog's "command or bootstrap
  mode" path; bootstrap still only installs the registry file.
- Compatibility: custom TOML replacement is intentionally narrow but covered by
  idempotence and unrelated-table preservation tests for the current Codex
  `[mcp_servers.NAME]` format.
- Operations: apply mode checks `op read` readiness for missing env vars by
  default; dry-run output omits launcher command values and secret values.
