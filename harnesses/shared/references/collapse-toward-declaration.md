# Collapse Toward Declaration — the hundred-line koan

Two students boast of a million and ten million lines; the master's greatest
program is a hundred, and the students are enlightened. Ratified as fleet
doctrine by the operator 2026-07-06, after a session that demonstrated both
sides of it.

## The principle

Mastery is *refusal*, not compression. Lines are liability. And the modern
twist: **the master's hundred lines might be English** — a role declaration,
a system prompt, a spec. Measured live in this fleet: a twenty-line curator
prompt replaced what the heuristic lineage would have grown into a thousand
brittle rules; a 2,305-byte agent identity found three real production bugs
on its first dispatch; an eight-agent roster of ~40-line YAML declarations
commanded a weekend that shipped sixty cards. Meanwhile a 2,333-line
deterministic report generator produced formatted logs with zero insight —
mass without judgment.

## The four-layer end-state (every component collapses toward this)

1. **Spine** — small, deterministic, boring: policy, persistence, approval,
   sandboxing, gates. Carries a *mechanism budget* (a LOC tripwire raised
   only with a named, dated justification). Nothing that requires judgment
   lives here.
2. **Declarations** — identities (role.yaml), specs, schemas, contracts,
   tiers tables. Deterministic code may branch on declared fields; this is
   the legitimate home of structure.
3. **Model judgment at the seams** — classification, distillation, triage,
   synthesis, review, routing. Options at a semantic seam are a declared
   field or a model call, never keyword heuristics.
4. **Shared renderers** — one presentation layer per artifact class, many
   specs. Hand-rolled HTML in application code is debt to this layer.

**Imperative lines are the residue of judgment not yet moved to layer 2 or
layer 3.**

## Applications

- **Prompts and skills are programs.** A 200-line SKILL.md that could be 30
  is the same sin as bloated code. Point at resources; don't enumerate.
  Unreferenced prose mass is a deletion candidate, not an archive candidate.
- **Briefs shrink as the system matures.** Identities + tickets exist so
  dispatches can be three lines: *claim X; the card is the spec; you are
  \<identity\>*. A growing brief signals a missing declaration — fix the
  layer, not the brief.
- **Identity deltas are declarations, never patches.** Hand-editing a
  vendored agent file is mechanism squatting where an overlay declaration
  belongs.
- **Organ test:** every service states the one question it answers in one
  sentence. An organ that can't is two organs, or zero. Every kill under
  this test (Gazette, Lantern, Hermes, Gradient) was a hundred-lines move.
- **Measure it if contested:** the *koan ratio* — declarative + prompt lines
  over imperative lines — should rise per repo over time; a falling ratio is
  architecture drift toward the students' boast.

## What this is not

Not code golf, not minification, not an excuse to delete tests or gates
(gates are spine — they stay). The hundred lines are *chosen*, and choosing
requires the verification systems that prove the chosen lines work.
