# harness-kit — ARCHIVED

**This repository is archived (2026-07-07). Its successor is
[misty-step/roster](https://github.com/misty-step/roster).**

Everything here migrated to roster under the roster-926 epic: the 26
first-party skills (refactored to the roster authoring standard), 33 vendored
externals with pinned provenance, the shared operating doctrine
(`primitives/shared/AGENTS.md`), the harness bootstrap templates, the live
Claude hooks (`roster-hooks`), the catalog gate (`roster check`), and the
machine sync (`roster sync`). The full path-by-path accounting — migrated /
consciously dropped / stays-with-archive — is
`roster:docs/harness-kit-disposition-ledger.md`.

- Skills, agents, doctrine, sync: `roster sync --catalog full --all-agents`
- Was here, deliberately not ported: the ~25-verb `harness-kit-checks` gate
  surface (semantic string-matching verbs and platform tooling — see the
  ledger's "gate the deterministic consumers, model-judge the model
  consumers" ruling), the docs-site generator (successor: roster-928 live
  UI), `bundles.yaml` (superseded by role.yaml skill lists +
  `sync --catalog curated`), the dispatch-agent receipt system (receipts
  live on Powder runs now).
- Local hooks and `bootstrap.sh` stand down when roster's sync manifest is
  present; `ROSTER_ROLLBACK=1` forces the old behavior for explicit rollback.

History preserved in full — this repo is the archaeological record.
