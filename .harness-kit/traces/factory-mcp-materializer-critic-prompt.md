Role: fresh-context critic.

Objective: review the current working-tree diff for backlog 135, "Add a
profile-aware factory MCP materializer".

Scope:
- Read only. Do not edit files, stage files, commit, or run destructive commands.
- Inspect `git diff -- . ':!docs/site' ':!index.yaml'`, the moved backlog item
  at `backlog.d/_done/135-profile-aware-factory-mcp-materializer.md`, and the
  live command/test surfaces named there.
- Judge the diff against the backlog oracle, especially public CLI workflow,
  profile/repo-scope filtering, env readiness, dry-run redaction, idempotent
  Codex config updates, and preservation of unrelated MCP config.

Output exactly:
- `BLOCKING: yes` or `BLOCKING: no`
- Findings by category: public surface, human workflow, compatibility,
  operations.
- Any exact file/line references needed.

Ignore style nits unless they hide one of those failures.
