---
name: shape
description: |
  Shape a raw idea into something buildable. Product + technical exploration.
  Spec, design, critique, plan. Output is a context packet.
  Use when: "shape this", "write a spec", "design this feature",
  "plan this", "spec out", "context packet", "technical design".
  Trigger: /shape, /spec, /plan, /cp.
argument-hint: "[idea|issue|backlog-item] [--spec-only] [--design-only]"
---

# /shape

Turn a raw idea into a **context packet** — the unit of specification that
`/deliver` consumes. Spec before code, always.

## Contract

A packet is ready when all of these are true. How you get there is your
judgment; size the effort to the stakes — a one-file fix needs a paragraph,
an architecture choice needs the full treatment.

- **Premise challenged.** The request is a first-draft framing, not a locked
  problem. Name the underlying user outcome before designing; the best path
  may not be the feature asked for. Shaping the wrong problem well is the
  failure this skill exists to prevent.
- **Grounded in live repo evidence.** You read the load-bearing source files,
  tests, and ADRs yourself. Subagent summaries add coverage; they do not
  replace direct reads of what the builder must understand.
- **Alternatives genuinely explored.** Real alternatives fail differently —
  include the boring/manual path and one that inverts a load-bearing
  assumption. Same idea in three outfits is one option. Kill the losers on
  the record and **recommend one**; a menu is not a shape.
- **Scope is fenced.** Goal (outcome, not mechanism), explicit non-goals,
  and invariants that must survive the change.
- **Oracle is executable.** "It should work" is not an oracle; "these
  commands pass, this route returns X" is. If you can't write the oracle,
  the goal isn't clear yet — go back. See `references/executable-oracles.md`.
- **Deliverable visible up front.** Code, research, docs, or decision — a
  reader should not have to reach the implementation sequence to find out.
- **Premise source named.** The packet cites the artifact that explains why
  this shape exists (`Premise Source: sha256:<digest> <path-or-url>`) or
  carries an explicit waiver with residual risk. Voice/raw-transcript
  premises take the metadata block from
  `references/voice-transcript-metadata.md`; never store raw audio paths.
  This is grader-enforced (see Verification).

Lock product direction with the user before technical design when the
direction is genuinely contestable — one question at a time, with your
recommended answer and what breaks if it's wrong. Don't manufacture
questions for shapes the evidence already locks.

## Packet Skeleton

Sections carry weight or they don't appear. For substantial work, follow the
PRD shape in `references/prd-ticket-quality.md`; for CLI surfaces, include
the block from `references/cli-design.md`.

```markdown
# Context Packet: <title>

## Goal            — one sentence, outcome not mechanism
## Non-Goals       — scope that stays out, even if tempting
## Constraints     — invariants that must remain true
## Repo Anchors    — the 3–10 files whose patterns must be followed
## Alternatives    — what was considered, how each fails, verdicts
## Design          — chosen shape, surfaces touched, data/control flow,
                     rejected alternatives and why, ADR decision if any
## Oracle          — executable definition of done
## Premise Source  — sha256 + artifact, or explicit waiver
## Risks + Rollout — how it fails, how to undo it
```

When the oracle depends on an acceptance artifact (fixture, golden file,
contract, screenshot), pin it: `sha256:<digest> <path>`. If implementation
intentionally changes that artifact, the handoff carries a contract-change
acknowledgment. High-risk work (money/auth/migrations, expensive-to-detect
regressions) earns formal examples and a test-strength budget — note it in
the packet for `/deliver` and `/qa` rather than inflating the packet itself.

## Delegation Judgment

Delegate on judgment per the shared Roster contract: native subagents by
default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: one lane to map repo constraints, one for prior art or
premise challenge; fresh-context critique of the draft packet when the
design is contestable.

## Critique

Your own design read is not a review. When the design is contestable, hand
the draft packet — artifact only, not your reasoning trail — to adversarial
fresh-context critique, preferably a different model family, and ask for the
production failure that would embarrass us. Lens prompts live in
`references/critique-personas.md`. Skip for trivial shapes.

## Gotchas

- **Over-speccing HOW.** Specify WHAT and WHY; let the builder own the how.
  Detailed pseudocode cascades its own bugs into implementation.
- **Speccing after building.** That's documentation, not specification.
- **Ready-but-vague.** A packet is not ready while a load-bearing choice
  still says "preferably" or "decide during implementation".
- **50 repo anchors.** If everything is an anchor, nothing is.
- **Editing live shape docs without ripple check.** Files marked
  `shaping: true` feed other streams; trace consequences after editing.

## Verification

Premise-source discipline is enforced by the Rust grader:

```sh
cargo run --locked -p harness-kit-checks -- premise-source validate <packet>
cargo run --locked -p harness-kit-checks -- premise-source self-test
```

Optional reviewer-facing HTML render for visual/doc-heavy handoffs
(`shape-render`), critiqued via `/design`; the Markdown packet stays
authoritative.
