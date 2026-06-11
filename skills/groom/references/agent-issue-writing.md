# Agent-Ready Issue Writing

## Principle

Treat the issue as a prompt for a senior coding agent.
Give the agent the goal, the local context, the quality bar, and the boundaries.
Do not replace thinking with a brittle step-by-step script.

## What to include

### Problem

State what is wrong or missing, with concrete evidence when available.

### Outcome

State what should be true when the issue is done.

### Context

Include only the context needed to make good decisions:
- user workflow
- architecture seam
- existing patterns
- relevant linked issues or docs

Use domain names and behavioral contracts before file paths. Paths are
touchpoints, not the source of truth; they go stale faster than the behavior.

### Acceptance criteria

Write them so they can map to tests, commands, or visible behaviors.
Good tags:
- `[behavioral]`
- `[test]`
- `[command]`

### Boundaries

Say what should not change. This is often more valuable than extra implementation steps.

### Verification

Provide runnable commands when possible.

### Touchpoints

List likely files, modules, routes, tests, or data paths when known. These are starting points, not
prisons.

### Agent Autonomy

Label the expected execution mode when it matters:

- **AFK:** the issue is fully specified, has an executable or inspectable
  oracle, and can be completed by an agent without new product judgment.
- **HITL:** the issue needs a human decision, credential, design review,
  production access, or external confirmation before implementation can safely
  finish.

Do not mark an issue AFK unless dependencies, boundaries, and verification are
visible in the issue body.

## What to avoid

- vague requests like “clean this up”
- giant multi-outcome tickets (an epic with one coherent outcome, an oracle,
  and ordered children is fine; a grab-bag without done criteria is not)
- hidden dependencies in comments only
- instructions that describe exact shell steps instead of the desired result
- “etc”, “as needed”, or other scope leak phrases
- acceptance criteria that cannot be observed or tested
- stale line-number directives as the only source of truth
- AFK labels on work that still needs product or architecture decisions

## Type-specific guidance

### Bug

Include:
- repro
- expected vs actual
- impact
- regression clues if known

### Feature

Include:
- user or operator value
- triggering surface
- rendering or API constraints
- deterministic facts the model must not invent

### Refactor

Include:
- invariants to preserve
- code smells or coupling to remove
- tests or checks that must stay green

### Research

Include:
- the decision to make
- what evidence to gather
- the expected output artifact
- what counts as sufficient confidence

## Recommended issue skeleton

```md
## Problem

## Outcome

## Context

## Acceptance Criteria
- [behavioral] Given ...
- [test] Given ...
- [command] When `...`, then ...

## Touchpoints
- `path/to/file`

## Verification
```bash
pnpm test ...
pnpm typecheck
```

## Boundaries

## Related
```

## AI-agent modification

Optimize for first-pass execution:
- prefer one issue per coherent diff
- split broad plans into vertical tracer bullets that are independently
  demoable or verifiable
- keep prompts goal-oriented, not step-prescriptive
- include deterministic constraints explicitly
- separate exploration from implementation when uncertainty is high
- rewrite oversized issues before assigning them

## Sources

- https://developers.openai.com/api/docs/guides/prompt-engineering
- https://developers.openai.com/api/docs/guides/function-calling
- https://docs.anthropic.com/en/docs/build-with-claude/prompt-engineering/overview
- https://docs.github.com/en/copilot/how-tos/agents/copilot-coding-agent/troubleshoot-copilot-coding-agent
