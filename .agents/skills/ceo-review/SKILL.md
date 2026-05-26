---
name: ceo-review
description: |
  Dialectical premise-and-alternatives audit for a plan, spec, or context packet.
  Four moves: premise challenge (is this the right problem?), mandatory
  structurally-distinct alternatives, cross-model outside voice, user-ratified
  convergence. Operationalizes the AGENTS.md "Diverge Before You Converge"
  doctrine at the plan-review stage.
  Use when: about to commit to a plan/spec/design, reviewing a ticket before
  shape, when "is this the right problem" would be useful, or any time a
  proposal smells like a symptom fix instead of a root-cause fix.
  Trigger: /ceo-review, /challenge, /premise-check.
argument-hint: "[plan|spec|context-packet|ticket]"
---

# /ceo-review

Dialectical audit. Subject the plan to premise-challenge, mandatory
alternatives, and a cross-model outside voice before committing.

## When to run

- Before committing to a context packet or spec
- When a backlog ticket's framing smells like a symptom, not a root cause
- After `/shape` produces a design and before `/deliver` starts building
- Any time someone asks "is this the right thing to build?"

Skip trivial plans. CEO-review catches expensive misdirection, not typos.

## Delegation Floor

When `.spellbook/agents.yaml` exists, `/ceo-review` uses two or more roster
members before convergence: one lane attacks the premise, another develops
structurally distinct alternatives or an outside-model critique. The lead owns
ratification questions, synthesis, and receipts. Direct lead-only review is
limited to mechanical formatting, explicit user-forbidden delegation, or an
explicit waiver when fewer than two roster members are available.

## The Four Moves

### 1. Premise Challenge

Before touching the stated solution:

- **Five-whys the goal.** If the plan says "build X," keep asking why until
  you've named the underlying user outcome.
- **Proxy check.** Is the stated solution the most direct path to the
  outcome, or something easier to scope?
- **"Do nothing" test.** What happens if this never ships? If the answer is
  "it's fine," the plan has a priority problem, not an execution problem.
- **Provenance.** Who asked for this, what did they actually say? Tickets
  drift from original intent through paraphrase.

If the premise is wrong, stop. Reframe and re-shape — do not review forward.

### 2. Mandatory Alternatives

Produce ≥2 structurally distinct approaches. Not one idea in costumes.

- **Minimal viable:** smallest thing that tests the outcome
- **Ideal architecture:** what you'd build if effort were free
- **Assumption-inverter:** pick one load-bearing assumption and flip it
  (sync → async, per-user → per-team, pull → push, etc.)

For each: one-paragraph sketch, its load-bearing assumption, and one way it
would fail *differently* from the others. If all three fail the same way,
they're the same plan.

### 3. Outside Voice

Consult a different *foundation*, not a different persona.

- Prefer: `codex exec "..."`, `gemini "..."`, `thinktank ...`, or `/research`
- Acceptable fallback: a fresh-context Claude subagent with an adversarial prompt
- Feed it: the premise, the plan, and the alternatives
- Ask: "Which framing is load-bearing wrong? Which alternative would you
  pick and why? What are we missing?"

The outside voice is informational, not authoritative. It informs your
judgment; it does not replace it.

### 4. Ratify

Present to the user:

```
## CEO Review: <title>

### Premise Verdict
[stands / reframed — if reframed, state the new premise]

### Alternatives
1. <minimal> — load-bearing assumption: ... — fails by: ...
2. <ideal>   — load-bearing assumption: ... — fails by: ...
3. <inverted> — load-bearing assumption: ... — fails by: ...

### Outside Voice (<source>)
<what they saw that we didn't>

### Recommendation
<one concrete next step — argue for it>
```

User decides: proceed with recommendation, pick a different alternative,
reframe the premise, or abandon. Silence is not consent.

## Gotchas

- **Premise-challenge theater.** Asking "is this the right problem?" and
  immediately answering "yes" is not a challenge. If the premise stands,
  state *what specifically survived* the audit — otherwise you didn't do it.
- **Same-foundation alternatives.** Three Claude-generated alternatives from
  one prompt are not three alternatives. The outside voice must have a
  different model underneath; persona diversity is theater.
- **Authoritative outside voice.** "Codex said X, so X." No. Cross-model
  consensus is signal; disagreement is signal. Your judgment arbitrates.
- **Skipping on "small" plans.** The most expensive failures seemed small
  enough to skip review. If effort-to-ship > ~one agent-day, review.
- **Ratification by absence.** Presenting the review and moving on without
  an explicit "proceed" is silent absorption, not ratification.

## Relationship to other skills

- **`/shape`** runs the philosophy bench (ousterhout/carmack/grug) at Phase 3
  for M+ effort designs — that's persona diversity. `/ceo-review` adds the
  foundation diversity (outside voice) and premise challenge. Use both.
- **`/office-hours`** is the pre-shape dialectic: raw idea → sharpened
  problem statement. `/ceo-review` is the post-shape dialectic: shaped plan
  → ratified plan.
- **`/groom`** surfaces themes; its synthesis protocol includes a premise
  challenge. If `/groom` already reframed, `/ceo-review` picks up from there.
- **Pattern provenance:** This is the spellbook-native version of the
  dialectical plan-review pattern, self-contained and integrated with our
  workflow.
