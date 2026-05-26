---
name: office-hours
description: |
  YC-partner-style interrogation of a raw idea. Six forcing questions before
  you shape anything: demand reality, status quo, desperate specificity,
  narrowest wedge, observation & surprise, future-fit. Operationalizes the
  AGENTS.md "Diverge Before You Converge" doctrine at the ideation stage —
  problem-diamond, pre-/shape.
  Use when: user arrives with a rough idea, backlog item is fuzzy, you can't
  already write a one-sentence goal with a testable outcome, or the phrase
  "what should we build" would be useful.
  Trigger: /office-hours, /oh, /interrogate.
argument-hint: "[raw-idea-or-concept]"
---

# /office-hours

Raw idea → sharpened problem statement. Six forcing questions that catch
premature scoping, proxy goals, and solution-shaped problems before they
reach `/shape`.

## When to run

- User has a rough concept ("I want a thing that does X")
- Backlog item is fuzzy — no clear user outcome
- You can't write a one-sentence goal with a testable oracle
- The plan is "do the obvious thing" — interrogation catches non-obvious
  failure modes
- Before `/shape` on an unshaped concept

Skip for small, well-understood work. Office hours is for moments where the
idea is not yet an idea.

## Delegation Floor

When `.spellbook/agents.yaml` exists, `/office-hours` uses two or more roster
members for substantive premise interrogation: one lane challenges demand and
status quo, another challenges wedge and future-fit. The lead asks the user
the final questions and synthesizes the sharpened problem. Direct lead-only
office-hours work is limited to mechanical note formatting, explicit
user-forbidden delegation, or an explicit waiver when fewer than two roster
members are available.

## The Six Forcing Questions

Ask each, in order. Record the answers.

### 1. Demand Reality
Who specifically wants this, and how do you know? Not "users" — *which*
users. What did they actually say? If you can't name three specific people
or instances, the demand is hypothetical.

### 2. Status Quo
What do they do today? If the answer is "they live with it," check whether
they actually feel pain. If the answer is "they use tool X," your bar is
"10× better than X," not "exists."

### 3. Desperate Specificity
Describe the worst outcome of not building this. Concrete: what breaks, who
suffers, what do they do next? If the answer is "they keep going, fine,"
the need isn't desperate — scope accordingly.

### 4. Narrowest Wedge
What's the smallest, ugliest version that would make ONE named user happy?
If you can't name it, you're designing for a crowd you haven't met.

### 5. Observation & Surprise
What would we learn by shipping this that we don't already know? If the
outcome is predictable, the value is bounded — ship the prediction cheaply
if at all.

### 6. Future-Fit
Three years out, does this survive? If the use case evaporates when (a)
LLMs improve, (b) underlying tools change, (c) the team reorgs, it's a
tactical patch — name it as such.

## Output

```
## Office Hours: <idea>

### Demand Reality
[who, when, what they said — specific]

### Status Quo
[what they do today; bar to clear]

### Desperate Specificity
[concrete failure mode of not building]

### Narrowest Wedge
[smallest ship that makes one named user happy]

### Observation & Surprise
[what we'd learn; confidence level]

### Future-Fit
[three-year survival: yes / no / tactical-patch]

### Sharpened Problem Statement
<one sentence ready for /shape, OR "needs more demand reality before shape">
```

## Gotchas

- **Answering your own questions.** "Probably" and "I think" are speculation,
  not interrogation. The user supplies demand-reality evidence; the skill
  pressure-tests it.
- **Accepting "users want it."** *Users* is a cop-out. Which users, when,
  what they said. Named people or logged instances beat persona sketches.
- **Skipping to the wedge.** Narrowest wedge without demand reality produces
  a clean ship of something nobody wants.
- **Future-fit as disqualifier.** "Will be obsolete in 3 years" is not
  always disqualifying — tactical patches are fine if *named* as such. The
  gotcha is shipping tactical patches labeled strategic.
- **Post-shape office hours.** This is PRE-shape. If `/shape` already ran,
  use `/ceo-review` — office hours is the wrong tool for an already-shaped
  plan.

## Relationship to other skills

- **`/groom`** surfaces raw themes from a codebase. Pipe the most promising
  theme into `/office-hours` before shaping.
- **`/shape`** accepts a sharpened problem statement as input. If office
  hours surfaces absent demand reality, do not shape — the work isn't ready.
- **`/ceo-review`** is the post-shape counterpart. Office hours sharpens
  the *problem*; CEO review challenges the *plan*.
- **Pattern provenance:** This is the spellbook-native version of the
  demand-reality interview pattern, self-contained and integrated with our
  workflow.
