Role: fresh-context verifier / release-risk critic.
Objective: Re-check the previous blocker: lane harness manifests must not smuggle an off-provider model into dispatch.

Scope:
- Read only. Do not edit files.
- Inspect the current diff in crates/harness-kit-checks/src/lane_harness.rs and crates/harness-kit-checks/src/agent_roster.rs.
- Confirm whether a manifest with provider_target codex and model_override claude-opus-4-8 now fails as projection_failed before provider dispatch.
- Confirm whether allowed values remain simple: provider model, model_variants key, or model_variants value.

Output shape, <=30 lines:
1. Verdict: pass or block.
2. Blocking gaps, if any, with exact file paths and line references.
3. Non-blocking risks.
4. One sentence on whether this remains a thin harness primitive.
