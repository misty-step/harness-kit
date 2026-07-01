# Fixture 01 — telemetry --top <N>

Add a `--top <N>` flag to the `telemetry` subcommand of harness-kit-checks that
limits the report to the N most-used skills (by invocation count).

Repo: harness-kit @ 3bf0b46
Anchors: crates/harness-kit-checks/src/main.rs (dispatch + usage ~L938),
         crates/harness-kit-checks/src/skill_invocation_analytics.rs
Forbidden edits: any crates/** (spec-only; deliverable is the packet)

Arms:
- A = follow skills/shape/SKILL.md to produce the packet
- B = raw "flesh out this spec into something buildable", no skill
Family: both Claude (shared-family smoke waiver — proves loop fires, not margin)
