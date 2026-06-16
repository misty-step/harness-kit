# Works Critique Sample

Fixture: a CLI change adds `harness-kit-checks publish` and tests only assert
that the command exits zero.

Critic output using `harnesses/shared/references/works-critique.md`:

`BLOCKING: yes`

- Public surface: `publish` does not say whether it pushes, opens a PR, writes
  local artifacts, or mutates generated docs; the nearby CLI uses explicit
  verbs like `build-docs-site`, `sync-external`, and `check-index-drift`.
- Human workflow: no `--dry-run`, no evidence path, and no final remote-sync
  output, so an operator cannot tell what changed without reading git state.
- Compatibility: the command silently assumes `origin/master`; repos with a
  different intended remote would appear to "work" locally and fail closeout.
- Operations: no receipt, transcript, or log line is written, so a failed
  publish cannot be audited later.

Verdict: tests are necessary but insufficient; the public surface and operator
workflow do not work yet.
