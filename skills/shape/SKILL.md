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

Size the shape before raising the floor. Trivial shapes are mechanical
one-file fixes, typo-level docs, or already-decided packet formatting.
Non-trivial/M+ shapes are everything with product judgment, architecture
choice, user workflow, data model, dependency, privacy, or rollout risk. The
diversity, roster, matrix, and review-bench gates apply to non-trivial/M+
shapes.

### Phase 1: Understand

Accept: raw idea, backlog.d/ item, issue ID, or observation.

## Delegation Floor

When a provider roster is available (repo `.spellbook/agents.yaml` or system `~/.spellbook/agents.yaml`), `/shape` starts by probing the roster
and dispatching two or more available roster members before converging on a
problem frame or solution. Use one lane to map repo constraints and another
for prior-art or premise challenge; add more lanes when product, technical,
or customer-risk perspectives diverge. The lead agent owns synthesis,
questions to the user, final context-packet wording, and receipts. Direct
lead-only shaping is limited to mechanical packet formatting, emergency
unblocks, explicit user-forbidden delegation, or fewer than two available
roster members.
Native in-thread subagents can add fresh-context signal, but they do not
satisfy the roster floor. At least two real provider ids and two distinct
model/provider classes must be dispatched or waived explicitly.

Spawn parallel sub-agents to gather context fast: one to map the relevant
codebase area (files, patterns, constraints), another to search for prior art
(how do other projects solve this? check codebase first, then /research).
For non-trivial shapes, fill the bench instead of running a two-agent
formality: include at least one repo investigator, one product/premise critic,
one architecture critic, one implementation-risk reviewer, and one test/oracle
reviewer when the available harness can support it. Use the lead model for
synthesis; record meaningful external lanes as delegation receipts.
Synthesize their findings before proceeding.

If `exemplars.md` exists at project root, read it. Include relevant exemplar
techniques in the context packet with specific files to study during build.

### Phase 2: Product Exploration

**GATE: Do NOT write code until product direction is locked.**

1. **Investigate** — Problem space, user impact, prior art
2. **Brainstorm** — At least 6 materially different product forms,
   workflows, stacks, tools, or solution families for M+ shapes. Include the
   boring/manual path, the local-first path, the ideal/high-leverage path, and
   one path that inverts a load-bearing assumption. Describe how each fails.
   Candidates are structurally distinct only when they differ on at least one
   major axis: form factor, paradigm, placement, build/buy/rent/host,
   persistence model, dependency footprint, or trust boundary. Cull to 2-3
   finalists on the record; the kill list ships in the packet.
   **Recommend one.**
3. **Discuss** — One question at a time. Iterate until locked.
4. **Draft spec** — Goal, non-goals, acceptance criteria

### Phase 3: Technical Exploration

1. **Explore** — At least 6 technical approaches or system forms for M+
   shapes. These must be structurally different, not cosmetic variants. For
   each: architecture sketch, files to modify, pattern alignment, effort,
   privacy/security posture, reversibility, operability, testability, and
   failure modes.
   **Recommend one.**

2. **Evaluate** — Build a tradeoff matrix before choosing. Score the viable
   options against outcome fit, implementation size, delete-ability, data
   ownership, privacy/security, agent-manageability, failure blast radius,
   migration path, verification cost, and long-term operating burden. Reject
   weak options explicitly; do not let them disappear. If the matrix shows
   fewer than 4 rows with at least 2 distinct values across the core axes, the
   brainstorm has not diverged enough.

3. **Validate** — For effort M or larger, spawn the adversarial design review
   bench in parallel:
   ousterhout reviews for module depth and information hiding, carmack for
   shippability and over-engineering, grug for complexity, beck for executable
   oracles, and at least one roster-backed external provider lane for an
   independent critique when available. Give each the design summary and ask
   for the production failure mode that would embarrass us, the evidence that
   would prove it, and a verdict + concerns. If any has blocking concerns,
   synthesize the evidence and revise the design before proceeding.

4. **Discuss** — No limit on rounds. Design isn't ready until user says so.

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

## Alternatives Considered
| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| <name> | <how it works> | <why it could win> | <how it fails> | <choose/reject/defer> |

Include at least 6 rows for M+ shapes.

## Tradeoff Matrix
| Option | Fit | Size | Privacy | Agent-manageable | Reversible | Testable | Operating Burden |
|---|---:|---:|---:|---:|---:|---:|---:|
| <name> | <1-5> | <1-5> | <1-5> | <1-5> | <1-5> | <1-5> | <1-5> |

Explain the scoring in prose; numbers alone are not evidence.

## Delegation Evidence
- Roster providers used: <provider/model/role>
- Native subagents used: <role/purpose>
- Accepted evidence: <what changed the spec>
- Rejected evidence: <what was discarded and why>
- Waivers: <provider/model/role gaps>

No context packet ships without this synthesis artifact for non-trivial/M+
shapes. The recommendation must cite lane evidence, not only the lead's own
preference.

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
- **Alternatives-in-name-only:** Three "options" that are the same idea in three outfits is one option. Real divergence means structurally distinct approaches — typically one minimal-viable, one ideal, and one that inverts a load-bearing assumption. If you can't articulate how each would fail differently, go back. For M+ effort, the philosophy bench (ousterhout/carmack/grug) is persona diversity, not foundation diversity — also use `/research` and the roster floor for genuinely heterogeneous signal.
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
