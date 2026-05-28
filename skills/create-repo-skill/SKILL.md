---
name: create-repo-skill
description: |
  Generate a repository-local skill from live repo discovery and user intent.
  Use when: "create QA skill", "generate repo skill", "make a local skill",
  "persona acceptance skill", "value proposition QA", "bespoke repo QA",
  "scaffold local skill". Creates concrete `.agents/skills/<name>/` guidance
  with harness-specific bridges when useful. Trigger: /create-repo-skill,
  /repo-skill, /create-qa-skill.
argument-hint: "[qa|persona-acceptance|<skill-name>] [target]"
---

# /create-repo-skill

Generate a local skill for this repo, not Harness Kit globally. This is a thin
skill authoring lane, not a persona runtime or scheduling engine.

## Route

| Need | Load |
|---|---|
| repo-specific QA skill | `references/qa.md` |
| persona/value-proposition acceptance skill | `references/persona-acceptance.md` |
| skill-writing rules | `references/authoring.md` |

## Contract

- Discover the live repo before drafting.
- Ask the user for missing product truth: value proposition, target users,
  critical workflows, production/local target, and allowed side effects.
- Dispatch independent lanes: repo mapper, product/persona mapper, critic.
- Write only repo-local skill artifacts in the target repo.
- Prefer `.agents/skills/<name>/` as shared root; bridge `.claude/skills/`,
  `.codex/skills/`, and `.pi/skills/` when those dirs exist.
- Do not shadow a first-party global skill unless the local repo explicitly
  needs the same command name, such as `.agents/skills/qa/`.
- Include at least one eval seed or smoke oracle in the generated skill.
- Include an acceptance block with live repo evidence, exact command/path,
  repo-fit check, and residual risk.
- Keep generated `SKILL.md` under 300 lines unless the repo proves otherwise.

## Delegation Floor

When a provider roster is available (repo `.harness-kit/agents.yaml` or system
`~/.harness-kit/agents.yaml`), `/create-repo-skill` starts by probing the roster
and dispatching two or more available providers. Use split lanes: one maps repo
truth, one attacks the generated skill for generic wording, missing oracles, and
workflow mismatch. Give each lane scoped files, expected output, and boundaries.
The lead owns synthesis, writes, final verification, and receipts.
Direct lead-only work is limited to mechanical edits, emergency state
preservation, explicit user-forbidden delegation, or fewer than two available
providers.

Native in-thread subagents may supplement the investigation, but they do not
satisfy the roster floor. Count only configured provider ids in receipts and
final roster reports.

## Output

Generated local skill includes:

- `SKILL.md` with trigger phrases, repo commands, live surfaces, output format.
- `references/` only for material too large for the entry file.
- `evals/README.md`, `evals/cases/<case>.md`, and a small grader or rubric.
- A closeout note naming what the lead accepted, rejected, and left risky.
- A generated-skill quality gate: fail on placeholders, guessed commands,
  missing live surfaces, or no eval seed.

## Gotchas

- Generic local skills are worse than no local skill.
- Personas are not job titles. They need a goal, context, constraint, and task.
- Value propositions are claims to test, not marketing copy to repeat.
- Browser automation is a tool choice after workflow mapping, not the default.
- Do not generate a semantic workflow engine. Generate instructions and oracles.
