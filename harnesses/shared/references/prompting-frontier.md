# Prompting Fable-class models

Adapted for this harness from Matt Shumer's "How I Prompt Fable" (2026-07-03),
operator-ratified the same day. Applies to Fable/GPT-5.5-class lanes and to
the lead's own operation. Older/cheaper models still need more mechanism —
tier the prescription to the model.

## The seven rules

1. **Goal, not steps.** Hand big, underspecified outcomes. Every step you
   dictate overrides the model's judgment with yours, and on frontier models
   yours is usually worse. A brief that reads like a recipe is a smell —
   unless a step encodes an operator RULING, in which case it is a house
   rule, not a step (label it so).
2. **House rules, not scripts.** Fence the open goal with the few invariants
   that must survive any path: repo red lines, ratified design verdicts,
   security boundaries, "describe behavior in the agent's prompt instead of
   hard-coding special cases." House rules live in AGENTS.md / VISION.md /
   the lane card's constraints block — short, absolute, checkable.
3. **A bar, not adjectives.** "High quality" stops at the model's own idea of
   good enough. Give an executable bar (see verification-system-first.md) —
   and when you can't define the measuring stick, DELEGATE ITS INVENTION:
   "figure out how to measure X, then hit it" is a legal, often superior
   oracle assignment.
4. **The builder never grades its own work.** A build lane carries a
   trajectory of self-justification. Verification is a FRESH context pointed
   at the real output — rendered pixels, the running app, the live route,
   never the diff alone — commissioned to PROVE THE WORK FAILS the bar and
   the house rules before anything ships. (Extends fresh-context-critique:
   the critic's target is the artifact-in-reality, not the artifact-in-repo.)
5. **Loop until the bar, never until satisfied.** For creative or
   quality-chasing work: build → fresh-check → name the biggest gap → close
   it → again. The model never declares itself finished; the loop ends at
   the bar or by operator call. Long runs keep a LIVE STATUS ARTIFACT the
   operator can glance from a phone (in this shop: post to the Bridge feed
   and/or a dedicated status page — screenshots, current gap, next move).
6. **Old work is fuel.** Point lanes at prior artifacts as the quality bar
   ("match this, then beat it") and at prior SESSION TRACES as technique
   ("read what the forest build tried; learn what worked"). In this shop the
   traces are searchable: QMD collections cover Claude/Codex session
   history; `.harness-kit/traces/` and `~/.factory-lanes/wave*/` hold lane
   receipts. Re-explaining a solved problem to a fresh lane is waste.
7. **Clear the road up front.** Budgets instead of permission-asks; key
   LOCATIONS documented (never values); "make your own calls, return only
   when truly blocked or facing an operator-only decision" written into the
   brief. The one exception: for huge, hard-to-reverse foundations, demand
   the plan first and surface every uncertainty as upfront questions — then
   run without stopping.

## Two formations

- **Engineering:** several sessions pulling tasks in parallel, each
  triple-checked by its own sub-agents, PRs carrying evidence; ONE
  integrator session that only merges, runs everything end-to-end like a
  real user, and keeps the tree green. Overlapping features: one lane reads
  the other's traces as it builds and integrates as it lands.
- **Creative:** same loop and bar, but fan sub-agents out per piece
  (one perfecting each tree in the forest), and/or run independent parallel
  attempts, keep the best, carry what worked into the next round.

## Ultracode

Reserve for FOUNDATIONS: a new system you'll build on for months, where the
base being right compounds forever. A good loop with an ambitious bar covers
nearly everything else. (Also the ShadCN lesson: when cloning/matching
something, existing scaffolding you're fighting is baggage — starting from
nothing is often the better foundation.)

## Anti-patterns this file exists to kill

- Recipe-briefs to frontier lanes (mechanism dictation without a ruling
  behind it).
- Adjective oracles ("polished", "world-class") with no measuring stick.
- Builder-graded "done" — a lane's own checklist as acceptance evidence
  (see the self-graded-oracle-inflation incident, 2026-07-03).
- Long silent runs — no status artifact, results living in scrollback
  (see the memory-bakeoff loss, 2026-07-03).
- Cold lanes re-deriving what a prior trace already learned.
