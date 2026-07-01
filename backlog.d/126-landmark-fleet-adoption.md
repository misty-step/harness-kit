# Landmark fleet adoption

Priority: P1
Status: pending
Estimate: M

## Goal

Make Landmark release intelligence the default release layer for active Factory
repos, with each repo either wired through a workflow/manifest or explicitly
classified as skipped/non-release.

## Context

The Factory lane set raised Landmark as the shared release-intelligence
primitive. Harness Kit now carries its own `.landmark.yml` and
`.github/workflows/landmark-release.yml`; the broader fleet still needs a
deliberate adoption pass instead of ad hoc per-repo YAML.

The Landmark lane playbook is the source of truth for workflow shape:

- read the target repo's release surface and existing workflows first
- prefer attaching Landmark to the existing release owner when one exists
- use a fallback `landmark-release.yml` only when no release owner exists
- run local preview or dry-run evidence before opening branches
- require `GH_RELEASE_TOKEN` and `OPENROUTER_API_KEY` only for GitHub release
  modes

## Active Factory Coverage

- [x] harness-kit
- [ ] powder
- [ ] exocortex
- [ ] threshold
- [ ] bitterblossom
- [ ] crucible
- [ ] cerberus
- [ ] canary

## Oracle

- [ ] Each active Factory repo has a `.landmark.yml` manifest or an explicit
      skip record with reason.
- [ ] Each releasable repo has Landmark wired through an existing release
      workflow or a dedicated `.github/workflows/landmark-release.yml`.
- [ ] Each GitHub workflow names required secrets without printing values.
- [ ] At least one repo is validated through Landmark local preview or the
      closest repo-safe dry run before fleet rollout continues.
- [ ] Harness Kit's own gate passes:
      `cargo run --locked -p harness-kit-checks -- check --repo .`.

## Notes

Do not batch-open fleet PRs from Harness Kit. Use Landmark's fleet tooling or a
short branch per repo, preserve concurrent lane work, and merge one downstream
adoption before widening.
