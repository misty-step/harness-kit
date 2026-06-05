# Dagger as merge gate — replace GitHub Actions CI

Priority: high
Status: done
Estimate: M

Paired with 022 (swarm review default). Raised from medium → high during
2026-04-14 grooming — swarm review without a CI merge gate is a half-story;
these two land together.

## Goal

Make Dagger the merge gate, not just the pre-push local check. Currently
Dagger runs locally before push, but there's no server-side enforcement.
A merge to master should be impossible without passing `dagger call check`.

## Design

For solo/small-team (Harness Kit's current reality):
- Pre-merge git hook runs `dagger call check` before allowing merge to master
- `/land` command (from 021) enforces this as part of its workflow
- No GitHub Actions needed — enforcement is local + git hooks

For collaboration scale (future):
- Lightweight webhook handler triggers `dagger call check` on push to review branches
- Reports status back via git notes or verdict refs
- Dagger Cloud as optional hosted runner

## Why Not Just GitHub Actions

- Dagger runs identical pipelines locally and remotely
- No YAML — pipelines are Python code
- Agentic LLM integration (v0.18+) enables self-healing
- Eliminates push-wait-read loop entirely

## Oracle

- [x] `git merge feat-foo` into master fails without passing Dagger check
- [x] `/ship` runs Dagger check as part of the current landing workflow
- [x] No `.github/workflows/` files needed for CI enforcement

## Non-Goals

- Building a webhook server (keep it local for now)
- Replacing GitHub as git remote

## What Was Built

Delivered on `deliver/025-dagger-merge-gate`.

- Extended `.githooks/pre-merge-commit` so merges into `master` / `main` run
  `dagger call check --source=.` after the existing verdict gate.
- Added explicit `HARNESS_KIT_NO_DAGGER=1` as the Dagger-only escape hatch.
  `HARNESS_KIT_NO_REVIEW=1` now bypasses review only; Dagger still runs.
- Made missing Dagger or unavailable Docker fail closed for repos with
  `dagger.json`, because the ticket's contract is "no merge without Dagger."
- Expanded `.githooks/test_pre_merge_commit.sh` to 17 deterministic cases with
  fake `dagger` and `docker`, covering fail/pass/bypass/non-master/no-config.
- Added Dagger lane `check-git-hooks` so hook behavior is part of
  `dagger call check --source=.`.
- Updated `/ship` because squash merges do not fire `pre-merge-commit`; the
  git-native landing path now runs `dagger call check --source=.` explicitly
  before `git merge --squash`.
- Regenerated docs and updated the root AGENTS gate count to 17.

## Verification

- Provider lanes:
  - `claude`: `ef2e98d5-64ec-4098-851d-f0c5dd75b12f`
  - `grok-build`: `a3826d58-a256-4996-a137-95048900c2d2`
- Acceptance artifact hash before closeout:
  `5e7bdc23fde9553e102e458264b8f516dba2b0fa83557bed49877c22f6a2c3e8`
- `bash .githooks/test_pre_merge_commit.sh`
- `shellcheck --severity=error .githooks/pre-merge-commit .githooks/test_pre_merge_commit.sh`
- `python3 -m py_compile ci/src/harness_kit_ci/main.py`
- `dagger call check-git-hooks --source=.`
- `python3 scripts/check-frontmatter.py`
- `python3 scripts/check-agent-roster.py`
- `bash scripts/check-docs-site.sh`
- `git diff --check`
- `dagger call check --source=.` -> 17 passed, 0 failed

## Notes

The backlog text predated the current `/ship` boundary and referred to
`/land`. The repo no longer carries a separate `/land` script; `/ship` is the
current landing owner, so the oracle was satisfied there instead of reviving a
second landing workflow.
