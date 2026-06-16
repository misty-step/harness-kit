# Context Packet: HarnessX evolution evaluation

Priority: P3
Status: ready
Estimate: L

## PRD Summary

- User: Harness Kit maintainers considering trace-driven harness improvement.
- Problem: HarnessX-style self-evolving harnesses are promising, but blindly
  letting a loop edit Harness Kit would risk reward hacking, forgetting, and
  self-modifying infrastructure without held-out acceptance.
- Goal: create a research/evaluation packet for a typed, review-only harness
  evolution experiment, likely owned by Bitterblossom, using Harness Kit traces
  as input and producing human-reviewed diffs as output.
- Why now: the HarnessX paper is recent and directly about the harness layer;
  the local anatomy file reinforces that harness changes can move performance
  materially without changing model weights.
- UX enabled: maintainers get a measured "is this worth building?" answer
  before adding self-improvement machinery to core Harness Kit.
- Deliverable type: research/eval design and Mode B handoff.
- Success signal: a held-out benchmark shows whether trace-proposed harness
  edits improve a weak-model lane without degrading current strong-model flows.

## Product Requirements

- P0: No self-merging harness edits.
- P0: No source edit is accepted without human review, full Harness Kit gate,
  and held-out task evidence.
- P0: First experiment must be typed and narrow: one primitive family such as
  skill trigger descriptions, reference routing, or reviewer prompts.
- P0: Candidate generation belongs to a detached Mode B/eval lane, not the
  lead's live session.
- P0: Measurements must include negative controls and rollback.
- Non-goals: no full HarnessX implementation, no RL training loop, no model
  fine-tuning, no automatic AGENTS rewrite, no production dependency on
  unreleased code.

## Repo Anchors

- `meta/CONTRACTS.md` - Mode B boundary, receipts, lane cards, and hard-stop
  loop requirements.
- `skills/harness-engineering/SKILL.md` - primitive test, delete before adding,
  and model-release re-audit gotcha.
- `crates/harness-kit-checks/src/skill_invocation_analytics.rs` - existing
  skill usage and work/delegation report surface.
- `crates/harness-kit-checks/src/summarize_delegations.rs` - receipt summary
  and provider evidence.
- `registry.yaml` and `skills/` - possible typed primitive families.
- `cargo run --locked -p harness-kit-checks -- check --repo .` - acceptance
  gate for any proposed source diff.

## External Evidence

- arXiv:2606.14249, submitted June 12, 2026, frames runtime harnesses as
  prompts, tools, memory, and control flow, and reports average benchmark gains
  from composable/evolvable harnesses. It also says the complete codebase will
  be open-sourced in a future release, so there is no current implementation to
  vendor as of shaping.
- The local anatomy article argues the harness is a product layer: context,
  tools, state, permissions, verification, and memory decide whether an agent
  works, and stronger models can justify deleting scaffolding rather than
  adding it.

## Alternatives

| Option | What it buys | Where it fails | Verdict |
|---|---|---|---|
| Ignore HarnessX | Avoids speculative work. | Misses a direct research signal in Harness Kit's domain. | Reject. |
| Implement HarnessX in Harness Kit core | Ambitious and on-theme. | Too much surface, no released code, violates thin-harness and self-edit safety. | Reject. |
| Wait for upstream code | Avoids reimplementation. | Leaves no prepared evaluation or fit criteria. | Reject as only path. |
| Shape a Mode B evaluation | Lets us test the idea with receipts, held-out tasks, and human review. | Bigger than a docs-only task. | Choose. |
| Apply only as doctrine | Cheap. | Does not answer whether trace-driven edits improve our harness. | Defer, not enough. |

## Design

Create an eval design that treats Harness Kit primitives as typed candidates:

- Candidate families: skill frontmatter descriptions, reference routing tables,
  reviewer prompt templates, or gate fixture thresholds.
- Input traces: sanitized skill invocation analytics, delegation summaries, and
  failed-review receipts.
- Candidate generator: Mode B lane proposes a patch plus rationale.
- Critic: fresh-context lane sees only patch + held-out oracle.
- Gate: Harness Kit full check plus task-specific held-out suite.
- Output: a reviewed PR/diff and eval report; no automatic merge.

The first eval should use a weak/open-model lane because the HarnessX premise
claims the weakest baseline benefits most. Strong-model impact is tracked as a
regression guard, not expected lift.

## Oracle

- A research/eval artifact under `.evidence/harnessx-eval/` defines:
  candidate primitive family, trace schema, held-out task set, baseline model,
  candidate model, scorer, safety gate, and rollback.
- If no code is added, the eval artifact explicitly waives the focused Rust
  test. If code is added, `cargo test --locked -p harness-kit-checks harnessx`
  or an equivalent focused test covers report parsing and no-self-merge
  constraints.
- A dry-run candidate patch is generated but not applied; the report includes
  predicted benefit, held-out task scores, and fresh critic verdict.
- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Premise Source

Premise Source: sha256:3170c020d2d2fd805387ad1ec5ae6a69973449146cc1af40020c9c15a4a81605 anatomy-of-an-agent-harness.md

Related source: sha256:ba4b4b28887bd991ea21311c4f9c0c9b38a8d0d4c5f27535b941c62dd50b8663 HIT-LIST.md

External source checked on 2026-06-16:

- https://arxiv.org/abs/2606.14249

## HTML Plan

HTML Plan: `.evidence/shape-hit-list/hit-list-shape-index.html#harnessx`

## Risks + Rollout

| Risk | Mitigation |
|---|---|
| Reward hacking | Held-out tasks, negative controls, fresh critic, and no self-merge. |
| Catastrophic forgetting | Compare against current strong-model flows and rollback source diff. |
| Overbuilding unreleased research | Eval first; no core Harness Kit primitive until evidence beats baseline. |
| Privacy leakage in traces | Use sanitized receipts/analytics only; no raw private transcripts. |

Rollback: delete eval artifacts; no runtime behavior changes.
