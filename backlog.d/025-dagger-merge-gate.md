# Dagger as merge gate — replace GitHub Actions CI

Priority: high
Status: pending
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

- [ ] `git merge feat-foo` into master fails without passing Dagger check
- [ ] `/land` runs Dagger check as part of merge workflow
- [ ] No `.github/workflows/` files needed for CI enforcement

## Non-Goals

- Building a webhook server (keep it local for now)
- Replacing GitHub as git remote
