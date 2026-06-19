# Context Packet: Evaluate OpenCode as the code-review runner substrate

Priority: P1
Status: shaped
Estimate: M

## Goal

Prove whether Harness Kit should add an OpenCode session/service runner path
for code-review and review-eval lanes, or explicitly stay with the current thin
CLI dispatch surfaces.

## Premise

A 2026-06-19 coding-agent substrate report recommends OpenCode as the strongest
open per-job kernel for an owned PR-review system because it is
server/session-shaped rather than terminal-first. That maps to Harness Kit's
review/eval needs, but adopting it without evidence would recreate the
historical semantic-wrapper failure mode.

The outcome is not "use OpenCode because the report says so." The outcome is a
small bake-off that tells us whether OpenCode's session/event surface improves
review lane observability, context hygiene, retries, and structured evidence
over the existing `dispatch-agent` CLI path.

## Non-Goals

- Do not build a production review control plane in Harness Kit.
- Do not add semantic provider ranking, automatic fallback trees, or a workflow
  engine around provider CLIs.
- Do not move Mode B event orchestration into Harness Kit; Bitterblossom and
  product repos own event-triggered loops.
- Do not expose GitHub write credentials, model-provider keys, or user secrets
  to untrusted repository execution.
- Do not claim review-quality superiority from one model or one fixture.

## Repo Anchors

- `skills/code-review/SKILL.md` - current dispatch-shaped review contract.
- `skills/roster/references/model-provider-harness-index.md` - factual
  provider/harness index.
- `skills/harness-engineering/references/open-model-roster.md` - role-fit
  policy for Pi, Goose, and OpenCode.
- `.harness-kit/agents.yaml` - current provider roster.
- `crates/harness-kit-checks/src/agent_roster.rs` and
  `crates/harness-kit-checks/src/lane_harness.rs` - dispatch and projection
  implementation.
- `backlog.d/112-harness-eval-bench.md` - matched outcome eval protocol.

## Design Options

| Option | What it buys | Where it fails | Verdict |
|---|---|---|---|
| Keep OpenCode as CLI-only roster lane | No new surface; already smoked with Kimi K2.7 Code. | Does not test the report's strongest claim: programmatic sessions and event capture. | Baseline. |
| Add a narrow experimental OpenCode session runner | Tests session/event value without changing the roster contract. | Requires Rust/TS boundary choice and fixture discipline. | Preferred. |
| Adopt OpenCode as default review backend immediately | Fast policy shift. | Unevidenced, over-broad, and unsafe without queue/sandbox/eval boundaries. | Reject. |
| Use Goose for the review runner | Strong MCP workflow story. | The report says Goose wins when side effects across systems dominate, not for review-first kernels. | Defer to MCP lanes. |

## Design

Run a bake-off, not a migration.

1. Define a minimal `AgentRunner`-shaped experiment for review lanes:
   `create session`, `send task`, `stream/export events`, `cancel`, and
   `write receipt`.
2. Use OpenCode first because the report's differentiator is its session/API
   shape. Keep the adapter experimental and behind an explicit flag or fixture.
3. Compare against the current `dispatch-agent --provider-target opencode`
   CLI path on the same review fixture from `backlog.d/112-*`.
4. Grade on operational evidence, not vibes:
   - structured event completeness;
   - failure classification;
   - prompt/context boundary visibility;
   - receipt quality;
   - ease of replay;
   - no secret exposure;
   - no extra global config mutation.
5. Only after the pilot, decide one of:
   - keep CLI-only;
   - add an experimental OpenCode runner;
   - adapt the lane-harness projection first;
   - graduate the problem to a product repo / Mode B control plane.

## Oracle

- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.
- A pilot report under `.evidence/harness-evals/` compares OpenCode CLI dispatch
  with an OpenCode session/service path or documents why the service path is not
  locally runnable.
- The pilot uses the same diff/oracle for both conditions and records sanitized
  receipts.
- The report answers whether OpenCode improves event capture, retry/debug
  evidence, or context hygiene enough to justify new tooling.
- A fresh critic reviews the pilot artifacts and returns no blocking
  methodological flaw.

## Notes

- This is the Harness Kit slice of the report. Olympus/Argus may later consume
  the outcome, but Olympus remains responsible for PR webhooks, GitHub App
  posting, durable run state, Sprite isolation, and Habitat writeback.
- The report's security lesson is non-negotiable: no write token or model key
  goes into a sandbox that can run repository-controlled code.
