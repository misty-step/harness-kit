---
name: landmark
description: |
  Use when an agent needs release intelligence from Landmark: version analysis,
  changelog synthesis, release notes, release-kit planning, GitHub Action
  adoption, fleet rollout, classification, or release artifact evidence.
  Trigger phrases: "Landmark", "release intelligence", "changelog",
  "release notes", "version bump", "release kit".
argument-hint: "[describe|run|setup|fleet|release-kit|classify]"
---

# Landmark

Landmark owns release intelligence. Use it before hand-writing release truth
from git memory or ad-hoc commit summaries.

Read `VISION.md` before changing release boundaries, adoption modes,
agent-native contracts, or release-kit producer responsibilities.

## Route

| Need | Surface |
|---|---|
| Describe the current release state | `landmark describe --json` |
| Dry-run release analysis | `landmark run --provider local --dry-run` |
| Install in a repo | `landmark setup` |
| Fleet adoption | `fleet scan`, `fleet plan`, `fleet open-prs` |
| GitHub Action use | `misty-step/landmark@v0` |
| Local development | `cargo run --locked -p landmark -- ...` |
| Full repo gate | `bin/gate` |

Landmark is pre-stable (0.x): repos below 1.0.0 stay on the `@v0` major-pin
line until Landmark itself promotes to 1.0.0. See the landmark README's
Versioning Philosophy section. Pass the Action's `github-token` input as
`${{ github.token }}` — the default `GITHUB_TOKEN` — under a job (or
workflow) `permissions` block granting `contents`/`issues`/`pull-requests`
write; a PAT-backed secret is only needed when downstream automation must
trigger further workflow runs.

## Operating Rules

- Start with live git and Landmark's CLI/action surfaces, not remembered
  release state.
- Keep release analysis, synthesis, artifact planning, feed generation,
  evidence, approval state, and provider policy in the Rust CLI.
- GitHub is an adapter. Non-GitHub callers must be able to use CLI commands,
  JSON artifacts, local git state, and manifest files.
- Treat user-facing release notes as a model-native product surface with
  evidence and replay paths, not as static prose.
- Release-kit artifacts are the planning/evidence boundary for richer final
  output. Do not embed bespoke media production in the core runtime.

## MCP

Landmark does not currently expose an MCP server. Use the CLI/action contract
as the agent surface until a real MCP server earns itself.

## Verification

In the Landmark repo:

```sh
bin/gate
```

For release-orchestration changes, also use the relevant replay path, especially
`bin/replay-action` when touching action behavior.
