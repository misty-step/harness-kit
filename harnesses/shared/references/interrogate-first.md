# Interrogate-First Lens

Use before shaping a contestable idea or framing a strategic groom. The posture
is an interview, not a questionnaire: walk the operator down the decision tree
until the hidden choices are visible and resolved. `/shape`'s own grill step
(`skills/shape/SKILL.md`) loads this reference and is the primary consumer of
the stance.

1. **One question at a time.** Never batch. Each answer reshapes the next
   question; a wall of questions is a form, and forms get skimmed.
2. **Resolve the tree, branch by branch.** Surface the decisions that depend on
   each other and settle them in dependency order. A load-bearing choice left
   implicit is a decision deferred to the worst possible moment.
3. **Recommend an answer.** For every question give your best answer and what
   breaks if it's wrong. An interrogation that only extracts is lazy; the point
   is to move toward a resolved design, not to quiz the operator.
4. **Explore before you ask.** If the codebase, vision, or a command can answer
   it, go read it — don't spend the operator's attention on what you can resolve
   yourself.
5. **Relentless until shared understanding.** Stop when the design is genuinely
   pinned, not when the operator sounds tired. "Decide during implementation" on
   a load-bearing choice means the interview isn't done.

## Which Move for Which Unknown

The interview handles decisions the operator knows are open. Not every gap is
that kind; route by the type of unknown:

- **Resolvable** — the repo, vision, or a command can answer it. Read it
  (rule 4). Never interview for facts.
- **Known unknown** — a decision the operator knows is open. Interview: the
  body of this file.
- **Unknown known** — "I'll know it when I see it" (design, copy, feel).
  Don't extract prose criteria that don't exist yet; route to disposable
  prototype variations and let the operator react before the spec locks.
- **Unknown unknown** — new domain; the operator doesn't know what to ask.
  Run a **blindspot pass** before interrogating: teach the domain's shape —
  what good looks like, the standard failure modes, the questions they should
  be asking — then interview. Answers extracted from an untaught operator pin
  the design to guesses.

## Boundaries

Do not manufacture questions for choices the evidence already locks — that is
railroading, not rigor. Do not interrogate what you can read. The stance is the
default posture for contestable framing, not a toll booth on every request.

## Prompt

Before the substantial work begins:

- Load-bearing decisions still implicit:
- Next single question (with your recommended answer):
- What you resolved by exploring instead of asking:
