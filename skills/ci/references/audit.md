# CI Audit Rubric

Use this to decide whether the Rust-owned local gate is strong enough to trust.
For Harness Kit, default CI lives in
`crates/harness-kit-checks/src/ci_check.rs`, not Dagger.

For consumer repos, first identify the repo-owned gate from root instructions,
package manifests, CI workflows, hooks, and shipped scripts. Do not assume the
Harness Kit Rust gate runs there. Apply this rubric to the repo's actual gate.

## Required Checks

- `cargo run --locked -p harness-kit-checks -- check --repo .` exists and runs.
- Root `AGENTS.md` names that command as the repo gate.
- `.githooks/pre-push` uses changed-path classification and calls the Rust gate
  for source/harness changes.
- `pre-merge-commit` calls the Rust gate, not Dagger.
- Generated `index.yaml` and `docs/site` drift are checked.
- Frontmatter, roster, evidence, offline evidence, runtime primitive, skill
  eval, no-claims, portable-path, conflict-marker, and deliver-composition
  checks are covered.
- Rust `fmt`, `test`, and `clippy -D warnings` are covered.
- Secret scanning covers both working-tree content and Git/PR metadata:
  commit message file, outbound commit subjects/bodies, PR title/body, release
  notes, changelog text, generated summaries, and logs. Findings must identify
  the field and rule without printing the secret value.
- At least one protection runs before remote publication: server-side push
  protection or pre-receive hook when available; otherwise `commit-msg` plus
  `pre-push` hooks. CI is still required because local hooks can be bypassed.

## Speed Rules

- Docs/backlog-only push path should be seconds, not minutes.
- Source/harness full local gate should avoid Docker and network.
- Expensive checks belong behind explicit commands or path-scoped triggers.
- If the full local gate is too slow, fix `ci_check.rs`; do not reintroduce
  Dagger as the default.

## Audit Findings

| Severity | Meaning | Action |
|---|---|---|
| high | Missing local gate, hook still calls Dagger, source changes bypass checks, or secrets can reach remote commit/PR metadata unscanned | Fix inline |
| med | Gate is too slow, noisy, or duplicates an invariant | Simplify inline or file backlog |
| low | Naming/docs drift | Fix when touching nearby files |

Historical Dagger references in archived backlog are not findings. Live skills,
root docs, hooks, and generated reference pages must not tell operators to use
Dagger as the canonical gate.
