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

Shape a raw idea into something buildable. Output is a **context packet** —
the unit of specification that precedes implementation.

## Workflow

### Phase 1: Understand

Accept: raw idea, backlog.d/ item, issue ID, or observation.

Spawn parallel sub-agents to gather context fast: one to map the relevant
codebase area (files, patterns, constraints), another to search for prior art
(how do other projects solve this? check codebase first, then /research).
Synthesize their findings before proceeding.

If `.spellbook/agents.yaml` exists and the shape is M+ effort, consult the
provider roster for a heterogeneous second voice before converging. Use the
lead model for synthesis; record meaningful external lanes as delegation
receipts.

If `exemplars.md` exists at project root, read it. Include relevant exemplar
techniques in the context packet with specific files to study during build.

### Phase 2: Product Exploration

**GATE: Do NOT write code until product direction is locked.**

1. **Investigate** — Problem space, user impact, prior art
2. **Brainstorm** — 2-3 approaches with tradeoffs. **Recommend one.**
3. **Discuss** — One question at a time. Iterate until locked.
4. **Draft spec** — Goal, non-goals, acceptance criteria

### Phase 3: Technical Exploration

1. **Explore** — 3-5 technical approaches. For each:
   architecture sketch, files to modify, pattern alignment, effort, tradeoffs.
   **Recommend one.**

2. **Validate** — For effort M or larger, spawn the design review bench in parallel:
   ousterhout reviews for module depth and information hiding, carmack for
   shippability and over-engineering, grug for complexity. Give each the design
   summary and ask for a verdict + concerns. If any has blocking concerns,
   revise the design before proceeding.

3. **Discuss** — No limit on rounds. Design isn't ready until user says so.

### Phase 4: Context Packet

The output of shape. This is what `/deliver` and `/implement` consume.

```markdown
# Context Packet: <title>

## Goal
<1 sentence — what outcome, not mechanism>

## Non-Goals
- <what NOT to do, even if it seems like a good idea>

## Constraints / Invariants
- <things that must remain true before, during, and after>

## Authority Order
tests > type system > code > docs > lore

## Repo Anchors
- `src/auth/middleware.ts` — current pattern to follow
- `tests/auth/` — existing coverage

## Prior Art
- `src/payments/middleware.ts` — similar pattern

## Exemplar Techniques
- <technique from exemplars.md> — <specific file to study during build>

## Oracle (Definition of Done)
- [ ] All existing auth tests pass
- [ ] New endpoint returns 200 with valid token
- [ ] Response time < 100ms p99

## Implementation Sequence
1. <first chunk>
2. <second chunk>

## Risk + Rollout
- <how it could fail, how to undo it>
```

If you can't write an oracle, the goal isn't clear enough. Go back to Phase 2.

## Gotchas

- **Premise unchallenged:** A shape request accepts the stated framing by default. Before Phase 2, five-whys the goal. If the request says "feature X," name the underlying user outcome — the best path to it may not be X. A solid shape of the wrong problem is the failure mode this skill exists to prevent, not cause.
- **Alternatives-in-name-only:** Three "options" that are the same idea in three outfits is one option. Real divergence means structurally distinct approaches — typically one minimal-viable, one ideal, and one that inverts a load-bearing assumption. If you can't articulate how each would fail differently, go back. For M+ effort, the philosophy bench (ousterhout/carmack/grug) is persona diversity, not foundation diversity — also invoke `/research` or consult the provider roster for genuinely heterogeneous signal.
- **Vague oracles:** "It should work" is not an oracle. "These 3 tests pass and this endpoint returns 200" is. See `references/executable-oracles.md`.
- **Checkbox oracles:** Prose checklists drift. Write oracles as commands that return pass/fail, not prose that requires interpretation.
- **Speccing after building:** A context packet written after implementation is documentation, not specification. Spec first.
- **50 repo anchors:** If everything is an anchor, nothing is. Pick 3-10 files whose patterns MUST be followed.
- **Skipping non-goals:** Agents drift toward scope expansion. Non-goals are load-bearing constraints. Write them.
- **Over-speccing implementation details:** Specify WHAT and WHY. Let the builder figure out HOW. Detailed pseudocode cascades errors.
- **Editing shape docs without ripple check:** Files with `shaping: true` frontmatter are live specs. Before editing, check: do affordance tables need updating? Does this change ripple to other work streams or context packets? Edit the doc, then trace the consequences.

## Principles

- Minimize touch points (fewer files = less risk)
- Design for deletion (easy to remove later)
- Favor existing patterns over novel ones
- YAGNI ruthlessly
- Recommend, don't just list options
- One question at a time
