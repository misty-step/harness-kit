# CI Audit Rubric

Use this to decide whether a repo's CI is strong, fast, portable, and
agent-legible enough to trust. For host-agnostic CI design, load
`host-agnostic-ci.md` first.

For consumer repos, first identify the repo-owned gate from root instructions,
package manifests, CI workflows, hooks, and shipped scripts. Do not assume the
Harness Kit Rust gate runs there. Apply this rubric to the repo's actual gate.

For Harness Kit itself, the source-repo maintenance gate lives in
`crates/harness-kit-checks/src/ci_check.rs`. That Rust gate is Harness Kit
plumbing, not a general CI framework.

## Required Checks

- One repo-owned contract is named: command, script, Dagger function,
  Make/Just/Task target, or build-system target.
- Local, GitHub Actions, Azure, or other runners call the same repo-owned
  contract or a clearly documented fast/full tier of it.
- Fast and full gates are distinct unless live evidence proves one gate is
  both comprehensive and fast enough for repeated agent use.
- Generated `index.yaml`, docs, API clients, schemas, fixtures, or lockfiles
  are checked when relevant.
- Format, lint, typecheck, tests, and build/package checks are covered in the
  tier where they belong.
- Secret scanning covers both working-tree content and Git/PR metadata:
  commit message file, outbound commit subjects/bodies, PR title/body, release
  notes, changelog text, generated summaries, and logs. Findings must identify
  the field and rule without printing the secret value.
- At least one protection runs before remote publication: server-side push
  protection or pre-receive hook when available; otherwise `commit-msg` plus
  `pre-push` hooks. CI is still required because local hooks can be bypassed.
- Reports are generated or explicitly waived: run digest, test report,
  coverage/diff-coverage, security findings, artifact checksums, and perf or
  mutation output where relevant.
- Required hosted checks cannot be skipped into false-green or stuck-pending
  states by path filters; add sentinel checks when necessary.

## Speed Rules

- Docs/backlog-only push path should be seconds, not minutes.
- Fast local gate should avoid Docker and network unless container/service
  orchestration is the point.
- Dagger is valid when it buys portability, pinned services, caching,
  containerized dependency graphs, or traceability; it is not valid as a
  slow wrapper around ordinary host commands in the inner loop.
- If external API behavior is required and `emulate.dev` supports the provider,
  use local emulation plus seeded fixtures as the offline behavioral gate.
- Expensive checks belong behind explicit commands or path-scoped triggers.
- If the full local gate is too slow, split fast/full tiers; do not delete the
  invariant.
- Track or report gate duration, critical path, and cache behavior when the
  substrate exposes them.

## Harness Kit Source Repo

Only when auditing Harness Kit itself:

- Root `AGENTS.md` names
  `cargo run --locked -p harness-kit-checks -- check --repo .`.
- `.githooks/pre-push` uses changed-path classification and calls the Rust gate
  for source/harness changes.
- `pre-merge-commit` calls the Rust gate.
- Generated `index.yaml` and `docs/site` drift are checked.
- Frontmatter, roster, evidence, offline evidence, runtime primitive, skill
  eval, no-claims, portable-path, conflict-marker, and deliver-composition
  checks are covered.
- Rust `fmt`, `test`, and `clippy -D warnings` are covered.
- Active skill prose does not tell operators that archived Dagger history is
  the current Harness Kit source-repo gate.

Do not add consumer-repo CI framework code to Harness Kit to satisfy this
rubric. Update skill guidance, templates, or repo-local consumer gates.

## Audit Findings

| Severity | Meaning | Action |
|---|---|---|
| high | Missing repo-owned gate, hosted provider owns the only contract, source changes bypass required checks, false-green path filters, or secrets can reach remote commit/PR metadata unscanned | Fix inline |
| med | Gate is too slow, noisy, host-specific, report-poor, or duplicates an invariant | Simplify inline or file backlog |
| low | Naming/docs drift | Fix when touching nearby files |

Historical Dagger references in archived backlog are not findings. Live skills,
root docs, hooks, and generated reference pages must not imply one universal
substrate. Dagger, direct host scripts, and build-system targets are all
acceptable when the repo evidence earns them.
