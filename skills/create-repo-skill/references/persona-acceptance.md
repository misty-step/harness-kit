# Persona Acceptance Skill Generator

Generate a local acceptance skill that tests fuzzy product promises against live
behavior. Do not split this into a standalone persona runtime; the generated
skill is a repo-local QA/acceptance primitive with persona lanes.

Use when unit/E2E tests miss the question: "Can the target customer do the job
they came here to do?"

## Conversation

Derive and confirm:

- value proposition: the promise the product makes;
- target market: who buys or adopts it;
- personas: who uses it, with goal, context, constraint, and tolerance;
- workflows: jobs they must complete;
- target surface: local app, preview, staging, production, CLI, API, or docs;
- side effects: allowed writes, forbidden actions, safe tenant/account;
- report card scale: pass/fail, scored rubric, or qualitative findings.
- completion gate: exact behavior, live evidence, command/path, repo fit,
  residual risk.
- persona outcome: observable completion, blocked action, exceeded expectation,
  or friction.

## Lanes

Dispatch in parallel:

| Lane | Objective | Output |
|---|---|---|
| product mapper | extract claims, workflows, docs promises, onboarding copy | claim map |
| repo/app mapper | find runnable surfaces, commands, routes, auth boundaries | surface map |
| persona critic | challenge personas for generic roles or untestable goals | blockers |
| probe designer | draft live-app charters and evidence plan | charters |

## Generated Skill Contents

The local skill should:

- launch persona lanes with clear charters;
- drive the live product, not read screens in isolation;
- capture evidence per persona;
- compare actual behavior to the persona goal and value claim;
- produce a report card.

Report card:

```markdown
# Persona Acceptance Report

| Persona | Goal | Completed? | Friction | Evidence | Severity |
|---|---|---|---|---|---|

## Expectations
- Met:
- Unmet:
- Exceeded:

## Product Gaps
- P0:
- P1:
- P2:

## Follow-up Oracles
- Candidate unit/E2E tests:
- Candidate docs/product changes:
- Residual risk:
```

## Eval Seed

Case prompt: "Run persona acceptance for [persona] trying to [workflow]."

Grader passes when output includes:

- persona goal and constraint;
- live target exercised;
- evidence path;
- expectation met/unmet/exceeded judgment;
- friction and severity;
- follow-up oracle candidates.
- no placeholders or invented routes.
