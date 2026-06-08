# Delivery Brief: 101 Focused Lane Harness Projection

## Goal
Deliver backlog 101: a thin Rust primitive for projecting a bespoke, per-lane
harness so dispatched provider lanes do not inherit the full global skill
catalog.

## Behavior Delivered
- Added `lane_harness.v1` manifest validation and materialization.
- Projected selected local skills into provider discovery roots under ignored
  runtime path `.harness-kit/tmp/lane-harness/<id>/`.
- Integrated optional `dispatch-agent --lane-harness` support with child-only
  environment overlays for Codex, Claude, Pi, and Antigravity/Gemini config
  roots.
- Rejected provider/model mismatches before dispatch. Manifest model overrides
  must match the selected provider roster `model`, a `model_variants` key, or a
  `model_variants` value.
- Extended delegation receipts and summaries with lane harness refs, manifest
  sha256, projection status, failure kind, and optional output sentinel checks.
- Added fake-provider and manifest tests proving selected skills appear and
  excluded skills such as `shape` and `groom` do not leak into projected roots.

## Key Files
- `crates/harness-kit-checks/src/lane_harness.rs`
- `crates/harness-kit-checks/src/agent_roster.rs`
- `crates/harness-kit-checks/src/summarize_delegations.rs`
- `crates/harness-kit-checks/src/check_agent_roster.rs`
- `.harness-kit/examples/lane-harness.yaml`
- `.harness-kit/examples/delegation-receipt.jsonl`
- `skills/ship/SKILL.md` (gate hygiene: compressed to the 500-line skill cap
  without changing shipping semantics)

## Design Guardrail
The helper validates, materializes, launches one requested provider, records one
receipt, and returns. It does not select providers, score outputs, retry
semantically, generate skills, or mutate the global harness install.
