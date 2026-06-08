# Lane Cards

Lane cards are the prompt-native form of dynamic subagents. They keep global
delegation strong without turning Harness Kit into a scheduler.

## Required Fields

- Role: specialist identity for this lane.
- Objective: one-sentence outcome.
- Scope: files, commands, sources, or boundaries.
- Inputs / oracle: what the lane receives and how success is judged.
- Allowed skills: skills the lane should use.
- Allowed tools: tools the lane should use.
- Output shape: exact format and length.
- Do not touch: explicit non-goals.
- Receipt expectation: what the lead must record.
- Lane harness: optional `lane_harness.v1` manifest path, or `none`.

## When To Use Projection

Use `lane_harness.v1` when excess skills would mislead the lane:

- CI-only critic that should not shape or groom.
- Docs verifier that should not edit code.
- Review lane that should see only `code-review` and `critique`.
- QA lane that should see only app-driving and evidence-capture skills.

Skip projection when the provider already has a narrow prompt, the work is
small, or the lane needs broad repo context.

## Common Lanes

```markdown
Role: implementation critic
Objective: Find blockers in the diff before merge.
Scope: diff + acceptance oracle only.
Inputs / oracle: `git diff <base>...HEAD`; ticket acceptance checks.
Allowed skills/tools: code-review, critique; read, grep, git diff.
Output shape: <=600 words: blocking findings, evidence, verdict.
Do not touch: no edits, no broad repo audit, no author reasoning.
Receipt expectation: record accepted/rejected findings and receipt id.
Lane harness: none, or `.harness-kit/examples/lane-harness.yaml`
```

```markdown
Role: persona QA tester
Objective: Exercise the changed workflow as the named persona.
Scope: running app route or CLI command chosen by `/qa`.
Inputs / oracle: app URL/command, persona goal, success criteria.
Allowed skills/tools: qa, browser; browser/shell evidence capture.
Output shape: status, path exercised, artifact refs, product friction.
Do not touch: no fixes, no backlog edits.
Receipt expectation: link screenshot/transcript and persona outcome.
Lane harness: none
```

```markdown
Role: product-owner synthesizer
Objective: Turn persona QA reports into backlog proposals.
Scope: persona reports only.
Inputs / oracle: QA artifacts and product goal.
Allowed skills/tools: groom, shape.
Output shape: backlog candidates with evidence and rejected ideas.
Do not touch: no code changes, no commits.
Receipt expectation: accepted proposals and discarded reports.
Lane harness: none
```

## Lead Checklist

- Lanes are independent or explicitly ordered.
- At least two non-manual providers are used for substantive work, or a waiver
  names why not.
- Each lane has a different risk, method, source, or model/harness property.
- Provider failures are classified and not counted as successful evidence.
- The final answer names providers, shape, accepted/rejected outputs, failures,
  waivers, receipt ids, and residual risk.
