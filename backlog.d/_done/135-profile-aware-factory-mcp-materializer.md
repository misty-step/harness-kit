# Add a profile-aware factory MCP materializer

Priority: P1 (shipped) · Status: done · Estimate: M · Shipped: 2026-07-03

## Goal

Turn `.harness-kit/factory-mcps.yaml` from a validated installed registry into
an applyable MCP installation surface for each supported harness.

## Oracle

- [x] A command or bootstrap mode reads `.harness-kit/factory-mcps.yaml` and can
      install active MCP entries for at least Codex without hand-written
      `codex mcp add` steps.
- [x] Profile and repo-scope filtering are enforced: Canary applies globally;
      Powder applies to non-Adminifi/non-r90 project scopes only; Bitterblossom
      applies only to factory-ops scopes.
- [x] `required_env_any` is honored before activation, so Powder is skipped with
      an explicit explanation until `POWDER_API_BASE_URL` plus `POWDER_API_KEY`,
      or `POWDER_DB_PATH`, exists. This machine now satisfies the API group via
      Agents-vault `env_sources`; fixture tests cover the missing-source skip
      path.
- [x] The command supports dry-run output that shows planned add/skip/update
      actions without printing secret values.
- [x] Existing manually registered MCPs are updated idempotently when the
      registry command changes, and unrelated user MCPs are preserved.
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Verification System

- Claim: factory MCP policy can be centrally managed by Harness Kit without
  creating broken global tools or clobbering user-owned harness config.
- Falsifier: a profile-mismatched repo gets Powder; missing Powder env still
  installs a failing MCP; unrelated MCP config changes; dry-run leaks secrets;
  bootstrap and active config disagree.
- Driver: registry fixture tests, dry-run transcript, one real Codex apply
  against a temporary `$CODEX_HOME`, and one real local apply on this machine.
- Grader: generated/active MCP configs contain only profile-matched runnable
  entries; `codex mcp list` shows expected names; smoke tests pass for Canary
  and Bitterblossom; Powder skip reason is explicit.
- Evidence packet: dry-run output, temp-home config diff, active `codex mcp
  list` excerpt with secrets redacted by the tool, and MCP initialize/tools-list
  smoke transcripts.
- Cadence: first implementation, then whenever `.harness-kit/factory-mcps.yaml`
  schema changes.

## Notes

Source: 2026-07-03 factory-app audit/remediation. The first remediation added
the product-owned skill imports, a validated factory MCP registry, bootstrap
symlinks for that registry, and active local Codex registration for Canary and
Bitterblossom. A follow-up checked the 1Password Agents vault and found the
deployed Powder endpoint and API key, so this machine now has active local Codex
registration for Canary, Powder, and Bitterblossom. This ticket remains the
installer layer: profile-aware materialization from registry to active harness
config without manual `codex mcp add` commands.

## Closure Evidence

- Added `harness-kit-checks apply-factory-mcps` for Codex, with `--profile`,
  `--all-profiles`, `--project`, `--codex-home`, `--dry-run`,
  `--check-env`, and `--skip-env-check`.
- Proved dry-run on the live registry: after apply, Canary, Powder, and
  Bitterblossom report `UNCHANGED`; Adminifi and r90 project dry-runs skip
  Powder with an explicit scope reason.
- Proved temp `$CODEX_HOME` apply with env checks: Canary, Powder, and
  Bitterblossom were added from `.harness-kit/factory-mcps.yaml`.
- Applied the registry to this machine's active Codex config; `codex mcp list`
  shows `canary`, `powder`, and `bitterblossom` enabled.
- Fresh-context critic verdict: `BLOCKING: no`, recorded at
  `.harness-kit/traces/factory-mcp-materializer-critic-verdict.md`.
