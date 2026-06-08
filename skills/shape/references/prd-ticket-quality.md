# PRD Ticket Quality

Use this reference when shaping M+ backlog items or any ticket that will feed
`/deliver`. A world-class ticket is a compact PRD plus technical design. It is
not a diary, a brainstorm dump, or an implementation transcript.

## Required Top Blocks

Every M+ shaped ticket needs these blocks near the top, before repo anchors and
oracles:

```markdown
## PRD Summary
- User: the person or operator whose workflow changes.
- Problem: the painful condition or opportunity, not the requested mechanism.
- Goal: one sentence naming the concrete outcome, not the tool to use.
- Why now: what makes this worth doing before nearby work.
- UX enabled: what the user can see, do, decide, trust, or avoid after this ships.
- Deliverable type: working code, research report, docs artifact, harness primitive, cleanup, migration, or decision memo.
- Success signal: the first observable proof that the deliverable worked.

## Product Requirements
- P0: non-negotiable user outcomes and constraints.
- P1: useful follow-ons inside the current slice.
- Non-goals: attractive scope that must stay out.

## Technical Design
- Chosen architecture: the concrete system shape.
- Files/systems touched: bounded surfaces and ownership.
- Data/control flow: how the behavior moves through the system.
- Build/check boundary: what fails during build, what fails during verification.
- ADR decision: required / not required, with reason and escalation trigger.
- ADR-style invariants: conditions that must remain true, consequence if
  violated, and core file references.
- Design X vs Y: the main alternatives, explicit verdicts, and failure modes.

## Lead Repo Read
- Source files, ADRs, tests, commands, or rendered artifacts the lead read
  directly before implementation.
- Subagent summaries used only for search, critique, or coverage; never as the
  only evidence for code the builder must understand.

## Alignment Questions
- At most five unresolved product or architecture questions, each with the
  recommended answer, supporting evidence, and risk if wrong.
- Use `none; assumptions accepted` only when live repo evidence and the user
  request already lock the choice.

## Deliverable
- Output: exact thing to leave behind.
- Acceptance oracle: executable command, rendered artifact, report shape, or decision record.
- Evidence artifacts: receipts, screenshots, fixtures, hashes, traces, or links.
- Residual risk: what remains unproven and who must accept it.
```

## Writing Rules

- Keep `Goal` to one outcome sentence with a well-defined exit condition. If it
  starts with "explore", "consider", or "maybe" while `Status: ready`, rewrite
  it or set the deliverable type to a research/decision report.
- Put the deliverable type in the first screen of the ticket. Do not make the
  implementer infer whether the output is code, research, a report, or a
  doctrine decision.
- Name the user. "Agents" can be a user only when the behavior is directly for
  agent operation; otherwise name the human operator, reviewer, maintainer, or
  reader.
- Alternatives must be structurally different. Cosmetic variants do not count.
- Every alternative needs a verdict. Undecided alternatives are open product
  work, not background context.
- Acceptance must have a pass/fail surface. If no executable command exists,
  define the report/artifact shape and the reviewer action that accepts it.
- Architecture decisions must state the selected boundary and the rejected
  boundary. "Use existing patterns" is not enough.
- ADR decisions must include an escalation trigger. "Not required" is valid only
  when the slice stays inside an existing architecture.
- Runtime-visible changes need a dogfood artifact or an explicit no-running-
  surface waiver in the evidence plan.

## Failure Modes

- **Hidden user:** the ticket describes a system change but never says who gets
  a better workflow.
- **Buried deliverable:** the actual output appears only in an implementation
  sequence or oracle.
- **Mechanism goal:** the goal names the tool or architecture instead of the
  user outcome.
- **Ready-but-vague:** `Status: ready` with "preferably", "confirm later", or
  an unspecified first target.
- **Architecture fog:** fields are listed, but schema shape, ownership, or
  build/check responsibilities are not decided.
- **ADR theater:** an ADR is required for every small choice, or never required
  for cross-cutting policy.
- **Subagent-only understanding:** the packet cites summaries but does not name
  the source files, ADRs, tests, commands, or rendered artifacts the lead read.
- **Checkbox oracle:** acceptance is a prose list without commands, artifacts,
  hashes, screenshots, or a named reviewer action.
- **Transcript bulk:** delegation history overwhelms the product and technical
  decision. Summarize accepted/rejected evidence instead.
