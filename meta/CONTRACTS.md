# Shared Contracts — Mode A / Mode B Boundary

Two modes of agent work, two planes, one disk surface.

**Mode A (this repo):** ad-hoc, operator-driven sessions. Harness Kit loads
judgment and context into a session a frontier model drives.

**Mode B (bitterblossom):** event-driven workflows — code review on
PR-ready, production error → diagnose/fix/postmortem, scheduled and outer
loops. CI-native or webhook-triggered, never run by the authoring agent
(the Cloudflare/Stripe pattern). Every Mode B flow must also be runnable ad
hoc from a terminal; the platform webhook is just one trigger.

Harness Kit defines these contracts; both planes read and write them.

## 1. Backlog

- Open work: `backlog.d/NNN-<slug>.md` (bare numeric IDs). Closed:
  `backlog.d/_done/`.
- Every active ticket has a Goal and an acceptance Oracle.
- Either plane may file tickets; only the operator (or an operator-approved
  flow) deletes them.

## 2. Commit trailers

Recognized backlog keys (`harness-kit-checks backlog trailer-keys`):

- `Closes-backlog: <id>` — closes the ticket (archival intent).
- `Ships-backlog: <id>` — synonym, closes.
- `Refs-backlog: <id>` — references without closing.

IDs are bare numerics (`029`, not `BACKLOG-029`). Inject via
`git interpret-trailers --trailer`, never hand-formatted. Archive the
ticket file on the shipping ref BEFORE the squash-merge so closure rides
the merge commit. GitHub squash bodies drop commit trailers — pass the
trailer block explicitly.

Agent provenance trailers are advisory, not backlog-closing keys:

- `Agent: <name-or-role>`
- `Agent-Surface: <surface>`
- `Agent-Runner: <runtime>`
- `Agent-Model: <provider/model>`
- `Agent-Reasoning: <level>`
- `Agent-Task: <ticket-board-pr-issue>`
- `Agent-Context: <session-or-pane-or-receipt>`

Use them when an agent materially authors code, durable docs, repo
configuration, PR text, or a shippable artifact. Do not fake unknown values.

## 3. Lane cards

The unit of delegated work, identical for local subagents, roster
providers, sprites, and Mode B workers: end state, success criteria,
verification affordances, boundaries, output shape, receipt expectation.
Template: `skills/sprites/templates/lane-card.md`. The card is the entire
context the remote agent gets; the oracle field is load-bearing.

## 4. Receipts

- Delegation receipts: append-only JSONL at
  `.harness-kit/traces/delegations.jsonl` (repo-local) — written by
  `harness-kit-checks record-delegation` / `dispatch-agent`.
- Sprite-lane receipts: `~/.harness-kit/receipts/sprite-lane/<lane-id>.json`.
- Mode B runs emit the same receipt shapes; provider output is evidence,
  not authority.

## 5. Evidence

Artifacts that prove behavior (screenshots, transcripts, request replays)
live under `.evidence/<branch>/<date>/` when committed, and are linked
directly from briefs/PRs — never described without a path.

## 6. Loop guardrails (Mode B precondition)

A loop that can't halt is a billing incident, not automation. Before any
Mode B loop runs unattended, it names all three hard stops:

For the Mode A decision checklist that decides whether an idea is ready to
leave ad-hoc work and become an unattended loop, see
`harnesses/shared/references/loop-readiness.md`.

- **Max iterations** — a ceiling, not a vibe.
- **No-progress detection** — same failure twice, or no state change
  between ticks, halts and surfaces instead of retrying.
- **Token/dollar budget** — per run and per day.

A halted loop files what it found (ticket, receipt, alert) and stops. The
verifier that decides "done" runs in a fresh context, never the worker
that produced the result.

## Mode B roadmap (owned by bitterblossom)

First workload: orchestrated code review (the absorbed Cerberus mission) —
coordinator + specialized reviewers, risk-tiered compute, tiered model
stack, shared context files, JSONL streaming, incremental re-review.
Later: monitor/deploy watchers and the unattended outer loop (the retired
/flywheel). When the review workload is live, Harness Kit's `/code-review`
stays as the ad-hoc dispatch form; the event form belongs to the plane.
