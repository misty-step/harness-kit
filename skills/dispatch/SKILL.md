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

Global delegation, prompt-native. The roster is mandatory for substantive
work; this skill keeps the implementation small: lane cards, native harness
commands, receipts, and lead synthesis.

## Route

| Need | Load |
|---|---|
| write or review lane cards | `references/lane-cards.md` |
| copy a lane card | `templates/lane-card.md` |
| track a multi-lane run | `templates/run-card.md` |
| choose providers/models | `/harness-engineering models` |

Use the existing `dispatch-agent` helper to launch configured providers. Use
`lane_harness.v1` projection only when context hygiene matters; otherwise a
lane card is just the prompt packet.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use prompt-native lane cards with role, objective, scope,
inputs/oracle, allowed skills/tools, output shape, boundaries, and receipt
expectations. Give each lane scoped context and record the lead's accepted or
rejected evidence.

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
- Do not let "two providers" become ceremony; each lane needs a distinct
  question, risk, artifact, or model/harness property.
- Do not use projection as a permission system; it is context hygiene.
