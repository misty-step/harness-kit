# Lane Card: Runtime and scout implementation critique

Role: implementation investigator / critic.

Objective: Review the planned delivery for backlog `105`, `106`, and `107` and identify concrete implementation risks before code is written.

Scope:
- Read `backlog.d/105-exa-agent-research-lane.md`, `backlog.d/106-ponytail-simplicity-skill.md`, `backlog.d/107-agent-skill-market-scout.md`.
- Inspect `skills/research/*`, `skills/research/__tests__/*`, `registry.yaml`, `crates/harness-kit-checks/src/main.rs`, and `crates/harness-kit-checks/src/external_sync.rs`.
- Do not edit files.

Output shape:
- `BLOCKING:` yes/no.
- Up to 8 bullets of implementation risks or test cases the lead should not miss.
- Up to 5 suggested file/function placements.
- Keep under 1200 words.

Success criteria:
- Focus on cost/privacy routing for Exa Agent, deterministic scout tests, and registry/sync fit for Ponytail.
- Reject scope creep such as auto registry edits or live paid calls in CI.
