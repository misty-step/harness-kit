---
name: dispatch
description: |
  Compose and launch roster-backed specialist lanes with prompt-native lane
  cards and receipts. Use when: "dispatch agents", "use subagents",
  "compose a team", "run provider lanes", "make lane cards".
  Trigger: /dispatch, /subagents, /lanes.
argument-hint: "[objective|lane-card|provider target]"
---

# /dispatch

Global delegation, prompt-native. Native subagents are the default path;
roster providers and sprites earn lanes on judgment, not quota. This skill
keeps the implementation small: lane cards, native harness commands,
receipts, and lead synthesis.

## Route

| Need | Load |
|---|---|
| write or review lane cards | `references/lane-cards.md` |
| copy a lane card | `templates/lane-card.md` |
| track a multi-lane run | `templates/run-card.md` |
| choose providers/models | `/harness-engineering models` |
| run a lane on a remote sandbox | `/sprites` (sprite-lane runner) |

Use the existing `dispatch-agent` helper to launch configured providers. Use
`lane_harness.v1` projection only when context hygiene matters; otherwise a
lane card is just the prompt packet.

## Delegation Judgment

No provider quota. Native subagents by default; cross-model critics for
review of your own work; roster providers for bounded cards where the
provider is better, cheaper, or independent in a nameable way; sprites for
heavy, long-running, parallel, or isolation-needing lanes. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: lane cards are outcome-shaped and big — end state,
success criteria, verification affordances, boundaries, output shape,
receipt expectations. The oracle field is load-bearing; the lane agent owns
its own decomposition. Give each lane scoped context and record the lead's
accepted or rejected evidence.

## Contract

- The lead composes the team; Harness Kit does not rank providers or schedule
  fallback loops.
- A lane card is a natural-language contract, not a new runtime object.
- A provider receipt is evidence, not authority.
- `lane_harness.v1` is an optional projection boundary for narrow lanes that
  should not inherit the full global skill catalog.
- Failed providers produce typed evidence and return to the lead; replacement
  is explicit lead judgment.

## Workflow

1. **Probe.** Run the repo roster probe or equivalent smoke path.
2. **Plan lanes.** Pick independent lanes: builder, critic, verifier, QA
   driver, performance critic, persona tester, product synthesizer, or a
   task-specific role.
3. **Write lane cards.** Use `templates/lane-card.md`. Keep each card small:
   one role, one objective, one boundary, one output shape.
4. **Project only when needed.** Add `--lane-harness <manifest>` when the lane
   needs a focused visible skill set.
5. **Dispatch in parallel.** Launch independent providers with
   `dispatch-agent`; do not wait serially unless a lane depends on another.
6. **Synthesize.** Accept, reject, or partially accept outputs. Name failures
   and waivers. The lead owns the final decision.

## Output

```markdown
## Dispatch Summary
- Objective:
- Providers used:
- Lane shape:
- Accepted evidence:
- Rejected evidence:
- Failures / waivers:
- Receipt ids:
- Residual risk:
```

## Gotchas

- Do not create static subagent files for roles that a lane card can express.
- Do not generate or mutate global skill installs during a run.
- Do not count probes, auth failures, or wrapper failures as successful lanes.
- Do not add lanes for ceremony; each lane needs a distinct question, risk,
  artifact, or model/harness property. One good native subagent beats two
  ritual provider lanes.
- Do not use projection as a permission system; it is context hygiene.
