# Context Packet: Loop engineering Mode B bridge

Priority: P1
Status: ready
Estimate: M

## PRD Summary

- User: operators deciding whether an agent workflow should remain a manual
  Harness Kit session or become a scheduled/event-driven loop.
- Problem: loop-engineering advice is useful but easy to misplace. Harness Kit
  should not become a scheduler, while Bitterblossom should not run loops
  without gates, state, budgets, and maker/checker separation.
- Goal: add a loop-readiness reference and Mode B handoff shape that lets
  Harness Kit design loops and lets Bitterblossom execute them.
- Why now: the local loop-engineering premise file names the key failure modes:
  no objective gate, no state file, vague stop conditions, no budget cap, and
  one agent grading its own work.
- UX enabled: operators can classify loop candidates and produce a Mode B lane
  card without smuggling event orchestration into this repo.
- Deliverable type: cross-plane contract/reference.
- Success signal: a sample loop candidate is accepted/rejected by a deterministic
  readiness checklist and emits a Bitterblossom handoff card with hard stops.

## Product Requirements

- P0: Harness Kit stays Mode A: no scheduler, webhook receiver, unattended
  worker, or queue runner in this repo.
- P0: The loop-readiness checklist must require repetition, automated verifier,
  runnable environment, budget, state, and human review boundary.
- P0: Every Mode B handoff must name max iterations, no-progress detection, and
  token/dollar budget, matching `meta/CONTRACTS.md`.
- P0: The verifier that decides "done" must run fresh-context, not as the worker
  that produced the output.
- Non-goals: no Codex/Claude-specific `/loop` wrapper, no scheduler
  implementation, no Bitterblossom code changes from this ticket.

## Repo Anchors

- `meta/CONTRACTS.md` - Mode A/Mode B boundary and loop hard-stop precondition.
- `harnesses/shared/AGENTS.md` - verification system first, lane cards,
  parallel lanes, and closeout contracts.
- `skills/deliver/SKILL.md` - HTML plan and verification system first.
- `skills/ci/SKILL.md` - canonical local gate.
- `skills/code-review/SKILL.md` - fresh-context reviewer topology.
- `skills/sprites/templates/lane-card.md` - lane-card template referenced by
  shared contracts.

## Premise Evidence

The local `loop-engineering.md` file emphasizes that a loop earns its cost only
when the task repeats, verification is automated, budget can absorb waste, and
the agent can run the changed surface. It also names common failures: no
objective verifier, soft completion conditions, no hard stops, missing state
file, self-review bias, and unattended security exposure.

## Alternatives

| Option | What it buys | Where it fails | Verdict |
|---|---|---|---|
| Add loops directly to Harness Kit | Simple local control. | Violates Mode A; turns ad-hoc harness into event plane. | Reject. |
| Put everything in Bitterblossom | Keeps event code in Mode B. | Loses Harness Kit's shaping/skill/gate design role. | Reject. |
| Add shared readiness reference + handoff artifact | Preserves boundary and makes loops safer before execution. | Requires Bitterblossom to consume the handoff separately. | Choose. |
| Ignore loop engineering | Avoids complexity. | Misses a real operator workflow and leaves future loops under-specified. | Reject. |

## Design

Add `harnesses/shared/references/loop-readiness.md` with:

- 4-condition strategic test: recurring work, automated verifier, budget, senior
  tools/repro environment.
- 30-second tactical check: weekly cadence, gate, run capability, hard stop,
  human approval before irreversible action.
- Minimum viable loop shape: one automation, one skill, one state file, one
  gate.
- Hard-stop block: max iterations, no-progress detection, token/dollar budget.
- Handoff fields: owner repo, trigger, lane card, state file path, verifier
  command, review boundary, evidence path, halt behavior.
- Explicit routing: Harness Kit designs and validates the handoff; Bitterblossom
  implements schedules, webhooks, queueing, and unattended execution.

Update `meta/CONTRACTS.md` or a referenced file only if the new reference
clarifies rather than duplicates the existing Mode B contract.

## Oracle

- A loop-readiness reference exists and is linked from `skills/harness-engineering/SKILL.md`
  and `meta/CONTRACTS.md` or an adjacent contracts reference.
- A sample fixture classifies three loop candidates:
  - dependency bump loop: accepted when verifier/budget/state/human review are named;
  - architecture rewrite loop: rejected;
  - CI triage loop with no automated verifier: rejected.
- `cargo test --locked -p harness-kit-checks loop` or a focused Rust test added
  by delivery validates any parser/checker if one is implemented.
- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Premise Source

Premise Source: sha256:65ac6bc6312322a65a800680044b4e246190522540a9f1f06c19ba6c5b8137a6 loop-engineering.md

Related source: sha256:ba4b4b28887bd991ea21311c4f9c0c9b38a8d0d4c5f27535b941c62dd50b8663 HIT-LIST.md

## HTML Plan

HTML Plan: `.evidence/shape-hit-list/hit-list-shape-index.html#loops`

## Risks + Rollout

| Risk | Mitigation |
|---|---|
| Harness Kit becomes scheduler | Explicit non-goal and Mode B owner; no worker code here. |
| Loop readiness becomes paperwork | Use sample fixtures or checker; require concrete verifier/hard stops. |
| Over-rejecting useful loops | Checklist accepts narrow, machine-checkable recurring work. |
| Bitterblossom handoff drift | Keep handoff fields aligned with `meta/CONTRACTS.md` lane-card/receipt contracts. |

Rollback: remove reference/routing; Mode B contract remains as-is.
