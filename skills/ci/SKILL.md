---
name: ci
description: |
  Audit CI gates, strengthen weak coverage, then drive green. Harness Kit uses
  the Rust-owned local gate, not Dagger, as the canonical repo check:
  `cargo run --locked -p harness-kit-checks -- check --repo .`. Acts directly
  on mechanical fixes and never returns red without structured diagnosis. Use
  when: "run ci", "check ci", "fix ci", "audit ci", "is ci passing", "run the
  gates", "why is ci failing", "strengthen ci", "tighten ci", "ci is red",
  "gates failing". Trigger: /ci, /gates.
argument-hint: "[--audit-only|--run-only]"
---

# /ci

Confidence in correctness without turning local work into a Docker tax.

Harness Kit's canonical gate is:

```sh
cargo run --locked -p harness-kit-checks -- check --repo .
```

The gate is implemented in Rust at
`crates/harness-kit-checks/src/ci_check.rs`. Dagger may exist as a legacy or
experimental runner, but it is not the default local gate, not required for
pre-push, and not the source of truth for shipping this repo.

When `/ci` runs in a consumer repo, do not assume Harness Kit's Rust gate is
installed there. Read that repo's root instructions, package manifests, CI
workflows, hook config, and shipped scripts, then strengthen the repo-owned
gate. Harness Kit can supply reusable checks, but the acceptance question is
whether that repo has an active gate an agent will actually hit.

## Modes

- Default: audit the gate surface, fix mechanical gaps, then run the Rust gate.
- `--audit-only`: produce audit report and gap proposals; do not run gates.
- `--run-only`: skip audit, just drive the Rust gate green.

## Stance

1. **Rust owns the gate.** Add or remove default gates in
   `crates/harness-kit-checks/src/ci_check.rs`. Do not add default CI behavior
   through Dagger, shell glue, or provider YAML.
2. **Fast enough to use.** A default local gate that routinely takes many
   minutes for harness/docs changes is a design failure. Keep expensive,
   networked, mutation, browser, provider, and experimental checks opt-in or
   path-scoped.
3. **No quality lowering.** Removing Dagger is not permission to remove the
   invariant. Preserve meaningful checks by moving them into direct Rust
   functions or narrowly scoped commands.
4. **Act, do not propose.** Mechanical strengthenings are applied directly.
   Escalate only when the choice is product scope, not CI plumbing.
5. **Fix-until-green on self-healable failures.** Formatting drift, stale
   generated docs/index, and trivial lints get fixed. Logic failures get a
   precise diagnosis.
6. **Security floor is part of CI.** A credible repo gate prevents or fails on
   secret leaks in source files, generated artifacts, logs, and Git/PR
   metadata. Commit subjects/bodies, PR titles/bodies, release notes, and
   agent-generated summaries are in scope. Prefer server-side push protection
   or pre-receive hooks when available; otherwise require repo hooks plus CI.

## Delegation Judgment

For substantive gate-policy changes, delegate on judgment per the shared
Roster contract: native subagents by default; when the decision is
architectural or risky, add a cross-model critic or scoped roster lanes with
lane handoff prompts. See `harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Each lane states responsibilities, context boundary,
output evidence, and lead verification. Direct work is limited to mechanical
repair and emergency preservation. The lead owns synthesis.

## Audit

Check the live gate surface:

- Root contract names `cargo run --locked -p harness-kit-checks -- check --repo .`.
- `.githooks/pre-push` routes through `harness-kit-checks git-hook pre-push`.
- `git_hooks.rs` uses changed-path classification and calls the Rust gate for
  source/harness changes.
- `ci_check.rs` contains the default lane list.
- Generated docs/index are current after skill/docs/backlog changes.
- Dagger references are legacy/optional only, never mandatory local closeout.
- Secret scanning covers both committed content and metadata that never appears
  in the working tree: commit message file, outbound commit range, PR title/body,
  and release/changelog text. The report must redact matched values.

For non-Harness Kit repos, replace the Harness Kit-specific bullets above with
that repo's equivalent gate contract, then apply the same security floor.

## Run

Run:

```sh
cargo run --locked -p harness-kit-checks -- check --repo .
```

If red:

- Fix deterministic generated drift.
- Run focused tests for the failing Rust module.
- Re-run the aggregate gate.
- Stop after three self-heal attempts per gate and report the exact failing
  command, file/path, and likely cause.

## Output

Report:

- **Audit:** gaps found, severity, what was strengthened, what was deferred.
- **Run:** gate command, pass/fail, self-heals, escalations.
- **Final:** green/red, residual risk, and any deferred heavyweight checks.

Never claim green from Dagger alone in this repo.
